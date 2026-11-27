//! Pure recovery fixture. Ledger, staged output and committed revision here
//! represent synthetic trusted observations, not a persisted format or API.
use super::{Failure, Prepared, Purpose, Scope, Sha256};
use sha2::Digest;

type RequestId = [u8; 16];
type Hash = [u8; 32];

#[derive(Clone, Copy)]
struct Operation {
    summary_id: RequestId,
    summary_binding: Hash,
    answer_id: RequestId,
    base: Scope,
    next_revision: u64,
}
impl Operation {
    fn new(
        summary: &Prepared,
        summary_id: RequestId,
        answer_id: RequestId,
    ) -> Result<Self, Failure> {
        if summary.purpose != Purpose::Summary {
            return Err(Failure::WrongPurpose);
        }
        if summary_id == [0; 16] || answer_id == [0; 16] || summary_id == answer_id {
            return Err(Failure::Input);
        }
        Ok(Self {
            summary_id,
            summary_binding: summary.binding,
            answer_id,
            base: summary.scope,
            next_revision: summary
                .scope
                .revision
                .checked_add(1)
                .ok_or(Failure::Limit)?,
        })
    }
}

#[derive(Clone, Copy)]
enum Ledger {
    NotDispatched(RequestId),
    DispatchedUnresolved(RequestId),
    ReconciledWithoutTerminalUsage(RequestId),
    Settled(RequestId),
}

struct Candidate<'a> {
    summary_id: RequestId,
    summary_binding: Hash,
    terminal_text: &'a str,
    staged_digest: Hash,
}

#[derive(Clone, Copy)]
struct Checkpoint {
    scope: Scope,
    summary_id: RequestId,
    terminal_digest: Hash,
    answer_id_seen: bool,
}

#[derive(Debug, PartialEq, Eq)]
enum Recovery {
    NoAutomaticRetry,
    AwaitLedgerResolution,
    PreservePriorContext,
    RecountProposedContext,
    PrepareIndependentAnswerReview(RequestId),
    DeferToExistingAnswer,
}

fn recover(
    op: Operation,
    ledger: Ledger,
    current: Scope,
    staged: Option<Candidate<'_>>,
    checkpoint: Option<Checkpoint>,
) -> Result<Recovery, Failure> {
    let id = match ledger {
        Ledger::NotDispatched(id)
        | Ledger::DispatchedUnresolved(id)
        | Ledger::ReconciledWithoutTerminalUsage(id)
        | Ledger::Settled(id) => id,
    };
    if id != op.summary_id
        || current.chat != op.base.chat
        || current.credential_generation != op.base.credential_generation
        || (current.revision != op.base.revision && current.revision != op.next_revision)
    {
        return Err(Failure::Stale);
    }
    if checkpoint.is_none() && current != op.base {
        return Err(Failure::Stale);
    }
    if !matches!(ledger, Ledger::Settled(_)) && checkpoint.is_some() {
        return Err(Failure::Receipt);
    }
    match ledger {
        Ledger::NotDispatched(_) => return Ok(Recovery::NoAutomaticRetry),
        Ledger::DispatchedUnresolved(_) => return Ok(Recovery::AwaitLedgerResolution),
        Ledger::ReconciledWithoutTerminalUsage(_) => return Ok(Recovery::PreservePriorContext),
        Ledger::Settled(_) => {}
    }
    let Some(staged) = staged else {
        return if checkpoint.is_none() {
            Ok(Recovery::PreservePriorContext)
        } else {
            Err(Failure::Receipt)
        };
    };
    if staged.summary_id != id
        || staged.summary_binding != op.summary_binding
        || staged.terminal_text.len() > 512 * 1024
        || staged.terminal_text.trim().is_empty()
    {
        return Err(Failure::Receipt);
    }
    let digest: Hash = Sha256::digest(staged.terminal_text.as_bytes()).into();
    if digest != staged.staged_digest {
        return Err(Failure::Receipt);
    }
    if let Some(commit) = checkpoint {
        if commit.scope != current
            || current.revision != op.next_revision
            || commit.summary_id != id
            || commit.terminal_digest != digest
        {
            return Err(Failure::Stale);
        }
        if commit.answer_id_seen {
            return Ok(Recovery::DeferToExistingAnswer);
        }
        return Ok(Recovery::PrepareIndependentAnswerReview(op.answer_id));
    }
    Ok(Recovery::RecountProposedContext)
}

