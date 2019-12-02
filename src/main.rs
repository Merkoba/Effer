#![allow(clippy::suspicious_else_formatting)]
#![allow(clippy::collapsible_if)]

mod macros;
mod structs;
mod globals;
mod arguments;
mod colors;
mod menu;
mod input;
mod settings;
mod notes;
mod file;
mod encryption;
mod modes;
mod other;

use crate::
{
    globals::
    {
        g_set_started
    },
    arguments::
    {
        check_arguments
    },
    file::
    {
        get_file_path,
        file_path_check,
        handle_file_path_check,
        handle_source
    },
    notes::
    {
        get_notes,
        update_notes_statics,
        goto_last_page
    },
    settings::
    {
        get_settings
    },
    menu::
    {
        create_menus
    },
    encryption::
    {
        get_password
    },
    other::
    {
        exit,
        change_screen
    }
};

// First function to execute
fn main()
{
    check_arguments(); // <-- It might exit here
    handle_file_path_check(file_path_check(get_file_path()));
    if get_password(false).is_empty() {exit()};
    let notes = get_notes(false); if notes.is_empty() {exit()}
    update_notes_statics(notes); 
    handle_source(); 
    get_settings(); 
    create_menus();
    change_screen(); 
    g_set_started(true);

    // Start loop
    goto_last_page();
}