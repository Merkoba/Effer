use crate::{
    e,
    file::update_file,
    globals::{g_get_derivation, g_get_password, g_set_derivation, g_set_password},
    input::{ask_string, get_input},
    notes::get_notes,
    other::exit,
    p, s,
};

use argon2::{Algorithm, Argon2, Params, Version};

use chacha20poly1305::{
    aead::{Aead, Payload},
    ChaCha20Poly1305, KeyInit, XChaCha20Poly1305,
};

use rand::{rngs::OsRng, RngCore};

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
    }
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

// Get a key from the password for V4
fn key_from_pw(password: &str, salt: &[u8], derivation: u8) -> Result<[u8; 32], ()> {
    let mut key = [0u8; 32];
    match derivation {
        0 => {
            return Ok(key);
        }
        1 => {
            let params = Params::new(65536, 2, 1, Some(32)).map_err(|_| ())?;
            let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
            argon2
                .hash_password_into(password.as_bytes(), salt, &mut key)
                .map_err(|_| ())?;
        }
        2 => {
            let params = Params::new(1048576, 4, 1, Some(32)).map_err(|_| ())?;
            let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
            argon2
                .hash_password_into(password.as_bytes(), salt, &mut key)
                .map_err(|_| ())?;
        }
        _ => {
            e!("Wrong key derivation.");
            exit()
        }
    };

    Ok(key)
}

// Fallback logic for V2 and V3 using sodiumoxide
fn legacy_decrypt(
    version: u8,
    derivation: u8,
    salt_bytes: &[u8],
    nonce_bytes: &[u8],
    ciphertext: &[u8],
    password: &str,
) -> Result<Vec<u8>, ()> {
    use sodiumoxide::crypto::aead;
    use sodiumoxide::crypto::aead::chacha20poly1305_ietf as aead_v2;
    use sodiumoxide::crypto::pwhash;
    let salt = pwhash::Salt::from_slice(salt_bytes).ok_or(())?;
    let mut key = aead::Key([0; aead::KEYBYTES]);

    match derivation {
        0 => return Ok(ciphertext.to_vec()),
        1 => {
            pwhash::derive_key_interactive(&mut key.0, password.as_bytes(), &salt)
                .map_err(|_| ())?;
        }
        2 => {
            pwhash::derive_key_sensitive(&mut key.0, password.as_bytes(), &salt)
                .map_err(|_| ())?;
        }
        _ => return Err(()),
    }

    if version < 3 {
        let key_v2 = aead_v2::Key(key.0);
        let nonce = aead_v2::Nonce::from_slice(nonce_bytes).ok_or(())?;
        aead_v2::open(ciphertext, Some(&salt.0), &nonce, &key_v2).map_err(|_| ())
    } else {
        let nonce = aead::Nonce::from_slice(nonce_bytes).ok_or(())?;
        let mut header_ad = vec![version, derivation];
        header_ad.extend_from_slice(&salt.0);
        header_ad.extend_from_slice(&nonce.0);

        aead::open(ciphertext, Some(header_ad.as_slice()), &nonce, &key).map_err(|_| ())
    }
}

// Enum for nonce types (v2 uses 12-byte, v3+ uses 24-byte)
enum NonceType {
    V2([u8; 12]),
    V3([u8; 24]),
}

impl NonceType {
    fn as_bytes(&self) -> &[u8] {
        match self {
            NonceType::V2(n) => n,
            NonceType::V3(n) => n,
        }
    }
}

// Struct for encrypted data
struct EncryptedData {
    version: u8,
    derivation: u8,
    salt: Vec<u8>,
    nonce: NonceType,
    ciphertext: Vec<u8>,
}

