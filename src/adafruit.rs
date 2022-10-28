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
use std::sync::mpsc;

pub trait MetricClient {
    fn publish(&self, metric: Metric);
}

#[derive(Debug)]
pub struct Metric {
    pub feed: String,
    pub value: f32,
}

pub fn aio_sender(client: &dyn MetricClient, rx: mpsc::Receiver<Metric>) {
    while let Ok(m) = rx.recv() {
        debug!("Received {:?}", m);
        client.publish(m);
    }
    info!("aio_sender finished");
}
