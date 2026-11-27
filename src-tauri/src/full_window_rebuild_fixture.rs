//! Text-only, simulated continuation. Authority remains with the existing route;
//! this crate does not write history or consume a user's consent.
use super::*;

struct Rebuild {
    ordinary: Prepared,
    next_summary: Prepared,
}
struct ProposedContext<'a> {
    summary_text: &'a str,
    retained: &'a [Turn<'a>],
    pending: &'a str,
    next_prefix_len: usize,
    summary_output: u64,
}
fn rebuild(
    original: &Prepared,
    selected_prefix: &Prepared,
    prefix_count: SimulatedCount,
    proposed: ProposedContext<'_>,
) -> Result<Rebuild, Failure> {
    // This still requires a separate authenticated completed/settled summary;
    // the fixture caller provides simulated summary data.
    summary_room(original, selected_prefix, prefix_count)?;
    let scope = Scope {
        revision: original
            .scope
            .revision
            .checked_add(1)
            .ok_or(Failure::Limit)?,
        ..original.scope
    };
    let ordinary = prepare_with_summary(
        scope,
        proposed.retained,
        Intent::Continue(proposed.pending),
        original.output,
        Some(proposed.summary_text),
    )?;
    if original.pending_digest != ordinary.pending_digest
        || original.turn_digests[selected_prefix.turn_digests.len()..] != ordinary.turn_digests
    {
        return Err(Failure::PrefixMismatch);
    }
    if !(1..proposed.retained.len()).contains(&proposed.next_prefix_len) {
        return Err(Failure::PrefixMismatch);
    }
    let next_summary = prepare_with_summary(
        scope,
        &proposed.retained[..proposed.next_prefix_len],
        Intent::CompactEligiblePrefix,
        proposed.summary_output,
        Some(proposed.summary_text),
    )?;
    Ok(Rebuild {
        ordinary,
        next_summary,
    })
}

impl Rebuild {
    fn check_fresh_counts(
        &self,
        ordinary_count: SimulatedCount,
        next_summary_count: SimulatedCount,
        margin: VerifiedMargin,
    ) -> Result<(), Failure> {
        // Neither a zero answer margin nor a non-fitting next summarization
        // qualifies this prospective revision.
        if plan_next(&self.ordinary, ordinary_count, Some(margin))? == Step::PrepareSummary {
            return Err(Failure::HeadroomUnverified);
        }
        summary_room(&self.ordinary, &self.next_summary, next_summary_count)?;
        Ok(())
    }
}

