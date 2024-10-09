use std::env;
use tokio::{io, signal, sync::mpsc};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

mod adafruit;
mod finance;

#[tokio::main]
async fn main() -> io::Result<()> {
    let task_tracker = TaskTracker::new();
    let cancellation_token = CancellationToken::new();
    let (tx, rx) = mpsc::channel(100);

    println!("Starting...");

    let aio_username = env::var("IO_USERNAME").expect("IO_USERNAME env variable is missing");
    let aio_api_key = env::var("IO_KEY").expect("IO_KEY env variable is missing");
    let aio =
        adafruit::AdafruitIO::new(cancellation_token.clone(), &aio_username, &aio_api_key, rx);
    task_tracker.spawn(async move { aio.run().await });

    let finhub_api_key =
        env::var("FINHUB_API_KEY").expect("FINHUB_API_KEY env variable is missing");
    let finhub = finance::Finance::new(
        cancellation_token.clone(),
        &finhub_api_key,
        vec!["AAPL".to_owned(), "GOOG".to_owned(), "MSFT".to_owned()],
        tx,
    );
    task_tracker.spawn(async move { finhub.run().await });

    task_tracker.close();
    println!("Running (^C to exit)");

    match signal::ctrl_c().await {
        Ok(()) => {
            println!("")
        }
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        }
    }

    println!("Shutting down...");
    cancellation_token.cancel();
    task_tracker.wait().await;

    println!("Done.");
    Ok(())
}
