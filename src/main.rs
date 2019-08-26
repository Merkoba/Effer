#![allow(clippy::suspicious_else_formatting)]
#![allow(clippy::collapsible_if)]

mod macros;

mod structs;
use structs::
{
    FilePathCheckResult,
    MenuAnswer, RustyHelper,
    Settings
};

mod globals;
use globals::*;

mod colors;
use colors::
{
    parse_color, random_color,
    color_to_string
};

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
    screen, color
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
    check_arguments(); // <-- It might exit here
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
    .arg(Arg::with_name("no-colors")
        .long("no-colors")
        .multiple(false)
        .help("Disables the custom color theme"))
    .arg(Arg::with_name("config")
        .long("config")
        .value_name("Path")
        .help("Use a config TOML file to import settings")
        .takes_value(true))
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
        let lines = get_notes_vec();
        if lines.is_empty() {exit()}

        for (i, line) in lines.iter().enumerate()
        {
            if i == 0 {continue}; let s = s!(line);

            match print_mode
            {
                "print" => p!(format_note(&(i, s), false, 0)),
                "print2" => p!(s),
                _ => {}
            }
        }

        exit();
    }

    if matches.occurrences_of("no-colors") > 0
    {
        g_set_use_colors(false);
    }

    else
    {
        g_set_use_colors(true);
    }

    if let Some(path) = matches.value_of("source")
    {
        get_source_content(path);
    }

    // Settings

    if let Some(path) = matches.value_of("config")
    {
        if let Ok(text) = read_file(path)
        {
            if let Ok(tom) = toml::from_str(&text)
            {
                let sets: Settings = tom;

                if let Some(ps) = sets.page_size
                {
                    g_set_arg_page_size(ps);
                }

                if let Some(rs) = sets.row_space
                {
                    g_set_arg_row_space(rs);
                }

                if let Some(c) = sets.color_1
                {
                    g_set_arg_color_1(c);
                }

                if let Some(c) = sets.color_2
                {
                    g_set_arg_color_2(c);
                }

                if let Some(c) = sets.color_3
                {
                    g_set_arg_color_3(c);
                }
            }
        }
    }

    else
    {
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
}

// Switch to the alternative screen
// Place the cursor at the bottom left
fn change_screen()
{
    p!("{}", screen::ToAlternateScreen);
    g_set_altscreen(true);
    let size = termion::terminal_size().unwrap();
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}", termion::cursor::Goto(1, size.1)).unwrap();
}

// Centralized function to exit the program
// Switches back to main screen before exiting
fn exit() -> !
{
    if g_get_altscreen()
    {
        p!("{}", screen::ToMainScreen);
    }

    process::exit(0)
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
// Can make typing invisible for cases like password input
fn get_input(message: &str, initial: &str, mask: bool) -> String
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
        Ok(input) => input,
        Err(_) => s!()
    };

    if mask {ee!("{}", termion::cursor::Show)} ans
}

// Asks the user for a yes/no answer
fn ask_bool(message: &str, critical:bool) -> bool
{
    let prompt = if critical {" (Y, n)"} else {" (y, n)"};

    loop
    {
        let ans = get_input(&[message, prompt].concat(), "", false);

        match ans.trim()
        {
            "y" => 
            {
                if !critical {return true}
            },
            "Y" => return true,
            "n" | "N" => return false,
            "" => return false,
            _ => {}
        }
    }
}