fn prep_count(prepared: &Prepared, tokens: u64) -> SimulatedCount {
    prepared
        .simulated_reply(count_json(tokens).as_bytes())
        .unwrap()
}
const PENDING: &str = "Continue retaining the existing decisions.";
const SUMMARY: &str = "Earlier confirmed decision: use the synthetic Atlas name.";
const MARGIN: u64 = 4_096;
fn margin(p: &Prepared) -> VerifiedMargin {
    VerifiedMargin {
        ordinary_binding: p.binding,
        minimum_free: MARGIN,
    }
}
fn proposal<'a>(retained: &'a [Turn<'a>]) -> ProposedContext<'a> {
    ProposedContext {
        summary_text: SUMMARY,
        retained,
        pending: PENDING,
        next_prefix_len: 1,
        summary_output: 1024,
    }
}
fn fixture(history: &[Turn<'_>]) -> (Prepared, Prepared) {
    let original = prepare(scope(), history, Intent::Continue(PENDING), MAX_OUTPUT).unwrap();
    let summary = prepare(
        scope(),
        &history[..history.len() - 2],
        Intent::CompactEligiblePrefix,
        1024,
    )
    .unwrap();
    (original, summary)
}
fn history() -> [Turn<'static>; 3] {
    [
        Turn {
            user: "Original",
            assistant: "Old fact",
        },
        Turn {
            user: "Recent A",
            assistant: "A retained",
        },
        Turn {
            user: "Recent B",
            assistant: "B retained",
        },
    ]
}

#[test]
fn sequence_crosses_legacy_limits_then_compacts_at_the_synthetic_boundary() {
    let text = "á界".repeat(5_000);
    let prior: Vec<_> = (0..24)
        .map(|_| Turn {
            user: "Original",
            assistant: &text,
        })
        .collect();
    assert!(prior.len() > 20 && text.len() * prior.len() > 512 * 1024);
    let (original, prefix) = fixture(&prior);
    let boundary = WINDOW - original.output - MARGIN;
    assert_eq!(
        plan_next(
            &original,
            prep_count(&original, boundary - 1),
            Some(margin(&original))
        ),
        Ok(Step::Continue {
            remaining: MARGIN + 1
        })
    );
    assert_eq!(
        plan_next(
            &original,
            prep_count(&original, boundary),
            Some(margin(&original))
        ),
        Ok(Step::PrepareSummary)
    );
    let next = rebuild(
        &original,
        &prefix,
        prep_count(&prefix, 700_000),
        proposal(&prior[22..]),
    )
    .unwrap();
    assert_eq!(
        next.check_fresh_counts(
            prep_count(&next.ordinary, 25_000),
            prep_count(&next.next_summary, 13_000),
            margin(&next.ordinary)
        ),
        Ok(())
    );
    assert_eq!(prior.len(), 24);
    assert_eq!(original.turn_digests.len(), 24);
    assert_eq!(next.ordinary.turn_digests.len(), 2);
    assert_eq!(next.ordinary.scope.revision, original.scope.revision + 1);
    for request in [&next.ordinary, &next.next_summary] {
        let count: serde_json::Value = serde_json::from_slice(&request.count_body).unwrap();
        let sent: serde_json::Value = serde_json::from_slice(&request.generation_body).unwrap();
        assert_eq!(count["input"], sent["input"]);
        assert_eq!(sent["input"][0]["role"], "assistant");
        assert_eq!(sent["input"][0]["content"][0]["text"], SUMMARY);
        assert_ne!(sent["instructions"], SUMMARY);
    }
}

#[test]
fn changed_tail_or_pending_prompt_cannot_reuse_the_candidate() {
    let h = history();
    let (original, prefix) = fixture(&h);
    let result = |p| rebuild(&original, &prefix, prep_count(&prefix, 100), p).err();
    assert_eq!(result(proposal(&h[..2])), Some(Failure::PrefixMismatch));
    assert_eq!(
        result(ProposedContext {
            pending: "Different",
            ..proposal(&h[1..])
        }),
        Some(Failure::PrefixMismatch)
    );
}

#[test]
fn require_both_new_counts_and_reject_loss_of_summary_on_next_cycle() {
    let h = history();
    let (original, prefix) = fixture(&h);
    let next = rebuild(
        &original,
        &prefix,
        prep_count(&prefix, 100),
        proposal(&h[1..]),
    )
    .unwrap();
    let o = &next.ordinary;
    let s = &next.next_summary;
    let fit = |a, b| next.check_fresh_counts(a, b, margin(o));
    assert_eq!(
        fit(prep_count(&original, 50), prep_count(s, 30)),
        Err(Failure::Stale)
    );
    assert_eq!(
        fit(prep_count(o, 100), prep_count(&prefix, 30)),
        Err(Failure::Stale)
    );
    assert_eq!(
        fit(prep_count(o, WINDOW - o.output - MARGIN), prep_count(s, 30)),
        Err(Failure::HeadroomUnverified)
    );
    assert_eq!(
        fit(prep_count(o, 100), prep_count(s, WINDOW)),
        Err(Failure::Limit)
    );
    for wrong in [None, Some("Substituted first summary")] {
        let changed = prepare_with_summary(
            o.scope,
            &h[1..2],
            Intent::CompactEligiblePrefix,
            1024,
            wrong,
        )
        .unwrap();
        assert_eq!(
            summary_room(o, &changed, prep_count(&changed, 30)),
            Err(Failure::PrefixMismatch)
        );
    }
}

#[test]
fn a_later_compaction_carries_old_summary_into_new_summary_and_keeps_newest_turns() {
    let prior = [
        Turn {
            user: "Recently compacted A",
            assistant: "Retained after first summary",
        },
        Turn {
            user: "Recently compacted B",
            assistant: "Previously retained",
        },
        Turn {
            user: "Newest C",
            assistant: "New material",
        },
        Turn {
            user: "Newest D",
            assistant: "More new material",
        },
    ];
    let scope = Scope {
        revision: scope().revision + 1,
        ..scope()
    };
    let original = prepare_with_summary(
        scope,
        &prior,
        Intent::Continue(PENDING),
        MAX_OUTPUT,
        Some(SUMMARY),
    )
    .unwrap();
    let selected = prepare_with_summary(
        scope,
        &prior[..2],
        Intent::CompactEligiblePrefix,
        1024,
        Some(SUMMARY),
    )
    .unwrap();
    let second_summary = "Retain Atlas and newer decisions A and B.";
    let selected_wire: serde_json::Value = serde_json::from_slice(&selected.count_body).unwrap();
    assert_eq!(selected_wire["input"][0]["content"][0]["text"], SUMMARY);
    let next = rebuild(
        &original,
        &selected,
        prep_count(&selected, 800_000),
        ProposedContext {
            summary_text: second_summary,
            ..proposal(&prior[2..])
        },
    )
    .unwrap();
    let ordinary_wire: serde_json::Value =
        serde_json::from_slice(&next.ordinary.count_body).unwrap();
    assert_eq!(next.ordinary.scope.revision, scope.revision + 1);
    assert_eq!(next.ordinary.turn_digests, original.turn_digests[2..]);
    assert_eq!(
        ordinary_wire["input"][0]["content"][0]["text"],
        second_summary
    );
    assert_eq!(ordinary_wire["input"].as_array().unwrap().len(), 6);
    assert_eq!(
        next.check_fresh_counts(
            prep_count(&next.ordinary, 600_000),
            prep_count(&next.next_summary, 300_000),
            margin(&next.ordinary)
        ),
        Ok(())
    );
}
