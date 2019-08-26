use std::borrow::Cow::{self, Borrowed, Owned};
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::error::ReadlineError;
use rustyline::{Context, Helper};
use unicode_width::UnicodeWidthStr;
use serde::Deserialize;

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
    FindNotes, SwapNotes, DeleteNotes, ResetFile,
    ChangePassword, CycleLeft, CycleRight, FirstPage,
    LastPage, RefreshPage, PageNumber, ChangeMenu,
    ShowAllNotes, ShowAbout, GotoPage, Exit,
    IncreasePageSize, DecreasePageSize, ShowStats, ScreenSaver,
    FetchSource, OpenFromPath, Destroy, ChangeRowSpace,
    MoveNotes, ChangeColors, FindNotesSuggest, ModeAction,
    AddNoteStart
}

#[derive(Debug, Deserialize)]
pub struct Settings
{
    pub page_size: Option<String>, pub row_space: Option<String>,
    pub color_1: Option<String>, pub color_2: Option<String>, 
    pub color_3: Option<String>
}

pub struct RustyHelper 
{
    pub masking: bool,
    pub completer: FilenameCompleter
}

impl Highlighter for RustyHelper 
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

impl Completer for RustyHelper 
{
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        self.completer.complete(line, pos, ctx)
    }
}

impl Hinter for RustyHelper {}
impl Helper for RustyHelper {}