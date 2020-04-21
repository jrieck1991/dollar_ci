use serde::{Deserialize, Serialize};

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
            _ => Ok(StatusCode::BAD_REQUEST),
        }
    }
}

// http client
mod client {

    use serde_json::*;
    use time::Instant;

    // tell github to create 'check_run'
    pub async fn check_run_create(name: String, head_sha: String, url: String) {
        // init http client
        let client = reqwest::Client::new();

        // create body
        let body = json!({"name": name,"head_sha": head_sha});

        // send post
        match client.post(&url).json(&body).send().await {
            Ok(res) => println!("check_run_create status_code: {}", res.status()),
            Err(e) => println!("check_run_create error: {}", e),
        };
    }

    // mark 'check_run' as 'in_progress'
    pub async fn check_run_start(name: String, url: String) {
        // init http client
        let client = reqwest::Client::new();

        // create body
        let body = json!({"name": name, "status": "in_progress", "started_at": format!("{:?}", Instant::now())});

        // send post
        match client.post(&url).json(&body).send().await {
            Ok(res) => println!("check_run_start status_code: {}", res.status()),
            Err(e) => println!("check_run_start error: {}", e),
        };
    }

    // mark 'check_run' as 'complete' with either a fail or pass
    pub async fn check_run_complete(name: String, url: String, success: bool) {
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
        match client.post(&url).json(&body).send().await {
            Ok(res) => println!("check_run_complete status_code: {}", res.status()),
            Err(e) => println!("check_run_complete error: {}", e),
        };
    }
}

// JWT formation module
mod jwt {
    use jsonwebtoken::{encode, Header, Algorithm, EncodingKey}; 
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
       sub: String,
       company: String,
       exp: u64,
    }

    // create jwt from pem file
    fn create(pem_str: String) -> Result<String, Error> {

        // define claims
        let claims = Claims {
            sub: String::from("dollar-ci"),
            company: String::from("dollar-ci"),
            exp: 10000000000,
        };
    
        // setup header
        let mut header = Header::default();
        header.alg = Algorithm::RS256;
    
        // encode and receive token that can be used in http headers
        let token = match encode(
            &header,
            &claims,
            &EncodingKey::from_secret(pem_str.as_bytes()),
        ) {
            Ok(t) => Ok(t),
            Err(e) => Err(e),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::filters;
    use std::fs;
    use warp::http::StatusCode;

    // read test github json into string
    // only for tests
    fn get_payload() -> String {
        fs::read_to_string("test_github_payload.json").expect("unable to read file to string")
    }

    #[tokio::test]
    async fn test_events() {
        // get test payload from file
        let payload = get_payload();

        // send request
        let resp = warp::test::request()
            .method("POST")
            .body(&payload.as_bytes())
            .reply(&filters::events())
            .await;

        // verify status code
        assert_eq!(resp.status(), StatusCode::OK)
    }
}
