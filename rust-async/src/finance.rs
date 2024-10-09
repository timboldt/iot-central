use crate::adafruit;
use finnhub_rs::{client::Client, types::CompanyQuote};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

pub struct Finance {
    api_key: String,
    symbols: Vec<String>,
    cancel: CancellationToken,
    tx: mpsc::Sender<adafruit::Metric>,
}

impl Finance {
    pub fn new(
        cancel: CancellationToken,
        api_key: &str,
        symbols: Vec<String>,
        tx: mpsc::Sender<adafruit::Metric>,
    ) -> Self {
        Finance {
            cancel,
            api_key: api_key.to_owned(),
            symbols,
            tx,
        }
    }

    pub async fn run(&self) {
        println!("Finance task starting.");
        loop {
            self.process().await;
            tokio::select! {
                _ = self.cancel.cancelled() => {
                    break;
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(3600)) => {
                    continue;
                }
            }
        }
        println!("Finance task shutting down.");
    }

    async fn process(&self) {
        'next_symbol: for symbol in &self.symbols {
            let client = Client::new(self.api_key.clone());
            let quote_res = client.quote(symbol.to_string()).await;
            match quote_res {
                Ok(quote) => {
                    if quote.t == 0 {
                        println!("Ignoring probably-invalid symbol '{symbol}'.");
                        continue 'next_symbol;
                    }
                    self.send_quote_to_adafruit(symbol, quote).await;
                }
                Err(err) => {
                    println!("Failed to fetch quote for '{symbol}': {:?}.", err);
                    continue 'next_symbol;
                }
            }
        }
    }

    async fn send_quote_to_adafruit(&self, symbol: &str, quote: CompanyQuote) {
        let aio_res = self
            .tx
            .send(adafruit::Metric {
                feed: format!("finance.{}", symbol.to_lowercase().replace(':', "-")),
                value: quote.c as f32,
            })
            .await;
        if let Err(err) = aio_res {
            println!(
                "Failed to send quote to Adafruit for '{symbol}': {:?}.",
                err
            );
        }
    }
}
