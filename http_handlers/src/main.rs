use warp::Filter;

#[tokio::main]
async fn main() {

    let events = warp::post()
        .and(warp::path("events")
        .map(|| "Hello" ));
    
    warp::serve(events).run(([0, 0, 0, 0], 80)).await;
}
