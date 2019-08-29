use crate::s;
use crate::p;

use crate::globals::
{
    DARK_THEME_COLOR_1,
    DARK_THEME_COLOR_2,
    DARK_THEME_COLOR_3,
    LIGHT_THEME_COLOR_1,
    LIGHT_THEME_COLOR_2,
    LIGHT_THEME_COLOR_3,
    PURPLE_THEME_COLOR_1,
    PURPLE_THEME_COLOR_2,
    PURPLE_THEME_COLOR_3,
    g_get_use_colors,
    g_set_use_colors,
    g_get_color_1,
    g_get_color_2,
    g_get_color_3,
    g_set_color_1,
    g_set_color_2,
    g_set_color_3,
    g_get_prev_color_1,
    g_get_prev_color_2,
    g_get_prev_color_3
};
use crate::menu::
{
    create_menus
};
use crate::file::
{
    update_header
};
use crate::notes::
{
    refresh_page
};
use crate::input::
{
    ask_bool,
    ask_string
};

use rand::Rng;
use colorsys::{Rgb, Hsl};

// These are the degrees used to
// make colors darker or lighter
const DEGREES_1: f64 = 15.0;
const DEGREES_2: f64 = 30.0;
const DEGREES_3: f64 = 45.0;

// Changes individual colors
// Or all colors at once
// Or generates a random theme
pub fn change_colors()
{
    if !g_get_use_colors()
    {
        if ask_bool("Enable colors?", false)
        {
            g_set_use_colors(true);
            create_menus(); update_header(); refresh_page(); 
        }

        return
    }

    if !g_get_use_colors() {return}
    p!("(1) BG | (2) FG | (3) Other | (4) All");
    p!("(d) Dark | (t) Light | (p) Purple");
    p!("(x) Disable | (v) Invert | (u) Undo");
    let ans = ask_string("Choice", "", true);
    if ans.is_empty() {return};
    let tip = "E.g: 0,0,0 | red | darker | lighter2 | random";
    let prompts = ["BG Color", "FG Color", "Other Color"];

    match &ans[..]
    {
        "1" | "2" | "3" =>
        {
            let n = ans.parse::<u8>().unwrap();

            let c = match n
            {
                1 => g_get_color_1(),
                2 => g_get_color_2(),
                3 => g_get_color_3(),
                _ => (0, 0, 0)
            };

            let prompt = s!(prompts[n as usize - 1]);
            let suggestion = color_to_string(c); p!(tip);
            let ans = ask_string(&prompt, &suggestion, true);
            if ans.is_empty() {return}
            let nc = parse_color(&ans, c);

            match n
            {
                1 => g_set_color_1(nc),
                2 => g_set_color_2(nc),
                3 => g_set_color_3(nc),
                _ => {}
            }
        },
        "4" =>
        {
            let mut suggestion = s!();
            let c1 = g_get_color_1();
            let c2 = g_get_color_2();
            let c3 = g_get_color_3();
            suggestion += &color_to_string(c1);
            suggestion += &format!(" - {}", color_to_string(c2));
            suggestion += &format!(" - {}", color_to_string(c3));
            p!(tip);
            
            let ans = ask_string("All Colors", &suggestion, false);
            if ans.is_empty() {return}
            let mut split = ans.split('-').map(|s| s.trim());

            g_set_color_1(parse_color(split.next().unwrap_or("0"), c1));
            g_set_color_2(parse_color(split.next().unwrap_or("0"), c2));
            g_set_color_3(parse_color(split.next().unwrap_or("0"), c3));
        },
        "d" =>
        {
            g_set_color_1(DARK_THEME_COLOR_1);
            g_set_color_2(DARK_THEME_COLOR_2);
            g_set_color_3(DARK_THEME_COLOR_3);
        },
        "t" =>
        {
            g_set_color_1(LIGHT_THEME_COLOR_1);
            g_set_color_2(LIGHT_THEME_COLOR_2);
            g_set_color_3(LIGHT_THEME_COLOR_3);
        },
        "p" =>
        {
            g_set_color_1(PURPLE_THEME_COLOR_1);
            g_set_color_2(PURPLE_THEME_COLOR_2);
            g_set_color_3(PURPLE_THEME_COLOR_3);
        },
        "x" =>
        {
            g_set_use_colors(false);
        },
        "v" =>
        {
            let c2 = g_get_color_2();
            let c3 = g_get_color_3();
            g_set_color_2(c3);
            g_set_color_3(c2);
        },
        "u" =>
        {
            p!("Restore the previous color of:");
            p!("(1) BG | (2) FG | (3) Other | (4) All");
            let ans = ask_string("Choice", "", true);
            if ans.is_empty() {return}
            let n = ans.parse::<u8>().unwrap_or(0);

            match n
            {
                1 => g_set_color_1(g_get_prev_color_1()),
                2 => g_set_color_2(g_get_prev_color_2()),
                3 => g_set_color_3(g_get_prev_color_3()),
                4 =>
                {
                    g_set_color_1(g_get_prev_color_1());
                    g_set_color_2(g_get_prev_color_2());
                    g_set_color_3(g_get_prev_color_3());
                },
                _ => return
            }
        },
        _ => return
    }

    create_menus(); update_header(); refresh_page();
}

