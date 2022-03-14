use crate::{
    globals::{g_get_altscreen, g_set_altscreen, g_set_last_edit, g_set_prev_notes},
    menu::create_menus,
    notes::{show_notes, update_notes_statics},
    p, s,
    settings::get_settings,
};

use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::{
    io::{stdout, Write},
    iter, process,
};
use termion::raw::IntoRawMode;

// Switch to the alternative screen
// Place the cursor at the bottom left
pub fn change_screen() {
    p!("{}", termion::screen::ToAlternateScreen);
    g_set_altscreen(true);
    let size = termion::terminal_size().unwrap();
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}", termion::cursor::Goto(1, size.1)).unwrap();
}

// Centralized function to exit the program
// Switches back to main screen before exiting
pub fn exit() -> ! {
    if g_get_altscreen() {
        p!("{}", termion::screen::ToMainScreen);
    }

    process::exit(0)
}

// Generic function to show a message instead of notes
pub fn show_message(message: &str) {
    show_notes(0, vec![], s!(message));
}

// Creates random text
pub fn gibberish(n: usize) -> String {
    let mut rng = thread_rng();

    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(n)
        .collect::<String>()
}

// Resets some properties to defaults
// This is used when the file changes
pub fn reset_state(notes: String) {
    g_set_prev_notes(s!());
    update_notes_statics(notes);
    get_settings();
    create_menus();
    g_set_last_edit(0);
}
