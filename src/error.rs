// Copyright (c) 2025 Stuart Stock
// SPDX-License-Identifier: MIT OR Apache-2.0

use i2cdev::linux::LinuxI2CError;
use std::fmt;

/// Errors returned when communicating with the UPS Hat E.
#[derive(Debug)]
pub enum Error {
    /// An error from the underlying I2C device.
    I2CError(LinuxI2CError),
    /// A block read from `register` returned a different number of bytes than expected.
    InvalidDataLen {
        register: u8,
        expected: usize,
        actual: usize,
    },
    /// The 3-bit charger activity field held a value with no defined meaning.
    InvalidChargerActivity(u8),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::I2CError(e) => write!(f, "I2C error: {e}"),
            Error::InvalidDataLen {
                register,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Reading register {register:#04x} invalid data length: expected {expected}, got {actual}"
                )
            }
            Error::InvalidChargerActivity(val) => {
                write!(f, "Invalid charger activity value: {val:#05b}")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::I2CError(e) => Some(e),
            Error::InvalidDataLen { .. } | Error::InvalidChargerActivity(_) => None,
        }
    }
}

impl From<LinuxI2CError> for Error {
    fn from(err: LinuxI2CError) -> Self {
        Error::I2CError(err)
    }
}
