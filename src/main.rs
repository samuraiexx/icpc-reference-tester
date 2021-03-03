use std::path::Path;
use tokio;

mod processor;
mod scraper;
mod verdicts;

#[tokio::main]
async fn main() -> Result<(), ()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        let program_name = args[0].as_str();
        println!("Missing tests path");
        println!("Usage: {} <tests path>", program_name);
        return Err(());
    }

    let test_ok = processor::find_and_process_files(Path::new(&args[1])).await;
    if test_ok { Err(()) } else { Ok(()) }
}
