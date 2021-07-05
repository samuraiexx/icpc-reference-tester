use crate::test_result::TestResult;

use headless_chrome::Tab;
use std::{env, time::Duration};
use std::{path::PathBuf, sync::Arc};

pub fn try_login(tab: &Arc<Tab>) -> Result<(), failure::Error> {
    tab.navigate_to("https://www.spoj.com/login")?;

    // Type User
    tab.wait_for_element_with_custom_timeout("input#inputUsername", Duration::from_secs(300))?
        .click()?;
    tab.type_str(env::var("SPOJ_USER").unwrap().as_str())?;

    // Type Password
    tab.wait_for_element_with_custom_timeout("input#inputPassword", Duration::from_secs(300))?
        .click()?;
    tab.type_str(env::var("SPOJ_PASSWORD").unwrap().as_str())?
        .press_key("Enter")?;

    tab.wait_for_element_with_custom_timeout(".username_dropdown", Duration::from_secs(300))?;

    Ok(())
}

pub fn try_submit(
    tab: &Arc<Tab>,
    url: &str,
    uuid: &str,
    path: &PathBuf,
) -> Result<TestResult, failure::Error> {
    tab.navigate_to(url)?;
    tab.wait_for_element_with_custom_timeout("#problem-btn-submit", Duration::from_secs(300))?
        .click()?;

    tab.wait_for_element_with_custom_timeout("#subm_file", Duration::from_secs(300))?
        .set_input_files(&[path.to_str().unwrap()])?;

    let javascript_set_cpp_lang = r#"
      function() {
        let selector = document.querySelector("select#lang");
        selector.selectedIndex = [...selector.options].findIndex(option => option.label.includes("C++14")); 
      }
    "#;

    tab.find_element("select#lang")?
        .call_js_fn(javascript_set_cpp_lang, vec![], false)?;

    tab.find_element("#submit")?.click()?;

    tab.wait_for_element_with_custom_timeout(
        ".problems.table.newstatus",
        Duration::from_secs(300),
    )?;

    tab.find_element(".username_dropdown")?.click()?;
    tab.find_element(".fa-history")?.click()?;

    let submissions = tab
        .wait_for_elements(".statustext > [data-sid]")?
        .into_iter()
        .map(|element| {
            let attributes = element.get_attributes().unwrap().unwrap();
            attributes.get("data-sid").unwrap().clone()
        })
        .collect::<Vec<_>>();

    let mut current_submission = None;

    for submission in submissions {
        tab.find_element(format!(r#"[data-sid="{}"]"#, submission).as_str())?
            .click()?;
        let is_current_submission = tab
            .wait_for_elements(".modal-body ol li div")?
            .into_iter()
            .map(|element| element.get_inner_text().unwrap().contains(uuid))
            .any(|x| x == true);

        tab.find_element(".close").unwrap().click()?;
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
            format!(r#"#statusres_{}"#, current_submission).as_str(),
            Duration::from_secs(20),
        )?;

        let attributes = status.get_attributes().unwrap().unwrap();
        let is_final = attributes.get("final").unwrap();

        if is_final == "1" {
            if status.get_inner_text()?.contains("accepted") {
                return Ok(TestResult::Accepted);
            } else {
                return Ok(TestResult::NotAccepted);
            }
        }

        std::thread::sleep(Duration::from_secs(2));
    }
}
