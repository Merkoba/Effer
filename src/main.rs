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
    event::
    {
        Event, Key, 
        MouseEvent, MouseButton
    },
    input::TermRead,
    raw::IntoRawMode,
    color
};
use regex::Regex;
use clap::{App, Arg};
use rustyline::
{
    Editor, Cmd, KeyPress,
    Config, OutputStreamType, CompletionType, 
    completion::FilenameCompleter
};

type Aes256Cbc = Cbc<Aes256, Pkcs7>;

// First function to execute
fn main() 
{
    check_arguments();
    handle_file_path_check(file_path_check(get_file_path()));
    handle_source(); if get_password(false).is_empty() {exit()};
    let notes = get_notes(false); if notes.is_empty() {exit()}
    update_notes_statics(notes); get_settings(); create_menus(); 
    change_screen(); g_set_started(true);
    
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
        .value_name("Path")
        .help("Sets a custom file path")
        .takes_value(true))
    .arg(Arg::with_name("source")
        .long("source")
        .value_name("Path")
        .help("Creates notes from a text file")
        .takes_value(true))
    .arg(Arg::with_name("page_size")
        .long("page_size")
        .value_name("Multiple of 5")
        .help("Set the page size setting")
        .takes_value(true))
    .arg(Arg::with_name("row_space")
        .long("row_space")
        .value_name("true|false")
        .help("Set the row space setting")
        .takes_value(true))
    .arg(Arg::with_name("color_1")
        .long("color_1")
        .value_name("r,g,b")
        .help("Set the color 1 setting")
        .takes_value(true))
    .arg(Arg::with_name("color_2")
        .long("color_2")
        .value_name("r,g,b")
        .help("Set the color 2 setting")
        .takes_value(true))
    .arg(Arg::with_name("color_3")
        .long("color_3")
        .value_name("r,g,b")
        .help("Set the color 3 setting")
        .takes_value(true))
    .get_matches();

    let path = match matches.value_of("path")
    {
        Some(pth) => s!(pth),
        None => s!(get_default_file_path().to_str().unwrap())
    };
    
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
            if i == 0 {continue}; let s = s!(line);

            match print_mode
            {
                "print" => result.push(format_note(&(i, s), false, 0)),
                "print2" => result.push(s),
                _ => {}
            }
        }

        pp!(result.join("\n")); exit();
    }

    if let Some(path) = matches.value_of("source")
    {
        get_source_content(path);
    }

    if let Some(ps) = matches.value_of("page_size")
    {
        g_set_arg_page_size(s!(ps));
    }

    if let Some(rs) = matches.value_of("row_space")
    {
        g_set_arg_row_space(s!(rs));
    }

    if let Some(c) = matches.value_of("color_1")
    {
        g_set_arg_color_1(s!(c));
    }

    if let Some(c) = matches.value_of("color_2")
    {
        g_set_arg_color_2(s!(c));
    }

    if let Some(c) = matches.value_of("color_3")
    {
        g_set_arg_color_3(s!(c));
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
fn ask_string(message: &str, initial: &str, trim: bool) -> String
{
    if trim
    {
        get_input(message, initial, |a| a.trim().to_string(), String::new, false)
    }

    else
    {
        get_input(message, initial, |a| a, String::new, false)
    }
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
fn encrypt_text(plain_text: &str) -> String
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
fn decrypt_text(encrypted_text: &str) -> String
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
    let mut s = format!("{}{}({}){}", nline, get_color(3), key, get_color(2));
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
            menu_item("Arrows", "Change Pages", true, true, true),
            menu_item("Backspace", "Edit Last", true, false, false),
            menu_item("H", "Show All", true, true, true),
            menu_item("?", "About", true, true, false),
            menu_item("Q", "Exit", true, true, false),
            menu_item("Space", ">", true, false, false)
        ].concat(),
        [
            menu_item("+/-", "Change Page Size", true, true, true),
            menu_item("g", "Goto", true, false, false),
            menu_item("R", "Reset File", true, true, true),
            menu_item("P", "Change Password", true, false, false),
            menu_item("$", "Change Colors", true, true, true),
            menu_item(":", "Screen Saver", true, false, false),
            // menu_item("@", "Color 2", true, true, false),
            // menu_item("#", "Color 3", true, false, false)
        ].concat(),
        [
            menu_item("^", "Change Row Spacing", true, true, true),
            menu_item("s", "Swap", true, false, false),
            menu_item("O", "Open Encrypted File", true, true, true),
            menu_item("T", "Stats", true, false, false),
            menu_item("U", "Add From Source File", true, true, true),
            menu_item("X", "Destroy", true, false, false)
        ].concat()
    ]);
}

