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

use core::f32::consts;

// Local altitude in meters.
const ALTITUDE: f32 = 100.0;

pub fn celsius_to_fahrenheit(celsius: f32) -> f32 {
    celsius * 1.8 + 32.0
}

pub fn celsius_to_kelvin(celsius: f32) -> f32 {
    celsius + 273.15
}

pub fn raw_pressure_to_sealevel(raw_hpa: f32, celsius: f32) -> f32 {
    raw_hpa * (1.0 - 0.0065 * ALTITUDE / (0.0065 + celsius_to_kelvin(celsius))).powf(-5.257)
}

pub fn hpa_to_inhg(hpa: f32) -> f32 {
    hpa / 33.863_888
}

// https://sensirion.com/media/documents/984E0DD5/61644B8B/Sensirion_Gas_Sensors_Datasheet_SGP30.pdf
// relative_humidity should be a percentage value between 0 and 100.
// Output is in grams per cubic meter.
pub fn relative_humidity_to_absolute(relative_humidity: f32, celsius: f32) -> f32 {
    let saturating_pressure = 6.112 * consts::E.powf(17.62 * celsius / (243.12 + celsius));
    let pressure = saturating_pressure * relative_humidity / 100.0;
    216.7 * pressure / celsius_to_kelvin(celsius)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_to_f_works() {
        assert_eq!(68.0, celsius_to_fahrenheit(20.0));
    }

    #[test]
    fn c_to_k_works() {
        assert_eq!(293.15, celsius_to_kelvin(20.0));
    }

    #[test]
    fn hpa_to_inhg_works() {
        assert_eq!(29.9, (hpa_to_inhg(1013.25) * 10.0).round() / 10.0);
    }

    #[test]
    fn rp_to_s_works() {
        assert_eq!(
            1_012.0,
            raw_pressure_to_sealevel(1000.0, 15.0).round()
        );
        assert_eq!(
            1_025.0,
            raw_pressure_to_sealevel(1_013.25, 15.0).round()
        );
        assert_eq!(
            1_010.0,
            raw_pressure_to_sealevel(999.0, 40.0).round()
        );
        assert_eq!(
            1_010.0,
            raw_pressure_to_sealevel(999.0, 40.0).round()
        );
    }

    #[test]
    fn rh_to_ah_works() {
        assert_eq!(
            6.4,
            (relative_humidity_to_absolute(50.0, 15.0) * 100.0).round() / 100.0
        );
    }
}
