use tokio::{io, signal, sync::mpsc};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct Metric {
    pub feed: String,
    pub value: f32,
}
pub struct AdafruitIO {
    api_username: String,
    api_key: String,
    cancel: CancellationToken,
    rx: mpsc::Receiver<Metric>,
}

impl AdafruitIO {
    pub fn new(
        cancel: CancellationToken,
        api_username: &str,
        api_key: &str,
        rx: mpsc::Receiver<Metric>,
    ) -> Self {
        AdafruitIO {
            cancel,
            api_username: api_username.to_owned(),
            api_key: api_key.to_owned(),
            rx,
        }
    }

    pub async fn run(&self) {
        println!("AdafruitIO task starting.");
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
        println!("AdafruitIO task shutting down.");
    }

    async fn process(&self) {
        // for symbol in &self.symbols {
        //     let client = Client::new(self.api_key.clone());
        //     let res = client.quote(symbol.to_string()).await.unwrap();
        //     println!("{} {:#?}", symbol, res);
        // }
    }
}
