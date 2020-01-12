use crate::{
    file::update_header,
    globals::{
        g_get_color_1, g_get_color_2, g_get_color_3, g_get_prev_color_1, g_get_prev_color_2,
        g_get_prev_color_3, g_get_use_colors, g_set_color_1, g_set_color_2, g_set_color_3,
        g_set_use_colors, DARK_THEME_COLOR_1, DARK_THEME_COLOR_2, DARK_THEME_COLOR_3,
        LIGHT_THEME_COLOR_1, LIGHT_THEME_COLOR_2, LIGHT_THEME_COLOR_3, PURPLE_THEME_COLOR_1,
        PURPLE_THEME_COLOR_2, PURPLE_THEME_COLOR_3,
    },
    input::{ask_bool, ask_string},
    menu::create_menus,
    notes::refresh_page,
    p, s,
};

use colorskill::{color_to_string, parse_color};

// Changes individual colors
// Or all colors at once
// Or generates a random theme
pub fn change_colors() {
    if !g_get_use_colors() {
        if ask_bool("Enable colors?", false) {
            g_set_use_colors(true);
            create_menus();
            update_header();
            refresh_page();
        }

        return;
    }

    if !g_get_use_colors() {
        return;
    }
    p!("(1) BG | (2) FG | (3) Other | (4) All");
    p!("(d) Dark | (t) Light | (p) Purple");
    p!("(x) Disable | (v) Invert | (u) Undo");
    let ans = ask_string("Choice", "", true);
    if ans.is_empty() {
        return;
    };
    let tip = "E.g: 0,0,0 | red | darker | lighter2 | random";
    let prompts = ["BG Color", "FG Color", "Other Color"];

    match &ans[..] {
        "1" | "2" | "3" => {
            let n = ans.parse::<u8>().unwrap();

            let c = match n {
                1 => g_get_color_1(),
                2 => g_get_color_2(),
                3 => g_get_color_3(),
                _ => (0, 0, 0),
            };

            let prompt = s!(prompts[n as usize - 1]);
            let suggestion = color_to_string(c);
            p!(tip);
            let ans = ask_string(&prompt, &suggestion, true);
            if ans.is_empty() {
                return;
            }
            let nc = parse_color(&ans, c);

            match n {
                1 => g_set_color_1(nc),
                2 => g_set_color_2(nc),
                3 => g_set_color_3(nc),
                _ => {}
            }
        }
        "4" => {
            let mut suggestion = s!();
            let c1 = g_get_color_1();
            let c2 = g_get_color_2();
            let c3 = g_get_color_3();
            suggestion += &color_to_string(c1);
            suggestion += &format!(" - {}", color_to_string(c2));
            suggestion += &format!(" - {}", color_to_string(c3));
            p!(tip);

            let ans = ask_string("All Colors", &suggestion, false);
            if ans.is_empty() {
                return;
            }
            let mut split = ans.split('-').map(|s| s.trim());

            g_set_color_1(parse_color(split.next().unwrap_or("0"), c1));
            g_set_color_2(parse_color(split.next().unwrap_or("0"), c2));
            g_set_color_3(parse_color(split.next().unwrap_or("0"), c3));
        }
        "d" => {
            g_set_color_1(DARK_THEME_COLOR_1);
            g_set_color_2(DARK_THEME_COLOR_2);
            g_set_color_3(DARK_THEME_COLOR_3);
        }
        "t" => {
            g_set_color_1(LIGHT_THEME_COLOR_1);
            g_set_color_2(LIGHT_THEME_COLOR_2);
            g_set_color_3(LIGHT_THEME_COLOR_3);
        }
        "p" => {
            g_set_color_1(PURPLE_THEME_COLOR_1);
            g_set_color_2(PURPLE_THEME_COLOR_2);
            g_set_color_3(PURPLE_THEME_COLOR_3);
        }
        "x" => {
            g_set_use_colors(false);
        }
        "v" => {
            let c2 = g_get_color_2();
            let c3 = g_get_color_3();
            g_set_color_2(c3);
            g_set_color_3(c2);
        }
        "u" => {
            p!("Restore the previous color of:");
            p!("(1) BG | (2) FG | (3) Other | (4) All");
            let ans = ask_string("Choice", "", true);
            if ans.is_empty() {
                return;
            }
            let n = ans.parse::<u8>().unwrap_or(0);

            match n {
                1 => g_set_color_1(g_get_prev_color_1()),
                2 => g_set_color_2(g_get_prev_color_2()),
                3 => g_set_color_3(g_get_prev_color_3()),
                4 => {
                    g_set_color_1(g_get_prev_color_1());
                    g_set_color_2(g_get_prev_color_2());
                    g_set_color_3(g_get_prev_color_3());
                }
                _ => return,
            }
        }
        _ => return,
    }

    create_menus();
    update_header();
    refresh_page();
}

// Gets the current theme
pub fn get_color(n: usize) -> String {
    if !g_get_use_colors() {
        return s!();
    }

    match n {
        // Background Color
        1 => {
            let t = g_get_color_1();
            s!(termion::color::Bg(termion::color::Rgb(t.0, t.1, t.2)))
        }
        // Foreground Color
        2 => {
            let t = g_get_color_2();
            s!(termion::color::Fg(termion::color::Rgb(t.0, t.1, t.2)))
        }
        // Other Color
        3 => {
            let t = g_get_color_3();
            s!(termion::color::Fg(termion::color::Rgb(t.0, t.1, t.2)))
        }
        // Input Colors
        4 => s!(termion::color::Bg(termion::color::Rgb(10, 10, 10))),
        5 => s!(termion::color::Fg(termion::color::Rgb(210, 210, 210))),
        _ => s!(""),
    }
}
