use crate::{
    e,
    encryption::{encrypt_text, get_key_derivation, get_password},
    globals::{
        g_get_color_1, g_get_color_2, g_get_color_3, g_get_last_path, g_get_notes_length,
        g_get_notes_vec, g_get_notes_vec_item, g_get_page_size, g_get_password, g_get_path,
        g_get_row_space, g_get_source, g_get_started, g_get_use_colors, g_set_last_edit,
        g_set_last_path, g_set_password, g_set_path, g_set_source,
    },
    input::{ask_bool, ask_string},
    menu::create_menus,
    notes::{get_notes, goto_first_page, goto_last_page, update_notes_statics},
    other::{exit, gibberish, reset_state, show_message},
    p, s,
    settings::reset_settings,
    structs::FilePathCheckResult,
};

use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    thread, time,
};

use colorskill::color_to_string;

// Gets a specific line from the notes
pub fn get_line(n: usize) -> String {
    if n > g_get_notes_length() {
        return s!();
    }
    g_get_notes_vec_item(n)
}

// Replaces a line from the notes with a new line
pub fn replace_line(n: usize, new_text: String) {
    let mut lines = g_get_notes_vec();
    lines[n] = new_text;
    update_file(lines.join("\n"));
}

// Swaps two lines from the notes
pub fn swap_lines(n1: usize, n2: usize) {
    let mut lines = g_get_notes_vec();
    lines.swap(n1, n2);
    update_file(lines.join("\n"));
}

// Moves a range of lines to another index
pub fn move_lines(n1: usize, n2: usize, dest: usize) {
    let mut left = g_get_notes_vec();
    let mut joined: Vec<String> = vec![];
    let mut moved = left.split_off(n1);
    let mut right = moved.split_off(n2 - n1 + 1);
    let nto = if dest < n1 {
        dest
    } else {
        dest - moved.len() + 1
    };
    joined.append(&mut left);
    joined.append(&mut right);
    joined.splice(nto..nto, moved.iter().cloned());
    update_file(joined.join("\n"));
}

// Deletes a line from the notes then updates the file
pub fn delete_lines(numbers: Vec<usize>) {
    let lines = g_get_notes_vec();
    let mut new_lines: Vec<&str> = vec![];

    for (i, line) in lines.iter().enumerate() {
        if i == 0 || !numbers.contains(&i) {
            new_lines.push(line);
        }
    }

    update_file(new_lines.join("\n"));
}

// Tries to get the user's home path
pub fn get_home_path() -> PathBuf {
    match dirs::home_dir() {
        Some(path) => path,
        None => {
            e!("Can't get your Home path.");
            exit();
        }
    }
}

// Gets the default file path
pub fn get_default_file_path() -> PathBuf {
    Path::new(&get_home_path().join(".config/effer/effer.dat")).to_path_buf()
}

// Gets the path of the file
pub fn get_file_path() -> PathBuf {
    Path::new(&shell_expand(&g_get_path())).to_path_buf()
}

// Gets the path of the file's parent
pub fn get_file_parent_path() -> PathBuf {
    get_file_path().parent().unwrap().to_path_buf()
}

// Checks the existence of the file
pub fn file_path_check(path: PathBuf) -> FilePathCheckResult {
    match fs::metadata(path) {
        Ok(attr) => {
            if !attr.is_file() {
                return FilePathCheckResult::NotAFile;
            }
        }
        Err(_) => {
            return FilePathCheckResult::DoesNotExist;
        }
    }

    FilePathCheckResult::Exists
}

// Reacts to previous check of file existence
pub fn handle_file_path_check(result: FilePathCheckResult) {
    match result {
        FilePathCheckResult::Exists => {
            //
        }
        FilePathCheckResult::DoesNotExist => {
            e!(result.message());
            let answer = ask_bool("Do you want to make the file now?", false);
            if answer {
                if !create_file() {
                    exit()
                }
            } else {
                exit()
            }
        }
        FilePathCheckResult::NotAFile => {
            e!(result.message());
            e!("Nothing to do. Exiting.");
            exit();
        }
    }
}

// Reads the file
pub fn get_file_bytes() -> Vec<u8> {
    let mut bytes: Vec<u8> = vec![];

    let mut file = match fs::File::open(get_file_path()) {
        Ok(fil) => fil,
        Err(_) => {
            p!("Can't open the file.");
            return bytes;
        }
    };

    file.read_to_end(&mut bytes).unwrap();
    return bytes;
}

