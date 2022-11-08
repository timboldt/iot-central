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

mod bme;
mod sgp;
mod tsl;

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
    let mut bme_state = bme::State {
        sensor_is_valid: false,
        last_update: Instant::now(),
        temperature_sum: 0.0,
        humidity_sum: 0.0,
        pressure_sum: 0.0,
        count: 0,
    };
    match bme.init() {
        Ok(()) => {
            info!("BME280 initialized");
            bme_state.sensor_is_valid = true;
        }
        Err(e) => error!("BME280 not found: {:?}", e),
    };

    #[cfg(feature = "rpi")]
    let delay = hal::Delay;
    let sgp30_address = 0x58;
    let mut sgp = Sgp30::new(i2c.acquire_i2c(), sgp30_address, delay);
    let mut sgp_state = sgp::State {
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

    #[cfg(feature = "rpi")]
    let delay = hal::Delay;
    let mut tsl_state = tsl::State {
        sensor_is_valid: true,
        delay,
        last_update: Instant::now(),
        lux_sum: 0.0,
        lux_count: 0,
    };
    let mut tsl = match tsl2591::Driver::new(i2c.acquire_i2c()) {
        Ok(mut t) => {
            match t.enable() {
                Ok(()) => {}
                Err(e) => {
                    tsl_state.sensor_is_valid = false;
                    error!("TSL2591 not enabled: {:?}", e);
                }
            };
            match t.set_timing(None) {
                Ok(()) => {}
                Err(e) => {
                    tsl_state.sensor_is_valid = false;
                    error!("TSL2591 timing not set: {:?}", e);
                }
            };
            match t.set_gain(None) {
                Ok(()) => {}
                Err(e) => {
                    tsl_state.sensor_is_valid = false;
                    error!("TSL2591 gain not set: {:?}", e);
                }
            };
            Some(t)
        }
        Err(e) => {
            tsl_state.sensor_is_valid = false;
            error!("TSL2591 not found: {:?}", e);
            None
        }
    };
    if tsl_state.sensor_is_valid {
        info!("TSL2591 initialized");
    }

    loop {
        if bme_state.sensor_is_valid {
            bme::poll(&mut bme, &mut bme_state, &params.tx);
        }
        if sgp_state.sensor_is_valid {
            sgp::poll(&mut sgp, &mut sgp_state, &params.tx);
        }
        if tsl_state.sensor_is_valid {
            if let Some(t) = tsl.as_mut() {
                tsl::poll(t, &mut tsl_state, &params.tx);
            }
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