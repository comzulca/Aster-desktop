//! Versioned local Contributor data admission, not a universal DLP claim.
//! No remote classifier and no human confirmation may override a denial.

use crate::policy::{ChatMessage, MAX_MESSAGE_BYTES};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataClass {
    Synthetic,
    PublicEligible,
}

impl DataClass {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "synthetic" => Some(Self::Synthetic),
            "public_eligible" => Some(Self::PublicEligible),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Synthetic => "synthetic",
            Self::PublicEligible => "public_eligible",
        }
    }
}

pub fn opaque_id(value: &str) -> bool {
    let compact = value.replace('-', "");
    (value.len() == 32
        || (value.len() == 36 && [8, 13, 18, 23].iter().all(|i| value.as_bytes()[*i] == b'-')))
        && compact.len() == 32
        && compact
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn email_pattern(text: &str) -> bool {
    text.split(|c: char| c.is_whitespace() || matches!(c, '"' | '\'' | '<' | '>' | ',' | ';'))
        .any(|token| {
            token.split_once('@').is_some_and(|(local, domain)| {
                !local.is_empty()
                    && domain.split_once('.').is_some_and(|(host, suffix)| {
                        !host.is_empty()
                            && suffix.chars().filter(char::is_ascii_alphabetic).count() >= 2
                    })
            })
        })
}

pub fn admissible_text(text: &str) -> bool {
    if text.is_empty() || text.contains('\0') {
        return false;
    }
    let lower = text.to_ascii_lowercase();
    let prohibited = [
        "llm|",
        "nvapi-",
        "-----begin private key",
        "-----begin rsa private key",
        "authorization: bearer",
        "api_key=",
        "api_key:",
        "secret_key=",
        "password=",
        "c:\\users\\",
        "/home/",
        "confidential:",
        "sensitive personal data:",
    ];
    !prohibited.iter().any(|p| lower.contains(p)) && !email_pattern(text)
}

pub fn validate_new_message(class: &str, text: &str) -> Result<DataClass, &'static str> {
    let class = DataClass::parse(class).ok_or("data_class_denied")?;
    if text.trim().is_empty() || text.len() > MAX_MESSAGE_BYTES || !admissible_text(text) {
        return Err("data_admission_denied");
    }
    Ok(class)
}

pub fn validate_context(messages: &[ChatMessage]) -> Result<(), &'static str> {
    if messages.is_empty()
        || messages
            .iter()
            .any(|m| !matches!(m.role, "user" | "assistant") || !admissible_text(&m.content))
    {
        return Err("data_admission_denied");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classification_is_closed_and_cannot_override_a_secret() {
        for class in [
            "",
            "unknown",
            "private",
            "confidential",
            "personal",
            "sensitive",
        ] {
            assert!(validate_new_message(class, "A small invented Rust example.").is_err());
        }
        for class in ["synthetic", "public_eligible"] {
            assert!(validate_new_message(class, "An invented ownership example.").is_ok());
            for text in [
                "LLM|0|SYNTHETIC_NOT_A_SECRET",
                "password=example",
                "person@example.invalid",
                "C:\\Users\\Example\\document",
                "CONFIDENTIAL: example",
            ] {
                assert!(validate_new_message(class, text).is_err());
            }
        }
    }

    #[test]
    fn utf8_input_exact_boundary_and_continuation_are_revalidated() {
        let exact = "é".repeat(MAX_MESSAGE_BYTES / 2);
        assert!(validate_new_message("synthetic", &exact).is_ok());
        assert!(validate_new_message("synthetic", &(exact + "x")).is_err());
        let history = vec![
            ChatMessage {
                role: "assistant",
                content: "contact@example.invalid".into(),
            },
            ChatMessage {
                role: "user",
                content: "Continue the synthetic example.".into(),
            },
        ];
        assert!(validate_context(&history).is_err());
    }

    #[test]
    fn identifiers_are_opaque_not_names_or_paths() {
        assert!(opaque_id("0123456789abcdef0123456789abcdef"));
        assert!(opaque_id("01234567-89ab-cdef-0123-456789abcdef"));
        for value in [
            "owner_name",
            "../file",
            "",
            "0123456789abcdef0123456789abcdeG",
        ] {
            assert!(!opaque_id(value));
        }
    }
}
