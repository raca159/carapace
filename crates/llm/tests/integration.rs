use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

use game_llm::{ChatMessage, LlmConfig, OpenAiProvider, OpenRouterProvider, send_messages};

fn mock_response(status: u16, status_text: &str, body: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{}",
        status,
        status_text,
        body.len(),
        body
    )
    .into_bytes()
}

fn start_mock_server<F>(handler: F) -> (u16, Arc<Mutex<Option<String>>>)
where
    F: Fn(&str) -> Vec<u8> + Send + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let captured_request: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let captured_request_clone = captured_request.clone();

    thread::spawn(move || {
        if let Ok((stream, _)) = listener.accept() {
            let mut reader = BufReader::new(&stream);
            let mut request_line = String::new();
            let mut headers = String::new();
            let mut content_length: usize = 0;

            reader.read_line(&mut request_line).ok();

            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).ok() != Some(0) && line != "\r\n" {
                    let lower = line.to_lowercase();
                    if let Some(len) = lower
                        .strip_prefix("content-length:")
                        .and_then(|s| s.trim().parse().ok())
                    {
                        content_length = len;
                    }
                    headers.push_str(&line);
                } else {
                    break;
                }
            }

            let mut body = vec![0u8; content_length];
            reader.read_exact(&mut body).ok();
            let body_str = String::from_utf8_lossy(&body).to_string();
            let full_request = format!("{}{}\r\n{}", request_line.trim(), headers.trim(), body_str);
            *captured_request_clone.lock().unwrap() = Some(full_request.clone());

            let response = handler(&body_str);
            let mut stream = stream;
            stream.write_all(&response).ok();
        }
    });

    (port, captured_request)
}

fn make_config(port: u16) -> LlmConfig {
    LlmConfig {
        provider: "openrouter".into(),
        model: "openai/gpt-4o-mini".into(),
        endpoint: format!("http://127.0.0.1:{}/v1/chat/completions", port),
        api_key: Some("sk-test-key".into()),
        max_tokens: Some(256),
        temperature: Some(0.7),
        site_url: Some("https://test.game".into()),
        app_name: Some("TestApp".into()),
        prompts: Default::default(),
    }
}

fn make_messages() -> Vec<ChatMessage> {
    vec![
        ChatMessage {
            role: "system".into(),
            content: "You are a test narrator.".into(),
        },
        ChatMessage {
            role: "user".into(),
            content: "Describe the scene.".into(),
        },
    ]
}

#[test]
fn integration_happy_path_returns_content() {
    let handler = move |body: &str| {
        assert!(
            body.contains("openai/gpt-4o-mini"),
            "request body should contain model name"
        );
        mock_response(
            200,
            "OK",
            r#"{"choices":[{"message":{"role":"assistant","content":"The ruined cathedral looms in the moonlight."}}]}"#,
        )
    };

    let (port, _captured) = start_mock_server(handler);
    let config = make_config(port);
    let provider = OpenRouterProvider;
    let messages = make_messages();

    let result = send_messages(&config, &provider, messages);
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "The ruined cathedral looms in the moonlight."
    );
}

#[test]
fn integration_http_error_status_returns_error() {
    let handler = move |_body: &str| {
        mock_response(
            429,
            "Too Many Requests",
            r#"{"error":"rate limit exceeded"}"#,
        )
    };

    let (port, _captured) = start_mock_server(handler);
    let config = make_config(port);
    let provider = OpenRouterProvider;
    let messages = make_messages();

    let result = send_messages(&config, &provider, messages);
    assert!(result.is_err());
}

