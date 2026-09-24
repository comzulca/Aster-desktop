//! Fixed NVIDIA-hosted pair contracts. No provider URL or request field comes
//! from a model, a project file or the renderer. This module never writes files.

use crate::policy::{ChatMessage, Model};
use futures_util::StreamExt;
use reqwest::{header, redirect::Policy, Client, StatusCode};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::sync::watch;
use zeroize::Zeroizing;

const ENDPOINT: &str = "https://integrate.api.nvidia.com/v1/chat/completions";
const MAX_REQUEST_BYTES: usize = 512 * 1024;
const MAX_RESPONSE_BYTES: usize = 8 * 1024 * 1024;
const MAX_FRAME_BYTES: usize = 256 * 1024;
const MAX_EVENTS: usize = 262_144;
const MAX_ANSWER_BYTES: usize = 512 * 1024;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ProviderFailure {
    InvalidInput,
    InvalidContract,
    Authentication,
    Forbidden,
    RateLimited,
    ProviderUnavailable,
    Network,
    Timeout,
    ResponseLimit,
    Incomplete,
    Cancelled,
}

impl ProviderFailure {
    pub fn code(self) -> &'static str {
        match self {
            Self::InvalidInput => "invalid_request",
            Self::InvalidContract => "provider_contract_mismatch",
            Self::Authentication => "nvidia_authentication_failed",
            Self::Forbidden => "nvidia_access_denied",
            Self::RateLimited => "nvidia_rate_limited",
            Self::ProviderUnavailable => "nvidia_unavailable",
            Self::Network => "network_error",
            Self::Timeout => "request_timeout",
            Self::ResponseLimit => "response_limit",
            Self::Incomplete => "incomplete_answer",
            Self::Cancelled => "cancelled",
        }
    }
}

pub fn exact_body(model: Model, messages: &[ChatMessage]) -> Result<Vec<u8>, ProviderFailure> {
    if messages.is_empty() || messages.last().is_none_or(|m| m.role != "user") {
        return Err(ProviderFailure::InvalidInput);
    }
    if !messages.iter().enumerate().all(|(i, message)| {
        !message.content.is_empty()
            && (if i % 2 == 0 { "user" } else { "assistant" }) == message.role
    }) {
        return Err(ProviderFailure::InvalidInput);
    }
    let body = match model {
        Model::Nemotron => json!({
            "model": model.id(),
            "messages": messages,
            "temperature": 1,
            "top_p": 0.95,
            "max_tokens": 16384,
            "chat_template_kwargs": {"enable_thinking": true},
            "reasoning_budget": 16384,
            "stream": true
        }),
        Model::Muse => json!({
            "model": model.id(),
            "messages": messages,
            "temperature": 0.95,
            "top_p": 1,
            "max_tokens": 8192,
            "tool_choice": "none",
            "stream": true
        }),
    };
    let bytes = serde_json::to_vec(&body).map_err(|_| ProviderFailure::InvalidInput)?;
    if bytes.len() > MAX_REQUEST_BYTES {
        return Err(ProviderFailure::InvalidInput);
    }
    Ok(bytes)
}

fn client() -> Result<Client, ProviderFailure> {
    Client::builder()
        .https_only(true)
        .no_proxy()
        .redirect(Policy::none())
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(900))
        .build()
        .map_err(|_| ProviderFailure::Network)
}

#[derive(Debug, PartialEq, Eq)]
enum StreamOutput {
    Content(String),
    Done,
}

/// Byte-level parser: waits for full LF/CRLF-delimited SSE event before UTF-8
/// decoding, so a multibyte character split between TCP chunks stays intact.
pub struct StreamDecoder {
    model: Model,
    pending: Vec<u8>,
    scan_from: usize,
    response_bytes: usize,
    events: usize,
    stream_id: Option<String>,
    saw_content: bool,
    saw_finish: bool,
    truncated: bool,
    done: bool,
}

