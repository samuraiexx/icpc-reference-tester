use crate::test_result::TestResult;

use headless_chrome::Tab;
use std::{env, time::Duration};
use std::{path::PathBuf, sync::Arc};

pub fn try_login(tab: &Arc<Tab>) -> Result<(), failure::Error> {
    tab.navigate_to("https://codeforces.com/enter")?;

    // Type User
    tab.wait_for_element_with_custom_timeout("input#handleOrEmail", Duration::from_secs(300))?
        .click()?;
    tab.type_str(env::var("CF_USER").unwrap().as_str())?;

    // Type Password
    tab.wait_for_element("input#password")?.click()?;
    tab.type_str(env::var("CF_PASSWORD").unwrap().as_str())?
        .press_key("Enter")?;

    tab.wait_for_element_with_custom_timeout(r#"a[href$="/logout"]"#, Duration::from_secs(20))?;

    Ok(())
}

pub fn try_submit(
    tab: &Arc<Tab>,
    url: &str,
    uuid: &str,
    path: &PathBuf,
) -> Result<TestResult, failure::Error> {
    tab.navigate_to(url)?;

    tab.wait_for_element_with_custom_timeout(r#"[name="sourceFile"]"#, Duration::from_secs(300))?
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
        let status = tab.wait_for_element_with_custom_timeout(
            format!(r#"[submissionid="{}"][waiting]"#, current_submission).as_str(),
            Duration::from_secs(20),
        )?;

        let attributes = status.get_attributes().unwrap().unwrap();
        let waiting = attributes.get("waiting").unwrap();

        if waiting == "false" {
            match status.find_element(".verdict-accepted") {
                Ok(_) => return Ok(TestResult::Accepted),
                Err(_) => return Ok(TestResult::NotAccepted),
            }
        }

        std::thread::sleep(Duration::from_secs(2));
        tab.reload(false, None)?;
    }
}
