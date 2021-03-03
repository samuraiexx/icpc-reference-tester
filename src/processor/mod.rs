use std::{
    io,
    fs,
    path::Path,
};

use crate::scraper::Scraper;

pub fn find_and_process_files(dir: &Path) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            // @TODO don't fail fast
            find_and_process_files(&path)?;
        } else {
            process_file(&path)?;
        }
    }

    Ok(())
}

fn process_file(path: &Path) -> io::Result<()> {
    println!("processing file: {}", path.to_str().unwrap());

    let create_err = || {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("file doesn't have the test file extension: {}", path.to_str().unwrap())
        )
    };

    let file_name = path.file_name()
        .ok_or_else(|| create_err())? // wrong error
        .to_str()
        .ok_or_else(|| create_err())?; // wrong error

    // Check if it's an ignored file
    if file_name.starts_with("_") {
        // @TODO use log
        println!("file ignored: {}", path.to_str().unwrap());
        return Ok(());
    }

    // Check if it's a test file
    if !file_name.ends_with(".test.cpp") { return Err(create_err()); }

    let (problem_url, processed_file_content) = process_file_content(path)?;

    let mut scraper = Scraper::new();
    match scraper.submit(problem_url.as_str(), processed_file_content.as_str()) {
        Ok(_) => {},
        Err(_) => {}
    }

    Ok(())
}

// Process file
// Finds the C++ comment with "@problem_url: <problem_url>"
// @XXX ideally we should force it to be in the first line of the file
// @TODO include dependencies (@include)

// Returns problem_url and processed_file_content
fn process_file_content(path: &Path) -> io::Result<(String, String)> {
    // Read file content to memory
    let file_content = fs::read_to_string(path)?;

    let create_err = || {
        io::Error::new(
                io::ErrorKind::InvalidData,
                format!("test file has multiple problem urls: {}", path.to_str().unwrap())
        )
    };

    let mut problem_url = Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!("test file has no problem url: {}", path.to_str().unwrap())
    ));

    let processed_file_content: String = file_content
        .lines()
        .filter_map(|line| {
            let mut iter = line.split_whitespace();

            if let Some(s) = iter.next() {
                if s != "//" { return Some(Ok(line.to_string())); }

                while let Some(s) = iter.next() {
                    match s {
                        "@problem_url:" => {
                            if problem_url.is_ok() {
                                return Some(Err(create_err()));
                            }

                            if let Some(url) = iter.find(|s| !s.is_empty()) {
                                problem_url = Ok(url.to_string());
                            }
                        },

                        "@include:" => {
                            if let Some(inc) = iter.find(|s| !s.is_empty()) {
                                // @Future maybe accept comma separated includes
                                //inc.trim_end_matches(",");

                                match fs::read_to_string(inc) {
                                    Err(err) => return Some(Err(err)), // @TODO return a better error message
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

    let problem_url = problem_url?;
    Ok((problem_url, processed_file_content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_template_file_content() -> io::Result<()> {
        let (problem_url, _processed_file_content) = process_file_content(
            Path::new("test/test_folder/template.test.cpp")
        )?;
        assert_eq!(problem_url, "https://codeforces.com/contest/1083/problem/E");
        Ok(())
    }

    #[test]
    fn test_process_template_file() -> io::Result<()> {
        process_file(
            Path::new("test/test_folder/template.test.cpp")
        )?;
        Ok(())
    }

    #[test]
    fn test_find_and_process_files() -> io::Result<()> {
        find_and_process_files(Path::new("test/test_folder"))?;
        Ok(())
    }
}
