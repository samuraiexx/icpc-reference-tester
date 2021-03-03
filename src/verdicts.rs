pub enum Verdict {
    Accepted,
    NotAccepted,
    Ignored,
    ParsingError(ParsingError),
}

pub enum ParsingError {
    NoUrl,
    MultipleUrls,
    IncludeNotFound,
    WrongExtension,
}
