use finnhub_rs::client::Client;
use tokio_util::sync::CancellationToken;

pub struct Finance {
    api_key: String,
    symbols: Vec<String>,
    cancel: CancellationToken,
}

impl Finance {
    pub fn new(cancel: CancellationToken, api_key: &str, symbols: Vec<String>) -> Self {
        Finance {
            cancel,
            api_key: api_key.to_owned(),
            symbols: symbols,
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

    pub async fn process(&self) {
        for symbol in &self.symbols {
            let client = Client::new(self.api_key.clone());
            let res = client.quote(symbol.to_string()).await.unwrap();
            println!("{} {:#?}", symbol, res);
        }
    }
}
