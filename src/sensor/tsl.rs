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
use embedded_hal::blocking::i2c;
#[cfg(feature = "ftdi")]
use ftdi_embedded_hal as hal;
#[cfg(feature = "rpi")]
use linux_embedded_hal as hal;
use log::debug;
use std::sync::mpsc;
use std::time::{Duration, Instant};

const UPDATE_PERIOD: Duration = Duration::from_secs(60);

pub struct State {
    pub sensor_is_valid: bool,
    pub delay: hal::Delay,
    pub last_update: Instant,
    pub lux_sum: f32,
    pub lux_count: i32,
}

pub fn poll<I2C, E>(
    tsl: &mut tsl2591::Driver<I2C>,
    state: &mut State,
    tx: &mpsc::Sender<adafruit::Metric>,
) where
    I2C: i2c::Read<Error = E> + i2c::Write<Error = E> + i2c::WriteRead<Error = E>,
{
    let (ch_0, ch_1) = tsl
        .get_channel_data(&mut state.delay)
        .unwrap_or((0xFFFF, 0xFFFF));
    let lux = tsl.calculate_lux(ch_0, ch_1).unwrap_or(f32::NAN);

    if !lux.is_nan() {
        debug!("TSL2591: lux = {}", lux);
        state.lux_sum += lux;
        state.lux_count += 1;
    }

    let now = Instant::now();
    if now.duration_since(state.last_update) > UPDATE_PERIOD {
        if state.lux_count > 0 {
            tx.send(adafruit::Metric {
                feed: "indoor-env.lux".into(),
                value: state.lux_sum / state.lux_count as f32,
            })
            .unwrap();
        }

        state.lux_sum = 0.0;
        state.lux_count = 0;
        state.last_update = now;
    }
}
