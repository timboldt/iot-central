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

use log::{debug, info};

#[derive(Debug)]
pub struct CallParams {
    pub base_url: String,
    pub io_user: String,
    pub io_key: String,
}

#[derive(Debug)]
pub struct Metric {
    pub feed: String,
    pub value: f32,
}

pub async fn aio_sender(params: CallParams, rx: async_channel::Receiver<Metric>) {
    info!("aio_sender starting");
    debug!("aio_sender parameters {:?}", params);
    let client = reqwest::Client::new();
    while let Ok(m) = rx.recv().await {
        debug!("Received {:?}", m);
        let url = format!(
            "{}/{}/feeds/{}/data",
            params.base_url, params.io_user, m.feed
        );
        debug!("POSTing to {}", url);
        let form = reqwest::multipart::Form::new().text("value", m.value.to_string());
        let resp = client
            .post(url)
            .header("X-AIO-Key", params.io_key.as_bytes())
            .multipart(form)
            .send()
            .await;
        match resp {
            Ok(r) => {
                debug!("POST succeeded: {:?}", r.status());
            }
            _ => {
                debug!("POST failed: {:?}", resp.err());
            }
        }
    }
    info!("aio_sender finished");
}
