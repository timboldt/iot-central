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
use bme280::BME280;
use crate::conversion;
use embedded_hal::blocking::{delay, i2c};
use log::debug;
use std::sync::mpsc;
use std::time::{Duration, Instant};

const UPDATE_PERIOD: Duration = Duration::from_secs(60);

pub struct State {
    pub sensor_is_valid: bool,
    pub last_abs_humidity: f32,
    pub last_update: Instant,
    pub temperature_sum: f32,
    pub humidity_sum: f32,
    pub pressure_sum: f32,
    pub count: i32,
}

pub fn poll<I2C, D, E>(
    bme: &mut BME280<I2C, D>,
    state: &mut State,
    tx: &mpsc::Sender<adafruit::Metric>,
) where
    I2C: i2c::Read<Error = E> + i2c::Write<Error = E> + i2c::WriteRead<Error = E>,
    D: delay::DelayUs<u8> + delay::DelayMs<u8>,
{
    if let Ok(measurements) = bme.measure() {
        debug!(
            "BME: temp = {} humid = {} press = {}",
            measurements.temperature, measurements.humidity, measurements.pressure,
        );
        state.temperature_sum += measurements.temperature;
        state.humidity_sum += measurements.humidity;
        state.pressure_sum += measurements.pressure;
        state.count += 1;
    }

    let now = Instant::now();
    if now.duration_since(state.last_update) > UPDATE_PERIOD {
        if state.count > 0 {
            let celsius = state.temperature_sum / state.count as f32;
            let relative_humidity = state.humidity_sum / state.count as f32;
            let raw_pressure_hpa = state.pressure_sum / state.count as f32 / 100.0;

            tx.send(adafruit::Metric {
                feed: "mbr-bme280.temperature".into(),
                value: celsius,
            })
            .unwrap();
            tx.send(adafruit::Metric {
                feed: "mbr-bme280.humidity".into(),
                value: relative_humidity,
            })
            .unwrap();
            tx.send(adafruit::Metric {
                feed: "mbr-bme280.pressure".into(),
                value: raw_pressure_hpa,
            })
            .unwrap();

            let fahrenheit = conversion::celsius_to_fahrenheit(celsius);
            state.last_abs_humidity =
                conversion::relative_humidity_to_absolute(relative_humidity, celsius);
            let sealevel_pressure = conversion::hpa_to_inhg(conversion::raw_pressure_to_sealevel(
                raw_pressure_hpa,
                celsius,
            ));

            tx.send(adafruit::Metric {
                feed: "mbr.temperature".into(),
                value: fahrenheit,
            })
            .unwrap();
            tx.send(adafruit::Metric {
                feed: "mbr.humidity".into(),
                value: relative_humidity,
            })
            .unwrap();
            tx.send(adafruit::Metric {
                feed: "mbr.abs-humidity".into(),
                value: state.last_abs_humidity,
            })
            .unwrap();
            tx.send(adafruit::Metric {
                feed: "mbr.pressure".into(),
                value: sealevel_pressure,
            })
            .unwrap();
        }

        state.temperature_sum = 0.0;
        state.humidity_sum = 0.0;
        state.pressure_sum = 0.0;
        state.count = 0;
        state.last_update = now;
    }
}