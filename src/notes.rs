use crate::
{
    s, p, pp,
    globals::
    {
        MAX_PAGE_SIZE,
        PAGE_SIZE_DIFF,
        g_get_notes,
        g_set_notes,
        g_get_notes_vec,
        g_set_notes_vec,
        g_get_notes_length,
        g_set_notes_length,
        g_set_last_edit,
        g_get_last_edit,
        g_get_last_find,
        g_set_last_find,
        g_set_found,
        g_set_mode,
        g_get_mode,
        g_get_page,
        g_set_page,
        g_get_page_size,
        g_set_page_size,
        g_get_row_space,
        g_set_row_space,
        g_get_prev_notes,
        g_get_found_next,
        g_get_notes_vec_range,
        g_get_found_remaining,
        g_get_found_length,
        g_get_path
    },
    file::
    {
        update_file,
        replace_line,
        move_lines,
        swap_lines,
        update_header,
        get_file_text,
        shell_contract,
        delete_lines,
        get_line
    },
    encryption::
    {
        decrypt_text
    },
    colors::
    {
        get_color
    },
    input::
    {
        ask_bool,
        ask_string
    },
    menu::
    {
        show_menu
    },
    other::
    {
        show_message
    }
};

use std::
{
    cmp::{min, max}
};

use regex::Regex;

// Provides an input to add a new note
pub fn add_note(prepend: bool)
{
    let prompt;

    if prepend
    {
        prompt = "Add At Start";
    }

    else
    {
        prompt = "Add At End";
        p!("Shift+A To Add At Start");
    }
    
    let note = ask_string(prompt, "", false);
    if note.trim().is_empty() {return} let new_text;

    if prepend
    {
        let mut notes = g_get_notes_vec();
        let rest = notes.split_off(1);
        new_text = format!("{}\n{}\n{}", notes[0], note, rest.join("\n"));
        update_file(new_text);
        g_set_last_edit(1);
        goto_first_page();
    }

    else
    {
        new_text = format!("{}\n{}", get_notes(false), note);
        update_file(new_text);
        g_set_last_edit(g_get_notes_length());
        goto_last_page();
    }
}

// Asks for a note number to edit
// The note is then showed and editable
pub fn edit_note(mut n: usize)
{
    if n == 0
    {
        let last_edit = g_get_last_edit();
        let suggestion = if last_edit == 0 {s!()}
        else {expand_note_number(last_edit)};
        n = parse_note_ans(&ask_string("Edit #", &(suggestion), true));
    }

    if !check_line_exists(n) {return}
    let edited = ask_string("Edit Note", &get_line(n), false);
    if edited.is_empty() {return}
    g_set_last_edit(n);
    replace_line(n, edited);
    show_page(get_note_page(n));
}

// Finds a note by a filter
// Case insensitive
// Substrings are counted
pub fn find_notes(suggest: bool)
{
    pp!("Enter Filter | "); p!("Or Regex (re:\\d+)");
    
    if !suggest && !g_get_last_find().is_empty()
    {
        p!("Shift+F To Use Previous Filter");
    }

    let last_find = g_get_last_find();
    let suggestion = if suggest && !last_find.is_empty() {&last_find} else {""};
    let filter = ask_string("Find", suggestion, true).to_lowercase();
    let mut found: Vec<(usize, String)> = vec![];
    if filter.is_empty() {return}
    let info = format!("{}{}{} >", 
        get_color(3), filter, get_color(2));

    if filter.starts_with("re:")
    {
        if let Ok(re) = Regex::new(format!("(?i){}", filter.replacen("re:", "", 1)).trim())
        {
            for (i, line) in g_get_notes_vec().iter().enumerate()
            {
                if i == 0 {continue}
                if re.is_match(line) {found.push((i, s!(line)))}
            }
        }

        else
        {
            return show_message(&format!("< Invalid Regex: {}", info));
        }
    }

    else
    {
        let ifilter = filter.to_lowercase();

        for (i, line) in g_get_notes_vec().iter().enumerate()
        {
            if i == 0 {continue}
            if line.to_lowercase().contains(&ifilter) {found.push((i, s!(line)))}
        }
    }

    if found.is_empty()
    {
        return show_message(&format!("< No Results for {}", info));
    }

    g_set_last_find(filter); g_set_found(found);
    g_set_mode(s!("found")); next_found();
}

