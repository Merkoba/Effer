#![allow(clippy::suspicious_else_formatting)]
#![allow(clippy::collapsible_if)]

mod arguments;
mod colors;
mod encryption;
mod file;
mod globals;
mod input;
mod macros;
mod menu;
mod modes;
mod notes;
mod other;
mod settings;
mod structs;

use crate::{
    arguments::check_arguments,
    encryption::get_password,
    file::{file_path_check, get_file_path, handle_file_path_check, handle_source},
    globals::g_set_started,
    menu::create_menus,
    notes::{get_notes, goto_last_page, update_notes_statics},
    other::{change_screen, exit},
    settings::get_settings,
};

// First function to execute
fn main() {
    check_arguments(); // <-- It might exit here
    handle_file_path_check(file_path_check(get_file_path()));
    if get_password(false).is_empty() {
        exit()
    };
    let notes = get_notes(false);
    update_notes_statics(notes);
    handle_source();
    get_settings();
    create_menus();
    change_screen();
    g_set_started(true);

    // Start loop
    goto_last_page();
}
