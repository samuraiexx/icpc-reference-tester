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
                .headless(false) // Uncomment to test
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

    fn try_login(&mut self) -> Result<(), failure::Error> {
        let tab = &self.tab;

        tab.navigate_to("https://codeforces.com/enter")?;

        // Type User
        tab.wait_for_element_with_custom_timeout("input#handleOrEmail", Duration::from_secs(300))?
            .click()?;
        tab.type_str(env::var("CF_USER")?.as_str())?;

        // Type Password
        tab.wait_for_element("input#password")?.click()?;
        tab.type_str(env::var("CF_PASSWORD").unwrap().as_str())?
            .press_key("Enter")?;

        tab.wait_for_element_with_custom_timeout(r#"a[href$="/logout"]"#, Duration::from_secs(20))?;

        Ok(())
    }

    fn login(&mut self) {
        let max_wait = 65.0;
        let mut wait_time = 2.0;

        loop {
            if self.try_login().is_ok() {
                break;
            }

            if wait_time > max_wait {
                panic!("Exponential backoff max wait time exceded");
            }

            std::thread::sleep(Duration::from_secs_f32(wait_time * rand::random::<f32>()));
            wait_time = wait_time * 2.0;
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
        let submission;

        let max_wait = 1000.0;
        let mut wait_time = 3.0;

        loop {
            let uuid = Uuid::new_v4().to_string();
            let input_file = format!("{}\n// UUID: {}", input_file, uuid);
            let path = Self::save_temporary_file(input_file.as_str(), uuid.as_str());

            let submission_try = self.try_submit(url, uuid.as_str(), &path);
            std::fs::remove_file(path).unwrap();

            match submission_try {
                Ok(submission_try) => {
                    submission = submission_try;
                    break;
                }
                Err(err) => println!("{:?}", err),
            };

            if wait_time > max_wait {
                panic!("Exponential backoff max wait time exceded");
            }

            std::thread::sleep(Duration::from_secs_f32(wait_time * rand::random::<f32>()));
            wait_time = wait_time * 10.0;
        }

        submission
    }

    fn try_submit(
        &mut self,
        url: &str,
        uuid: &str,
        path: &PathBuf,
    ) -> Result<Result<(), SubmissionError>, failure::Error> {
        let tab = &self.tab;
        tab.navigate_to(url)?;

        tab.wait_for_element_with_custom_timeout(
            r#"[name="sourceFile"]"#,
            Duration::from_secs(300),
        )?
        .set_input_files(&[path.to_str().unwrap()])?;

        tab.wait_for_element(r#".submit[value="Submit"]"#)?
            .click()?;

        tab.wait_until_navigated()?;

        std::thread::sleep(Duration::from_secs(2));
        let submissions = tab
            .wait_for_elements("[data-submission-id]")?
            .into_iter()
            .map(|element| {
                let attributes = element.get_attributes().unwrap().unwrap();
                attributes.get("data-submission-id").unwrap().clone()
            })
            .collect::<Vec<_>>();

        let mut current_submission = None;

        for submission in submissions {
            tab.find_element(format!(r#"tr[data-submission-id="{}"]"#, submission).as_str())?
                .find_element(".id-cell > a")?
                .click()?;
            tab.wait_until_navigated()?;
            std::thread::sleep(Duration::from_secs(1));

            let popup =
                tab.wait_for_element_with_custom_timeout("div.popup", Duration::from_secs(300))?;
            let is_current_submission = popup.get_inner_text()?.contains(uuid);

            popup.find_element(".close")?.click()?;
            tab.wait_until_navigated()?;
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

        // Waits for result
        loop {
            let status = tab.find_element(
                format!(r#"[submissionid="{}"][waiting]"#, current_submission).as_str(),
            )?;

            let attributes = status.get_attributes().unwrap().unwrap();
            let waiting = attributes.get("waiting").unwrap();

            if waiting == "false" {
                match status.find_element(".verdict-accepted") {
                    Ok(_) => return Ok(Ok(())),
                    Err(_) => return Ok(Err(SubmissionError)),
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
