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
    },
    collections::VecDeque,
    cmp::min
};

// Constants
pub const VERSION: &str = "v1.11.0";
pub const UNLOCK_CHECK: &str = "<Notes Unlocked>";
pub const DEFAULT_PAGE_SIZE: usize = 10;
pub const MAX_PAGE_SIZE: usize = 100;
pub const DEFAULT_ROW_SPACE: bool = true;
pub const DEFAULT_USE_COLORS: bool = true;

// Color Constants
pub const DARK_THEME_COLOR_1: (u8, u8, u8) = (37, 41, 51);
pub const DARK_THEME_COLOR_2: (u8, u8, u8) = (202, 207, 218);
pub const DARK_THEME_COLOR_3: (u8, u8, u8) = (136, 192, 209);
pub const LIGHT_THEME_COLOR_1: (u8, u8, u8) = (240, 240, 240);
pub const LIGHT_THEME_COLOR_2: (u8, u8, u8) = (20, 20, 20);
pub const LIGHT_THEME_COLOR_3: (u8, u8, u8) = (12, 130, 89);
pub const PURPLE_THEME_COLOR_1: (u8, u8, u8) = (42,37,65);
pub const PURPLE_THEME_COLOR_2: (u8, u8, u8) = (143,55,249);
pub const PURPLE_THEME_COLOR_3: (u8, u8, u8) = (193,47,105);

// Global variables
lazy_static! 
{
    static ref PATH: Mutex<String> = Mutex::new(s!());
    static ref PASSWORD: Mutex<String> = Mutex::new(s!());
    static ref NOTES: Mutex<String> = Mutex::new(s!());
    static ref SOURCE: Mutex<String> = Mutex::new(s!());
    static ref LAST_FIND: Mutex<String> = Mutex::new(s!());
    static ref MODE: Mutex<String> = Mutex::new(s!());
    static ref LAST_PATH: Mutex<String> = Mutex::new(s!());
    static ref MENUS: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref NOTES_VEC: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref FOUND: Mutex<VecDeque<(usize, String)>> = Mutex::new(VecDeque::new());
    static ref NOTES_LENGTH: AtomicUsize = AtomicUsize::new(0);
    static ref PAGE: AtomicUsize = AtomicUsize::new(1);
    static ref CURRENT_MENU: AtomicUsize = AtomicUsize::new(0);
    static ref LAST_EDIT: AtomicUsize = AtomicUsize::new(0);
    static ref FOUND_LENGTH: AtomicUsize = AtomicUsize::new(0);
    static ref STARTED: AtomicBool = AtomicBool::new(false);
    static ref ALTSCREEN: AtomicBool = AtomicBool::new(false);
}

