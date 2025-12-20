// Copyright (c) 2025 Stuart Stock
// SPDX-License-Identifier: MIT OR Apache-2.0

#![doc = include_str!("../README.md")]

pub mod error;
pub mod registers;

use error::Error;
use i2cdev::core::I2CDevice;
use i2cdev::linux::LinuxI2CDevice;
use registers::{
    BATTERY_REG, CELL_VOLTAGE_REG, CHARGING_REG, USBC_VBUS_REG,
    ChargerActivity, ChargingState, UsbCInputState, UsbCPowerDelivery,
};

/// Default I2C address of the Waveshare UPS Hat E
pub const DEFAULT_I2C_ADDRESS: u16 = 0x2d;

/// The default I2C bus device path to interface with the UPS Hat E
pub const DEFAULT_I2C_DEV_PATH: &str = "/dev/i2c-1";

/// Represent the composite power state of the UPS Hat E.
#[derive(Debug)]
pub struct PowerState {
    pub charging_state: ChargingState,
    pub charger_activity: ChargerActivity,
    pub usbc_input_state: UsbCInputState,
    pub usbc_power_delivery: UsbCPowerDelivery,
}

/// Aggregate battery state of the UPS Hat E.
///
/// A negative `milliamps` value indicates the UPS is discharging the battery cells. A positive
/// `milliamps` value indicates the UPS has USB-C power and is charging.
///
/// The Waveshare wiki states it may take a few charge cycles for the UPS to calibrate the
/// `remaining_*` and `time_to_full_minutes` values correctly.
#[derive(Debug)]
pub struct BatteryState {
    pub millivolts: u16,
    pub milliamps: i16,
    pub remaining_percent: u16,
    pub remaining_capacity_milliamphours: u16,
    pub remaining_runtime_minutes: u16,
    pub time_to_full_minutes: u16,
}

/// Voltage readings for each of the four battery cells.
#[derive(Debug)]
pub struct CellVoltage {
    pub cell_1_millivolts: u16,
    pub cell_2_millivolts: u16,
    pub cell_3_millivolts: u16,
    pub cell_4_millivolts: u16,
}

/// Voltage and current readings from the USB-C port.
#[derive(Debug)]
pub struct UsbCVBus {
    pub millivolts: u16,
    pub milliamps: u16,
    pub milliwatts: u16,
}

/// Monitor a [Waveshare UPS HAT E](https://www.waveshare.com/wiki/UPS_HAT_(E))
/// (Uninterruptible Power Supply model E) for a Raspberry Pi.
///
/// This struct can monitor the UPS HAT status, such as battery voltage, current, power, and
/// other interesting information
pub struct UpsHatE {
    i2c_bus: LinuxI2CDevice,
}

impl Default for UpsHatE {
    /// Create a new instance of the UPS Hat E monitor using the default I2C bus device path and
    /// address. This works in most cases.
    fn default() -> Self {
        let i2c = LinuxI2CDevice::new(DEFAULT_I2C_DEV_PATH, DEFAULT_I2C_ADDRESS)
            .expect("Failed to open I2C device");

        Self { i2c_bus: i2c }
    }
}

impl UpsHatE {
    /// Create a new instance of the UPS Hat E monitor using the default I2C bus device path and
    /// address. This works in most cases.
    pub fn new() -> Self {
        Self::default()
    }

    /// Expert option: create a new instance of the UPS Hat E monitor using a custom I2C bus device
    /// (custom path and address).
    pub fn from_i2c_device(i2c_bus: LinuxI2CDevice) -> Self {
        Self { i2c_bus }
    }

    pub fn get_cell_voltage(&mut self) -> Result<CellVoltage, Error> {
        let data = self.read_block(CELL_VOLTAGE_REG.id, CELL_VOLTAGE_REG.length)?;

        let voltages = CellVoltage {
            cell_1_millivolts: data[0] as u16 | (data[1] as u16) << 8,
            cell_2_millivolts: data[2] as u16 | (data[3] as u16) << 8,
            cell_3_millivolts: data[4] as u16 | (data[5] as u16) << 8,
            cell_4_millivolts: data[6] as u16 | (data[7] as u16) << 8,
        };

        Ok(voltages)
    }

    pub fn get_usbc_vbus(&mut self) -> Result<UsbCVBus, Error> {
        let data = self.read_block(USBC_VBUS_REG.id, USBC_VBUS_REG.length)?;

        let vbus = UsbCVBus {
            millivolts: data[0] as u16 | (data[1] as u16) << 8,
            milliamps: data[2] as u16 | (data[3] as u16) << 8,
            milliwatts: data[4] as u16 | (data[5] as u16) << 8,
        };

        Ok(vbus)
    }

    pub fn get_battery_state(&mut self) -> Result<BatteryState, Error> {
        let data = self.read_block(BATTERY_REG.id, BATTERY_REG.length)?;

        let milliamps: i16 = {
            let mut current = data[2] as i32 | (data[3] as i32) << 8;
            // sign treatment mimics the reference python code
            if current > 0x7fff {
                current -= 0xffff;
            }
            current as i16
        };

        let mut remaining_runtime_minutes: u16 = 0;
        let mut time_to_full_minutes: u16 = 0;

        if milliamps < 0 {
            // negative means discharging the battery
            remaining_runtime_minutes = data[8] as u16 | (data[9] as u16) << 8;
        } else {
            // positive means charging the battery, power is available
            time_to_full_minutes = data[10] as u16 | (data[11] as u16) << 8;
        }

        let state = BatteryState {
            millivolts: data[0] as u16 | (data[1] as u16) << 8,
            milliamps,
            remaining_percent: data[4] as u16 | (data[5] as u16) << 8,
            remaining_capacity_milliamphours: data[6] as u16 | (data[7] as u16) << 8,
            remaining_runtime_minutes,
            time_to_full_minutes,
        };

        Ok(state)
    }

    pub fn get_power_state(&mut self) -> Result<PowerState, Error> {
        let data = self.read_block(CHARGING_REG.id, CHARGING_REG.length)?;
        let byte = data[0];

        let charger_activity = ChargerActivity::try_from(byte & 0b111)?;
        let usbc_input_state = UsbCInputState::from(byte & (1 << 5) != 0);
        let usbc_power_delivery = UsbCPowerDelivery::from(byte & (1 << 6) != 0);
        let charging_state = ChargingState::from(byte & (1 << 7) != 0);

        Ok(PowerState {
            charging_state,
            charger_activity,
            usbc_input_state,
            usbc_power_delivery,
        })
    }

    fn read_block(&mut self, register: u8, length: u8) -> Result<Vec<u8>, Error> {
        let data = self.i2c_bus.smbus_read_i2c_block_data(register, length)?;

        if data.len() != length as usize {
            return Err(Error::InvalidDataLen(register, length as usize, data.len()));
        }

        Ok(data)
    }
}
