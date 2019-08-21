mod macros;

mod structs;
use structs::
{
    FilePathCheckResult,
    MenuAnswer, RustyHelper
};

mod globals; 
use globals::*;

use std::
{
    fs, process, iter,
    thread, time,
    path::{Path, PathBuf},
    io::{self, Write, stdout, stdin},
    cmp::{min, max},
    str::FromStr
};
use block_modes::
{
    BlockMode, Cbc, 
    block_padding::Pkcs7
};
use aes_soft::Aes256;
use dirs;
use rand::
{
    Rng, thread_rng, prelude::*,
    distributions::Alphanumeric}
;
use sha3::{Sha3_256, Digest};
use termion::
{
    event::Key,
    input::TermRead,
    raw::IntoRawMode
};
use regex::Regex;
use clap::{App, Arg};
use rustyline::
{
    Editor, Cmd, KeyPress,
    Config, OutputStreamType, CompletionType, 
    completion::FilenameCompleter
};
use prettytable::
{
    Table, Row, Cell, 
    format::FormatBuilder
};

type Aes256Cbc = Cbc<Aes256, Pkcs7>;

// First function to execute
fn main() 
{
    check_arguments();
    handle_file_path_check(file_path_check(get_file_path()));
    handle_source(); if get_password(false).is_empty() {exit()};
    let notes = get_notes(false); if notes.is_empty() {exit()}
    update_notes_statics(notes); get_settings(); change_screen();
    create_themes(); create_menus(); g_set_started(true);
    
    // Start loop
    goto_last_page();
}

// Starts the argument system and responds to actions
fn check_arguments()
{
    let matches = App::new("Effer")
    .version(VERSION)
    .about("Encrypted CLI Notepad")
    .arg(Arg::with_name("print")
        .long("print")
        .multiple(false)
        .help("Prints all the notes instead of entering the program"))
    .arg(Arg::with_name("print2")
        .long("print2")
        .multiple(false)
        .help("Same as print but doesn't show the numbers"))
    .arg(Arg::with_name("path")
        .long("path")
        .value_name("PATH")
        .help("Sets a custom file path")
        .takes_value(true))
    .arg(Arg::with_name("source")
        .long("source")
        .value_name("PATH")
        .help("Creates notes from a text file")
        .takes_value(true))
    .get_matches();

    let path = s!(matches.value_of("path")
        .unwrap_or(get_default_file_path().to_str().unwrap()));
    
    g_set_path(shell_expand(&path));

    let mut print_mode = "";

    if matches.occurrences_of("print") > 0
    {
        print_mode = "print";
    }

    else if matches.occurrences_of("print2") > 0
    {
        print_mode = "print2";
    }

    if print_mode == "print" || print_mode == "print2"
    {
        let notes = get_notes(false);
        let lines: Vec<&str> = notes.lines().collect();
        if lines.is_empty() {exit()}
        let mut result: Vec<String> = vec![];

        for (i, line) in lines.iter().enumerate()
        {
            if i == 0 {continue}

            match print_mode
            {
                "print" => result.push(format_item(i, line)),
                "print2" => result.push(s!(line)),
                _ => {}
            }
        }

        pp!(result.join("\n")); exit();
    }

    if let Some(path) = matches.value_of("source")
    {
        get_source_content(path);
    }
}

// Switch to the alternative screen
// Place the cursor at the bottom left
fn change_screen()
{
    p!("\x1b[?1049h"); p!("\x1b[r");
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
            e!("Can't get your Home path."); exit();
        }
    }
}

// Gets the default file path
fn get_default_file_path() -> PathBuf
{
    Path::new(&get_home_path().join(".config/effer/effer.dat")).to_path_buf()
}

// Gets the path of the file
fn get_file_path() -> PathBuf
{
    Path::new(&shell_expand(&g_get_path())).to_path_buf()
}

// Gets the path of the file's parent
fn get_file_parent_path() -> PathBuf
{
    get_file_path().parent().unwrap().to_path_buf()
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
            e!(result.message());
            let answer = ask_bool("Do you want to make the file now?", false);
            if answer {if !create_file() {exit()}} else {exit()}
        },
        FilePathCheckResult::NotAFile =>
        {
            e!(result.message()); e!("Nothing to do. Exiting."); exit();
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
        .output_stream(OutputStreamType::Stderr)
        .completion_type(CompletionType::List)
        .build();

    let h = RustyHelper {masking: mask, completer: FilenameCompleter::new()};
    let mut editor = Editor::with_config(config);
    editor.set_helper(Some(h));
    editor.bind_sequence(KeyPress::Esc, Cmd::Interrupt);
    let prompt = format!("{}: ", message);
    if mask {ee!("{}", termion::cursor::Hide)}

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

    if mask {ee!("{}", termion::cursor::Show)} ans
}

