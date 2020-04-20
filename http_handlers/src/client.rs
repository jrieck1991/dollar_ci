
mod client {

    // tell github to create 'check_run'
    pub async fn check_run_create(name: String, head_sha: String, url: String) {
        // init http client
        let client = reqwest::Client::new();

        // create body
        let body = json!({"name": name,"head_sha": head_sha});

        // send post
        let res = client.post(&url).json(&body).send().await;
    }

    // update 'check_run' to 'in progress'
    pub async fn check_run_start(name: String, url: String) {
        // init http client
        let client = reqwest::Client::new();

        // create body
        let body = json!({"name": name, "status": "in_progress", "started_at": format!("{:?}", Instant::now())});

        // send post
        let res = client.post(&url).json(&body).send().await;
    }

    // mark check_run as complete
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

        // post
        let res = client.post(&url).json(&body).send().await;
    }
}
