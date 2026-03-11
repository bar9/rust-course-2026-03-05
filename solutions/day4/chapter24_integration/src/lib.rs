#![cfg_attr(not(test), no_std)]

pub mod temperature;
pub mod communication;

pub use temperature::{Temperature, TemperatureBuffer, TemperatureReading};
pub use communication::{Command, Response, TemperatureComm};
