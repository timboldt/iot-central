use finnhub_rs::client::Client;
use tokio::time::{sleep, Duration};

pub struct Finance {
    api_key: String,
    symbols: Vec<String>,
}

impl Finance {
    pub fn new(api_key: &str, symbols: Vec<String>) -> Self {
        Finance {
            api_key: api_key.to_owned(),
            symbols: symbols,
        }
    }

    pub async fn run(&self) {
        // Lame implementation; needs graceful shutdown, etc.
        loop {
            self.process().await;
            sleep(Duration::from_secs(3600)).await;
        }
    }

    pub async fn process(&self) {
        for symbol in &self.symbols {
            let client = Client::new(self.api_key.clone());
            let res = client.quote(symbol.to_string()).await.unwrap();
            println!("{} {:#?}", symbol, res);
        }
    }
}
