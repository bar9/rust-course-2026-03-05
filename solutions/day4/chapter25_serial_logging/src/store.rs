use serde::Serialize;

pub struct TemperatureStore {
    raw_celsius: Option<f32>,
    offset: f32,
}

/// A snapshot of the current readings, serializable to JSON.
#[derive(Serialize)]
pub struct Reading {
    pub raw_celsius: f32,
    pub ambient_celsius: f32,
    pub ambient_fahrenheit: f32,
    pub offset: f32,
}

impl TemperatureStore {
    pub const fn new(offset: f32) -> Self {
        Self {
            raw_celsius: None,
            offset,
        }
    }

    pub fn update(&mut self, raw_celsius: f32) {
        self.raw_celsius = Some(raw_celsius);
    }

    pub fn raw_celsius(&self) -> Option<f32> {
        self.raw_celsius
    }

    pub fn ambient_celsius(&self) -> Option<f32> {
        self.raw_celsius.map(|raw| raw - self.offset)
    }

    pub fn ambient_fahrenheit(&self) -> Option<f32> {
        self.ambient_celsius().map(|c| c * 9.0 / 5.0 + 32.0)
    }

    pub fn offset(&self) -> f32 {
        self.offset
    }

    pub fn set_offset(&mut self, offset: f32) {
        self.offset = offset;
    }

    /// Returns a serializable snapshot of the current state, if a reading exists.
    pub fn reading(&self) -> Option<Reading> {
        let raw = self.raw_celsius?;
        let amb_c = raw - self.offset;
        Some(Reading {
            raw_celsius: raw,
            ambient_celsius: amb_c,
            ambient_fahrenheit: amb_c * 9.0 / 5.0 + 32.0,
            offset: self.offset,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_before_first_update() {
        let store = TemperatureStore::new(30.0);
        assert_eq!(store.raw_celsius(), None);
        assert_eq!(store.ambient_celsius(), None);
    }

    #[test]
    fn ambient_celsius_subtracts_offset() {
        let mut store = TemperatureStore::new(30.0);
        store.update(52.0);
        let ambient = store.ambient_celsius().unwrap();
        assert!((ambient - 22.0).abs() < f32::EPSILON);
    }

    #[test]
    fn ambient_fahrenheit_converts_correctly() {
        let mut store = TemperatureStore::new(30.0);
        store.update(52.0);
        let fahrenheit = store.ambient_fahrenheit().unwrap();
        assert!((fahrenheit - 71.6).abs() < 0.1);
    }

    #[test]
    fn reading_returns_none_before_update() {
        let store = TemperatureStore::new(30.0);
        assert!(store.reading().is_none());
    }

    #[test]
    fn reading_serializes_to_json() {
        let mut store = TemperatureStore::new(30.0);
        store.update(52.0);
        let reading = store.reading().unwrap();

        let mut buf = [0u8; 128];
        let len = serde_json_core::to_slice(&reading, &mut buf).unwrap();
        let json = core::str::from_utf8(&buf[..len]).unwrap();

        assert!(json.contains("\"raw_celsius\""));
        assert!(json.contains("\"ambient_celsius\""));
        assert!(json.contains("\"offset\""));
    }
}
