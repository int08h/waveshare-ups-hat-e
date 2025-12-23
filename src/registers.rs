// Copyright (c) 2025 Stuart Stock
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::error::Error;

/// A logical grouping of registers for similar/related functionality
pub(crate) struct RegisterBlock {
    /// Register ID
    pub(crate) id: u8,
    /// Number of bytes to read from the register
    pub(crate) length: u8,
}

/// Plugged in, power delivery detected, charger activity
pub(crate) const CHARGING_REG: RegisterBlock = RegisterBlock {
    id: 0x02,
    length: 1,
};

/// Status of the BQ4050 and IP2368
pub(crate) const COMMUNICATION_REG: RegisterBlock = RegisterBlock {
    id: 0x03,
    length: 1,
};

/// USB-C Power voltage, current, power
pub(crate) const USBC_VBUS_REG: RegisterBlock = RegisterBlock {
    id: 0x10,
    length: 6,
};

/// Battery voltage, current, remaining runtime, time-to-full
pub(crate) const BATTERY_REG: RegisterBlock = RegisterBlock {
    id: 0x20,
    length: 12,
};

/// Cell voltages for all four batteries
pub(crate) const CELL_VOLTAGE_REG: RegisterBlock = RegisterBlock {
    id: 0x30,
    length: 8,
};

pub (crate) const POWEROFF_REG: RegisterBlock = RegisterBlock {
    id: 0x01,
    length: 1,
};

/// What kind of charging (if any) is taking place?
#[derive(Debug)]
pub enum ChargerActivity {
    Standby = 0b000,
    Trickle = 0b001,
    ConstantCurrent = 0b010,
    ConstantVoltage = 0b011,
    Pending = 0b100,
    Full = 0b101,
    Timeout = 0b110,
}

impl TryFrom<u8> for ChargerActivity {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0b000 => Ok(ChargerActivity::Standby),
            0b001 => Ok(ChargerActivity::Trickle),
            0b010 => Ok(ChargerActivity::ConstantCurrent),
            0b011 => Ok(ChargerActivity::ConstantVoltage),
            0b100 => Ok(ChargerActivity::Pending),
            0b101 => Ok(ChargerActivity::Full),
            0b110 => Ok(ChargerActivity::Timeout),
            _ => Err(Error::InvalidChargerActivity(value)),
        }
    }
}

/// State of the UPS microcontroller's communications with an on-board chip.
#[derive(Debug)]
pub enum CommState {
    Error = 0b0,
    Normal = 0b1,
}

impl From<bool> for CommState {
    fn from(value: bool) -> Self {
        if value  {
            CommState::Normal
        } else {
            CommState::Error
        }
    }
}

/// Is USB-C power detected?
#[derive(Debug)]
pub enum UsbCInputState {
    NoPower = 0b0,
    Powered = 0b1,
}

impl From<bool> for UsbCInputState {
    fn from(value: bool) -> Self {
        if value {
            UsbCInputState::Powered
        } else {
            UsbCInputState::NoPower
        }
    }
}

/// Was USB-C power delivery negotiated (`FastCharging`) or not (`StandardCharging`)?
#[derive(Debug)]
pub enum UsbCPowerDelivery {
    StandardCharging = 0b0,
    FastCharging = 0b1,
}

impl From<bool> for UsbCPowerDelivery {
    fn from(value: bool) -> Self {
        if value {
            UsbCPowerDelivery::FastCharging
        } else {
            UsbCPowerDelivery::StandardCharging
        }
    }
}

/// Is the UPS charging or not?
#[derive(Debug)]
pub enum ChargingState {
    NotCharging = 0b0,
    Charging = 0b1,
}

impl From<bool> for ChargingState {
    fn from(value: bool) -> Self {
        if value {
            ChargingState::Charging
        } else {
            ChargingState::NotCharging
        }
    }
}
