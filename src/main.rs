#[macro_use]
extern crate strum_macros;
extern crate strum;
#[macro_use]
extern crate lazy_static;
extern crate rpassword;
extern crate aes_soft as aes;
extern crate block_modes;
extern crate hex;
extern crate rustyline;

mod macros;

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::io::{Write};
use std::process::exit;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use aes::Aes256;
use dirs;
use rand::prelude::*;
use sha3::{Sha3_256, Digest};
use std::sync::Mutex;
use strum::IntoEnumIterator;
use rustyline::Editor;

type Aes256Cbc = Cbc<Aes256, Pkcs7>;

const FIRST_LINE: &str = "<Notes Unlocked>";

lazy_static! 
{
    static ref PASSWORD: Mutex<String> = Mutex::new(s!());
    static ref NOTES: Mutex<String> = Mutex::new(s!());
}

enum FilePathCheckResult
{
    DoesNotExist,
    NotAFile,
    Exists
}

impl FilePathCheckResult
{
    fn message(&self) -> &str
    {
        match *self
        {
            FilePathCheckResult::Exists => "File exists.",
            FilePathCheckResult::DoesNotExist => "File doesn't exist.",
            FilePathCheckResult::NotAFile => "The path exists but it's not a file."
        }
    }
}

#[derive(EnumIter)]
enum MenuAnswer
{
    Nothing,
    AddNote,
    EditNote,
    FindNotes,
    DeleteNotes,
    RemakeFile,
    ChangePassword
}

impl MenuAnswer
{
    fn new(n: usize) -> MenuAnswer
    {
        MenuAnswer::iter().nth(n).unwrap_or(MenuAnswer::Nothing)
    }

    fn get_label(&self) -> String
    {
        match self
        {
            MenuAnswer::AddNote => s!("Add Note"),
            MenuAnswer::FindNotes => s!("Find Notes"),
            MenuAnswer::EditNote => s!("Edit Note"),
            MenuAnswer::DeleteNotes => s!("Delete Notes"),
            MenuAnswer::RemakeFile => s!("Remake File"),
            MenuAnswer::ChangePassword => s!("Change Password"),
            _ => s!()
        }
    }

    fn display_menu()
    {
        for (i, a) in MenuAnswer::iter().enumerate()
        {
            if i == 0 {continue}
            let s = format_item(i, &a.get_label());
            if i % 3 == 0 {p!(s)} else {pp!("{} | ", s)}
        }
    }

    fn exec(&self, full: bool)
    {
        match self
        {
            MenuAnswer::AddNote =>
            {
                add_note();
            },
            MenuAnswer::FindNotes =>
            {
                show_notes(find_notes());
            },
            MenuAnswer::EditNote =>
            {
                edit_note();
            },
            MenuAnswer::DeleteNotes =>
            {
                delete_notes();
            },
            MenuAnswer::RemakeFile =>
            {
                remake_file();
            },
            MenuAnswer::ChangePassword =>
            {
                change_password();
            },
            MenuAnswer::Nothing =>
            {
                if full {exit(0)} else {show_notes(vec![])}
            }
        }
    }
}

fn main() 
{
    handle_file_path_check(file_path_check(get_file_path()));
    get_password();
    check_password();
    show_notes(vec![]);
}

fn get_home_path() -> PathBuf
{
    match dirs::home_dir()
    {
        Some(path) => path,
        None => 
        {
            p!("Can't get your Home path."); exit(0);
        }
    }
}

fn get_file_path() -> PathBuf
{
    get_home_path().join(Path::new(".config/effer/effer.dat"))
}

fn get_file_parent_path() -> PathBuf
{
    get_home_path().join(Path::new(".config/effer"))
}

fn file_path_check(path: PathBuf) -> FilePathCheckResult
{
    match fs::metadata(path)
    {
        Ok(attr) => 
        {
            if !attr.is_file()
            {
                return FilePathCheckResult::NotAFile;
            }
        },
        Err(_) =>
        {
            return FilePathCheckResult::DoesNotExist;
        }
    }
    
    FilePathCheckResult::Exists
}

fn handle_file_path_check(result: FilePathCheckResult)
{
    match result
    {
        FilePathCheckResult::Exists =>
        {
            //
        },
        FilePathCheckResult::DoesNotExist =>
        {
            p!(result.message());
            let answer = ask_bool(s!("Do you want to make the file now?"));
            if answer {create_file()} else {exit(0)}
        },
        FilePathCheckResult::NotAFile =>
        {
            p!(result.message());
            let answer = ask_bool(s!("Do you want to re-make the file?"));
            if answer {create_file()} else {exit(0)}
        }
    }
}

