#![no_std]
#![allow(clippy::redundant_static_lifetimes)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use core::fmt::{Display, Formatter};

mod compat;
mod ffi;

pub use ffi::bsec_library_return_t;
pub use ffi::bsec_virtual_sensor_t;

use crate::Accuracy::{High, Low, Medium, Unreliable};
use core::convert::TryInto;
use drogue_bme680::Oversampling;
use embedded_time::duration::Milliseconds;
use ffi::*;

const BSEC_SAMPLE_RATE_DISABLED: f32 = 65535.0f32;
const BSEC_SAMPLE_RATE_ULP: f32 = 0.0033333f32;
const BSEC_SAMPLE_RATE_LP: f32 = 0.33333f32;
const BSEC_SAMPLE_RATE_ULP_MEASUREMENT_ON_DEMAND: f32 = 0.0f32;

pub enum SampleRate {
    Disabled,
    LowPower,
    UltraLowPower,
    OnDemand,
}

impl SampleRate {
    pub(crate) fn frequency(&self) -> f32 {
        match self {
            SampleRate::Disabled => BSEC_SAMPLE_RATE_DISABLED,
            SampleRate::LowPower => BSEC_SAMPLE_RATE_LP,
            SampleRate::UltraLowPower => BSEC_SAMPLE_RATE_ULP,
            SampleRate::OnDemand => BSEC_SAMPLE_RATE_ULP_MEASUREMENT_ON_DEMAND,
        }
    }
}

impl From<SampleRate> for f32 {
    fn from(value: SampleRate) -> Self {
        value.frequency()
    }
}

impl From<&SampleRate> for f32 {
    fn from(value: &SampleRate) -> Self {
        value.frequency()
    }
}

const NUM_VIRTUAL_SENSORS: u8 = 14;
const NUM_PHYSICAL_SENSORS: u8 = 8;

#[derive(Copy, Clone, Debug)]
pub struct Error(pub bsec_library_return_t);
type Result<T> = core::result::Result<T, Error>;

const EMPTY_SENSORS: [bsec_sensor_configuration_t; 0] = [];
const EMPTY_INPUT: bsec_input_t = bsec_input_t {
    sensor_id: 0,
    signal: 0.0,
    signal_dimensions: 0,
    time_stamp: 0,
};
const EMPTY_OUTPUT: bsec_output_t = bsec_output_t {
    sensor_id: 0,
    signal: 0.0,
    signal_dimensions: 0,
    time_stamp: 0,
    accuracy: 0,
};

