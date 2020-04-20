use warp::Filter;
use serde_derive::{Deserialize, Serialize};
use git2::Repository;
use time::Instant;
  
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{encode, EncodingKey, Header};

#[derive(Deserialize, Serialize)]
struct Event {
    action: String,
    check_suite: Check_Suite,
    app: App,
    repo: Repository,
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
    check_runs_url: String,
}

#[derive(Deserialize, Serialize)]
struct Repository {
    clone_url: String,
}

#[tokio::main]
async fn main() {

    let events = warp::post()
        .and(warp::body::json())
        .map(|action, mut event: Event|  {
            
        });
    
    warp::serve(events).run(([0, 0, 0, 0], 80)).await;
}

// requested listens for actions 'requested' and 'rerequested'
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

// assert body is json and within size limit
fn json_body() -> impl Filter<Extract = (Event,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

// create jwt from pem
fn get_jwt(pem_str: String) -> Result<String, Error> {

    // define claims
    let my_claims = Claims { sub: "dollar-ci".to_owned(), company: "dollar-ci".to_owned(), exp: 10000000000 };

    // setup header
    let mut header = Header::default();
    header.kid = Some("signing_key".to_owned());
    header.alg = Algorithm::RS256;

    // encode and receive token that can be used in http headers
    let token = match encode(&header, &my_claims, &EncodingKey::from_secret(pem_str.to_bytes())) {
        Ok(t) => Ok(t),
        Err(e) => Err(e),
    };
}

// tell github to create 'check_run'
fn check_run_create(name: String, head_sha: String, url: String) {

    // init http client
    let client = reqwest::Client::new();

    // create body
    let body = format!(r#"
        {
            "name": "{}",
            "head_sha": "{}"
        }"#, name, head_sha)

    // send post
    let res = client.post(url)
        .json(body)
        .send()
        .await?; 
}

// update 'check_run' to 'in progress'
fn check_run_start(name: String, url: String) {

    // init http client
    let client = reqwest::Client::new();

    // form request body
    let body = format!(r#"
        {
            "name": "{}",
            "status": "in_progress",
            "started_at": {}
        }"#, name, Instant::now())

    // send post
    let res = client.post(url)
        .json(body)
        .send()
        .await?;
}

// mark check_run as complete
fn check_run_complete(url: String, success: bool) {

    // init http client
    let client = reqwest::Client::new();

    // define success param
    let conclusion = String::from("success");
    if !success {
        conclusion = String::from("failure");
    };

    // form request body
    let body = format!(r#"
        {
            "name": "{}",
            "status": "completed",
            "conclusion": "{}",
            "completed_at": {}
        }"#, name, conclusion, Instant::now());

    // post
    let res = client.post(url)
        .json(body)
        .send()
        .await?;
}

// clone head_sha of git branch
// requires token of type 'x-access-token'
// path is the repository path
fn clone(head_sha: String, token: String, path: String) {

    // form clone url from token and path
    let url = format!("https://{}@github.com/{}.git", token, path)

    // clone repo
    let repo = match Repository::clone(url) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to clone: {}", e),
    };

    // package repo
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
