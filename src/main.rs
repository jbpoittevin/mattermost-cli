// optional feature: json

use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut headers = HashMap::new();
    headers.insert("Accept".to_string(), "application/json".to_string());

    let client = reqwest::Client::new();
    let res = client.post("http://salon.noisy.dgfip/api/v4/users/login")
        .json(&headers)
        .send()
        .await?
        .json::<HashMap<String, String>>()
        .await?;

    println!("{:#?}", res);
    Ok(())
}