// Asks the user for a yes/no answer
fn ask_bool(message: &str, critical:bool) -> bool
{
    if critical
    {
        get_input(&[message, " (Y, n)"].concat(), "", |a| a.trim() == "Y", || false, false)
    }

    else
    {
        get_input(&[message, " (y, n)"].concat(), "", |a| a.trim().to_lowercase() == "y", || false, false)
    }
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
    let mut pw = g_get_password();

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
                if password != confirmation {e!("Error: Passwords Don't Match.")} else {break}
            }
        }

        else
        {
            password = get_input("Password", "", |a| a, String::new, true);
        }

        pw = s!(password); g_set_password(password);
    }

    pw
}

// Attempts to create the file
// It adds UNLOCK_CHECK as its only initial content
// The content is encrypted using the password
fn create_file() -> bool
{
    if get_password(true).is_empty() {return false}
    let encrypted = encrypt_text(&s!(UNLOCK_CHECK));

    match fs::create_dir_all(get_file_parent_path())
    {
        Ok(_) => {}, Err(_) => {e!("Can't create parent directories."); return false}
    }

    let mut file = match fs::File::create(get_file_path())
    {
        Ok(f) => f, Err(_) => {e!("Error creating the file."); return false}
    };

    match file.write_all(encrypted.as_bytes())
    {
        Ok(_) => {}, Err(_) => {e!("Unable to write initial text to file."); return false}
    }

    true
}

// Encrypts the notes using Aes256
// Turns the encrypted data into hex
fn encrypt_text(plain_text: &String) -> String
{
    let mut hasher = Sha3_256::new(); hasher.input(get_password(false).as_bytes());
    let key = hasher.result(); let iv = generate_iv(&key);

    let cipher = match Aes256Cbc::new_var(&key, &iv)
    {
        Ok(cip) => cip, Err(_) => {e!("Can't init the encrypt cipher."); return s!()}
    };

    let encrypted = cipher.encrypt_vec(plain_text.as_bytes()); hex::encode(&encrypted)
}

// Decodes the hex data and decrypts it
fn decrypt_text(encrypted_text: &String) -> String
{
    if encrypted_text.trim().is_empty() {return s!()}
    let mut hasher = Sha3_256::new(); hasher.input(get_password(false).as_bytes());
    let key = hasher.result(); let iv = generate_iv(&key);

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
        Some(hd) => hd, None => {e!("Can't read last line from the file."); return s!()}
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

// Creates a menu item
fn menu_item(key: &str, label: &str, spacing:bool, separator: bool, newline: bool) -> String
{
    let nline = if newline {"\n"} else {""};
    let mut s = format!("{}{}({}){}", nline, get_current_theme().0, key, RESET);
    if spacing {s += " "}; s += label;
    if separator {s += " | "} s
}

// Creates all the menus and stores them in a global
// Instead of making them on each iteration
fn create_menus()
{
    g_set_menus(vec!
    [
        [
            menu_item("a", "dd", false, true, true),
            menu_item("e", "dit", false, true, false),
            menu_item("f", "ind", false, true, false),
            menu_item("m", "ove", false, true, false),
            menu_item("d", "elete", false, false, false),
            menu_item("Left/Right", "Cycle Pages", true, true, true),
            menu_item("Up", "Edit Last Note", true, false, false),
            menu_item("H", "Show All", true, true, true),
            menu_item("?", "About", true, true, false),
            menu_item("Q", "Exit", true, true, false),
            menu_item("Space", ">", true, false, false)
        ].concat(),
        [
            menu_item("+/-", "Change Page Size", true, true, true),
            menu_item("g", "Goto", true, true, false),
            menu_item("T", "Stats", true, false, false),
            menu_item("R", "Remake File", true, true, true),
            menu_item("P", "Change Password", true, true, false),
            menu_item(":", "😎", true, false, false),
            menu_item("Home", "First Page", true, true, true),
            menu_item("End", "Last Page", true, true, false),
            menu_item("Space", ">", true, false, false)
        ].concat(),
        [
            menu_item("^", "Change Row Spacing", true, true, true),
            menu_item("*", "Change Theme", true, false, false),
            menu_item("O", "Open Other Encrypted File", true, true, true),
            menu_item("X", "Destroy", true, false, false),
            menu_item("U", "Add From Source File", true, true, true),
            menu_item("s", "Swap", true, true, false),
            menu_item("Space", ">", true, false, false)
        ].concat()
    ]);
}

// Listens and interprets live keyboard input from the main menu
fn menu_input() -> (MenuAnswer, usize)
{
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}", termion::cursor::Hide).unwrap();
    stdout.flush().unwrap(); let mut data = 0;

    let ans = match stdin.keys().next().unwrap().unwrap()
    {
        Key::Left => MenuAnswer::CycleLeft,
        Key::PageUp => MenuAnswer::CycleLeft,
        Key::Right => MenuAnswer::CycleRight,
        Key::PageDown => MenuAnswer::CycleRight,
        Key::Up => MenuAnswer::EditLastNote,
        Key::Down => MenuAnswer::LastPage,
        Key::Home => MenuAnswer::FirstPage,
        Key::End => MenuAnswer::LastPage,
        Key::Esc => MenuAnswer::RefreshPage,
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
                'g' => MenuAnswer::GotoPage,
                'R' => MenuAnswer::RemakeFile,
                'P' => MenuAnswer::ChangePassword,
                'H' => MenuAnswer::ShowAllNotes,
                'T' => MenuAnswer::ShowStats,
                '?' => MenuAnswer::ShowAbout,
                'O' => MenuAnswer::OpenFromPath,
                'U' => MenuAnswer::FetchSource,
                'm' => MenuAnswer::MoveNotes,
                'Q' => MenuAnswer::Exit,
                '+' => MenuAnswer::IncreasePageSize,
                '-' => MenuAnswer::DecreasePageSize,
                ':' => MenuAnswer::ScreenSaver,
                'X' => MenuAnswer::Destroy,
                '\n' => MenuAnswer::RefreshPage,
                '^' => MenuAnswer::ChangeRowSpace,
                '*' => MenuAnswer::ChangeTheme,
                ' ' => MenuAnswer::ChangeMenu,
                _ => MenuAnswer::Nothing
            }
        }
        _ => MenuAnswer::Nothing
    };

    write!(stdout, "{}", termion::cursor::Show).unwrap(); 
    stdout.flush().unwrap(); (ans, data)
}

