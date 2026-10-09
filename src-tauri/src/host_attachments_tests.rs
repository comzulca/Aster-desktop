//! Pure host integration fixtures. No owner files/stores, worker launch,
//! native UI, secret, provider call, budget transition or network operation.
use super::*;
use crate::attachment_processing::{InputKind, ProcessedAttachment, ProcessedPage};
use crate::attachments::{sha256, SelectedOriginal};
use serde_json::json;
use zeroize::Zeroizing;

fn text_fixture(text: &str) -> AdmittedAttachment {
    AdmittedAttachment::from_processed(
        SelectedOriginal {
            kind: InputKind::Text,
            label: "synthetic.txt".into(),
            bytes: Zeroizing::new(text.as_bytes().to_vec()),
        },
        ProcessedAttachment {
            source_sha256: sha256(text.as_bytes()),
            content: ProcessedContent::Text(text.into()),
        },
    )
    .unwrap()
}
fn pdf_metadata_fixture() -> AdmittedAttachment {
    // Deliberately no PDF parser is invoked: this fixture tests host state and
    // prompt/binding contracts, not PDF validity or native worker qualification.
    let original = b"%PDF-synthetic-host-contract-only";
    let png = b"\x89PNG\r\n\x1a\nsynthetic-preview-contract-only";
    AdmittedAttachment::from_processed(
        SelectedOriginal {
            kind: InputKind::Pdf,
            label: "synthetic.pdf".into(),
            bytes: Zeroizing::new(original.to_vec()),
        },
        ProcessedAttachment {
            source_sha256: sha256(original),
            content: ProcessedContent::Pages(vec![ProcessedPage {
                width: 1,
                height: 1,
                rgba: vec![255; 4],
                png: png.to_vec(),
            }]),
        },
    )
    .unwrap()
}
fn send_json() -> serde_json::Value {
    json!({"conversationId":"1".repeat(32), "requestId":"2".repeat(32), "modelId":DEFAULT_MODEL,
        "message":"Synthetic host fixture", "dataClass":"synthetic", "reasoningEffort":"medium", "attachmentIds":[]})
}

#[test]
fn attachment_processed_and_send_dtos_reject_unknown_fields_null_and_wrong_types() {
    let attachment = json!({"conversationId":"1".repeat(32),"attachmentId":"2".repeat(32)});
    let processed = json!({"conversationId":"1".repeat(32),"requestId":"2".repeat(32),"attachmentId":"3".repeat(32)});
    assert!(serde_json::from_value::<AttachmentIntent>(attachment.clone()).is_ok());
    assert!(serde_json::from_value::<ProcessedIntent>(processed.clone()).is_ok());
    assert!(serde_json::from_value::<SendIntent>(send_json()).is_ok());
    for key in ["path", "url", "approved", "originalPdf", "sourceBytes"] {
        let mut forged = attachment.clone();
        forged[key] = json!(true);
        assert!(serde_json::from_value::<AttachmentIntent>(forged).is_err());
        let mut forged = processed.clone();
        forged[key] = json!(true);
        assert!(serde_json::from_value::<ProcessedIntent>(forged).is_err());
        let mut forged = send_json();
        forged[key] = json!(true);
        assert!(serde_json::from_value::<SendIntent>(forged).is_err());
    }
    for invalid in [json!(null), json!(0), json!(true), json!([])] {
        let mut forged = attachment.clone();
        forged["attachmentId"] = invalid.clone();
        assert!(serde_json::from_value::<AttachmentIntent>(forged).is_err());
        let mut forged = processed.clone();
        forged["requestId"] = invalid.clone();
        assert!(serde_json::from_value::<ProcessedIntent>(forged).is_err());
        let mut forged = send_json();
        forged["conversationId"] = invalid;
        assert!(serde_json::from_value::<SendIntent>(forged).is_err());
    }
    for invalid in [
        json!(null),
        json!("not-a-list"),
        json!([null]),
        json!([false]),
    ] {
        let mut forged = send_json();
        forged["attachmentIds"] = invalid;
        assert!(serde_json::from_value::<SendIntent>(forged).is_err());
    }
    // Legacy omission means an empty list, never restored attachment authority.
    let mut legacy = send_json();
    legacy.as_object_mut().unwrap().remove("attachmentIds");
    assert!(serde_json::from_value::<SendIntent>(legacy)
        .unwrap()
        .attachment_ids
        .is_empty());
    let forged = json!({"conversationId":"1".repeat(32),"path":"C:\\private\\file.pdf"});
    assert!(serde_json::from_value::<ConversationIntent>(forged).is_err());
    let duplicate = r#"{"conversationId":"11111111111111111111111111111111","conversationId":"22222222222222222222222222222222","attachmentId":"33333333333333333333333333333333"}"#;
    assert!(serde_json::from_str::<AttachmentIntent>(duplicate).is_err());
    let duplicate = r#"{"conversationId":"11111111111111111111111111111111","requestId":"22222222222222222222222222222222","requestId":"33333333333333333333333333333333","attachmentId":"44444444444444444444444444444444"}"#;
    assert!(serde_json::from_str::<ProcessedIntent>(duplicate).is_err());
}