#[derive(Clone, Debug)]
pub struct Control {
    pub next_call: Milliseconds,
    pub heater_temperature: u16,
    pub heating_duration: Milliseconds,
    pub run_gas: bool,
    pub pressure_oversampling: Oversampling,
    pub temperature_oversampling: Oversampling,
    pub humidity_oversampling: Oversampling,
    pub trigger_measurement: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Inputs {
    pub temperature: Option<f32>,
    pub humidity: Option<f32>,
    pub pressure: Option<f32>,
    pub gas_resistance: Option<f32>,
}

#[derive(Clone, Debug)]
pub enum Accuracy<T> {
    Unreliable,
    Low(T),
    Medium(T),
    High(T),
}

impl<T> Accuracy<T> {
    pub fn value(&self) -> Option<&T> {
        match self {
            Unreliable => None,
            Low(v) | Medium(v) | High(v) => Some(v),
        }
    }
}

impl<T> Default for Accuracy<T> {
    fn default() -> Self {
        Unreliable
    }
}

impl<T> From<Option<Accuracy<T>>> for Accuracy<T> {
    fn from(v: Option<Accuracy<T>>) -> Self {
        match v {
            Some(v) => v,
            None => Unreliable,
        }
    }
}

impl From<&bsec_output_t> for Accuracy<f32> {
    fn from(value: &bsec_output_t) -> Self {
        match value.accuracy {
            1 => Low(value.signal),
            2 => Medium(value.signal),
            3 => High(value.signal),
            _ => Unreliable,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Outputs {
    pub iaq: Option<Accuracy<f32>>,
    pub static_iaq: Option<Accuracy<f32>>,
    pub co2_equivalent: Option<Accuracy<f32>>,
    pub breath_voc_equivalent: Option<Accuracy<f32>>,
    pub raw_temperature: Option<f32>,
    pub raw_pressure: Option<f32>,
    pub raw_humidity: Option<f32>,
    pub raw_gas_resistance: Option<f32>,
    pub stabilization_status: Option<f32>,
    pub run_in_status: Option<f32>,
    pub sensor_heat_compensated_temperature: Option<f32>,
    pub sensor_heat_compensated_humidity: Option<f32>,
    pub compensated_gas: Option<Accuracy<f32>>,
    pub gas_percentage: Option<Accuracy<f32>>,
}

macro_rules! sensors {
    ($id:ident, $($next:ident),+) => {
        [ sensors!($id), $(sensors!($next)),* ]
    };
    ($id:ident) => {
        bsec_sensor_configuration_t {
            sensor_id: crate::bsec_virtual_sensor_t::$id as u8,
            sample_rate: BSEC_SAMPLE_RATE_DISABLED,
        }
    };
}

pub struct Bsec {
    virtual_sensors: [bsec_sensor_configuration_t; NUM_VIRTUAL_SENSORS as usize],
    internal_physical_sensors: [bsec_sensor_configuration_t; NUM_PHYSICAL_SENSORS as usize],
    num_physical_sensors: usize,
}

impl Bsec {
    pub fn new() -> Result<Option<Self>> {
        Some(
            Bsec {
                virtual_sensors: sensors![
                    BSEC_OUTPUT_IAQ,
                    BSEC_OUTPUT_STATIC_IAQ,
                    BSEC_OUTPUT_CO2_EQUIVALENT,
                    BSEC_OUTPUT_BREATH_VOC_EQUIVALENT,
                    BSEC_OUTPUT_RAW_TEMPERATURE,
                    BSEC_OUTPUT_RAW_PRESSURE,
                    BSEC_OUTPUT_RAW_HUMIDITY,
                    BSEC_OUTPUT_RAW_GAS,
                    BSEC_OUTPUT_STABILIZATION_STATUS,
                    BSEC_OUTPUT_RUN_IN_STATUS,
                    BSEC_OUTPUT_SENSOR_HEAT_COMPENSATED_TEMPERATURE,
                    BSEC_OUTPUT_SENSOR_HEAT_COMPENSATED_HUMIDITY,
                    BSEC_OUTPUT_COMPENSATED_GAS,
                    BSEC_OUTPUT_GAS_PERCENTAGE
                ],
                internal_physical_sensors: [bsec_sensor_configuration_t {
                    sensor_id: 0,
                    sample_rate: BSEC_SAMPLE_RATE_DISABLED,
                }; NUM_PHYSICAL_SENSORS as usize],
                num_physical_sensors: 0,
            }
            .init(),
        )
        .transpose()
    }

    fn init(self) -> Result<Self> {
        match unsafe { bsec_init() } {
            bsec_library_return_t::BSEC_OK => Ok(self),
            e => Err(Error(e)),
        }
    }

    fn physical_sensors(&self) -> &[bsec_sensor_configuration_t] {
        &self.internal_physical_sensors[0..self.num_physical_sensors]
    }

    pub fn update_subscription(
        &mut self,
        sample_rate: SampleRate,
        sensors: &[bsec_virtual_sensor_t],
    ) -> Result<()> {
        self.do_set_subscription(false, sample_rate, sensors)
    }

    pub fn set_subscription(
        &mut self,
        sample_rate: SampleRate,
        sensors: &[bsec_virtual_sensor_t],
    ) -> Result<()> {
        self.do_set_subscription(true, sample_rate, sensors)
    }

    fn do_set_subscription(
        &mut self,
        disable_others: bool,
        sample_rate: SampleRate,
        sensors: &[bsec_virtual_sensor_t],
    ) -> Result<()> {
        if disable_others {
            // reset all sensors
            for sensor in &mut self.virtual_sensors {
                sensor.sample_rate = BSEC_SAMPLE_RATE_DISABLED;
            }
        }

        // set new sensors
        for update in sensors {
            for sensor in &mut self.virtual_sensors {
                if sensor.sensor_id == *update as u8 {
                    sensor.sample_rate = sample_rate.frequency();
                }
            }
        }

        log::info!("Subscription: {:?}", self.virtual_sensors);

        let mut required_sensors_len = NUM_PHYSICAL_SENSORS;
        match unsafe {
            bsec_update_subscription(
                self.virtual_sensors.as_ptr(),
                NUM_VIRTUAL_SENSORS,
                self.internal_physical_sensors.as_mut_ptr(),
                &mut required_sensors_len,
            )
        } {
            bsec_library_return_t::BSEC_OK => {
                assert!(required_sensors_len <= NUM_PHYSICAL_SENSORS);
                self.num_physical_sensors = required_sensors_len as usize;
                Ok(())
            }
            e => Err(Error(e)),
        }
    }

    pub fn sensor_control(&self, now: Milliseconds) -> Result<Control> {
        let now = (now.0 as i64) * 1_000_000;

        let mut settings = bsec_bme_settings_t {
            next_call: 0,
            process_data: 0,
            heater_temperature: 0,
            heating_duration: 0,
            run_gas: 0,
            pressure_oversampling: 0,
            temperature_oversampling: 0,
            humidity_oversampling: 0,
            trigger_measurement: 0,
        };

        let result = unsafe { bsec_sensor_control(now, &mut settings) };

        log::info!("bsec_sensor_control -> {:?} = {:?}", result, settings);

        match result {
            bsec_library_return_t::BSEC_OK => Ok(Control {
                next_call: Milliseconds((settings.next_call / 1_000_000) as u32),
                heater_temperature: settings.heater_temperature,
                heating_duration: Milliseconds(settings.heating_duration as u32),
                run_gas: settings.run_gas != 0,
                pressure_oversampling: settings.pressure_oversampling.into(),
                temperature_oversampling: settings.temperature_oversampling.into(),
                humidity_oversampling: settings.humidity_oversampling.into(),
                trigger_measurement: settings.trigger_measurement != 0,
            }),
            e => Err(Error(e)),
        }
    }

    pub fn process_data(
        &mut self,
        timestamp: Milliseconds,
        input_data: &Inputs,
    ) -> Result<Outputs> {
        let mut input = [EMPTY_INPUT; NUM_PHYSICAL_SENSORS as usize];

        let timestamp = (timestamp.0 as i64) * 1_000_000;

        let mut idx = 0u8;
        if let Some(temperature) = input_data.temperature {
            input[idx as usize] = bsec_input_t {
                sensor_id: bsec_physical_sensor_t::BSEC_INPUT_TEMPERATURE as u8,
                signal: temperature / 100.0,
                time_stamp: timestamp,
                signal_dimensions: 0,
            };
            idx += 1;
        }
        if let Some(humidity) = input_data.humidity {
            input[idx as usize] = bsec_input_t {
                sensor_id: bsec_physical_sensor_t::BSEC_INPUT_HUMIDITY as u8,
                signal: humidity / 100.0,
                time_stamp: timestamp,
                signal_dimensions: 0,
            };
            idx += 1;
        }
        if let Some(pressure) = input_data.pressure {
            input[idx as usize] = bsec_input_t {
                sensor_id: bsec_physical_sensor_t::BSEC_INPUT_PRESSURE as u8,
                signal: pressure,
                time_stamp: timestamp,
                signal_dimensions: 0,
            };
            idx += 1;
        }
        if let Some(gas) = input_data.gas_resistance {
            input[idx as usize] = bsec_input_t {
                sensor_id: bsec_physical_sensor_t::BSEC_INPUT_GASRESISTOR as u8,
                signal: gas,
                time_stamp: timestamp,
                signal_dimensions: 0,
            };
            idx += 1;
        }

        let mut output = [EMPTY_OUTPUT; NUM_VIRTUAL_SENSORS as usize];
        let mut num_outputs = NUM_VIRTUAL_SENSORS;

        log::info!(
            "Provided inputs: {}, available outputs: {}",
            idx,
            num_outputs
        );

        let result =
            unsafe { bsec_do_steps(input.as_ptr(), idx, output.as_mut_ptr(), &mut num_outputs) };

        log::info!("do_steps = {:?}", result);

        assert!(num_outputs <= NUM_VIRTUAL_SENSORS);

        log::info!("Returned outputs: {}", num_outputs);
        let output = &output[..num_outputs as usize];

        let mut outputs = Outputs::default();

        for out in output {
            log::info!("Output: {:?}", out);

            match out.sensor_id.try_into() {
                Ok(bsec_virtual_sensor_t::BSEC_OUTPUT_IAQ) => outputs.iaq = Some(out.into()),
                Ok(bsec_virtual_sensor_t::BSEC_OUTPUT_STATIC_IAQ) => {
                    outputs.static_iaq = Some(out.into())
                }
                Ok(bsec_virtual_sensor_t::BSEC_OUTPUT_CO2_EQUIVALENT) => {
                    outputs.co2_equivalent = Some(out.into())
                }
                Ok(bsec_virtual_sensor_t::BSEC_OUTPUT_BREATH_VOC_EQUIVALENT) => {
                    outputs.breath_voc_equivalent = Some(out.into())
                }
                Ok(bsec_virtual_sensor_t::BSEC_OUTPUT_RAW_TEMPERATURE) => {
                    outputs.raw_temperature = Some(out.signal)
                }
                Ok(bsec_virtual_sensor_t::BSEC_OUTPUT_RAW_PRESSURE) => {
                    outputs.raw_pressure = Some(out.signal)
                }
                Ok(bsec_virtual_sensor_t::BSEC_OUTPUT_RAW_HUMIDITY) => {
                    outputs.raw_humidity = Some(out.signal)
                }
                Ok(bsec_virtual_sensor_t::BSEC_OUTPUT_RAW_GAS) => {
                    outputs.raw_gas_resistance = Some(out.signal)
                }
                Ok(bsec_virtual_sensor_t::BSEC_OUTPUT_STABILIZATION_STATUS) => {
                    outputs.stabilization_status = Some(out.signal)
                }
                Ok(bsec_virtual_sensor_t::BSEC_OUTPUT_RUN_IN_STATUS) => {
                    outputs.run_in_status = Some(out.signal)
                }
                Ok(bsec_virtual_sensor_t::BSEC_OUTPUT_SENSOR_HEAT_COMPENSATED_TEMPERATURE) => {
                    outputs.sensor_heat_compensated_temperature = Some(out.signal)
                }
                Ok(bsec_virtual_sensor_t::BSEC_OUTPUT_SENSOR_HEAT_COMPENSATED_HUMIDITY) => {
                    outputs.sensor_heat_compensated_humidity = Some(out.signal)
                }
                Ok(bsec_virtual_sensor_t::BSEC_OUTPUT_COMPENSATED_GAS) => {
                    outputs.compensated_gas = Some(out.into())
                }
                Ok(bsec_virtual_sensor_t::BSEC_OUTPUT_GAS_PERCENTAGE) => {
                    outputs.gas_percentage = Some(out.into())
                }
                _ => {
                    // Unknown output, we ignore it
                }
            }
        }

        Ok(outputs)
    }
}

#[derive(Copy, Clone, Debug, Eq, Ord, PartialOrd, PartialEq)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub major_bugfix: u8,
    pub minor_bugfix: u8,
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}.{} ({}.{})",
            self.major, self.minor, self.major_bugfix, self.minor_bugfix
        )
    }
}

pub fn version() -> Version {
    unsafe {
        let mut v = bsec_version_t {
            major: 0,
            minor: 0,
            major_bugfix: 0,
            minor_bugfix: 0,
        };

        bsec_get_version(&mut v);

        Version {
            major: v.major,
            minor: v.minor,
            major_bugfix: v.major_bugfix,
            minor_bugfix: v.minor_bugfix,
        }
    }
}
