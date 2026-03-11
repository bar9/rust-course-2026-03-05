# Chapter 17: Serde & Serialization

Serde is Rust's de facto serialization framework. The name is a portmanteau of **ser**ialize and **de**serialize. Nearly every Rust project that reads or writes structured data -- JSON APIs, configuration files, binary protocols, CSV exports -- uses serde. This chapter introduces serde's derive model, the `serde_json` crate, and the attributes you will use extensively in the Day 4 project (Chapters 22-23).

## 1. Why Serde?

Three properties make serde the standard choice:

1. **Format-agnostic.** You derive `Serialize` and `Deserialize` once on your types. The same derives work with JSON, TOML, YAML, MessagePack, bincode, CSV, and dozens of other formats -- you just swap the format crate.

2. **Zero-cost abstraction.** Serde uses Rust's trait system and monomorphization to generate specialized code at compile time. There is no runtime reflection and no boxing overhead in the common path.

3. **Zero-copy deserialization.** For formats that support it, serde can deserialize borrowed data (`&str`, `&[u8]`) without allocating, which matters in performance-sensitive applications.

### Brief comparison with C#

C# developers typically reach for `System.Text.Json` (built-in since .NET Core 3.0) or `Newtonsoft.Json`. Serde fills the same role but is format-agnostic by design -- `System.Text.Json` is JSON-only, whereas serde's traits work across all supported formats. The derive-macro approach is similar to C#'s JSON source generators introduced in .NET 6.

## 2. The Derive Model

Serde provides two traits: `Serialize` (for turning Rust values into a data format) and `Deserialize` (for parsing a data format back into Rust values). You almost never implement these by hand. Instead, you use derive macros -- the same mechanism covered in [Chapter 15: Macros & Code Generation](./15_macros.md).

### Cargo.toml setup

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

The `"derive"` feature flag enables the `#[derive(Serialize, Deserialize)]` proc macros. Without it, you would need to implement the traits manually.

### Basic example

```rust,ignore
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct Sensor {
    id: u32,
    label: String,
    temperature: f64,
    active: bool,
}

fn main() {
    let sensor = Sensor {
        id: 42,
        label: "main-hall".to_string(),
        temperature: 21.5,
        active: true,
    };

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&sensor).unwrap();
    println!("{json}");

    // Deserialize back
    let parsed: Sensor = serde_json::from_str(&json).unwrap();
    println!("{parsed:?}");
}
```

Output:

```text
{
  "id": 42,
  "label": "main-hall",
  "temperature": 21.5,
  "active": true
}
Sensor { id: 42, label: "main-hall", temperature: 21.5, active: true }
```

### Debug vs JSON: different things

Notice that `Debug` output (`{:?}`) and JSON output are distinct representations. `Debug` is for developer diagnostics and uses Rust syntax. JSON is a data interchange format with its own rules. Deriving both is standard practice, but they serve different purposes.

## 3. Working with serde_json

`serde_json` is the JSON format implementation for serde. It provides the functions you will use most often.

### Serialization

```rust,ignore
use serde::Serialize;

#[derive(Serialize)]
struct Point {
    x: f64,
    y: f64,
}

fn main() -> Result<(), serde_json::Error> {
    let p = Point { x: 1.0, y: 2.5 };

    // Compact output: {"x":1.0,"y":2.5}
    let compact = serde_json::to_string(&p)?;
    println!("{compact}");

    // Pretty-printed output
    let pretty = serde_json::to_string_pretty(&p)?;
    println!("{pretty}");

    Ok(())
}
```

### Deserialization

The target type must be specified so serde knows what to parse into. You can use a type annotation or the turbofish syntax:

```rust,ignore
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Point {
    x: f64,
    y: f64,
}

fn main() -> Result<(), serde_json::Error> {
    let input = r#"{"x": 3.0, "y": 4.0}"#;

    // Type annotation
    let p1: Point = serde_json::from_str(input)?;
    println!("{p1:?}");

    // Turbofish syntax
    let p2 = serde_json::from_str::<Point>(input)?;
    println!("{p2:?}");

    Ok(())
}
```

### Untyped JSON with `serde_json::Value`

When you do not know the schema ahead of time -- or want to inspect JSON without defining a full struct -- use `serde_json::Value`:

