//! Synthetic pre-bootstrap diagnostic. No UI, credential, store, real file or
//! provider operation is entered. Reports contain only closed case/status codes.

use crate::attachment_processing::{
    self, InputKind, ProcessedAttachment, ProcessedContent, ProcessingError, MAX_PDF_PAGES,
    MAX_SOURCE_BYTES, MAX_TEXT_BYTES,
};
use crate::attachment_worker::{self, ResourceProbeResult, ResourceScenario};
use image::ImageEncoder;
use serde::Serialize;
use std::sync::atomic::AtomicBool;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FixtureReport {
    pub schema_version: u32,
    pub classification: &'static str,
    /// All functional cases and the real bounded-child resource probes.
    pub all_passed: bool,
    pub qualification_complete: bool,
    pub cases: Vec<CaseResult>,
    pub resource_kill_checks: Vec<ResourceProbeResult>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseResult {
    code: &'static str,
    status: &'static str,
    reason: &'static str,
}

type Expected = fn(&[u8], &ProcessedAttachment) -> bool;
struct Case {
    code: &'static str,
    kind: InputKind,
    source: Vec<u8>,
    accepted: Option<Expected>,
}

fn source_binding(original: &[u8], result: &ProcessedAttachment) -> bool {
    result.source_sha256 == attachment_processing::source_hash(original)
}
fn text_expected(original: &[u8], result: &ProcessedAttachment) -> bool {
    source_binding(original, result)
        && matches!(&result.content, ProcessedContent::Text(text) if text.as_bytes() == original)
}
fn image_expected(original: &[u8], result: &ProcessedAttachment) -> bool {
    if !source_binding(original, result) {
        return false;
    }
    let ProcessedContent::Pages(pages) = &result.content else {
        return false;
    };
    let pixels_match = pages.first().is_some_and(|page| {
        page.rgba.chunks_exact(4).all(|pixel| {
            if original.starts_with(b"\x89PNG\r\n\x1a\n") {
                pixel == [80, 80, 80, 80]
            } else if original.starts_with(b"GIF") {
                pixel == [20, 80, 130, 255]
            } else {
                pixel[..3].iter().all(|channel| channel.abs_diff(50) <= 2) && pixel[3] == 255
            }
        })
    });
    pages.len() == 1
        && pages[0].width == 3
        && pages[0].height == 2
        && pages[0].rgba.len() == 24
        && !pages[0].png.is_empty()
        && pixels_match
}
fn pdf_expected(original: &[u8], result: &ProcessedAttachment) -> bool {
    if !source_binding(original, result) {
        return false;
    }
    let ProcessedContent::Pages(pages) = &result.content else {
        return false;
    };
    let centre = (75 * 150 + 75) * 4;
    pages.len() == 2
        && pages.iter().all(|page| {
            page.width == 150
                && page.height == 150
                && page.rgba.len() == 150 * 150 * 4
                && !page.png.is_empty()
        })
        && pages[0].rgba[centre..centre + 4] == [255, 0, 0, 255]
        && pages[1].rgba[centre..centre + 4] == [0, 0, 255, 255]
}

fn synthetic_pdf(page_count: usize) -> Vec<u8> {
    let mut objects = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        format!(
            "<< /Type /Pages /Count {page_count} /Kids [{}] >>",
            (0..page_count)
                .map(|index| format!("{} 0 R", 3 + index * 2))
                .collect::<Vec<_>>()
                .join(" ")
        ),
    ];
    for index in 0..page_count {
        let content_id = 4 + index * 2;
        objects.push(format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 72 72] /Resources << >> /Contents {content_id} 0 R >>"));
        let paint = if index % 2 == 0 {
            "1 0 0 rg\n0 0 72 72 re f\n"
        } else {
            "0 0 1 rg\n0 0 72 72 re f\n"
        };
        objects.push(format!(
            "<< /Length {} >>\nstream\n{paint}endstream",
            paint.len()
        ));
    }
    let mut pdf = b"%PDF-1.4\n".to_vec();
    let mut offsets = vec![0usize];
    for (index, object) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.extend(format!("{} 0 obj\n{object}\nendobj\n", index + 1).as_bytes());
    }
    let xref = pdf.len();
    pdf.extend(format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len()).as_bytes());
    for offset in &offsets[1..] {
        pdf.extend(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            offsets.len()
        )
        .as_bytes(),
    );
    pdf
}

