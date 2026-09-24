//! Host-owned, versioned admission and per-conversation state.
//! Only a successfully completed answer is eligible for ephemeral continuation.

use serde::Serialize;
use std::collections::{HashMap, HashSet};
use tokio::sync::watch;

pub const POLICY_VERSION: u32 = 1;
pub const DEFAULT_MODEL: &str = "nvidia/nemotron-3.5-lightning-30b-a3b";
pub const MAX_MESSAGE_BYTES: usize = 64 * 1024;
pub const MAX_CONTEXT_BYTES: usize = 512 * 1024;
const MAX_TURNS: usize = 20;
const MAX_CONVERSATIONS: usize = 32;
const MAX_SESSION_REQUESTS: usize = 64;
const MAX_ACTIVE_REQUESTS: usize = 1;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Model {
    Nemotron,
    Muse,
}

impl Model {
    pub fn parse(id: &str) -> Option<Self> {
        match id {
            DEFAULT_MODEL => Some(Self::Nemotron),
            "meta/muse-glimmer-30b" => Some(Self::Muse),
            _ => None,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::Nemotron => DEFAULT_MODEL,
            Self::Muse => "meta/muse-glimmer-30b",
        }
    }
}

#[derive(Clone, Serialize)]
pub struct ChatMessage {
    pub role: &'static str,
    pub content: String,
}

#[derive(Clone)]
struct CompletedTurn {
    prompt: String,
    answer: String,
}

struct Active {
    request_id: String,
    model: Model,
    prompt: String,
    cancel: watch::Sender<bool>,
    cancel_requested: bool,
    stage: Stage,
}

