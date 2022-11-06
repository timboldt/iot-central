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

#[cfg(all(feature = "ftdi", feature = "rpi"))]
compile_error!("feature \"ftdi\" and feature \"rpi\" cannot be enabled at the same time");

#[cfg(feature = "ftdi")]
extern crate ftdi;

use crate::adafruit;

use bme280::BME280;
#[cfg(feature = "ftdi")]
use ftdi_embedded_hal as hal;
#[cfg(feature = "rpi")]
use linux_embedded_hal as hal;
use log::{debug, error, info};
use sgp30::Sgp30;
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

const UPDATE_PERIOD: Duration = Duration::from_secs(60);
const SENSOR_PERIOD: Duration = Duration::from_millis(1000);

#[derive(Debug)]
pub struct CallParams {
    pub shutdown: Arc<(Mutex<bool>, Condvar)>,
    pub tx: mpsc::Sender<adafruit::Metric>,
}

pub fn sensor_updater(params: CallParams) {
    info!("sensor_updater starting");
    debug!("sensor_updater parameters {:?}", params);

    #[cfg(feature = "ftdi")]
    let ftdi_hal = {
        let device = ftdi::find_by_vid_pid(0x0403, 0x6014)
            .interface(ftdi::Interface::A)
            .open()
            .expect("FTDI USB device not found.");
        hal::FtHal::init_default(device).expect("Unable to initialize FTDI USB device.")
    };

    #[cfg(feature = "ftdi")]
    let (i2c, delay) = (
        ftdi_hal.i2c().expect("Unable to find FTDI I2C bus."),
        hal::Delay::default(),
    );

    #[cfg(feature = "rpi")]
    let (i2c, delay) = (
        hal::I2cdev::new("/dev/i2c-1").expect("Unable to find RPI I2C-1 bus."),
        hal::Delay,
    );

    let i2c = shared_bus::BusManagerSimple::new(i2c);
    let mut bme = BME280::new_secondary(i2c.acquire_i2c(), delay);
    let mut bme_state = BME280State {
        sensor_is_valid: false,
        last_update: Instant::now(),
        temperature_sum: 0.0,
        temperature_count: 0,
        humidity_sum: 0.0,
        humidity_count: 0,
        pressure_sum: 0.0,
        pressure_count: 0,
    };
    match bme.init() {
        Ok(()) => {
            info!("BME280 initialized");
            bme_state.sensor_is_valid = true;
        }
        Err(e) => error!("BME280 not found: {:?}", e),
    };

    let sgp30_address = 0x58;
    let mut sgp = Sgp30::new(i2c.acquire_i2c(), sgp30_address, delay);
    let mut sgp_state = SGP30State {
        sensor_is_valid: false,
        last_update: Instant::now(),
        co2_sum: 0.0,
        co2_count: 0,
        tvoc_sum: 0.0,
        tvoc_count: 0,
    };
    match sgp.init() {
        Ok(()) => {
            info!("SGP30 initialized");
            sgp_state.sensor_is_valid = true;
        }
        Err(e) => error!("SGP30 not found: {:?}", e),
    };

    // TODO: Fix this driver so that it doesn't have to fail during new().
    let mut tsl = tsl2591::Driver::new(i2c.acquire_i2c()).unwrap();
    // let mut tsl = match tsl2591::Driver::new(i2c.acquire_i2c()) {
    //     Ok(mut t) => Some(mut t),
    //     Err(e) => {
    //         error!("TSL2591 not found: {:?}", e);
    //         None
    //     }
    // };
    let mut tsl_state = TSL2591State {
        sensor_is_valid: false,
        delay: delay,
        last_update: Instant::now(),
        lux_sum: 0.0,
        lux_count: 0,
    };

    match tsl.enable() {
        Ok(()) => {}
        Err(e) => {
            error!("TSL2591 not enabled: {:?}", e);
        }
    };
    match tsl.set_timing(None) {
        Ok(()) => {}
        Err(e) => {
            error!("TSL2591 timing not set: {:?}", e);
        }
    };
    match tsl.set_gain(None) {
        Ok(()) => {}
        Err(e) => {
            error!("TSL2591 gain not set: {:?}", e);
        }
    };

    loop {
        if bme_state.sensor_is_valid {
            poll_bme280(&mut bme, &mut bme_state, &params.tx);
        }
        if sgp_state.sensor_is_valid {
            poll_sgp30(&mut sgp, &mut sgp_state, &params.tx);
        }
        if tsl_state.sensor_is_valid {
            poll_tsl2591(&mut tsl, &mut tsl_state, &params.tx);
        }

        // Wait for next sensor period, or shutdown signal.
        let (lock, cvar) = &*params.shutdown;
        let shutdown = cvar
            .wait_timeout_while(lock.lock().unwrap(), SENSOR_PERIOD, |&mut shutdown| {
                !shutdown
            })
            .unwrap();
        if *shutdown.0 {
            break;
        }
    }
    info!("sensor_updater finished");
}

