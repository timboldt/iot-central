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
use std::sync::{Arc, Condvar, Mutex, mpsc};
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

#[derive(Deserialize, Debug)]
struct OneCallWeather {
    current: CurrentConditions,
}

#[derive(Deserialize, Debug)]
struct CurrentConditions {
    #[serde(rename = "dt")]
    utc_timestamp: i64,
    sunrise: i64,
    sunset: i64,
    // Temperature in degrees (C, F or K).
    #[serde(rename = "temp")]
    temperature: f32,
    // Percent humidity.
    humidity: i32,
    // Pressure in hPa.
    pressure: i32,
    // Wind speed (kmph or mph).
    wind_speed: f32,
    wind_deg: i16,
    weather: Vec<WeatherInfo>,
}

#[derive(Deserialize, Debug)]
struct WeatherInfo {
    id: i16,
    main: String,
    description: String,
    icon: String,
}

/*
let mut timeout_remaining = timeout;
loop {
    park_timeout(timeout_remaining);
    let elapsed = beginning_park.elapsed();
    if elapsed >= timeout {
        break;
    }
    println!("restarting park_timeout after {elapsed:?}");
    timeout_remaining = timeout - elapsed;
*/

pub fn weather_updater(params: CallParams) {
    info!("weather_updater starting");
    debug!("weather_updater parameters {:?}", params);
    let client = reqwest::blocking::Client::new();
    let update_period = Duration::from_secs(60*60);
    loop {
        // TODO: Fetch the weather and send it to adafruit.io.

        let (lock, cvar) = &*params.shutdown;
        let shutdown = cvar
            .wait_timeout_while(lock.lock().unwrap(), update_period, |&mut shutdown| !shutdown)
            .unwrap();
        if *shutdown.0 {
            break;
        }
    }
    info!("weather_updater finished");

    /*
           let url = format!(
               "{}/{}/feeds/{}/data",
               params.base_url, params.io_user, m.feed
           );


           let url = format!(
               "https://api.openweathermap.org/data/2.5/onecall?lat={}&lon={}&units={}&exclude=minutely,daily&appid={}",
               lat, lon, units, api_key
           );
           println!("{}", url);
           let resp: OneCallWeather = reqwest::blocking::get(url)?.json()?;
           println!("{:?}", resp);
           let dt = Utc
               .timestamp(resp.current.utc_timestamp, 0)
               .with_timezone(&Local);
           println!("{}", dt);

           let url = format!(
               "https://api.openweathermap.org/data/2.5/onecall/timemachine?dt={}&lat={}&lon={}&units={}&exclude=minutely,daily&appid={}",
               resp.current.utc_timestamp - 60 * 60 * 8, lat, lon, units, api_key
           );
           println!("{}", url);
           let resp: OneCallWeather = reqwest::blocking::get(url)?.json()?;
           println!("{:?}", resp);
           // let resp = reqwest::blocking::get(url)?.text()?;
           // println!("{}", resp);

           // TODO:
           // 1) Convert this information to something useful to the caller, instead of printing it.
           // 2) Correctly gather useful historical data or cache it.
           // 3) Possibly daemonize this instead of making it a one-shot.

           // TODO:    tx.send(Metric{feed: "weather.temp".into(), value: resp.x.x}).unwrap();


           debug!("POSTing to {}", url);
           let form = reqwest::blocking::multipart::Form::new().text("value", m.value.to_string());
           let resp = client
               .post(url)
               .header("X-AIO-Key", params.io_key.as_bytes())
               .multipart(form)
               .send();
           match resp {
               Ok(r) => { debug!("POST succeeded: {:?}", r.status()); },
               _ => { debug!("POST failed: {:?}", resp.err()); },
           }
       }
    */
}
