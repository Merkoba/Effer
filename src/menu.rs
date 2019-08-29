use crate::p;

use crate::globals::
{
    g_set_menus,
    g_get_menus_length,
    g_get_current_menu,
    g_set_current_menu,
    g_get_menus_item
};
use crate::notes::
{
    refresh_page,
    goto_last_page,
    change_page_size,
    goto_page,
    add_note,
    edit_note,
    swap_notes,
    move_notes,
    find_notes,
    delete_notes,
    undo_last_edit,
    cycle_left,
    cycle_right,
    show_page,
    show_all_notes,
    edit_last_note,
    change_row_space,
    goto_first_page
};
use crate::colors::
{
    get_color
};
use crate::structs::
{
    MenuAnswer
};
use crate::file::
{
    open_from_path,
    fetch_source,
    destroy,
    reset_file
};
use crate::modes::
{
    show_about,
    show_stats,
    show_screensaver,
    mode_action
};
use crate::other::
{
    ask_exit
};
use crate::encryption::
{
    change_password
};
use crate::colors::
{
    change_colors
};

use std::
{
    io::{stdin, stdout, Write},
    cmp::max
};

use termion::
{
    event::
    {
        Event, Key,
        MouseEvent, MouseButton
    },
    raw::IntoRawMode,
    input::TermRead
};

// Creates a menu item
pub fn menu_item(key: &str, label: &str, spacing:bool, separator: bool, newline: bool) -> String
{
    let nline = if newline {"\n"} else {""};
    let mut s = format!("{}{}({}){}", nline, get_color(3), key, get_color(2));
    if spacing {s += " "}; s += label;
    if separator {s += " | "} s
}

// Creates all the menus and stores them in a global
// Instead of making them on each iteration
pub fn create_menus()
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
            menu_item("U", "Undo Last Edit", true, false, false)
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
pub fn menu_input() -> (MenuAnswer, usize)
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
                            let n = d.to_digit(10).unwrap() as usize;
                            
                            if n > 0 {data = n; MenuAnswer::PageNumber}
                                else {MenuAnswer::Nothing}
                        },
                        'a' => MenuAnswer::AddNote,
                        'A' => MenuAnswer::AddNoteStart,
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
                        'N' => MenuAnswer::FetchSource,
                        'Q' => MenuAnswer::Exit,
                        '+' => MenuAnswer::IncreasePageSize,
                        '-' => MenuAnswer::DecreasePageSize,
                        ':' => MenuAnswer::ScreenSaver,
                        'U' => MenuAnswer::Undo,
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
pub fn menu_action(ans: (MenuAnswer, usize))
{
    match ans.0
    {
        MenuAnswer::AddNote => add_note(false),
        MenuAnswer::AddNoteStart => add_note(true),
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
        MenuAnswer::Undo => undo_last_edit(),
        MenuAnswer::Exit => ask_exit(),
        MenuAnswer::Nothing => {}
    }
}

// Changes to the next menu
// Wraps if at the end
pub fn cycle_menu()
{
    let mlength = g_get_menus_length();
    let menu = g_get_current_menu();
    if menu >= (mlength -1) {g_set_current_menu(0)}
    else {g_set_current_menu(menu + 1)}
    refresh_page();
}

// Shows and the menu
// and starts listening
pub fn show_menu()
{
    // Print menu
    p!(g_get_menus_item(g_get_current_menu()));

    // Listen and respond to input
    menu_action(menu_input());
}