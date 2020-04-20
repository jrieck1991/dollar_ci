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
    use super::Event;
    use std::convert::Infallible;
    use warp::http::StatusCode;

    // handle github event payload
    pub async fn event(event: Event) -> Result<impl warp::Reply, Infallible> {

        // route event based on action
        match event.action.as_str() {
            "requested" => Ok(StatusCode::OK), // create check run
            "rerequested" => Ok(StatusCode::OK), // create check run
            "created" => Ok(StatusCode::OK), // update check run to in progress
            _ => Ok(StatusCode::BAD_REQUEST),
        }
    }
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