fn png(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<u8>, ProcessingError> {
    let mut bytes = Vec::new();
    image::codecs::png::PngEncoder::new(&mut bytes)
        .write_image(rgba, width, height, image::ExtendedColorType::Rgba8)
        .map_err(|_| ProcessingError::Worker)?;
    Ok(bytes)
}
fn gif(frame_count: usize) -> Result<Vec<u8>, ProcessingError> {
    let mut bytes = Vec::new();
    {
        let mut encoder = image::codecs::gif::GifEncoder::new(&mut bytes);
        for _ in 0..frame_count {
            encoder
                .encode_frame(image::Frame::new(image::RgbaImage::from_pixel(
                    3,
                    2,
                    image::Rgba([20, 80, 130, 255]),
                )))
                .map_err(|_| ProcessingError::Worker)?;
        }
    }
    Ok(bytes)
}
fn cases() -> Result<Vec<Case>, ProcessingError> {
    let mut jpeg = Vec::new();
    image::codecs::jpeg::JpegEncoder::new(&mut jpeg)
        .encode(&[50u8; 18], 3, 2, image::ExtendedColorType::Rgb8)
        .map_err(|_| ProcessingError::Worker)?;
    // This untrusted PNG has legal individual axes but too many total pixels.
    // The synthetic encoder is trusted; the fixed child must refuse before decode.
    let pixel_overflow = png(4000, 1100, &vec![0u8; 4000 * 1100 * 4])?;
    let dimension_overflow = png(4097, 1, &vec![0u8; 4097 * 4])?;
    Ok(vec![
        Case {
            code: "native-text",
            kind: InputKind::Text,
            source: b"synthetic worker text".to_vec(),
            accepted: Some(text_expected),
        },
        Case {
            code: "native-png",
            kind: InputKind::Png,
            source: png(3, 2, &[80u8; 24])?,
            accepted: Some(image_expected),
        },
        Case {
            code: "native-jpeg",
            kind: InputKind::Jpeg,
            source: jpeg,
            accepted: Some(image_expected),
        },
        Case {
            code: "native-static-gif",
            kind: InputKind::Gif,
            source: gif(1)?,
            accepted: Some(image_expected),
        },
        Case {
            code: "native-two-page-pdf",
            kind: InputKind::Pdf,
            source: synthetic_pdf(2),
            accepted: Some(pdf_expected),
        },
        Case {
            code: "deny-malformed-png",
            kind: InputKind::Png,
            source: b"\x89PNG\r\n\x1a\nbroken".to_vec(),
            accepted: None,
        },
        Case {
            code: "deny-animated-gif",
            kind: InputKind::Gif,
            source: gif(2)?,
            accepted: None,
        },
        Case {
            code: "deny-pixel-overflow",
            kind: InputKind::Png,
            source: pixel_overflow,
            accepted: None,
        },
        Case {
            code: "deny-dimension-overflow",
            kind: InputKind::Png,
            source: dimension_overflow,
            accepted: None,
        },
        Case {
            code: "deny-pdf-page-overflow",
            kind: InputKind::Pdf,
            source: synthetic_pdf(MAX_PDF_PAGES + 1),
            accepted: None,
        },
        Case {
            code: "deny-malformed-pdf",
            kind: InputKind::Pdf,
            source: b"%PDF-1.4\nnot a document\n%%EOF".to_vec(),
            accepted: None,
        },
        Case {
            code: "deny-renamed-pdf",
            kind: InputKind::Text,
            source: synthetic_pdf(2),
            accepted: None,
        },
        Case {
            code: "deny-invalid-utf-eight",
            kind: InputKind::Text,
            source: vec![0xff],
            accepted: None,
        },
        Case {
            code: "deny-text-overflow",
            kind: InputKind::Text,
            source: vec![b'x'; MAX_TEXT_BYTES + 1],
            accepted: None,
        },
        Case {
            code: "deny-source-overflow",
            kind: InputKind::Pdf,
            source: vec![b'x'; MAX_SOURCE_BYTES + 1],
            accepted: None,
        },
    ])
}

/// Executes each fixed synthetic operation once. A launch/security/system failure
/// stops further child creation; there is no retry, fallback or OS-setting change.
pub fn run() -> FixtureReport {
    run_cases(
        cases(),
        attachment_processing::process_isolated,
        attachment_worker::run_resource_probe,
    )
}

/// Private test seam for the actual classification/stop/report computation.
/// Production uses only its fixed synthetic inputs and real bounded callbacks.
fn run_cases(
    cases: Result<Vec<Case>, ProcessingError>,
    mut process: impl FnMut(
        InputKind,
        &[u8],
        &AtomicBool,
    ) -> Result<ProcessedAttachment, ProcessingError>,
    mut resource_probe: impl FnMut(ResourceScenario) -> ResourceProbeResult,
) -> FixtureReport {
    let mut report = FixtureReport {
        schema_version: 1,
        classification: "synthetic-byte-worker-fixture",
        all_passed: false,
        qualification_complete: false,
        cases: Vec::new(),
        resource_kill_checks: Vec::new(),
    };
    let cases = match cases {
        Ok(cases) => cases,
        Err(_) => {
            report.cases.push(CaseResult {
                code: "fixture-construction",
                status: "FAIL",
                reason: "synthetic-construction-failed",
            });
            return report;
        }
    };
    let cancelled = AtomicBool::new(false);
    let mut boundary_available = true;
    for case in cases {
        if !boundary_available {
            report.cases.push(CaseResult {
                code: case.code,
                status: "NOT RUN",
                reason: "earlier-boundary-failure",
            });
            continue;
        }
        let (passed, reason) = match process(case.kind, &case.source, &cancelled) {
            Ok(result) => match case.accepted {
                Some(expected) if expected(&case.source, &result) => {
                    (true, "exact-synthetic-content-and-boundary")
                }
                Some(_) => (false, "processed-content-mismatch"),
                None => (false, "invalid-input-was-admitted"),
            },
            Err(
                error @ (ProcessingError::Isolation
                | ProcessingError::Timeout
                | ProcessingError::Cancelled),
            ) => {
                boundary_available = false;
                (
                    false,
                    match error {
                        ProcessingError::Isolation => "boundary-unavailable",
                        ProcessingError::Timeout => "unexpected-fixture-timeout",
                        _ => "unexpected-cancellation",
                    },
                )
            }
            Err(ProcessingError::Pdf(stage)) if case.accepted.is_some() => {
                boundary_available = false;
                (false, stage.diagnostic_reason())
            }
            Err(ProcessingError::Worker) if case.accepted.is_some() => {
                boundary_available = false;
                (false, "worker-failed-on-valid-synthetic-input")
            }
            Err(_) if case.accepted.is_none() => (true, "whole-operation-denied"),
            Err(_) => (false, "valid-synthetic-input-was-refused"),
        };
        report.cases.push(CaseResult {
            code: case.code,
            status: if passed { "PASS" } else { "FAIL" },
            reason,
        });
    }
    for scenario in ResourceScenario::ALL {
        if boundary_available {
            let result = resource_probe(scenario);
            if !result.boundary_available {
                boundary_available = false;
            }
            report.resource_kill_checks.push(result);
        } else {
            report.resource_kill_checks.push(ResourceProbeResult {
                code: scenario.code(),
                status: "NOT RUN",
                reason: "earlier-boundary-failure",
                elapsed_millis: 0,
                exit_code: None,
                cpu_user_100ns: 0,
                peak_memory_bytes: 0,
                limit_terminations: 0,
                boundary_available: false,
                cpu_proof: None,
            });
        }
    }
    report.all_passed = !report.cases.is_empty()
        && report.cases.iter().all(|case| case.status == "PASS")
        && report.resource_kill_checks.len() == ResourceScenario::ALL.len()
        && report
            .resource_kill_checks
            .iter()
            .all(|case| case.status == "PASS");
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn injected_pdf_stage_failure_stops_later_work_and_reports_static_reason() {
        for stage in attachment_processing::PdfFailureStage::ALL {
            let process_calls = std::cell::Cell::new(0);
            let resource_calls = std::cell::Cell::new(0);
            let report = run_cases(
                Ok(vec![
                    Case {
                        code: "native-two-page-pdf",
                        kind: InputKind::Pdf,
                        source: b"%PDF-closed-synthetic-stage-fixture".to_vec(),
                        accepted: Some(pdf_expected),
                    },
                    Case {
                        code: "native-text",
                        kind: InputKind::Text,
                        source: b"later-synthetic-operation".to_vec(),
                        accepted: Some(text_expected),
                    },
                ]),
                |kind, source, _| {
                    process_calls.set(process_calls.get() + 1);
                    assert_eq!(kind, InputKind::Pdf);
                    assert_eq!(source, b"%PDF-closed-synthetic-stage-fixture");
                    Err(ProcessingError::Pdf(stage))
                },
                |_| {
                    resource_calls.set(resource_calls.get() + 1);
                    panic!("failed PDF boundary must stop resource probes")
                },
            );
            assert_eq!(process_calls.get(), 1);
            assert_eq!(resource_calls.get(), 0);
            assert_eq!(report.cases.len(), 2);
            assert_eq!(report.cases[0].status, "FAIL");
            assert_eq!(report.cases[1].status, "NOT RUN");
            assert_eq!(report.cases[1].reason, "earlier-boundary-failure");
            assert_eq!(
                report.resource_kill_checks.len(),
                ResourceScenario::ALL.len()
            );
            assert!(report.resource_kill_checks.iter().all(|result| {
                result.status == "NOT RUN"
                    && result.reason == "earlier-boundary-failure"
                    && !result.boundary_available
            }));
            let body = serde_json::to_value(&report).unwrap();
            assert_eq!(body["allPassed"], false);
            assert_eq!(body["qualificationComplete"], false);
            assert_eq!(body["cases"][0]["status"], "FAIL");
            assert_eq!(body["cases"][0]["reason"], stage.diagnostic_reason());
            let case = body["cases"][0].as_object().unwrap();
            assert_eq!(case.len(), 3);
            assert!(case
                .keys()
                .all(|key| ["code", "status", "reason"].contains(&key.as_str())));
        }
    }

    #[test]
    fn construction_is_closed_synthetic_and_bounded() {
        let cases = cases().unwrap();
        assert_eq!(cases.len(), 15);
        assert_eq!(
            cases.iter().filter(|case| case.accepted.is_some()).count(),
            5
        );
        assert!(cases
            .iter()
            .all(|case| case.source.len() <= MAX_SOURCE_BYTES + 1));
        assert!(cases.iter().all(|case| case
            .code
            .chars()
            .all(|c| c.is_ascii_lowercase() || c == '-')));
    }
    #[test]
    fn report_codes_cannot_include_private_worker_error_messages() {
        let report = FixtureReport {
            schema_version: 1,
            classification: "synthetic-byte-worker-fixture",
            all_passed: false,
            qualification_complete: false,
            cases: vec![CaseResult {
                code: "native-text",
                status: "FAIL",
                reason: "boundary-unavailable",
            }],
            resource_kill_checks: vec![],
        };
        let body = serde_json::to_value(&report).unwrap();
        assert_eq!(body["qualificationComplete"], false);
        assert_eq!(body["cases"][0]["reason"], "boundary-unavailable");
        assert!(body.get("content").is_none());
        assert!(body.get("path").is_none());
        assert!(body.get("environment").is_none());
    }
}