// Swaps 2 notes specified by 2 numbers separated by whitespace (1 10)
pub fn swap_notes()
{
    let ans = ask_string("Swap (n1 n2)", "", true);
    if ans.is_empty() {return}
    let mut split = ans.split_whitespace().map(|s| s.trim());
    let n1 = parse_note_ans(split.next().unwrap_or("0"));
    let n2 = parse_note_ans(split.next().unwrap_or("0"));
    if !check_line_exists(n1) || !check_line_exists(n2) {return}

    // If one of the two items is the last edited note
    // Swap it so it points to the correct one
    let  last_edit = g_get_last_edit();
    if last_edit == n1 {g_set_last_edit(n2)}
    else if last_edit == n2 {g_set_last_edit(n1)}

    swap_lines(n1, n2);
}

// Deletes 1 or more notes
// Can delete by a specific note number (3)
// Or a comma separated list (1,2,3)
// Or a range (1-3)
pub fn delete_notes()
{
    pp!("Enter Note # | ");
    p!("Or List (1,2,3)");
    pp!("Or Range (1-3) | ");
    p!("Or Regex (re:\\d+)");

    let ans = ask_string("Delete", "", true);
    if ans.is_empty() {return}
    let mut numbers: Vec<usize> = vec![];

    pub fn nope()
    {
        show_message("< No Messages Were Deleted >")
    }

    if ans.starts_with("re:")
    {
        if let Ok(re) = Regex::new(format!("(?i){}", ans.replacen("re:", "", 1)).trim())
        {
            for (i, line) in g_get_notes_vec().iter().enumerate()
            {
                if i == 0 {continue}
                if re.is_match(line) {numbers.push(i)}
            }
        }

        else
        {
            return show_message("< Invalid Regex | (Enter) Return >");
        }
    }

    else if ans.contains(',')
    {
        numbers.extend(ans.split(',').map(|n| parse_note_ans(n.trim())).collect::<Vec<usize>>());
    }

    else if ans.contains('-')
    {
        if ans.matches('-').count() > 1 {return nope()}
        let note_length = g_get_notes_length();
        let mut split = ans.split('-').map(|n| n.trim());
        let num1 = parse_note_ans(split.next().unwrap_or("0"));
        let mut num2 = parse_note_ans(split.next().unwrap_or("0"));
        if num1 == 0 || num2 == 0 {return nope()}
        if num2 > note_length {num2 = note_length}
        if num1 >= num2 {return nope()}
        numbers.extend(num1..=num2);
    }

    else
    {
        numbers.push(parse_note_ans(&ans));
    }

    numbers = numbers.iter()
        .filter(|n| check_line_exists(**n))
        .copied().collect();

    let length = numbers.len();

    if length >= 5
    {
        if !ask_bool(&format!("Delete {} notes?", length), true)
        {
            return;
        }
    }

    if numbers.is_empty()
    {
        return nope()
    }

    // If the deleted not is the last edit
    // reset last edit since it's now invalid
    let last_edit = g_get_last_edit();
    if numbers.contains(&last_edit) {g_set_last_edit(0)}

    delete_lines(numbers);
}

// Updates the notes and notes length global variables
pub fn update_notes_statics(text: String) -> String
{
    let v: Vec<String> = text.lines().map(|s| s!(s)).collect();
    g_set_notes(text); g_set_notes_length(v.len() - 1); 
    g_set_notes_vec(v); g_get_notes()
}

