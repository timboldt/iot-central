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
use embedded_hal::blocking::{delay, i2c};
use log::debug;
use sgp30::Sgp30;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

const UPDATE_PERIOD: Duration = Duration::from_secs(60);

pub struct State {
    pub sensor_is_valid: bool,
    pub abs_humidity: f32,
    pub last_update: Instant,
    pub co2_sum: f32,
    pub co2_count: i32,
    pub tvoc_sum: f32,
    pub tvoc_count: i32,
    pub raw_h2_sum: f32,
    pub raw_ethanol_sum: f32,
    pub raw_count: i32,
}

pub fn poll<I2C, D, E>(
    sgp: &mut Sgp30<I2C, D>,
    state: &mut State,
    tx: &mpsc::Sender<adafruit::Metric>,
) where
    I2C: i2c::Read<Error = E> + i2c::Write<Error = E> + i2c::WriteRead<Error = E>,
    D: delay::DelayUs<u16> + delay::DelayMs<u16>,
{
    if let Ok(humidity) = sgp30::Humidity::from_f32(state.abs_humidity) {
        let _ = sgp.set_humidity(Some(&humidity));
    }
    let measurements = sgp.measure().unwrap_or(sgp30::Measurement {
        co2eq_ppm: 0,
        tvoc_ppb: 0,
    });
    let raw = sgp
        .measure_raw_signals()
        .unwrap_or(sgp30::RawSignals { h2: 0, ethanol: 0 });

    if measurements.co2eq_ppm != 400 {
        debug!("SGP: CO₂eq = {}", measurements.co2eq_ppm);
        state.co2_sum += measurements.co2eq_ppm as f32;
        state.co2_count += 1;
    }
    if measurements.tvoc_ppb != 0 {
        debug!("TVOC = {} ppb", measurements.tvoc_ppb);
        state.tvoc_sum += measurements.tvoc_ppb as f32;
        state.tvoc_count += 1;
    }
    if raw.h2 > 0 || raw.ethanol > 0 {
        state.raw_h2_sum += raw.h2 as f32;
        state.raw_ethanol_sum += raw.ethanol as f32;
        state.raw_count += 1;
    }

    let now = Instant::now();
    if now.duration_since(state.last_update) > UPDATE_PERIOD {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async move {
            if state.co2_count > 0 {
                tx.send(adafruit::Metric {
                    feed: "mbr-sgp30.co2".into(),
                    value: state.co2_sum / state.co2_count as f32,
                })
                .await
                .unwrap();
            }

            if state.tvoc_count > 0 {
                tx.send(adafruit::Metric {
                    feed: "mbr-sgp30.tvoc".into(),
                    value: state.tvoc_sum / state.tvoc_count as f32,
                })
                .await
                .unwrap();
            }

            if state.raw_count > 0 {
                tx.send(adafruit::Metric {
                    feed: "mbr-sgp30.raw-h2".into(),
                    value: state.raw_h2_sum / state.raw_count as f32,
                })
                .await
                .unwrap();
                tx.send(adafruit::Metric {
                    feed: "mbr-sgp30.raw-ethanol".into(),
                    value: state.raw_ethanol_sum / state.raw_count as f32,
                })
                .await
                .unwrap();
            }

            state.co2_sum = 0.0;
            state.co2_count = 0;
            state.tvoc_sum = 0.0;
            state.tvoc_count = 0;
            state.last_update = now;
            state.raw_h2_sum = 0.0;
            state.raw_ethanol_sum = 0.0;
            state.raw_count = 0;
        });
    }
}
