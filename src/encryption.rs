use crate::
{
    s, p, e,
    globals::
    {
        UNLOCK_CHECK,
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

use std::iter;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;

use sodiumoxide::crypto::secretbox;
use sodiumoxide::crypto::pwhash;
use sodiumoxide::crypto::pwhash::scryptsalsa208sha256::Salt;

use aes_gcm::Aes256Gcm;
use aead::{Aead, NewAead, generic_array::GenericArray};

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

// Encrypts the notes using Aes256
// Turns the encrypted data into hex
pub fn encrypt_text(plain_text: &str) -> String
{
    let salt = pwhash::gen_salt();

    let mut key = secretbox::Key([0; secretbox::KEYBYTES]);
    {
        let secretbox::Key(ref mut kb) = key;
        pwhash::derive_key(kb, get_password(false).as_bytes(), &salt,
            pwhash::OPSLIMIT_INTERACTIVE,
            pwhash::MEMLIMIT_INTERACTIVE).unwrap();
    }

    let arkey = GenericArray::clone_from_slice(key.as_ref());
    let aead = Aes256Gcm::new(arkey);
    let nonce = generate_nonce();
    let arnonce = GenericArray::clone_from_slice(nonce.as_bytes());

    let ciphertext = match aead.encrypt(&arnonce, plain_text.as_ref())
    {
        Ok(ct) => ct, Err(_) => {e!("Encryption failed."); return s!()}
    };

    format!("{}-;-{}-;-{}", base64::encode(&salt), nonce, hex::encode(&ciphertext))
}

// Decodes the hex data and decrypts it
pub fn decrypt_text(encrypted_text: &str) -> String
{
    if encrypted_text.trim().is_empty() {return s!()}
    
    let mut xalt = "";
    let mut nonx = "";
    let mut clines: Vec<&str> = vec![];
    
    for (i, line) in encrypted_text.lines().enumerate()
    {
        if i == 0
        {
            let split = line.split("-;-").collect::<Vec<&str>>();
            xalt = split[0];
            nonx = split[1];
            clines.push(split[2]);
        } else {clines.push(line)}
    }

    let enctext = clines.join("\n");
    let salt = Salt::from_slice(base64::decode(xalt).unwrap().as_ref()).unwrap();
    let nonce = GenericArray::clone_from_slice(nonx.as_bytes());

    let mut key = secretbox::Key([0; secretbox::KEYBYTES]);
    {
        let secretbox::Key(ref mut kb) = key;
        pwhash::derive_key(kb, get_password(false).as_bytes(), &salt,
            pwhash::OPSLIMIT_INTERACTIVE,
            pwhash::MEMLIMIT_INTERACTIVE).unwrap();
    }

    let arkey = GenericArray::clone_from_slice(key.as_ref());
    let aead = Aes256Gcm::new(arkey);

    let ciphertext = match hex::decode(enctext)
    {
        Ok(ct) => ct, Err(_) => {e!("Can't decode the hex text to decrypt."); return s!()}
    };

    let decrypted = match aead.decrypt(&nonce, ciphertext.as_ref())
    {
        Ok(txt) => txt, Err(_) => {e!("Decryption failed."); return s!()}
    };

    let text = match String::from_utf8(decrypted)
    {
        Ok(txt) => txt, Err(_) => {e!("Can't turn the decrypted data into a string."); return s!()}
    };

    let header = match text.lines().nth(0)
    {
        Some(hd) => hd, None => {e!("Can't read the header."); return s!()}
    };

    if !header.starts_with(UNLOCK_CHECK)
    {
        e!("Wrong password."); return s!();
    }

    text
}

pub fn generate_nonce() -> String
{
    let mut rng = thread_rng();

    let chars: String = iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(12)
        .collect();

    chars
}