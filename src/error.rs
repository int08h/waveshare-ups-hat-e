// Copyright (c) 2025 Stuart Stock
// SPDX-License-Identifier: MIT OR Apache-2.0

use i2cdev::linux::LinuxI2CError;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    I2CError(LinuxI2CError),
    InvalidDataLen(u8, usize, usize),
    InvalidChargerActivity(u8),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::I2CError(e) => write!(f, "I2C error: {e}"),
            Error::InvalidDataLen(reg, expected, got) => {
                write!(
                    f,
                    "Reading register {reg} invalid data length: expected {expected}, got {got}"
                )
            }
            Error::InvalidChargerActivity(val) => {
                write!(f, "Invalid charger activity value: {val}")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::I2CError(e) => Some(e),
            Error::InvalidDataLen(..) | Error::InvalidChargerActivity(_) => None,
        }
    }
}

impl From<LinuxI2CError> for Error {
    fn from(err: LinuxI2CError) -> Self {
        Error::I2CError(err)
    }
}