// Reacts to the live keyboard input from the main menu
fn menu_action(ans: (MenuAnswer, usize))
{
    match ans.0
    {
        MenuAnswer::AddNote => add_note(),
        MenuAnswer::EditNote => edit_note(0),
        MenuAnswer::FindNotes => find_notes(),
        MenuAnswer::SwapNotes => swap_notes(),
        MenuAnswer::DeleteNotes => delete_notes(),
        MenuAnswer::RemakeFile => remake_file(),
        MenuAnswer::ChangePassword => change_password(),
        MenuAnswer::CycleLeft => cycle_left(),
        MenuAnswer::CycleRight => cycle_right(),
        MenuAnswer::FirstPage => goto_first_page(),
        MenuAnswer::LastPage => goto_last_page(),
        MenuAnswer::RefreshPage => refresh_page(),
        MenuAnswer::EditLastNote => edit_last_note(),
        MenuAnswer::PageNumber => show_page(max(1, ans.1)),
        MenuAnswer::ChangeMenu => cycle_menu(),
        MenuAnswer::ShowAllNotes => show_all_notes(),
        MenuAnswer::ShowAbout => show_about(),
        MenuAnswer::GotoPage => goto_page(),
        MenuAnswer::IncreasePageSize => change_page_size(true),
        MenuAnswer::DecreasePageSize => change_page_size(false),
        MenuAnswer::ShowStats => show_stats(),
        MenuAnswer::ScreenSaver => show_screensaver(),
        MenuAnswer::OpenFromPath => open_from_path(),
        MenuAnswer::FetchSource => fetch_source(),
        MenuAnswer::Destroy => destroy(),
        MenuAnswer::ChangeRowSpace => change_row_space(),
        MenuAnswer::ChangeTheme => change_theme(),
        MenuAnswer::MoveNotes => move_notes(),
        MenuAnswer::Exit => exit(),
        MenuAnswer::Nothing => {}
    }
}

// Main renderer function
// Shows the notes and the menu at the bottom
// Then waits and reacts for input
fn show_notes(mut page: usize, lines: Vec<(usize, String)>, message: String)
{
    loop
    {
        // Clear the screen
        p!("\x1b[2J");

        page = check_page_number(page, true);
        
        if page > 0
        {
            create_table(&get_page_notes(page)).printstd();
        }

        else
        {
            create_table(&lines).printstd();
        }

        if page > 0
        {
            g_set_page(page);
            p!(format!("\n< Page {} of {} >", page, get_max_page_number()));
        }

        else if !message.is_empty() {p!(format!("\n{}", message))}

        let cm = g_get_current_menu();
        let menu = g_get_menus_item(cm);
        p!(menu); menu_action(menu_input());
    }
}

