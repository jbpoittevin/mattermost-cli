// optional feature: json

use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut headers = HashMap::new();
    headers.insert("Accept".to_string(), "application/json".to_string());

    let uri = "http://salon.noisy.dgfip/api/v4/users/login";
    let client = reqwest::Client::new();
    let res: serde_json::Value = client.post(uri)
        .json(&headers)
        .send()
        .await?
        .json()
        .await?;

    println!("{:#?}", res);
    Ok(())
}