// Listens and interprets live keyboard input from the main menu
fn menu_input() -> (MenuAnswer, usize)
{
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}{}{}", 
        color::Bg(color::Rgb(10,10,10)),  color::Fg(color::Rgb(210,210,210)), termion::cursor::Hide).unwrap();
    stdout.flush().unwrap(); let mut data = 0;

    let ans = match stdin.events().next().unwrap().unwrap()
    {
        Event::Key(key) =>
        {
            match key
            {
                Key::Left => MenuAnswer::CycleLeft,
                Key::Right => MenuAnswer::CycleRight,
                Key::Up => MenuAnswer::FirstPage,
                Key::Down => MenuAnswer::LastPage,
                Key::PageUp => MenuAnswer::CycleLeft,
                Key::PageDown => MenuAnswer::CycleRight,
                Key::Home => MenuAnswer::FirstPage,
                Key::End => MenuAnswer::LastPage,
                Key::Esc => MenuAnswer::RefreshPage,
                Key::Backspace => MenuAnswer::EditLastNote,
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
                        'F' => MenuAnswer::FindNotesSuggest,
                        's' => MenuAnswer::SwapNotes,
                        'd' => MenuAnswer::DeleteNotes,
                        'g' => MenuAnswer::GotoPage,
                        'R' => MenuAnswer::ResetFile,
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
                        '$' => MenuAnswer::ChangeColors,
                        ' ' => MenuAnswer::ChangeMenu,
                        _ => MenuAnswer::Nothing
                    }
                },
                Key::Ctrl(ch) =>
                {
                    match ch
                    {
                        'c' => MenuAnswer::Exit,
                        _ => MenuAnswer::Nothing
                    }
                },
                _ => MenuAnswer::Nothing
            }
        },
        Event::Mouse(moe) =>
        {
            match moe
            {
                MouseEvent::Press(btn, _, _) =>
                {
                    match btn
                    {
                        MouseButton::WheelUp => MenuAnswer::CycleLeft,
                        MouseButton::WheelDown => MenuAnswer::CycleRight,
                        _ => MenuAnswer::Nothing
                    }
                }
                _ => MenuAnswer::Nothing
            }
        },
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
        MenuAnswer::FindNotes => find_notes(false),
        MenuAnswer::FindNotesSuggest => find_notes(true),
        MenuAnswer::SwapNotes => swap_notes(),
        MenuAnswer::DeleteNotes => delete_notes(),
        MenuAnswer::ResetFile => reset_file(),
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
        MenuAnswer::ChangeColors => change_colors(),
        MenuAnswer::MoveNotes => move_notes(),
        MenuAnswer::Exit => exit(),
        MenuAnswer::Nothing => {}
    }
}

// Main renderer function
// Shows the notes and the menu at the bottom
// Then waits and reacts for input
fn show_notes(mut page: usize, notes: Vec<(usize, String)>, message: String)
{
    loop
    {
        // Clear the screen
        p!("{}\x1b[2J", get_color(1));

        page = check_page_number(page, true);
        
        if page > 0
        {
           print_notes(&get_page_notes(page));
        }

        else
        {
            print_notes(&notes)
        }

        if page > 0
        {
            g_set_page(page); show_page_indicator(page);
        }

        else if !message.is_empty() {p!(format!("\n{}", message))}

        let cm = g_get_current_menu();
        let menu = g_get_menus_item(cm);
        p!(menu); menu_action(menu_input());
    }
}