// Reads the file
fn get_file_text() -> String
{
    match read_file(get_file_path().to_str().unwrap())
    {
        Ok(text) => text, Err(_) => {p!("Can't read file content."); return s!()}
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
    let notes = g_get_notes();
    if notes.is_empty() || update {decrypt_text(&get_file_text())} else {notes}
}

// Encrypts and saves the updated notes to the file
fn update_file(text: String)
{
    let encrypted = encrypt_text(&text);
    
    if encrypted.is_empty()
    {
        show_message("< Error Encrypting File >");
        return
    }

    update_notes_statics(text);

    match fs::write(get_file_path(), encrypted.as_bytes())
    {
        Ok(_) => {}, 
        Err(_) => 
        {
            if g_get_started()
            {
                show_message("< Can't Write To File >");
            }

            else
            {
                e!("Unable to write text to file."); exit();
            }
        }
    }
}

// Updates the notes and notes length global variables
fn update_notes_statics(text: String) -> String
{
    g_set_notes_length(text.lines().count() - 1); 
    g_set_notes(text); g_get_notes()
}

// Gets settings from the header
// Sets defaults if they're not defined
fn get_settings()
{
    let notes = get_notes(false);
    let header = notes.lines().nth(0).unwrap();

    let re1 = Regex::new(r"page_size=(?P<page_size>\d+)").unwrap();

    if let Some(caps) = re1.captures(header)
    {
        let ps = &caps["page_size"]; 
        g_set_page_size(ps.parse::<usize>().unwrap_or(DEFAULT_PAGE_SIZE));
    }

    else
    {
        g_set_page_size(DEFAULT_PAGE_SIZE);
    }

    let re2 = Regex::new(r"row_space=(?P<row_space>\w+)").unwrap();

    if let Some(caps) = re2.captures(header)
    {
        let ps = &caps["row_space"]; 
        g_set_row_space(FromStr::from_str(ps).unwrap_or(DEFAULT_ROW_SPACE));
    }

    else
    {
        g_set_row_space(DEFAULT_ROW_SPACE);
    }

    let re3 = Regex::new(r"theme=(?P<theme>\w+)").unwrap();

    if let Some(caps) = re3.captures(header)
    {
        let ps = &caps["theme"]; 
        g_set_theme(ps.parse::<usize>().unwrap_or(DEFAULT_THEME));
    }

    else
    {
        g_set_theme(DEFAULT_THEME);
    }
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
    update_file(lines.join("\n"));
}

// Swaps two lines from the notes
fn swap_lines(n1: usize, n2: usize)
{
    let notes = get_notes(false);
    let mut lines: Vec<&str> = notes.lines().collect();
    lines.swap(n1, n2);

    // If one of the two items is the last edited note
    // Swap it so it points to the correct one
    let  last_edit = g_get_last_edit();
    if last_edit == n1 {g_set_last_edit(n2)}
    else if last_edit == n2 {g_set_last_edit(n1)}

    update_file(lines.join("\n"));
}

// Moves a range of lines to another index
fn move_lines(from: Vec<usize>, to: usize)
{
    let notes = get_notes(false);
    let lines: Vec<&str> = notes.lines().collect();
    let range = &lines[from[0]..=from[1]]; 
    let nto = if to < from[0] {to} else {to - range.len() + 1};
    let first_half = &lines[0..from[0]];
    let second_half = &lines[(from[1] + 1)..];
    let mut joined: Vec<&str> = vec![];
    joined.extend(first_half); joined.extend(second_half);
    joined.splice(nto..nto, range.iter().cloned());
    update_file(joined.join("\n"));
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

    update_file(new_lines.join("\n"));
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
        let last_edit = g_get_last_edit();
        let suggestion = if last_edit == 0 {s!()} else {s!(last_edit)};
        n = parse_note_ans(&ask_string("Edit #", &suggestion));
    }

    if !check_line_exists(n) {return}
    let edited = ask_string("Edit Note", &get_line(n));
    if edited.is_empty() {return} 
    g_set_last_edit(n);
    replace_line(n, edited);
    show_page(get_note_page(n));
}

// Finds a note by a filter
// Case insensitive
// Substrings are counted
fn find_notes()
{
    let filter = ask_string("Regex Filter", "").to_lowercase();
    let mut found: Vec<(usize, String)> = vec![];
    if filter.is_empty() {return}
    let notes = get_notes(false);

    if let Ok(re) = Regex::new(&format!("(?i){}", filter))
    {
        for (i, line) in notes.lines().enumerate()
        {
            if i == 0 {continue}
            if re.is_match(line) {found.push((i, s!(line)))}
        }
    }

    else
    {
        return show_message("< Invalid Regex | (Enter) Return >");
    }

    let mut message;

    if found.is_empty()
    {
        message = s!("< No Results");
    }

    else if found.len() == 1
    {
        message = s!("< 1 Result");
    }

    else
    {
        message = format!("< {} Results", found.len());
    }

    message += " | (Enter) Return >";

    show_notes(0, found, message);
}

