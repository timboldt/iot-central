use finnhub_rs::client::Client;

pub struct Finance {
    api_key: String,
}

impl Finance {
    pub fn new(api_key: &str) -> Self {
        Finance {
            api_key: api_key.to_owned(),
        }
    }

    pub async fn process(&self) {
        let client = Client::new(self.api_key.clone());
        // Get a list of supported stocks given the exchange.
        let res = client.stock_symbol("US".to_string()).await.unwrap();
        // Print out the results.
        println!("{:#?}", res);
    }
}