// Gets the current theme
pub fn get_color(n: usize) -> String
{
    if !g_get_use_colors() {return s!()}

    match n
    {
        // Background Color
        1 =>
        {
            let t = g_get_color_1();
            s!(termion::color::Bg(termion::color::Rgb(t.0, t.1, t.2)))
        }
        // Foreground Color
        2 =>
        {
            let t = g_get_color_2();
            s!(termion::color::Fg(termion::color::Rgb(t.0, t.1, t.2)))
        }
        // Other Color
        3 =>
        {
            let t = g_get_color_3();
            s!(termion::color::Fg(termion::color::Rgb(t.0, t.1, t.2)))
        },
        // Input Colors
        4 => s!(termion::color::Bg(termion::color::Rgb(10,10,10))),
        5 => s!(termion::color::Fg(termion::color::Rgb(210,210,210))),
        _ => s!("")
    }
}

// Gets an RGB tuple from a color name
pub fn get_color_name_rgb(name: &str, fallback: (u8, u8, u8)) -> (u8, u8, u8)
{
    match name
    {
        "maroon" => (128,0,0),
        "darkred" => (139,0,0),
        "brown" => (165,42,42),
        "firebrick" => (178,34,34),
        "crimson" => (220,20,60),
        "red" => (255,0,0),
        "tomato" => (255,99,71),
        "coral" => (255,127,80),
        "indianred" => (205,92,92),
        "lightcoral" => (240,128,128),
        "darksalmon" => (233,150,122),
        "salmon" => (250,128,114),
        "lightsalmon" => (255,160,122),
        "orangered" => (255,69,0),
        "darkorange" => (255,140,0),
        "orange" => (255,165,0),
        "gold" => (255,215,0),
        "darkgoldenrod" => (184,134,11),
        "goldenrod" => (218,165,32),
        "palegoldenrod" => (238,232,170),
        "darkkhaki" => (189,183,107),
        "khaki" => (240,230,140),
        "olive" => (128,128,0),
        "yellow" => (255,255,0),
        "yellowgreen" => (154,205,50),
        "darkolivegreen" => (85,107,47),
        "olivedrab" => (107,142,35),
        "lawngreen" => (124,252,0),
        "chartreuse" => (127,255,0),
        "greenyellow" => (173,255,47),
        "darkgreen" => (0,100,0),
        "green" => (0,128,0),
        "forestgreen" => (34,139,34),
        "lime" => (0,255,0),
        "limegreen" => (50,205,50),
        "lightgreen" => (144,238,144),
        "palegreen" => (152,251,152),
        "darkseagreen" => (143,188,143),
        "mediumspringgreen" => (0,250,154),
        "springgreen" => (0,255,127),
        "seagreen" => (46,139,87),
        "mediumaquamarine" => (102,205,170),
        "mediumseagreen" => (60,179,113),
        "lightseagreen" => (32,178,170),
        "darkslategray" => (47,79,79),
        "teal" => (0,128,128),
        "darkcyan" => (0,139,139),
        "aqua" => (0,255,255),
        "cyan" => (0,255,255),
        "lightcyan" => (224,255,255),
        "darkturquoise" => (0,206,209),
        "turquoise" => (64,224,208),
        "mediumturquoise" => (72,209,204),
        "paleturquoise" => (175,238,238),
        "aquamarine" => (127,255,212),
        "powderblue" => (176,224,230),
        "cadetblue" => (95,158,160),
        "steelblue" => (70,130,180),
        "cornflowerblue" => (100,149,237),
        "deepskyblue" => (0,191,255),
        "dodgerblue" => (30,144,255),
        "lightblue" => (173,216,230),
        "skyblue" => (135,206,235),
        "lightskyblue" => (135,206,250),
        "midnightblue" => (25,25,112),
        "navy" => (0,0,128),
        "darkblue" => (0,0,139),
        "mediumblue" => (0,0,205),
        "blue" => (0,0,255),
        "royalblue" => (65,105,225),
        "blueviolet" => (138,43,226),
        "indigo" => (75,0,130),
        "darkslateblue" => (72,61,139),
        "slateblue" => (106,90,205),
        "mediumslateblue" => (123,104,238),
        "mediumpurple" => (147,112,219),
        "darkmagenta" => (139,0,139),
        "darkviolet" => (148,0,211),
        "darkorchid" => (153,50,204),
        "mediumorchid" => (186,85,211),
        "purple" => (128,0,128),
        "thistle" => (216,191,216),
        "plum" => (221,160,221),
        "violet" => (238,130,238),
        "magenta" => (255,0,255),
        "fuchsia" => (255,0,255),
        "orchid" => (218,112,214),
        "mediumvioletred" => (199,21,133),
        "palevioletred" => (219,112,147),
        "deeppink" => (255,20,147),
        "hotpink" => (255,105,180),
        "lightpink" => (255,182,193),
        "pink" => (255,192,203),
        "antiquewhite" => (250,235,215),
        "beige" => (245,245,220),
        "bisque" => (255,228,196),
        "blanchedalmond" => (255,235,205),
        "wheat" => (245,222,179),
        "cornsilk" => (255,248,220),
        "lemonchiffon" => (255,250,205),
        "lightgoldenrodyellow" => (250,250,210),
        "lightyellow" => (255,255,224),
        "saddlebrown" => (139,69,19),
        "sienna" => (160,82,45),
        "chocolate" => (210,105,30),
        "peru" => (205,133,63),
        "sandybrown" => (244,164,96),
        "burlywood" => (222,184,135),
        "tan" => (210,180,140),
        "rosybrown" => (188,143,143),
        "moccasin" => (255,228,181),
        "navajowhite" => (255,222,173),
        "peachpuff" => (255,218,185),
        "mistyrose" => (255,228,225),
        "lavenderblush" => (255,240,245),
        "linen" => (250,240,230),
        "oldlace" => (253,245,230),
        "papayawhip" => (255,239,213),
        "seashell" => (255,245,238),
        "mintcream" => (245,255,250),
        "slategray" => (112,128,144),
        "lightslategray" => (119,136,153),
        "lightsteelblue" => (176,196,222),
        "lavender" => (230,230,250),
        "floralwhite" => (255,250,240),
        "aliceblue" => (240,248,255),
        "ghostwhite" => (248,248,255),
        "honeydew" => (240,255,240),
        "ivory" => (255,255,240),
        "azure" => (240,255,255),
        "snow" => (255,250,250),
        "black" => (0,0,0),
        "dimgray" => (105,105,105),
        "dimgrey" => (105,105,105),
        "gray" => (128,128,128),
        "grey" => (128,128,128),
        "darkgray" => (169,169,169),
        "darkgrey" => (169,169,169),
        "silver" => (192,192,192),
        "lightgray" => (211,211,211),
        "lightgrey" => (211,211,211),
        "gainsboro" => (220,220,220),
        "whitesmoke" => (245,245,245),
        "white" => (255,255,255),
        _ => fallback
    }
}