// Swaps 2 notes specified by 2 numbers separated by whitespace (1 10)
fn swap_notes()
{
    let ans = ask_string("Swap (n1 n2)", "");
    if ans.is_empty() {return}
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
    p!("Enter Note Number");
    p!("Or Note List (1,2,3)");
    p!("Or Note Range (1-3)");
    p!("Or Regex Filter (re:\\d+)");

    let ans = ask_string("Delete", "");
    if ans.is_empty() {return}
    let mut numbers: Vec<usize> = vec![];
    let notes = get_notes(false);

    if ans.starts_with("re:")
    {
        if let Ok(re) = Regex::new(format!("(?i){}", ans.replace("re:", "")).trim())
        {
            for (i, line) in notes.lines().enumerate()
            {
                if i == 0 {continue}
                if re.is_match(line) {numbers.push(i)}
            }
        }

        else 
        {
            return show_message("< Invalid Regex | (Enter) Return >");
        }
    }

    else if ans.contains(',')
    {
        numbers.extend(ans.split(',').map(|n| parse_note_ans(n.trim())).collect::<Vec<usize>>());
    }

    else if ans.contains('-')
    {
        if ans.matches('-').count() > 1 {return}
        let note_length = g_get_notes_length();
        let mut split = ans.split('-').map(|n| n.trim());
        let num1 = parse_note_ans(split.next().unwrap_or("0"));
        let mut num2 = parse_note_ans(split.next().unwrap_or("0"));
        if num1 == 0 || num2 == 0 {return}
        if num2 > note_length {num2 = note_length}
        if num1 >= num2 {return}
        numbers.extend(num1..=num2);
    }

    else
    {
        numbers.push(parse_note_ans(&ans));
    }

    numbers = numbers.iter().filter(|n| check_line_exists(**n)).copied().collect();
    let length = numbers.len();

    if length >= 5
    {
        if !ask_bool(&format!("Are you sure you want to delete {} notes?", length), false)
        {
            return;
        }
    }

    if numbers.is_empty()
    {
        return show_message("< No Messages Were Deleted >")
    }

    // If the deleted not is the last edit
    // reset last edit since it's now invalid
    let last_edit = g_get_last_edit();
    if numbers.contains(&last_edit) {g_set_last_edit(0)}
    delete_lines(numbers);
}

// Goes to the first page
fn goto_first_page()
{
    show_page(1);
}

// Goes to the last page
fn goto_last_page()
{
    show_page(get_max_page_number());
}

// Refreshes the current page (notes, menu, etc)
// This doesn't provoke a change unless on a different mode like Find results
fn refresh_page()
{
    show_page(g_get_page());
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
    if ask_bool("Are you sure you want to replace the file with an empty one?", true)
    {
        fs::remove_file(get_file_path()).unwrap();
        if !create_file() {return}
        reset_state(get_notes(true));
    }
}

// Changes the password and updates the file with it
fn change_password()
{
    if !get_password(true).is_empty() {update_file(get_notes(false))};
}

// Checks if a supplied page exists
fn check_page_number(page: usize, allow_zero: bool) -> usize
{
    if allow_zero && page == 0 {return 0}
    max(1, min(page, get_max_page_number()))
}

// Gets notes that belong to a certain page
fn get_page_notes(page: usize) -> Vec<(usize, String)>
{
    let mut result: Vec<(usize, String)> = vec![];
    let notes = get_notes(false);
    let lines: Vec<&str> = notes.lines().collect();
    if lines.is_empty() {return result}
    let page_size = g_get_page_size();
    let a = if page > 1 {((page - 1) * page_size) + 1} else {1};
    let b = min(page * page_size, lines.len() - 1);
    result = (a..).zip(lines[a..=b].iter().map(|x| s!(x))).collect();
    result
}

// Creates a table to display notes
fn create_table(items: &Vec<(usize, String)>) -> prettytable::Table
{
    let mut table = Table::new();

    let format = FormatBuilder::new()
        .padding(0, 1)
        .build();

    table.set_format(format);
    let theme = &get_current_theme().1;

    for item in items.iter()
    {
        let space = g_get_row_space();

        if space
        {
            table.add_row(Row::new(vec!
            [
                Cell::new("").style_spec(""),
                Cell::new("").style_spec("")
            ]));
        }

        table.add_row(Row::new(vec!
        [
            Cell::new(&format!("({})", item.0)).style_spec(theme),
            Cell::new(&item.1).style_spec("")
        ]));
    }

    table
}

