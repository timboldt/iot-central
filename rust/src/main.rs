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

extern crate ctrlc;
extern crate reqwest;
extern crate serde;

mod adafruit;
mod conversion;
mod finance;
mod sensor;
mod weather;

use log::info;
use std::env;
use std::sync::mpsc::channel;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

const ENABLE_FINANCE_THREAD: bool = false;
const ENABLE_WEATHER_THREAD: bool = false;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let shutdown = Arc::new((Mutex::new(false), Condvar::new()));
    let (tx, rx) = channel();

    // Start the Adafruit IO transmission agent.
    let aio_params = adafruit::CallParams {
        base_url: "https://io.adafruit.com/api/v2".to_owned(),
        io_user: env::var("IO_USERNAME").expect("Adafruit IO_USERNAME is not defined."),
        io_key: env::var("IO_KEY").expect("Adafruti IO_KEY is not defined."),
    };
    let aio_thread = thread::spawn(move || adafruit::aio_sender(aio_params, rx));

    // Start the sensor thread.
    let sensor_params = sensor::CallParams {
        shutdown: shutdown.clone(),
        tx: tx.clone(),
    };
    let sensor_thread = thread::spawn(move || sensor::sensor_updater(sensor_params));

    let finance_thread = if ENABLE_FINANCE_THREAD {
        // Start the finance thread.
        let finance_params = finance::CallParams {
            shutdown: shutdown.clone(),
            tx: tx.clone(),
            base_url: "https://finnhub.io/api/v1/quote".to_owned(),
            api_key: env::var("FINHUB_API_KEY").expect("FINHUB_API_KEY is not defined."),
            symbols: vec![
                "DIA".into(),
                "COINBASE:BTC-USD".into(),
                "BITFINEX:USTUSD".into(),
                "KRAKEN:USDTZUSD".into(),
                "QQQ".into(),
            ],
        };
        thread::spawn(move || finance::finance_updater(finance_params))
    } else {
        // Do nothing.
        thread::spawn(move || return)
    };

    let weather_thread = if ENABLE_WEATHER_THREAD {
        // Start the weather thread.
        let weather_params = weather::CallParams {
            shutdown: shutdown.clone(),
            tx: tx.clone(),
            base_url: "https://api.openweathermap.org/data/2.5/onecall".to_owned(),
            api_key: env::var("OPEN_WEATHER_KEY").expect("OPEN_WEATHER_KEY is not defined."),
            lat: env::var("OPEN_WEATHER_LAT").expect("OPEN_WEATHER_LAT is not defined."),
            lon: env::var("OPEN_WEATHER_LON").expect("OPEN_WEATHER_LON is not defined."),
            units: "metric".to_owned(),
        };
        thread::spawn(move || weather::weather_updater(weather_params))
    } else {
        // Do nothing.
        thread::spawn(move || return)
    };

    ctrlc::set_handler(move || {
        info!("Shutdown initiated...");

        // Signal all the producer threads.
        let (lock, cvar) = &*shutdown;
        let mut sig = lock.lock().unwrap();
        *sig = true;
        cvar.notify_all();
    })
    .unwrap();

    info!("Waiting for Sensor thread...");
    sensor_thread.join().expect("Failed to join sensor_thread.");

    info!("Waiting for Finance thread...");
    finance_thread
        .join()
        .expect("Failed to join finance_thread.");

    info!("Waiting for Weather thread...");
    weather_thread
        .join()
        .expect("Failed to join weather_thread.");

    // Signal the consumer thread (Adafruit IO sender).
    drop(tx);
    info!("Waiting for Adafruit IO thread...");
    aio_thread.join().expect("Failed to join aio_thread.");

    Ok(())
}