// Asks the user to input a string
fn ask_string(message: &str, initial: &str, trim: bool) -> String
{
    let ans = get_input(message, initial, false);
    if trim {s!(ans.trim())} else {ans}
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
fn decrypt_text(encrypted_text: &str) -> String
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
            menu_item("G", "Goto", true, false, false),
            menu_item("R", "Reset File", true, true, true),
            menu_item("P", "Change Password", true, false, false),
            menu_item("$", "Change Colors", true, true, true),
            menu_item(":", "Screen Saver", true, false, false)
        ].concat(),
        [
            menu_item("^", "Change Row Spacing", true, true, true),
            menu_item("S", "Swap", true, false, false),
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
        get_color(4), get_color(5), termion::cursor::Hide).unwrap();

    stdout.flush().unwrap(); let mut data = 0;

    let event = match stdin.events().next()
    {
        Some(ev) =>
        {
            match ev
            {
                Ok(eve) => eve,
                Err(_) => {return (MenuAnswer::Nothing, 0)}
            }
        }
        None => return (MenuAnswer::Nothing, 0)
    };

    let ans = match event
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
                        'm' => MenuAnswer::MoveNotes,
                        'd' => MenuAnswer::DeleteNotes,
                        'F' => MenuAnswer::FindNotesSuggest,
                        'S' => MenuAnswer::SwapNotes,
                        'G' => MenuAnswer::GotoPage,
                        'R' => MenuAnswer::ResetFile,
                        'P' => MenuAnswer::ChangePassword,
                        'H' => MenuAnswer::ShowAllNotes,
                        'T' => MenuAnswer::ShowStats,
                        '?' => MenuAnswer::ShowAbout,
                        'O' => MenuAnswer::OpenFromPath,
                        'U' => MenuAnswer::FetchSource,
                        'Q' => MenuAnswer::Exit,
                        '+' => MenuAnswer::IncreasePageSize,
                        '-' => MenuAnswer::DecreasePageSize,
                        ':' => MenuAnswer::ScreenSaver,
                        'X' => MenuAnswer::Destroy,
                        '\n' => MenuAnswer::ModeAction,
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
        MenuAnswer::ModeAction => mode_action(),
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
        // Clear the screen and sets colors
        println!("{}{}{}", get_color(1), get_color(2), termion::clear::All);

        page = check_page_number(page, true);

        if page > 0
        {
            g_set_mode(s!("notes"));
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

        // Print menu
        p!(g_get_menus_item(g_get_current_menu()));

        // Listen and respond to input
        menu_action(menu_input());
    }
}

// Prints notes to the screen
fn print_notes(notes: &[(usize, String)])
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

    if notes.is_empty() || update 
        {decrypt_text(&get_file_text())} 
    else {notes}
}

// Returns the notes in a vector
fn get_notes_vec() -> Vec<String>
{
    get_notes(false).lines().map(|s| s!(s)).collect()
}

// Returns the file's header
fn get_header() -> String
{
    s!(get_notes(false).lines().nth(0).unwrap())
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
    let header = get_header();
    let mut update = false;

    let re = Regex::new(r"page_size=(?P<page_size>\d+)").unwrap();
    let cap = re.captures(&header);
    let arg = g_get_arg_page_size();
    let arg_empty = arg.is_empty();

    if cap.is_some() || !arg_empty
    {
        let stored_value = if cap.is_some() 
            {s!(cap.unwrap()["page_size"])}
            else {s!()};

        let argx = if arg_empty 
        {
            stored_value
        }

        else 
        {
            update = stored_value != arg; arg
        };

        let num = argx.parse::<usize>().unwrap_or(DEFAULT_PAGE_SIZE);
        let mut mult = 5.0 * (num as f64 / 5.0).round();
        if mult <= 0.0 {mult = 5.0}; let mut value = mult as usize;
        if value > MAX_PAGE_SIZE {value = MAX_PAGE_SIZE};
        g_set_page_size(value);
    }

    else
    {
        g_set_page_size(DEFAULT_PAGE_SIZE);
    }

    let re = Regex::new(r"row_space=(?P<row_space>\w+)").unwrap();
    let cap = re.captures(&header);
    let arg = g_get_arg_row_space();
    let arg_empty = arg.is_empty();

    if cap.is_some() || !arg_empty
    {
        let stored_value = if cap.is_some() 
            {s!(cap.unwrap()["row_space"])}
            else {s!()};

        let argx = if arg_empty 
        {
            stored_value
        }

        else 
        {
            update = stored_value != arg; arg
        };

        let value = FromStr::from_str(&argx).unwrap_or(DEFAULT_ROW_SPACE);
        g_set_row_space(value);
    }

    else
    {
        g_set_row_space(DEFAULT_ROW_SPACE);
    }

    let re = Regex::new(r"color_1=(?P<color_1>\d+,\d+,\d+)").unwrap();
    let cap = re.captures(&header);
    let arg = g_get_arg_color_1();
    let arg_empty = arg.is_empty();

    if cap.is_some() || !arg_empty
    {
        let stored_value = if cap.is_some() 
            {s!(cap.unwrap()["color_1"])}
            else {s!()};

        let argx = if arg_empty 
        {
            stored_value
        }

        else 
        {
            update = stored_value != arg; arg
        };

        let v: Vec<u8> = argx.split(',')
            .map(|s| s.trim())
            .map(|n| n.parse::<u8>()
            .unwrap_or(0))
            .collect();

        let value = if v.len() != 3 {DARK_THEME_COLOR_1}
            else {(v[0], v[1], v[2])};

        g_set_color_1(value);
    }

    else
    {
        g_set_color_1(DARK_THEME_COLOR_1);
    }

    let re = Regex::new(r"color_2=(?P<color_2>\d+,\d+,\d+)").unwrap();
    let cap = re.captures(&header);
    let arg = g_get_arg_color_2();
    let arg_empty = arg.is_empty();

    if cap.is_some() || !arg_empty
    {
        let stored_value = if cap.is_some() 
            {s!(cap.unwrap()["color_2"])}
            else {s!()};

        let argx = if arg_empty 
        {
            stored_value
        }

        else 
        {
            update = stored_value != arg; arg
        };

        let v: Vec<u8> = argx.split(',')
            .map(|s| s.trim())
            .map(|n| n.parse::<u8>()
            .unwrap_or(0))
            .collect();

        let value = if v.len() != 3 {DARK_THEME_COLOR_2}
            else {(v[0], v[1], v[2])};

        g_set_color_2(value);
    }

    else
    {
        g_set_color_2(DARK_THEME_COLOR_2);
    }

    let re = Regex::new(r"color_3=(?P<color_3>\d+,\d+,\d+)").unwrap();
    let cap = re.captures(&header);
    let arg = g_get_arg_color_3();
    let arg_empty = arg.is_empty();

    if cap.is_some() || !arg_empty
    {
        let stored_value = if cap.is_some() 
            {s!(cap.unwrap()["color_3"])}
            else {s!()};

        let argx = if arg_empty 
        {
            stored_value
        }

        else 
        {
            update = stored_value != arg; arg
        };

        let v: Vec<u8> = argx.split(',')
            .map(|s| s.trim())
            .map(|n| n.parse::<u8>()
            .unwrap_or(0))
            .collect();

        let value = if v.len() != 3 {DARK_THEME_COLOR_3}
            else {(v[0], v[1], v[2])};

        g_set_color_3(value);
    }

    else
    {
        g_set_color_3(DARK_THEME_COLOR_3);
    }

    if update {update_header()}
}

