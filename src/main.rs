use std::{env, fs::File, io::BufReader};

use serde::Deserialize;


#[derive(Deserialize, Debug)]
struct TestCases {
    test_cases: Vec<TestCase>,
}

#[derive(Deserialize, Debug)]
struct TestCase {
    case: String,
}



fn main() {
    let args: Vec<String> = env::args().collect();
    let filepath = &args[1];

    let data = read_yaml(filepath);
    println!("{:?}", data.unwrap());
    
}
    
fn read_yaml(filepath: &String) -> Result<TestCases, Box<dyn std::error::Error>> {
    let file = File::open(filepath).expect("File not found");
    let reader = BufReader::new(file);
    let data: TestCases = serde_yaml::from_reader(reader).expect("Error while reading file");
    Ok(data)
}
