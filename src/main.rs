use std::io;

mod processor;
mod scraper;

fn main() -> io::Result<()> {
    /*
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        let program_name = args[0].as_str();
        println!("Missing tests path");
        println!("Usage: {} <tests path>", program_name);
        return Err(io::Error::new(io::ErrorKind::InvalidInput, ""));
    }
    */

    //find_and_process_files(Path::new("test_folder"))?;

    Ok(())
}