// Generic format for note items
pub fn format_note(note: &(usize, String), colors: bool, padding: usize, indent: bool) -> String
{
    let mut pad = s!();

    if padding > 0
    {
        let len = note.0.to_string().len();

        if len < padding
        {
            for _ in 0..(padding  - len)
            {
                pad += " ";
            }
        }
    }

    let mut space = s!();

    if indent
    {
        for _ in 0..(note.0.to_string().len() + pad.len() + 3)
        {
            space += " ";
        }
    }

    let n = termion::terminal_size().unwrap().0 as usize - note.0.to_string().len() - pad.len() - 5;
    let txt = textwrap::fill(&note.1, min(50, n)); let text = s!(textwrap::indent(&txt, &space).trim());

    if colors
    {
        format!("{}({}) {}{}{}", get_color(3), note.0, pad, get_color(2), text)
    }

    else
    {
        format!("({}) {}{}", note.0, pad, text)
    }
}

// Checks if a supplied page exists
pub fn check_page_number(page: usize, allow_zero: bool) -> usize
{
    if allow_zero && page == 0 {return 0}
    max(1, min(page, get_max_page_number()))
}

// Gets notes that belong to a certain page
pub fn get_page_notes(page: usize) -> Vec<(usize, String)>
{
    let notes_length = g_get_notes_length();
    if notes_length == 0 {return vec![]}
    let page_size = g_get_page_size();

    let a = if page > 1 {((page - 1) * page_size) + 1} 
        else {1};

    let b = min(page * page_size, notes_length);

    (a..).zip(g_get_notes_vec_range(a, b))
        .collect::<Vec<(usize, String)>>()
}

// Gets the maximum number of pages
pub fn get_max_page_number() -> usize
{
    let notes_length = g_get_notes_length();
    let n = notes_length as f64 / g_get_page_size() as f64;
    max(1, n.ceil() as usize)
}

// Goes to the previous page
pub fn cycle_left()
{
    let page = g_get_page();
    if page == 1 {return}
    show_page(page - 1);
}

// Goes to the next page
pub fn cycle_right()
{
    let page = g_get_page();
    let max_page = get_max_page_number();
    if page == max_page {return}
    show_page(page + 1);
}

// Edits the most recent note
pub fn edit_last_note()
{
    edit_note(g_get_notes_length());
}

// Checks a line number from the notes exist
pub fn check_line_exists(n: usize) -> bool
{
    n > 0 && n <= g_get_notes_length()
}

// Replaces keywords to note numbers
// Or parses the string to a number
pub fn parse_note_ans(ans: &str) -> usize
{
    match ans
    {
        "first" => 1,
        "last" => g_get_notes_length(),
        _ => ans.parse().unwrap_or(0)
    }
}

// Replaces keywords to page numbers
// Or parses the string to a number
pub fn parse_page_ans(ans: &str) -> usize
{
    match ans
    {
        "first" => 1,
        "last" => get_max_page_number(),
        _ => ans.parse().unwrap_or(0)
    }
}

// Refreshes the current page (notes, menu, etc)
// This doesn't provoke a change unless on a different mode like Find results
pub fn refresh_page()
{
    show_page(g_get_page());
}

// Main renderer function
// Shows the notes and the menu at the bottom
// Then waits and reacts for input
pub fn show_notes(mut page: usize, notes: Vec<(usize, String)>, message: String)
{
    loop
    {
        // Clear the screen and sets colors
        println!("{}{}{}", get_color(1), get_color(2), termion::clear::All);

        page = check_page_number(page, true);

        if page > 0
        {
            g_set_mode(s!("notes"));
            print_notes(&get_page_notes(page));
        }

        else
        {
            print_notes(&notes)
        }

        if page > 0
        {
            g_set_page(page); show_page_indicator(page);
        }

        else if !message.is_empty() {p!(format!("\n{}", message))}

        show_menu();
    }
}

// Prints notes to the screen
pub fn print_notes(notes: &[(usize, String)])
{
    let space = g_get_row_space();
    let padding = calculate_padding(&notes);

    for note in notes.iter()
    {
        if space {p!("")}
        p!(format_note(note, true, padding, true));
    }
}

