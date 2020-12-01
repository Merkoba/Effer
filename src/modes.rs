use crate::{
    colors::get_color,
    file::{get_file_bytes, shell_contract},
    globals::{g_get_mode, g_get_notes_length, g_get_path, g_set_mode, VERSION},
    notes::{get_notes, next_found, refresh_page},
    other::show_message,
    s,
};

// Show some statistics
pub fn show_stats() {
    if g_get_mode() == "stats" {
        return refresh_page();
    } else {
        g_set_mode(s!("stats"));
    }

    let notes = get_notes(false);
    let len = g_get_notes_length();
    let mut wcount = 0;
    let mut lcount = 0;

    for (i, line) in notes.lines().enumerate() {
        if i == 0 {
            continue;
        }
        wcount += line.split_whitespace().count();
        lcount += line.chars().filter(|c| *c != ' ').count();
    }

    let enc_size = get_file_bytes().len();
    let dec_size = notes.as_bytes().len();
    let path = shell_contract(&g_get_path());

    let s = format!("Stats For: {}\n\nNotes: {}\nWords: {}\nLetters: {}\nEncrypted Size: {} Bytes\nDecrypted Size: {} Bytes",
        path, len, wcount, lcount, enc_size, dec_size);

    show_message(&s);
}

// Hides notes from the screen with some characters
pub fn show_screensaver() {
    if g_get_mode() == "screen_saver" {
        return refresh_page();
    } else {
        g_set_mode(s!("screen_saver"));
    }

    let mut lines: Vec<String> = vec![];

    for _ in 0..7 {
        lines.push(s!(("😎 ".repeat(14)).trim()));
    }

    let message = lines.join("\n\n");
    show_message(&message);
}

// Information about the program
pub fn show_about() {
    if g_get_mode() == "about" {
        return refresh_page();
    } else {
        g_set_mode(s!("about"));
    }

    let art = r#"
8888888888  .d888  .d888
888        d88P"  d88P"
888        888    888
8888888    888888 888888 .d88b.  888d888
888        888    888   d8P  Y8b 888P"
888        888    888   88888888 888
888        888    888   Y8b.     888
8888888888 888    888    "Y8888  888"#;

    let name = format!("Effer {} | Encrypted Notepad", VERSION);

    pub fn make_tip(s: &str) -> String {
        format!("{}Tip:{} {}", get_color(3), get_color(2), s)
    }

    let tips = [
        make_tip("Different major versions may not be compatible"),
        make_tip("You can use 'first' and 'last' as note numbers"),
        make_tip("1-9 can be used to navigate the first 9 pages"),
        make_tip("Start the program with --help to check arguments"),
    ]
    .join("\n");

    let s = format!(
        "{}{}{}\n\n{}\n\n{}",
        get_color(3),
        art,
        get_color(2),
        name,
        tips
    );

    show_message(&s);
}

// Performs actions based on current mode
pub fn mode_action() {
    match &g_get_mode()[..] {
        "found" => next_found(),
        _ => refresh_page(),
    }
}
