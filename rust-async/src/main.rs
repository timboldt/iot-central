use std::env;
use tokio::{io, signal};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

mod finance;

#[tokio::main]
async fn main() -> io::Result<()> {
    let task_tracker = TaskTracker::new();
    let cancellation_token = CancellationToken::new();

    println!("Starting...");

    let api_key = env::var("FINHUB_API_KEY").expect("FINHUB_API_KEY env variable is missing");
    let f = finance::Finance::new(
        cancellation_token.clone(),
        &api_key,
        vec!["AAPL".to_owned(), "GOOG".to_owned(), "MSFT".to_owned()],
    );
    task_tracker.spawn(async move { f.run().await });

    task_tracker.close();
    println!("Running (^C to exit)");

    match signal::ctrl_c().await {
        Ok(()) => {
            println!("")
        }
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
            // we also shut down in case of error
        }
    }

    println!("Shutting down...");
    cancellation_token.cancel();
    task_tracker.wait().await;

    println!("Done.");
    Ok(())
}