impl StreamDecoder {
    pub fn new(model: Model) -> Self {
        Self {
            model,
            pending: Vec::new(),
            scan_from: 0,
            response_bytes: 0,
            events: 0,
            stream_id: None,
            saw_content: false,
            saw_finish: false,
            truncated: false,
            done: false,
        }
    }

    fn feed(&mut self, bytes: &[u8]) -> Result<Vec<StreamOutput>, ProviderFailure> {
        self.response_bytes = self
            .response_bytes
            .checked_add(bytes.len())
            .filter(|n| *n <= MAX_RESPONSE_BYTES)
            .ok_or(ProviderFailure::ResponseLimit)?;
        self.pending.extend_from_slice(bytes);
        let mut output = Vec::new();
        // Scan only the new bytes (plus the three-byte delimiter boundary),
        // and drain once, avoiding quadratic work for byte-at-a-time streams.
        let mut cursor = 0;
        let mut scan = self.scan_from;
        while scan < self.pending.len() {
            if scan.saturating_sub(cursor) > MAX_FRAME_BYTES {
                return Err(ProviderFailure::ResponseLimit);
            }
            let delimiter_len = if self.pending[scan..].starts_with(b"\n\n") {
                2
            } else if self.pending[scan..].starts_with(b"\r\n\r\n") {
                4
            } else {
                scan += 1;
                continue;
            };
            let event = self.pending[cursor..scan].to_vec();
            self.events += 1;
            if self.events > MAX_EVENTS {
                return Err(ProviderFailure::ResponseLimit);
            }
            if let Some(item) = self.parse_event(&event)? {
                output.push(item);
            }
            scan += delimiter_len;
            cursor = scan;
        }
        if cursor != 0 {
            self.pending.drain(..cursor);
        }
        if self.pending.len() > MAX_FRAME_BYTES {
            return Err(ProviderFailure::ResponseLimit);
        }
        self.scan_from = self.pending.len().saturating_sub(3);
        Ok(output)
    }

    fn parse_event(&mut self, event: &[u8]) -> Result<Option<StreamOutput>, ProviderFailure> {
        if self.done {
            return Err(ProviderFailure::InvalidContract);
        }
        let text = std::str::from_utf8(event).map_err(|_| ProviderFailure::InvalidContract)?;
        let mut data = String::new();
        for raw_line in text.split('\n') {
            let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
            if line.starts_with(':') || line.is_empty() {
                continue;
            }
            let line = line
                .strip_prefix("data:")
                .ok_or(ProviderFailure::InvalidContract)?;
            if !data.is_empty() {
                data.push('\n');
            }
            data.push_str(line.strip_prefix(' ').unwrap_or(line));
        }
        if data.is_empty() {
            return Ok(None);
        }
        if data == "[DONE]" {
            if !self.saw_finish || !self.saw_content || self.truncated {
                return Err(ProviderFailure::Incomplete);
            }
            self.done = true;
            return Ok(Some(StreamOutput::Done));
        }
        if self.saw_finish {
            // Usage-only events after finish are valid, but may never contain
            // another choice, answer or action.
            let v: Value =
                serde_json::from_str(&data).map_err(|_| ProviderFailure::InvalidContract)?;
            if v.get("choices")
                .is_some_and(|choices| !choices.as_array().is_some_and(|array| array.is_empty()))
            {
                return Err(ProviderFailure::InvalidContract);
            }
            self.validate_identity(&v)?;
            return Ok(None);
        }
        let value: Value =
            serde_json::from_str(&data).map_err(|_| ProviderFailure::InvalidContract)?;
        self.validate_identity(&value)?;
        let choices = value
            .get("choices")
            .and_then(Value::as_array)
            .ok_or(ProviderFailure::InvalidContract)?;
        if choices.is_empty() {
            return Ok(None);
        }
        if choices.len() != 1 || choices[0].get("index").and_then(Value::as_u64) != Some(0) {
            return Err(ProviderFailure::InvalidContract);
        }
        let choice = &choices[0];
        if choice.get("message").is_some() {
            return Err(ProviderFailure::InvalidContract);
        }
        let delta = choice
            .get("delta")
            .and_then(Value::as_object)
            .ok_or(ProviderFailure::InvalidContract)?;
        if delta.contains_key("tool_calls")
            || delta.contains_key("function_call")
            || choice.get("finish_reason") == Some(&Value::String("tool_calls".into()))
        {
            return Err(ProviderFailure::InvalidContract);
        }
        // Reasoning is allowed by both provider contracts but never emitted
        // to the normal answer channel, logged, stored or interpreted as a tool.
        if !matches!(
            delta.get("reasoning_content"),
            None | Some(Value::Null) | Some(Value::String(_))
        ) {
            return Err(ProviderFailure::InvalidContract);
        }
        let content = match delta.get("content") {
            Some(Value::String(text)) => Some(text.clone()),
            None | Some(Value::Null) => None,
            _ => return Err(ProviderFailure::InvalidContract),
        };
        if let Some(reason) = choice.get("finish_reason") {
            match reason {
                Value::Null => {}
                Value::String(text) if text == "stop" => self.saw_finish = true,
                Value::String(text) if text == "length" => {
                    self.saw_finish = true;
                    self.truncated = true;
                }
                _ => return Err(ProviderFailure::Incomplete),
            }
        }
        if let Some(text) = content {
            if !text.is_empty() {
                self.saw_content = true;
                return Ok(Some(StreamOutput::Content(text)));
            }
        }
        Ok(None)
    }

