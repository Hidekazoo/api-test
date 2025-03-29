use reqwest::Method;
use serde::Deserialize;
use std::{fs::File, io::BufReader};

#[derive(Deserialize, Debug)]
pub struct TestCase {
    pub case: String,
    pub method: String,
    pub path: String,
    pub expected: Expected,
}

#[derive(Deserialize, Debug)]
pub struct Expected {
    pub status: u16,
    pub response: serde_json::Value,
}

pub fn parse_method(method_str: &str) -> Method {
    match method_str.to_uppercase().as_str() {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        _ => Method::GET,
    }
}

pub async fn execute_test_case(base_url: &str, test_case: &TestCase) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let method = parse_method(&test_case.method);
    let url = format!("{}{}", base_url, test_case.path);

    let response = client.request(method, &url).send().await?;
    if response.status() == test_case.expected.status {
        println!("\x1b[32m PASS \x1b[0m{}", test_case.case);
    } else {
        println!("\x1b[31m FAIL \x1b[0m{}", test_case.case);
    }
    
    Ok(())
}

pub fn read_yaml<T>(filepath: &str) -> Result<T, Box<dyn std::error::Error>> 
where
    T: for<'de> Deserialize<'de>,
{
    let file = File::open(filepath)?;
    let reader = BufReader::new(file);
    let data: T = serde_yaml::from_reader(reader)?;
    Ok(data)
} 