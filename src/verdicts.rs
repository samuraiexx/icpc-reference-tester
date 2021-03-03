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
    #[allow(dead_code)]
    pub fn accepted(&self) -> bool {
        match self {
            Verdict::Accepted => true,
            _ => false,
        }
    }
}

impl std::fmt::Display for Verdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Verdict::Accepted => write!(f, "... OK"),
            Verdict::NotAccepted => write!(f, "... FAILED: wrong verdict"),
            Verdict::Timeout => write!(f, "... FAILED: timeout"),
            Verdict::Ignored => write!(f, "... IGNORED"),
            Verdict::ParsingError(err) => write!(f, "... FAILED: {}", err),
        }
    }
}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsingError::NoUrl => write!(f, "no problem_url tag"),
            ParsingError::MultipleUrls => write!(f, "multiple problem_url tags"),
            ParsingError::IncludeNotFound => write!(f, "include file not found"),
            ParsingError::WrongExtension => write!(f, "wrong test file extension"),
        }
    }
}