// Converts a color tuple
// into a comma separated string
pub fn color_to_string(c: (u8, u8, u8)) -> String
{
    format!("{},{},{}", c.0, c.1, c.2)
}

// Generates a random RGB tuple
pub fn random_color() -> (u8, u8, u8)
{
    let mut rng = rand::thread_rng();
    let mut v: Vec<u8> = vec![];

    for _ in 1..=3
    {
        let n: u8 = rng.gen(); v.push(n);
    }

    (v[0], v[1], v[2])
}

// Parses a color code
// Replaces common color names to RGB,
// Or returns the provided RGB values
pub fn parse_color(ans: &str, reference: (u8, u8, u8)) -> (u8, u8, u8)
{
    let ans = ans.trim().to_lowercase().replace(" ", "");

    match &ans[..]
    {
        // Check if color should be darker or lighter
        "darker" | "darker1" => make_color_darker(reference, DEGREES_1),
        "darker2" => make_color_darker(reference, DEGREES_2),
        "darker3" => make_color_darker(reference, DEGREES_3),
        "lighter" | "lighter1" => make_color_lighter(reference, DEGREES_1),
        "lighter2" => make_color_lighter(reference, DEGREES_2),
        "lighter3" => make_color_lighter(reference, DEGREES_3),
        "random" => random_color(),
        _ => 
        {
            // If not then check if it's an RGB value
            if ans.contains(',')
            {
                let v: Vec<u8> = ans.split(',')
                    .map(|n| n.parse::<u8>().unwrap_or(0)).collect();

                if v.len() != 3 {return reference} (v[0], v[1], v[2])
            }

            else
            {
                // If not then check if it's a color name
                get_color_name_rgb(&ans, reference)
            }
        }
    }
}

