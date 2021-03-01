use headless_chrome::{Browser, LaunchOptions, Tab};
use std::{env, fs::write, time::Duration};
use std::{fs::read_to_string, sync::Arc};
use uuid::Uuid;

#[derive(Debug)]
pub struct SubmissionError;

pub struct Scraper {
    _browser: Browser,
    tab: Arc<Tab>,
}

impl Scraper {
    pub fn new() -> Scraper {
        let browser = Browser::new(
            LaunchOptions::default_builder()
                .headless(false)
                .build()
                .expect("Could not find chrome-executable"),
        )
        .unwrap();
        let tab = browser.wait_for_initial_tab().unwrap();
        Scraper {
            _browser: browser,
            tab,
        }
    }

    pub fn login(&mut self) {
        let tab = &self.tab;
        tab.navigate_to("https://codeforces.com/enter").unwrap();
        tab.wait_until_navigated().unwrap();

        // Type User
        tab.wait_for_element_with_custom_timeout("input#handleOrEmail", Duration::from_secs(60))
            .unwrap();
        tab.wait_for_element("input#handleOrEmail")
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

        // Wait for page to load
        std::thread::sleep(Duration::from_secs(2));
    }

    fn set_problem_uuid(path: &str, uuid: &str) {
        let uuid_line = format!("// UUID: {}", uuid);
        let file = read_to_string(path).unwrap();
        let mut file = file.split("\n").collect::<Vec<_>>();

        if file.last().unwrap().contains("UUID") {
            file.pop();
        }

        file.push(uuid_line.as_str());
        let file = file.join("\n");

        write(path, file).unwrap();
    }

    pub fn submit(&mut self, url: &str, input_file: &str) -> Result<(), SubmissionError> {
        let uuid = Uuid::new_v4().to_string();
        Self::set_problem_uuid(input_file, uuid.as_str());

        let tab = &self.tab;
        tab.navigate_to(url).unwrap();

        tab.wait_for_element_with_custom_timeout(r#"[name="sourceFile"]"#, Duration::from_secs(60))
            .unwrap()
            .set_input_files(&[input_file])
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
                .wait_for_element_with_custom_timeout("div.popup", Duration::from_secs(60))
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
            println!("Waiting for judge...");
            std::thread::sleep(Duration::from_secs(1));
        }
    }
}
