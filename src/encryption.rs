use crate::s;
use crate::p;
use crate::e;

use crate::globals::
{
    UNLOCK_CHECK,
    g_get_password,
    g_set_password
};
use crate::file::
{
    update_file
};
use crate::notes::
{
    get_notes
};
use crate::input::
{
    get_input
};

use rand::
{
    Rng, prelude::*
};
use sha3::
{
    Sha3_256, Digest
};
use block_modes::
{
    BlockMode, Cbc,
    block_padding::Pkcs7
};

use aes_soft::Aes256;
type Aes256Cbc = Cbc<Aes256, Pkcs7>;

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
    let mut hasher = Sha3_256::new();
    hasher.input(get_password(false).as_bytes());
    let key = hasher.result();
    let iv = generate_iv(&key);

    let cipher = match Aes256Cbc::new_var(&key, &iv)
    {
        Ok(cip) => cip, Err(_) => {e!("Can't init the encrypt cipher."); return s!()}
    };

    let encrypted = cipher.encrypt_vec(plain_text.as_bytes()); hex::encode(&encrypted)
}

// Decodes the hex data and decrypts it
pub fn decrypt_text(encrypted_text: &str) -> String
{
    if encrypted_text.trim().is_empty() {return s!()}
    let mut hasher = Sha3_256::new();
    hasher.input(get_password(false).as_bytes());
    let key = hasher.result();
    let iv = generate_iv(&key);

    let ciphertext = match hex::decode(encrypted_text)
    {
        Ok(ct) => ct, Err(_) => {e!("Can't decode the hex text to decrypt."); return s!()}
    };

    let cipher = match Aes256Cbc::new_var(&key, &iv)
    {
        Ok(cip) => cip, Err(_) => {e!("Can't init the decrypt cipher."); return s!()}
    };

    let decrypted = match cipher.decrypt_vec(&ciphertext)
    {
        Ok(dec) => dec, Err(_) => {e!("Wrong password."); return s!()}
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

// <Alipha> madprops: an IV is an Initialization Vector and (generally) must be randomly-generated
// and different each and every time you encrypt using the same key. Not using a different,
// random IV means someone will be able to decrypt your ciphertext

// <Alipha> madprops: AES-CBC works by xor'ing the previous block with the current block.
// So for the first block, there's no previous block. So the IV is used as the previous block.
// <Alipha> madprops: also, for your encryption scheme to be secure, you need to authenticate your
// ciphertext to make sure it hasn't been maliciously modified. This can be done with HMAC, poly1305
// or other algorithms. Or, you can use AES-GCM instead of AES-CBC, which authenticates the ciphertext for you.

// Generates the IV used to encrypt and decrypt the file
pub fn generate_iv(key: &[u8]) -> Vec<u8>
{
    let hex_chars = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];
    let mut chars: Vec<char> = vec![];
    let mut rng: StdRng = SeedableRng::from_seed(get_seed_array(key));

    for _ in 0..key.len()
    {
        chars.push(hex_chars[((rng.gen::<u8>()) % 16) as usize]);
    }

    hex::decode(chars.iter().collect::<String>()).unwrap()
}

// Fills and array based on the key to generate the IV
pub fn get_seed_array(source: &[u8]) -> [u8; 32]
{
    let mut array = [0; 32];
    let items = &source[..array.len()];
    array.copy_from_slice(items); array
}