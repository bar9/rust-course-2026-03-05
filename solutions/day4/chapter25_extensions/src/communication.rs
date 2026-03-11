
use heapless::String;
use serde::{Deserialize, Serialize};
use serde_json_core;

use crate::temperature::{TemperatureBuffer, TemperatureReading, TemperatureStats};

/// Commands that can be sent to the temperature monitor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    GetStatus,
    GetLatestReading,
    GetStats,
    SetSampleRate { rate_hz: u8 },
    SetThreshold { threshold_celsius: f32 },
    Reset,
}

/// Responses from the temperature monitor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Status {
        uptime_ms: u32,
        sample_rate_hz: u8,
        threshold_celsius: f32,
        buffer_usage: u8,
    },
    Reading(TemperatureReading),
    Stats(TemperatureStats),
    SampleRateSet(u8),
    ThresholdSet(f32),
    ResetComplete,
    Error { code: u8, message: String<32> },
}

impl Response {
    pub fn error(code: u8, message: &str) -> Self {
        let mut error_message = String::new();
        error_message.push_str(message).ok();
        Self::Error {
            code,
            message: error_message,
        }
    }
}

/// Communication handler for temperature monitor
pub struct TemperatureComm {
    sample_rate_hz: u8,
    threshold_celsius: f32,
    start_time_ms: u32,
}

impl TemperatureComm {
    pub const fn new() -> Self {
        Self {
            sample_rate_hz: 1,
            threshold_celsius: 35.0,
            start_time_ms: 0,
        }
    }

    pub fn init(&mut self, start_time_ms: u32) {
        self.start_time_ms = start_time_ms;
    }

    /// Process a command and return appropriate response
    pub fn process_command<const N: usize>(
        &mut self,
        command: Command,
        buffer: &TemperatureBuffer<N>,
        current_time_ms: u32
    ) -> Response {
        match command {
            Command::GetStatus => {
                let uptime = current_time_ms.saturating_sub(self.start_time_ms);
                let buffer_usage = if buffer.capacity() > 0 {
                    ((buffer.len() * 100) / buffer.capacity()) as u8
                } else {
                    0
                };

                Response::Status {
                    uptime_ms: uptime,
                    sample_rate_hz: self.sample_rate_hz,
                    threshold_celsius: self.threshold_celsius,
                    buffer_usage,
                }
            }

            Command::GetLatestReading => {
                if let Some(temp) = buffer.latest() {
                    let reading = TemperatureReading::new(temp, current_time_ms, 0);
                    Response::Reading(reading)
                } else {
                    Response::error(1, "No readings available")
                }
            }

            Command::GetStats => {
                if let Some(stats) = TemperatureStats::from_buffer(buffer, current_time_ms) {
                    Response::Stats(stats)
                } else {
                    Response::error(2, "No data for statistics")
                }
            }

            Command::SetSampleRate { rate_hz } => {
                if rate_hz > 0 && rate_hz <= 10 {
                    self.sample_rate_hz = rate_hz;
                    Response::SampleRateSet(rate_hz)
                } else {
                    Response::error(3, "Rate must be 1-10 Hz")
                }
            }

            Command::SetThreshold { threshold_celsius } => {
                if threshold_celsius > 0.0 && threshold_celsius < 100.0 {
                    self.threshold_celsius = threshold_celsius;
                    Response::ThresholdSet(threshold_celsius)
                } else {
                    Response::error(4, "Threshold must be 0-100°C")
                }
            }

            Command::Reset => {
                self.start_time_ms = current_time_ms;
                self.sample_rate_hz = 1;
                self.threshold_celsius = 35.0;
                Response::ResetComplete
            }
        }
    }

    /// Serialize response to JSON string for transmission
    pub fn response_to_json(&self, response: &Response) -> Result<String<256>, ()> {
        match serde_json_core::to_string::<_, 256>(response) {
            Ok(json) => Ok(json),
            Err(_) => Err(()),
        }
    }

    /// Deserialize command from JSON string
    pub fn json_to_command(&self, json: &str) -> Result<Command, ()> {
        match serde_json_core::from_str(json) {
            Ok((command, _)) => Ok(command),
            Err(_) => Err(()),
        }
    }