// Wrapper function to make a color darker
fn make_color_darker(t: (u8, u8, u8), amount: f64) -> (u8, u8, u8)
{
    change_color_lightness(t, true, amount)
}

// Wrapper function to make a color lighter
fn make_color_lighter(t: (u8, u8, u8), amount: f64) -> (u8, u8, u8)
{
    change_color_lightness(t, false, amount)
}

// Turns a color a darker or lighter
pub fn change_color_lightness(t: (u8, u8, u8), darker: bool, amount: f64) -> (u8, u8, u8)
{
    // Get RGB struct
    let rgb = Rgb::from
    ((
        f64::from(t.0),
        f64::from(t.1),
        f64::from(t.2)
    ));

    // Convert to HSL
    let mut hsl = Hsl::from(&rgb);
    let current_lightness = hsl.get_lightness();

    if darker
    {
        // Decrease lightness by the specified degrees
        let mut lightness = current_lightness - amount;
        if lightness < 0.0 {lightness = 0.0}
        hsl.set_lightness(lightness);
    }

    else
    {
        // Increase lightness by the specified degrees
        let mut lightness = current_lightness + amount;
        if lightness > 359.0 {lightness = 359.0}
        hsl.set_lightness(lightness);
    }

    // Convert back to RGB
    let rgb2 = Rgb::from(&hsl);

    // Return RGB tuple
    (
        rgb2.get_red().round() as u8, 
        rgb2.get_green().round() as u8, 
        rgb2.get_blue().round() as u8
    )
}