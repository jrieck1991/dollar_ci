mod client;

// http route filters
pub mod filters {
    use super::handlers;
    use warp::{Filter, Reply, Rejection, post};

    // events listens for github events
    pub fn events() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        post()
            .and(warp::body::json())
            .and_then(handlers::event)
    }
}

// handlers handle github event payloads
mod handlers {

    use std::convert::Infallible;
    use warp::http::StatusCode;
    use warp::Reply;

    use crate::models::Event;
    use super::client;

    // handle github event payload
    pub async fn event(event: Event) -> Result<impl Reply, Infallible> {
        debug!("event received: {:?}", event);

        // route event based on action
        match event.action.as_str() {
            "requested" | "rerequested" => {
                match client::check_run_create(
                    &event.repository.full_name,
                    &event.check_suite.head_sha,
                    &event.check_suite.check_runs_url,
                    event.installation.id,
                )
                .await
                {
                    Ok(code) => {
                        info!(
                            "check_run_create for {}. status_code: {}",
                            event.repository.full_name, code
                        );
                    }
                    Err(e) => {
                        error!(
                            "check_run_create for {}. Error: {}",
                            event.repository.full_name, e
                        );
                    }
                };

                Ok(StatusCode::OK)
            }
            "created" => {
                match client::check_run_start(
                    &event.repository.full_name,
                    &event.check_suite.check_runs_url,
                    event.installation.id,
                )
                .await
                {
                    Ok(code) => {
                        info!(
                            "check_run_complete for {}. status_code: {}",
                            event.repository.full_name, code
                        );
                    }
                    Err(e) => {
                        error!(
                            "check_run_complete for {}. Error: {}",
                            event.repository.full_name, e
                        );
                    }
                };

                Ok(StatusCode::OK)
            }
            _ => {
                warn!("no match for event: {:?}", event);
                Ok(StatusCode::BAD_REQUEST)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::filters;
    use std::fs;
    use warp::http::StatusCode;

    #[tokio::test]
    async fn test_events() {
        // get test payload from file
        let payload =
            fs::read_to_string("action_requested.json").unwrap();

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
