use std::env;
use std::fmt;
use std::error::Error;
use log::debug;
use std::collections::HashMap;
use reqwest::Method;

#[derive(Debug)]
struct MattermostError {
    msg: String,
}

impl fmt::Display for MattermostError {
    fn fmt(&self, f:&mut fmt::Formatter) -> fmt::Result {
        write!(f, "MattermostError: {}", self.msg)
    }
}

impl Error for MattermostError {}

struct MattermostSession {
    host: String,
    login_id: String,
    user_id: String,
    token: String,
    client: reqwest::Client,
}

#[derive(Debug, serde::Deserialize)]
struct TeamsUnreadDataItems {
    team_id: String,
    msg_count: u32,
    mention_count: u32,
}

impl MattermostSession {
    async fn new(host: &str, login_id: &str, password: &str)
    -> Result<MattermostSession, Box<dyn std::error::Error>> {
        let mut map = HashMap::new();
        map.insert("login_id", login_id);
        map.insert("password", password);

        let uri = format!("http://{}/api/v4/users/login", host);
        let client = reqwest::Client::new();
        let req = client.post(&uri)
            .json(&map);

        let response = req.send().await?;
        debug!("response.status = {:?}", response.status());

        if !response.status().is_success() {
            return Err(Box::new(
                MattermostError {
                    msg: response.text().await?,
                }
            ))
        }

        let token = response.headers().get("token").unwrap().to_str()?.to_string();
        let data: serde_json::Value = response.json().await?;
        let user_id = data["id"].to_string().replace(r#"""#, "");
        debug!("id = {}", user_id);

        Ok(
            MattermostSession {
                host: host.to_string(),
                login_id: login_id.to_string(),
                user_id: user_id,
                token: token.to_string(),
                client: client,
            }
        )
    }

    async fn call_endpoint(&self, method: reqwest::Method, endpoint: &str)
    -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let uri = format!("http://{}/api/v4{}", self.host, endpoint);
        let response = self.client.request(method, &uri)
            .bearer_auth(self.token.clone())
            .send().await?;

        if !response.status().is_success() {
            return Err(Box::new(
                MattermostError {
                    msg: response.text().await?,
                }
            ))
        }

        let data: serde_json::Value = response.json().await?;
        Ok(data)
    }

    async fn get_me(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let data = self.call_endpoint(Method::GET, "/users/me").await?;
        Ok(data)
    }

    async fn get_unread(&self) -> Result<u32, Box<dyn std::error::Error>> {
        let mut n: u32 = 0;
        debug!("userid = {}", self.user_id);
        let endpoint = format!("/users/{}/teams/unread", self.user_id);
        let data = self.call_endpoint(Method::GET, &endpoint).await?;

        let s = data.to_string();
        debug!("s = {:?}", s);
        let v: Vec<TeamsUnreadDataItems> = serde_json::from_str(&s)?;
        for item in &v {
            n += item.msg_count;
        }
        Ok(n)
    }

    async fn logout(self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let data = self.call_endpoint(Method::POST, "/users/logout").await?;
        Ok(data)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let host = &args[1];
    let user = &args[2];
    let password = &args[3];
    let session = MattermostSession::new(host, user, password).await?;
    debug!("getme = {:?}", session.get_me().await?);
    let unread = session.get_unread().await?;
    println!("unread: {:?}", unread);
    session.logout().await?;
    Ok(())
}
