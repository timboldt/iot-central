//  Copyright 2022 Google LLC
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//      https://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.

#![warn(clippy::all)]

use crate::adafruit;

//use chrono::{offset::TimeZone, Local, Utc};
use async_channel;
use log::{debug, info};
use serde::Deserialize;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

#[derive(Debug)]
pub struct CallParams {
    pub shutdown: Arc<(Mutex<bool>, Condvar)>,
    pub tx: async_channel::Sender<adafruit::Metric>,
    pub base_url: String,
    pub api_key: String,
    pub symbols: Vec<String>,
}

#[derive(Deserialize, Debug, Default)]
struct Quote {
    #[serde(rename = "c")]
    current_price: f32,
}

pub async fn finance_updater(params: CallParams) {
    info!("finance_updater starting");
    debug!("finance_updater parameters {:?}", params);
    let client = reqwest::Client::new();
    let update_period = Duration::from_secs(10 * 60);
    loop {
        for symbol in &params.symbols {
            let url = format!("{}?symbol={}", params.base_url, symbol);
            debug!("Getting finance from {}", url);
            let resp = client
                .get(url)
                .header("X-Finnhub-Token", &params.api_key)
                .send()
                .await;
            match resp {
                Ok(r) => {
                    debug!("GET finance (symbol: {}): {:?}", symbol, r.status());
                    let q: Quote = r.json().await.unwrap_or_default();
                    if q.current_price != 0.0 {
                        params
                            .tx
                            .send(adafruit::Metric {
                                feed: format!(
                                    "finance.{}",
                                    symbol.to_lowercase().replace(':', "-")
                                ),
                                value: q.current_price,
                            })
                            .await
                            .unwrap()
                    }
                }
                _ => {
                    debug!("GET finance (symbol: {}) failed: {:?}", symbol, resp.err());
                }
            }
        }

        // Wait for next update period, or  shutdown signal.
        let (lock, cvar) = &*params.shutdown;
        let shutdown = cvar
            .wait_timeout_while(lock.lock().unwrap(), update_period, |&mut shutdown| {
                !shutdown
            })
            .unwrap();
        if *shutdown.0 {
            break;
        }
    }
    info!("finance_updater finished");
}