// Resets settings to default state
fn reset_settings()
{
    g_set_page_size(DEFAULT_PAGE_SIZE);
    g_set_row_space(DEFAULT_ROW_SPACE);
    g_set_color_1(DARK_THEME_COLOR_1);
    g_set_color_2(DARK_THEME_COLOR_2);
    g_set_color_3(DARK_THEME_COLOR_3);
}

// Gets a specific line from the notes
fn get_line(n: usize) -> String
{
    let lines = get_notes_vec();
    if n >= lines.len() {return s!()}
    lines[n].to_string()
}

// Replaces a line from the notes with a new line
fn replace_line(n: usize, new_text: String)
{
    let mut lines = get_notes_vec();
    lines[n] = new_text;
    update_file(lines.join("\n"));
}

// Swaps two lines from the notes
fn swap_lines(n1: usize, n2: usize)
{
    let mut lines = get_notes_vec();
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
    let mut left = get_notes_vec();
    let mut joined: Vec<String> = vec![];
    let mut moved = left.split_off(from[0]);
    let mut right = moved.split_off(from[1] - from[0] + 1);
    let nto = if to < from[0] {to} else {to - moved.len() + 1};
    joined.append(&mut left); joined.append(&mut right);
    joined.splice(nto..nto, moved.iter().cloned());

    // Reset last edit if it's no longer valid
    let last_edit = g_get_last_edit();
    
    if (from[0]..=from[1]).contains(&last_edit) || to == last_edit
    || (*from.first().unwrap() > last_edit && to < last_edit)
    || (*from.last().unwrap() < last_edit && to > last_edit)
    {
        g_set_last_edit(0);
    }

    update_file(joined.join("\n"));
}

