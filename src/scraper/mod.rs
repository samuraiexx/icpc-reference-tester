mod codeforces;
mod spoj;

use headless_chrome::{Browser, LaunchOptions, Tab};
use std::{fs::write, time::Duration};
use std::{path::PathBuf, sync::Arc};
use uuid::Uuid;

use crate::test_result::{SubmissionError, TestResult};

enum Judge {
    Codeforces,
    Spoj,
}

fn login(tab: &Arc<Tab>, judge: &Judge) {
    let max_wait = 65.0;
    let mut wait_time = 2.0;

    loop {
        let success = match judge {
            Judge::Codeforces => codeforces::try_login(tab),
            Judge::Spoj => spoj::try_login(tab),
        }
        .is_ok();

        if success {
            break;
        }

        if wait_time > max_wait {
            panic!("Couldn't login into judge");
        }

        std::thread::sleep(Duration::from_secs_f32(wait_time * rand::random::<f32>()));
        wait_time = wait_time * 2.0;
    }
}

fn save_temporary_file(file: &str, uuid: &str) -> PathBuf {
    if std::fs::read_dir("tmp").is_err() {
        std::fs::create_dir("tmp").unwrap();
    }

    let path = std::env::current_dir()
        .unwrap()
        .join(format!("tmp/{}.cpp", uuid));
    write(&path, file).unwrap();

    path
}

fn find_judge(url: &str) -> Option<Judge> {
    if url.to_lowercase().contains("codeforces") {
        return Some(Judge::Codeforces);
    }

    if url.to_lowercase().contains("spoj") {
        return Some(Judge::Spoj);
    }

    None
}

pub fn submit(url: &str, input_file: &str) -> TestResult {
    let browser = Browser::new(
        LaunchOptions::default_builder()
            // .headless(false) // Uncomment to test
            .build()
            .expect("Could not find chrome-executable"),
    )
    .unwrap();
    let tab = browser.wait_for_initial_tab().unwrap();
    let judge = match find_judge(url) {
        Some(judge) => judge,
        None => return TestResult::SubmissionError(SubmissionError::JudgeNotSupported),
    };

    login(&tab, &judge);

    let submission;

    let max_wait = 1000.0;
    let mut wait_time = 3.0;

    loop {
        let uuid = Uuid::new_v4().to_string();
        let input_file = format!("{}\n// UUID: {}", input_file, uuid);
        let path = save_temporary_file(input_file.as_str(), uuid.as_str());

        let submission_try = match judge {
            Judge::Codeforces => codeforces::try_submit(&tab, url, uuid.as_str(), &path),
            Judge::Spoj => spoj::try_submit(&tab, url, uuid.as_str(), &path),
        };

        std::fs::remove_file(path).unwrap();

        match submission_try {
            Ok(submission_try) => {
                submission = submission_try;
                break;
            }
            Err(_) => {}
        };

        if wait_time > max_wait {
            return TestResult::SubmissionError(SubmissionError::Timeout);
        }

        std::thread::sleep(Duration::from_secs_f32(wait_time * rand::random::<f32>()));
        wait_time = wait_time * 10.0;
    }

    submission
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codeforces_wa() {
        let problem = std::fs::read_to_string(r".\test\1131A-WA.cpp").unwrap();

        let result = submit(
            "https://codeforces.com/contest/1131/problem/A",
            problem.as_str(),
        );

        match result {
            TestResult::NotAccepted => (),
            _ => panic!("Wrong Result"),
        }
    }

    #[test]
    fn test_codeforces_ac() {
        let problem = std::fs::read_to_string(r".\test\1131A.cpp").unwrap();

        let result = submit(
            "https://codeforces.com/contest/1131/problem/A",
            problem.as_str(),
        );

        match result {
            TestResult::Accepted => (),
            _ => panic!("Wrong Result"),
        }
    }

    #[test]
    fn test_spoj_wa() {
        let problem = std::fs::read_to_string(r".\test\1131A-WA.cpp").unwrap();

        let result = submit("https://www.spoj.com/problems/FENTREE/", problem.as_str());

        match result {
            TestResult::NotAccepted => (),
            _ => panic!("Wrong Result"),
        }
    }

    #[test]
    fn test_spoj_ac() {
        let problem = std::fs::read_to_string(r".\test\FENTREE.cpp").unwrap();

        let result = submit("https://www.spoj.com/problems/FENTREE/", problem.as_str());

        match result {
            TestResult::Accepted => (),
            _ => panic!("Wrong Result"),
        }
    }
}