// Generic function to read text from files
pub fn read_file(path: &str) -> Result<String, io::Error> {
    fs::read_to_string(Path::new(&shell_expand(path)))
}

// Replaces ~ and env variables to their proper form
pub fn shell_expand(path: &str) -> String {
    s!(*shellexpand::full(path).unwrap())
}

// Replaces /home/user with ~ if it's user's Home
pub fn shell_contract(path: &str) -> String {
    let hpath = get_home_path();
    let home = hpath.to_str().unwrap();

    if path.starts_with(home) {
        s!(path.replacen(home, "~", 1))
    } else {
        s!(path)
    }
}

// Asks the user if it wants to remake the file
// Or just reset the settings to default
// Then does it if response was positive
// Variables are then updated to reflect change
pub fn reset_file() {
    p!("(f) Remake File | (s) Reset Settings");
    let ans = ask_string("Choice", "", true);

    if ans == "f" {
        p!("This will delete all the notes.");
        p!("And remake the file with a new password.");

        if ask_bool("Are you sure?", true) {
            fs::remove_file(get_file_path()).unwrap();
            if !create_file() {
                exit()
            }
            reset_state(get_notes(true));
        }
    } else if ans == "s" {
        p!("This will restore all settings to defaults.");
        p!("E.g: page_size, row_space, color_1");

        if ask_bool("Are you sure?", true) {
            reset_settings();
            update_header();
            create_menus();
        }
    }
}

// Does the write operation to a file
pub fn do_file_write(encrypted: Vec<u8>) {
    match fs::write(get_file_path(), encrypted) {
        Ok(_) => {}
        Err(_) => {
            if g_get_started() {
                show_message("< Can't Write To File >");
            } else {
                e!("Unable to write text to file.");
                exit();
            }
        }
    }
}

// Attempts to create the file
// It adds a default header as its only initial content
// The content is encrypted using the password
pub fn create_file() -> bool {
    get_key_derivation();
    if get_password(true).is_empty() {
        return false;
    }
    let encrypted = encrypt_text("Dummy Space");

    match fs::create_dir_all(get_file_parent_path()) {
        Ok(_) => {}
        Err(_) => {
            e!("Can't create parent directories.");
            return false;
        }
    }

    match fs::File::create(get_file_path()) {
        Ok(f) => f,
        Err(_) => {
            e!("Error creating the file.");
            return false;
        }
    };

    do_file_write(encrypted);
    true
}

// Returns the file's header
pub fn get_header() -> String {
    g_get_notes_vec_item(0)
}

// Generates a header line
pub fn format_header() -> String {
    let ps = g_get_page_size();
    let rs = g_get_row_space();
    let c1 = color_to_string(g_get_color_1());
    let c2 = color_to_string(g_get_color_2());
    let c3 = color_to_string(g_get_color_3());
    let cc = g_get_use_colors();

    format!(
        "page_size={} row_space={} color_1={} color_2={} color_3={} use_colors={}",
        ps, rs, c1, c2, c3, cc
    )
}

// Modifies the header (first line) with new settings
pub fn update_header() {
    replace_line(0, format_header());
}

// Tries to get the content of a source path
pub fn get_source_content(path: &str) {
    match read_file(path) {
        Ok(text) => {
            g_set_source(if text.is_empty() { s!() } else { text });
        }
        Err(_) => {
            if g_get_started() {
                g_set_source(s!());
                show_message("< Invalid Source Path >");
            } else {
                e!("Invalid source path.");
                exit();
            }
        }
    }
}