// Gets the maximum number of pages
fn get_max_page_number() -> usize
{
    let notes_length = g_get_notes_length();
    let n = notes_length as f64 / g_get_page_size() as f64;
    max(1, n.ceil() as usize)
}

// Goes to the previous page
fn cycle_left()
{   
    let page = g_get_page();
    if page == 1 {return}
    show_page(page - 1);
}

// Goes to the next page
fn cycle_right()
{
    let page = g_get_page();
    let max_page = get_max_page_number();
    if page == max_page {return}
    show_page(page + 1);
}

// Edits the most recent note
fn edit_last_note()
{
    edit_note(g_get_notes_length());
}

// Checks a line number from the notes exist
fn check_line_exists(n: usize) -> bool
{
    n > 0 && n <= g_get_notes_length()
}

// Replaces keywords to note numbers
// Or parses the string to a number
fn parse_note_ans(ans: &str) -> usize
{
    match ans
    {
        "first" => 1,
        "last" => g_get_notes_length(),
        _ => ans.parse().unwrap_or(0)
    }
}

// Replaces keywords to page numbers
// Or parses the string to a number
fn parse_page_ans(ans: &str) -> usize
{
    match ans
    {
        "first" => 1,
        "last" => get_max_page_number(),
        _ => ans.parse().unwrap_or(0)
    }
}

// Changes to the next menu
// Wraps if at the end
fn cycle_menu()
{
    let mlength = g_get_menus_length();
    let menu = g_get_current_menu();
    if menu >= (mlength -1) {g_set_current_menu(0)} 
    else {g_set_current_menu(menu + 1)}
    refresh_page();
}

// Shows all notes at once
fn show_all_notes()
{
    let notes = get_notes(false);
    let lines: Vec<&str> = notes.lines().collect();
    let mut notes: Vec<(usize, String)> = vec![];

    for (i, line) in lines.iter().enumerate()
    {
        if i == 0 {continue}
        notes.push((i, s!(line)));
    }

    show_notes(0, notes, s!());
}

// Information about the program
fn show_about()
{
    let art = 
r#"
_________   _________
____/      452\ /     453 \____
/| ------------- |  ------------ |\
||| ------------- | ------------- |||
||| ------------- | ------------- |||
||| ------- ----- | ------------- |||
||| ------------- | ------------- |||
||| ------------- | ------------- |||
|||  ------------ | ----------    |||
||| ------------- |  ------------ |||
||| ------------- | ------------- |||
||| ------------- | ------ -----  |||
||| ------------  | ------------- |||
|||_____________  |  _____________|||
L/_____/--------\\_//W-------\_____\J"#;

    let name = format!("Effer {}", VERSION);

    let info =
    [
        "Info: Different major versions are not compatible",
        "\nTip: You can use 'first' and 'last' as note numbers",
        "\nTip: 1-9 can be used to navigate the first 9 pages",
        "\nTip: Start the program with --help to check arguments"
    ].concat();

    let s = format!("{}\n\n{}\n\n{}", art, name, info);
    show_message(&s);
}

// Asks the user to input a page number to go to
fn goto_page()
{
    let n = parse_page_ans(&ask_string("Page #", ""));
    if n < 1 || n > get_max_page_number() {return}
    show_page(n);
}

// Changes how many items appear per page
// It increases or decreases by 5
// Maximum number is 100
// Minimum number is 5
fn change_page_size(increase: bool)
{
    let max_page = get_max_page_number();
    let page_size = g_get_page_size();

    if increase
    {
        if page_size < 100 && max_page > 1 {g_set_page_size(page_size + 5)} else {return}
    }
    
    else
    {
        if page_size >= 10 {g_set_page_size(page_size - 5)} else {return}
    }

    update_header();
}

// Modifies the header (first line) with new settings
fn update_header()
{   
    let uc = UNLOCK_CHECK;
    let ps = g_get_page_size();
    let rs = g_get_row_space();
    let th = g_get_theme();

    let s = format!("{} page_size={} row_space={} theme={}", uc, ps, rs, th);

    replace_line(0, s);
}

// Generic function to show a message instead of notes
fn show_message(message: &str)
{
    show_notes(0, vec![], s!(message));
}

