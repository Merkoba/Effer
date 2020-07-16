use crate::{ee, s, structs::RustyHelper};

use rustyline::{
    completion::FilenameCompleter, Cmd, CompletionType, Config, Editor, KeyPress, OutputStreamType,
};

// Centralized function to handle user input
// Can make typing invisible for cases like password input
pub fn get_input(message: &str, initial: &str, mask: bool) -> String {
    let config: Config = Config::builder()
        .keyseq_timeout(50)
        .output_stream(OutputStreamType::Stderr)
        .completion_type(CompletionType::List)
        .build();

    let h = RustyHelper {
        masking: mask,
        completer: FilenameCompleter::new(),
    };
    let mut editor = Editor::with_config(config);
    editor.set_helper(Some(h));
    editor.bind_sequence(KeyPress::Esc, Cmd::Interrupt);
    let prompt = format!("{}: ", message);
    if mask {
        ee!("{}", termion::cursor::Hide)
    }

    let ans = match editor.readline_with_initial(&prompt, (initial, &s!())) {
        Ok(input) => input,
        Err(_) => s!(),
    };

    if mask {
        ee!("{}", termion::cursor::Show)
    }
    ans
}

// Asks the user for a yes/no answer
pub fn ask_bool(message: &str, critical: bool) -> bool {
    let prompt = if critical { " (Y, n)" } else { " (y, n)" };

    loop {
        let ans = get_input(&[message, prompt].concat(), "", false);

        match ans.trim() {
            "y" => {
                if !critical {
                    return true;
                }
            }
            "Y" => return true,
            "n" | "N" => return false,
            "" => return false,
            _ => {}
        }
    }
}

// Asks the user to input a string
pub fn ask_string(message: &str, initial: &str, trim: bool) -> String {
    let ans = get_input(message, initial, false);
    if trim {
        s!(ans.trim())
    } else {
        ans
    }
}
