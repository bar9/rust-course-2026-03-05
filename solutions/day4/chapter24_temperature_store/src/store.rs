pub struct TemperatureStore {
    raw_celsius: Option<f32>,
    offset: f32,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_before_first_update() {
        let store = TemperatureStore::new(30.0);
        assert_eq!(store.raw_celsius(), None);
        assert_eq!(store.ambient_celsius(), None);
        assert_eq!(store.ambient_fahrenheit(), None);
    }

    #[test]
    fn update_stores_raw_value() {
        let mut store = TemperatureStore::new(30.0);
        store.update(52.0);
        assert_eq!(store.raw_celsius(), Some(52.0));
    }

    #[test]
    fn ambient_celsius_subtracts_offset() {
        let mut store = TemperatureStore::new(30.0);
        store.update(52.0);
        // 52.0 - 30.0 = 22.0
        let ambient = store.ambient_celsius().unwrap();
        assert!((ambient - 22.0).abs() < f32::EPSILON);
    }

    #[test]
    fn ambient_fahrenheit_converts_correctly() {
        let mut store = TemperatureStore::new(30.0);
        store.update(52.0);
        // ambient = 22.0C -> 22 * 9/5 + 32 = 71.6F
        let fahrenheit = store.ambient_fahrenheit().unwrap();
        assert!((fahrenheit - 71.6).abs() < 0.1);
    }

    #[test]
    fn negative_ambient_temperature() {
        let mut store = TemperatureStore::new(30.0);
        store.update(20.0);
        // 20.0 - 30.0 = -10.0C
        let ambient = store.ambient_celsius().unwrap();
        assert!((ambient - (-10.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn update_overwrites_previous() {
        let mut store = TemperatureStore::new(30.0);
        store.update(50.0);
        store.update(55.0);
        assert_eq!(store.raw_celsius(), Some(55.0));
    }
}