lazy_static!
{
    // Settings Provided As Arguments
    static ref ARG_PAGE_SIZE: Mutex<String> = Mutex::new(s!());
    static ref ARG_ROW_SPACE: Mutex<String> = Mutex::new(s!());
    static ref ARG_COLOR_1: Mutex<String> = Mutex::new(s!());
    static ref ARG_COLOR_2: Mutex<String> = Mutex::new(s!());
    static ref ARG_COLOR_3: Mutex<String> = Mutex::new(s!());
    static ref ARG_USE_COLORS: Mutex<String> = Mutex::new(s!());

    // Settings Globals
    static ref PAGE_SIZE: AtomicUsize = AtomicUsize::new(DEFAULT_PAGE_SIZE);
    static ref ROW_SPACE: AtomicBool = AtomicBool::new(true);
    static ref COLOR_1: Mutex<(u8, u8, u8)> = Mutex::new((50, 50, 50));
    static ref COLOR_2: Mutex<(u8, u8, u8)> = Mutex::new((50, 50, 50));
    static ref COLOR_3: Mutex<(u8, u8, u8)> = Mutex::new((50, 50, 50));
    static ref PREV_COLOR_1: Mutex<(u8, u8, u8)> = Mutex::new((50, 50, 50));
    static ref PREV_COLOR_2: Mutex<(u8, u8, u8)> = Mutex::new((50, 50, 50));
    static ref PREV_COLOR_3: Mutex<(u8, u8, u8)> = Mutex::new((50, 50, 50));
    static ref USE_COLORS: AtomicBool = AtomicBool::new(true);
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

// Sets the arg page size global value
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

// Sets the arg color 1 global value
pub fn g_set_arg_color_1(s: String)
{
    *ARG_COLOR_1.lock().unwrap() = s;
}

// Returns the arg color 2 global value
pub fn g_get_arg_color_2() -> String
{
    s!(ARG_COLOR_2.lock().unwrap())
}

// Sets the arg color 2 global value
pub fn g_set_arg_color_2(s: String)
{
    *ARG_COLOR_2.lock().unwrap() = s;
}

// Returns the arg color 3 global value
pub fn g_get_arg_color_3() -> String
{
    s!(ARG_COLOR_3.lock().unwrap())
}

// Sets the arg color 3 global value
pub fn g_set_arg_color_3(s: String)
{
    *ARG_COLOR_3.lock().unwrap() = s;
}

// Returns the arg use colors global value
pub fn g_get_arg_use_colors() -> String
{
    s!(ARG_USE_COLORS.lock().unwrap())
}

// Sets the arg use colors global value
pub fn g_set_arg_use_colors(s: String)
{
    *ARG_USE_COLORS.lock().unwrap() = s;
}

// Returns the mode global value
pub fn g_get_mode() -> String
{
    s!(MODE.lock().unwrap())
}

// Sets the mode global value
pub fn g_set_mode(s: String)
{
    *MODE.lock().unwrap() = s;
}

// Returns the last path global value
pub fn g_get_last_path() -> String
{
    s!(LAST_PATH.lock().unwrap())
}

// Sets the last path global value
pub fn g_set_last_path(s: String)
{
    *LAST_PATH.lock().unwrap() = s;
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

// Returns an item from the found global
pub fn g_get_found_next(amount: usize) -> Vec<(usize, String)>
{
    let mut found = FOUND.lock().unwrap();
    let upper = min(found.len(), amount);
    found.drain(0..upper).collect::<Vec<(usize, String)>>()
}

// Returns an item from the found global
pub fn g_get_found_remaining() -> usize
{
    FOUND.lock().unwrap().len()
}

// Sets the found global value
pub fn g_set_found(v: Vec<(usize, String)>)
{
    FOUND_LENGTH.store(v.len(), Ordering::SeqCst);
    *FOUND.lock().unwrap() = VecDeque::from(v);
}

// Returns the notes vec global
pub fn g_get_notes_vec() -> Vec<String>
{
    NOTES_VEC.lock().unwrap().clone()
}

// Returns an item from the notes vec global
pub fn g_get_notes_vec_item(i: usize) -> String
{
    s!(NOTES_VEC.lock().unwrap()[i])
}

// Returns a range from the notes vec global
pub fn g_get_notes_vec_range(a: usize, b:usize) -> Vec<String>
{
    NOTES_VEC.lock().unwrap()[a..=b].iter().map(|x| s!(x)).collect()
}

// Sets the note _vec global value
pub fn g_set_notes_vec(v: Vec<String>)
{
    *NOTES_VEC.lock().unwrap() = v;
}


/// MUTEX TUPLE


// Returns the color 1 global value
pub fn g_get_color_1() -> (u8, u8, u8)
{
    *COLOR_1.lock().unwrap()
}

// Sets the color 1 global value
pub fn g_set_color_1(t: (u8, u8, u8))
{
    let mut c = COLOR_1.lock().unwrap();
    *PREV_COLOR_1.lock().unwrap() = *c; 
    *c = t;
}

// Returns the color 2 global value
pub fn g_get_color_2() -> (u8, u8, u8)
{
    *COLOR_2.lock().unwrap()
}

// Sets the color 2 global value
pub fn g_set_color_2(t: (u8, u8, u8))
{
    let mut c = COLOR_2.lock().unwrap();
    *PREV_COLOR_2.lock().unwrap() = *c; 
    *c = t;
}

// Returns the color 3 global value
pub fn g_get_color_3() -> (u8, u8, u8)
{
    *COLOR_3.lock().unwrap()
}

// Sets the color 3 global value
pub fn g_set_color_3(t: (u8, u8, u8))
{
    let mut c = COLOR_3.lock().unwrap();
    *PREV_COLOR_3.lock().unwrap() = *c; 
    *c = t;
}

// Returns the prev color 1 global value
pub fn g_get_prev_color_1() -> (u8, u8, u8)
{
    *PREV_COLOR_1.lock().unwrap()
}

// Returns the prev color 2 global value
pub fn g_get_prev_color_2() -> (u8, u8, u8)
{
    *PREV_COLOR_2.lock().unwrap()
}

// Returns the prev color 3 global value
pub fn g_get_prev_color_3() -> (u8, u8, u8)
{
    *PREV_COLOR_3.lock().unwrap()
}


/// ATOMIC USIZE


// Returns the page global value
pub fn g_get_page() -> usize
{
    PAGE.load(Ordering::SeqCst)
}

// Sets the page global value
pub fn g_set_page(n: usize)
{
    PAGE.store(n, Ordering::SeqCst)
}

// Returns the current menu global value
pub fn g_get_current_menu() -> usize
{
  CURRENT_MENU.load(Ordering::SeqCst)
}

// Sets the current menu global value
pub fn g_set_current_menu(n: usize)
{
  CURRENT_MENU.store(n, Ordering::SeqCst)
}

// Returns the last edit global value
pub fn g_get_last_edit() -> usize
{
  LAST_EDIT.load(Ordering::SeqCst)
}

// Sets the last edit global value
pub fn g_set_last_edit(n: usize)
{
  LAST_EDIT.store(n, Ordering::SeqCst)
}

// Returns the notes length global value
pub fn g_get_notes_length() -> usize
{
    NOTES_LENGTH.load(Ordering::SeqCst)
}

// Sets the notes length global value
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

// Returns the found length global value
pub fn g_get_found_length() -> usize
{
    FOUND_LENGTH.load(Ordering::SeqCst)
}


/// ATOMIC BOOL


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

// Returns the use colors global value
pub fn g_get_use_colors() -> bool
{
    USE_COLORS.load(Ordering::SeqCst)
}

// Sets the use colors global value
pub fn g_set_use_colors(b: bool)
{
    USE_COLORS.store(b, Ordering::SeqCst);
}

// Returns the altscreen global value
pub fn g_get_altscreen() -> bool
{
    ALTSCREEN.load(Ordering::SeqCst)
}

// Sets the altscreen global value
pub fn g_set_altscreen(b: bool)
{
    ALTSCREEN.store(b, Ordering::SeqCst);
}