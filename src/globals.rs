use crate::s;

use lazy_static::lazy_static;
use std::sync::
{
    Mutex, atomic::{AtomicUsize, Ordering}
};

// Constants
pub const UNLOCK_CHECK: &str = "<Notes Unlocked>";
pub const VERSION: &str = "v1.4.0";
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
    static ref NOTES_LENGTH: AtomicUsize = AtomicUsize::new(0);
    static ref PAGE: AtomicUsize = AtomicUsize::new(1);
    static ref CURRENT_MENU: AtomicUsize = AtomicUsize::new(0);
    static ref PAGE_SIZE: AtomicUsize = AtomicUsize::new(DEFAULT_PAGE_SIZE);
    static ref LAST_EDIT: AtomicUsize = AtomicUsize::new(0);
    static ref THEME: AtomicUsize = AtomicUsize::new(0);
}


/// MUTEX STRING


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


/// MUTEX BOOL


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


/// MUTEX VECTOR


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


/// ATOMIC USIZE


// Returns the page global value
pub fn g_get_page() -> usize
{
    PAGE.load(Ordering::SeqCst)
}

// Sets the page  global value
pub fn g_set_page(n: usize)
{
    PAGE.store(n, Ordering::SeqCst)
}

// Returns the current menu global value
pub fn g_get_current_menu() -> usize
{
  CURRENT_MENU.load(Ordering::SeqCst)
}

// Sets the current menu  global value
pub fn g_set_current_menu(n: usize)
{
  CURRENT_MENU.store(n, Ordering::SeqCst)
}

// Returns the last edit global value
pub fn g_get_last_edit() -> usize
{
  LAST_EDIT.load(Ordering::SeqCst)
}

// Sets the last edit  global value
pub fn g_set_last_edit(n: usize)
{
  LAST_EDIT.store(n, Ordering::SeqCst)
}

// Returns the theme global value
pub fn g_get_theme() -> usize
{
  THEME.load(Ordering::SeqCst)
}

// Sets the theme  global value
pub fn g_set_theme(n: usize)
{
  THEME.store(n, Ordering::SeqCst)
}

// Returns the notes length global value
pub fn g_get_notes_length() -> usize
{
    NOTES_LENGTH.load(Ordering::SeqCst)
}

// Sets the notes length  global value
pub fn g_set_notes_length(n: usize)
{
    NOTES_LENGTH.store(n, Ordering::SeqCst);
}

// Returns the page size global value
pub fn g_get_page_size() -> usize
{
    PAGE_SIZE.load(Ordering::SeqCst)
}

// Sets the page size global value
pub fn g_set_page_size(n: usize)
{
    PAGE_SIZE.store(n, Ordering::SeqCst);
}