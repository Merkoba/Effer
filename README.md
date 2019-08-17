![](http://i.imgur.com/XriIvL7.png)

![](http://i.imgur.com/9Xmz8We.png)

![](http://i.imgur.com/6Dbs9Rj.png)

This is a small CLI program to store notes that will be saved to a text file in gibberish hex format.

To read the notes you need to use the password used to encrypt the file within the program.

It can, edit, and delete notes.

It can delete using a single number, a list, or a range.

The file can be remade (replaced with an empty one).

The password can be changed.

Instead of displaying all notes at once there are pages of 20 notes.

Pages can be changed using arrows, home/end, or numbers.

Up arrow is a shortcut to edit the last note.

It goes into an alternative screen to not flood your terminal.

Notes can be swapped.

The number of displayed notes per page can be configured.

All notes can be shown at once.

Arguments allow the program to just output note terminal (with the correct password).
This allows piping output to other programs like grep.

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

# Arguments
Check --help to find out about arguments that can be used.

# Contributing
Contributions are welcome. Making it more secure can be a way to improve it.
If the encryption method changes, the program gets a new major release.
Major release jumps (like 1.0.0 to 2.0.0) means files created in one can't
be opened in another one.