// Deletes a line from the notes then updates the file
fn delete_lines(numbers: Vec<usize>)
{
    let lines = get_notes_vec();
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
    
    if !suggest && !g_get_last_find().is_empty()
    {
        p!("Shift+F To Use Previous Filter");
    }

    let last_find = g_get_last_find();
    let suggestion = if suggest && !last_find.is_empty() {&last_find} else {""};
    let filter = ask_string("Find", suggestion, true).to_lowercase();
    let mut found: Vec<(usize, String)> = vec![];
    if filter.is_empty() {return}
    let info = format!("{}{}{} >", 
        get_color(3), filter, get_color(2));

    if filter.starts_with("re:")
    {
        if let Ok(re) = Regex::new(format!("(?i){}", filter.replacen("re:", "", 1)).trim())
        {
            for (i, line) in get_notes_vec().iter().enumerate()
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

        for (i, line) in get_notes_vec().iter().enumerate()
        {
            if i == 0 {continue}
            if line.to_lowercase().contains(&ifilter) {found.push((i, s!(line)))}
        }
    }

    if found.is_empty()
    {
        return show_message(&format!("< No Results for {}", info));
    }

    g_set_last_find(filter); g_set_found(found);
    g_set_mode(s!("found")); next_found();
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

    fn nope()
    {
        show_message("< No Messages Were Deleted >")
    }

    if ans.starts_with("re:")
    {
        if let Ok(re) = Regex::new(format!("(?i){}", ans.replacen("re:", "", 1)).trim())
        {
            for (i, line) in get_notes_vec().iter().enumerate()
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

    numbers = numbers.iter()
        .filter(|n| check_line_exists(**n))
        .copied().collect();

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
        p!("This will delete all notes");
        
        if ask_bool("Are you sure?", true)
        {
            fs::remove_file(get_file_path()).unwrap();
            if !create_file() {exit()}
            reset_state(get_notes(true));
        }
    }

    else if ans == "s"
    {
        p!("This will restore settings to defaults");
        p!("E.g: page_size, row_space, color_1");

        if ask_bool("Are you sure?", true)
        {
            reset_settings(); update_header(); create_menus();
        }
    }
}

// Changes the password and updates the file with it
fn change_password()
{
    p!("This will change the file's password");
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
    let lines = get_notes_vec();
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
    if g_get_mode() == "all_notes"
    {
        return refresh_page();
    }

    else
    {
        g_set_mode(s!("all_notes"));
    }

    let mut notes: Vec<(usize, String)> = vec![];

    for (i, line) in get_notes_vec().iter().enumerate()
    {
        if i == 0 {continue}
        notes.push((i, s!(line)));
    }

    show_notes(0, notes, s!());
}

// Information about the program
fn show_about()
{
    if g_get_mode() == "about"
    {
        return refresh_page();
    }

    else
    {
        g_set_mode(s!("about"));
    }

    let art =
r#"
8888888888  .d888  .d888
888        d88P"  d88P"
888        888    888
8888888    888888 888888 .d88b.  888d888
888        888    888   d8P  Y8b 888P"
888        888    888   88888888 888
888        888    888   Y8b.     888
8888888888 888    888    "Y8888  888"#;

    let name = format!("Effer {} | Encrypted Notepad", VERSION);

    fn make_tip(s: &str) -> String
    {
        format!("{}Tip:{} {}", get_color(3), get_color(2), s)
    }

    let tips =
    [
        make_tip("Different major versions are not compatible"),
        make_tip("You can use 'first' and 'last' as note numbers"),
        make_tip("1-9 can be used to navigate the first 9 pages"),
        make_tip("Start the program with --help to check arguments")
    ].join("\n");

    let s = format!("{}{}{}\n\n{}\n\n{}", get_color(3), art, get_color(2), name, tips);

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
    let c1 = color_to_string(g_get_color_1());
    let c2 = color_to_string(g_get_color_2());
    let c3 = color_to_string(g_get_color_3());

    let s = format!("{} page_size={} row_space={} color_1={} color_2={} color_3={}",
        uc, ps, rs, c1, c2, c3);

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
    if g_get_mode() == "stats"
    {
        return refresh_page();
    }

    else
    {
        g_set_mode(s!("stats"));
    }

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
    if g_get_mode() == "screen_saver"
    {
        return refresh_page();
    }

    else
    {
        g_set_mode(s!("screen_saver"));
    }

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
    let mut notes = get_notes_vec();
    let started = g_get_started();

    // If there are no notes just fill it with source
    if notes.len() == 1
    {
        let mut lines: Vec<&str> = vec![&notes[0]];
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
                    let mut lines: Vec<&str> = vec![&notes[0]];
                    lines.extend(source.lines().filter(|s| !s.trim().is_empty()));
                    update_file(lines.join("\n")); g_set_last_edit(0);
                    if started {goto_last_page()}
                }
            },
            // Append
            "a" =>
            {
                let mut lines: Vec<String> = vec![]; lines.append(&mut notes);

                lines.append(&mut source.lines()
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| s!(s))
                    .collect());

                update_file(lines.join("\n")); if started {goto_last_page()}
            },
            // Prepend
            "p" =>
            {
                let mut lines: Vec<String> = vec![]; 
                let mut notes2 = notes.split_off(1);
                lines.append(&mut notes);
                let mut new_lines: Vec<String> = source.lines()
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| s!(s))
                    .collect();
                lines.append(&mut new_lines);
                lines.append(&mut notes2);
                update_file(lines.join("\n")); g_set_last_edit(0);
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
    p!("Add notes from a plain text file");
    let ans = ask_string("Path", "", true);
    if ans.is_empty() {return}
    get_source_content(&ans);
    handle_source();
}

