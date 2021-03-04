#[derive(PartialEq, Eq, Debug)]
pub enum TestResult {
    Accepted,
    NotAccepted,
    Ignored,
    SubmissionError(SubmissionError),
    ParsingError(ParsingError),
}

#[derive(PartialEq, Eq, Debug)]
pub enum SubmissionError {
    Timeout,
    JudgeNotSupported,
}

#[derive(PartialEq, Eq, Debug)]
pub enum ParsingError {
    NoUrl,
    MultipleUrls,
    IncludeNotFound,
    WrongExtension,
}

impl std::fmt::Display for TestResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestResult::Accepted             => write!(f, "OK"),
            TestResult::NotAccepted          => write!(f, "FAILED: wrong verdict"),
            TestResult::Ignored              => write!(f, "IGNORED"),
            TestResult::SubmissionError(err) => write!(f, "FAILED: {}", err),
            TestResult::ParsingError(err)    => write!(f, "FAILED: {}", err),
        }
    }
}

impl std::fmt::Display for SubmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubmissionError::Timeout           => write!(f, "submission timeout"),
            SubmissionError::JudgeNotSupported => write!(f, "judge not supported"),
        }
    }
}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsingError::NoUrl           => write!(f, "no problem_url tag"),
            ParsingError::MultipleUrls    => write!(f, "multiple problem_url tags"),
            ParsingError::IncludeNotFound => write!(f, "include file not found"),
            ParsingError::WrongExtension  => write!(f, "wrong test file extension (not cpp)"),
        }
    }
}
