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

#[cfg(feature = "ftdi")]
use ftdi_embedded_hal as hal;

use log::{debug, info};
use sgp30::Sgp30;
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::time::Duration;

#[derive(Debug)]
pub struct CallParams {
    pub shutdown: Arc<(Mutex<bool>, Condvar)>,
    pub tx: mpsc::Sender<adafruit::Metric>,
}

pub fn sensor_updater(params: CallParams) {
    info!("sensor_updater starting");
    debug!("sensor_updater parameters {:?}", params);
    //let update_period = Duration::from_secs(3 * 60);
    let sensor_period = Duration::from_millis(1000 - 12 - 25);  // What are these magic numbers from?

    #[cfg(feature = "ftdi")]
    let device = ftdi::find_by_vid_pid(0x0403, 0x6014)
        .interface(ftdi::Interface::A)
        .open()
        .unwrap();

    let hal = hal::FtHal::init_default(device).unwrap();
    let i2c = hal.i2c().unwrap();
    let address = 0x58;
    let mut sgp = Sgp30::new(i2c, address, hal::Delay::default());
    println!("Initializing SGP30...");
    sgp.init().unwrap();
    loop {
        let measurements = sgp.measure().unwrap_or(sgp30::Measurement{ co2eq_ppm: 0, tvoc_ppb: 0});
        println!(
            "SGP: CO₂eq = {} ppm, TVOC = {} ppb",
            measurements.co2eq_ppm, measurements.tvoc_ppb
        );

        // Wait for next update period, or  shutdown signal.
        let (lock, cvar) = &*params.shutdown;
        let shutdown = cvar
            .wait_timeout_while(lock.lock().unwrap(), sensor_period, |&mut shutdown| {
                !shutdown
            })
            .unwrap();
        if *shutdown.0 {
            break;
        }
    }
    info!("sensor_updater finished");
}