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
use log::{debug, info};
use serde::Deserialize;
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::time::Duration;

#[derive(Debug)]
pub struct CallParams {
    pub shutdown: Arc<(Mutex<bool>, Condvar)>,
    pub tx: mpsc::Sender<adafruit::Metric>,
    pub base_url: String,
    pub api_key: String,
    pub lat: String,
    pub lon: String,
    pub units: String,
}

#[derive(Deserialize, Debug, Default)]
struct OneCallWeather {
    current: CurrentConditions,
}

#[derive(Deserialize, Debug, Default)]
struct CurrentConditions {
    #[serde(rename = "dt")]
    utc_timestamp: i64,
    // sunrise: i64,
    // sunset: i64,
    #[serde(rename = "temp")]
    temperature: f32,
    humidity: i32,
    pressure: i32,
    // wind_speed: f32,
    // wind_deg: i16,
    // weather: Vec<WeatherInfo>,
}

// #[derive(Deserialize, Debug, Default)]
// struct WeatherInfo {
//     id: i16,
//     main: String,
//     description: String,
//     icon: String,
// }

pub fn weather_updater(params: CallParams) {
    info!("weather_updater starting");
    debug!("weather_updater parameters {:?}", params);
    let client = reqwest::blocking::Client::new();
    let update_period = Duration::from_secs(10 * 60);
    loop {
        let url = format!(
            "{}?lat={}&lon={}&units={}&exclude=minutely,daily&appid={}",
            params.base_url, params.lat, params.lon, params.units, params.api_key
        );
        debug!("Getting weather from {}", url);
        let resp = client.get(url).send();
        match resp {
            Ok(r) => {
                debug!("GET weather: {:?}", r.status());
                let w: OneCallWeather = r.json().unwrap_or_default();
                if w.current.utc_timestamp != 0 {
                    params
                        .tx
                        .send(adafruit::Metric {
                            feed: "weather.temp".into(),
                            value: w.current.temperature,
                        })
                        .unwrap();
                    params
                        .tx
                        .send(adafruit::Metric {
                            feed: "weather.humidity".into(),
                            value: w.current.humidity as f32,
                        })
                        .unwrap();
                    params
                        .tx
                        .send(adafruit::Metric {
                            feed: "weather.pressure".into(),
                            value: w.current.pressure as f32,
                        })
                        .unwrap();
                }
            }
            _ => {
                debug!("GET weather failed: {:?}", resp.err());
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
    info!("weather_updater finished");
}
