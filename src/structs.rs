use std::borrow::Cow::{self, Borrowed, Owned};
use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::Helper;
use unicode_width::UnicodeWidthStr;

pub enum FilePathCheckResult
{
    DoesNotExist,
    NotAFile,
    Exists
}

impl FilePathCheckResult
{
    pub fn message(&self) -> &str
    {
        match *self
        {
            FilePathCheckResult::Exists => "File exists.",
            FilePathCheckResult::DoesNotExist => "File doesn't exist.",
            FilePathCheckResult::NotAFile => "The path exists but it's not a file."
        }
    }
}

#[derive(Debug)]
pub enum MenuAnswer
{
    Nothing, AddNote, EditNote, EditLastNote,
    FindNotes, SwapNotes, DeleteNotes, RemakeFile,
    ChangePassword, CycleLeft, CycleRight, FirstPage,
    LastPage, RefreshPage, PageNumber, ChangeMenu,
    ShowAllNotes, ShowAbout, GotoPage, Exit,
    IncreasePageSize, DecreasePageSize
}

pub struct MaskingHighlighter 
{
    pub masking: bool
}

impl Highlighter for MaskingHighlighter 
{
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> 
    {
        if self.masking 
        {
            Owned("".repeat(line.width()))
        } 
        
        else 
        {
            Borrowed(line)
        }
    }

    fn highlight_char(&self, _line: &str, _pos: usize) -> bool 
    {
        self.masking
    }
}

impl Completer for MaskingHighlighter 
{
    type Candidate = String;
}

impl Hinter for MaskingHighlighter {}
impl Helper for MaskingHighlighter {}