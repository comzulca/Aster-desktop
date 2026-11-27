//! Closed synthetic terminal extraction for the text-only compaction path.
//! No response ID, extra output item or model instruction becomes a grant.
use super::{Failure, Prepared, Purpose, SimulatedCount, MODEL, WINDOW};
use serde::Deserialize;

const MAX_ENVELOPE: usize = 2 * 1024 * 1024;
const MAX_SUMMARY: usize = 16 * 1024;

#[derive(Deserialize)]
struct VisibleResponse {
    object: String,
    model: String,
    status: String,
    store: bool,
    #[serde(default)]
    previous_response_id: Option<serde::de::IgnoredAny>,
    #[serde(default)]
    error: Option<serde::de::IgnoredAny>,
    #[serde(default)]
    incomplete_details: Option<serde::de::IgnoredAny>,
    output: Vec<OutputItem>,
    usage: Usage,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum OutputItem {
    Reasoning {
        #[serde(default)]
        encrypted_content: Option<serde::de::IgnoredAny>,
    },
    Message {
        role: String,
        status: String,
        #[serde(default)]
        phase: Option<String>,
        content: Vec<OutputContent>,
    },
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum OutputContent {
    OutputText {
        text: String,
        #[serde(default)]
        annotations: Vec<serde::de::IgnoredAny>,
    },
    Refusal {},
}

#[derive(Deserialize)]
struct Usage {
    input_tokens: u64,
    output_tokens: u64,
    total_tokens: u64,
    input_tokens_details: InputDetails,
}

#[derive(Deserialize)]
struct InputDetails {
    cached_tokens: u64,
}

#[derive(Debug, PartialEq, Eq)]
struct AcceptedSummary {
    request_binding: [u8; 32],
    text: String,
    metered_input_tokens: u64,
    metered_output_tokens: u64,
}

// Accept a candidate only when an independently supplied response for the
// same summary matches the measured window and the exact projected model.
// Billing settlement, native approval and transaction commit are separate.
fn checked_summary(
    summary: &Prepared,
    count: SimulatedCount,
    raw: &[u8],
) -> Result<AcceptedSummary, Failure> {
    if summary.purpose != Purpose::Summary {
        return Err(Failure::WrongPurpose);
    }
    if raw.is_empty() || raw.len() > MAX_ENVELOPE {
        return Err(Failure::Limit);
    }
    let projected_tokens = count.tokens;
    summary.remaining(count)?;
    let raw: VisibleResponse = serde_json::from_slice(raw).map_err(|_| Failure::Receipt)?;
    if raw.object != "response"
        || raw.model != MODEL
        || raw.status != "completed"
        || raw.store
        || raw.previous_response_id.is_some()
        || raw.error.is_some()
        || raw.incomplete_details.is_some()
        || !(1..=2).contains(&raw.output.len())
    {
        return Err(Failure::Receipt);
    }
    let usage = raw.usage;
    if usage.input_tokens != projected_tokens
        || !(1..=summary.output).contains(&usage.output_tokens)
        || usage.input_tokens_details.cached_tokens > usage.input_tokens
        || usage.input_tokens.checked_add(usage.output_tokens) != Some(usage.total_tokens)
        || usage.total_tokens > WINDOW
    {
        return Err(Failure::Receipt);
    }

    let mut output = raw.output.into_iter();
    let mut message = output.next().ok_or(Failure::Receipt)?;
    if let OutputItem::Reasoning { encrypted_content } = message {
        if encrypted_content.is_some() {
            return Err(Failure::Receipt);
        }
        message = output.next().ok_or(Failure::Receipt)?;
    }
    let OutputItem::Message {
        role,
        status,
        phase,
        content,
    } = message
    else {
        return Err(Failure::Receipt);
    };
    if role != "assistant"
        || status != "completed"
        || phase.is_some_and(|phase| phase != "final_answer")
        || content.len() != 1
        || output.next().is_some()
    {
        return Err(Failure::Receipt);
    }
    let Some(OutputContent::OutputText { text, annotations }) = content.into_iter().next() else {
        return Err(Failure::Receipt);
    };
    if !annotations.is_empty()
        || text.trim().is_empty()
        || text.len() > MAX_SUMMARY
        || text
            .chars()
            .any(|ch| ch.is_control() && !matches!(ch, '\n' | '\r' | '\t'))
    {
        return Err(Failure::Receipt);
    }
    Ok(AcceptedSummary {
        request_binding: summary.binding,
        text,
        metered_input_tokens: usage.input_tokens,
        metered_output_tokens: usage.output_tokens,
    })
}

fn prepare() -> Prepared {
    super::prepare(
        super::scope(),
        &[super::Turn {
            user: "Retain an eligible fact",
            assistant: "Atlas is synthetic",
        }],
        super::Intent::CompactEligiblePrefix,
        super::MAX_OUTPUT,
    )
    .unwrap()
}

fn response() -> serde_json::Value {
    serde_json::json!({
        "object":"response", "model":MODEL, "status":"completed", "store":false,
        "previous_response_id":null, "error":null, "incomplete_details":null,
        "id":"resp_fixture",
        "output":[
            {"type":"reasoning","id":"rs_not_replayed"},
            {"type":"message","role":"assistant","phase":"final_answer","status":"completed",
             "content":[{"type":"output_text","text":"Atlas is synthetic.","annotations":[]}]}
        ],
        "usage":{"input_tokens":300,"output_tokens":20,"total_tokens":320,
                 "input_tokens_details":{"cached_tokens":10}}
    })
}

fn validate(body: serde_json::Value) -> Result<AcceptedSummary, Failure> {
    let prepared = prepare();
    let count = prepared
        .simulated_reply(super::count_json(300).as_bytes())
        .unwrap();
    checked_summary(&prepared, count, &serde_json::to_vec(&body).unwrap())
}

#[test]
fn completed_exact_usage_returns_only_visible_text_and_host_binding() {
    let result = validate(response()).unwrap();
    assert_eq!(result.text, "Atlas is synthetic.");
    assert_eq!(result.request_binding, prepare().binding);
    assert_eq!(
        (result.metered_input_tokens, result.metered_output_tokens),
        (300, 20)
    );
}

#[test]
fn terminal_shape_and_binding_fail_before_any_candidate_can_activate() {
    for (field, value) in [
        ("status", serde_json::json!("incomplete")),
        ("model", serde_json::json!("muse-spark-1.3")),
        ("store", serde_json::json!(true)),
        ("previous_response_id", serde_json::json!("resp_other")),
        ("error", serde_json::json!({"code": "server_error"})),
        (
            "incomplete_details",
            serde_json::json!({"reason":"max_output_tokens"}),
        ),
    ] {
        let mut body = response();
        body[field] = value;
        assert_eq!(validate(body), Err(Failure::Receipt), "{field}");
    }
    let mut body = response();
    body["usage"]["input_tokens"] = 302.into();
    body["usage"]["total_tokens"] = 322.into();
    assert_eq!(validate(body), Err(Failure::Receipt));

    let prepared = prepare();
    let other = super::prepare(
        super::scope(),
        &[super::Turn {
            user: "Other",
            assistant: "Mismatch",
        }],
        super::Intent::CompactEligiblePrefix,
        super::MAX_OUTPUT,
    )
    .unwrap();
    let stale = other
        .simulated_reply(super::count_json(300).as_bytes())
        .unwrap();
    assert_eq!(
        checked_summary(&prepared, stale, &serde_json::to_vec(&response()).unwrap()),
        Err(Failure::Stale)
    );
}

#[test]
fn refuses_partial_or_tool_output_opaque_replay_and_overlarge_text() {
    let cases = [
        (
            "content",
            serde_json::json!([{"type":"refusal","refusal":"Cannot summarise"}]),
        ),
        ("status", serde_json::json!("in_progress")),
        ("phase", serde_json::json!("commentary")),
        ("role", serde_json::json!("developer")),
        ("type", serde_json::json!("function_call")),
        (
            "content",
            serde_json::json!([{"type":"output_text","text":"x".repeat(MAX_SUMMARY + 1)}]),
        ),
    ];
    for (field, value) in cases {
        let mut body = response();
        body["output"][1][field] = value;
        assert_eq!(validate(body), Err(Failure::Receipt), "{field}");
    }
    let mut body = response();
    body["output"][0]["encrypted_content"] = "opaque".into();
    assert_eq!(validate(body), Err(Failure::Receipt));
}

#[test]
fn rejects_duplicate_critical_fields_and_bounded_envelopes() {
    let prepared = prepare();
    let count = || {
        prepared
            .simulated_reply(super::count_json(300).as_bytes())
            .unwrap()
    };
    let encoded = serde_json::to_string(&response()).unwrap();
    let duplicated = encoded.replacen('{', "{\"status\":\"completed\",", 1);
    assert_eq!(
        checked_summary(&prepared, count(), duplicated.as_bytes()),
        Err(Failure::Receipt)
    );
    assert_eq!(
        checked_summary(&prepared, count(), &vec![b' '; MAX_ENVELOPE + 1]),
        Err(Failure::Limit)
    );
}
