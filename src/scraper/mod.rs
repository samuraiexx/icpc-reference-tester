use headless_chrome::{Browser, LaunchOptions, Tab};
use std::path::PathBuf;
use std::{env, fs::write, time::Duration};
use uuid::Uuid;

#[derive(Debug)]
pub struct SubmissionError;

pub struct Scraper {
    _browser: Browser,
    tab: std::sync::Arc<Tab>,
}

impl Scraper {
    pub fn new() -> Scraper {
        let browser = Browser::new(
            LaunchOptions::default_builder()
                // .headless(false)
                .build()
                .expect("Could not find chrome-executable"),
        )
        .unwrap();
        let tab = browser.wait_for_initial_tab().unwrap();

        let mut scraper = Scraper {
            tab,
            _browser: browser,
        };
        scraper.login();

        scraper
    }

    fn login(&mut self) {
        let tab = &self.tab;

        loop {
            tab.navigate_to("https://codeforces.com/enter").unwrap();

            // Type User
            tab.wait_for_element_with_custom_timeout(
                "input#handleOrEmail",
                Duration::from_secs(300),
            )
            .unwrap()
            .click()
            .unwrap();
            tab.type_str(env::var("CF_USER").unwrap().as_str()).unwrap();

            // Type Password
            tab.wait_for_element("input#password")
                .unwrap()
                .click()
                .unwrap();
            tab.type_str(env::var("CF_PASSWORD").unwrap().as_str())
                .unwrap()
                .press_key("Enter")
                .unwrap();

            let logged_in = tab
                .wait_for_element_with_custom_timeout(
                    r#"a[href$="/logout"]"#,
                    Duration::from_secs(20),
                )
                .is_ok();

            if logged_in {
                break;
            }
        }
    }

    fn save_temporary_file(file: &str, uuid: &str) -> PathBuf {
        let path = std::env::current_dir()
            .unwrap()
            .join(format!("tmp_{}.cpp", uuid));
        write(&path, file).unwrap();

        path
    }

    pub fn submit(&mut self, url: &str, input_file: &str) -> Result<(), SubmissionError> {
        let uuid = Uuid::new_v4().to_string();
        let input_file = format!("{}\n// UUID: {}", input_file, uuid);
        let path = Self::save_temporary_file(input_file.as_str(), uuid.as_str());

        let tab = &self.tab;
        tab.navigate_to(url).unwrap();

        tab.wait_for_element_with_custom_timeout(
            r#"[name="sourceFile"]"#,
            Duration::from_secs(300),
        )
        .unwrap()
        .set_input_files(&[path.to_str().unwrap()])
        .unwrap();

        tab.wait_for_element(r#".submit[value="Submit"]"#)
            .unwrap()
            .click()
            .unwrap();

        tab.wait_until_navigated().unwrap();

        std::thread::sleep(Duration::from_secs(1));
        let submissions = tab
            .wait_for_elements("[data-submission-id]")
            .unwrap()
            .into_iter()
            .map(|element| {
                let attributes = element.get_attributes().unwrap().unwrap();
                attributes.get("data-submission-id").unwrap().clone()
            })
            .collect::<Vec<_>>();

        let mut current_submission = None;

        for submission in submissions {
            tab.find_element(format!(r#"tr[data-submission-id="{}"]"#, submission).as_str())
                .unwrap()
                .find_element(".id-cell > a")
                .unwrap()
                .click()
                .unwrap();
            tab.wait_until_navigated().unwrap();
            std::thread::sleep(Duration::from_secs(1));

            let popup = tab
                .wait_for_element_with_custom_timeout("div.popup", Duration::from_secs(300))
                .unwrap();
            let is_current_submission = popup.get_inner_text().unwrap().contains(uuid.as_str());

            popup.find_element(".close").unwrap().click().unwrap();
            tab.wait_until_navigated().unwrap();
            std::thread::sleep(Duration::from_secs(1));

            if is_current_submission {
                current_submission = Some(submission);
                break;
            }
        }

        let current_submission = match current_submission {
            None => panic!("No current submission"),
            Some(submission) => submission,
        };

        std::fs::remove_file(path).unwrap();

        // Waits for result
        loop {
            let status = tab
                .find_element(
                    format!(r#"[submissionid="{}"][waiting]"#, current_submission).as_str(),
                )
                .unwrap();

            let attributes = status.get_attributes().unwrap().unwrap();
            let waiting = attributes.get("waiting").unwrap();

            if waiting == "false" {
                match status.find_element(".verdict-accepted") {
                    Ok(_) => return Ok(()),
                    Err(_) => return Err(SubmissionError),
                }
            }
            std::thread::sleep(Duration::from_secs(1));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;

    #[test]
    fn test_multiple_submissions() {
        let problem_540_a = std::fs::read_to_string(
            r"C:\Users\mateu\Desktop\icpc-notebook\icpc-reference-tester\test\540A.cpp",
        )
        .unwrap();
        let problem_540_a_wa = std::fs::read_to_string(
            r"C:\Users\mateu\Desktop\icpc-notebook\icpc-reference-tester\test\540A-WA.cpp",
        )
        .unwrap();

        let children = (0..10)
            .map(|i| {
                let problem = match i % 2 {
                    0 => problem_540_a.clone(),
                    1 => problem_540_a_wa.clone(),
                    _ => panic!(),
                };

                thread::spawn(move || {
                    let mut scraper = Scraper::new();

                    let result = scraper.submit(
                        "https://codeforces.com/contest/1131/problem/A",
                        problem.as_str(),
                    );

                    if result.is_ok() as u32 == i % 2 {
                        panic!("Wrong Veredict");
                    }

                    match result {
                        Ok(()) => println!("Problem {}: ACCEPTED", i),
                        Err(_) => println!("Problem {}: WRONG ANSWER", i),
                    }
                })
            })
            .collect::<Vec<_>>();

        for child in children {
            child.join().unwrap();
        }
    }
}
