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

use log::{debug, info};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

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
