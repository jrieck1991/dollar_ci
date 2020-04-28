use crate::models::{HandlersErr, Result};

use chrono::Utc;
use reqwest::header::{HeaderMap, ACCEPT, AUTHORIZATION, USER_AGENT};
use reqwest::{Client, ClientBuilder, Response, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize, Serialize, Debug)]
struct InstallToken {
    token: String,
}

pub struct GithubClient {
    http_client: Client,
}

// an http client that talks to the github api
impl GithubClient {
    // init new GithubClient
    pub fn new() -> Result<GithubClient> {
        // default headers for all requests
        let mut headers = HeaderMap::new();
        headers.insert(
            ACCEPT,
            "application/vnd.github.machine-man-preview+json"
                .parse()
                .unwrap(),
        );
        headers.insert(USER_AGENT, "dollar-ci".parse().unwrap());

        // build new client with headers
        let client = ClientBuilder::new().default_headers(headers).build()?;

        Ok(GithubClient {
            http_client: client,
        })
    }

    // tell github to create 'check_run'
    pub async fn check_run_create(
        &self,
        name: &str,
        head_sha: &str,
        url: &str,
        installation_id: u64,
    ) -> Result<StatusCode> {
        // get installation token
        let token = self.get_installation_token(&name, installation_id).await?;

        // create body
        let body = json!({"name": name,"head_sha": head_sha});

        // send post
        let res = self
            .http_client
            .post(url)
            .json(&body)
            .header(AUTHORIZATION, format!("token {}", token))
            .send()
            .await?;

        // validate response is successful, log error response and exit
        match log_response("check_run_create", res).await {
            None => Err(HandlersErr::NotFound),
            Some(res) => Ok(res.status()),
        }
    }

    // mark 'check_run' as 'in_progress'
    pub async fn check_run_start(
        &self,
        name: &str,
        url: &str,
        installation_id: u64,
    ) -> Result<StatusCode> {
        // get installation token
        let token = self.get_installation_token(&name, installation_id).await?;

        // create request body
        let body =
            json!({"name": name, "status": "in_progress", "started_at": Utc::now().timestamp()});

        // send post
        let res = self
            .http_client
            .post(url)
            .json(&body)
            .header(AUTHORIZATION, format!("token {}", token))
            .send()
            .await?;

        // validate response is successful, log error response and exit
        match log_response("check_run_start", res).await {
            None => Err(HandlersErr::NotFound),
            Some(res) => Ok(res.status()),
        }
    }

    // mark 'check_run' as 'complete' with either a fail or pass
    pub async fn check_run_complete(
        &self,
        name: &str,
        url: &str,
        success: bool,
        installation_id: u64,
    ) -> Option<HandlersErr> {
        // get installation token
        let token = match self.get_installation_token(&name, installation_id).await {
            Ok(token) => token,
            Err(e) => match e {
                HandlersErr::Json(e) => return Some(HandlersErr::Json(e)),
                HandlersErr::Client(e) => return Some(HandlersErr::Client(e)),
                HandlersErr::Jwt(e) => return Some(HandlersErr::Jwt(e)),
                HandlersErr::Io(e) => return Some(HandlersErr::Io(e)),
                HandlersErr::NotFound => return Some(HandlersErr::NotFound),
            },
        };

        // define success param
        let mut conclusion = String::from("success");
        if !success {
            conclusion = String::from("failure");
        };

        // create body
        let body = json!({"name": name, "status": "completed", "conclusion": conclusion, "completed_at": Utc::now().timestamp()});

        // send post
        let res = match self
            .http_client
            .post(url)
            .json(&body)
            .header(AUTHORIZATION, format!("token {}", token))
            .send()
            .await
        {
            Ok(res) => res,
            Err(e) => {
                error!("check_run_complete error: {}\nrequest_body: {}", e, &body);
                return Some(HandlersErr::Client(e));
            }
        };

        // validate response is successful, log error response and exit
        match log_response("check_run_complete", res).await {
            None => return Some(HandlersErr::NotFound),
            Some(_) => None,
        }
    }
    // get_installation_token will create a jwt token from a pem file
    // use as bearer in request to generate installation token
    async fn get_installation_token(&self, name: &str, installation_id: u64) -> Result<String> {
        debug!("attempting to retrieve installation token for {}", name);

        // create jwt token
        let jwt_token = jwt::create(
            name,
            String::from("/home/ec2-user/dollar-ci.2020-04-18.private-key.pem"),
        )?;

        // form url
        let url = format!(
            "https://api.github.com/app/installations/{}/access_tokens",
            installation_id
        );

        // send post with jwt token
        let res = self
            .http_client
            .post(&url)
            .bearer_auth(jwt_token)
            .send()
            .await?;

        // validate response is successful, log error response and exit
        let success_res = match log_response("get_installation_token", res).await {
            None => return Err(HandlersErr::NotFound),
            Some(success_res) => success_res,
        };

        // get installation access token from successful response
        match success_res.json::<InstallToken>().await {
            Ok(install_token) => {
                debug!(
                    "successfully retrieved installation access token for {}",
                    name
                );
                Ok(install_token.token)
            }
            Err(e) => Err(HandlersErr::Client(e)),
        }
    }
}

// log_response will log response errors, only returns the Reponse type
// if the request was successful so that we can do additional processing
async fn log_response(name: &str, response: Response) -> Option<Response> {
    // if no error, return the response given
    match &response.error_for_status_ref() {
        Ok(_) => return Some(response),
        Err(e) => error!("{} response error code: {:?}", name, e.status()),
    };

    // if error log the response body error message
    match response.text().await {
        Ok(content) => error!("{} response error body: {}", name, content),
        Err(e) => error!("{} response body parse error: {:?}", name, e),
    };

    None
}

mod jwt {
    use crate::models::Result;
    use chrono::{Duration, Utc};
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use serde::{Deserialize, Serialize};
    use std::fs;

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        iat: i64,
        sub: String,
        company: String,
        iss: u64,
        exp: i64,
    }

    // create jwt from pem file
    pub fn create(name: &str, pem_path: String) -> Result<String> {
        // read pem file into string var
        let pem = fs::read_to_string(pem_path)?;

        // get current time in UTC
        let now = Utc::now();

        // JWT token expiration_time must be <= 10 minutes
        let expiration_time = now + Duration::minutes(9);

        // define claims
        let claims = Claims {
            iat: now.timestamp(),
            sub: name.to_string(),
            iss: 61447, // app id given by github
            company: String::from("dollar-ci"),
            exp: expiration_time.timestamp(),
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