```rust,ignore
use serde_json::Value;

fn main() -> Result<(), serde_json::Error> {
    let input = r#"{"name": "Alice", "scores": [95, 87, 92]}"#;

    let v: Value = serde_json::from_str(input)?;

    // Index into the value with [] or .get()
    println!("Name: {}", v["name"]);
    println!("First score: {}", v["scores"][0]);

    // .get() returns Option<&Value> -- safer than indexing
    if let Some(name) = v.get("name").and_then(Value::as_str) {
        println!("Name as &str: {name}");
    }

    Ok(())
}
```

`Value` is an enum with variants `Null`, `Bool(bool)`, `Number(Number)`, `String(String)`, `Array(Vec<Value>)`, and `Object(Map<String, Value>)`. It is similar to `JsonNode` (.NET 6+) or `JToken`/`JObject` (Newtonsoft.Json) in C#.

You can also build JSON values programmatically with the `json!` macro:

```rust,ignore
use serde_json::json;

let response = json!({
    "status": "ok",
    "count": 3,
    "items": ["a", "b", "c"]
});

println!("{}", serde_json::to_string_pretty(&response).unwrap());
```

This macro is used in the Day 4 Axum handlers (Chapter 22) for constructing ad-hoc JSON responses.

### Error handling

`serde_json::from_str` returns `Result<T, serde_json::Error>`. The error type provides location information (line and column) for parse failures:

```rust,ignore
use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    port: u16,
}

fn main() {
    let bad_input = r#"{"port": "not_a_number"}"#;
    match serde_json::from_str::<Config>(bad_input) {
        Ok(config) => println!("Port: {}", config.port),
        Err(e) => eprintln!("Parse error: {e}"),
        // Output: Parse error: invalid type: string "not_a_number",
        //         expected u16 at line 1 column 23
    }
}
```

## 4. Serde Attributes

Serde attributes let you control how fields and types map to the data format without writing custom serialization logic. These are the attributes you will encounter in the Day 4 project.

### `#[serde(rename_all = "...")]`

Applies a naming convention to all fields or variants:

```rust,ignore
use serde::{Serialize, Deserialize};

// camelCase -- common for JSON APIs
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserProfile {
    first_name: String,      // -> "firstName"
    last_name: String,       // -> "lastName"
    email_address: String,   // -> "emailAddress"
}

// lowercase -- used on enums in Day 4 (Chapter 23)
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum JobState {
    Pending,     // -> "pending"
    Processing,  // -> "processing"
    Complete,    // -> "complete"
    Failed,      // -> "failed"
}
```

Other supported conventions: `"UPPERCASE"`, `"PascalCase"`, `"SCREAMING_SNAKE_CASE"`, `"kebab-case"`, `"SCREAMING-KEBAB-CASE"`.

### `#[serde(skip_serializing_if = "...")]`

Omits a field from the output when a condition is true. Most commonly used with `Option` fields to skip `null` values:

```rust,ignore
use serde::Serialize;

#[derive(Serialize)]
struct JobStatus {
    id: String,
    state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

// When error is None, the JSON output omits the field entirely:
// {"id":"abc","state":"complete"}
//
// When error is Some, it appears:
// {"id":"abc","state":"failed","error":"disk full"}
```

This is used on `JobStatus` in Day 4 (Chapter 22) so that successful jobs produce cleaner JSON responses.

### `#[serde(default)]`

Uses `Default::default()` for fields missing during deserialization:

```rust,ignore
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ServerConfig {
    host: String,
    #[serde(default)]
    port: u16,           // defaults to 0 if missing
    #[serde(default)]
    debug: bool,         // defaults to false if missing
    #[serde(default)]
    tags: Vec<String>,   // defaults to empty Vec if missing
}

fn main() {
    let input = r#"{"host": "localhost"}"#;
    let config: ServerConfig = serde_json::from_str(input).unwrap();
    println!("{config:?}");
    // ServerConfig { host: "localhost", port: 0, debug: false, tags: [] }
}
```

You can also provide a custom default function: `#[serde(default = "default_port")]` where `fn default_port() -> u16 { 8080 }`.

### `#[serde(rename = "...")]`

Renames an individual field. Useful when the JSON key is a Rust keyword or follows a different convention:

```rust,ignore
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct ApiResponse {
    #[serde(rename = "type")]
    kind: String,              // JSON key is "type", which is a Rust keyword

    #[serde(rename = "error_code")]
    code: u32,                 // JSON key differs from Rust field name
}
```