// Prints notes to the screen
fn print_notes(notes: &Vec<(usize, String)>)
{
    let space = g_get_row_space();
    let padding = calculate_padding(&notes);
            
    for note in notes.iter() 
    {
        if space {p!("")}
        p!(format_note(note, true, padding));
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
    let mut update = false; 

    let re = Regex::new(r"page_size=(?P<page_size>\d+)").unwrap();
    let cap = re.captures(header);
    let arg = g_get_arg_page_size();
    let arg_empty = arg.is_empty();

    if cap.is_some() || !arg_empty
    {
        let s = if !arg_empty {arg} 
        else {s!(cap.unwrap()["page_size"])};
        let num = s.parse::<usize>().unwrap_or(DEFAULT_PAGE_SIZE);
        let mut n5 = (5.0 * (num as f64 / 5.0).round()) as usize;
        if n5 <= 0 {n5 = 5} else if n5 > MAX_PAGE_SIZE {n5 = MAX_PAGE_SIZE};
        if arg_empty && n5 != g_get_page_size() {update=true}
        g_set_page_size(n5);
    }

    else
    {
        g_set_page_size(DEFAULT_PAGE_SIZE);
    }

    let re = Regex::new(r"row_space=(?P<row_space>\w+)").unwrap();
    let cap = re.captures(header);
    let arg = g_get_arg_row_space();
    let arg_empty = arg.is_empty();

    if cap.is_some() || !arg_empty
    {
        let s = if !arg_empty {arg} 
        else {s!(cap.unwrap()["row_space"])};
        let rs = FromStr::from_str(&s).unwrap_or(DEFAULT_ROW_SPACE);
        if arg_empty && rs != g_get_row_space() {update=true}
        g_set_row_space(rs);
    }

    else
    {
        g_set_row_space(DEFAULT_ROW_SPACE);
    }

    let re = Regex::new(r"color_1=(?P<color_1>\d+,\d+,\d+)").unwrap();
    let cap = re.captures(header);
    let arg = g_get_arg_color_1();
    let arg_empty = arg.is_empty();

    if cap.is_some() || !arg_empty
    {
        let s = if !arg_empty {arg} 
        else {s!(cap.unwrap()["color_1"])};

        let v: Vec<u8> = s.split(",")
            .map(|n| n.parse::<u8>()
            .unwrap_or(0))
            .collect();

        let c = if v.len() != 3 {DEFAULT_COLOR_1}
        else {(v[0], v[1], v[2])};
        if arg_empty && c != g_get_color_1() {update=true}
        g_set_color_1(c);
    }

    else
    {
        g_set_color_1(DEFAULT_COLOR_1);
    }

    let re = Regex::new(r"color_2=(?P<color_2>\d+,\d+,\d+)").unwrap();
    let cap = re.captures(header);
    let arg = g_get_arg_color_2();
    let arg_empty = arg.is_empty();

    if cap.is_some() || !arg_empty
    {
        let s = if !arg_empty {arg} 
        else {s!(cap.unwrap()["color_2"])};

        let v: Vec<u8> = s.split(",")
            .map(|n| n.parse::<u8>()
            .unwrap_or(0))
            .collect();

        let c = if v.len() != 3 {DEFAULT_COLOR_2}
        else {(v[0], v[1], v[2])};
        if arg_empty && c != g_get_color_2() {update=true}
        g_set_color_2(c);
    }

    else
    {
        g_set_color_2(DEFAULT_COLOR_2);
    }

    let re = Regex::new(r"color_3=(?P<color_3>\d+,\d+,\d+)").unwrap();
    let cap = re.captures(header);
    let arg = g_get_arg_color_3();
    let arg_empty = arg.is_empty();

    if cap.is_some() || !arg_empty
    {
        let s = if !arg_empty {arg} 
        else {s!(cap.unwrap()["color_3"])};

        let v: Vec<u8> = s.split(",")
            .map(|n| n.parse::<u8>()
            .unwrap_or(0))
            .collect();

        let c = if v.len() != 3 {DEFAULT_COLOR_3}
        else {(v[0], v[1], v[2])};
        if arg_empty && c != g_get_color_3() {update=true}
        g_set_color_3(c);
    }

    else
    {
        g_set_color_3(DEFAULT_COLOR_3);
    }

    if update {update_header()}
}

// Resets settings to default state
fn reset_settings()
{
    g_set_page_size(DEFAULT_PAGE_SIZE);
    g_set_row_space(DEFAULT_ROW_SPACE);
    g_set_color_1(DEFAULT_COLOR_1);
    g_set_color_2(DEFAULT_COLOR_2);
    g_set_color_3(DEFAULT_COLOR_3);
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
    let note = ask_string("Add Note", "", false);
    if note.is_empty() {return}
    let new_text = format!("{}\n{}", get_notes(false), note);
    update_file(new_text); g_set_last_edit(g_get_notes_length());
    goto_last_page();
}

// Asks for a note number to edit
// The note is then showed and editable
fn edit_note(mut n: usize)
{
    if n == 0
    {
        let last_edit = g_get_last_edit();
        let suggestion = if last_edit == 0 {s!()} 
        else {expand_note_number(last_edit)};
        n = parse_note_ans(&ask_string("Edit #", &(suggestion), true));
    }

    if !check_line_exists(n) {return}
    let edited = ask_string("Edit Note", &get_line(n), false);
    if edited.is_empty() {return} 
    g_set_last_edit(n);
    replace_line(n, edited);
    show_page(get_note_page(n));
}

// Finds a note by a filter
// Case insensitive
// Substrings are counted
fn find_notes(suggest: bool)
{
    pp!("Enter Filter | "); p!("Or Regex (re:\\d+)");
    let last_find = g_get_last_find();
    let suggestion = if suggest && !last_find.is_empty() {&last_find} else {""};
    let filter = ask_string("Find", suggestion, true).to_lowercase();
    let mut found: Vec<(usize, String)> = vec![];
    if filter.is_empty() {return} let notes = get_notes(false);
    let info = format!("{}{}{}{} >", get_color(3), filter, RESET_FG_COLOR, get_color(2));

    if filter.starts_with("re:")
    {
        if let Ok(re) = Regex::new(format!("(?i){}", filter.replacen("re:", "", 1)).trim())
        {
            for (i, line) in notes.lines().enumerate()
            {
                if i == 0 {continue}
                if re.is_match(line) {found.push((i, s!(line)))}
            }
        }

        else
        {
            return show_message(&format!("< Invalid Regex: {}", info));
        }
    }

    else
    {
        let ifilter = filter.to_lowercase();

        for (i, line) in notes.lines().enumerate()
        {
            if i == 0 {continue}
            if line.to_lowercase().contains(&ifilter) {found.push((i, s!(line)))}
        }
    }

    g_set_last_find(filter); let mut message;

    if found.is_empty()
    {
        message = s!("< No Results for ");
    }

    else if found.len() == 1
    {
        message = s!("< 1 Result for ");
    }

    else
    {
        message = format!("< {} Results for ", found.len());
    }

    message += &info; show_notes(0, found, message);
}

// Swaps 2 notes specified by 2 numbers separated by whitespace (1 10)
fn swap_notes()
{
    let ans = ask_string("Swap (n1 n2)", "", true);
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
    pp!("Enter Note # | ");
    p!("Or List (1,2,3)");
    pp!("Or Range (1-3) | ");
    p!("Or Regex (re:\\d+)");

    let ans = ask_string("Delete", "", true);
    if ans.is_empty() {return}
    let mut numbers: Vec<usize> = vec![];
    let notes = get_notes(false);

    fn nope()
    {
        show_message("< No Messages Were Deleted >")
    }

    if ans.starts_with("re:")
    {
        if let Ok(re) = Regex::new(format!("(?i){}", ans.replacen("re:", "", 1)).trim())
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
        if ans.matches('-').count() > 1 {return nope()}
        let note_length = g_get_notes_length();
        let mut split = ans.split('-').map(|n| n.trim());
        let num1 = parse_note_ans(split.next().unwrap_or("0"));
        let mut num2 = parse_note_ans(split.next().unwrap_or("0"));
        if num1 == 0 || num2 == 0 {return nope()}
        if num2 > note_length {num2 = note_length}
        if num1 >= num2 {return nope()}
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
        return nope()
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
fn format_note(note: &(usize, String), colors: bool, padding: usize) -> String
{
    let mut pad = s!();

    if padding > 0
    {
        let len = note.0.to_string().len();

        if len < padding
        {
            for _ in 0..(padding  - len)
            {
                pad += " ";
            }
        }
    }

    if colors
    {
        format!("{}({}) {}{}{}", get_color(3), note.0, pad, get_color(2), note.1)
    }

    else
    {
        format!("({}) {}{}", note.0, pad, note.1)
    }
}

// Asks the user if it wants to remake the file
// Or just reset the settings to default
// Then does it if response was positive
// Variables are then updated to reflect change
fn reset_file()
{
    p!("(f) Remake File | (s) Reset Settings");
    let ans = ask_string("Choice", "", true);

    if ans == "f"
    {
        if ask_bool("Are you sure you want to replace the file with an empty one?", true)
        {
            fs::remove_file(get_file_path()).unwrap();
            if !create_file() {return}
            reset_state(get_notes(true));
        }
    }

    else if ans == "s"
    {
        if ask_bool("Are you sure you want to reset settings to default?", true)
        {
            reset_settings(); update_header(); create_menus();
        }
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

    let name = format!("Effer {} | Encrypted Notepad", VERSION);

    fn make_tip(s: &str) -> String
    {
        format!("{}Tip:{} {}{}", get_color(3), RESET_FG_COLOR, get_color(2), s)
    }

    let info =
    [
        make_tip("Different major versions are not compatible"),
        make_tip("You can use 'first' and 'last' as note numbers"),
        make_tip("1-9 can be used to navigate the first 9 pages"),
        make_tip("Start the program with --help to check arguments"),
        make_tip("Shift+F uses the last find filter")
    ].join("\n");

    let s = format!("{}\n\n{}\n\n{}", art,  name, info);

    show_message(&s);
}

// Asks the user to input a page number to go to
fn goto_page()
{
    let n = parse_page_ans(&ask_string("Page #", "", true));
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
        if page_size < MAX_PAGE_SIZE && max_page > 1 {g_set_page_size(page_size + 5)} else {return}
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
    let c1 = g_get_color_1();
    let sc1 = format!("{},{},{}", c1.0, c1.1, c1.2);
    let c2 = g_get_color_2();
    let sc2 = format!("{},{},{}", c2.0, c2.1, c2.2);
    let c3 = g_get_color_3();
    let sc3 = format!("{},{},{}", c3.0, c3.1, c3.2);

    let s = format!("{} page_size={} row_space={} color_1={} color_2={} color_3={}", 
        uc, ps, rs, sc1, sc2, sc3);

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
    let path = shell_contract(&g_get_path().to_string());

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
        let ans = ask_string("Source: (r) Replace | (a) Append | (p) Prepend", "", true);

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
    let ans = ask_string("Source Path", "", true);
    if ans.is_empty() {return}
    get_source_content(&ans);
    handle_source();
}

// Changes the current notes file with another one
fn open_from_path()
{
    let ans = ask_string("Encrypted File Path", "", true);
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

// Replaces /home/user with ~ if it's user's Home
fn shell_contract(path: &str) -> String
{
    let hpath =  get_home_path();
    let home = hpath.to_str().unwrap();

    if path.starts_with(home)
    {
        s!(path.replacen(home, "~", 1))
    }

    else
    {
        s!(path)
    }
}

// Enables or disables spacing between notes
fn change_row_space()
{
    g_set_row_space(!g_get_row_space());
    update_header();
}

// Changes some color to the next one
fn change_colors()
{

    p!("(1) Background | (2) Foreground (3) Other");
    let ans = ask_string("Choice", "", true);
    if ans.is_empty() {return};
    let n = ans.parse::<usize>().unwrap_or(0);
    if n < 1 || n > 3 {return}
    let ans = ask_string("Color (r,g,b)", "", true);
    if ans.is_empty() {return}

    let v: Vec<u8> = ans.split(",")
        .map(|s| s.trim())
        .map(|n| n.parse::<u8>().unwrap_or(0)).collect();

    if v.len() != 3 {return}
    
    match n
    {
        1 => g_set_color_1((v[0], v[1], v[2])),
        2 => g_set_color_2((v[0], v[1], v[2])),
        3 => g_set_color_3((v[0], v[1], v[2])),
        _ => {}
    }

    create_menus(); update_header();
}

// Gets the current theme
fn get_color(n: usize) -> String
{
    match n
    {
        1 => 
        {
            let t = g_get_color_1();
            s!(color::Bg(color::Rgb(t.0, t.1, t.2)))
        }
        2 => 
        {
            let t = g_get_color_2();
            s!(color::Fg(color::Rgb(t.0, t.1, t.2)))
        }
        3 => 
        {
            let t = g_get_color_3();
            s!(color::Fg(color::Rgb(t.0, t.1, t.2)))
        }
        _ => s!("")
    }
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
    pp!("From To (n1 n2) | "); p!("Or Range (4-10 2)");

    let ans = ask_string("Move", "", true);
    if ans.is_empty() {return}

    if ans.contains('-')
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

// Shows the page indicator above the menu
fn show_page_indicator(page: usize)
{
    p!(format!("\n< Page {} of {} >\n{}", 
        page, get_max_page_number(), 
        shell_contract(&g_get_path().to_string())));
}

// Changes note numbers to equivalents
// like first and last
fn expand_note_number(n: usize) -> String
{
    if n == 1 {s!("first")}
    else if n == g_get_notes_length() {s!("last")}
    else {s!(n)}
}

// Calculates if some padding 
// must be given between note numbers
// and note text. So all notes look aligned
// Returns the difference and the max length
fn calculate_padding(notes: &Vec<(usize, String)>) -> usize
{
    let mut max = 0;
    let mut len = 0;

    for note in notes.iter()
    {
        let nl = note.0.to_string().len();

        if len == 0 
        {
            len = nl; continue;
        }

        else if nl > len
        {
            len = nl; max = nl;
        }
    }

    max
}