use warp::Filter;
use serde_derive::{Deserialize, Serialize};
use git2::Repository;

#[derive(Deserialize, Serialize)]
struct Event {
    action: String,
    check_suite: Check_Suite,
    app: App,
}

#[derive(Deserialize, Serialize)]
struct Check_Suite {
    id: u64,
    status: String,
    head_sha: String,
}

#[derive(Deserialize, Serialize)]
struct App {
    id: u64,
}

#[tokio::main]
async fn main() {

    let events = warp::post()
        .and(warp::body::json())
        .map(|action, mut event: Event|  {
            
        });
    
    warp::serve(events).run(([0, 0, 0, 0], 80)).await;
}

// requested listenes for actions of 'requested' and 'rerequested'
fn requested() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("events")
        .and(warp::post())
        .and(json_body())
}

// created listens for 'check_run' 'created' events
fn created() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("events")
        .and(warp::post())
        .and(json_body())
}

// assert body is json and within a size limit
fn json_body() -> impl Filter<Extract = (Event,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

// create jwt from pem
fn get_jwt() -> String {
    let jwt = String::from("JWT STRING HERE")
}

// tell github to create 'check_run'
fn check_run_create(name: String, head_sha: String) {
    let client = reqwest::Client::new();
    let res = client.post("http://somewhere.com")
        .json("body")
        .send()
        .await?;
}

// update 'check_run' to 'in progress'
fn check_run_start(check_id: String) {
    let client = reqwest::Client::new();
    let res = client.post("http://somewhere.com")
        .json("body")
        .send()
        .await?;
}

// mark check_run as complete
fn check_run_complete(check_id: String, success: bool) {
    let client = reqwest::Client::new();
    let res = client.post("http://somewhere.com")
        .json("body")
        .send()
        .await?;
}

// clone head_sha of git branch
fn clone(head_sha: String) {
    let repo = match Repository::clone("https://token:repo.git") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to clone: {}", e),
    };
}

// run the tests
// clone head_sha
// read check config in root of repo
// run defined tests
// return success bool
fn run_check(head_sha: String) -> bool {
    true
}

// listen for 'check_suite' of type 'requested', this means new code is pushed to a repo
// 'rerequested' means the user manually requested a re run of the check
// create JWT from pem to authenticate https requests to github
// create a 'check_run' next
// github will create a 'check_run' and send back a 'created' action
// update 'check_run' to in progress
// use 'x-access-token' to clone repo
// run CI
// make 'check_run' as complete
