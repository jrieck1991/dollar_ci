// http client
pub mod client {

    use super::jwt;
    use crate::models::{Result, HandlersErr};

    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use time::Instant;
    use reqwest::{StatusCode, Client};

    #[derive(Deserialize, Serialize, Debug)]
    struct InstallToken {
        token: String,
    }

    // tell github to create 'check_run'
    pub async fn check_run_create(
        name: &str,
        head_sha: &str,
        url: &str,
        installation_id: u64,
    ) -> Result<StatusCode> {
        // get installation token
        let token = get_installation_token(&name, installation_id).await?;

        // init http client
        let client = Client::new();

        // create body
        let body = json!({"name": name,"head_sha": head_sha});

        // send post
        Ok(client
            .post(url)
            .json(&body)
            .bearer_auth(token)
            .send()
            .await?
            .status())
    }

    // mark 'check_run' as 'in_progress'
    pub async fn check_run_start(
        name: &str,
        url: &str,
        installation_id: u64,
    ) -> Result<reqwest::StatusCode> {
        // get installation token
        let token = get_installation_token(&name, installation_id).await?;

        // init http client
        let client = reqwest::Client::new();

        // create body
        let body = json!({"name": name, "status": "in_progress", "started_at": format!("{:?}", Instant::now())});

        // send post
        Ok(client
            .post(url)
            .json(&body)
            .bearer_auth(token)
            .send()
            .await?
            .status())
    }

    // mark 'check_run' as 'complete' with either a fail or pass
    pub async fn check_run_complete(
        name: &str,
        url: &str,
        success: bool,
        installation_id: u64,
    ) -> Option<HandlersErr> {
        // get installation token
        let token = match get_installation_token(&name, installation_id).await {
            Ok(token) => token,
            Err(e) => match e {
                HandlersErr::Json(e) => return Some(HandlersErr::Json(e)),
                HandlersErr::Client(e) => return Some(HandlersErr::Client(e)),
                HandlersErr::Jwt(e) => return Some(HandlersErr::Jwt(e)),
                HandlersErr::Io(e) => return Some(HandlersErr::Io(e)),
                HandlersErr::NotFound => return Some(HandlersErr::NotFound),
            },
        };

        // init http client
        let client = reqwest::Client::new();

        // define success param
        let mut conclusion = String::from("success");
        if !success {
            conclusion = String::from("failure");
        };

        // create body
        let body = json!({"name": name, "status": "completed", "conclusion": conclusion, "completed_at": format!("{:?}", Instant::now())});

        // send post
        match client.post(url).json(&body).bearer_auth(token).send().await {
            Ok(res) => {
                info!("check_run_complete status_code: {}", res.status());
                None
            }
            Err(e) => {
                error!("check_run_complete error: {}\nrequest_body: {}", e, &body);
                Some(HandlersErr::Client(e))
            }
        }
    }

    // get_installation_token will create a jwt token from a pem file
    // use as bearer in request to generate installation token
    pub async fn get_installation_token(name: &str, installation_id: u64) -> Result<String> {
        // create jwt token
        let jwt_token = jwt::create(
            name,
            String::from("/home/ec2-user/dollar-ci.2020-04-18.private-key.pem"),
        )?;

        // init http client
        let client = reqwest::Client::new();

        // form url
        let url = format!(
            "https://api.github.com/app/installations/{}/access_tokens",
            installation_id
        );

        // send post with jwt token
        let res = client.post(&url).bearer_auth(jwt_token).send().await?;

        Ok("no".to_string())
    }
}

mod jwt {
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use serde::{Deserialize, Serialize};
    use std::fs;
    use crate::models::{Result};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        company: String,
        iss: u64,
        exp: usize,
    }

    // create jwt from pem file
    pub fn create(name: &str, pem_path: String) -> Result<String> {
        // read pem file into string var
        let pem = fs::read_to_string(pem_path)?;

        // define claims
        let claims = Claims {
            sub: name.to_string(),
            iss: 61447, // app id given by github
            company: String::from("dollar-ci"),
            exp: 10000000000, // TODO update to 10 minutes
        };

        // setup header
        let header = Header::new(Algorithm::RS256);

        // create rsa pem from file
        let key = EncodingKey::from_rsa_pem(pem.as_bytes())?;

        // encode token that can be used in http headers
        Ok(encode(&header, &claims, &key)?)
    }
}

#[cfg(test)]
mod tests {
    use super::jwt;

    #[test]
    fn jwt_create() {
        match jwt::create(
            "unit",
            String::from("../build/dollar-ci.2020-04-18.private-key.pem"),
        ) {
            Ok(token) => println!("{}", token),
            Err(e) => panic!(e),
        }
    }

}
