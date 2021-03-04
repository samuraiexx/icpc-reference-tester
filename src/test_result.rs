#[derive(Debug)]
pub enum TestResult {
    Accepted,
    NotAccepted,
    Ignored,
    SubmissionError(SubmissionError),
    ParsingError(ParsingError),
}

#[derive(Debug)]
pub enum SubmissionError {
    Timeout,
    JudgeNotSupported,
}

#[derive(Debug)]
pub enum ParsingError {
    NoUrl,
    MultipleUrls,
    IncludeError(String, std::io::Error),
    WrongExtension,
    IoError(std::io::Error),
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
            ParsingError::NoUrl          => write!(f, "no problem_url tag"),
            ParsingError::MultipleUrls   => write!(f, "multiple problem_url tags"),
            ParsingError::WrongExtension => write!(f, "wrong test file extension (not cpp)"),
            ParsingError::IncludeError(inc, err) =>
                write!(f, "could not include file {}. Error: \"{}\"", inc, err),
            ParsingError::IoError(err) =>
                write!(f, "could not open test file. Error: \"{}\"", err),
        }
    }
}
