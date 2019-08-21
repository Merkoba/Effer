use crate::s;

use lazy_static::lazy_static;
use std::sync::Mutex;

// Constants
pub const UNLOCK_CHECK: &str = "<Notes Unlocked>";
pub const VERSION: &str = "v1.3.2";
pub const RESET: &str = "\x1b[0m";
pub const DEFAULT_PAGE_SIZE: usize = 10;
pub const DEFAULT_ROW_SPACE: bool = true;
pub const DEFAULT_THEME: usize = 0;

// Global variables
lazy_static! 
{
    static ref PATH: Mutex<String> = Mutex::new(s!());
    static ref PASSWORD: Mutex<String> = Mutex::new(s!());
    static ref NOTES: Mutex<String> = Mutex::new(s!());
    static ref SOURCE: Mutex<String> = Mutex::new(s!());
    static ref MENUS: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref THEMES: Mutex<Vec<(String, String)>> = Mutex::new(vec![]);
    static ref STARTED: Mutex<bool> = Mutex::new(false);
    static ref ROW_SPACE: Mutex<bool> = Mutex::new(true);
    static ref NOTES_LENGTH: Mutex<usize> = Mutex::new(0);
    static ref PAGE: Mutex<usize> = Mutex::new(1);
    static ref CURRENT_MENU: Mutex<usize> = Mutex::new(0);
    static ref PAGE_SIZE: Mutex<usize> = Mutex::new(DEFAULT_PAGE_SIZE);
    static ref LAST_EDIT: Mutex<usize> = Mutex::new(0);
    static ref THEME: Mutex<usize> = Mutex::new(0);
}

// Returns the started global value
pub fn g_get_started() -> bool
{
    *STARTED.lock().unwrap() 
}

// Sets the started global value
pub fn g_set_started(b: bool)
{
    *STARTED.lock().unwrap() = b;
}

// Returns the row space global value
pub fn g_get_row_space() -> bool
{
    *ROW_SPACE.lock().unwrap()
}

// Sets the row space global value
pub fn g_set_row_space(b: bool)
{
    *ROW_SPACE.lock().unwrap() = b;
}

// Returns the page size global value
pub fn g_get_page_size() -> usize
{
    *PAGE_SIZE.lock().unwrap()
}

// Sets the page size global value
pub fn g_set_page_size(n: usize)
{
    *PAGE_SIZE.lock().unwrap() = n;
}

// Returns the notes global value
pub fn g_get_notes() -> String
{
    s!(NOTES.lock().unwrap())
}

// Sets the notes global value
pub fn g_set_notes(s: String)
{
    *NOTES.lock().unwrap() = s;
}

// Returns the notes length global value
pub fn g_get_notes_length() -> usize
{
    *NOTES_LENGTH.lock().unwrap()
}

// Sets the notes length  global value
pub fn g_set_notes_length(n: usize)
{
    *NOTES_LENGTH.lock().unwrap() = n;
}

// Returns the path global value
pub fn g_get_path() -> String
{
    s!(PATH.lock().unwrap())
}

// Sets the path global value
pub fn g_set_path(s: String)
{
    *PATH.lock().unwrap() = s;
}

// Returns the password global value
pub fn g_get_password() -> String
{
    s!(PASSWORD.lock().unwrap())
}

// Sets the password global value
pub fn g_set_password(s: String)
{
    *PASSWORD.lock().unwrap() = s;
}

// Returns the source global value
pub fn g_get_source() -> String
{
    s!(SOURCE.lock().unwrap())
}

// Sets the source global value
pub fn g_set_source(s: String)
{
    *SOURCE.lock().unwrap() = s;
}

// Returns an item from the menus global
pub fn g_get_menus_item(i: usize) -> String
{
    s!(MENUS.lock().unwrap()[i])
}

// Returns the length of the menus global
pub fn g_get_menus_length() -> usize
{
    MENUS.lock().unwrap().len()
}

// Sets the menus global value
pub fn g_set_menus(v: Vec<String>)
{
    *MENUS.lock().unwrap() = v;
}

// Returns the current menu global value
pub fn g_get_current_menu() -> usize
{
    *CURRENT_MENU.lock().unwrap()
}

// Sets the current menu  global value
pub fn g_set_current_menu(n: usize)
{
    *CURRENT_MENU.lock().unwrap() = n;
}

// Returns the last edit global value
pub fn g_get_last_edit() -> usize
{
    *LAST_EDIT.lock().unwrap()
}

// Sets the last edit  global value
pub fn g_set_last_edit(n: usize)
{
    *LAST_EDIT.lock().unwrap() = n;
}

// Returns the theme global value
pub fn g_get_theme() -> usize
{
    *THEME.lock().unwrap()
}

// Sets the theme  global value
pub fn g_set_theme(n: usize)
{
    *THEME.lock().unwrap() = n;
}

// Returns an item from the themes global
pub fn g_get_themes_item(i: usize) -> (String, String)
{
    THEMES.lock().unwrap()[i].clone()
}

// Returns the length of the themes global
pub fn g_get_themes_length() -> usize
{
    THEMES.lock().unwrap().len()
}

// Sets the themes global value
pub fn g_set_themes(v: Vec<(String, String)>)
{
    *THEMES.lock().unwrap() = v;
}

// Returns the page global value
pub fn g_get_page() -> usize
{
    *PAGE.lock().unwrap()
}

// Sets the page  global value
pub fn g_set_page(n: usize)
{
    *PAGE.lock().unwrap() = n;
}