// Chapter 9: Pattern Matching Exercise Solution

// =============================================================================
// Exercise: HTTP Status Handler
// =============================================================================

#[derive(Debug, PartialEq)]
pub enum HttpStatus {
    Ok,                    // 200
    NotFound,             // 404
    ServerError,          // 500
    Custom(u16),          // Any other code
}

#[derive(Debug)]
pub struct HttpResponse {
    pub status: HttpStatus,
    pub body: Option<String>,
    pub headers: Vec<(String, String)>,
}

impl HttpResponse {
    pub fn new(status: HttpStatus) -> Self {
        HttpResponse {
            status,
            body: None,
            headers: Vec::new(),
        }
    }

    pub fn with_body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }

    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.push((key, value));
        self
    }
}

/// Handle HTTP responses with pattern matching
/// Returns appropriate messages based on status and content
pub fn handle_response(response: HttpResponse) -> String {
    match response {
        // Ok with body content
        HttpResponse { status: HttpStatus::Ok, body: Some(body), .. } => {
            format!("Success: {}", body)
        },

        // Ok without body content
        HttpResponse { status: HttpStatus::Ok, body: None, .. } => {
            "Success: No content".to_string()
        },

        // NotFound error
        HttpResponse { status: HttpStatus::NotFound, .. } => {
            "Error: Resource not found".to_string()
        },

        // ServerError
        HttpResponse { status: HttpStatus::ServerError, .. } => {
            "Error: Internal server error".to_string()
        },

        // Custom status codes with guards
        HttpResponse { status: HttpStatus::Custom(code), .. } if code < 400 => {
            format!("Info: Status {}", code)
        },

        HttpResponse { status: HttpStatus::Custom(code), .. } if code >= 400 => {
            format!("Error: Status {}", code)
        },

        // This case should never be reached due to guard coverage, but Rust requires it
        HttpResponse { status: HttpStatus::Custom(code), .. } => {
            format!("Status {}", code)
        }
    }
}

/// Alternative implementation showing different pattern matching approaches
pub fn handle_response_alternative(response: HttpResponse) -> String {
    // First match on status, then handle body separately
    match response.status {
        HttpStatus::Ok => {
            match response.body {
                Some(body) => format!("Success: {}", body),
                None => "Success: No content".to_string(),
            }
        },
        HttpStatus::NotFound => "Error: Resource not found".to_string(),
        HttpStatus::ServerError => "Error: Internal server error".to_string(),
        HttpStatus::Custom(code) => {
            if code < 400 {
                format!("Info: Status {}", code)
            } else {
                format!("Error: Status {}", code)
            }
        }
    }
}

/// Extract status code as number for logging or metrics
pub fn extract_status_code(response: &HttpResponse) -> u16 {
    match &response.status {
        HttpStatus::Ok => 200,
        HttpStatus::NotFound => 404,
        HttpStatus::ServerError => 500,
        HttpStatus::Custom(code) => *code,
    }
}

/// Check if response indicates success (2xx range)
pub fn is_success(response: &HttpResponse) -> bool {
    match &response.status {
        HttpStatus::Ok => true,
        HttpStatus::Custom(code) if (200..300).contains(code) => true,
        _ => false,
    }
}

