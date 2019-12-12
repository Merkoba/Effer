use crate::
{
    s, p, e,
    globals::
    {
        g_get_password,
        g_set_password
    },
    file::
    {
        update_file
    },
    notes::
    {
        get_notes
    },
    input::
    {
        get_input
    },
    other::
    {
        exit
    }
};

use sodiumoxide::crypto::aead;
use sodiumoxide::crypto::pwhash;

// Changes the password and updates the file with it
pub fn change_password()
{
    p!("This will change the file's password.");
    if !get_password(true).is_empty() {update_file(get_notes(false))};
}

// Gets the file's password saved globally
// Or asks the user for the file's password
// Or changes the file's password
pub fn get_password(change: bool) -> String
{
    let mut pw = g_get_password();

    if pw.is_empty() || change
    {
        let mut password: String;

        if change
        {
            loop
            {
                password = get_input("New Password", "", true);
                if password.is_empty() {return s!()}
                let confirmation = get_input("Confirm Password", "", true);
                if password != confirmation {e!("Error: Passwords Don't Match.")} else {break}
            }
        }

        else
        {
            password = get_input("Password", "", true);
        }

        pw = s!(password); g_set_password(password);
    }

    pw
}


fn key_from_pw(password: &str, salt: pwhash::Salt) -> Result<aead::Key, ()> 
{
    let mut key = aead::Key([0; aead::KEYBYTES]);
    pwhash::derive_key_interactive(&mut key.0, password.as_bytes(), &salt)?;
    Ok(key)
}

struct EncryptedData 
{
    version: u8,
    salt: pwhash::Salt,
    nonce: aead::Nonce,
    ciphertext: Vec<u8>,
}

impl EncryptedData 
{
    fn new(plaintext: &str, password: &str) -> Result<Self, ()> 
    {
        let version = 2;
        let salt = pwhash::gen_salt();
        let nonce = aead::gen_nonce();
        let key = key_from_pw(password, salt)?;
        let ciphertext = aead::seal(plaintext.as_bytes(), Some(&salt.0), &nonce, &key);
        Ok(EncryptedData { version, salt, nonce, ciphertext })
    }

    fn decrypt(&self, password: &str) -> Result<Vec<u8>, ()> 
    {
        let key = key_from_pw(password, self.salt)?;
        aead::open(&self.ciphertext, Some(&self.salt.0), &self.nonce, &key)
    }

    fn to_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = vec![self.version];
        bytes.extend(self.salt.0.iter());
        bytes.extend(self.nonce.0.iter());
        bytes.extend(self.ciphertext.iter());
        return bytes
    }

    fn from_bytes(bytes: &Vec<u8>) -> Result<Self, base64::DecodeError> 
    {
        let sbytes = pwhash::SALTBYTES;
        let nbytes = aead::NONCEBYTES;
        let ciphertext: Vec<u8> = bytes[(1 + sbytes + nbytes)..]
                                .iter().map(|x| x.to_owned()).collect();
        let version = bytes[0];
        let salt = pwhash::Salt::from_slice(&bytes[1..(1 + sbytes)]).unwrap();
        let nonce = aead::Nonce::from_slice(&bytes[(1 + sbytes)..(1 + sbytes + nbytes)]).unwrap();
        Ok(EncryptedData { version, salt, nonce, ciphertext })
    }
}

// Encrypts the notes using sodiumoxyde::aead and sodiumoxyde::pwhash
// Turns the encrypted data into base64
pub fn encrypt_bytes(plaintext: &str) -> Vec<u8>
{
    let password = get_password(false);
    let ciphertext = EncryptedData::new(plaintext, &password).unwrap();
    ciphertext.to_bytes()
}

// Decodes the base64 data and decrypts it
pub fn decrypt_bytes(bytes: &Vec<u8>) -> String
{
    let ciphertext = EncryptedData::from_bytes(bytes).unwrap();
    let password = get_password(false);

    let plaintext = match ciphertext.decrypt(&password)
    {
        Ok(text) => text,
        Err(_) => {e!("Can't decrypt the file."); exit()}
    };

    String::from_utf8(plaintext).unwrap()
}
