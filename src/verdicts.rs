#[derive(PartialEq, Eq, Debug)]
pub enum Verdict {
    Accepted,
    NotAccepted,
    Timeout,
    Ignored,
    ParsingError(ParsingError),
}

#[derive(PartialEq, Eq, Debug)]
pub enum ParsingError {
    NoUrl,
    MultipleUrls,
    IncludeNotFound,
    WrongExtension,
}

impl Verdict {
    pub fn accepted(&self) -> bool {
        match self {
            Verdict::Accepted => true,
            _ => false,
        }
    }
}
