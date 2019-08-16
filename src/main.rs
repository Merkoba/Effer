mod macros;
mod structs;
use structs::FilePathCheckResult;
use structs::MenuAnswer;
use structs::MaskingHighlighter;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::io::{Write, stdout, stdin};
use std::process;
use std::cmp::max;
use std::cmp::min;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use aes_soft::Aes256;
use dirs;
use rand::prelude::*;
use sha3::{Sha3_256, Digest};
use std::sync::Mutex;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use lazy_static::lazy_static;
use rustyline::
{
    Editor, Cmd, KeyPress,
    Config, OutputStreamType
};

type Aes256Cbc = Cbc<Aes256, Pkcs7>;
const FIRST_LINE: &str = "<Notes Unlocked>";
const ITEMS_PER_LEVEL: usize = 15;

// Global variables
lazy_static! 
{
    static ref PASSWORD: Mutex<String> = Mutex::new(s!());
    static ref NOTES: Mutex<String> = Mutex::new(s!());
    static ref NOTES_LENGTH: Mutex<usize> = Mutex::new(0);
    static ref LEVEL: Mutex<usize> = Mutex::new(1);
}

// First function to execute
fn main() 
{
    handle_file_path_check(file_path_check(get_file_path()));
    if get_password(false).is_empty() {exit()};
    update_notes_static(get_notes(false));
    check_password(); change_screen(); goto_last_page();
}

// Switch to the alternative screen
// Place the cursor at the bottom left
fn change_screen()
{
    p!("\x1b[?1049h");
    let size = termion::terminal_size().unwrap();
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}", termion::cursor::Goto(1, size.1)).unwrap();
}

// Centralized function to exit the program
// Switches back to main screen before exiting
fn exit() -> !
{
    p!("\x1b[?1049l"); process::exit(0)
}

// Tries to get the user's home path
fn get_home_path() -> PathBuf
{
    match dirs::home_dir()
    {
        Some(path) => path,
        None => 
        {
            p!("Can't get your Home path."); exit();
        }
    }
}

// Gets the path of the file
fn get_file_path() -> PathBuf
{
    get_home_path().join(Path::new(".config/effer/effer.dat"))
}

// Gets the path of the file's parent
fn get_file_parent_path() -> PathBuf
{
    get_home_path().join(Path::new(".config/effer"))
}

// Checks the existence of the file
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

// Reacts to previous check of file existence
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
            let answer = ask_bool("Do you want to make the file now?");
            if answer {if !create_file() {exit()}} else {exit()}
        },
        FilePathCheckResult::NotAFile =>
        {
            p!(result.message());
            let answer = ask_bool("Do you want to re-make the file?");
            if answer {if !create_file() {exit()}} else {exit()}
        }
    }
}

// Centralized function to handle user input
// It's generic and can return different types
// Closures are supplied to react on success or failure
// Can make typing invisible for cases like password input
fn get_input<F, E, T>(message: &str, initial: &str, f_ok: F, f_err: E, mask: bool) -> T 
where F: Fn(String) -> T, E: Fn() -> T
{
    let config: Config = Config::builder()
        .keyseq_timeout(50)
        .output_stream(OutputStreamType::Stdout)
        .build();

    let h = MaskingHighlighter {masking: mask};
    let mut editor = Editor::with_config(config);
    editor.set_helper(Some(h));
    editor.bind_sequence(KeyPress::Esc, Cmd::Interrupt);
    let prompt = format!("{}: ", message);
    if mask {pp!("{}", termion::cursor::Hide)}

    let ans = match editor.readline_with_initial(&prompt, (initial, &s!()))
    {
        Ok(input) => 
        {
            f_ok(input)
        },
        Err(_) => 
        {
            f_err()
        }
    };

    if mask {pp!("{}", termion::cursor::Show)} ans
}

// Asks the user for a yes/no answer
fn ask_bool(message: &str) -> bool
{
    get_input(&[message, " (y, n)"].concat(), "", |a| a.trim().to_lowercase() == "y", || false, false)
}

// Asks the user to input a string
fn ask_string(message: &str, initial: &str) -> String
{
    get_input(message, initial, |a| a.trim().to_string(), String::new, false)
}

// Gets the file's password saved globally
// Or asks the user for the file's password
// Or changes the file's password
fn get_password(change: bool) -> String
{
    let mut pw = PASSWORD.lock().unwrap();

    if pw.is_empty() || change
    {
        let mut password: String;

        if change
        {
            loop
            {
                password = get_input("New Password", "", |a| a, String::new, true);
                if password.is_empty() {return s!()}
                let confirmation = get_input("Confirm Password", "", |a| a, String::new, true);
                if password != confirmation {p!("Error: Passwords Don't Match.")} else {break}
            }
        }

        else
        {
            password = get_input("Password", "", |a| a, String::new, true);
        }

        *pw = password;
    }

    pw.to_string()
}