pub enum TransportOutcome {
    Completed(String),
    Cancelled,
    Failed,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FinishDisposition {
    Completed,
    Cancelled,
    Failed,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Stage {
    Reserved,
    Streaming,
}

#[derive(Default)]
pub struct SessionState {
    // One active request per conversation; request IDs never reused in a session.
    active: HashMap<String, Active>,
    seen: HashSet<String>,
    conversation_ids: HashSet<String>,
    completed: HashMap<(String, &'static str), Vec<CompletedTurn>>,
}

pub struct Admission {
    pub messages: Vec<ChatMessage>,
    pub cancellation: watch::Receiver<bool>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AdmissionError {
    InvalidInput,
    Busy,
    RequestReplayed,
    SessionLimit,
    ContextLimit,
}

impl AdmissionError {
    pub fn code(self) -> &'static str {
        match self {
            Self::InvalidInput => "invalid_input",
            Self::Busy => "busy",
            Self::RequestReplayed => "request_replayed",
            Self::SessionLimit => "session_limit",
            Self::ContextLimit => "context_limit",
        }
    }
}

pub fn valid_id(id: &str) -> bool {
    (1..=80).contains(&id.len())
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

pub fn valid_prompt(prompt: &str) -> bool {
    !prompt.trim().is_empty()
        && prompt.len() <= MAX_MESSAGE_BYTES
        && !prompt.chars().any(|c| c == '\0')
}

impl SessionState {
    pub fn active_count(&self) -> usize {
        self.active.len()
    }

    pub fn reserve(
        &mut self,
        conversation_id: &str,
        request_id: &str,
        model: Model,
        prompt: String,
    ) -> Result<Admission, AdmissionError> {
        if !valid_id(conversation_id) || !valid_id(request_id) || !valid_prompt(&prompt) {
            return Err(AdmissionError::InvalidInput);
        }
        if self.active.contains_key(conversation_id) {
            return Err(AdmissionError::Busy);
        }
        if self.active.len() >= MAX_ACTIVE_REQUESTS {
            // Bound simultaneous external spend globally, including callers
            // that forge a new conversation ID for each request.
            return Err(AdmissionError::Busy);
        }
        if self.seen.contains(request_id) {
            return Err(AdmissionError::RequestReplayed);
        }
        if self.seen.len() >= MAX_SESSION_REQUESTS
            || (!self.conversation_ids.contains(conversation_id)
                && self.conversation_ids.len() >= MAX_CONVERSATIONS)
        {
            return Err(AdmissionError::SessionLimit);
        }
        let prior = self
            .completed
            .get(&(conversation_id.to_string(), model.id()))
            .cloned()
            .unwrap_or_default();
        let context_bytes = prior.iter().fold(prompt.len(), |sum, t| {
            sum.saturating_add(t.prompt.len())
                .saturating_add(t.answer.len())
        });
        if context_bytes > MAX_CONTEXT_BYTES || prior.len() >= MAX_TURNS {
            return Err(AdmissionError::ContextLimit);
        }
        let mut messages = Vec::with_capacity(prior.len() * 2 + 1);
        for turn in prior {
            messages.push(ChatMessage {
                role: "user",
                content: turn.prompt,
            });
            messages.push(ChatMessage {
                role: "assistant",
                content: turn.answer,
            });
        }
        messages.push(ChatMessage {
            role: "user",
            content: prompt.clone(),
        });
        let (cancel, cancellation) = watch::channel(false);
        self.active.insert(
            conversation_id.to_string(),
            Active {
                request_id: request_id.to_string(),
                model,
                prompt,
                cancel,
                cancel_requested: false,
                stage: Stage::Reserved,
            },
        );
        self.seen.insert(request_id.to_string());
        self.conversation_ids.insert(conversation_id.to_string());
        Ok(Admission {
            messages,
            cancellation,
        })
    }

    pub fn mark_streaming(&mut self, conversation_id: &str, request_id: &str) -> bool {
        match self.active.get_mut(conversation_id) {
            Some(active)
                if active.request_id == request_id
                    && !active.cancel_requested
                    && active.stage == Stage::Reserved =>
            {
                active.stage = Stage::Streaming;
                true
            }
            Some(active)
                if active.request_id == request_id
                    && !active.cancel_requested
                    && active.stage == Stage::Streaming =>
            {
                true
            }
            _ => false,
        }
    }

    pub fn cancel(&mut self, request_id: &str) -> bool {
        if let Some(active) = self
            .active
            .values_mut()
            .find(|a| a.request_id == request_id)
        {
            // Cancelling twice is idempotent. Only the worker emits a terminal event.
            active.cancel_requested = true;
            let _ = active.cancel.send(true);
            return true;
        }
        false
    }

    pub fn finish(
        &mut self,
        conversation_id: &str,
        request_id: &str,
        outcome: TransportOutcome,
    ) -> Option<FinishDisposition> {
        let active = self.active.get(conversation_id)?;
        if active.request_id != request_id {
            return None;
        }
        let active = self
            .active
            .remove(conversation_id)
            .expect("active checked above");
        if active.cancel_requested || matches!(&outcome, TransportOutcome::Cancelled) {
            return Some(FinishDisposition::Cancelled);
        }
        match outcome {
            TransportOutcome::Completed(answer) => {
                let key = (conversation_id.to_string(), active.model.id());
                self.completed.entry(key).or_default().push(CompletedTurn {
                    prompt: active.prompt,
                    answer,
                });
                Some(FinishDisposition::Completed)
            }
            TransportOutcome::Failed => Some(FinishDisposition::Failed),
            TransportOutcome::Cancelled => unreachable!("cancelled checked above"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_is_closed_and_default_is_exact() {
        assert_eq!(Model::parse(DEFAULT_MODEL), Some(Model::Nemotron));
        assert_eq!(Model::parse("meta/muse-glimmer-30b"), Some(Model::Muse));
        assert_eq!(Model::parse("attacker/unknown"), None);
    }

    #[test]
    fn repeated_input_has_same_decision_and_one_terminal() {
        let mut state = SessionState::default();
        let first = state.reserve("c_1", "r_1", Model::Nemotron, "hola".to_owned());
        assert!(first.is_ok());
        assert!(state.mark_streaming("c_1", "r_1"));
        assert!(state.cancel("r_1"));
        assert_eq!(
            state.finish("c_1", "r_1", TransportOutcome::Failed),
            Some(FinishDisposition::Cancelled)
        );
        assert_eq!(state.finish("c_1", "r_1", TransportOutcome::Failed), None);
        assert_eq!(
            state
                .reserve("c_1", "r_1", Model::Nemotron, "hola".to_owned())
                .err(),
            Some(AdmissionError::RequestReplayed)
        );
        let mut other = SessionState::default();
        assert!(other
            .reserve("c_1", "r_1", Model::Nemotron, "hola".to_owned())
            .is_ok());
        assert!(other.mark_streaming("c_1", "r_1"));
        assert!(other.cancel("r_1"));
        assert_eq!(
            other.finish("c_1", "r_1", TransportOutcome::Failed),
            Some(FinishDisposition::Cancelled)
        );
        assert_eq!(other.finish("c_1", "r_1", TransportOutcome::Failed), None);
    }

    #[test]
    fn model_context_is_separated_and_bounded() {
        let mut state = SessionState::default();
        state
            .reserve("c", "r1", Model::Nemotron, "first".into())
            .unwrap();
        state.finish("c", "r1", TransportOutcome::Completed("answer".into()));
        let muse = state
            .reserve("c", "r2", Model::Muse, "second".into())
            .unwrap();
        assert_eq!(muse.messages.len(), 1);
        state.finish("c", "r2", TransportOutcome::Failed);
        let nemotron = state
            .reserve("c", "r3", Model::Nemotron, "third".into())
            .unwrap();
        assert_eq!(nemotron.messages.len(), 3);
        assert_eq!(nemotron.messages[1].content, "answer");
        assert_eq!(
            state
                .reserve("c", "r4", Model::Nemotron, "busy".into())
                .err(),
            Some(AdmissionError::Busy)
        );
    }

    #[test]
    fn forged_conversation_ids_cannot_inflate_concurrent_external_calls() {
        let mut state = SessionState::default();
        state
            .reserve("c0", "r0", Model::Nemotron, "send".into())
            .unwrap();
        assert_eq!(
            state
                .reserve("c1", "r1", Model::Muse, "parallel".into())
                .err(),
            Some(AdmissionError::Busy)
        );
        assert_eq!(state.active_count(), 1);
        state.finish("c0", "r0", TransportOutcome::Completed("reply".into()));
        for i in 1..MAX_CONVERSATIONS {
            let conversation = format!("c{i}");
            let request = format!("r{i}");
            state
                .reserve(&conversation, &request, Model::Nemotron, "send".into())
                .unwrap();
            state.finish(
                &conversation,
                &request,
                TransportOutcome::Completed("reply".into()),
            );
        }
        assert_eq!(
            state
                .reserve("new_33", "r33", Model::Nemotron, "send".into())
                .err(),
            Some(AdmissionError::SessionLimit)
        );

        let mut failed = SessionState::default();
        for i in 0..MAX_CONVERSATIONS {
            let conversation = format!("failed_c{i}");
            let request = format!("failed_r{i}");
            failed
                .reserve(&conversation, &request, Model::Muse, "send".into())
                .unwrap();
            failed.finish(&conversation, &request, TransportOutcome::Failed);
        }
        assert_eq!(
            failed
                .reserve("other", "new", Model::Muse, "send".into())
                .err(),
            Some(AdmissionError::SessionLimit)
        );
    }

    #[test]
    fn cancellation_and_completion_are_linearized_by_one_host_lock() {
        let mut cancelled_first = SessionState::default();
        cancelled_first
            .reserve("c", "r1", Model::Nemotron, "prompt".into())
            .unwrap();
        assert!(cancelled_first.cancel("r1"));
        assert!(!cancelled_first.mark_streaming("c", "r1"));
        assert_eq!(
            cancelled_first.finish("c", "r1", TransportOutcome::Completed("late answer".into())),
            Some(FinishDisposition::Cancelled)
        );
        let next = cancelled_first
            .reserve("c", "r2", Model::Nemotron, "new prompt".into())
            .unwrap();
        assert_eq!(next.messages.len(), 1, "cancelled answer must not persist");

        let mut finished_first = SessionState::default();
        finished_first
            .reserve("c", "r1", Model::Nemotron, "prompt".into())
            .unwrap();
        assert_eq!(
            finished_first.finish("c", "r1", TransportOutcome::Completed("answer".into())),
            Some(FinishDisposition::Completed)
        );
        assert!(
            !finished_first.cancel("r1"),
            "late cancellation must not be acknowledged"
        );
        let next = finished_first
            .reserve("c", "r2", Model::Nemotron, "new prompt".into())
            .unwrap();
        assert_eq!(next.messages.len(), 3);
    }
}