    fn validate_identity(&mut self, value: &Value) -> Result<(), ProviderFailure> {
        if value.get("model").and_then(Value::as_str) != Some(self.model.id()) {
            return Err(ProviderFailure::InvalidContract);
        }
        let id = value
            .get("id")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty() && s.len() <= 128)
            .ok_or(ProviderFailure::InvalidContract)?;
        if let Some(prior) = &self.stream_id {
            if prior != id {
                return Err(ProviderFailure::InvalidContract);
            }
        } else {
            self.stream_id = Some(id.to_owned());
        }
        if value.get("tool_calls").is_some() || value.get("function_call").is_some() {
            return Err(ProviderFailure::InvalidContract);
        }
        Ok(())
    }
}

fn classify_status(status: StatusCode) -> ProviderFailure {
    match status.as_u16() {
        401 => ProviderFailure::Authentication,
        403 => ProviderFailure::Forbidden,
        429 => ProviderFailure::RateLimited,
        500..=599 => ProviderFailure::ProviderUnavailable,
        _ => ProviderFailure::InvalidContract,
    }
}

pub async fn stream_chat<F>(
    model: Model,
    messages: &[ChatMessage],
    key: Zeroizing<String>,
    cancel: watch::Receiver<bool>,
    on_content: F,
) -> Result<String, ProviderFailure>
where
    F: FnMut(String) -> Result<(), ProviderFailure>,
{
    let http = client()?;
    // There is exactly one production call site for the transport and its
    // target is a constant. The injectable seam below is module-private and
    // exists for loopback fake-server fixtures only.
    stream_with_client(&http, ENDPOINT, model, messages, key, cancel, on_content).await
}