// Attempts to create the file
// It adds FIRST_LINE as its only initial content
// The content is encrypted using the password
fn create_file() -> bool
{
    if get_password(true).is_empty() {return false}
    let encrypted = encrypt_text(s!(FIRST_LINE));
    let parent_path = get_file_parent_path();
    fs::create_dir_all(parent_path).unwrap();
    let file_path = get_file_path();
    let mut file = fs::File::create(&file_path).expect("Error creating the file.");
    file.write_all(encrypted.as_bytes()).expect("Unable to write initial text to file");
    p!("File created at {}", &file_path.display()); true
}

// Encrypts the notes using Aes256
// Turns the encrypted data into hex
fn encrypt_text(plain_text: String) -> String
{
    let text = plain_text.trim().to_string();
    let mut hasher = Sha3_256::new(); hasher.input(get_password(false).as_bytes());
    let key = hasher.result(); let iv = generate_iv(&key);
    let cipher = Aes256Cbc::new_var(&key, &iv).expect("Can't init the encrypt cipher.");
    let encrypted = cipher.encrypt_vec(text.as_bytes()); hex::encode(&encrypted)
}

// Decodes the hex data and decrypts it
fn decrypt_text(encrypted_text: String) -> String
{
    if encrypted_text.trim().is_empty() {return s!()}
    let mut hasher = Sha3_256::new(); hasher.input(get_password(false).as_bytes());
    let key = hasher.result(); let iv = generate_iv(&key);
    let ciphertext = hex::decode(encrypted_text).expect("Can't decode the hex text to decrypt.");
    let cipher = Aes256Cbc::new_var(&key, &iv).expect("Can't init the decrypt cipher.");
    let decrypted = cipher.decrypt_vec(&ciphertext);

    match decrypted
    {
        Ok(_) => (),
        Err(_) => 
        {
            p!("Wrong password."); exit();
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

// Generates the IV used to encrypt and decrypt the file
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

// Main renderer function
// Shows the notes and the menu at the bottom
// Then waits and reacts for input
fn show_notes(mut level: usize, lines: Vec<String>)
{
    loop
    {
        // Clear the screen
        p!("\x1b[2J");

        level = check_level(level, true);
        
        if level > 0
        {
            for line in get_note_range(level).iter() {p!(line)}
        }

        else
        {
            for line in lines.iter() {p!(line)}
        }

        if level > 0
        {
            { let mut lvl = LEVEL.lock().unwrap(); *lvl = level }
            p!(format!("\n< Page {} of {} >", level, get_max_level()));
        }
        
        let s = 
        [
            "\n(a) Add | ",
            "(e) Edit | ",
            "(f) Find | ",
            "(s) Swap",
            "\n(d) Delete | ",
            "(R) Remake | ",
            "(P) Change Password",
            "\n(Left/Right) Cycle Pages | ",
            "(Up) Edit Last Note",
            "\n(Home) First Page | ",
            "(End) Last Page | ",
            "(X) Exit",
        ].concat();

        p!(s); menu_action(menu_input());
    }
}

// Listens and interprets live keyboard input from the main menu
fn menu_input() -> (MenuAnswer, usize)
{
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}", termion::cursor::Hide).unwrap();
    stdout.flush().unwrap();

    let mut data = 0;

    let ans = match stdin.keys().next().unwrap().unwrap()
    {
        Key::Left => MenuAnswer::CycleLeft,
        Key::Right => MenuAnswer::CycleRight,
        Key::Up => MenuAnswer::EditLastNote,
        Key::Down => MenuAnswer::LastPage,
        Key::Home => MenuAnswer::FirstPage,
        Key::End => MenuAnswer::LastPage,
        Key::Char(ch) =>
        {
            match ch
            {
                d if d.is_digit(10) => 
                {
                    data = d.to_digit(10).unwrap() as usize; 
                    MenuAnswer::PageNumber
                },
                'a' => MenuAnswer::AddNote,
                'e' => MenuAnswer::EditNote,
                'f' => MenuAnswer::FindNotes,
                's' => MenuAnswer::SwapNotes,
                'd' => MenuAnswer::DeleteNotes,
                'R' => MenuAnswer::RemakeFile,
                'P' => MenuAnswer::ChangePassword,
                'X' => MenuAnswer::Exit,
                '\n' => MenuAnswer::RefreshPage,
                _ => MenuAnswer::Nothing
            }
        }
        Key::Ctrl('c') => MenuAnswer::Exit,
        _ => MenuAnswer::Nothing
    };

    stdout.flush().unwrap();
    write!(stdout, "{}", termion::cursor::Show).unwrap(); (ans, data)
}

// Reacts to the live keyboard input from the main menu
fn menu_action(ans: (MenuAnswer, usize))
{
    match ans.0
    {
        MenuAnswer::AddNote => {add_note()},
        MenuAnswer::EditNote => {edit_note(0)},
        MenuAnswer::FindNotes => {find_notes()},
        MenuAnswer::SwapNotes => {swap_notes()},
        MenuAnswer::DeleteNotes => {delete_notes()},
        MenuAnswer::RemakeFile => {remake_file()},
        MenuAnswer::ChangePassword => {change_password()},
        MenuAnswer::CycleLeft => {cycle_left()},
        MenuAnswer::CycleRight => {cycle_right()},
        MenuAnswer::FirstPage => {goto_first_page()},
        MenuAnswer::LastPage => {goto_last_page()},
        MenuAnswer::RefreshPage => {refresh_page()},
        MenuAnswer::EditLastNote => {edit_last_note()},
        MenuAnswer::PageNumber => {show_notes(max(1, ans.1), vec![])},
        MenuAnswer::Exit => {exit()},
        MenuAnswer::Nothing => {}
    }
}

// Reads the file
fn get_file_text() -> String
{
    fs::read_to_string(get_file_path()).expect("Can't read file content.")
}

// Checks if the first line was decrypted correctly
// This is a simple password and file integrity check
// If it fails the program exits
fn check_password()
{
    let text = get_notes(false);
    let first_line = text.lines().nth(0).expect("Can't read last line from the file.");

    if first_line != FIRST_LINE
    {
        p!("Wrong password."); exit();
    }
}

// Fills and array based on the key to generate the IV
fn get_seed_array(source: &[u8]) -> [u8; 32]
{
    let mut array = [0; 32];
    let items = &source[..array.len()];
    array.copy_from_slice(items); array
}

// Gets the notes form the global variable or reads them from the file
fn get_notes(update: bool) -> String
{
    let notes = NOTES.lock().unwrap();
    if notes.is_empty() || update {decrypt_text(get_file_text())} else {notes.to_string()}
}

// Retrieves the value of the global variable that holds the current amount of notes
fn get_notes_length() -> usize
{
    get_notes(false); *NOTES_LENGTH.lock().unwrap()
}

// Encrypts and saves the updated notes to the file
fn update_file(text: String)
{
    fs::write(get_file_path(), encrypt_text(update_notes_static(text))
        .as_bytes()).expect("Unable to write new text to file");
}

// Updates the notes and notes length global variables
fn update_notes_static(text: String) -> String
{
    let mut notes = NOTES.lock().unwrap();
    let mut length = NOTES_LENGTH.lock().unwrap();
    *length = text.lines().count() - 1; *notes = text; notes.to_string()
}

// Gets a specific line from the notes
fn get_line(n: usize) -> String
{
    let notes = get_notes(false);
    let lines: Vec<&str> = notes.lines().collect();
    if n >= lines.len() {return s!()}
    lines[n].to_string()
}

// Replaces a line from the notes with a new line
fn replace_line(n: usize, new_text: String)
{
    let notes = get_notes(false);
    let mut lines: Vec<&str> = notes.lines().collect();
    lines[n] = &new_text[..];

    update_file(lines.iter()
        .map(|l| l.to_string())
        .collect::<Vec<String>>().join("\n"));
}

// Swaps two lines from the notes
fn swap_lines(n1: usize, n2: usize)
{
    let notes = get_notes(false);
    let mut lines: Vec<&str> = notes.lines().collect();
    lines.swap(n1, n2);

    update_file(lines.iter()
        .map(|l| l.to_string())
        .collect::<Vec<String>>().join("\n"));
    }

// Deletes a line from the notes then updates the file
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
}

// Provides an input to add a new note
fn add_note()
{
    let note = ask_string("New Note", "");
    if note.is_empty() {return}
    let new_text = format!("{}\n{}", get_notes(false), note);
    update_file(new_text);
    goto_last_page();
}

// Asks for a note number to edit
// The note is then showed and editable
fn edit_note(mut n: usize)
{
    if n == 0
    {
        n = parse_note_ans(&ask_string("Edit #", ""));
    }

    if !check_line_exists(n) {return}
    let edited = ask_string("Edit Note", &get_line(n));
    if edited.is_empty() {return} replace_line(n, edited);
}

// Finds a note by a filter
// Case insensitive
// Substrings are counted
fn find_notes()
{
    let filter = ask_string("Filter", "").to_lowercase();
    let mut found: Vec<String> = vec![];
    if filter.is_empty() {return}
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

    let msg = "| (Enter) Go Back";

    if found.is_empty()
    {
        found.push(format!("< No Results {} >", msg));
    }

    else if found.len() == 1
    {
        found.push(format!("\n< 1 Result {} >", msg));
    }

    else
    {
        found.push(format!("\n< {} Results {} >", found.len(), msg));
    }

    show_notes(0, found);
}

// Swaps 2 notes specified by 2 numbers separated by whitespace (1 10)
fn swap_notes()
{
    let ans = ask_string("Swap (n1 n2)", "");
    let mut split = ans.split_whitespace().map(|s| s.trim());
    let n1 = parse_note_ans(split.next().unwrap_or("0"));
    let n2 = parse_note_ans(split.next().unwrap_or("0"));
    if !check_line_exists(n1) || !check_line_exists(n2) {return}
    swap_lines(n1, n2);
}

// Deletes 1 or more notes
// Can delete by a specific note number (3)
// Or a comma separated list (1,2,3)
// Or a range (1-3)
fn delete_notes()
{
    p!("Note Number");
    p!("Or Note List (e.g 1,2,3)");
    p!("Or Note Range (e.g 1-3)");

    let ans = ask_string("Delete", "");
    if ans.is_empty() {return}
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
    if !numbers.is_empty() {delete_lines(numbers)}
}

// Goes to the first page
fn goto_first_page()
{
    show_notes(1, vec![]);
}

// Goes to the last page
fn goto_last_page()
{
    show_notes(get_max_level(), vec![]);
}

// Refreshes the current page (notes, menu, etc)
// This doesn't provoke a change unless on a different mode like Find results
fn refresh_page()
{
    let lvl;

    {
        lvl = *LEVEL.lock().unwrap();
    }

    show_notes(lvl, vec![]);
}

// Generic format for note items
fn format_item(n: usize, s: &str) -> String
{
    format!("({}) {}", n, s)
}

// Asks the user if it wants to delete the current file and make a new one
// Then does it if response was positive
// Variables are then updated to reflect change
fn remake_file()
{
    if ask_bool("Are you sure you want to replace the file with an empty one?")
    {
        fs::remove_file(get_file_path()).unwrap();
        if !create_file() {exit()} 
        update_notes_static(get_notes(true));
    }
}

// Changes the password and updates the file with it
fn change_password()
{
    if !get_password(true).is_empty() {update_file(get_notes(false))};
}

// Checks if a supplied level (page) exists
fn check_level(level: usize, allow_zero: bool) -> usize
{
    if allow_zero && level == 0 {return 0}
    max(1, min(level, get_max_level()))
}

// Gets notes the fall under a certain level
fn get_note_range(level: usize) -> Vec<String>
{
    let mut result: Vec<String> = vec![];
    let notes = get_notes(false);
    let lines: Vec<&str> = notes.lines().collect();
    if lines.is_empty() {return result}

    let a = if level > 1 {((level - 1) * ITEMS_PER_LEVEL) + 1} else {1};
    let b = min(level * ITEMS_PER_LEVEL, lines.len() - 1);
    
    let selected: Vec<&str> = lines[a..=b].iter().copied().collect();
    let mut n = a;

    for line in selected.iter()
    {
        result.push(format_item(n, line)); n += 1;
    }

    result
}

// Gets the maximum number of levels
fn get_max_level() -> usize
{
    let notes_length = get_notes_length();
    let n = notes_length as f64 / ITEMS_PER_LEVEL as f64;
    max(1, n.ceil() as usize)
}

// Goes to the previous page
// It can wrap to the last one
fn cycle_left()
{   
    let lvl: usize;

    {
        let level = LEVEL.lock().unwrap();
        let max_level = get_max_level();
        lvl = if *level <= 1 {max_level} else {*level - 1};
    }

    show_notes(lvl, vec![]);
}

// Goes to the next page
// It can wrap to the first one
fn cycle_right()
{
    let lvl: usize;

    {
        let level = LEVEL.lock().unwrap();
        let max_level = get_max_level();
        lvl = if *level >= max_level {1} else {*level + 1};
    }

    show_notes(lvl, vec![]);
}

// Edits the most recent note
fn edit_last_note()
{
    let n: usize;

    {
        n = *NOTES_LENGTH.lock().unwrap();
    }

    edit_note(n);
}

// Checks a line number from the notes exist
fn check_line_exists(n: usize) -> bool
{
    n > 0 && n <= get_notes_length()
}

// Replaces keywords to note numbers
// Or parses the string to a number
fn parse_note_ans(ans: &str) -> usize
{
    match ans
    {
        "first" => 1,
        "last" => get_notes_length(),
        _ => ans.parse().unwrap_or(0)
    }
}