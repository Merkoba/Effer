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
    Nothing,
    AddNote,
    EditNote,
    EditLast,
    FindNotes,
    DeleteNotes,
    RemakeFile,
    ChangePassword,
    CycleLeft,
    CycleRight,
    FirstPage,
    LastPage,
    RefreshPage,
    PageNumber,
    Exit
}