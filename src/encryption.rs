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
    }
};

use std::io;
use std::io::Write;

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


fn key_from_pw(password: &str, salt: pwhash::Salt) -> Result<aead::Key, ()> {
    let mut key = aead::Key([0; aead::KEYBYTES]);
    pwhash::derive_key_interactive(&mut key.0, password.as_bytes(), &salt)?;
    Ok(key)
}

struct EncryptedData {
    salt: pwhash::Salt,
    nonce: aead::Nonce,
    ciphertext: Vec<u8>,
}

impl EncryptedData {
    fn new(plaintext: &str, password: &str) -> Result<Self, ()> {
        let salt = pwhash::gen_salt();
        let nonce = aead::gen_nonce();
        let key = key_from_pw(password, salt)?;

        let ciphertext = aead::seal(plaintext.as_bytes(), Some(&salt.0), &nonce, &key);

        Ok(EncryptedData { salt, nonce, ciphertext })
    }

    fn decrypt(&self, password: &str) -> Result<Vec<u8>, ()> {
        let key = key_from_pw(password, self.salt)?;
        aead::open(&self.ciphertext, Some(&self.salt.0), &self.nonce, &key)
    }

    fn to_string(&self) -> Result<String, io::Error> {
        let mut bytes = Vec::new();

        {
            let mut encoder = base64::write::EncoderWriter::new(&mut bytes,
                                                                base64::STANDARD);
            encoder.write_all(&self.salt.0)?;
            encoder.write_all(&self.nonce.0)?;
            encoder.write_all(&self.ciphertext)?;
            encoder.finish()?;
        }

        // This needlessly checks that the base64 encoding is valid UTF-8.
        // It all could be made much faster and more compact with a binary format.
        Ok(String::from_utf8(bytes).unwrap())
    }

    fn from_string(text: &str) -> Result<Self, base64::DecodeError> {
        let mut bytes = base64::decode(text)?;
        // TODO: Could avoid a bunch of copying; again, this would be faster
        // and more efficient for a binary format anyhow
        let ciphertext = bytes.split_off(pwhash::SALTBYTES + aead::NONCEBYTES);
        let salt  =     pwhash::Salt::from_slice(&bytes[..pwhash::SALTBYTES]).unwrap();
        let nonce = aead::Nonce::from_slice(&bytes[pwhash::SALTBYTES..]).unwrap();

        Ok(EncryptedData { salt, nonce, ciphertext })
    }

}


// Encrypts the notes using sodiumoxyde::aead and sodiumoxyde::pwhash
// Turns the encrypted data into base64
pub fn encrypt_text(plaintext: &str) -> String
{
    let password = get_password(false);
    let ciphertext = EncryptedData::new(plaintext, &password).unwrap();
    ciphertext.to_string().unwrap()
}

// Decodes the base64 data and decrypts it
pub fn decrypt_text(encrypted_text: &str) -> String
{
    let ciphertext = EncryptedData::from_string(encrypted_text).unwrap();
    let password = get_password(false);
    let plaintext = ciphertext.decrypt(&password).unwrap();
    String::from_utf8(plaintext).unwrap()
}
