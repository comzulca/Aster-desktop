//! CI-only synthetic codec tests against the actual production module.
//! No duplicate parser or source repair is substituted for the application.
#![allow(dead_code)]

#[cfg(windows)]
compile_error!("Linux-only codec fixture harness; use the complete Windows host suite on Windows");

#[path = "../../../../src-tauri/src/attachment_processing.rs"]
mod attachment_processing;

// Complex parsing is allowed here only for the module's closed synthetic
// inline fixtures on an ephemeral hosted runner. Host/worker admission cannot
// be exercised or credited by this Linux-only harness.
mod attachment_worker {
    use super::attachment_processing::{InputKind, ProcessedAttachment, ProcessingError};
    use std::sync::atomic::AtomicBool;

    pub fn run_isolated(
        _: InputKind,
        _: &[u8],
        _: &AtomicBool,
    ) -> Result<Vec<u8>, ProcessingError> {
        panic!("test-only codec harness must not enter worker admission");
    }

    pub fn decode_response(
        _: InputKind,
        _: &[u8],
        _: &[u8],
    ) -> Result<ProcessedAttachment, ProcessingError> {
        panic!("test-only codec harness must not credit worker output");
    }
}
