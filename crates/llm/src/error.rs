use thiserror::Error;

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("provider returned an error: {0}")]
    Provider(String),

    #[error("missing configuration key: {0}")]
    MissingConfig(String),

    #[error("invalid response format")]
    InvalidResponse,

    #[error("configuration parse error: {0}")]
    ConfigParse(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_error_display() {
        let err = LlmError::Http(reqwest::blocking::Client::new()
            .get("http://127.0.0.1:1")
            .send()
            .unwrap_err());
        let msg = err.to_string();
        assert!(msg.starts_with("HTTP request failed: "));
    }

    #[test]
    fn json_error_display() {
        let json_err = serde_json::from_str::<()>("invalid json").unwrap_err();
        let err = LlmError::Json(json_err);
        let msg = err.to_string();
        assert!(msg.starts_with("JSON error: "));
    }

    #[test]
    fn provider_error_display() {
        let err = LlmError::Provider("rate limit exceeded".into());
        assert_eq!(
            err.to_string(),
            "provider returned an error: rate limit exceeded"
        );
    }

    #[test]
    fn missing_config_error_display() {
        let err = LlmError::MissingConfig("OPENROUTER_API_KEY not set".into());
        assert_eq!(
            err.to_string(),
            "missing configuration key: OPENROUTER_API_KEY not set"
        );
    }

    #[test]
    fn invalid_response_display() {
        let err = LlmError::InvalidResponse;
        assert_eq!(err.to_string(), "invalid response format");
    }

    #[test]
    fn config_parse_error_display() {
        let err = LlmError::ConfigParse("invalid toml at line 1".into());
        assert_eq!(
            err.to_string(),
            "configuration parse error: invalid toml at line 1"
        );
    }

    #[test]
    fn error_trait_is_implemented() {
        fn assert_error<E: std::error::Error>() {}
        assert_error::<LlmError>();
    }

    #[test]
    fn http_error_from_reqwest() {
        let reqwest_err = reqwest::blocking::Client::new()
            .get("http://127.0.0.1:1")
            .send()
            .unwrap_err();
        let err: LlmError = reqwest_err.into();
        assert!(matches!(err, LlmError::Http(_)));
    }

    #[test]
    fn json_error_from_serde() {
        let serde_err = serde_json::from_str::<()>("not json").unwrap_err();
        let err: LlmError = serde_err.into();
        assert!(matches!(err, LlmError::Json(_)));
    }
}
