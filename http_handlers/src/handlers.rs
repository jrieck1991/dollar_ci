use serde::{Deserialize, Serialize};

use log::{debug, info, warn, error};

#[derive(Deserialize, Serialize, Debug)]
pub struct Event {
    action: String,
    check_suite: CheckSuite,
    repository: Repo,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CheckSuite {
    id: u64,
    status: String,
    head_sha: String,
    check_runs_url: String,
    app: App,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct App {
    id: u64,
    name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Repo {
    clone_url: String,
}

// Error wrapper for the project
#[derive(Debug)]
pub enum HandlersErr {
    Json(serde_json::Error),
    Client(reqwest::Error),
    Jwt(jsonwebtoken::errors::Error),
    Io(std::io::Error),
}

impl From<reqwest::Error> for HandlersErr {
    fn from(err: reqwest::Error) -> HandlersErr {
        HandlersErr::Client(err)
    }
}

impl From<serde_json::Error> for HandlersErr {
    fn from(err: serde_json::Error) -> HandlersErr {
        HandlersErr::Json(err)
    }
}

impl From<jsonwebtoken::errors::Error> for HandlersErr {
    fn from(err: jsonwebtoken::errors::Error) -> HandlersErr {
        HandlersErr::Jwt(err)
    }
}

impl From<std::io::Error> for HandlersErr {
    fn from(err: std::io::Error) -> HandlersErr {
        HandlersErr::Io(err)
    }
}

// http route filters
pub mod filters {
    use super::handlers;

    use warp::Filter;

    // events listens for github events
    pub fn events() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::post()
            .and(warp::body::json())
            .and_then(handlers::event)
    }
}

// handlers handle github event payloads
mod handlers {
    use super::client;
    use super::Event;

    use std::convert::Infallible;
    use warp::http::StatusCode;

    // handle github event payload
    pub async fn event(event: Event) -> Result<impl warp::Reply, Infallible> {
        debug!("event received: {:?}", event);

        // route event based on action
        match event.action.as_str() {
            "requested" => {
                client::check_run_create(
                    event.check_suite.app.name,
                    event.check_suite.head_sha,
                    event.check_suite.check_runs_url,
                )
                .await;
                Ok(StatusCode::OK)
            }
            "rerequested" => {
                client::check_run_create(
                    event.check_suite.app.name,
                    event.check_suite.head_sha,
                    event.check_suite.check_runs_url,
                )
                .await;
                Ok(StatusCode::OK)
            }
            "created" => {
                client::check_run_start(
                    event.check_suite.app.name,
                    event.check_suite.check_runs_url,
                )
                .await;
                Ok(StatusCode::OK)
            }
            _ => {
                warn!("no match for event: {:?}", event);
                Ok(StatusCode::BAD_REQUEST)
            }
        }
    }
}

// http client
mod client {

    use super::jwt;
    use super::HandlersErr;
    use serde_json::*;
    use time::Instant;

    // tell github to create 'check_run'
    pub async fn check_run_create(
        name: String,
        head_sha: String,
        url: String,
    ) -> Option<HandlersErr> {
        // create jwt token
        let token = match jwt::create(
            &name,
            String::from("/home/ec2-user/dollar-ci.2020-04-18.private-key.pem"),
        ) {
            Ok(token) => token,
            Err(e) => return Some(e),
        };

        // init http client
        let client = reqwest::Client::new();

        // create body
        let body = json!({"name": name,"head_sha": head_sha});

        // send post
        match client
            .post(&url)
            .json(&body)
            .bearer_auth(token)
            .send()
            .await
        {
            Ok(res) => info!("check_run_create status_code: {}", res.status()),
            Err(e) => {
                error!("check_run_create_error: {}", e);
                return Some(HandlersErr::Client(e))
            }
        };

        None
    }

    // mark 'check_run' as 'in_progress'
    pub async fn check_run_start(name: String, url: String) -> Option<HandlersErr> {
        // create jwt token
        let token = match jwt::create(
            &name,
            String::from("/home/ec2-user/dollar-ci.2020-04-18.private-key.pem"),
        ) {
            Ok(token) => token,
            Err(e) => return Some(e),
        };

        // init http client
        let client = reqwest::Client::new();

        // create body
        let body = json!({"name": name, "status": "in_progress", "started_at": format!("{:?}", Instant::now())});

        // send post
        match client
            .post(&url)
            .json(&body)
            .bearer_auth(token)
            .send()
            .await
        {
            Ok(res) => info!("check_run_start status_code: {}", res.status()),
            Err(e) => {
                error!("check_run_start error: {}", e);
                return Some(HandlersErr::Client(e))
            }
        };

        None
    }

    // mark 'check_run' as 'complete' with either a fail or pass
    pub async fn check_run_complete(
        name: String,
        url: String,
        success: bool,
    ) -> Option<HandlersErr> {
        // create jwt token
        let token = match jwt::create(
            &name,
            String::from("/home/ec2-user/dollar-ci.2020-04-18.private-key.pem"),
        ) {
            Ok(token) => token,
            Err(e) => {
                error!("jwt::create error: {:?}", e);
                return Some(e)
            }
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
        match client
            .post(&url)
            .json(&body)
            .bearer_auth(token)
            .send()
            .await
        {
            Ok(res) => {
                info!("check_run_complete status_code: {}", res.status());
                None
            }
            Err(e) => {
                error!("check_run_complete error: {}", e);
                Some(HandlersErr::Client(e))
            },
        }
    }
}

// JWT formation module
mod jwt {
    use super::HandlersErr;
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use serde::{Deserialize, Serialize};
    use std::fs;

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        company: String,
        exp: usize,
    }

    // create jwt from pem file
    pub fn create(name: &str, pem_path: String) -> Result<String, HandlersErr> {
        // read pem file into string var
        let pem = match fs::read_to_string(pem_path) {
            Ok(pem) => pem,
            Err(e) => return Err(HandlersErr::Io(e)),
        };

        // define claims
        let claims = Claims {
            sub: name.to_string(),
            company: String::from("dollar-ci"),
            exp: 10000000000,
        };

        // setup header
        let header = Header::new(Algorithm::RS256);

        // create rsa pem from file
        let key = EncodingKey::from_rsa_pem(pem.as_bytes())?;

        // encode token that can be used in http headers
        match encode(&header, &claims, &key) {
            Ok(token) => Ok(token),
            Err(e) => Err(HandlersErr::Jwt(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::filters;
    use super::jwt;
    use std::fs;
    use warp::http::StatusCode;

    #[tokio::test]
    async fn test_events() {
        // get test payload from file
        let payload =
            fs::read_to_string("test_github_payload.json").expect("unable to read file to string");

        // send request
        let resp = warp::test::request()
            .method("POST")
            .body(&payload.as_bytes())
            .reply(&filters::events())
            .await;

        // verify status code
        assert_eq!(resp.status(), StatusCode::OK)
    }

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
