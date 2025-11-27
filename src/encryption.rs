use crate::{
    e,
    file::update_file,
    globals::{g_get_derivation, g_get_password, g_set_derivation, g_set_password},
    input::{ask_string, get_input},
    notes::get_notes,
    other::exit,
    p, s,
};

use sodiumoxide::crypto::aead;
use sodiumoxide::crypto::aead::chacha20poly1305_ietf as aead_v2;
use sodiumoxide::crypto::pwhash;

// Lets the user change password or derivation
pub fn change_security() {
    p!("(1) Change Password");
    p!("(2) Change Key Derivation");
    let ans = ask_string("Choice", "", true);
    let mut save = true;

    match &ans[..] {
        "1" => change_password(),
        "2" => change_key_derivation(),
        _ => save = false,
    };

    if save {
        update_file(get_notes(false))
    }
}

// Change key derivation method
pub fn change_key_derivation() {
    let og_deriv = g_get_derivation();
    p!("This will change the file's key derivation method");
    get_key_derivation();

    if og_deriv == 0 && g_get_derivation() != 0 {
        change_password();
    }
}

// Lets the user decide between fast or secure derivation
pub fn get_key_derivation() {
    loop {
        p!("Key Derivation:");
        p!("1) Interactive (Faster)");
        p!("2) Sensitive (More Secure)");
        p!("0) Plain (No Encryption)");
        let derivation = ask_string("Choice", "", true);

        if derivation == "1" || derivation == "2" || derivation == "0" {
            g_set_derivation(derivation.parse::<usize>().unwrap());
            break;
        }
    }
}

// Changes the password and updates the file with it
pub fn change_password() {
    p!("This will change the file's password.");

    if !get_password(true).is_empty() {
        update_file(get_notes(false))
    };
}

// Gets the file's password saved globally
// Or asks the user for the file's password
// Or changes the file's password
pub fn get_password(change: bool) -> String {
    if g_get_derivation() == 0 {
        return s!("none");
    }

    let mut pw = g_get_password();

    if pw.is_empty() || change {
        let mut password: String;

        if change {
            loop {
                password = get_input("New Password", "", true);

                if password.is_empty() {
                    return s!();
                }

                let confirmation = get_input("Confirm Password", "", true);

                if password != confirmation {
                    e!("Error: Passwords Don't Match.")
                } else {
                    break;
                }
            }
        } else {
            password = get_input("Password", "", true);
        }

        pw = s!(password);
        g_set_password(password);
    }

    pw
}

// Get a key from the password
fn key_from_pw(password: &str, salt: pwhash::Salt, derivation: u8) -> Result<aead::Key, ()> {
    let mut key = aead::Key([0; aead::KEYBYTES]);

    match derivation {
        0 => return Ok(key),
        1 => pwhash::derive_key_interactive(&mut key.0, password.as_bytes(), &salt)?,
        2 => pwhash::derive_key_sensitive(&mut key.0, password.as_bytes(), &salt)?,
        _ => {
            e!("Wrong key derivation.");
            exit()
        }
    };

    Ok(key)
}

// Enum for nonce types (v2 uses 12-byte, v3+ uses 24-byte)
enum NonceType {
    V2(aead_v2::Nonce),
    V3(aead::Nonce),
}

impl NonceType {
    fn as_bytes(&self) -> &[u8] {
        match self {
            NonceType::V2(n) => &n.0,
            NonceType::V3(n) => &n.0,
        }
    }
}

// Struct for encrypted data
struct EncryptedData {
    version: u8,
    derivation: u8,
    salt: pwhash::Salt,
    nonce: NonceType,
    ciphertext: Vec<u8>,
}

// Implement the encryption struct
impl EncryptedData {
    fn new(plaintext: &str, password: &str) -> Result<Self, ()> {
        let version = 3;
        let derivation = g_get_derivation() as u8;
        let salt = pwhash::gen_salt();
        let nonce = aead::gen_nonce();
        let key = key_from_pw(password, salt, derivation)?;

        let ciphertext = if derivation == 0 {
            plaintext.as_bytes().to_vec()
        } else if Self::use_authenticated_header(version) {
            let header_ad = Self::associated_data(version, derivation, &salt, &nonce);

            aead::seal(
                plaintext.as_bytes(),
                Some(header_ad.as_slice()),
                &nonce,
                &key,
            )
        } else {
            aead::seal(plaintext.as_bytes(), Some(&salt.0), &nonce, &key)
        };

        Ok(EncryptedData {
            version,
            derivation,
            salt,
            nonce: NonceType::V3(nonce),
            ciphertext,
        })
    }