#[test]
fn integration_server_error_returns_error() {
    let handler = move |_body: &str| {
        mock_response(500, "Internal Server Error", r#"{"error":"server error"}"#)
    };

    let (port, _captured) = start_mock_server(handler);
    let config = make_config(port);
    let provider = OpenRouterProvider;
    let messages = make_messages();

    let result = send_messages(&config, &provider, messages);
    assert!(result.is_err());
}

#[test]
fn integration_malformed_json_returns_error() {
    let handler = move |_body: &str| {
        mock_response(200, "OK", "this is not valid json")
    };

    let (port, _captured) = start_mock_server(handler);
    let config = make_config(port);
    let provider = OpenRouterProvider;
    let messages = make_messages();

    let result = send_messages(&config, &provider, messages);
    assert!(result.is_err());
}

#[test]
fn integration_empty_choices_returns_error() {
    let handler = move |_body: &str| {
        mock_response(200, "OK", r#"{"choices":[]}"#)
    };

    let (port, _captured) = start_mock_server(handler);
    let config = make_config(port);
    let provider = OpenRouterProvider;
    let messages = make_messages();

    let result = send_messages(&config, &provider, messages);
    assert!(result.is_err());
}

#[test]
fn integration_request_contains_model_name() {
    let captured: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let captured_clone = captured.clone();

    let handler = move |body: &str| {
        *captured_clone.lock().unwrap() = Some(body.to_string());
        mock_response(
            200,
            "OK",
            r#"{"choices":[{"message":{"role":"assistant","content":"response"}}]}"#,
        )
    };

    let (port, _) = start_mock_server(handler);
    let config = make_config(port);
    let provider = OpenRouterProvider;
    let messages = make_messages();

    let _ = send_messages(&config, &provider, messages);

    let body = captured.lock().unwrap().take().unwrap();
    assert!(body.contains("openai/gpt-4o-mini"));
    assert!(body.contains("max_tokens"));
    assert!(body.contains("temperature"));
    assert!(body.contains("messages"));
}

#[test]
fn integration_request_has_auth_header() {
    let handler = move |_body: &str| {
        mock_response(
            200,
            "OK",
            r#"{"choices":[{"message":{"role":"assistant","content":"ok"}}]}"#,
        )
    };

    let (port, request_captured) = start_mock_server(handler);
    let config = make_config(port);
    let provider = OpenRouterProvider;
    let messages = make_messages();
    let _ = send_messages(&config, &provider, messages);

    let full_request_lower = request_captured.lock().unwrap().take().unwrap().to_lowercase();
    assert!(full_request_lower.contains("authorization: bearer sk-test-key"));
    assert!(full_request_lower.contains("content-type: application/json"));
    assert!(full_request_lower.contains("http-referer: https://test.game"));
    assert!(full_request_lower.contains("x-title: testapp"));
}

#[test]
fn integration_openai_path_works_with_mock() {
    let body_captured: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let body_captured_clone = body_captured.clone();

    let handler = move |body: &str| {
        *body_captured_clone.lock().unwrap() = Some(body.to_string());
        mock_response(
            200,
            "OK",
            r#"{"choices":[{"message":{"role":"assistant","content":"OpenAI response"}}]}"#,
        )
    };

    let (port, _) = start_mock_server(handler);
    let config = LlmConfig {
        provider: "openai".into(),
        model: "gpt-4o-mini".into(),
        endpoint: format!("http://127.0.0.1:{}/v1/chat/completions", port),
        api_key: Some("sk-openai-key".into()),
        max_tokens: Some(256),
        temperature: Some(0.7),
        site_url: None,
        app_name: None,
        prompts: Default::default(),
    };
    let provider = OpenAiProvider;
    let messages = make_messages();

    let result = send_messages(&config, &provider, messages);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "OpenAI response");

    let body = body_captured.lock().unwrap().take().unwrap();
    assert!(body.contains("gpt-4o-mini"));
}

#[test]
fn integration_empty_content_null_returns_error() {
    let handler = move |_body: &str| {
        mock_response(
            200,
            "OK",
            r#"{"choices":[{"message":{"role":"assistant","content":null}}]}"#,
        )
    };

    let (port, _captured) = start_mock_server(handler);
    let config = make_config(port);
    let provider = OpenRouterProvider;
    let messages = make_messages();

    let result = send_messages(&config, &provider, messages);
    assert!(result.is_err());
}

#[test]
fn integration_sends_messages_in_request_body() {
    let body_captured: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let body_captured_clone = body_captured.clone();

    let handler = move |body: &str| {
        *body_captured_clone.lock().unwrap() = Some(body.to_string());
        mock_response(
            200,
            "OK",
            r#"{"choices":[{"message":{"role":"assistant","content":"response"}}]}"#,
        )
    };

    let (port, _) = start_mock_server(handler);
    let config = make_config(port);
    let provider = OpenRouterProvider;
    let messages = make_messages();

    let _ = send_messages(&config, &provider, messages);

    let body = body_captured.lock().unwrap().take().unwrap();
    assert!(body.contains("You are a test narrator."));
    assert!(body.contains("Describe the scene."));
    assert!(body.contains("system"));
    assert!(body.contains("user"));
}
