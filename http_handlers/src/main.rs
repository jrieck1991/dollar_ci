use warp::Filter;

#[tokio::main]
async fn main() {

    let events = warp::post()
        .and(warp::path("events")
        .map(|| "Hello" ));
    
    warp::serve(events).run(([0, 0, 0, 0], 80)).await;
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