// Show some statistics
fn show_stats()
{   
    let notes = get_notes(false);
    let len = g_get_notes_length();
    let mut wcount = 0;
    let mut lcount = 0;

    for (i, line) in notes.lines().enumerate()
    {   
        if i == 0 {continue}
        wcount += line.split_whitespace().count();
        lcount += line.chars().filter(|c| *c != ' ').count();
    }

    let enc_size = get_file_text().as_bytes().len();
    let dec_size = notes.as_bytes().len();
    let path = g_get_path();

    let s = format!("Stats For: {}\n\nNotes: {}\nWords: {}\nLetters: {}\nEncrypted Size: {} Bytes\nDecrypted Size: {} Bytes", 
        path, len, wcount, lcount, enc_size, dec_size);

    show_message(&s);
}

// Hides notes from the screen with some characters
fn show_screensaver()
{
    let mut lines: Vec<String> = vec![];

    for _ in 0..7
    {
        lines.push(s!(("😎 ".repeat(14)).trim()));
    }

    let message = lines.join("\n\n");
    show_message(&message);
}

// Tries to get the content of a source path
fn get_source_content(path: &str)
{
    match read_file(path)
    {
        Ok(text) => 
        {
            g_set_source(if text.is_empty() {s!()} else {text});
        }
        Err(_) => 
        {
            if g_get_started()
            {
                g_set_source(s!());
                show_message("< Invalid Source Path >");
            }

            else
            {
                e!("Invalid source path."); exit();
            }
        }
    }
}

// What to do when a source path is given 
// Either replaces, appends, or prepends notes
// using the source file lines
fn handle_source()
{
    let source = g_get_source();
    if source.is_empty() {return}
    let notes = get_notes(false);
    let started = g_get_started();

    // If there are no notes just fill it with source
    if notes.lines().count() == 1
    {
        let mut lines: Vec<&str> = vec![notes.lines().nth(0).unwrap()];
        lines.extend(source.lines().filter(|s| !s.trim().is_empty())); 
        update_file(lines.join("\n")); if started {goto_last_page()}
    }

    // If notes already exist ask what to do
    else
    {
        let ans = ask_string("Source: (r) Replace | (a) Append | (p) Prepend", "");

        match &ans[..]
        {
            // Replace
            "r" =>
            {
                if ask_bool("Are you sure you want to replace everything?", true)
                {
                    let mut lines: Vec<&str> = vec![notes.lines().nth(0).unwrap()];
                    lines.extend(source.lines().filter(|s| !s.trim().is_empty())); 
                    update_file(lines.join("\n")); g_set_last_edit(0);
                    if started {goto_last_page()}
                }
            },
            // Append
            "a" =>
            {
                let mut lines: Vec<&str> = notes.lines().collect();
                lines.extend(source.lines().filter(|s| !s.trim().is_empty())); 
                update_file(lines.join("\n")); if started {goto_last_page()}
            },
            // Prepend
            "p" =>
            {
                let mut lines = notes.lines();
                let mut xlines = vec![lines.next().unwrap()];
                let nlines: Vec<&str> = source.lines()
                    .filter(|s| !s.trim().is_empty()).collect();
                let olines: Vec<&str> = lines.collect();
                xlines.extend(nlines); xlines.extend(olines); 
                update_file(xlines.join("\n")); g_set_last_edit(0);
                if started {goto_first_page()}
            },
            _ => {}
        }
    }

    g_set_source(s!());
}

// Uses a source from within the program
fn fetch_source()
{
    let ans = ask_string("Source Path", "");
    if ans.is_empty() {return}
    get_source_content(&ans);
    handle_source();
}

// Changes the current notes file with another one
fn open_from_path()
{
    let ans = ask_string("Encrypted File Path", "");
    if ans.is_empty() {return}; let pth = shell_expand(&ans);

    match file_path_check(Path::new(&pth).to_path_buf())
    {
        FilePathCheckResult::Exists =>
        {
            do_open_path(pth, false);
        },
        FilePathCheckResult::DoesNotExist =>
        {
            e!("File doesn't exist.");
            
            if ask_bool("Do you want to make the file now?", false)
            {
                do_open_path(pth, true)
            }
        },
        FilePathCheckResult::NotAFile =>
        {
            show_message("< Path Is Not A File >");
        }
    }
}

// Does the open from path action
fn do_open_path(pth: String, create: bool)
{
    let opassword; let opath;

    let password = g_get_password();
    opassword = password; g_set_password(s!());

    let path = g_get_path();
    opath = path; g_set_path(pth);

    if create
    {
        if !create_file()
        {
            g_set_password(opassword); g_set_path(opath);
            show_message("< Can't Create File >");
            return
        }
    }

    let notes = get_notes(true);

    if notes.is_empty()
    {
        g_set_password(opassword); g_set_path(opath);
        show_message("< Can't Decrypt File >");
    }
            
    else
    {
        reset_state(notes); goto_last_page();
    }
}

