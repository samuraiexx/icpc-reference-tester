mod scraper;
use scraper::*;

fn main() {
    let mut scraper = Scraper::new();
    scraper.login();
    scraper
        .submit(
            "https://codeforces.com/contest/1131/problem/A",
            r"C:\Users\mateu\Desktop\icpc-notebook\icpc-reference-tester\test\540A.cpp",
        )
        .expect("Did not pass the test =s");
}
