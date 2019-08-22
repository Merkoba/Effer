use crate::s;

use lazy_static::lazy_static;
use std::
{
    sync::
    {
        Mutex, 
        atomic::
        {
            AtomicUsize, AtomicBool, Ordering
        }
    }
};

// Constants
pub const UNLOCK_CHECK: &str = "<Notes Unlocked>";
pub const VERSION: &str = "v1.5.0";
pub const RESET_FG_COLOR: &str = "\x1b[39m";
pub const DEFAULT_PAGE_SIZE: usize = 10;
pub const DEFAULT_ROW_SPACE: bool = true;
pub const DEFAULT_COLOR_1: &str = "Reset";
pub const DEFAULT_COLOR_2: &str = "Reset";
pub const DEFAULT_COLOR_3: &str = "Reset";

// Global variables
lazy_static! 
{
    static ref PATH: Mutex<String> = Mutex::new(s!());
    static ref PASSWORD: Mutex<String> = Mutex::new(s!());
    static ref NOTES: Mutex<String> = Mutex::new(s!());
    static ref SOURCE: Mutex<String> = Mutex::new(s!());
    static ref LAST_FIND: Mutex<String> = Mutex::new(s!());
    static ref COLOR_1: Mutex<String> = Mutex::new(s!());
    static ref COLOR_2: Mutex<String> = Mutex::new(s!());
    static ref COLOR_3: Mutex<String> = Mutex::new(s!());
    static ref MENUS: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref COLORS: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref NOTES_LENGTH: AtomicUsize = AtomicUsize::new(0);
    static ref PAGE: AtomicUsize = AtomicUsize::new(1);
    static ref CURRENT_MENU: AtomicUsize = AtomicUsize::new(0);
    static ref PAGE_SIZE: AtomicUsize = AtomicUsize::new(DEFAULT_PAGE_SIZE);
    static ref LAST_EDIT: AtomicUsize = AtomicUsize::new(0);
    static ref STARTED: AtomicBool = AtomicBool::new(false);
    static ref ROW_SPACE: AtomicBool = AtomicBool::new(true);
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

// Returns the last find global value
pub fn g_get_last_find() -> String
{
    s!(LAST_FIND.lock().unwrap())
}

// Sets the last find global value
pub fn g_set_last_find(s: String)
{
    *LAST_FIND.lock().unwrap() = s;
}

// Returns the color_1 global value
pub fn g_get_color_1() -> String
{
    s!(COLOR_1.lock().unwrap())
}

// Sets the color_1  global value
pub fn g_set_color_1(s: String)
{
    *COLOR_1.lock().unwrap() = s;
}

// Returns the color_2 global value
pub fn g_get_color_2() -> String
{
    s!(COLOR_2.lock().unwrap())
}

// Sets the color_2  global value
pub fn g_set_color_2(s: String)
{
    *COLOR_2.lock().unwrap() = s;
}

// Returns the color_3 global value
pub fn g_get_color_3() -> String
{
    s!(COLOR_3.lock().unwrap())
}

// Sets the color_3  global value
pub fn g_set_color_3(s: String)
{
    *COLOR_3.lock().unwrap() = s;
}


/// MUTEX VECTOR


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

// Returns an item from the colors global
pub fn g_get_colors_item(i: usize) -> String
{
    s!(COLORS.lock().unwrap()[i])
}

// Returns an item from the colors global
pub fn g_get_colors_position(s: String) -> usize
{
    COLORS.lock().unwrap().iter().position(|i| *i == s).unwrap()
}

// Returns the length of the colors global
pub fn g_get_colors_length() -> usize
{
    COLORS.lock().unwrap().len()
}

// Sets the colors global value
pub fn g_set_colors(v: Vec<String>)
{
    *COLORS.lock().unwrap() = v;
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


/// MUTEX BOOL


// Returns the started global value
pub fn g_get_started() -> bool
{
    STARTED.load(Ordering::SeqCst)
}

// Sets the started global value
pub fn g_set_started(b: bool)
{
    STARTED.store(b, Ordering::SeqCst);
}

// Returns the row space global value
pub fn g_get_row_space() -> bool
{
    ROW_SPACE.load(Ordering::SeqCst)
}

// Sets the row space global value
pub fn g_set_row_space(b: bool)
{
    ROW_SPACE.store(b, Ordering::SeqCst);
}