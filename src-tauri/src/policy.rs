//! Host-owned, versioned admission and per-conversation state.
//! Only a successfully completed ordinary answer is eligible for continuation.
//! Project-bearing requests and derived answers require fresh disclosure consent.

use serde::Serialize;
use std::collections::{HashMap, HashSet};
use tokio::sync::watch;

pub const POLICY_VERSION: u32 = 4;
pub const DEFAULT_MODEL: &str = "muse-spark-1.3-contributor";
pub const MAX_MESSAGE_BYTES: usize = 32 * 1024;
pub const MAX_CONTEXT_BYTES: usize = 512 * 1024;
const MAX_TURNS: usize = 20;
const MAX_CONVERSATIONS: usize = 32;
const MAX_SESSION_REQUESTS: usize = 64;
const MAX_ACTIVE_REQUESTS: usize = 1;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Model {
    MuseSpark,
}

impl Model {
    pub fn parse(id: &str) -> Option<Self> {
        match id {
            DEFAULT_MODEL => Some(Self::MuseSpark),
            _ => None,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::MuseSpark => DEFAULT_MODEL,
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
    retain_history: bool,
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
        self.reserve_with_retention(conversation_id, request_id, model, prompt, true)
    }

    // Project dispatch reviews the exact outbound request and never retains
    // the project prompt or derived answer in ordinary continuation history.
    #[cfg(test)]
    pub(crate) fn reserve_ephemeral(
        &mut self,
        conversation_id: &str,
        request_id: &str,
        model: Model,
        prompt: String,
    ) -> Result<Admission, AdmissionError> {
        self.reserve_with_retention(conversation_id, request_id, model, prompt, false)
    }

    fn reserve_with_retention(
        &mut self,
        conversation_id: &str,
        request_id: &str,
        model: Model,
        prompt: String,
        retain_history: bool,
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
                retain_history,
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

    /// Restore completed host-validated text only; never restore requests,
    /// approvals, cancellation channels, replay IDs, grants or finances.
    pub fn restore_completed(
        &mut self,
        conversation_id: &str,
        turns: &[(String, String)],
    ) -> Result<(), AdmissionError> {
        if !valid_id(conversation_id) {
            return Err(AdmissionError::InvalidInput);
        }
        if !self.active.is_empty() {
            return Err(AdmissionError::Busy);
        }
        if !self.conversation_ids.contains(conversation_id)
            && self.conversation_ids.len() >= MAX_CONVERSATIONS
        {
            return Err(AdmissionError::SessionLimit);
        }
        if turns.len() > MAX_TURNS {
            return Err(AdmissionError::ContextLimit);
        }
        let mut total = 0usize;
        for (prompt, answer) in turns {
            if !valid_prompt(prompt)
                || answer.trim().is_empty()
                || answer.contains('\0')
                || answer.len() > MAX_CONTEXT_BYTES
            {
                return Err(AdmissionError::InvalidInput);
            }
            total = total
                .checked_add(prompt.len())
                .and_then(|n| n.checked_add(answer.len()))
                .filter(|n| *n <= MAX_CONTEXT_BYTES)
                .ok_or(AdmissionError::ContextLimit)?;
        }
        self.completed.insert(
            (conversation_id.to_owned(), DEFAULT_MODEL),
            turns
                .iter()
                .map(|(prompt, answer)| CompletedTurn {
                    prompt: prompt.clone(),
                    answer: answer.clone(),
                })
                .collect(),
        );
        self.conversation_ids.insert(conversation_id.to_owned());
        Ok(())
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
                let prior = self
                    .completed
                    .get(&(conversation_id.to_owned(), active.model.id()));
                let total = prior
                    .map_or(0, |turns| {
                        turns.iter().fold(0usize, |sum, turn| {
                            sum.saturating_add(turn.prompt.len())
                                .saturating_add(turn.answer.len())
                        })
                    })
                    .saturating_add(active.prompt.len())
                    .saturating_add(answer.len());
                if answer.trim().is_empty() || answer.contains('\0') || total > MAX_CONTEXT_BYTES {
                    return Some(FinishDisposition::Failed);
                }
                if active.retain_history {
                    let key = (conversation_id.to_string(), active.model.id());
                    self.completed.entry(key).or_default().push(CompletedTurn {
                        prompt: active.prompt,
                        answer,
                    });
                }
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
        assert_eq!(Model::parse(DEFAULT_MODEL), Some(Model::MuseSpark));
        for retired in [
            "meta/muse-glimmer-30b",
            "nvidia/nemotron-3.5-lightning-30b-a3b",
            "muse-spark-1.3",
            "muse-spark-1.2-contributor",
        ] {
            assert_eq!(Model::parse(retired), None);
        }
        assert_eq!(Model::parse("attacker/unknown"), None);
    }

    #[test]
    fn repeated_input_has_same_decision_and_one_terminal() {
        let mut state = SessionState::default();
        let first = state.reserve("c_1", "r_1", Model::MuseSpark, "hola".to_owned());
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
                .reserve("c_1", "r_1", Model::MuseSpark, "hola".to_owned())
                .err(),
            Some(AdmissionError::RequestReplayed)
        );
        let mut other = SessionState::default();
        assert!(other
            .reserve("c_1", "r_1", Model::MuseSpark, "hola".to_owned())
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
    fn single_model_context_is_retained_and_bounded() {
        let mut state = SessionState::default();
        state
            .reserve("c", "r1", Model::MuseSpark, "first".into())
            .unwrap();
        state.finish("c", "r1", TransportOutcome::Completed("answer".into()));
        let muse = state
            .reserve("c", "r2", Model::MuseSpark, "second".into())
            .unwrap();
        assert_eq!(muse.messages.len(), 3);
        state.finish("c", "r2", TransportOutcome::Failed);
        let again = state
            .reserve("c", "r3", Model::MuseSpark, "third".into())
            .unwrap();
        assert_eq!(again.messages.len(), 3);
        assert_eq!(again.messages[1].content, "answer");
        assert_eq!(
            state
                .reserve("c", "r4", Model::MuseSpark, "busy".into())
                .err(),
            Some(AdmissionError::Busy)
        );
    }

    #[test]
    fn ephemeral_project_turn_and_derived_answer_never_enter_continuation() {
        {
            let model = Model::MuseSpark;
            let mut state = SessionState::default();
            state
                .reserve("c", "ordinary", model, "ordinary prompt".into())
                .unwrap();
            assert_eq!(
                state.finish(
                    "c",
                    "ordinary",
                    TransportOutcome::Completed("ordinary answer".into())
                ),
                Some(FinishDisposition::Completed)
            );
            let project = state
                .reserve_ephemeral("c", "project", model, "RESTRICTED_PROJECT_CONTENT".into())
                .unwrap();
            assert_eq!(project.messages.len(), 3);
            assert_eq!(project.messages[0].content, "ordinary prompt");
            assert_eq!(project.messages[1].content, "ordinary answer");
            assert_eq!(project.messages[2].content, "RESTRICTED_PROJECT_CONTENT");
            assert_eq!(
                state.finish(
                    "c",
                    "project",
                    TransportOutcome::Completed("DERIVED_PRIVATE_ANSWER".into())
                ),
                Some(FinishDisposition::Completed)
            );
            assert_eq!(state.active_count(), 0);
            assert_eq!(
                state
                    .completed
                    .get(&("c".to_owned(), model.id()))
                    .unwrap()
                    .len(),
                1
            );
            let next = state
                .reserve("c", "next", model, "ordinary follow-up".into())
                .unwrap();
            assert_eq!(next.messages.len(), 3);
            assert_eq!(next.messages[0].content, "ordinary prompt");
            assert_eq!(next.messages[1].content, "ordinary answer");
            assert_eq!(next.messages[2].content, "ordinary follow-up");
            assert!(next.messages.iter().all(|message| {
                !message.content.contains("RESTRICTED_PROJECT_CONTENT")
                    && !message.content.contains("DERIVED_PRIVATE_ANSWER")
            }));
        }
    }

    #[test]
    fn ephemeral_reservations_preserve_admission_replay_and_terminal_denials() {
        let mut state = SessionState::default();
        assert_eq!(
            state
                .reserve_ephemeral("c", "invalid", Model::MuseSpark, "invalid\0prompt".into())
                .err(),
            Some(AdmissionError::InvalidInput)
        );
        assert_eq!(
            state
                .reserve_ephemeral(
                    "c",
                    "oversized",
                    Model::MuseSpark,
                    "x".repeat(MAX_MESSAGE_BYTES + 1)
                )
                .err(),
            Some(AdmissionError::InvalidInput)
        );
        state
            .reserve_ephemeral(
                "c",
                "project",
                Model::MuseSpark,
                "RESTRICTED_PROJECT_CONTENT".into(),
            )
            .unwrap();
        assert_eq!(
            state
                .reserve("other", "parallel", Model::MuseSpark, "ordinary".into())
                .err(),
            Some(AdmissionError::Busy)
        );
        assert!(state.cancel("project"));
        assert_eq!(
            state.finish(
                "c",
                "project",
                TransportOutcome::Completed("DERIVED_PRIVATE_ANSWER".into())
            ),
            Some(FinishDisposition::Cancelled)
        );
        assert_eq!(state.finish("c", "project", TransportOutcome::Failed), None);
        assert_eq!(
            state
                .reserve_ephemeral("c", "project", Model::MuseSpark, "replayed".into())
                .err(),
            Some(AdmissionError::RequestReplayed)
        );
        state
            .reserve_ephemeral(
                "c",
                "failed",
                Model::MuseSpark,
                "RESTRICTED_FAILED_CONTENT".into(),
            )
            .unwrap();
        assert_eq!(
            state.finish("c", "failed", TransportOutcome::Failed),
            Some(FinishDisposition::Failed)
        );
        let next = state
            .reserve("c", "next", Model::MuseSpark, "ordinary".into())
            .unwrap();
        assert_eq!(next.messages.len(), 1);
        assert_eq!(next.messages[0].content, "ordinary");
        assert!(state.completed.is_empty());
    }

    #[test]
    fn forged_conversation_ids_cannot_inflate_concurrent_external_calls() {
        let mut state = SessionState::default();
        state
            .reserve("c0", "r0", Model::MuseSpark, "send".into())
            .unwrap();
        assert_eq!(
            state
                .reserve("c1", "r1", Model::MuseSpark, "parallel".into())
                .err(),
            Some(AdmissionError::Busy)
        );
        assert_eq!(state.active_count(), 1);
        state.finish("c0", "r0", TransportOutcome::Completed("reply".into()));
        for i in 1..MAX_CONVERSATIONS {
            let conversation = format!("c{i}");
            let request = format!("r{i}");
            state
                .reserve(&conversation, &request, Model::MuseSpark, "send".into())
                .unwrap();
            state.finish(
                &conversation,
                &request,
                TransportOutcome::Completed("reply".into()),
            );
        }
        assert_eq!(
            state
                .reserve("new_33", "r33", Model::MuseSpark, "send".into())
                .err(),
            Some(AdmissionError::SessionLimit)
        );

        let mut failed = SessionState::default();
        for i in 0..MAX_CONVERSATIONS {
            let conversation = format!("failed_c{i}");
            let request = format!("failed_r{i}");
            failed
                .reserve(&conversation, &request, Model::MuseSpark, "send".into())
                .unwrap();
            failed.finish(&conversation, &request, TransportOutcome::Failed);
        }
        assert_eq!(
            failed
                .reserve("other", "new", Model::MuseSpark, "send".into())
                .err(),
            Some(AdmissionError::SessionLimit)
        );
    }

    #[test]
    fn cancellation_and_completion_are_linearized_by_one_host_lock() {
        let mut cancelled_first = SessionState::default();
        cancelled_first
            .reserve("c", "r1", Model::MuseSpark, "prompt".into())
            .unwrap();
        assert!(cancelled_first.cancel("r1"));
        assert!(!cancelled_first.mark_streaming("c", "r1"));
        assert_eq!(
            cancelled_first.finish("c", "r1", TransportOutcome::Completed("late answer".into())),
            Some(FinishDisposition::Cancelled)
        );
        let next = cancelled_first
            .reserve("c", "r2", Model::MuseSpark, "new prompt".into())
            .unwrap();
        assert_eq!(next.messages.len(), 1, "cancelled answer must not persist");

        let mut finished_first = SessionState::default();
        finished_first
            .reserve("c", "r1", Model::MuseSpark, "prompt".into())
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
            .reserve("c", "r2", Model::MuseSpark, "new prompt".into())
            .unwrap();
        assert_eq!(next.messages.len(), 3);
    }
    #[test]
    fn restore_completed_never_restores_authority_or_resets_replay_ids() {
        let mut state = SessionState::default();
        state
            .restore_completed("saved", &[("question".into(), "answer".into())])
            .unwrap();
        assert_eq!(state.active_count(), 0);
        assert_eq!(
            state
                .reserve("saved", "once", Model::MuseSpark, "follow-up".into())
                .unwrap()
                .messages
                .len(),
            3
        );
        assert_eq!(
            state.restore_completed("other", &[]),
            Err(AdmissionError::Busy)
        );
        state.finish("saved", "once", TransportOutcome::Failed);
        state.restore_completed("saved", &[]).unwrap();
        assert_eq!(
            state
                .reserve("saved", "once", Model::MuseSpark, "again".into())
                .err(),
            Some(AdmissionError::RequestReplayed)
        );
        assert_eq!(
            state.restore_completed("../bad", &[]),
            Err(AdmissionError::InvalidInput)
        );
        assert_eq!(
            state.restore_completed("saved", &vec![("p".into(), "a".into()); MAX_TURNS + 1]),
            Err(AdmissionError::ContextLimit)
        );
        assert_eq!(
            state.restore_completed("saved", &[("p".into(), "x".repeat(MAX_CONTEXT_BYTES))]),
            Err(AdmissionError::ContextLimit)
        );
        assert_eq!(
            state.restore_completed("saved", &[("p".into(), "a\0b".into())]),
            Err(AdmissionError::InvalidInput)
        );
    }

    #[test]
    fn utf8_input_and_restored_conversation_capacity_are_bounded() {
        assert!(valid_prompt(&"é".repeat(MAX_MESSAGE_BYTES / 2)));
        assert!(!valid_prompt(&"é".repeat(MAX_MESSAGE_BYTES / 2 + 1)));
        let mut state = SessionState::default();
        for i in 0..MAX_CONVERSATIONS {
            state.restore_completed(&format!("c{i}"), &[]).unwrap();
        }
        assert_eq!(
            state.restore_completed("extra", &[]),
            Err(AdmissionError::SessionLimit)
        );
        assert!(state.restore_completed("c0", &[]).is_ok());
    }
}
