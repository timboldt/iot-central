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

extern crate reqwest;
extern crate serde;

mod adafruit;

use adafruit::Metric;
use log::{debug, info};
use std::env;
use std::sync::mpsc::channel;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let (tx, rx) = channel();
    let params = adafruit::CallParams{
        client: reqwest::blocking::Client::new(),
        base_url: "https://io.adafruit.com/api/v2".to_owned(),
        io_user: env::var("IO_USERNAME").expect("Adafruit IO_USERNAME is not defined."),
        io_key: env::var("IO_KEY").expect("Adafruti IO_KEY is not defined."),
    };
    let h = thread::spawn(move || adafruit::aio_sender(params, rx));
    tx.send(Metric{feed: "test1".into(), value: 42.0}).unwrap();
    tx.send(Metric{feed: "test1".into(), value: 3.22}).unwrap();
    tx.send(Metric{feed: "test1".into(), value: 18.0}).unwrap();
    drop(tx);
    h.join().unwrap();

    // Some simple CLI args requirements...
    let url = match std::env::args().nth(1) {
        Some(url) => url,
        None => {
            info!("No CLI URL provided, using default.");
            "https://hyper.rs".into()
        }
    };

    debug!("Fetching {:?}...", url);

    // reqwest::blocking::get() is a convenience function.
    //
    // In most cases, you should create/build a reqwest::Client and reuse
    // it for all requests.
    let res = reqwest::blocking::get(url)?;

    info!("Response: {:?} {}", res.version(), res.status());
    // debug!("Headers: {:#?}\n", res.headers());

    // copy the response body directly to stdout
    //res.copy_to(&mut std::io::stdout())?;
    info!("Result: {}", res.content_length().unwrap_or_default());

    Ok(())
}