// Changes the current notes file with another one
fn open_from_path()
{
    p!("Open and switch to other encrypted file");
    let ans = ask_string("Path", "", true);
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
    p!("This overwrites the file with junk several times.");
    p!("The file is then deleted and the program exits.");

    if ask_bool("Are you sure you want to destroy the file?", true)
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

// Changes individual colors
// Or all colors at once
// Or generates a random theme
fn change_colors()
{
    if !g_get_use_colors() {return}
    p!("(1) BG | (2) FG | (3) Other | (4) All");
    p!("(d) Dark | (t) Light | (p) Purple");
    p!("(r) Random | (v) Invert | (u) Undo");
    let ans = ask_string("Choice", "", true);
    if ans.is_empty() {return};
    let tip = "Example Keywords: 'red', 'darker', 'lighter'";
    let prompts = ["BG Color", "FG Color", "Other Color"];

    match &ans[..]
    {
        "1" | "2" | "3" =>
        {
            let n = ans.parse::<u8>().unwrap();

            let c = match n
            {
                1 => g_get_color_1(),
                2 => g_get_color_2(),
                3 => g_get_color_3(),
                _ => (0, 0, 0)
            };

            let prompt = format!("{} (r,g,b)", prompts[n as usize - 1]);
            let suggestion = color_to_string(c); p!(tip);
            let ans = ask_string(&prompt, &suggestion, true);
            if ans.is_empty() {return}
            let nc = parse_color(&ans, c);

            match n
            {
                1 => g_set_color_1(nc),
                2 => g_set_color_2(nc),
                3 => g_set_color_3(nc),
                _ => {}
            }
        },
        "4" =>
        {
            let mut suggestion = s!();
            let c1 = g_get_color_1();
            let c2 = g_get_color_2();
            let c3 = g_get_color_3();
            suggestion += &color_to_string(c1);
            suggestion += &format!(" - {}", color_to_string(c2));
            suggestion += &format!(" - {}", color_to_string(c3));
            p!(tip);
            let ans = ask_string("All Colors", &suggestion, false);
            if ans.is_empty() {return}
            let mut split = ans.split('-').map(|s| s.trim());

            g_set_color_1(parse_color(split.next().unwrap_or("0"), c1));
            g_set_color_2(parse_color(split.next().unwrap_or("0"), c2));
            g_set_color_3(parse_color(split.next().unwrap_or("0"), c3));
        },
        "d" =>
        {
            g_set_color_1(DARK_THEME_COLOR_1);
            g_set_color_2(DARK_THEME_COLOR_2);
            g_set_color_3(DARK_THEME_COLOR_3);
        },
        "t" =>
        {
            g_set_color_1(LIGHT_THEME_COLOR_1);
            g_set_color_2(LIGHT_THEME_COLOR_2);
            g_set_color_3(LIGHT_THEME_COLOR_3);
        },
        "p" =>
        {
            g_set_color_1(PURPLE_THEME_COLOR_1);
            g_set_color_2(PURPLE_THEME_COLOR_2);
            g_set_color_3(PURPLE_THEME_COLOR_3);
        },
        "r" =>
        {
            p!("Apply a random color to:");
            p!("(1) BG | (2) FG | (3) Other | (4) All");
            let ans = ask_string("Choice", "", true);
            if ans.is_empty() {return}
            let n = ans.parse::<u8>().unwrap_or(0);

            match n
            {
                1 => g_set_color_1(random_color()),
                2 => g_set_color_2(random_color()),
                3 => g_set_color_3(random_color()),
                4 =>
                {
                    g_set_color_1(random_color());
                    g_set_color_2(random_color());
                    g_set_color_3(random_color());
                },
                _ => return
            }
        },
        "v" =>
        {
            let c2 = g_get_color_2();
            let c3 = g_get_color_3();
            g_set_color_2(c3);
            g_set_color_3(c2);
        },
        "u" =>
        {
            p!("Restore the previous color of:");
            p!("(1) BG | (2) FG | (3) Other | (4) All");
            let ans = ask_string("Choice", "", true);
            if ans.is_empty() {return}
            let n = ans.parse::<u8>().unwrap_or(0);

            match n
            {
                1 => g_set_color_1(g_get_prev_color_1()),
                2 => g_set_color_2(g_get_prev_color_2()),
                3 => g_set_color_3(g_get_prev_color_3()),
                4 =>
                {
                    g_set_color_1(g_get_prev_color_1());
                    g_set_color_2(g_get_prev_color_2());
                    g_set_color_3(g_get_prev_color_3());
                },
                _ => return
            }
        },
        _ => return
    }

    create_menus(); update_header(); refresh_page();
}

// Gets the current theme
fn get_color(n: usize) -> String
{
    if !g_get_use_colors() {return s!()}

    match n
    {
        // Background Color
        1 =>
        {
            let t = g_get_color_1();
            s!(color::Bg(color::Rgb(t.0, t.1, t.2)))
        }
        // Foreground Color
        2 =>
        {
            let t = g_get_color_2();
            s!(color::Fg(color::Rgb(t.0, t.1, t.2)))
        }
        // Other Color
        3 =>
        {
            let t = g_get_color_3();
            s!(color::Fg(color::Rgb(t.0, t.1, t.2)))
        },
        // Input Colors
        4 => s!(color::Bg(color::Rgb(10,10,10))),
        5 => s!(color::Fg(color::Rgb(210,210,210))),
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
    pp!("From To (n1 n2) | "); 
    p!("Or Range (4-10 2)");
    pp!("Or Up (4 up 2) | ");
    p!("Or Down (4 down 2)");

    let ans = ask_string("Move", "", true);
    if ans.is_empty() {return}
    let num1; let mut num2;
    let max = g_get_notes_length();

    // Get the range to move
    if ans.contains('-')
    {
        if ans.matches('-').count() > 1 {return}
        let mut split = ans.split('-').map(|n| n.trim());
        num1 = parse_note_ans(split.next().unwrap_or("0"));
        let right_side = split.next().unwrap_or("nothing");
        let mut split_right = right_side.split_whitespace().map(|n| n.trim());
        num2 = parse_note_ans(split_right.next().unwrap_or("0"));
        if num1 == 0 || num2 == 0 {return}
        if num2 > max {num2 = max}
        if num1 >= num2 {return}
    }

    else
    {
        let mut split = ans.split_whitespace().map(|n| n.trim());
        num1 = parse_note_ans(split.next().unwrap_or("0"));
        if num1 == 0 {return}
        if !check_line_exists(num1) {return}
        num2 = num1;
    }

    // Get the destination index
    let dest = if ans.contains("up")
    {
        let steps = ans.split("up").last().unwrap_or("0").trim().parse::<usize>()
            .unwrap_or(0);
            
        if steps == 0 {return}
        if (num1 as isize - steps as isize) < 1 {return}
        num1 - steps
    }

    else if ans.contains("down")
    {
        let steps = ans.split("down").last().unwrap_or("0").trim().parse::<usize>()
            .unwrap_or(0);
            
        if steps == 0 {return}
        if num2 + steps > max {return}
        num2 + steps
    }

    else
    {
        let split = ans.split_whitespace().map(|n| n.trim());
        parse_note_ans(split.last().unwrap_or("0"))
    };
    
    if !check_line_exists(dest) {return}
    if dest >= num1 && dest <= num2 {return}
    if num1 == dest {return}
    
    move_lines(vec![num1, num2], dest);
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
fn calculate_padding(notes: &[(usize, String)]) -> usize
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

// Performs actions based on current mode
fn mode_action()
{
    match &g_get_mode()[..]
    {
        "found" => next_found(),
        _ => refresh_page()
    }
}

// Attemps to show the next found notes
fn next_found()
{
    let found = g_get_found_next(g_get_page_size());
    let remaining = g_get_found_remaining();

    if found.is_empty()
    {
        return refresh_page()
    }

    let tip = if remaining > 0 {" | (Enter) More"} else {""};
    let len = g_get_found_length();

    let info = format!("{}{}{}{} >",
        get_color(3), g_get_last_find(), get_color(2), tip);

    let diff = len - remaining;
    let mut message;

    if len == 1
    {
        message = s!("< 1 Result for ");
    }

    else if len <= 10
    {
        message = format!("< {} Results for ", len);
    }

    else
    {
        message = format!("< {}/{} Results for ", diff, len);
    }

    message += &info; show_notes(0, found, message);
}