fn get_input<F, E, T>(message: String, initial: String, f_ok: F, f_err: E) -> T 
where F: Fn(String) -> T, E: Fn() -> T
{
    let mut editor = Editor::<()>::new();
    let prompt = format!("{}: ", message);

    match editor.readline_with_initial(&prompt, (&initial, &s!()))
    {
        Ok(input) => 
        {
            f_ok(input)
        },
        Err(_) => 
        {
            f_err()
        }
    }
}

fn ask_bool(message: String) -> bool
{
    get_input(message + " (y, n)", s!(), |a| a.trim().to_lowercase() == "y", || false)
}

fn ask_int(message: String) -> usize
{
    get_input(message, s!(), |a| a.trim().parse::<usize>().unwrap_or(0), || 0)
}

fn ask_string(message: String, initial: String) -> String
{
    get_input(message, initial, |a| a.trim().to_string(), || s!())
}

fn get_password() -> String
{
    let mut pw = PASSWORD.lock().unwrap();

    if pw.chars().count() == 0
    {
        let password = rpassword::prompt_password_stdout("Password: ").unwrap();
        if password.chars().count() == 0 {exit(0)}
        *pw = password;
    }

    pw.to_string()
}

fn unset_password()
{
    let mut password = PASSWORD.lock().unwrap();
    *password = s!();
}

fn create_file()
{
    let password = get_password();

    if password.chars().count() == 0
    {
        exit(0);
    }

    let encrypted = encrypt_text(s!(FIRST_LINE));
    let parent_path = get_file_parent_path();
    fs::create_dir_all(parent_path).unwrap();
    let file_path = get_file_path();
    let mut file = fs::File::create(&file_path).expect("Error creating the file.");
    file.write_all(encrypted.as_bytes()).expect("Unable to write initial text to file");
    p!("File created at {}", &file_path.display());
}

fn encrypt_text(plain_text: String) -> String
{
    let password = get_password();
    let text = plain_text.trim().to_string();
    let mut hasher = Sha3_256::new();
    hasher.input(password.as_bytes());
    let key = hasher.result();
    let iv = generate_iv(&key);
    let cipher = Aes256Cbc::new_var(&key, &iv).expect("Can't init the encrypt cipher.");
    let encrypted = cipher.encrypt_vec(text.as_bytes());
    hex::encode(&encrypted)
}

fn decrypt_text(encrypted_text: String) -> String
{
    if encrypted_text.trim() == ""
    {
        return s!();
    }

    let password = get_password();
    let mut hasher = Sha3_256::new();
    hasher.input(password.as_bytes());
    let key = hasher.result();
    let iv = generate_iv(&key);
    let ciphertext = hex::decode(encrypted_text).expect("Can't decode the hex text to decrypt.");
    let cipher = Aes256Cbc::new_var(&key, &iv).expect("Can't init the decrypt cipher.");
    let decrypted = cipher.decrypt_vec(&ciphertext);

    match decrypted
    {
        Ok(_) => (),
        Err(_) => 
        {
            p!("Wrong password."); exit(0);
        }
    }
    
    String::from_utf8(decrypted.unwrap()).expect("Can't turn the decrypted data into a string.")
}

// <Alipha> madprops: an IV is an Initialization Vector and (generally) must be randomly-generated 
// and different each and every time you encrypt using the same key. Not using a different, 
// random IV means someone will be able to decrypt your ciphertext

// <Alipha> madprops: AES-CBC works by xor'ing the previous block with the current block. 
// So for the first block, there's no previous block. So the IV is used as the previous block.
// <Alipha> madprops: also, for your encryption scheme to be secure, you need to authenticate your 
// ciphertext to make sure it hasn't been maliciously modified. This can be done with HMAC, poly1305 
// or other algorithms. Or, you can use AES-GCM instead of AES-CBC, which authenticates the ciphertext for you.

fn generate_iv(key: &[u8]) -> Vec<u8>
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

fn show_notes(lines: Vec<String>)
{
    loop
    {
        p!(""); p!("---------------"); p!("");

        if lines.is_empty()
        {
            let notes = get_notes(false);

            for (i, line) in notes.lines().enumerate()
            {
                if i > 0
                {
                    p!(format_item(i, line));
                }
            }
        }

        else
        {
            for line in lines.iter()
            {
                p!(line);
            }
        }

        p!("");

        let full = lines.is_empty();
        let prompt = if full {"Number (Empty To Exit)"} else {"Number (Empty To Show All)"};
        MenuAnswer::display_menu();
        MenuAnswer::new(ask_int(s!(prompt))).exec(full);
    }
}

fn get_file_text() -> String
{
    fs::read_to_string(get_file_path()).expect("Can't read file content.")
}