    fn decrypt(&self, password: &str) -> Result<Vec<u8>, ()> {
        if self.derivation == 0 {
            Ok(self.ciphertext.clone())
        } else {
            let key = key_from_pw(password, self.salt, self.derivation)?;

            match &self.nonce {
                NonceType::V2(nonce) => {
                    let key_v2 = aead_v2::Key(key.0);
                    aead_v2::open(&self.ciphertext, Some(&self.salt.0), nonce, &key_v2)
                }
                NonceType::V3(nonce) => {
                    if Self::use_authenticated_header(self.version) {
                        let header_ad =
                            Self::associated_data(self.version, self.derivation, &self.salt, nonce);
                        aead::open(&self.ciphertext, Some(header_ad.as_slice()), nonce, &key)
                    } else {
                        aead::open(&self.ciphertext, Some(&self.salt.0), nonce, &key)
                    }
                }
            }
        }
    }

    fn use_authenticated_header(version: u8) -> bool {
        version >= 3
    }

    fn associated_data(
        version: u8,
        derivation: u8,
        salt: &pwhash::Salt,
        nonce: &aead::Nonce,
    ) -> Vec<u8> {
        let mut data = vec![version, derivation];
        data.extend_from_slice(&salt.0);
        data.extend_from_slice(&nonce.0);
        data
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![self.version, self.derivation];
        bytes.extend(self.salt.0.iter());
        bytes.extend(self.nonce.as_bytes().iter());
        bytes.extend(self.ciphertext.iter());
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, base64::DecodeError> {
        let n = 2;
        let sbytes = pwhash::SALTBYTES;
        let version = bytes[0];
        let derivation = bytes[1];

        // v2 used chacha20poly1305_ietf (12-byte nonce)
        // v3+ uses xchacha20poly1305_ietf (24-byte nonce)
        let (nonce, nbytes) = if version < 3 {
            let nbytes = aead_v2::NONCEBYTES;

            if bytes.len() < (n + sbytes + nbytes) {
                return Err(base64::DecodeError::InvalidLength);
            }

            let nonce =
                aead_v2::Nonce::from_slice(&bytes[(n + sbytes)..(n + sbytes + nbytes)]).unwrap();
            (NonceType::V2(nonce), nbytes)
        } else {
            let nbytes = aead::NONCEBYTES;

            if bytes.len() < (n + sbytes + nbytes) {
                return Err(base64::DecodeError::InvalidLength);
            }

            let nonce =
                aead::Nonce::from_slice(&bytes[(n + sbytes)..(n + sbytes + nbytes)]).unwrap();
            (NonceType::V3(nonce), nbytes)
        };

        let ciphertext: Vec<u8> = bytes[(n + sbytes + nbytes)..]
            .iter()
            .map(|x| x.to_owned())
            .collect();

        let salt = pwhash::Salt::from_slice(&bytes[n..(n + sbytes)]).unwrap();

        Ok(EncryptedData {
            version,
            derivation,
            salt,
            nonce,
            ciphertext,
        })
    }
}

// Encrypt the notes text
pub fn encrypt_text(plaintext: &str) -> Vec<u8> {
    let password = get_password(false);
    let ciphertext = EncryptedData::new(plaintext, &password).unwrap();
    ciphertext.to_bytes()
}

// Decrypt the file's bytes
pub fn decrypt_bytes(bytes: &[u8]) -> String {
    let ciphertext = EncryptedData::from_bytes(bytes).unwrap();
    g_set_derivation(ciphertext.derivation as usize);
    let password = get_password(false);

    let plaintext = match ciphertext.decrypt(&password) {
        Ok(text) => text,
        Err(_) => {
            e!("Can't decrypt the file.");
            exit()
        }
    };

    String::from_utf8(plaintext).unwrap()
}
