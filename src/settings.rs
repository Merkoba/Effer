use crate::{
    file::{get_header, update_header},
    globals::{
        g_get_arg_color_1, g_get_arg_color_2, g_get_arg_color_3, g_get_arg_page_size,
        g_get_arg_row_space, g_get_arg_use_colors, g_set_color_1, g_set_color_2, g_set_color_3,
        g_set_page_size, g_set_row_space, g_set_use_colors, DARK_THEME_COLOR_1, DARK_THEME_COLOR_2,
        DARK_THEME_COLOR_3, DEFAULT_PAGE_SIZE, DEFAULT_ROW_SPACE, DEFAULT_USE_COLORS,
        MAX_PAGE_SIZE,
    },
    s,
};

use std::str::FromStr;

use regex::Regex;

// Gets settings from the header
// Sets defaults if they're not defined
pub fn get_settings() {
    let header = get_header();
    let mut update = false;

    let re = Regex::new(r"page_size=(?P<page_size>\d+)").unwrap();
    let cap = re.captures(&header);
    let arg = g_get_arg_page_size();
    let arg_empty = arg.is_empty();

    if cap.is_some() || !arg_empty {
        let stored_value = if cap.is_some() {
            s!(cap.unwrap()["page_size"])
        } else {
            s!()
        };

        let argx = if arg_empty {
            stored_value
        } else {
            update = stored_value != arg;
            arg
        };

        let num = argx.parse::<usize>().unwrap_or(DEFAULT_PAGE_SIZE);
        let mut value = num;
        if value <= 0 {
            value = DEFAULT_PAGE_SIZE
        }
        if value > MAX_PAGE_SIZE {
            value = MAX_PAGE_SIZE
        };
        g_set_page_size(value);
    } else {
        g_set_page_size(DEFAULT_PAGE_SIZE);
    }

    let re = Regex::new(r"row_space=(?P<row_space>\w+)").unwrap();
    let cap = re.captures(&header);
    let arg = g_get_arg_row_space();
    let arg_empty = arg.is_empty();

    if cap.is_some() || !arg_empty {
        let stored_value = if cap.is_some() {
            s!(cap.unwrap()["row_space"])
        } else {
            s!()
        };

        let argx = if arg_empty {
            stored_value
        } else {
            update = stored_value != arg;
            arg
        };

        let value = FromStr::from_str(&argx).unwrap_or(DEFAULT_ROW_SPACE);
        g_set_row_space(value);
    } else {
        g_set_row_space(DEFAULT_ROW_SPACE);
    }

    let re = Regex::new(r"color_1=(?P<color_1>\d+,\d+,\d+)").unwrap();
    let cap = re.captures(&header);
    let arg = g_get_arg_color_1();
    let arg_empty = arg.is_empty();

    if cap.is_some() || !arg_empty {
        let stored_value = if cap.is_some() {
            s!(cap.unwrap()["color_1"])
        } else {
            s!()
        };

        let argx = if arg_empty {
            stored_value
        } else {
            update = stored_value != arg;
            arg
        };

        let v: Vec<u8> = argx
            .split(',')
            .map(|s| s.trim())
            .map(|n| n.parse::<u8>().unwrap_or(0))
            .collect();

        let value = if v.len() != 3 {
            DARK_THEME_COLOR_1
        } else {
            (v[0], v[1], v[2])
        };

        g_set_color_1(value);
    } else {
        g_set_color_1(DARK_THEME_COLOR_1);
    }

    let re = Regex::new(r"color_2=(?P<color_2>\d+,\d+,\d+)").unwrap();
    let cap = re.captures(&header);
    let arg = g_get_arg_color_2();
    let arg_empty = arg.is_empty();

    if cap.is_some() || !arg_empty {
        let stored_value = if cap.is_some() {
            s!(cap.unwrap()["color_2"])
        } else {
            s!()
        };

        let argx = if arg_empty {
            stored_value
        } else {
            update = stored_value != arg;
            arg
        };

        let v: Vec<u8> = argx
            .split(',')
            .map(|s| s.trim())
            .map(|n| n.parse::<u8>().unwrap_or(0))
            .collect();

        let value = if v.len() != 3 {
            DARK_THEME_COLOR_2
        } else {
            (v[0], v[1], v[2])
        };

        g_set_color_2(value);
    } else {
        g_set_color_2(DARK_THEME_COLOR_2);
    }

    let re = Regex::new(r"color_3=(?P<color_3>\d+,\d+,\d+)").unwrap();
    let cap = re.captures(&header);
    let arg = g_get_arg_color_3();
    let arg_empty = arg.is_empty();

    if cap.is_some() || !arg_empty {
        let stored_value = if cap.is_some() {
            s!(cap.unwrap()["color_3"])
        } else {
            s!()
        };

        let argx = if arg_empty {
            stored_value
        } else {
            update = stored_value != arg;
            arg
        };

        let v: Vec<u8> = argx
            .split(',')
            .map(|s| s.trim())
            .map(|n| n.parse::<u8>().unwrap_or(0))
            .collect();

        let value = if v.len() != 3 {
            DARK_THEME_COLOR_3
        } else {
            (v[0], v[1], v[2])
        };

        g_set_color_3(value);
    } else {
        g_set_color_3(DARK_THEME_COLOR_3);
    }

    let re = Regex::new(r"use_colors=(?P<use_colors>\w+)").unwrap();
    let cap = re.captures(&header);
    let arg = g_get_arg_use_colors();
    let arg_empty = arg.is_empty();

    if cap.is_some() || !arg_empty {
        let stored_value = if cap.is_some() {
            s!(cap.unwrap()["use_colors"])
        } else {
            s!()
        };

        let argx = if arg_empty {
            stored_value
        } else {
            update = stored_value != arg;
            arg
        };

        let value = FromStr::from_str(&argx).unwrap_or(DEFAULT_USE_COLORS);
        g_set_use_colors(value);
    } else {
        g_set_use_colors(DEFAULT_USE_COLORS);
    }

    // If any setting is new
    // update the header
    if update {
        update_header()
    }
}

// Resets settings to default state
pub fn reset_settings() {
    g_set_page_size(DEFAULT_PAGE_SIZE);
    g_set_row_space(DEFAULT_ROW_SPACE);
    g_set_color_1(DARK_THEME_COLOR_1);
    g_set_color_2(DARK_THEME_COLOR_2);
    g_set_color_3(DARK_THEME_COLOR_3);
}
