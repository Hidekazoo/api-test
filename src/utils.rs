use reqwest::Method;
use serde::Deserialize;
use std::{fs::File, io::BufReader};

#[derive(Deserialize, Debug)]
pub struct TestCase {
    pub case: String,
    pub method: String,
    pub headers: Option<Vec<Header>>,
    pub path: String,
    pub expected: Expected,
}

#[derive(Deserialize, Debug)]
pub struct Header {
    pub key: String,
    pub value: String,
}

#[derive(Deserialize, Debug)]
pub struct Expected {
    pub status: u16,
    pub response: Option<serde_json::Value>,
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

pub async fn execute_test_case(
    base_url: &str,
    test_case: &TestCase,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let method = parse_method(&test_case.method);
    let url = format!("{}{}", base_url, test_case.path);

    let mut request = client.request(method, &url);
    if let Some(headers) = &test_case.headers {
        for header in headers {
            request = request.header(header.key.as_str(), header.value.as_str());
        }
    }
    let response = request.send().await?;
    assert_test_case(response, test_case).await
}

async fn assert_test_case(
    response: reqwest::Response,
    test_case: &TestCase,
) -> Result<(), Box<dyn std::error::Error>> {
    if response.status() != test_case.expected.status {
        println!(
            "\x1b[31m FAIL \x1b[0m{} - Status code mismatch: expected={}, actual={}",
            test_case.case,
            test_case.expected.status,
            response.status()
        );
        return Ok(());
    }

    if let Some(expected_response) = &test_case.expected.response {
        match response.text().await {
            Ok(body) => {
                let parsed_response = match serde_json::from_str::<serde_json::Value>(&body) {
                    Ok(json) => json,
                    Err(e) => {
                        println!(
                            "\x1b[31m FAIL \x1b[0m{} - JSON parse error: {}",
                            test_case.case, e
                        );
                        println!("Raw response: {}", body);
                        return Ok(());
                    }
                };

                let expected_value = match expected_response {
                    serde_json::Value::String(json_str) => {
                        match serde_json::from_str::<serde_json::Value>(json_str) {
                            Ok(parsed) => parsed,
                            Err(_) => expected_response.clone(), // パースに失敗したら元の値を使用
                        }
                    }
                    _ => expected_response.clone(),
                };

                if parsed_response == expected_value {
                    println!("\x1b[32m PASS \x1b[0m{}", test_case.case);
                } else {
                    println!(
                        "\x1b[31m FAIL \x1b[0m{} - Response body mismatch",
                        test_case.case
                    );

                    let expected_str = serde_json::to_string_pretty(&expected_value)?;
                    let actual_str = serde_json::to_string_pretty(&parsed_response)?;

                    println!("\x1b[33m>> DIFFERENCES:\x1b[0m");

                    if let (Some(expected_obj), Some(actual_obj)) =
                        (expected_value.as_object(), parsed_response.as_object())
                    {
                        for (key, exp_val) in expected_obj {
                            if let Some(act_val) = actual_obj.get(key) {
                                if exp_val != act_val {
                                    println!("  Key: \x1b[36m{}\x1b[0m", key);
                                    println!("    Expected: \x1b[32m{}\x1b[0m", exp_val);
                                    println!("    Actual:   \x1b[31m{}\x1b[0m", act_val);
                                }
                            } else {
                                println!("  Key: \x1b[36m{}\x1b[0m", key);
                                println!("    Expected: \x1b[32m{}\x1b[0m", exp_val);
                                println!("    Actual:   \x1b[31m<missing>\x1b[0m");
                            }
                        }

                        // 期待されていないキー
                        for key in actual_obj.keys() {
                            if !expected_obj.contains_key(key) {
                                println!("  Key: \x1b[36m{}\x1b[0m", key);
                                println!("    Expected: \x1b[32m<not expected>\x1b[0m");
                                println!(
                                    "    Actual:   \x1b[31m{}\x1b[0m",
                                    actual_obj.get(key).unwrap()
                                );
                            }
                        }
                    }

                    // 完全な出力
                    println!("\n\x1b[33m>> COMPLETE VALUES:\x1b[0m");
                    println!("  Expected: \n\x1b[32m{}\x1b[0m", expected_str);
                    println!("  Actual: \n\x1b[31m{}\x1b[0m", actual_str);

                    return Ok(());
                }
            }
            Err(e) => {
                println!(
                    "\x1b[31m FAIL \x1b[0m{} - Failed to get response: {}",
                    test_case.case, e
                );
                return Ok(());
            }
        }
    } else {
        println!("\x1b[32m PASS \x1b[0m{}", test_case.case);
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
