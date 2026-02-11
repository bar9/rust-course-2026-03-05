use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct WeatherReport {
    station_id: String,
    location: Location,
    current: CurrentConditions,
    #[serde(skip_serializing_if = "Option::is_none")]
    alert: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Location {
    city: String,
    country: String,
    #[serde(rename = "lat")]
    latitude: f64,
    #[serde(rename = "lon")]
    longitude: f64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct CurrentConditions {
    temperature_celsius: f64,
    humidity_percent: u8,
    #[serde(default)]
    wind_speed_kmh: f64,
    condition: WeatherCondition,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum WeatherCondition {
    Sunny,
    Cloudy,
    Rainy,
    Snowy,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_report() -> WeatherReport {
        WeatherReport {
            station_id: "ZRH-01".to_string(),
            location: Location {
                city: "Zurich".to_string(),
                country: "CH".to_string(),
                latitude: 47.3769,
                longitude: 8.5417,
            },
            current: CurrentConditions {
                temperature_celsius: 18.5,
                humidity_percent: 65,
                wind_speed_kmh: 12.0,
                condition: WeatherCondition::Sunny,
            },
            alert: None,
        }
    }

    #[test]
    fn serialize_and_deserialize_round_trip() {
        let report = sample_report();
        let json = serde_json::to_string(&report).unwrap();
        let parsed: WeatherReport = serde_json::from_str(&json).unwrap();
        assert_eq!(report, parsed);
    }

    #[test]
    fn camel_case_field_names() {
        let report = sample_report();
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("stationId"));
        assert!(json.contains("temperatureCelsius"));
        assert!(json.contains("humidityPercent"));
        assert!(!json.contains("station_id"));
    }

    #[test]
    fn location_fields_renamed() {
        let report = sample_report();
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains(r#""lat""#));
        assert!(json.contains(r#""lon""#));
        assert!(!json.contains("latitude"));
    }

    #[test]
    fn none_alert_is_omitted() {
        let report = sample_report();
        let json = serde_json::to_string(&report).unwrap();
        assert!(!json.contains("alert"));
    }

    #[test]
    fn some_alert_is_included() {
        let mut report = sample_report();
        report.alert = Some("Heat warning".to_string());
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("alert"));
        assert!(json.contains("Heat warning"));
    }

    #[test]
    fn enum_serializes_lowercase() {
        let report = sample_report();
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains(r#""sunny""#));
        assert!(!json.contains("Sunny"));
    }

    #[test]
    fn missing_wind_speed_uses_default() {
        let input = r#"{
            "stationId": "ZRH-01",
            "location": {"city":"Zurich","country":"CH","lat":47.3769,"lon":8.5417},
            "current": {
                "temperatureCelsius": 18.5,
                "humidityPercent": 65,
                "condition": "sunny"
            }
        }"#;
        let report: WeatherReport = serde_json::from_str(input).unwrap();
        assert!((report.current.wind_speed_kmh - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn deserialize_from_json_string() {
        let input = r#"{
            "stationId": "BER-03",
            "location": {"city":"Berlin","country":"DE","lat":52.52,"lon":13.405},
            "current": {
                "temperatureCelsius": -2.0,
                "humidityPercent": 80,
                "windSpeedKmh": 25.0,
                "condition": "snowy"
            },
            "alert": "Freezing conditions"
        }"#;
        let report: WeatherReport = serde_json::from_str(input).unwrap();
        assert_eq!(report.station_id, "BER-03");
        assert_eq!(report.location.city, "Berlin");
        assert_eq!(report.current.condition, WeatherCondition::Snowy);
        assert_eq!(report.alert, Some("Freezing conditions".to_string()));
    }
}