fn check_password()
{
    let text = decrypt_text(get_file_text());
    let first_line = text.lines().nth(0).expect("Can't read last line from the file.");

    if first_line != FIRST_LINE
    {
        p!("Wrong password."); exit(0);
    }
}

fn get_seed_array(source: &[u8]) -> [u8; 32]
{
    let mut array = [0; 32];
    let items = &source[..array.len()];
    array.copy_from_slice(items); array
}

fn get_notes(force_update: bool) -> String
{
    let mut notes = NOTES.lock().unwrap();

    if notes.chars().count() == 0 || force_update
    {
        let encrypted_text = get_file_text();
        let text = decrypt_text(encrypted_text);
        *notes = text;
    }

    notes.to_string()
}

fn update_file(text: String)
{
    fs::write(get_file_path(), encrypt_text(text).as_bytes()).expect("Unable to write new text to file");
    get_notes(true);
}

fn get_line(n: usize) -> String
{
    let notes = get_notes(false);
    let lines: Vec<&str> = notes.lines().collect();
    if n >= lines.len() {return s!()}
    lines[n].to_string()
}

fn replace_line(n: usize, new_text: String)
{
    let notes = get_notes(false);
    let mut lines: Vec<&str> = notes.lines().collect();
    if n >= lines.len() {return}
    lines[n] = &new_text[..];

    update_file(lines.iter()
        .map(|l| l.to_string())
        .collect::<Vec<String>>().join("\n"));

    get_notes(true);
}

fn delete_lines(numbers: Vec<usize>)
{
    let notes = get_notes(false);
    let lines: Vec<&str> = notes.lines().collect();
    let mut new_lines: Vec<&str> = vec![];

    for (i, line) in lines.iter().enumerate()
    {
        if i == 0 || !numbers.contains(&i)
        {
            new_lines.push(line);
        }
    }

    update_file(new_lines.iter()
        .map(|l| l.to_string())
        .collect::<Vec<String>>().join("\n"));

    get_notes(true);
}

fn add_note()
{
    let note = ask_string(s!("New Note"), s!());
    if note.is_empty() {return}
    let new_text = format!("{}\n{}", get_notes(false), note);
    update_file(new_text);
}

fn edit_note()
{
    let n = ask_int(s!("Note Number"));
    if n == 0 {return}
    let line = get_line(n);
    if line == "" {return}
    let edited = ask_string(s!("Edit Note"), line);
    if edited.is_empty() {return}
    replace_line(n, edited);
}

fn find_notes() -> Vec<String>
{
    let filter = ask_string(s!("Filter"), s!()).to_lowercase();
    let mut found: Vec<String> = vec![];
    if filter == "" {return found}
    let filter_list: Vec<&str> = filter.split_whitespace().collect();
    let notes = get_notes(false);
    let lines: Vec<&str> = notes.lines().collect();

    for (i, line) in lines.iter().enumerate()
    {
        if i == 0 {continue}

        for word in filter_list.iter()
        {
            if line.to_lowercase().contains(word)
            {
                found.push(format_item(i, line));
                break;
            }
        }
    }

    if found.is_empty()
    {
        found.push(s!("<No Results>"));
    }

    found
}

fn delete_notes()
{
    p!("Note Number");
    p!("Or Note List (e.g 1,2,3)");
    p!("Or Note Range (e.g 1-3)");

    let ans = ask_string(s!(), s!());
    let mut numbers: Vec<usize> = vec![];

    if ans.contains(',')
    {
        numbers.extend(ans.split(',').map(|n| n.trim().parse::<usize>().unwrap_or(0)).collect::<Vec<usize>>());
    }

    else if ans.contains('-')
    {
        let mut split = ans.split('-').map(|n| n.trim());
        let mut num1 = split.next().unwrap_or("0").parse::<usize>().unwrap_or(0);
        let mut num2 = split.next().unwrap_or("0").parse::<usize>().unwrap_or(0);
        
        if num1 == 0 {num1 = 1}
        if num2 == 0 {num2 = get_notes(false).lines().count()}
        if num1 >= num2 {return}

        numbers.extend(num1..=num2);
    }

    else
    {
        numbers.push(ans.parse::<usize>().unwrap_or(0));
    }

    numbers = numbers.iter().filter(|x| **x != 0).copied().collect();

    if !numbers.is_empty()
    {
        delete_lines(numbers);
    }
}

fn format_item(n: usize, s: &str) -> String
{
    format!("({}) {}", n, s)
}

fn remake_file()
{
    if ask_bool(s!("Are you sure you want to replace the file with an empty one?"))
    {
        fs::remove_file(get_file_path()).unwrap();
        create_file();
        get_notes(true);
    }
}

fn change_password()
{
    unset_password();
    update_file(get_notes(false));
}