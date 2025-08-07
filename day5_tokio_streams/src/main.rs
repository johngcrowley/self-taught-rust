use std::time::Duration;
use std::thread::sleep;
use tokio_stream::{StreamExt};

#[tokio::main]
async fn main() {

    // there are two kinds of JoinHandles

    /* 
     * 1.) std::thread::JoinHandle
     */
    let mut thread_handles: Vec<std::thread::JoinHandle<()>> = vec![];
    for i in 0..4 {
        thread_handles.push(std::thread::spawn(move || {
            sleep(Duration::from_millis(1000));
            println!("hello from thread {i}");
        }));
    }

    for jh in thread_handles {
        jh.join().unwrap();
    }

    /* 
     * 2.) tokio::task::JoinHandle
     */

    let mut task_handles: Vec<tokio::task::JoinHandle<()>> = vec![];
    for i in 0..4 {
        task_handles.push(tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            println!("hello from task {i}");
        }));
    }

    for jh in task_handles {
        jh.await;
    }

    /*
     * You'll notice that `tokio::spawn` is the same as `JoinSet::spawn`. And here, we aren't
     * waiting for any handles! And, in the above case, we are looping over `task_handles` but the
     * results are interleaved in `stdout` -- the tasks were kicked off and running. They may
     * finish at different times, even if by microseconds.
     *
     * There are 2 means of driving a `Future`'s execution forward:
     * 1. `.await` (for `async fn`s and within them)
     * 2. at top-level of `async fn main` by `spawn`ing tasks right into the Executor
     *
     * This means in #2, we don't actually need to `.await` the task -- it's already running, 
     * whereas in #1, a `Future` is lazily-evaluated, and nothing is done until its polled via
     * `.await`.
     *
     * In #1, `spawn` returns a `JoinHandle` instead of a type that implements `Future` (like
     * `async fn` desugars to).
     */
    
    let mut tokio_join_set = tokio::task::JoinSet::new();
    for i in 0..4 {
        tokio_join_set.spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            println!("hello from task {i}");
        });
    }

    sleep(Duration::from_millis(3000));


    /* 
     * Briefly show a `Stream` as an iterator-like object of `Future`s.
     */
    
    // how come this is fine? is `tokio_stream::iter()` returning a  `Box<Pin` already?
    let v = vec![0,1,2,3]; 
    let mut stream = tokio_stream::iter(v);
    while let Some(fut) = stream.next().await {
        println!("{:?}", fut);
    }
    
    /*
     * `yield` pauses execution at that point in the program until the `.next().await` is called,
     *  where execution returns to the point below the yield. 
     *
     *  this is why continuation token logic lives below those, in an infinite `loop {}`:
     *  keep pulling and pulling items until `None` or we hit a custom threshold.
     */ 

    // i have the deliberately pin this.
    let mut stream = async_stream::stream!(
        let v = vec![0,1,2,3];
        for elem in v {
           yield(elem);     
        }
    );
    
    tokio::pin!(stream);
    
    while let Some(fut) = stream.next().await {
        println!("{:?}", fut);
    }

    // Stream from a large object in a bucket
    
    /*
     * the caller of this part would be a `tokio_util::io::StreamReader` that converts into an
     * `AsyncRead` that manually does the `while let Some(item)` pulling for you.
     */ 
    let provider = gcp_auth::provider().await.unwrap();
    let scopes = &["https://www.googleapis.com/auth/cloud-platform"];
    let token = provider.token(scopes).await.unwrap();

    /* 
     * https://cloud.google.com/storage/docs/json_api/v1/objects/get requires `alt=media` for the
     * object, default is `alt=json` for the metadata/headers
     */
    let uri: &'static str = "https://storage.googleapis.com/storage/v1/b/acrelab-production-us1c-transfer/o/boundary.jsonl?alt=media";
    let mut req_stream = reqwest::Client::new()
        .get(uri)
        .bearer_auth(token.as_str())
        .send()
        .await
        .unwrap()
        .bytes_stream();

    while let Some(item) = req_stream.next().await {
        println!("{:?}", item);
    }
} 
