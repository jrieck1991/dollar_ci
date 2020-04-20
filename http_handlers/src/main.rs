use serde::{Deserialize, Serialize};
use time::Instant;
use warp::Filter;
use git2::Repository;
use serde_json::*;

use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{encode, EncodingKey, Header};

mod handlers;

#[tokio::main]
async fn main() {
    //warp::serve(requested()).run(([0, 0, 0, 0], 80)).await;
}


// create jwt from pem
//fn get_jwt(pem_str: String) -> Result<String, Error> {
//    // define claims
//    let my_claims = Claims {
//        sub: "dollar-ci".to_owned(),
//        company: "dollar-ci".to_owned(),
//        exp: 10000000000,
//    };
//
//    // setup header
//    let mut header = Header::default();
//    header.kid = Some("signing_key".to_owned());
//    header.alg = Algorithm::RS256;
//
//    // encode and receive token that can be used in http headers
//    let token = match encode(
//        &header,
//        &my_claims,
//        &EncodingKey::from_secret(pem_str.as_bytes()),
//    ) {
//        Ok(t) => Ok(t),
//        Err(e) => Err(e),
//    };
//}

// tell github to create 'check_run'
async fn check_run_create(name: String, head_sha: String, url: String) {
    // init http client
    let client = reqwest::Client::new();

    // create body
    let body = json!({"name": name,"head_sha": head_sha});

    // send post
    let res = client.post(&url).json(&body).send().await;
}

// update 'check_run' to 'in progress'
async fn check_run_start(name: String, url: String) {
    // init http client
    let client = reqwest::Client::new();

    // create body
    let body = json!({"name": name, "status": "in_progress", "started_at": format!("{:?}", Instant::now())});

    // send post
    let res = client.post(&url).json(&body).send().await;
}

// mark check_run as complete
async fn check_run_complete(name: String, url: String, success: bool) {
    // init http client
    let client = reqwest::Client::new();

    // define success param
    let mut conclusion = String::from("success");
    if !success {
        conclusion = String::from("failure");
    };

    // create body
    let body = json!({"name": name, "status": "completed", "conclusion": conclusion, "completed_at": format!("{:?}", Instant::now())});

    // post
    let res = client.post(&url).json(&body).send().await;
}

// clone head_sha of git branch
// requires token of type 'x-access-token'
// path is the repository path
//fn clone(head_sha: String, token: String, path: String) {
//    // form clone url from token and path
//    let url = format!("https://{}@github.com/{}.git", token, path);
//
//    // clone repo
//    let repo = match Repository::clone(url) {
//        Ok(repo) => repo,
//        Err(e) => panic!("failed to clone: {}", e),
//    };
//
//    // package repo
//}

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
