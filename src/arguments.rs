use crate::
{
    s, p,
    globals::
    {
        VERSION,
        g_set_path,
        g_set_arg_page_size,
        g_set_arg_row_space,
        g_set_arg_color_1,
        g_set_arg_color_2,
        g_set_arg_color_3,
        g_set_arg_use_colors
    },
    file::
    {
        read_file,
        get_source_content,
        get_default_file_path,
        shell_expand
    },
    structs::
    {
        SettingsArgs
    },
    other::
    {
        exit
    },
    notes::
    {
        format_note,
        get_notes
    }
};

use clap::{App, Arg};

// Starts the argument system and responds to actions
pub fn check_arguments()
{
    let matches = App::new("Effer")
    .version(VERSION)
    .about("Encrypted CLI Notepad")
    .arg(Arg::with_name("PATH")
        .help("Use a custom file path")
        .required(false)
        .index(1))
    .arg(Arg::with_name("print")
        .long("print")
        .multiple(false)
        .help("Prints all the notes instead of entering the program"))
    .arg(Arg::with_name("print2")
        .long("print2")
        .multiple(false)
        .help("Same as print but with separator newlines"))
    .arg(Arg::with_name("fprint")
        .long("fprint")
        .multiple(false)
        .help("Same as print but it formats the notes"))
    .arg(Arg::with_name("fprint2")
        .long("fprint2")
        .multiple(false)
        .help("Same as fprint but with separator newlines"))
    .arg(Arg::with_name("fprint3")
        .long("fprint3")
        .multiple(false)
        .help("Same as fprint but it disables indentation"))
    .arg(Arg::with_name("fprint4")
        .long("fprint4")
        .multiple(false)
        .help("Same as fprint3 but with separator newlines"))
    .arg(Arg::with_name("config")
        .long("config")
        .value_name("Path")
        .help("Use a config TOML file to import settings")
        .takes_value(true))
    .arg(Arg::with_name("path")
        .long("path")
        .value_name("Path")
        .help("Use a custom file path")
        .takes_value(true))
    .arg(Arg::with_name("source")
        .long("source")
        .value_name("Path")
        .help("Creates notes from a text file")
        .takes_value(true))
    .arg(Arg::with_name("page_size")
        .long("page_size")
        .value_name("Multiple of 5")
        .help("Set the page size setting")
        .takes_value(true))
    .arg(Arg::with_name("row_space")
        .long("row_space")
        .value_name("true|false")
        .help("Set the row space setting")
        .takes_value(true))
    .arg(Arg::with_name("color_1")
        .long("color_1")
        .value_name("r,g,b")
        .help("Set the color 1 setting")
        .takes_value(true))
    .arg(Arg::with_name("color_2")
        .long("color_2")
        .value_name("r,g,b")
        .help("Set the color 2 setting")
        .takes_value(true))
    .arg(Arg::with_name("color_3")
        .long("color_3")
        .value_name("r,g,b")
        .help("Set the color 3 setting")
        .takes_value(true))
    .arg(Arg::with_name("use_colors")
        .long("use_colors")
        .value_name("true|false")
        .help("Set the use colors setting")
        .takes_value(true))
    .get_matches();

    let path;

    // Check for normal path argument
    if let Some(pth) = matches.value_of("PATH")
    {
        path = s!(pth);
    }

    // If not check for option path argument
    else if let Some(pth) = matches.value_of("path")
    {
        path = s!(pth);
    }

    else
    {
        // Else use default path
        path = s!(get_default_file_path().to_str().unwrap());
    }

    g_set_path(shell_expand(&path));

    let mut print_mode = "";

    if matches.occurrences_of("print") > 0
    {
        print_mode = "print";
    }

    else if matches.occurrences_of("print2") > 0
    {
        print_mode = "print2";
    }

    else if matches.occurrences_of("fprint") > 0
    {
        print_mode = "fprint";
    }

    else if matches.occurrences_of("fprint2") > 0
    {
        print_mode = "fprint2";
    }

    else if matches.occurrences_of("fprint3") > 0
    {
        print_mode = "fprint3";
    }

    else if matches.occurrences_of("fprint4") > 0
    {
        print_mode = "fprint4";
    }

    if print_mode == "print" || print_mode == "print2"
    || print_mode == "fprint" || print_mode == "fprint2"
    || print_mode == "fprint3" || print_mode == "fprint4"
    {
        let notes = get_notes(false);

        match print_mode
        {
            "print" => 
            {
                p!(notes.lines().collect::<Vec<&str>>().join("\n"));
            },
            "print2" =>
            {
                p!(notes.lines().collect::<Vec<&str>>().join("\n\n"));
            },
            "fprint" => 
            {
                p!(notes.lines().skip(1).enumerate().skip(1)
                    .map(|(i, n)| format_note(&(i, s!(n)), false, 0, true))
                    .collect::<Vec<String>>()
                    .join("\n"));
            },
            "fprint2" =>
            {
                p!(notes.lines().skip(1).enumerate().skip(1)
                    .map(|(i, n)| format_note(&(i, s!(n)), false, 0, true))
                    .collect::<Vec<String>>()
                    .join("\n\n"));
            }
            "fprint3" =>
            {
                p!(notes.lines().skip(1).enumerate().skip(1)
                    .map(|(i, n)| format_note(&(i, s!(n)), false, 0, false))
                    .collect::<Vec<String>>()
                    .join("\n"));
            },
            "fprint4" =>
            {
                p!(notes.lines().skip(1).enumerate().skip(1)
                    .map(|(i, n)| format_note(&(i, s!(n)), false, 0, false))
                    .collect::<Vec<String>>()
                    .join("\n\n"));
            }
            _ => {}
        }

        exit();
    }

    if let Some(path) = matches.value_of("source")
    {
        get_source_content(path);
    }

    // Settings

    if let Some(path) = matches.value_of("config")
    {
        if let Ok(text) = read_file(path)
        {
            if let Ok(tom) = toml::from_str(&text)
            {
                let sets: SettingsArgs = tom;

                if let Some(x) = sets.page_size
                {
                    g_set_arg_page_size(x);
                }

                if let Some(x) = sets.row_space
                {
                    g_set_arg_row_space(x);
                }

                if let Some(x) = sets.color_1
                {
                    g_set_arg_color_1(x);
                }

                if let Some(x) = sets.color_2
                {
                    g_set_arg_color_2(x);
                }

                if let Some(x) = sets.color_3
                {
                    g_set_arg_color_3(x);
                }

                if let Some(x) = sets.use_colors
                {
                    g_set_arg_use_colors(x);
                }
            }
        }
    }

    else
    {
        if let Some(x) = matches.value_of("page_size")
        {
            g_set_arg_page_size(s!(x));
        }

        if let Some(x) = matches.value_of("row_space")
        {
            g_set_arg_row_space(s!(x));
        }

        if let Some(x) = matches.value_of("color_1")
        {
            g_set_arg_color_1(s!(x));
        }

        if let Some(x) = matches.value_of("color_2")
        {
            g_set_arg_color_2(s!(x));
        }

        if let Some(x) = matches.value_of("color_3")
        {
            g_set_arg_color_3(s!(x));
        }

        if let Some(x) = matches.value_of("use_colors")
        {
            g_set_arg_use_colors(s!(x));
        }
    }
}