/// Extract content type from headers using pattern matching
pub fn extract_content_type(response: &HttpResponse) -> Option<String> {
    // Use pattern matching to find content-type header
    response.headers
        .iter()
        .find_map(|(key, value)| {
            match key.to_lowercase().as_str() {
                "content-type" => Some(value.clone()),
                _ => None,
            }
        })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ok_with_body() {
        let response = HttpResponse::new(HttpStatus::Ok)
            .with_body("Hello World".to_string());

        let result = handle_response(response);
        assert_eq!(result, "Success: Hello World");
    }

    #[test]
    fn test_ok_without_body() {
        let response = HttpResponse::new(HttpStatus::Ok);

        let result = handle_response(response);
        assert_eq!(result, "Success: No content");
    }

    #[test]
    fn test_not_found() {
        let response = HttpResponse::new(HttpStatus::NotFound)
            .with_body("Page not found".to_string());

        let result = handle_response(response);
        assert_eq!(result, "Error: Resource not found");
    }

    #[test]
    fn test_server_error() {
        let response = HttpResponse::new(HttpStatus::ServerError);

        let result = handle_response(response);
        assert_eq!(result, "Error: Internal server error");
    }

    #[test]
    fn test_custom_info_status() {
        let response = HttpResponse::new(HttpStatus::Custom(201));

        let result = handle_response(response);
        assert_eq!(result, "Info: Status 201");
    }

    #[test]
    fn test_custom_error_status() {
        let response = HttpResponse::new(HttpStatus::Custom(403));

        let result = handle_response(response);
        assert_eq!(result, "Error: Status 403");
    }

    #[test]
    fn test_boundary_custom_status() {
        // Test exactly 400
        let response_400 = HttpResponse::new(HttpStatus::Custom(400));
        let result_400 = handle_response(response_400);
        assert_eq!(result_400, "Error: Status 400");

        // Test 399 (just below 400)
        let response_399 = HttpResponse::new(HttpStatus::Custom(399));
        let result_399 = handle_response(response_399);
        assert_eq!(result_399, "Info: Status 399");
    }

    #[test]
    fn test_alternative_implementation() {
        let response = HttpResponse::new(HttpStatus::Ok)
            .with_body("Test content".to_string());

        let result1 = handle_response_alternative(response);

        let response2 = HttpResponse::new(HttpStatus::Ok)
            .with_body("Test content".to_string());
        let result2 = handle_response(response2);

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_extract_status_code() {
        assert_eq!(extract_status_code(&HttpResponse::new(HttpStatus::Ok)), 200);
        assert_eq!(extract_status_code(&HttpResponse::new(HttpStatus::NotFound)), 404);
        assert_eq!(extract_status_code(&HttpResponse::new(HttpStatus::ServerError)), 500);
        assert_eq!(extract_status_code(&HttpResponse::new(HttpStatus::Custom(418))), 418);
    }

    #[test]
    fn test_is_success() {
        assert!(is_success(&HttpResponse::new(HttpStatus::Ok)));
        assert!(is_success(&HttpResponse::new(HttpStatus::Custom(201))));
        assert!(is_success(&HttpResponse::new(HttpStatus::Custom(299))));

        assert!(!is_success(&HttpResponse::new(HttpStatus::Custom(300))));
        assert!(!is_success(&HttpResponse::new(HttpStatus::Custom(199))));
        assert!(!is_success(&HttpResponse::new(HttpStatus::NotFound)));
        assert!(!is_success(&HttpResponse::new(HttpStatus::ServerError)));
    }

    #[test]
    fn test_extract_content_type() {
        let response = HttpResponse::new(HttpStatus::Ok)
            .with_header("Content-Type".to_string(), "application/json".to_string())
            .with_header("Cache-Control".to_string(), "no-cache".to_string());

        let content_type = extract_content_type(&response);
        assert_eq!(content_type, Some("application/json".to_string()));
    }

    #[test]
    fn test_extract_content_type_case_insensitive() {
        let response = HttpResponse::new(HttpStatus::Ok)
            .with_header("content-type".to_string(), "text/html".to_string());

        let content_type = extract_content_type(&response);
        assert_eq!(content_type, Some("text/html".to_string()));
    }

    #[test]
    fn test_extract_content_type_missing() {
        let response = HttpResponse::new(HttpStatus::Ok)
            .with_header("Cache-Control".to_string(), "no-cache".to_string());

        let content_type = extract_content_type(&response);
        assert_eq!(content_type, None);
    }

    #[test]
    fn test_response_builder() {
        let response = HttpResponse::new(HttpStatus::Ok)
            .with_body("Test body".to_string())
            .with_header("Content-Type".to_string(), "text/plain".to_string())
            .with_header("X-Custom".to_string(), "test-value".to_string());

        assert_eq!(response.status, HttpStatus::Ok);
        assert_eq!(response.body, Some("Test body".to_string()));
        assert_eq!(response.headers.len(), 2);
        assert!(response.headers.contains(&("Content-Type".to_string(), "text/plain".to_string())));
        assert!(response.headers.contains(&("X-Custom".to_string(), "test-value".to_string())));
    }

    #[test]
    fn test_complex_pattern_matching_scenarios() {
        // Test with headers but different status combinations
        let responses = vec![
            (HttpResponse::new(HttpStatus::Ok).with_body("Success".to_string()), "Success: Success"),
            (HttpResponse::new(HttpStatus::NotFound).with_body("Not found page".to_string()), "Error: Resource not found"),
            (HttpResponse::new(HttpStatus::Custom(201)).with_body("Created".to_string()), "Info: Status 201"),
            (HttpResponse::new(HttpStatus::Custom(401)).with_body("Unauthorized".to_string()), "Error: Status 401"),
        ];

        for (response, expected) in responses {
            assert_eq!(handle_response(response), expected);
        }
    }

    #[test]
    fn test_guard_coverage() {
        // Test edge cases around the 400 boundary for custom status codes
        let test_cases = vec![
            (HttpStatus::Custom(100), "Info: Status 100"),
            (HttpStatus::Custom(199), "Info: Status 199"),
            (HttpStatus::Custom(300), "Info: Status 300"),
            (HttpStatus::Custom(399), "Info: Status 399"),
            (HttpStatus::Custom(400), "Error: Status 400"),
            (HttpStatus::Custom(401), "Error: Status 401"),
            (HttpStatus::Custom(500), "Error: Status 500"),
            (HttpStatus::Custom(999), "Error: Status 999"),
        ];

        for (status, expected) in test_cases {
            let response = HttpResponse::new(status);
            assert_eq!(handle_response(response), expected);
        }
    }
}