// Asks for a range or single note
// and a destination. The moves it
pub fn move_notes()
{
    pp!("From To (n1 n2) | "); 
    p!("Or Range (4-10 2)");
    pp!("Or Up (4 up 2) | ");
    p!("Or Down (4 down 2)");

    let ans = ask_string("Move", "", true);
    if ans.is_empty() {return}
    let n1; let mut n2;
    let max = g_get_notes_length();

    // Get the range to move
    if ans.contains('-')
    {
        if ans.matches('-').count() > 1 {return}
        let mut split = ans.split('-').map(|n| n.trim());
        n1 = parse_note_ans(split.next().unwrap_or("0"));
        let right_side = split.next().unwrap_or("nothing");
        let mut split_right = right_side.split_whitespace().map(|n| n.trim());
        n2 = parse_note_ans(split_right.next().unwrap_or("0"));
        if n1 == 0 || n2 == 0 {return}
        if n2 > max {n2 = max}
        if n1 >= n2 {return}
    }

    else
    {
        let mut split = ans.split_whitespace().map(|n| n.trim());
        n1 = parse_note_ans(split.next().unwrap_or("0"));
        if n1 == 0 {return}
        if !check_line_exists(n1) {return}
        n2 = n1;
    }

    // Get the destination index
    let dest = if ans.contains("up")
    {
        let steps = ans.split("up").last().unwrap_or("0").trim().parse::<usize>()
            .unwrap_or(0);
            
        if steps == 0 {return}
        if (n1 as isize - steps as isize) < 1 {return}
        n1 - steps
    }

    else if ans.contains("down")
    {
        let steps = ans.split("down").last().unwrap_or("0").trim().parse::<usize>()
            .unwrap_or(0);
            
        if steps == 0 {return}
        if n2 + steps > max {return}
        n2 + steps
    }

    else
    {
        let split = ans.split_whitespace().map(|n| n.trim());
        parse_note_ans(split.last().unwrap_or("0"))
    };
    
    if !check_line_exists(dest) {return}
    if dest >= n1 && dest <= n2 {return}
    if n1 == dest {return}

    move_lines(n1, n2, dest);
    fix_last_edit_after_move(n1, n2, dest);
    show_page(get_note_page(dest));
}

// Changes the last edit value after a move
// to reflect the new correct position
pub fn fix_last_edit_after_move(n1: usize, n2:usize, dest:usize)
{
    let last_edit = g_get_last_edit();
    let mut new_last_edit = last_edit;

    if (n1..=n2).contains(&last_edit)
    {
        if dest < n1
        {
            new_last_edit = last_edit - (n1 - dest);
        }

        else if dest > n2
        {
            new_last_edit = last_edit + (dest - n2);
        }
    }

    else if dest == last_edit
    {
        if dest < n1
        {
            new_last_edit = last_edit + (n2 - n1) + 1;
        }

        else if dest > n2
        {
            new_last_edit = last_edit - (n2 - n1) - 1;
        }
    }

    else if n1 > last_edit && dest < last_edit
    {
        new_last_edit = last_edit + (n2 - n1) + 1;
    }

    else if n2 < last_edit && dest > last_edit
    {
        new_last_edit = last_edit - (n2 - n1) - 1;
    }

    g_set_last_edit(new_last_edit);
}

// Shows the page indicator above the menu
pub fn show_page_indicator(page: usize)
{
    p!(format!("\n< Page {} of {} >\n{}",
        page, get_max_page_number(),
        shell_contract(&g_get_path().to_string())));
}

// Changes note numbers to equivalents
// like first and last
pub fn expand_note_number(n: usize) -> String
{
    if n == 1 {s!("first")}
    else if n == g_get_notes_length() {s!("last")}
    else {s!(n)}
}