### `#[serde(flatten)]`

Inlines the fields of a nested struct into the parent:

```rust,ignore
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Pagination {
    page: u32,
    per_page: u32,
}

#[derive(Serialize, Deserialize)]
struct UserList {
    users: Vec<String>,
    #[serde(flatten)]
    pagination: Pagination,
}

// Without flatten: {"users":["Alice"],"pagination":{"page":1,"per_page":10}}
// With flatten:    {"users":["Alice"],"page":1,"per_page":10}
```

### Attribute summary

| Attribute | Level | Effect |
|-----------|-------|--------|
| `#[serde(rename_all = "camelCase")]` | Struct / Enum | Rename all fields or variants |
| `#[serde(rename = "x")]` | Field / Variant | Rename a single field or variant |
| `#[serde(skip_serializing_if = "...")]` | Field | Omit field when condition is true |
| `#[serde(default)]` | Field / Struct | Use `Default` for missing fields |
| `#[serde(flatten)]` | Field | Inline nested struct fields |
| `#[serde(skip)]` | Field | Never serialize or deserialize |
| `#[serde(alias = "x")]` | Field | Accept an alternative name during deserialization |

The full list is available at [serde.rs/attributes.html](https://serde.rs/attributes.html).

## 5. Enums in Serde

Rust enums are more expressive than C# enums -- they can carry data in each variant. Serde supports four representations for enums with data, controlled by container attributes.

### Externally tagged (default)

Each variant wraps its data under the variant name as a key:

```rust,ignore
use serde::Serialize;

#[derive(Serialize)]
enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
}

// Shape::Circle { radius: 5.0 } serializes to:
// {"Circle":{"radius":5.0}}
```

### Internally tagged

The tag is a field inside the object. This is the most common pattern for API discriminated unions:

```rust,ignore
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum Event {
    Login { user: String },
    Logout { user: String, reason: String },
    Heartbeat,
}

// Event::Login { user: "alice".into() } serializes to:
// {"type":"Login","user":"alice"}
//
// Event::Heartbeat serializes to:
// {"type":"Heartbeat"}
```

This is similar to C#'s `[JsonDerivedType]` with a type discriminator, available since .NET 7.

### Adjacently tagged

The tag and content are sibling fields:

```rust,ignore
use serde::Serialize;

#[derive(Serialize)]
#[serde(tag = "t", content = "c")]
enum Message {
    Text(String),
    Image { url: String, alt: String },
}

// Message::Text("hello".into()) serializes to:
// {"t":"Text","c":"hello"}
```

### Untagged

No discriminator -- serde tries each variant in order until one matches:

```rust,ignore
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum StringOrNumber {
    Num(f64),
    Str(String),
}

fn main() {
    let a: StringOrNumber = serde_json::from_str("42").unwrap();
    let b: StringOrNumber = serde_json::from_str(r#""hello""#).unwrap();
    println!("{a:?}, {b:?}");
    // Num(42.0), Str("hello")
}
```

Untagged enums are convenient but produce poor error messages on failure, because serde cannot tell you which variant was "closest" to matching. Prefer tagged representations when possible.

### Enum representation summary

| Attribute | JSON shape | Best for |
|-----------|-----------|----------|
| *(none -- default)* | `{"Variant":{...}}` | Rust-to-Rust communication |
| `#[serde(tag = "type")]` | `{"type":"Variant",...}` | APIs with discriminator field |
| `#[serde(tag = "t", content = "c")]` | `{"t":"Variant","c":{...}}` | APIs separating tag and payload |
| `#[serde(untagged)]` | `{...}` | Flexible input parsing |

### Practical example: API event stream

```rust,ignore
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
enum WebhookEvent {
    OrderPlaced {
        order_id: u64,
        total_cents: u64,
    },
    OrderShipped {
        order_id: u64,
        tracking_number: String,
    },
    OrderCancelled {
        order_id: u64,
        reason: String,
    },
}

fn main() {
    let events_json = r#"[
        {"event":"order_placed","order_id":1001,"total_cents":4999},
        {"event":"order_shipped","order_id":1001,"tracking_number":"1Z999AA10123456784"},
        {"event":"order_cancelled","order_id":1002,"reason":"customer request"}
    ]"#;

    let events: Vec<WebhookEvent> = serde_json::from_str(events_json).unwrap();
    for event in &events {
        println!("{event:?}");
    }
}
```

## 6. Other Formats

Because `Serialize` and `Deserialize` are format-agnostic traits, the same struct works with any format crate. You only change the serialization call.

### Switching from JSON to TOML

```rust,ignore
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct DatabaseConfig {
    host: String,
    port: u16,
    name: String,
}

fn main() {
    let config = DatabaseConfig {
        host: "localhost".to_string(),
        port: 5432,
        name: "myapp".to_string(),
    };

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&config).unwrap();
    println!("JSON:\n{json}\n");

    // Serialize to TOML -- same struct, different format crate
    let toml = toml::to_string_pretty(&config).unwrap();
    println!("TOML:\n{toml}");
}
```

```text
JSON:
{
  "host": "localhost",
  "port": 5432,
  "name": "myapp"
}

TOML:
host = "localhost"
port = 5432
name = "myapp"
```

### Common format crates

| Crate | Format | Typical use case |
|-------|--------|-----------------|
| `serde_json` | JSON | Web APIs, configuration |
| `toml` | TOML | Configuration files (Cargo.toml itself is TOML) |
| `serde_yml` | YAML | Kubernetes manifests, CI configs (replaces deprecated `serde_yaml`) |
| `bincode` | Binary | Compact binary serialization, IPC |
| `csv` | CSV | Tabular data import/export |
| `rmp-serde` | MessagePack | Efficient binary alternative to JSON |

All of these crates expose `to_string` / `from_str` (or `to_vec` / `from_slice` for binary formats) with the same pattern. Once you know serde, switching formats is a one-line change.

## 7. Comparison with C#

| C# / .NET | Rust (serde) | Notes |
|-----------|-------------|-------|
| `[JsonPropertyName("x")]` | `#[serde(rename = "x")]` | Field renaming |
| `JsonNamingPolicy.CamelCase` | `#[serde(rename_all = "camelCase")]` | Naming convention |
| `[JsonIgnore(Condition = WhenWritingNull)]` | `#[serde(skip_serializing_if = "Option::is_none")]` | Skip null fields |
| `JsonSerializer.Serialize(obj)` | `serde_json::to_string(&obj)?` | Serialize to string |
| `JsonSerializer.Deserialize<T>(str)` | `serde_json::from_str::<T>(str)?` | Deserialize from string |
| `JsonDocument` / `JObject` | `serde_json::Value` | Untyped JSON access |
| `[JsonDerivedType]` discriminator | `#[serde(tag = "type")]` | Tagged unions |
| Source generators (.NET 6+) | Derive macros | Compile-time code generation |
| `JsonSerializerOptions` (global) | Per-type attributes | Configuration scope |

Key difference: in C#, serialization settings are typically configured globally via `JsonSerializerOptions`. In serde, attributes are placed directly on each type, making the serialization contract explicit and local. This means you can look at a struct definition and immediately understand its JSON representation without checking a distant configuration object.

## 8. Exercise

Define a `WeatherReport` struct with nested data, derive `Serialize` and `Deserialize`, apply serde attributes, and round-trip through JSON.

### Starter code

```rust,ignore
use serde::{Serialize, Deserialize};

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
```

## Summary

| Concept | Key takeaway |
|---------|-------------|
| `Serialize` / `Deserialize` | Derive macros that generate format-agnostic serialization code |
| `serde_json::to_string` | Serialize a value to a JSON string |
| `serde_json::from_str` | Deserialize a JSON string into a typed value |
| `serde_json::Value` | Untyped JSON for dynamic or unknown schemas |
| `#[serde(rename_all = "...")]` | Apply a naming convention to all fields or variants |
| `#[serde(rename = "...")]` | Rename a single field or variant |
| `#[serde(skip_serializing_if = "...")]` | Conditionally omit a field |
| `#[serde(default)]` | Provide a default for missing fields during deserialization |
| `#[serde(flatten)]` | Inline nested struct fields |
| `#[serde(tag = "...")]` | Internally tagged enum representation |
| `#[serde(untagged)]` | Try each variant in order, no discriminator |
| Format-agnostic | Same derives work with JSON, TOML, YAML, bincode, etc. |

This chapter prepares you for Day 4, where serde is used in the ESP32-C3 project to serialize temperature readings and commands as JSON over USB serial ([Chapter 23](../day4/23_data_communication.md)).