struct BME280State {
    sensor_is_valid: bool,
    last_update: Instant,
    temperature_sum: f32,
    temperature_count: i32,
    humidity_sum: f32,
    humidity_count: i32,
    pressure_sum: f32,
    pressure_count: i32,
}

fn poll_bme280(
    #[cfg(feature = "ftdi")] bme: &mut BME280<
        shared_bus::I2cProxy<shared_bus::NullMutex<hal::I2c<ftdi::Device>>>,
        hal::Delay,
    >,
    #[cfg(feature = "rpi")] bme: &mut BME280<
        shared_bus::I2cProxy<shared_bus::NullMutex<hal::I2cdev>>,
        hal::Delay,
    >,
    state: &mut BME280State,
    tx: &mpsc::Sender<adafruit::Metric>,
) {
    if let Ok(measurements) = bme.measure() {
        debug!("BME: measurements = {:?}", measurements);
        state.temperature_sum += measurements.temperature;
        state.temperature_count += 1;
        state.humidity_sum += measurements.humidity;
        state.humidity_count += 1;
        state.pressure_sum += measurements.pressure;
        state.pressure_count += 1;
    }

    let now = Instant::now();
    if now.duration_since(state.last_update) > UPDATE_PERIOD {
        if state.temperature_count > 0 {
            tx.send(adafruit::Metric {
                feed: "indoor-env.temp".into(),
                value: state.temperature_sum / state.temperature_count as f32,
            })
            .unwrap();
        }
        if state.humidity_count > 0 {
            tx.send(adafruit::Metric {
                feed: "indoor-env.humidity".into(),
                value: state.humidity_sum / state.humidity_count as f32,
            })
            .unwrap();
        }
        if state.pressure_count > 0 {
            tx.send(adafruit::Metric {
                feed: "indoor-env.pressure".into(),
                value: state.pressure_sum / state.pressure_count as f32,
            })
            .unwrap();
        }

        state.temperature_sum = 0.0;
        state.temperature_count = 0;
        state.humidity_sum = 0.0;
        state.humidity_count = 0;
        state.pressure_sum = 0.0;
        state.pressure_count = 0;
        state.last_update = now;
    }
}

struct SGP30State {
    sensor_is_valid: bool,
    last_update: Instant,
    co2_sum: f32,
    co2_count: i32,
    tvoc_sum: f32,
    tvoc_count: i32,
}

fn poll_sgp30(
    #[cfg(feature = "ftdi")] sgp: &mut Sgp30<
        shared_bus::I2cProxy<shared_bus::NullMutex<hal::I2c<ftdi::Device>>>,
        hal::Delay,
    >,
    #[cfg(feature = "rpi")] sgp: &mut Sgp30<
        shared_bus::I2cProxy<shared_bus::NullMutex<hal::I2cdev>>,
        hal::Delay,
    >,
    state: &mut SGP30State,
    tx: &mpsc::Sender<adafruit::Metric>,
) {
    let measurements = sgp.measure().unwrap_or(sgp30::Measurement {
        co2eq_ppm: 0,
        tvoc_ppb: 0,
    });

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

    let now = Instant::now();
    if now.duration_since(state.last_update) > UPDATE_PERIOD {
        if state.co2_count > 0 {
            tx.send(adafruit::Metric {
                feed: "indoor-env.co2".into(),
                value: state.co2_sum / state.co2_count as f32,
            })
            .unwrap();
        }

        if state.tvoc_count > 0 {
            tx.send(adafruit::Metric {
                feed: "indoor-env.tvoc".into(),
                value: state.tvoc_sum / state.tvoc_count as f32,
            })
            .unwrap();
        }

        state.co2_sum = 0.0;
        state.co2_count = 0;
        state.tvoc_sum = 0.0;
        state.tvoc_count = 0;
        state.last_update = now;
    }
}

struct TSL2591State {
    sensor_is_valid: bool,
    delay: hal::Delay,
    last_update: Instant,
    lux_sum: f32,
    lux_count: i32,
}

fn poll_tsl2591(
    #[cfg(feature = "ftdi")] tsl: &mut tsl2591::Driver<
        shared_bus::I2cProxy<shared_bus::NullMutex<hal::I2c<ftdi::Device>>>,
    >,
    #[cfg(feature = "rpi")] tsl: &mut tsl2591::Driver<
        shared_bus::I2cProxy<shared_bus::NullMutex<hal::I2cdev>>,
    >,
    state: &mut TSL2591State,
    tx: &mpsc::Sender<adafruit::Metric>,
) {
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