// Calculates if some padding
// must be given between note numbers
// and note text. So all notes look aligned
// Returns the difference and the max length
pub fn calculate_padding(notes: &[(usize, String)]) -> usize
{
    let mut max = 0;
    let mut len = 0;

    for note in notes.iter()
    {
        let nl = note.0.to_string().len();

        if len == 0
        {
            len = nl; continue;
        }

        else if nl > len
        {
            len = nl; max = nl;
        }
    }

    max
}

// Attemps to show the next found notes
pub fn next_found()
{
    let found = g_get_found_next(5);
    let remaining = g_get_found_remaining();

    if found.is_empty()
    {
        return refresh_page()
    }

    let tip = if remaining > 0 {" | (Enter) More"} else {""};
    let len = g_get_found_length();

    let info = format!("{}{}{}{} >",
        get_color(3), g_get_last_find(), get_color(2), tip);

    let diff = len - remaining; let mut message;

    if len == 1
    {
        message = s!("< 1 Result for ");
    }

    else if len <= 10
    {
        message = format!("< {} Results for ", len);
    }

    else
    {
        message = format!("< {}/{} Results for ", diff, len);
    }

    message += &info; show_notes(0, found, message);
}

// Restores the file 
// to the previous state
pub fn undo_last_edit()
{
    let notes = g_get_notes();
    let prev_notes = g_get_prev_notes();

    if prev_notes == notes
    {
        return show_message("< Nothing To Undo >");
    }

    p!("This will undo notes to the previous state.");
    p!("Multiple undos in a row can't be performed.");

    if ask_bool("Undo?", true)
    {
        update_file(prev_notes);
        refresh_page();
    }
}

// Enables or disables spacing between notes
pub fn change_row_space()
{
    g_set_row_space(!g_get_row_space());
    update_header();
}

// Show notes from a certain page
pub fn show_page(n: usize)
{
    show_notes(n, vec![], s!());
}

// Gets the page number where a note belongs to
pub fn get_note_page(n: usize) -> usize
{
    (n as f64 / g_get_page_size() as f64).ceil() as usize
}

// Go to a page or note's page
pub fn goto_page()
{
    let ans = ask_string("(p) Page | (n) Note", "", true);
    if ans.is_empty() {return}
    
    match &ans[..]
    {
        // Goto Page
        "p" =>
        {
            let pn = parse_page_ans(&ask_string("Page #", "", true));
            if pn < 1 || pn > get_max_page_number() {return}
            show_page(pn);
        },
        // Goto Note's Page
        "n" =>
        {
            let nn = parse_note_ans(&ask_string("Note #", "", true));
            if nn < 1 || nn > g_get_notes_length() {return}
            show_page(get_note_page(nn));
        },
        _ => {}
    }
}

// Changes how many items appear per page
pub fn change_page_size(increase: bool)
{
    let max_page = get_max_page_number();
    let page_size = g_get_page_size();

    if increase
    {
        if page_size < MAX_PAGE_SIZE && max_page > 1 {g_set_page_size(page_size + PAGE_SIZE_DIFF)} else {return}
    }

    else
    {
        if page_size >= (PAGE_SIZE_DIFF * 2) {g_set_page_size(page_size - PAGE_SIZE_DIFF)} else {return}
    }

    update_header();
}

// Shows all notes at once
pub fn show_all_notes()
{
    if g_get_mode() == "all_notes"
    {
        return refresh_page();
    }

    else
    {
        g_set_mode(s!("all_notes"));
    }

    let mut notes: Vec<(usize, String)> = vec![];

    for (i, line) in g_get_notes_vec().iter().enumerate()
    {
        if i == 0 {continue}
        notes.push((i, s!(line)));
    }

    show_notes(0, notes, s!());
}

// Gets the notes form the global variable 
// or reads them from the file
pub fn get_notes(update: bool) -> String
{
    let notes = g_get_notes();

    if notes.is_empty() || update 
        {decrypt_text(&get_file_text())} 
    else {notes}
}

// Goes to the first page
pub fn goto_first_page()
{
    show_page(1);
}

// Goes to the last page
pub fn goto_last_page()
{
    show_page(get_max_page_number());
}