#[test]
fn crash_states_never_reissue_a_summary_or_reserve_an_answer() {
    let op = operation();
    let id = op.summary_id;
    for (ledger, expected) in [
        (Ledger::NotDispatched(id), Recovery::NoAutomaticRetry),
        (
            Ledger::DispatchedUnresolved(id),
            Recovery::AwaitLedgerResolution,
        ),
        (
            Ledger::ReconciledWithoutTerminalUsage(id),
            Recovery::PreservePriorContext,
        ),
        (Ledger::Settled(id), Recovery::RecountProposedContext),
    ] {
        assert_eq!(
            recover(op, ledger, op.base, Some(staged(op)), None),
            Ok(expected)
        );
    }
    assert_eq!(
        recover(op, Ledger::Settled(id), op.base, None, None),
        Ok(Recovery::PreservePriorContext)
    );
}

#[test]
fn settled_output_and_current_checkpoint_allow_only_a_fresh_answer_review() {
    let op = operation();
    let current = Scope {
        revision: op.next_revision,
        ..op.base
    };
    let commit = Checkpoint {
        scope: current,
        summary_id: op.summary_id,
        terminal_digest: staged(op).staged_digest,
        answer_id_seen: false,
    };
    let settled = Ledger::Settled(op.summary_id);
    assert_eq!(
        recover(op, settled, current, Some(staged(op)), Some(commit)),
        Ok(Recovery::PrepareIndependentAnswerReview(op.answer_id))
    );
    assert_eq!(
        recover(
            op,
            settled,
            current,
            Some(staged(op)),
            Some(Checkpoint {
                answer_id_seen: true,
                ..commit
            })
        ),
        Ok(Recovery::DeferToExistingAnswer)
    );
    assert_eq!(
        recover(
            op,
            Ledger::DispatchedUnresolved(op.summary_id),
            current,
            Some(staged(op)),
            Some(commit)
        ),
        Err(Failure::Receipt)
    );
    assert_eq!(
        recover(op, settled, current, None, Some(commit)),
        Err(Failure::Receipt)
    );
    assert_eq!(
        recover(op, settled, current, Some(staged(op)), None),
        Err(Failure::Stale)
    );
}

#[test]
fn rejects_changed_output_revision_and_accounting_identity() {
    let op = operation();
    let forged = Candidate {
        terminal_text: "altered",
        ..staged(op)
    };
    let stale = Scope {
        credential_generation: op.base.credential_generation + 1,
        ..op.base
    };
    let settled = Ledger::Settled(op.summary_id);
    assert_eq!(
        recover(op, settled, op.base, Some(forged), None),
        Err(Failure::Receipt)
    );
    assert_eq!(
        recover(op, settled, stale, Some(staged(op)), None),
        Err(Failure::Stale)
    );
    assert_eq!(
        recover(
            op,
            Ledger::Settled(op.answer_id),
            op.base,
            Some(staged(op)),
            None
        ),
        Err(Failure::Stale)
    );
    let prepared = summary();
    assert!(matches!(
        Operation::new(&prepared, op.summary_id, op.summary_id),
        Err(Failure::Input)
    ));
}

fn summary() -> Prepared {
    super::prepare(
        super::scope(),
        &[super::Turn {
            user: "Old decision",
            assistant: "Synthetic receipt",
        }],
        super::Intent::CompactEligiblePrefix,
        1024,
    )
    .unwrap()
}

fn operation() -> Operation {
    Operation::new(&summary(), [11; 16], [22; 16]).unwrap()
}

fn staged(op: Operation) -> Candidate<'static> {
    let text = "Summary of the old decision.";
    Candidate {
        summary_id: op.summary_id,
        summary_binding: op.summary_binding,
        terminal_text: text,
        staged_digest: Sha256::digest(text.as_bytes()).into(),
    }
}
