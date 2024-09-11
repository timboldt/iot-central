use std::env;
use tokio::io::{self};
use tokio::signal;

mod finance;

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("Starting...");

    let api_key = env::var("FINHUB_API_KEY").expect("FINHUB_API_KEY env variable is missing");
    let f = finance::Finance::new(
        &api_key,
        vec!["AAPL".to_owned(), "GOOG".to_owned(), "MSFT".to_owned()],
    );
    f.run().await;
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

    println!("Done.");
    Ok(())
}