    /// Create a status response as JSON (read-only)
    pub fn status_json<const N: usize>(
        &self,
        buffer: &TemperatureBuffer<N>,
        current_time_ms: u32
    ) -> String<256> {
        let uptime = current_time_ms.saturating_sub(self.start_time_ms);
        let buffer_usage = if buffer.capacity() > 0 {
            ((buffer.len() * 100) / buffer.capacity()) as u8
        } else {
            0
        };

        let status = Response::Status {
            uptime_ms: uptime,
            sample_rate_hz: self.sample_rate_hz,
            threshold_celsius: self.threshold_celsius,
            buffer_usage,
        };

        self.response_to_json(&status)
            .unwrap_or_else(|_| {
                let mut error = String::new();
                error.push_str("{\"error\":\"serialization_failed\"}").ok();
                error
            })
    }

    /// Create latest reading as JSON (read-only)
    pub fn reading_json<const N: usize>(
        &self,
        buffer: &TemperatureBuffer<N>,
        current_time_ms: u32
    ) -> String<256> {
        let reading = if let Some(temp) = buffer.latest() {
            let reading = TemperatureReading::new(temp, current_time_ms, 0);
            Response::Reading(reading)
        } else {
            Response::error(1, "No readings available")
        };

        self.response_to_json(&reading)
            .unwrap_or_else(|_| {
                let mut error = String::new();
                error.push_str("{\"error\":\"no_reading\"}").ok();
                error
            })
    }

    pub fn sample_rate(&self) -> u8 {
        self.sample_rate_hz
    }

    pub fn threshold(&self) -> f32 {
        self.threshold_celsius
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::temperature::TemperatureBuffer;

    #[test]
    fn test_json_serialization() {
        let temp = crate::temperature::Temperature::from_celsius(23.5);
        let reading = TemperatureReading::new(temp, 1000, 0);

        // Test command serialization
        let command = Command::GetStatus;
        let json = serde_json_core::to_string::<_, 64>(&command).unwrap();
        assert_eq!(json, "\"GetStatus\"");

        // Test response serialization
        let response = Response::Reading(reading);
        let json = serde_json_core::to_string::<_, 256>(&response).unwrap();
        assert!(json.contains("Reading"));
    }

    #[test]
    fn test_command_processing() {
        let mut comm = TemperatureComm::new();
        comm.init(0);
        let buffer = TemperatureBuffer::<5>::new();

        // Test status command
        let status_resp = comm.process_command(Command::GetStatus, &buffer, 5000);
        if let Response::Status { uptime_ms, .. } = status_resp {
            assert_eq!(uptime_ms, 5000);
        } else {
            panic!("Expected status response");
        }

        // Test rate setting
        let rate_resp = comm.process_command(
            Command::SetSampleRate { rate_hz: 5 },
            &buffer,
            5000
        );
        assert!(matches!(rate_resp, Response::SampleRateSet(5)));
        assert_eq!(comm.sample_rate(), 5);
    }

    #[test]
    fn test_json_roundtrip() {
        let comm = TemperatureComm::new();

        // Test command deserialization
        let json_cmd = "\"GetStatus\"";
        let command = comm.json_to_command(json_cmd).unwrap();
        assert!(matches!(command, Command::GetStatus));

        // Test response serialization
        let response = Response::ResetComplete;
        let json_resp = comm.response_to_json(&response).unwrap();
        assert_eq!(json_resp, "\"ResetComplete\"");
    }

    #[test]
    fn test_error_handling() {
        let mut comm = TemperatureComm::new();
        let buffer = TemperatureBuffer::<5>::new();

        // Test invalid sample rate
        let response = comm.process_command(
            Command::SetSampleRate { rate_hz: 20 },  // Invalid: too high
            &buffer,
            0
        );

        if let Response::Error { code, message } = response {
            assert_eq!(code, 3);
            assert!(message.contains("Rate must be"));
        } else {
            panic!("Expected error response");
        }
    }
}