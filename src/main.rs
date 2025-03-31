use std::env;

use serde::Deserialize;

mod utils;
use utils::{execute_test_case, read_yaml, TestCase};

#[derive(Deserialize, Debug)]
struct Config {
    base_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <config_file> <test_file>", args[0]);
        std::process::exit(1);
    }

    let config_file = &args[1];
    let test_file = &args[2];

    let config: Config = read_yaml(config_file)?;
    let test_cases: Vec<TestCase> = read_yaml(test_file)?;

    let base_url = &config.base_url;
    for test_case in test_cases.iter() {
        execute_test_case(base_url, test_case).await?;
    }
    Ok(())
}
