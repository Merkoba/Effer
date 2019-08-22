![](https://i.imgur.com/NVi4vPV.png)

This is a small CLI program to store notes that will be saved to a text file in gibberish hex format.

To read the notes you need to use the password used to encrypt the file within the program.

It can, edit, find, and delete notes.

It can find using a case insensitive regex.

Last find filter remembered.

It can delete using a single number, a list, a range, or a case insensitive regex.

The file can be remade (replaced with an empty one).

The password can be changed.

Instead of displaying all notes at once there are pages of 20 notes.

Pages can be changed using arrows, home/end, or numbers.

Shortcut to edit the last note.

Last edited note number remembered.

It goes into an alternative screen to not flood your terminal.

Notes can be swapped.

The number of displayed notes per page can be configured.

All notes can be shown at once.

Input page number to go to a specific page.

Sreensaver mode to temporarily hide notes from curious eyes.

Arguments allow the program to just output note terminal (with the correct password).
This allows piping output to other programs like grep.

Can create notes from a given file path (replace, append, or prepend).

Can change to other encrypted files within the program.

Destroy function which overwrites the file several times and exits the program.

Path autocompletion with Tab. 

~ (Home) and environment variables expansion.

Row spacing can be enabled or disabled.

Color themes can be changed.

# About Security

This is not to be taken as a real security tool. The encryption is relatively weak.
Somebody that knows what they're doing might be able to decrypt it without much trouble.
But still it's an upgrade from saving notes in plain text.
No other files are created for decryption. This means the note files are portable and 
should be able to be opened with the same program on another machine using the same password.

# Installation

Install Rust: https://www.rust-lang.org/tools/install

To check using a debug version:
>cargo run

To build an optimized binary
>cargo build --release

Or just use the binary provided in the latest release.

# Arguments
Check --help to find out about arguments that can be used.

If running the program with cargo, use `cargo run -- --help`.

# Contributing
Contributions are welcome. Making it more secure can be a way to improve it.
The only restriction to the encryption algorithm is that it can't use multiple files,
for instance to save salts or IV's. The idea is to make it easy to move files around,
and open them in other machines where the user might have this program. Also since the 
note files are saved locally it would be easy anyway to gather those other extra files
if access to the file system is possible.
If the encryption method changes, the program gets a new major release.
Major-release-jumps (like 1.0.0 to 2.0.0) mean that files created in one can't
be opened in the other.