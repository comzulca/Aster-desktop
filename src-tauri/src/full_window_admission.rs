//! Private, content-free capacity planning under the reviewed T-087 contract.
//!
//! Supplied counts and opaque bindings are synthetic/trusted-host seams, not
//! proof of canonical content, eligibility, authentication, disclosure, spend or
//! live counting. This module neither serializes input nor grants an append or
//! send. It does not prove preventive headroom or implement autocompaction.
//! Initially unlinked; a later reviewed root may link it only under cfg(test).

#![cfg(test)]

const MODEL_WINDOW: u64 = 1_048_576;
const INHERITED_OUTPUT_CEILING: u64 = 8_192;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Purpose {
    Ordinary,
    Summary,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CountTemplate {
    StatelessResponsesV1,
    // Negative-only fixture: a Responses count cannot qualify this template.
    CurrentChatV4,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ModelContract {
    // Another model is structurally unrepresentable, not parsed and rejected.
    MuseSpark13Contributor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ContextIdentity {
    conversation: [u8; 16],
    revision: u64,
    digest: [u8; 32],
    // An invalidation identity only; it is not authentication evidence.
    credential_generation: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CandidateIdentity {
    id: [u8; 16],
    digest: [u8; 32],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CandidatePhase {
    BeforeAppend,
    AlreadyAppended,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PendingCandidate {
    base: ContextIdentity,
    identity: CandidateIdentity,
    phase: CandidatePhase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct InputBinding {
    purpose: Purpose,
    template: CountTemplate,
    model: ModelContract,
    base: ContextIdentity,
    // Ordinary input binds its prospective candidate. A pre-append summary
    // binds only the older eligible base and must carry no candidate identity.
    candidate: Option<CandidateIdentity>,
    input_bytes: u64,
    input_sha256: [u8; 32],
    ordered_source_digest: [u8; 32],
    generation_parameters_digest: [u8; 32],
    output_reserve: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SuppliedExactCount {
    binding: InputBinding,
    input_tokens: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CountEvidence {
    expected: InputBinding,
    supplied: SuppliedExactCount,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OutputReserves {
    ordinary: u64,
    summary: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PlanningInputs {
    ordinary: CountEvidence,
    summary: Option<CountEvidence>,
    reserves: OutputReserves,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Denial {
    InvalidContext,
    InvalidCandidate,
    StaleContext,
    AlreadyAppended,
    InvalidReserve,
    InvalidBinding,
    StaleBinding,
    CandidateBindingMismatch,
    CountBindingMismatch,
    ArithmeticOverflow,
    SummaryMissing,
    SummaryCannotFit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WindowFit {
    Fits { total: u64 },
    Exceeds { total: u64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NextStep {
    // Capacity only. Future orchestration must revalidate the current identity
    // before any admission/commit; this copyable result is never a grant.
    OrdinaryFits {
        total: u64,
    },
    SummaryRequiredAndFits {
        ordinary_total: u64,
        summary_total: u64,
    },
}

fn nonzero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn valid_context(context: &ContextIdentity) -> bool {
    nonzero(&context.conversation)
        && context.revision != 0
        && nonzero(&context.digest)
        && context.credential_generation != 0
}

fn valid_candidate(candidate: &CandidateIdentity) -> bool {
    nonzero(&candidate.id) && nonzero(&candidate.digest)
}

fn valid_reserve(reserve: u64) -> bool {
    (1..=INHERITED_OUTPUT_CEILING).contains(&reserve)
}

fn validate_evidence(
    evidence: &CountEvidence,
    purpose: Purpose,
    reserve: u64,
) -> Result<(), Denial> {
    if !valid_reserve(reserve) {
        return Err(Denial::InvalidReserve);
    }
    let expected = &evidence.expected;
    let valid_candidate_shape = match (purpose, expected.candidate) {
        (Purpose::Ordinary, Some(candidate)) => valid_candidate(&candidate),
        (Purpose::Summary, None) => true,
        _ => false,
    };
    if expected.purpose != purpose
        || expected.template != CountTemplate::StatelessResponsesV1
        || !valid_context(&expected.base)
        || !valid_candidate_shape
        || expected.input_bytes == 0
        || !nonzero(&expected.input_sha256)
        || !nonzero(&expected.ordered_source_digest)
        || !nonzero(&expected.generation_parameters_digest)
        || expected.output_reserve != reserve
    {
        return Err(Denial::InvalidBinding);
    }
    // Equality includes model/template/purpose, all source/input/settings
    // identities and the output reserve, not just conversation or token count.
    if evidence.supplied.binding != *expected {
        return Err(Denial::CountBindingMismatch);
    }
    Ok(())
}

fn check_fit(
    evidence: &CountEvidence,
    purpose: Purpose,
    reserve: u64,
) -> Result<WindowFit, Denial> {
    validate_evidence(evidence, purpose, reserve)?;
    let window = match evidence.expected.model {
        ModelContract::MuseSpark13Contributor => MODEL_WINDOW,
    };
    let total = evidence
        .supplied
        .input_tokens
        .checked_add(reserve)
        .ok_or(Denial::ArithmeticOverflow)?;
    if total <= window {
        Ok(WindowFit::Fits { total })
    } else {
        Ok(WindowFit::Exceeds { total })
    }
}

fn plan_before_append(
    current: &ContextIdentity,
    candidate: &PendingCandidate,
    inputs: &PlanningInputs,
) -> Result<NextStep, Denial> {
    if !valid_context(current) || !valid_context(&candidate.base) {
        return Err(Denial::InvalidContext);
    }
    if !valid_candidate(&candidate.identity) {
        return Err(Denial::InvalidCandidate);
    }
    if candidate.base != *current {
        return Err(Denial::StaleContext);
    }
    if candidate.phase != CandidatePhase::BeforeAppend {
        return Err(Denial::AlreadyAppended);
    }
    if !valid_reserve(inputs.reserves.ordinary) || !valid_reserve(inputs.reserves.summary) {
        return Err(Denial::InvalidReserve);
    }
    if inputs.ordinary.expected.base != *current {
        return Err(Denial::StaleBinding);
    }
    if inputs.ordinary.expected.candidate != Some(candidate.identity) {
        return Err(Denial::CandidateBindingMismatch);
    }
    // Supplied optional summary evidence must also be well-bound even if the
    // ordinary request fits. Its arithmetic is used only when it is required.
    if let Some(summary) = &inputs.summary {
        if summary.expected.base != *current {
            return Err(Denial::StaleBinding);
        }
        validate_evidence(summary, Purpose::Summary, inputs.reserves.summary)?;
    }
    let ordinary_total = match check_fit(
        &inputs.ordinary,
        Purpose::Ordinary,
        inputs.reserves.ordinary,
    )? {
        WindowFit::Fits { total } => return Ok(NextStep::OrdinaryFits { total }),
        WindowFit::Exceeds { total } => total,
    };
    let summary = inputs.summary.as_ref().ok_or(Denial::SummaryMissing)?;
    match check_fit(summary, Purpose::Summary, inputs.reserves.summary)? {
        WindowFit::Fits { total } => Ok(NextStep::SummaryRequiredAndFits {
            ordinary_total,
            summary_total: total,
        }),
        WindowFit::Exceeds { .. } => Err(Denial::SummaryCannotFit),
    }
}

mod tests {
    use super::*;

    // Synthetic metadata only: these repeatable tags are not real hashes,
    // actual history, credentials, disclosure receipts or count-service proof.
    fn context() -> ContextIdentity {
        ContextIdentity {
            conversation: [1; 16],
            revision: 1,
            digest: [2; 32],
            credential_generation: 1,
        }
    }

    fn candidate() -> PendingCandidate {
        PendingCandidate {
            base: context(),
            identity: CandidateIdentity {
                id: [3; 16],
                digest: [4; 32],
            },
            phase: CandidatePhase::BeforeAppend,
        }
    }

    fn evidence(purpose: Purpose, tokens: u64, reserve: u64) -> CountEvidence {
        let expected = InputBinding {
            purpose,
            template: CountTemplate::StatelessResponsesV1,
            model: ModelContract::MuseSpark13Contributor,
            base: context(),
            candidate: match purpose {
                Purpose::Ordinary => Some(candidate().identity),
                Purpose::Summary => None,
            },
            input_bytes: 100,
            input_sha256: [5; 32],
            ordered_source_digest: [6; 32],
            generation_parameters_digest: [7; 32],
            output_reserve: reserve,
        };
        CountEvidence {
            expected,
            supplied: SuppliedExactCount {
                binding: expected,
                input_tokens: tokens,
            },
        }
    }

    fn inputs(ordinary_tokens: u64, summary_tokens: Option<u64>) -> PlanningInputs {
        PlanningInputs {
            ordinary: evidence(Purpose::Ordinary, ordinary_tokens, 8_192),
            summary: summary_tokens.map(|tokens| evidence(Purpose::Summary, tokens, 4_096)),
            reserves: OutputReserves {
                ordinary: 8_192,
                // Fixture only, not a production summary output default.
                summary: 4_096,
            },
        }
    }

    fn plan(inputs: &PlanningInputs) -> Result<NextStep, Denial> {
        plan_before_append(&context(), &candidate(), inputs)
    }

    #[test]
    fn closed_model_represents_only_the_verified_full_window() {
        // Exhaustive, single-variant matching is the model boundary. There is
        // no string/parser seam or pretend forged-model rejection fixture.
        let window = match ModelContract::MuseSpark13Contributor {
            ModelContract::MuseSpark13Contributor => MODEL_WINDOW,
        };
        assert_eq!(window, 1_048_576);
        assert_eq!(INHERITED_OUTPUT_CEILING, 8_192);
    }

    #[test]
    fn ordinary_exact_window_fits_without_adding_the_separate_summary_reserve() {
        assert_eq!(
            plan(&inputs(1_040_384, None)),
            Ok(NextStep::OrdinaryFits {
                total: MODEL_WINDOW
            })
        );
        assert_eq!(
            plan(&inputs(1_040_384, Some(1_044_480))),
            Ok(NextStep::OrdinaryFits {
                total: MODEL_WINDOW
            })
        );
    }

    #[test]
    fn ordinary_one_token_over_requires_an_independently_fitting_summary() {
        assert_eq!(
            plan(&inputs(1_040_385, Some(1_044_480))),
            Ok(NextStep::SummaryRequiredAndFits {
                ordinary_total: MODEL_WINDOW + 1,
                summary_total: MODEL_WINDOW,
            })
        );
    }

    #[test]
    fn summary_one_token_over_blocks_without_modifying_the_candidate() {
        let current = context();
        let pending = candidate();
        let supplied = inputs(1_040_385, Some(1_044_481));
        let before = (current, pending, supplied);
        assert_eq!(
            plan_before_append(&current, &pending, &supplied),
            Err(Denial::SummaryCannotFit)
        );
        assert_eq!((current, pending, supplied), before);
    }

    #[test]
    fn full_window_input_with_positive_output_reserve_does_not_fit() {
        assert_eq!(
            check_fit(
                &evidence(Purpose::Ordinary, MODEL_WINDOW, 1),
                Purpose::Ordinary,
                1
            ),
            Ok(WindowFit::Exceeds {
                total: MODEL_WINDOW + 1
            })
        );
        assert_eq!(
            check_fit(
                &evidence(Purpose::Summary, MODEL_WINDOW, 1),
                Purpose::Summary,
                1
            ),
            Ok(WindowFit::Exceeds {
                total: MODEL_WINDOW + 1
            })
        );
    }

    #[test]
    fn ordinary_checked_add_overflow_denies_without_summary_fallback() {
        assert_eq!(
            plan(&inputs(u64::MAX, Some(1))),
            Err(Denial::ArithmeticOverflow)
        );
    }

    #[test]
    fn summary_checked_add_overflow_denies() {
        assert_eq!(
            plan(&inputs(1_040_385, Some(u64::MAX))),
            Err(Denial::ArithmeticOverflow)
        );
    }

    #[test]
    fn zero_or_over_ceiling_reserves_deny_in_both_roles() {
        for reserve in [0, 8_193, MODEL_WINDOW, u64::MAX] {
            for purpose in [Purpose::Ordinary, Purpose::Summary] {
                assert_eq!(
                    check_fit(&evidence(purpose, 1, reserve), purpose, reserve),
                    Err(Denial::InvalidReserve)
                );
            }
            let mut supplied = inputs(1, None);
            supplied.reserves.ordinary = reserve;
            assert_eq!(plan(&supplied), Err(Denial::InvalidReserve));
            supplied = inputs(1, None);
            supplied.reserves.summary = reserve;
            assert_eq!(plan(&supplied), Err(Denial::InvalidReserve));
        }
    }

    #[test]
    fn valid_reserves_have_independent_exact_ceiling_arithmetic() {
        for purpose in [Purpose::Ordinary, Purpose::Summary] {
            for reserve in [1, 4_096, 8_192] {
                assert_eq!(
                    check_fit(
                        &evidence(purpose, MODEL_WINDOW - reserve, reserve),
                        purpose,
                        reserve
                    ),
                    Ok(WindowFit::Fits {
                        total: MODEL_WINDOW
                    })
                );
                assert_eq!(
                    check_fit(
                        &evidence(purpose, MODEL_WINDOW - reserve + 1, reserve),
                        purpose,
                        reserve
                    ),
                    Ok(WindowFit::Exceeds {
                        total: MODEL_WINDOW + 1
                    })
                );
            }
        }
    }

    #[test]
    fn responses_count_cannot_qualify_the_current_chat_template() {
        for purpose in [Purpose::Ordinary, Purpose::Summary] {
            let mut supplied = evidence(purpose, 1, 4_096);
            supplied.expected.template = CountTemplate::CurrentChatV4;
            supplied.supplied.binding = supplied.expected;
            assert_eq!(
                check_fit(&supplied, purpose, 4_096),
                Err(Denial::InvalidBinding)
            );
        }
    }

    #[test]
    fn purpose_cannot_be_switched_even_when_both_bindings_match() {
        assert_eq!(
            check_fit(
                &evidence(Purpose::Summary, 1, 4_096),
                Purpose::Ordinary,
                4_096
            ),
            Err(Denial::InvalidBinding)
        );
        assert_eq!(
            check_fit(
                &evidence(Purpose::Ordinary, 1, 4_096),
                Purpose::Summary,
                4_096
            ),
            Err(Denial::InvalidBinding)
        );
    }

    #[test]
    fn unchanged_count_does_not_bind_changed_input_or_authority_metadata() {
        let changes: [fn(&mut InputBinding); 15] = [
            |binding| binding.purpose = Purpose::Summary,
            |binding| binding.template = CountTemplate::CurrentChatV4,
            |binding| binding.base.conversation = [8; 16],
            |binding| binding.base.revision += 1,
            |binding| binding.base.digest = [8; 32],
            |binding| binding.base.credential_generation += 1,
            |binding| binding.candidate.as_mut().unwrap().id = [8; 16],
            |binding| binding.candidate.as_mut().unwrap().digest = [8; 32],
            |binding| binding.candidate = None,
            |binding| binding.input_bytes += 1,
            |binding| binding.input_sha256 = [8; 32],
            |binding| binding.ordered_source_digest = [8; 32],
            |binding| binding.generation_parameters_digest = [8; 32],
            |binding| binding.output_reserve = 1,
            |binding| binding.base.revision = u64::MAX,
        ];
        for change in changes {
            let mut supplied = evidence(Purpose::Ordinary, 123, 8_192);
            change(&mut supplied.supplied.binding);
            assert_eq!(supplied.supplied.input_tokens, 123);
            assert_eq!(
                check_fit(&supplied, Purpose::Ordinary, 8_192),
                Err(Denial::CountBindingMismatch)
            );
        }
    }

    #[test]
    fn independently_expected_bytes_settings_and_sources_cannot_reuse_old_counts() {
        let changes: [fn(&mut InputBinding); 4] = [
            |binding| binding.input_bytes += 1,
            |binding| binding.input_sha256 = [8; 32],
            |binding| binding.ordered_source_digest = [8; 32],
            |binding| binding.generation_parameters_digest = [8; 32],
        ];
        for change in changes {
            let mut supplied = inputs(1, None);
            change(&mut supplied.ordinary.expected);
            assert_eq!(plan(&supplied), Err(Denial::CountBindingMismatch));
        }
    }

    #[test]
    fn output_reserve_changes_require_an_independently_bound_new_count() {
        let mut supplied = inputs(1, None);
        supplied.reserves.ordinary = 1;
        assert_eq!(plan(&supplied), Err(Denial::InvalidBinding));
        supplied.ordinary.expected.output_reserve = 1;
        assert_eq!(plan(&supplied), Err(Denial::CountBindingMismatch));
        supplied.ordinary.supplied.binding = supplied.ordinary.expected;
        assert_eq!(plan(&supplied), Ok(NextStep::OrdinaryFits { total: 2 }));
    }

    #[test]
    fn zero_context_identities_deny_before_arithmetic() {
        let changes: [fn(&mut ContextIdentity); 4] = [
            |identity| identity.conversation = [0; 16],
            |identity| identity.revision = 0,
            |identity| identity.digest = [0; 32],
            |identity| identity.credential_generation = 0,
        ];
        for change in changes {
            let mut current = context();
            change(&mut current);
            let mut pending = candidate();
            pending.base = current;
            assert_eq!(
                plan_before_append(&current, &pending, &inputs(u64::MAX, None)),
                Err(Denial::InvalidContext)
            );
        }
    }

    #[test]
    fn zero_candidate_identities_and_appended_candidates_deny() {
        for identity in [
            CandidateIdentity {
                id: [0; 16],
                digest: [4; 32],
            },
            CandidateIdentity {
                id: [3; 16],
                digest: [0; 32],
            },
        ] {
            let mut pending = candidate();
            pending.identity = identity;
            assert_eq!(
                plan_before_append(&context(), &pending, &inputs(1, None)),
                Err(Denial::InvalidCandidate)
            );
        }
        let mut pending = candidate();
        pending.phase = CandidatePhase::AlreadyAppended;
        assert_eq!(
            plan_before_append(&context(), &pending, &inputs(1, None)),
            Err(Denial::AlreadyAppended)
        );
    }

    #[test]
    fn zero_input_metadata_denies_even_with_matching_count_bindings() {
        let changes: [fn(&mut InputBinding); 4] = [
            |binding| binding.input_bytes = 0,
            |binding| binding.input_sha256 = [0; 32],
            |binding| binding.ordered_source_digest = [0; 32],
            |binding| binding.generation_parameters_digest = [0; 32],
        ];
        for purpose in [Purpose::Ordinary, Purpose::Summary] {
            for change in changes {
                let mut supplied = evidence(purpose, 1, 4_096);
                change(&mut supplied.expected);
                supplied.supplied.binding = supplied.expected;
                assert_eq!(
                    check_fit(&supplied, purpose, 4_096),
                    Err(Denial::InvalidBinding)
                );
            }
        }
    }

    #[test]
    fn changed_current_context_invalidates_even_an_ordinary_fits_plan() {
        let changes: [fn(&mut ContextIdentity); 4] = [
            |identity| identity.conversation = [8; 16],
            |identity| identity.revision += 1,
            |identity| identity.digest = [8; 32],
            |identity| identity.credential_generation += 1,
        ];
        let supplied = inputs(1, None);
        assert_eq!(plan(&supplied), Ok(NextStep::OrdinaryFits { total: 8_193 }));
        for change in changes {
            let mut current = context();
            change(&mut current);
            assert_eq!(
                plan_before_append(&current, &candidate(), &supplied),
                Err(Denial::StaleContext)
            );
        }
    }

    #[test]
    fn candidate_payload_or_id_changes_cannot_reuse_otherwise_matching_counts() {
        for identity in [
            CandidateIdentity {
                id: [8; 16],
                digest: [4; 32],
            },
            CandidateIdentity {
                id: [3; 16],
                digest: [8; 32],
            },
        ] {
            let mut pending = candidate();
            pending.identity = identity;
            assert_eq!(
                plan_before_append(&context(), &pending, &inputs(1, None)),
                Err(Denial::CandidateBindingMismatch)
            );
        }
    }

    #[test]
    fn ordinary_binding_for_another_base_denies() {
        let mut supplied = inputs(1, None);
        supplied.ordinary.expected.base.digest = [8; 32];
        supplied.ordinary.supplied.binding = supplied.ordinary.expected;
        assert_eq!(plan(&supplied), Err(Denial::StaleBinding));
    }

    #[test]
    fn missing_or_wrong_prefix_summary_blocks_without_losing_the_candidate() {
        let mut supplied = inputs(1_040_385, None);
        assert_eq!(plan(&supplied), Err(Denial::SummaryMissing));
        supplied.summary = Some(evidence(Purpose::Summary, 1, 4_096));
        let summary = supplied.summary.as_mut().unwrap();
        summary.expected.base.digest = [8; 32];
        summary.supplied.binding = summary.expected;
        assert_eq!(plan(&supplied), Err(Denial::StaleBinding));
        assert_eq!(candidate().phase, CandidatePhase::BeforeAppend);
    }

    #[test]
    fn summary_cannot_include_the_prospective_candidate_identity() {
        let mut supplied = inputs(1_040_385, Some(1));
        let summary = supplied.summary.as_mut().unwrap();
        summary.expected.candidate = Some(candidate().identity);
        summary.supplied.binding = summary.expected;
        assert_eq!(plan(&supplied), Err(Denial::InvalidBinding));
    }

    #[test]
    fn supplied_summary_binding_is_validated_even_when_ordinary_input_fits() {
        let mut supplied = inputs(1, Some(1));
        supplied
            .summary
            .as_mut()
            .unwrap()
            .supplied
            .binding
            .input_bytes += 1;
        assert_eq!(plan(&supplied), Err(Denial::CountBindingMismatch));
    }

    #[test]
    fn unused_summary_arithmetic_is_not_qualification_and_required_arithmetic_denies() {
        for (summary_tokens, required_denial) in [
            (MODEL_WINDOW, Denial::SummaryCannotFit),
            (u64::MAX, Denial::ArithmeticOverflow),
        ] {
            let unused = inputs(1_040_384, Some(summary_tokens));
            let before = unused;
            assert_eq!(
                plan(&unused),
                Ok(NextStep::OrdinaryFits {
                    total: MODEL_WINDOW,
                })
            );
            assert_eq!(unused, before);

            let mut required = unused;
            required.ordinary.supplied.input_tokens += 1;
            // The exact same separately bound summary count is now evaluated:
            // window exceedance and integer overflow are distinct denials.
            assert_eq!(required.summary, unused.summary);
            assert_eq!(plan(&required), Err(required_denial));
        }
    }

    #[test]
    fn summary_revision_requires_a_new_candidate_binding_and_fresh_count() {
        let old = inputs(1, None);
        let mut current = context();
        current.revision += 1;
        current.digest = [8; 32];
        let mut pending = candidate();
        pending.base = current;
        assert_eq!(
            plan_before_append(&current, &pending, &old),
            Err(Denial::StaleBinding)
        );
        let mut fresh = old;
        fresh.ordinary.expected.base = current;
        assert_eq!(
            plan_before_append(&current, &pending, &fresh),
            Err(Denial::CountBindingMismatch)
        );
        fresh.ordinary.supplied.binding = fresh.ordinary.expected;
        assert_eq!(
            plan_before_append(&current, &pending, &fresh),
            Ok(NextStep::OrdinaryFits { total: 8_193 })
        );
    }

    #[test]
    fn old_byte_limit_is_not_a_capacity_trigger_and_turn_count_is_absent() {
        let mut exact = inputs(1_040_384, None);
        exact.ordinary.expected.input_bytes = 512 * 1_024;
        exact.ordinary.supplied.binding = exact.ordinary.expected;
        let mut above = exact;
        above.ordinary.expected.input_bytes += 1;
        above.ordinary.expected.input_sha256 = [8; 32];
        above.ordinary.supplied.binding = above.ordinary.expected;
        assert_eq!(plan(&exact), plan(&above));
        assert_eq!(
            plan(&above),
            Ok(NextStep::OrdinaryFits {
                total: MODEL_WINDOW
            })
        );
        // There is deliberately no turn-count or old local-limit parameter.
    }

    #[test]
    fn repeated_planning_is_deterministic_and_cannot_mutate_any_input() {
        for supplied in [inputs(1, None), inputs(1_040_385, Some(1))] {
            let current = context();
            let pending = candidate();
            let before = (current, pending, supplied);
            let first = plan_before_append(&current, &pending, &supplied);
            for _ in 0..100 {
                assert_eq!(plan_before_append(&current, &pending, &supplied), first);
                assert_eq!((current, pending, supplied), before);
            }
        }
    }
}
