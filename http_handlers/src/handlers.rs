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
    use super::Event;

    use warp::Filter;

    // events listens for github events
    pub fn events() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::post().and(warp::body::json()).and_then(handlers::event)
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
                client::check_run_create(event.check_suite.app.name, event.check_suite.head_sha, event.check_suite.check_runs_url).await;
                Ok(StatusCode::OK)
            },
            "requested" => {
                client::check_run_create(event.check_suite.app.name, event.check_suite.head_sha, event.check_suite.check_runs_url).await;
                Ok(StatusCode::OK)
            },
            "created" => {
                client::check_run_start(event.check_suite.app.name, event.check_suite.check_runs_url).await;
                Ok(StatusCode::OK)
            },
            _ => Ok(StatusCode::BAD_REQUEST),
        }
    }
}

// http client
pub mod client {

    use serde_json::*;
    use time::Instant;

    // tell github to create 'check_run'
    pub async fn check_run_create(name: String, head_sha: String, url: String) -> reqwest::StatusCode {

        // init http client
        let client = reqwest::Client::new();

        // create body
        let body = json!({"name": name,"head_sha": head_sha});

        // send post
        let res = match client.post(&url).json(&body).send().await {
            Ok(res) => return res.status(),
            Err(e) => {
                println!("check_run_create error: {}", e);
                return reqwest::StatusCode::INTERNAL_SERVER_ERROR
            },
        };
    }

    // tell github to create 'check_run'
    pub async fn check_run_complete(name: String, url: String, success: bool) -> reqwest::StatusCode {
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
        let res = match client.post(&url).json(&body).send().await {
            Ok(res) => return res.status(),
            Err(e) => {
                println!("check_run_create error: {}", e);
                return reqwest::StatusCode::INTERNAL_SERVER_ERROR
            },
        };
    }

    // update 'check_run' to 'in progress'
    //pub async fn check_run_start(name: String, url: String) -> Result<reqwest::Response, Infallible> {
    //    // init http client
    //    let client = reqwest::Client::new();

    //    // create body
    //    let body = json!({"name": name, "status": "in_progress", "started_at": format!("{:?}", Instant::now())});

    //    // send post
    //    match client.post(&url).json(&body).send().await {
    //        Ok(res) => res,
    //        Err(e) => println!("check_run_create error: {}", e),
    //    };

    //    Ok(())
    //}

    //// mark check_run as complete
    //pub async fn check_run_complete(name: String, url: String, success: bool) -> Result<reqwest::Response, Infallible> {
    //    // init http client
    //    let client = reqwest::Client::new();

    //    // define success param
    //    let mut conclusion = String::from("success");
    //    if !success {
    //        conclusion = String::from("failure");
    //    };

    //    // create body
    //    let body = json!({"name": name, "status": "completed", "conclusion": conclusion, "completed_at": format!("{:?}", Instant::now())});

    //    // send post
    //    match client.post(&url).json(&body).send().await {
    //        Ok(res) => res,
    //        Err(e) => println!("check_run_create error: {}", e),
    //    };

    //    Ok(())
    //}
}

#[cfg(test)]
mod tests {
    use super::filters;
    use std::fs;
    use warp::http::StatusCode;
    use warp::test::request;

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
