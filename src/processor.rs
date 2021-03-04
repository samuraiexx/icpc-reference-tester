use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::scraper::Scraper;
use crate::test_result::*;

pub async fn find_and_process_files(dir: &Path) -> bool {
    let file_paths = find_files(dir);

    println!("found {} tests at {}\n", file_paths.len(), dir.canonicalize().unwrap().display());

    let mut process_futures = Vec::new();
    for path in file_paths.into_iter() {
        process_futures.push(process_file(path));
    }

    let test_results = futures::future::join_all(process_futures).await;

    let mut passed = 0;
    let mut failed = 0;
    let mut ignored = 0;

    for test_result in test_results {
        match test_result {
            TestResult::Accepted => passed  += 1,
            TestResult::Ignored  => ignored += 1,
            _                    => failed  += 1,
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

    for entry in fs::read_dir(dir).unwrap() {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_dir() {
                let subfolder = find_files(&path);
                files.extend(subfolder);
            } else if path.is_file() {
                files.push(path);
            } else {
                panic!("invalid path: {}", path.to_str().unwrap());
            }
        } else {
            panic!("failed on entry");
        }
    }

    files
}

async fn process_file(path: PathBuf) -> TestResult {
    // This shouldn't fail. We check if it's a file before calling this function and OsStr to_str
    // shouldn't fail (not sure when it fails. Maybe some weird unicode? It's not documented)
    let file_name = path.file_name().unwrap().to_str().unwrap();

    // Check if it's an ignored file
    if file_name.starts_with("_") {
        println!("{} ... {}", path.display(), TestResult::Ignored);
        return TestResult::Ignored;
    }

    // Check if it's a test file
    if !file_name.ends_with(".cpp") {
        println!("{} ... {}", path.display(), TestResult::ParsingError(ParsingError::WrongExtension));
        return TestResult::ParsingError(ParsingError::WrongExtension);
    }

    let (problem_url, processed_file_content) = match process_file_content(&path) {
        Ok(returns) => returns,
        Err(err) => {
            let test_result = TestResult::ParsingError(err);
            println!("{} ... {}", path.display(), test_result);
            return test_result;
        }
    };

    let result = tokio::task::spawn_blocking(move || {
        let mut scraper = Scraper::new();
        scraper.submit(problem_url.as_str(), processed_file_content.as_str())
    }).await.unwrap();

    println!("{} ... {}", path.display(), result);
    result
}

// Process file
// Finds the C++ comment with "@problem_url: <problem_url>"
// @XXX ideally we should force it to be in the first line of the file

// Returns problem_url and processed_file_content
fn process_file_content(path: &Path) -> Result<(String, String), ParsingError> {
    // Read file content to memory
    let file_content = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(err) => return Err(ParsingError::IoError(err)),
    };

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

                            if let Some(url) = iter.next() {
                                problem_url = Some(url.to_string());
                            }
                        },

                        "@include:" => {
                            if let Some(inc) = iter.next() {
                                // @Future maybe accept comma separated includes
                                //inc.trim_end_matches(",");

                                match fs::read_to_string(inc) {
                                    Err(err) => {
                                        return Some(Err(
                                            ParsingError::IncludeError(inc.to_string(), err)
                                        ));
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
            Path::new("test/test_folder/_template.test.cpp")
        )?;
        assert_eq!(problem_url, "https://codeforces.com/contest/1083/problem/E");
        Ok(())
    }

    #[tokio::test]
    async fn test_process_template_file() {
        let test_result = process_file(
            Path::new("test/test_folder/_template.test.cpp").to_path_buf()
        ).await;

        assert_eq!(test_result, TestResult::NotAccepted);
    }

    /*
    // @TODO create this test
    #[test]
    fn test_find_files() {
        let files = find_files(Path::new("test/test_folder"));
    }
    */

}