async fn stream_with_client<F>(
    client: &Client,
    endpoint: &str,
    model: Model,
    messages: &[ChatMessage],
    key: Zeroizing<String>,
    mut cancel: watch::Receiver<bool>,
    mut on_content: F,
) -> Result<String, ProviderFailure>
where
    F: FnMut(String) -> Result<(), ProviderFailure>,
{
    if *cancel.borrow() {
        return Err(ProviderFailure::Cancelled);
    }
    let body = exact_body(model, messages)?;
    let request = client
        .post(endpoint)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ACCEPT, "text/event-stream")
        .header(header::ACCEPT_ENCODING, "identity")
        .bearer_auth(key.as_str())
        .body(body);
    // The secret has been copied into an HTTP header by reqwest. Discard the
    // original OS-owned string before the first await. No debug logging.
    drop(key);
    let response = tokio::select! {
        _ = cancel.changed() => return Err(ProviderFailure::Cancelled),
        result = tokio::time::timeout(Duration::from_secs(30), request.send()) => {
            result.map_err(|_| ProviderFailure::Timeout)?
                .map_err(|e| if e.is_timeout() { ProviderFailure::Timeout } else { ProviderFailure::Network })?
        }
    };
    if response.status() != StatusCode::OK {
        return Err(classify_status(response.status()));
    }
    if response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .is_none_or(|s| {
            !s.split(';')
                .next()
                .is_some_and(|m| m.trim().eq_ignore_ascii_case("text/event-stream"))
        })
    {
        return Err(ProviderFailure::InvalidContract);
    }
    let mut decoder = StreamDecoder::new(model);
    let mut answer = String::new();
    let mut bytes = response.bytes_stream();
    loop {
        if *cancel.borrow() {
            return Err(ProviderFailure::Cancelled);
        }
        let next = tokio::select! {
            _ = cancel.changed() => return Err(ProviderFailure::Cancelled),
            result = tokio::time::timeout(Duration::from_secs(60), bytes.next()) => {
                result.map_err(|_| ProviderFailure::Timeout)?
            }
        };
        let chunk = match next {
            Some(Ok(chunk)) => chunk,
            Some(Err(error)) => {
                return Err(if error.is_timeout() {
                    ProviderFailure::Timeout
                } else {
                    ProviderFailure::Network
                })
            }
            None => return Err(ProviderFailure::Incomplete),
        };
        for item in decoder.feed(&chunk)? {
            match item {
                StreamOutput::Content(text) => {
                    if answer.len().saturating_add(text.len()) > MAX_ANSWER_BYTES {
                        return Err(ProviderFailure::ResponseLimit);
                    }
                    answer.push_str(&text);
                    on_content(text)?;
                }
                StreamOutput::Done => {
                    if *cancel.borrow() {
                        return Err(ProviderFailure::Cancelled);
                    }
                    return Ok(answer);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    fn fake_server(response: String) -> (String, thread::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = format!(
            "http://{}/v1/chat/completions",
            listener.local_addr().unwrap()
        );
        let handle = thread::spawn(move || {
            let (mut connection, _) = listener.accept().unwrap();
            connection
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut received = Vec::new();
            let mut buffer = [0_u8; 4096];
            let (header_end, content_len) = loop {
                let count = connection.read(&mut buffer).unwrap();
                assert!(count > 0, "fake server received a premature EOF");
                received.extend_from_slice(&buffer[..count]);
                assert!(received.len() < MAX_REQUEST_BYTES + 4096);
                if let Some(end) = received.windows(4).position(|w| w == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&received[..end]).to_ascii_lowercase();
                    let size = headers
                        .lines()
                        .find_map(|line| line.strip_prefix("content-length:"))
                        .and_then(|s| s.trim().parse::<usize>().ok())
                        .unwrap();
                    break (end + 4, size);
                }
            };
            while received.len() < header_end + content_len {
                let count = connection.read(&mut buffer).unwrap();
                assert!(count > 0, "fake server request body was truncated");
                received.extend_from_slice(&buffer[..count]);
            }
            connection.write_all(response.as_bytes()).unwrap();
            String::from_utf8(received).unwrap()
        });
        (address, handle)
    }

    fn fake_response(status: &str, content_type: &str, body: &str) -> String {
        format!(
            "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
    }

    fn test_client() -> Client {
        Client::builder()
            .no_proxy()
            .redirect(Policy::none())
            .build()
            .unwrap()
    }

    fn msg() -> Vec<ChatMessage> {
        vec![ChatMessage {
            role: "user",
            content: "hola".into(),
        }]
    }

    fn frame(model: Model, content: &str, finish: Option<&str>) -> String {
        let payload = json!({
            "id":"chat-1", "model":model.id(),
            "choices":[{"index":0,"delta":{"content":content},"finish_reason":finish}]
        });
        format!("data: {payload}\n\n")
    }

    #[test]
    fn exact_pairs_stream_with_distinct_contracts() {
        let n: Value =
            serde_json::from_slice(&exact_body(Model::Nemotron, &msg()).unwrap()).unwrap();
        assert_eq!(n["model"], "nvidia/nemotron-3.5-lightning-30b-a3b");
        assert_eq!(n["reasoning_budget"], 16384);
        assert_eq!(n["chat_template_kwargs"]["enable_thinking"], true);
        assert_eq!(n["stream"], true);
        assert!(n.get("tools").is_none());
        let m: Value = serde_json::from_slice(&exact_body(Model::Muse, &msg()).unwrap()).unwrap();
        assert_eq!(m["model"], "meta/muse-glimmer-30b");
        assert_eq!(m["tool_choice"], "none");
        assert_eq!(m["stream"], true);
        assert_eq!(m["max_tokens"], 8192);
        assert!(m.get("chat_template_kwargs").is_none());
    }

    #[test]
    fn split_utf8_crlf_reasoning_and_empty_choices() {
        let mut parser = StreamDecoder::new(Model::Nemotron);
        let a = "data: {\"id\":\"chat-1\",\"model\":\"nvidia/nemotron-3.5-lightning-30b-a3b\",\"choices\":[]}\r\n\r\n";
        assert_eq!(
            parser.feed(a.as_bytes()).unwrap(),
            Vec::<StreamOutput>::new()
        );
        let reasoning = json!({"id":"chat-1","model":Model::Nemotron.id(),
            "choices":[{"index":0,"delta":{"reasoning_content":"private", "content":null},"finish_reason":null}]});
        assert!(parser
            .feed(format!("data: {reasoning}\r\n\r\n").as_bytes())
            .unwrap()
            .is_empty());
        let event = frame(Model::Nemotron, "España", Some("stop"));
        let position = event.find('ñ').unwrap() + 1;
        assert!(parser
            .feed(&event.as_bytes()[..position])
            .unwrap()
            .is_empty());
        assert_eq!(
            parser.feed(&event.as_bytes()[position..]).unwrap(),
            vec![StreamOutput::Content("España".into())]
        );
        assert_eq!(
            parser.feed(b"data: [DONE]\r\n\r\n").unwrap(),
            vec![StreamOutput::Done]
        );
    }

    #[test]
    fn malformed_identity_tool_and_truncation_fail_closed() {
        let mut parser = StreamDecoder::new(Model::Muse);
        assert_eq!(
            parser
                .feed(frame(Model::Nemotron, "wrong", Some("stop")).as_bytes())
                .err(),
            Some(ProviderFailure::InvalidContract)
        );
        let mut parser = StreamDecoder::new(Model::Muse);
        let call = json!({"id":"chat-1", "model":Model::Muse.id(),
            "choices":[{"index":0,"delta":{"tool_calls":[{"id":"bad"}]},"finish_reason":"tool_calls"}]});
        assert_eq!(
            parser.feed(format!("data: {call}\n\n").as_bytes()).err(),
            Some(ProviderFailure::InvalidContract)
        );
        let mut parser = StreamDecoder::new(Model::Muse);
        parser
            .feed(frame(Model::Muse, "partial", Some("length")).as_bytes())
            .unwrap();
        assert_eq!(
            parser.feed(b"data: [DONE]\n\n").err(),
            Some(ProviderFailure::Incomplete)
        );
        let mut parser = StreamDecoder::new(Model::Muse);
        assert_eq!(
            parser.feed(b"data: not-json\n\n").err(),
            Some(ProviderFailure::InvalidContract)
        );
    }

    #[test]
    fn finite_frame_budget_duplicate_done_and_missing_finish() {
        let mut parser = StreamDecoder::new(Model::Muse);
        let oversized_frame = vec![b'a'; MAX_FRAME_BYTES + 1];
        assert_eq!(
            parser.feed(&oversized_frame).err(),
            Some(ProviderFailure::ResponseLimit)
        );
        let mut parser = StreamDecoder::new(Model::Muse);
        parser
            .feed(frame(Model::Muse, "answer", Some("stop")).as_bytes())
            .unwrap();
        assert_eq!(
            parser.feed(b"data: [DONE]\n\ndata: [DONE]\n\n").err(),
            Some(ProviderFailure::InvalidContract)
        );
        let mut parser = StreamDecoder::new(Model::Muse);
        parser
            .feed(frame(Model::Muse, "answer", None).as_bytes())
            .unwrap();
        assert_eq!(
            parser.feed(b"data: [DONE]\n\n").err(),
            Some(ProviderFailure::Incomplete)
        );
    }

    #[tokio::test]
    async fn loopback_fixture_sends_exact_muse_shape_and_separates_reasoning() {
        let reasoning = json!({"id":"chat-1","model":Model::Muse.id(),
            "choices":[{"index":0,"delta":{"reasoning_content":"not answer"},"finish_reason":null}]});
        let body = format!(
            "data: {reasoning}\n\n{}data: [DONE]\n\n",
            frame(Model::Muse, "Hola", Some("stop"))
        );
        let (url, listener) = fake_server(fake_response("200 OK", "text/event-stream", &body));
        let (_tx, cancel) = watch::channel(false);
        let mut parts = Vec::new();
        let answer = stream_with_client(
            &test_client(),
            &url,
            Model::Muse,
            &msg(),
            Zeroizing::new("synthetic-fixture-key-123".to_owned()),
            cancel,
            |text| {
                parts.push(text);
                Ok(())
            },
        )
        .await
        .unwrap();
        assert_eq!(answer, "Hola");
        assert_eq!(parts, vec!["Hola"]);
        let captured = listener.join().unwrap();
        assert!(captured.starts_with("POST /v1/chat/completions HTTP/1.1"));
        assert!(captured
            .to_ascii_lowercase()
            .contains("authorization: bearer synthetic-fixture-key-123\r\n"));
        let body: Value = serde_json::from_str(captured.split_once("\r\n\r\n").unwrap().1).unwrap();
        assert_eq!(body["model"], Model::Muse.id());
        assert_eq!(body["stream"], true);
        assert_eq!(body["tool_choice"], "none");
        assert!(body.get("tools").is_none());
    }

    #[tokio::test]
    async fn loopback_fixture_rejects_non_streaming_or_pending_and_does_not_retry() {
        for (status, content_type, body, expected) in [
            (
                "200 OK",
                "application/json",
                "{}",
                ProviderFailure::InvalidContract,
            ),
            (
                "202 Accepted",
                "text/event-stream",
                "",
                ProviderFailure::InvalidContract,
            ),
            (
                "401 Unauthorized",
                "application/json",
                "{}",
                ProviderFailure::Authentication,
            ),
            (
                "429 Too Many Requests",
                "application/json",
                "{}",
                ProviderFailure::RateLimited,
            ),
            (
                "302 Found",
                "text/event-stream",
                "",
                ProviderFailure::InvalidContract,
            ),
        ] {
            let (url, listener) = fake_server(fake_response(status, content_type, body));
            let (_tx, cancel) = watch::channel(false);
            let result = stream_with_client(
                &test_client(),
                &url,
                Model::Nemotron,
                &msg(),
                Zeroizing::new("synthetic-fixture-key-123".to_owned()),
                cancel,
                |_| Ok(()),
            )
            .await;
            assert_eq!(result.err(), Some(expected));
            let captured = listener.join().unwrap();
            assert!(captured.contains("\r\n\r\n"));
        }
    }

    #[tokio::test]
    async fn already_cancelled_is_rejected_before_network() {
        let (tx, cancel) = watch::channel(true);
        assert!(*tx.borrow());
        let response = stream_with_client(
            &test_client(),
            "http://127.0.0.1:9/v1/chat/completions",
            Model::Muse,
            &msg(),
            Zeroizing::new("synthetic-fixture-key-123".to_owned()),
            cancel,
            |_| Ok(()),
        )
        .await;
        assert_eq!(response.err(), Some(ProviderFailure::Cancelled));
    }
}