// Implement the encryption struct
impl EncryptedData {
    fn new(plaintext: &str, password: &str) -> Result<Self, ()> {
        let version = 4;
        let derivation = g_get_derivation() as u8;
        let mut salt = [0u8; 16];
        OsRng.fill_bytes(&mut salt);
        let mut nonce_bytes = [0u8; 24];
        OsRng.fill_bytes(&mut nonce_bytes);
        let key = key_from_pw(password, &salt, derivation)?;

        let ciphertext = if derivation == 0 {
            plaintext.as_bytes().to_vec()
        } else {
            let cipher = XChaCha20Poly1305::new(&key.into());
            let xnonce = chacha20poly1305::XNonce::from(nonce_bytes);

            let header_ad = if Self::use_authenticated_header(version) {
                Self::associated_data(version, derivation, &salt, &nonce_bytes)
            } else {
                salt.to_vec()
            };

            let payload = Payload {
                msg: plaintext.as_bytes(),
                aad: &header_ad,
            };

            cipher.encrypt(&xnonce, payload).map_err(|_| ())?
        };

        Ok(EncryptedData {
            version: version,
            derivation: derivation,
            salt: salt.to_vec(),
            nonce: NonceType::V3(nonce_bytes),
            ciphertext: ciphertext,
        })
    }

    fn decrypt(&self, password: &str) -> Result<Vec<u8>, ()> {
        if self.derivation == 0 {
            return Ok(self.ciphertext.clone());
        }

        if self.version < 4 {
            return legacy_decrypt(
                self.version,
                self.derivation,
                &self.salt,
                self.nonce.as_bytes(),
                &self.ciphertext,
                password,
            );
        }

        let key = key_from_pw(password, &self.salt, self.derivation)?;

        match &self.nonce {
            NonceType::V2(nonce_bytes) => {
                let cipher = ChaCha20Poly1305::new(&key.into());
                let n = chacha20poly1305::Nonce::from_slice(nonce_bytes);

                let payload = Payload {
                    msg: self.ciphertext.as_slice(),
                    aad: &self.salt,
                };

                cipher.decrypt(n, payload).map_err(|_| ())
            }
            NonceType::V3(nonce_bytes) => {
                let cipher = XChaCha20Poly1305::new(&key.into());
                let n = chacha20poly1305::XNonce::from_slice(nonce_bytes);

                let header_ad = if Self::use_authenticated_header(self.version) {
                    Self::associated_data(
                        self.version,
                        self.derivation,
                        &self.salt,
                        nonce_bytes,
                    )
                } else {
                    self.salt.clone()
                };

                let payload = Payload {
                    msg: self.ciphertext.as_slice(),
                    aad: &header_ad,
                };

                cipher.decrypt(n, payload).map_err(|_| ())
            }
        }
    }

    fn use_authenticated_header(version: u8) -> bool {
        version >= 3
    }

    fn associated_data(version: u8, derivation: u8, salt: &[u8], nonce: &[u8]) -> Vec<u8> {
        let mut data = vec![version, derivation];
        data.extend_from_slice(salt);
        data.extend_from_slice(nonce);
        data
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![self.version, self.derivation];
        bytes.extend(self.salt.iter());
        bytes.extend(self.nonce.as_bytes().iter());
        bytes.extend(self.ciphertext.iter());
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, base64::DecodeError> {
        let n = 2;
        let version = bytes[0];
        let derivation = bytes[1];

        let sbytes = if version < 4 {
            sodiumoxide::crypto::pwhash::SALTBYTES
        } else {
            16
        };

        let (nonce, nbytes) = if version < 3 {
            let nbytes = 12;

            if bytes.len() < (n + sbytes + nbytes) {
                return Err(base64::DecodeError::InvalidLength);
            }

            let mut nonce_bytes = [0u8; 12];
            nonce_bytes.copy_from_slice(&bytes[(n + sbytes)..(n + sbytes + nbytes)]);
            (NonceType::V2(nonce_bytes), nbytes)
        } else {
            let nbytes = 24;

            if bytes.len() < (n + sbytes + nbytes) {
                return Err(base64::DecodeError::InvalidLength);
            }

            let mut nonce_bytes = [0u8; 24];
            nonce_bytes.copy_from_slice(&bytes[(n + sbytes)..(n + sbytes + nbytes)]);
            (NonceType::V3(nonce_bytes), nbytes)
        };

        let ciphertext: Vec<u8> = bytes[(n + sbytes + nbytes)..]
            .iter()
            .map(|x| x.to_owned())
            .collect();

        let mut salt = vec![0u8; sbytes];
        salt.copy_from_slice(&bytes[n..(n + sbytes)]);

        Ok(EncryptedData {
            version: version,
            derivation: derivation,
            salt: salt,
            nonce: nonce,
            ciphertext: ciphertext,
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