use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::error;
use std::fmt;
use std::result;

#[derive(Deserialize, Serialize, Debug)]
pub struct Event {
    action: String,
    check_suite: CheckSuite,
    repository: Repo,
    installation: Installation,
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

#[derive(Deserialize, Serialize, Debug)]
pub struct Installation {
    id: u64,
}

// result type alias
type Result<T> = result::Result<T, HandlersErr>;

// Error wrapper for the project
#[derive(Debug)]
pub enum HandlersErr {
    Json(serde_json::Error),
    Client(reqwest::Error),
    Jwt(jsonwebtoken::errors::Error),
    Io(std::io::Error),
}

impl fmt::Display for HandlersErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HandlersErr::Json(ref err) => write!(f, "json error: {}", err),
            HandlersErr::Client(ref err) => write!(f, "client error: {}", err),
            HandlersErr::Jwt(ref err) => write!(f, "jwt error: {}", err),
            HandlersErr::Io(ref err) => write!(f, "IO error: {}", err),
        }
    }
}

// implement the Error trait
impl error::Error for HandlersErr {
    fn cause(&self) -> Option<&error::Error> {
        match *self {
            HandlersErr::Json(ref err) => Some(err),
            HandlersErr::Client(ref err) => Some(err),
            HandlersErr::Jwt(ref err) => Some(err),
            HandlersErr::Io(ref err) => Some(err),
        }
    }
}

// implement From for each error to be wrapped
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
                    event.installation.id,
                )
                .await;
                Ok(StatusCode::OK)
            }
            "rerequested" => {
                client::check_run_create(
                    event.check_suite.app.name,
                    event.check_suite.head_sha,
                    event.check_suite.check_runs_url,
                    event.installation.id,
                )
                .await;
                Ok(StatusCode::OK)
            }
            "created" => {
                client::check_run_start(
                    event.check_suite.app.name,
                    event.check_suite.check_runs_url,
                    event.installation.id,
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
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use time::Instant;
    use super::Result;

    #[derive(Deserialize, Serialize, Debug)]
    struct InstallToken {
        token: String,
    }

    // tell github to create 'check_run'
    pub async fn check_run_create(
        name: String,
        head_sha: String,
        url: String,
        installation_id: u64,
    ) -> Option<HandlersErr> {
        // create jwt token
        let token = match jwt::create(
            &name,
            String::from("/home/ec2-user/dollar-ci.2020-04-18.private-key.pem"),
        ) {
            Ok(token) => token,
            Err(e) => {
                error!("jwt::create error: {:?}", e);
                return Some(e);
            }
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
                error!("check_run_create_error: {}\nrequest_body: {}", e, &body);
                return Some(HandlersErr::Client(e));
            }
        };

        None
    }

    // mark 'check_run' as 'in_progress'
    pub async fn check_run_start(
        name: String,
        url: String,
        installation_id: u64,
    ) -> Option<HandlersErr> {
        // create jwt token
        let token = match jwt::create(
            &name,
            String::from("/home/ec2-user/dollar-ci.2020-04-18.private-key.pem"),
        ) {
            Ok(token) => token,
            Err(e) => {
                error!("jwt::create error: {:?}", e);
                return Some(e);
            }
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
                error!("check_run_start error: {}\nrequest_body: {}", e, &body);
                return Some(HandlersErr::Client(e));
            }
        };

        None
    }

    // mark 'check_run' as 'complete' with either a fail or pass
    pub async fn check_run_complete(
        name: String,
        url: String,
        success: bool,
        installation_id: u64,
    ) -> Option<HandlersErr> {
        // create jwt token
        let token = match jwt::create(
            &name,
            String::from("/home/ec2-user/dollar-ci.2020-04-18.private-key.pem"),
        ) {
            Ok(token) => token,
            Err(e) => {
                error!("jwt::create error: {:?}", e);
                return Some(e);
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
                error!("check_run_complete error: {}\nrequest_body: {}", e, &body);
                Some(HandlersErr::Client(e))
            }
        }
    }

    // get_installation_token will create a jwt token from a pem file
    // use as bearer in request to generate installation token
    pub async fn get_installation_token(name: String, installation_id: u64) -> Result<String> {

        // create jwt token
        let jwt_token = match jwt::create(
            &name,
            String::from("/home/ec2-user/dollar-ci.2020-04-18.private-key.pem"),
        ) {
            Ok(jwt_token) => jwt_token,
            Err(e) => match e {
                HandlersErr::Json(e) => return Err(HandlersErr::Json(e)),
                HandlersErr::Client(e) => return Err(HandlersErr::Client(e)),
                HandlersErr::Jwt(e) => return Err(HandlersErr::Jwt(e)),
                HandlersErr::Io(e) => return Err(HandlersErr::Io(e)),
            },
        };

        // init http client
        let client = reqwest::Client::new();

        // form url
        let url = format!(
            "https://api.github.com/app/installations/{}/access_tokens",
            installation_id
        );
        
        // send post
        let res = match client.post(&url).bearer_auth(jwt_token).send().await {
            Ok(res) => res,
            Err(e) => return Err(HandlersErr::Client(e)),
        };
        
        // get installation token from body
        let body = match res.json::<InstallToken>().await {
            Ok(body) => body,
            Err(e) => return Err(HandlersErr::Client(e)),
        };

        // return installation token
        Ok(body.token)
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
