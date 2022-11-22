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
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tsl2591::{Gain, IntegrationTimes};

const UPDATE_PERIOD: Duration = Duration::from_secs(60);

pub struct State<D>
where
    D: delay::DelayUs<u8> + delay::DelayMs<u8>,
{
    pub sensor_is_valid: bool,
    pub delay: D,
    pub integ_time: IntegrationTimes,
    pub gain: Gain,
    pub last_update: Instant,
    pub lux_sum: f32,
    pub full_spectrum_sum: f32,
    pub infrared_sum: f32,
    pub count: i32,
}

pub fn poll<I2C, D, E>(
    tsl: &mut tsl2591::Driver<I2C>,
    state: &mut State<D>,
    tx: &mpsc::Sender<adafruit::Metric>,
) where
    I2C: i2c::Read<Error = E> + i2c::Write<Error = E> + i2c::WriteRead<Error = E>,
    D: delay::DelayUs<u8> + delay::DelayMs<u8>,
{
    let (ch_0, ch_1) = tsl
        .get_channel_data(&mut state.delay)
        .unwrap_or((0xFFFF, 0xFFFF));
    let lux = calculate_lux(state, ch_0, ch_1);

    if !lux.is_nan() {
        debug!("TSL2591: lux = {}", lux);
        state.lux_sum += lux;
        state.full_spectrum_sum += ch_0 as f32 / gain_factor(state.gain);
        state.infrared_sum += ch_1 as f32 / gain_factor(state.gain);
        state.count += 1;
    }

    adjust_gain(state, ch_0, ch_1);

    let now = Instant::now();
    if now.duration_since(state.last_update) > UPDATE_PERIOD {
        if state.count > 0 {
            tx.send(adafruit::Metric {
                feed: "mbr-tsl2591.lux".into(),
                value: state.lux_sum / state.count as f32,
            })
            .unwrap();
            tx.send(adafruit::Metric {
                feed: "mbr-tsl2591.full-spectrum".into(),
                value: state.full_spectrum_sum / state.count as f32,
            })
            .unwrap();
            tx.send(adafruit::Metric {
                feed: "mbr-tsl2591.infrared".into(),
                value: state.infrared_sum / state.count as f32,
            })
            .unwrap();

            tx.send(adafruit::Metric {
                feed: "mbr.lux".into(),
                value: state.lux_sum / state.count as f32,
            })
            .unwrap();
        }

        state.lux_sum = 0.0;
        state.full_spectrum_sum = 0.0;
        state.infrared_sum = 0.0;
        state.count = 0;
        state.last_update = now;
    }
}

fn adjust_gain<D>(state: &mut State<D>, ch_0: u16, ch_1: u16)
where
    D: delay::DelayUs<u8> + delay::DelayMs<u8>,
{
    const MIN_THRESHOLD: u16 = 1_000;
    const MAX_THRESHOLD: u16 = 50_000;

    if ch_0 == 0xFFFF || ch_1 == 0xFFFF {
        // Lower the gain if we are clipping.
        state.gain = next_gain_down(state.gain);
    } else if ch_0 == 0 || ch_1 == 0 {
        // Raise the gain if we have no signal.
        state.gain = next_gain_up(state.gain);
    } else if ch_0 < MIN_THRESHOLD && ch_1 < MIN_THRESHOLD {
        // Raise the gain to get more resolution.
        state.gain = next_gain_up(state.gain);
    } else if ch_0 > MAX_THRESHOLD && ch_1 > MAX_THRESHOLD {
        // Lower the gain to avoid clipping.
        state.gain = next_gain_down(state.gain);
    }
}

fn next_gain_up(gain: Gain) -> Gain {
    match gain {
        Gain::LOW => Gain::MED,
        Gain::MED => Gain::HIGH,
        _ => Gain::MAX,
    }
}

fn next_gain_down(gain: Gain) -> Gain {
    match gain {
        Gain::MAX => Gain::HIGH,
        Gain::HIGH => Gain::MED,
        _ => Gain::LOW,
    }
}

fn gain_factor(gain: Gain) -> f32 {
    match gain {
        Gain::LOW => 1.,
        Gain::MED => 25.,
        Gain::HIGH => 428.,
        Gain::MAX => 9876.,
    }
}

fn calculate_lux<D>(state: &State<D>, ch_0: u16, ch_1: u16) -> f32
where
    D: delay::DelayUs<u8> + delay::DelayMs<u8>,
{
    if (ch_0 == 0xFFFF) | (ch_1 == 0xFFFF) {
        // Signal an overflow.
        return f32::NAN;
    }
    if ch_0 == 0 {
        // Signal an underflow.
        return f32::NAN;
    }

    let a_time = match state.integ_time {
        IntegrationTimes::_100MS => 100.,
        IntegrationTimes::_200MS => 200.,
        IntegrationTimes::_300MS => 300.,
        IntegrationTimes::_400MS => 400.,
        IntegrationTimes::_500MS => 500.,
        IntegrationTimes::_600MS => 600.,
    };

    let a_gain = gain_factor(state.gain);

    const TSL2591_LUX_DF: f32 = 408.;
    let cpl = (a_time * a_gain) / TSL2591_LUX_DF;
    // let lux = (ch_0 as f32 - ch_1 as f32) * (1.0 - (ch_1 as f32 / ch_0 as f32)) / cpl;
    // Alternative formula, per Adafruit:
    let lux = (ch_0 as f32 - 1.7 * ch_1 as f32) / cpl;

    lux
}