// What to do when a source path is given
// Either replaces, appends, or prepends notes
// using the source file lines
pub fn handle_source() {
    let source = g_get_source();
    if source.is_empty() {
        return;
    }
    let mut notes = g_get_notes_vec();
    let started = g_get_started();

    // If there are no notes just fill it with source
    if notes.len() == 1 {
        let mut lines: Vec<&str> = vec![&notes[0]];
        lines.extend(source.lines().filter(|s| !s.trim().is_empty()));
        update_file(lines.join("\n"));
        if started {
            goto_last_page()
        }
    }
    // If notes already exist ask what to do
    else {
        let ans = ask_string("Source: (r) Replace | (a) Append | (p) Prepend", "", true);

        match &ans[..] {
            // Replace
            "r" => {
                if ask_bool("Replace everything?", true) {
                    let mut lines: Vec<&str> = vec![&notes[0]];
                    lines.extend(source.lines().filter(|s| !s.trim().is_empty()));
                    update_file(lines.join("\n"));
                    g_set_last_edit(0);
                    if started {
                        goto_last_page()
                    }
                }
            }
            // Append
            "a" => {
                let mut lines: Vec<String> = vec![];
                lines.append(&mut notes);

                lines.append(
                    &mut source
                        .lines()
                        .filter(|s| !s.trim().is_empty())
                        .map(|s| s!(s))
                        .collect(),
                );

                update_file(lines.join("\n"));
                if started {
                    goto_last_page()
                }
            }
            // Prepend
            "p" => {
                let mut lines: Vec<String> = vec![];
                let mut notes2 = notes.split_off(1);
                lines.append(&mut notes);
                let mut new_lines: Vec<String> = source
                    .lines()
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| s!(s))
                    .collect();
                lines.append(&mut new_lines);
                lines.append(&mut notes2);
                update_file(lines.join("\n"));
                g_set_last_edit(0);
                if started {
                    goto_first_page()
                }
            }
            _ => {}
        }
    }

    g_set_source(s!());
}

// Uses a source from within the program
pub fn fetch_source() {
    p!("Add notes from a plain text file.");
    let ans = ask_string("Path", &g_get_last_path(), true);
    if ans.is_empty() {
        return;
    }
    let path = shell_expand(&ans);
    g_set_last_path(s!(path));
    get_source_content(&path);
    handle_source();
}

// Changes the current notes file with another one
pub fn open_from_path() {
    p!("Open and switch to other encrypted file.");
    let ans = ask_string("Path", &g_get_last_path(), true);
    if ans.is_empty() {
        return;
    };
    let path = shell_expand(&ans);
    g_set_last_path(s!(path));

    match file_path_check(Path::new(&path).to_path_buf()) {
        FilePathCheckResult::Exists => {
            do_open_path(path, false);
        }
        FilePathCheckResult::DoesNotExist => {
            e!("File doesn't exist.");

            if ask_bool("Do you want to make the file now?", false) {
                do_open_path(path, true)
            }
        }
        FilePathCheckResult::NotAFile => {
            show_message("< Path Is Not A File >");
        }
    }
}

// Does the open from path action
pub fn do_open_path(pth: String, create: bool) {
    let opassword;
    let opath;

    let password = g_get_password();
    opassword = password;
    g_set_password(s!());

    let path = g_get_path();
    opath = path;
    g_set_path(pth);

    if create {
        if !create_file() {
            g_set_password(opassword);
            g_set_path(opath);
            show_message("< Can't Create File >");
            return;
        }
    }

    let notes = get_notes(true);

    if notes.is_empty() {
        g_set_password(opassword);
        g_set_path(opath);
        show_message("< Can't Decrypt File >");
    } else {
        reset_state(notes);
        goto_last_page();
    }
}

// Writes gibberish to the file several times
// Then deletes the file
// Then exists the program
pub fn destroy() {
    p!("This overwrites the file with junk several times.");
    p!("The file is then deleted and the program exits.");

    if ask_bool("Destroy the file?", true) {
        p!("Destroying...");

        let path = get_file_path();

        for i in 1..=10 {
            match fs::write(&path, gibberish(10_000)) {
                Ok(_) => {
                    if i < 10 {
                        // Maybe there's not a good reason for this
                        // But the idea is to let the file system assimilate the write
                        // In case there's some debouncer system in place
                        thread::sleep(time::Duration::from_millis(500));
                    }
                }
                Err(_) => {
                    e!("Can't overwrite file.");
                    break;
                }
            }
        }

        match fs::remove_file(&path) {
            Ok(_) => exit(),
            Err(_) => show_message("< Can't Destroy File >"),
        }
    }
}

// Encrypts and saves the updated notes to the file
pub fn update_file(text: String) {
    let encrypted = encrypt_text(&text);

    if encrypted.is_empty() {
        show_message("< Error Encrypting File >");
        return;
    }

    update_notes_statics(text);
    do_file_write(encrypted);
}
