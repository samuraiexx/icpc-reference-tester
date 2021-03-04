use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::scraper::Scraper;
use crate::verdicts::*;


/*
    // ParsingError::MultipleUrls
    io::Error::new(
            io::ErrorKind::InvalidData,
            format!("test file has multiple problem urls: {}", path.to_str().unwrap())
    )

    // ParsingError::NoUrl
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("test file has no problem url: {}", path.to_str().unwrap())
    )

    // ParsingError::WrongExtension
    return Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("file doesn't have the test file extension: {}", path.to_str().unwrap())
    ));
*/

pub async fn find_and_process_files(dir: &Path) -> bool {
    let file_paths = find_files(dir);

    println!("running {} tests\n", file_paths.len());

    let mut process_futures = Vec::new();
    for path in file_paths.into_iter() {
        process_futures.push(process_file(path));
    }

    let verdicts = futures::future::join_all(process_futures).await;

    let mut passed = 0;
    let mut failed = 0;
    let mut ignored = 0;

    for verdict in verdicts {
        match verdict {
            Verdict::Accepted => passed  += 1,
            Verdict::Ignored  => ignored += 1,
            _                 => failed  += 1,
        }
    }

    let test_ok = failed == 0;
    let test_result_str = if test_ok { "ok" } else { "failed" };

    println!(
        "\ntest result: {}. {} passed; {} failed; {} ignored",
        test_result_str,
        passed, failed, ignored
    );

    test_ok
}

// @XXX This was moved outside find_and_process_files because recursived async functions require
//      dyn return. The best solution is to implement the file find iteratively instead of
//      recursively
fn find_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    // @TODO match result
    for entry in fs::read_dir(dir).unwrap() {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_dir() {
                // @TODO don't fail fast
                find_files(&path);
            } else if path.is_file() {
                files.push(path);
            } else {
                // @TODO investigate if this can happen
                panic!("invalid path: {}", path.to_str().unwrap());
            }
        } else {
            // @TODO investigate when this can happen
            panic!("failed on entry");
        }
    }

    files
}

async fn process_file(path: PathBuf) -> Verdict {
    // This shouldn't fail. We check if it's a file before calling this function and OsStr to_str
    // shouldn't fail (not sure when it fails. Maybe some weird unicode? It's not documented)
    let file_name = path.file_name().unwrap().to_str().unwrap();

    // Check if it's an ignored file
    if file_name.starts_with("_") {
        println!("{} {}", file_name, Verdict::Ignored);
        return Verdict::Ignored;
    }

    // Check if it's a test file
    if !file_name.ends_with(".cpp") {
        println!("{} {}", file_name, Verdict::ParsingError(ParsingError::WrongExtension));
        return Verdict::ParsingError(ParsingError::WrongExtension);
    }

    let (problem_url, processed_file_content) = match process_file_content(&path) {
        Ok(returns) => returns,
        Err(err) => {
            let verdict = Verdict::ParsingError(err);
            println!("{} {}", file_name, verdict);
            return verdict;
        }
    };

    let result = tokio::task::spawn_blocking(move || {
        let mut scraper = Scraper::new();
        scraper.submit(problem_url.as_str(), processed_file_content.as_str())
    }).await.unwrap();

    println!("{} {}", file_name, result);
    result
}

// Process file
// Finds the C++ comment with "@problem_url: <problem_url>"
// @XXX ideally we should force it to be in the first line of the file
// @TODO include dependencies (@include)

// Returns problem_url and processed_file_content
fn process_file_content(path: &Path) -> Result<(String, String), ParsingError> {
    // Read file content to memory
    // @TODO match result
    let file_content = fs::read_to_string(path).unwrap();

    let mut problem_url = None;

    let processed_file_content: String = file_content
        .lines()
        .filter_map(|line| {
            let mut iter = line.split_whitespace();

            if let Some(s) = iter.next() {
                if s != "//" { return Some(Ok(line.to_string())); }

                while let Some(s) = iter.next() {
                    match s {
                        "@problem_url:" => {
                            if problem_url.is_some() {
                                return Some(Err(ParsingError::MultipleUrls));
                            }

                            if let Some(url) = iter.find(|s| !s.is_empty()) {
                                problem_url = Some(url.to_string());
                            }
                        },

                        "@include:" => {
                            if let Some(inc) = iter.find(|s| !s.is_empty()) {
                                // @Future maybe accept comma separated includes
                                //inc.trim_end_matches(",");

                                match fs::read_to_string(inc) {
                                    Err(_) => {
                                        // @TODO return a better error message
                                        //       io failure could be many different errors
                                        return Some(Err(ParsingError::IncludeNotFound));
                                    },
                                    Ok(inc_file) => return Some(Ok(inc_file)),
                                }
                            }
                        },

                        _ => {}
                    }
                }

                return None;
            }

            Some(Ok(line.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");

    if problem_url.is_none() {
        return Err(ParsingError::NoUrl);
    }

    Ok((problem_url.unwrap(), processed_file_content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_template_file_content() -> Result<(), ParsingError> {
        let (problem_url, _processed_file_content) = process_file_content(
            Path::new("test/test_folder/template.test.cpp")
        )?;
        assert_eq!(problem_url, "https://codeforces.com/contest/1083/problem/E");
        Ok(())
    }

    #[tokio::test]
    async fn test_process_template_file() {
        let verdict = process_file(
            Path::new("test/test_folder/template.test.cpp").to_path_buf()
        ).await;

        assert_eq!(verdict, Verdict::NotAccepted);
    }

    #[tokio::test]
    async fn test_find_and_process_files() {
        let test_result = find_and_process_files(Path::new("test/test_folder")).await;
        assert_eq!(test_result.passed, 0);
        assert_eq!(test_result.failed, 1);
        assert_eq!(test_result.ignored, 1);
    }
}