// Writes gibberish to the file several times
// Then deletes the file
// Then exists the program
fn destroy()
{
    if ask_bool("Are you sure you want to destroy this file and exit?", true)
    {
        p!("Destroying...");

        let path = get_file_path();

        for i in 1..=10
        {
            match fs::write(&path, gibberish(10_000))
            {
                Ok(_) => 
                {
                    if i < 10
                    {
                        // Maybe there's not a good reason for this
                        // But the idea is to let the file system assimilate the write
                        // In case there's some debouncer system in place
                        thread::sleep(time::Duration::from_millis(500));
                    }
                },
                Err(_) => {e!("Can't overwrite file."); break}
            }
        }

        match fs::remove_file(&path)
        {
            Ok(_) => exit(),
            Err(_) => show_message("< Can't Destroy File >")
        }
    }
}

// Creates random text
fn gibberish(n: usize) -> String
{
    let mut rng = thread_rng();

    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(n)
        .collect::<String>()
}

// Generic function to read text from files
fn read_file(path: &str) -> Result<String, io::Error>
{
    fs::read_to_string(Path::new(&shell_expand(path)))
}

// Replaces ~ and env variables to their proper form
fn shell_expand(path: &str) -> String
{
    s!(*shellexpand::full(path).unwrap())
}

// Enables or disables spacing between notes
fn change_row_space()
{
    g_set_row_space(!g_get_row_space());
    update_header();
}

// Changes the theme to the next one
fn change_theme()
{
    let len = g_get_themes_length(); let theme = g_get_theme();
    if theme >= (len - 1) {g_set_theme(0)} else {g_set_theme(theme + 1)};
    create_menus(); update_header();
}

// Gets the current theme
fn get_current_theme() -> (String, String)
{
    g_get_themes_item(g_get_theme())
}

// Creates the list of available themes
fn create_themes()
{
    g_set_themes(vec!
    [
        // Magenta
        (s!("\x1b[1;35m"), s!("bFm")),
        // Red
        (s!("\x1b[1;31m"), s!("bFr")),
        // Blue
        (s!("\x1b[1;34m"), s!("bFb")),
        // Green
        (s!("\x1b[1;32m"), s!("bFg")),
        // Yellow
        (s!("\x1b[1;33m"), s!("bFy")),
        // Cyan
        (s!("\x1b[1;36m"), s!("bFc")),
        // White
        (s!("\x1b[1;97m"), s!("bFw")),
        // Black
        (s!("\x1b[1;30m"), s!("bFd")),
    ]);
}

// Show notes from a certain page
fn show_page(n: usize)
{
    show_notes(n, vec![], s!());
}

// Gets the page number where a note belongs to
fn get_note_page(n: usize) -> usize
{
    (n as f64 / g_get_page_size() as f64).ceil() as usize
}

// Resets some properties to defaults
// This is used when the file changes
fn reset_state(notes: String)
{
    update_notes_statics(notes);
    get_settings(); create_menus();
    g_set_last_edit(0);
}

// Asks for a range or single note
// and a destination. The moves it
fn move_notes()
{
    p!("From To (n1 n2)");
    p!("Or Range (4-10 2)");

    let ans = ask_string("Move", "");
    if ans.is_empty() {return}

    if ans.contains("-")
    {
        if ans.matches('-').count() > 1 {return}
        let note_length = g_get_notes_length();
        let mut split = ans.split('-').map(|n| n.trim());
        let num1 = parse_note_ans(split.next().unwrap_or("0"));
        let right_side = split.next().unwrap_or("nothing");
        let mut split_right = right_side.split_whitespace().map(|n| n.trim());
        let mut num2 = parse_note_ans(split_right.next().unwrap_or("0"));
        let dest = parse_note_ans(split_right.next().unwrap_or("0"));
        if num1 == 0 || num2 == 0 || dest == 0 {return}
        if num2 > note_length {num2 = note_length}
        if num1 >= num2 {return} 
        if !check_line_exists(dest) {return}
        if dest >= num1 && dest <= num2 {return}
        move_lines(vec![num1, num2], dest);
    }

    else
    {
        let mut split = ans.split_whitespace().map(|n| n.trim());
        let num1 = parse_note_ans(split.next().unwrap_or("0"));
        let dest = parse_note_ans(split.next().unwrap_or("0"));
        if num1 == 0 || dest == 0 {return}
        if !check_line_exists(num1) {return}
        if !check_line_exists(dest) {return}
        if num1 == dest {return} 
        move_lines(vec![num1, num1], dest);
    }
}