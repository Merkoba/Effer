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
pub const VERSION: &str = "v1.6.0";
pub const RESET_FG_COLOR: &str = "\x1b[39m";
pub const DEFAULT_PAGE_SIZE: usize = 10;
pub const DEFAULT_ROW_SPACE: bool = true;
pub const DEFAULT_COLOR_1: (u8, u8, u8) = (25, 25, 25);
pub const DEFAULT_COLOR_2: (u8, u8, u8) = (210, 210, 210);
pub const DEFAULT_COLOR_3: (u8, u8, u8) = (36, 166, 188);
pub const MAX_PAGE_SIZE: usize = 100;

// Global variables
lazy_static! 
{
    static ref PATH: Mutex<String> = Mutex::new(s!());
    static ref PASSWORD: Mutex<String> = Mutex::new(s!());
    static ref NOTES: Mutex<String> = Mutex::new(s!());
    static ref SOURCE: Mutex<String> = Mutex::new(s!());
    static ref LAST_FIND: Mutex<String> = Mutex::new(s!());
    static ref MENUS: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref NOTES_LENGTH: AtomicUsize = AtomicUsize::new(0);
    static ref PAGE: AtomicUsize = AtomicUsize::new(1);
    static ref CURRENT_MENU: AtomicUsize = AtomicUsize::new(0);
    static ref LAST_EDIT: AtomicUsize = AtomicUsize::new(0);
    static ref STARTED: AtomicBool = AtomicBool::new(false);

    // Settings Provided As Arguments
    static ref ARG_PAGE_SIZE: Mutex<String> = Mutex::new(s!());
    static ref ARG_ROW_SPACE: Mutex<String> = Mutex::new(s!());
    static ref ARG_COLOR_1: Mutex<String> = Mutex::new(s!());
    static ref ARG_COLOR_2: Mutex<String> = Mutex::new(s!());
    static ref ARG_COLOR_3: Mutex<String> = Mutex::new(s!());

    // Settings Globals
    static ref PAGE_SIZE: AtomicUsize = AtomicUsize::new(DEFAULT_PAGE_SIZE);
    static ref ROW_SPACE: AtomicBool = AtomicBool::new(true);
    static ref COLOR_1: Mutex<(u8, u8, u8)> = Mutex::new((0, 0, 0));
    static ref COLOR_2: Mutex<(u8, u8, u8)> = Mutex::new((0, 0, 0));
    static ref COLOR_3: Mutex<(u8, u8, u8)> = Mutex::new((0, 0, 0));
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

// Returns the arg page size global value
pub fn g_get_arg_page_size() -> String
{
    s!(ARG_PAGE_SIZE.lock().unwrap())
}

// Sets the arg_page_size global value
pub fn g_set_arg_page_size(s: String)
{
    *ARG_PAGE_SIZE.lock().unwrap() = s;
}

// Returns the arg row space global value
pub fn g_get_arg_row_space() -> String
{
    s!(ARG_ROW_SPACE.lock().unwrap())
}

// Sets the arg row_space global value
pub fn g_set_arg_row_space(s: String)
{
    *ARG_ROW_SPACE.lock().unwrap() = s;
}

// Returns the arg color 1 global value
pub fn g_get_arg_color_1() -> String
{
    s!(ARG_COLOR_1.lock().unwrap())
}

// Sets the arg_color_1 global value
pub fn g_set_arg_color_1(s: String)
{
    *ARG_COLOR_1.lock().unwrap() = s;
}

// Returns the arg color 2 global value
pub fn g_get_arg_color_2() -> String
{
    s!(ARG_COLOR_2.lock().unwrap())
}

// Sets the arg_color_2 global value
pub fn g_set_arg_color_2(s: String)
{
    *ARG_COLOR_2.lock().unwrap() = s;
}

// Returns the arg color 3 global value
pub fn g_get_arg_color_3() -> String
{
    s!(ARG_COLOR_3.lock().unwrap())
}

// Sets the arg_color_3 global value
pub fn g_set_arg_color_3(s: String)
{
    *ARG_COLOR_3.lock().unwrap() = s;
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


/// MUTEX TUPLE


// Returns the color_1 global value
pub fn g_get_color_1() -> (u8, u8, u8)
{
    *COLOR_1.lock().unwrap()
}

// Sets the color_1  global value
pub fn g_set_color_1(t: (u8, u8, u8))
{
    *COLOR_1.lock().unwrap() = t;
}

// Returns the color_2 global value
pub fn g_get_color_2() -> (u8, u8, u8)
{
    *COLOR_2.lock().unwrap()
}

// Sets the color_2  global value
pub fn g_set_color_2(t: (u8, u8, u8))
{
    *COLOR_2.lock().unwrap() = t;
}

// Returns the color_3 global value
pub fn g_get_color_3() -> (u8, u8, u8)
{
    *COLOR_3.lock().unwrap()
}

// Sets the color_3  global value
pub fn g_set_color_3(t: (u8, u8, u8))
{
    *COLOR_3.lock().unwrap() = t;
}