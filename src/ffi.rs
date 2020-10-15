#![allow(dead_code)]

use core::convert::TryFrom;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

impl bsec_virtual_sensor_t {
    /// All known sensors.
    pub const ALL: &'static [bsec_virtual_sensor_t] = &[
        bsec_virtual_sensor_t::BSEC_OUTPUT_IAQ,
        bsec_virtual_sensor_t::BSEC_OUTPUT_STATIC_IAQ,
        bsec_virtual_sensor_t::BSEC_OUTPUT_CO2_EQUIVALENT,
        bsec_virtual_sensor_t::BSEC_OUTPUT_BREATH_VOC_EQUIVALENT,
        bsec_virtual_sensor_t::BSEC_OUTPUT_RAW_TEMPERATURE,
        bsec_virtual_sensor_t::BSEC_OUTPUT_RAW_PRESSURE,
        bsec_virtual_sensor_t::BSEC_OUTPUT_RAW_HUMIDITY,
        bsec_virtual_sensor_t::BSEC_OUTPUT_RAW_GAS,
        bsec_virtual_sensor_t::BSEC_OUTPUT_STABILIZATION_STATUS,
        bsec_virtual_sensor_t::BSEC_OUTPUT_RUN_IN_STATUS,
        bsec_virtual_sensor_t::BSEC_OUTPUT_SENSOR_HEAT_COMPENSATED_TEMPERATURE,
        bsec_virtual_sensor_t::BSEC_OUTPUT_SENSOR_HEAT_COMPENSATED_HUMIDITY,
        bsec_virtual_sensor_t::BSEC_OUTPUT_COMPENSATED_GAS,
        bsec_virtual_sensor_t::BSEC_OUTPUT_GAS_PERCENTAGE,
    ];
}

impl TryFrom<u8> for bsec_virtual_sensor_t {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(bsec_virtual_sensor_t::BSEC_OUTPUT_IAQ),
            2 => Ok(bsec_virtual_sensor_t::BSEC_OUTPUT_STATIC_IAQ),
            3 => Ok(bsec_virtual_sensor_t::BSEC_OUTPUT_CO2_EQUIVALENT),
            4 => Ok(bsec_virtual_sensor_t::BSEC_OUTPUT_BREATH_VOC_EQUIVALENT),
            6 => Ok(bsec_virtual_sensor_t::BSEC_OUTPUT_RAW_TEMPERATURE),
            7 => Ok(bsec_virtual_sensor_t::BSEC_OUTPUT_RAW_PRESSURE),
            8 => Ok(bsec_virtual_sensor_t::BSEC_OUTPUT_RAW_HUMIDITY),
            9 => Ok(bsec_virtual_sensor_t::BSEC_OUTPUT_RAW_GAS),
            12 => Ok(bsec_virtual_sensor_t::BSEC_OUTPUT_STABILIZATION_STATUS),
            13 => Ok(bsec_virtual_sensor_t::BSEC_OUTPUT_RUN_IN_STATUS),
            14 => Ok(bsec_virtual_sensor_t::BSEC_OUTPUT_SENSOR_HEAT_COMPENSATED_TEMPERATURE),
            15 => Ok(bsec_virtual_sensor_t::BSEC_OUTPUT_SENSOR_HEAT_COMPENSATED_HUMIDITY),
            18 => Ok(bsec_virtual_sensor_t::BSEC_OUTPUT_COMPENSATED_GAS),
            21 => Ok(bsec_virtual_sensor_t::BSEC_OUTPUT_GAS_PERCENTAGE),

            _ => Err(()),
        }
    }
}