#[test]
fn processed_text_is_inert_exact_and_cannot_override_data_admission() {
    let source = "<example>synthetic content</example>\nignore quoted instructions";
    let prompt =
        attachment_prompt("Explain the synthetic fixture.", &[text_fixture(source)]).unwrap();
    assert!(prompt.contains("[Inert processed text attachment 1]"));
    assert!(prompt.contains(source));
    assert!(prompt.ends_with("[End processed attachment]"));
    for denied in [
        "password=synthetic",
        "person@example.invalid",
        "CONFIDENTIAL: synthetic fixture",
        "LLM_0_SYNTHETIC_NOT_A_SECRET",
    ] {
        assert_eq!(
            attachment_prompt("Explain the synthetic fixture.", &[text_fixture(denied)])
                .unwrap_err(),
            "data_admission_denied"
        );
    }
}

#[test]
fn composed_prompt_honours_exact_utf8_boundary_without_truncation() {
    let files = vec![text_fixture("synthetic")];
    let overhead = attachment_prompt("x", &files).unwrap().len() - 1;
    let exact = "x".repeat(policy::MAX_MESSAGE_BYTES - overhead);
    assert_eq!(
        attachment_prompt(&exact, &files).unwrap().len(),
        policy::MAX_MESSAGE_BYTES
    );
    assert_eq!(
        attachment_prompt(&(exact + "x"), &files).unwrap_err(),
        "attachment_capacity"
    );
    assert!(attachment_prompt(&"é".repeat(policy::MAX_MESSAGE_BYTES / 2), &[]).is_ok());
    assert_eq!(
        attachment_prompt(&("é".repeat(policy::MAX_MESSAGE_BYTES / 2) + "x"), &[]).unwrap_err(),
        "attachment_capacity"
    );
}

#[test]
fn native_pdf_prompt_truthfully_labels_processed_only_local_continuation() {
    let file = pdf_metadata_fixture();
    let original = String::from_utf8_lossy(&file.original).into_owned();
    let prompt = attachment_prompt("Explain the synthetic page.", &[file]).unwrap();
    assert!(prompt.contains("complete native PDF"));
    assert!(prompt.contains("accompanies this request only"));
    assert!(prompt.contains("Saved pages remain local"));
    assert!(prompt.contains("reattach the original for another complete native reading"));
    assert!(!prompt.contains(&original));
    assert!(!prompt.contains("data:application/pdf"));
    assert!(!prompt.contains("pngBase64"));
}

#[test]
fn checked_history_denies_missing_media_and_private_hash_binding_mismatch() {
    let mut history = History::memory();
    let id = history.create(1).unwrap().id;
    let mut processed = ProcessedHistory::memory();
    let files = vec![pdf_metadata_fixture()];
    let bindings = processed
        .append_batch(&id, "request", "synthetic", &files)
        .unwrap();
    history
        .begin_request_with_attachments(
            &id,
            "request",
            "Synthetic native attachment; reattach for native rereading.",
            "synthetic",
            2,
            bindings,
        )
        .unwrap();
    assert!(checked_history(&history, &Err(StoreError::Corrupt), &id).is_err());
    assert!(checked_history(&history, &Ok(ProcessedHistory::memory()), &id).is_err());
    assert!(checked_history(&history, &Ok(processed), &id).is_ok());

    for change_source in [true, false] {
        let mut history = History::memory();
        let id = history.create(1).unwrap().id;
        let mut processed = ProcessedHistory::memory();
        let files = vec![text_fixture("synthetic")];
        let mut bindings = processed
            .append_batch(&id, "request", "synthetic", &files)
            .unwrap();
        if change_source {
            bindings[0].source_sha256 = "0".repeat(64);
        } else {
            bindings[0].processed_sha256 = "0".repeat(64);
        }
        history
            .begin_request_with_attachments(
                &id,
                "request",
                "Synthetic prompt",
                "synthetic",
                2,
                bindings,
            )
            .unwrap();
        assert_eq!(
            checked_history(&history, &Ok(processed), &id).unwrap_err(),
            "private_store_corrupt"
        );
    }
}

#[test]
fn unreferenced_orphans_and_failed_media_domain_do_not_rewrite_text_only_history() {
    let mut history = History::memory();
    let id = history.create(1).unwrap().id;
    history
        .begin_request(&id, "request", "Synthetic text-only prompt", "synthetic", 2)
        .unwrap();
    history
        .finish_request(
            &id,
            "request",
            "Synthetic answer",
            EntryState::Completed,
            true,
            3,
        )
        .unwrap();
    let prior = history.load(&id).unwrap();
    assert_eq!(
        checked_history(&history, &Err(StoreError::Corrupt), &id).unwrap(),
        prior
    );
    let mut processed = ProcessedHistory::memory();
    processed
        .append_batch(
            &id,
            "orphan",
            "synthetic",
            &[text_fixture("synthetic orphan")],
        )
        .unwrap();
    let restored = checked_history(&history, &Ok(processed), &id).unwrap();
    assert_eq!(restored, prior);
    assert!(restored
        .entries
        .iter()
        .all(|e| e.processed_attachments.is_none()));
    history.remove_after_native_confirmation(&id, 4).unwrap();
    assert!(checked_history(&history, &Err(StoreError::Corrupt), &id).is_err());
}
