//! Private, cfg(test)-only T-087 / TM-033/038 fragment and supplied-quota seam.
//! No storage, eligibility, revision, commit, grant, count or network authority.
//! Supplied inventory health/length/completeness is not native evidence.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest as _, Sha256};

const SCHEMA_VERSION: u32 = 1;
const MAX_USER_BYTES: usize = 32 * 1024;
const MAX_ASSISTANT_BYTES: usize = 512 * 1024;
const MAX_FRAGMENT_BYTES: usize = 32 * 1024;
const MAX_ENCODED_FRAGMENT_BYTES: usize = 256 * 1024;
const MAX_PARTS: usize = 32;
const MAX_INVENTORY_OBJECTS: u64 = 65_536;
const MAX_CIPHERTEXT_TOTAL: u64 = 4_294_967_296;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LayoutError {
    Invalid,
    Limit,
    Overflow,
    Conflict,
    Integrity,
}

#[derive(Clone, PartialEq, Eq, Serialize)]
struct HistoryId(String);

impl HistoryId {
    fn new(value: &str) -> Result<Self, LayoutError> {
        if value.is_empty()
            || value.len() > 80
            || !value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(LayoutError::Invalid);
        }
        Ok(Self(value.to_owned()))
    }
}

impl<'de> Deserialize<'de> for HistoryId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::new(&value).map_err(|_| serde::de::Error::custom("invalid history identifier"))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Digest([u8; 32]);

impl Digest {
    fn of(bytes: &[u8]) -> Self {
        Self(Sha256::digest(bytes).into())
    }

    fn hex(self) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut result = String::with_capacity(64);
        for byte in self.0 {
            result.push(char::from(HEX[usize::from(byte >> 4)]));
            result.push(char::from(HEX[usize::from(byte & 15)]));
        }
        result
    }

    fn from_hex(value: &str) -> Result<Self, LayoutError> {
        fn digit(byte: u8) -> Result<u8, LayoutError> {
            match byte {
                b'0'..=b'9' => Ok(byte - b'0'),
                b'a'..=b'f' => Ok(byte - b'a' + 10),
                _ => Err(LayoutError::Invalid),
            }
        }
        if value.len() != 64 {
            return Err(LayoutError::Invalid);
        }
        let mut result = [0; 32];
        for (i, pair) in value.as_bytes().chunks_exact(2).enumerate() {
            result[i] = (digit(pair[0])? << 4) | digit(pair[1])?;
        }
        Ok(Self(result))
    }
}

impl Serialize for Digest {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.hex())
    }
}

impl<'de> Deserialize<'de> for Digest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::from_hex(&value).map_err(|_| serde::de::Error::custom("invalid digest"))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct RecordId([u8; 16]);

impl RecordId {
    fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum EntryTextKind {
    User,
    Assistant,
}

impl EntryTextKind {
    fn max_bytes(self) -> usize {
        match self {
            Self::User => MAX_USER_BYTES,
            Self::Assistant => MAX_ASSISTANT_BYTES,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
struct EntryBinding {
    conversation_id: HistoryId,
    entry_id: HistoryId,
    request_id: HistoryId,
    role: EntryTextKind,
    entry_sha256: Digest,
    entry_bytes: u64,
}

impl EntryBinding {
    fn from_text(
        conversation: &str,
        entry: &str,
        request: &str,
        role: EntryTextKind,
        text: &str,
    ) -> Result<Self, LayoutError> {
        validate_text(role, text)?;
        Ok(Self {
            conversation_id: HistoryId::new(conversation)?,
            entry_id: HistoryId::new(entry)?,
            request_id: HistoryId::new(request)?,
            role,
            entry_sha256: Digest::of(text.as_bytes()),
            entry_bytes: u64::try_from(text.len()).map_err(|_| LayoutError::Overflow)?,
        })
    }

    fn validate(&self) -> Result<usize, LayoutError> {
        let length = usize::try_from(self.entry_bytes).map_err(|_| LayoutError::Overflow)?;
        if length > self.role.max_bytes() {
            return Err(LayoutError::Limit);
        }
        if length == 0 && self.role == EntryTextKind::User {
            return Err(LayoutError::Invalid);
        }
        Ok(length)
    }
}

fn validate_text(role: EntryTextKind, text: &str) -> Result<(), LayoutError> {
    if text.len() > role.max_bytes() {
        return Err(LayoutError::Limit);
    }
    if text.contains('\0') || (role == EntryTextKind::User && text.trim().is_empty()) {
        return Err(LayoutError::Invalid);
    }
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FragmentTag {
    DisplayFragment,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DisplayFragmentV1 {
    schema_version: u32,
    kind: FragmentTag,
    conversation_id: HistoryId,
    entry_id: HistoryId,
    request_id: HistoryId,
    role: EntryTextKind,
    entry_sha256: Digest,
    entry_bytes: u64,
    part_ordinal: u32,
    part_count: u32,
    byte_offset: u64,
    text: String,
}

impl DisplayFragmentV1 {
    fn binding(&self) -> EntryBinding {
        EntryBinding {
            conversation_id: self.conversation_id.clone(),
            entry_id: self.entry_id.clone(),
            request_id: self.request_id.clone(),
            role: self.role,
            entry_sha256: self.entry_sha256,
            entry_bytes: self.entry_bytes,
        }
    }

    fn validate(&self) -> Result<(), LayoutError> {
        if self.schema_version != SCHEMA_VERSION
            || self.kind != FragmentTag::DisplayFragment
            || self.part_count == 0
            || self.part_count > MAX_PARTS as u32
            || self.part_ordinal >= self.part_count
            || self.text.contains('\0')
        {
            return Err(LayoutError::Invalid);
        }
        self.binding().validate()?;
        if self.text.len() > MAX_FRAGMENT_BYTES {
            return Err(LayoutError::Limit);
        }
        let size = u64::try_from(self.text.len()).map_err(|_| LayoutError::Overflow)?;
        let end = self
            .byte_offset
            .checked_add(size)
            .ok_or(LayoutError::Overflow)?;
        if end > self.entry_bytes
            || (self.part_ordinal == 0 && self.byte_offset != 0)
            || (self.part_ordinal > 0 && self.byte_offset == 0)
            || (self.part_ordinal + 1 == self.part_count && end != self.entry_bytes)
            || (self.part_ordinal + 1 < self.part_count && end >= self.entry_bytes)
        {
            return Err(LayoutError::Invalid);
        }
        if self.text.is_empty()
            && !(self.role == EntryTextKind::Assistant
                && self.entry_bytes == 0
                && self.part_count == 1
                && self.part_ordinal == 0
                && self.byte_offset == 0)
        {
            return Err(LayoutError::Invalid);
        }
        Ok(())
    }
}

fn encode_fragment(fragment: &DisplayFragmentV1) -> Result<Vec<u8>, LayoutError> {
    fragment.validate()?;
    let bytes = serde_json::to_vec(fragment).map_err(|_| LayoutError::Invalid)?;
    if bytes.len() > MAX_ENCODED_FRAGMENT_BYTES {
        return Err(LayoutError::Limit);
    }
    Ok(bytes)
}

fn decode_fragment(bytes: &[u8]) -> Result<DisplayFragmentV1, LayoutError> {
    if bytes.is_empty() {
        return Err(LayoutError::Invalid);
    }
    if bytes.len() > MAX_ENCODED_FRAGMENT_BYTES {
        return Err(LayoutError::Limit);
    }
    // Raw encoded input bounds transient serde allocation; this is no allocator proof.
    let fragment: DisplayFragmentV1 =
        serde_json::from_slice(bytes).map_err(|_| LayoutError::Invalid)?;
    fragment.validate()?;
    Ok(fragment)
}

fn next_boundary(text: &str, start: usize) -> Result<usize, LayoutError> {
    if start >= text.len() || !text.is_char_boundary(start) {
        return Err(LayoutError::Invalid);
    }
    let mut end = start
        .checked_add(MAX_FRAGMENT_BYTES)
        .ok_or(LayoutError::Overflow)?
        .min(text.len());
    while !text.is_char_boundary(end) {
        end = end.checked_sub(1).ok_or(LayoutError::Overflow)?;
    }
    if end <= start {
        return Err(LayoutError::Invalid);
    }
    Ok(end)
}

fn part_count(text: &str) -> Result<u32, LayoutError> {
    if text.is_empty() {
        return Ok(1);
    }
    let mut count = 0u32;
    let mut offset = 0;
    while offset < text.len() {
        offset = next_boundary(text, offset)?;
        count = count.checked_add(1).ok_or(LayoutError::Overflow)?;
        if count > MAX_PARTS as u32 {
            return Err(LayoutError::Limit);
        }
    }
    Ok(count)
}

struct FragmentIter<'a> {
    text: &'a str,
    binding: EntryBinding,
    part_count: u32,
    ordinal: u32,
    offset: usize,
    failed: bool,
}

impl<'a> FragmentIter<'a> {
    fn new(
        conversation: &str,
        entry: &str,
        request: &str,
        role: EntryTextKind,
        text: &'a str,
    ) -> Result<Self, LayoutError> {
        let binding = EntryBinding::from_text(conversation, entry, request, role, text)?;
        Ok(Self {
            text,
            binding,
            part_count: part_count(text)?,
            ordinal: 0,
            offset: 0,
            failed: false,
        })
    }
}

impl Iterator for FragmentIter<'_> {
    type Item = Result<DisplayFragmentV1, LayoutError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.failed || self.ordinal >= self.part_count {
            return None;
        }
        let end = if self.text.is_empty() {
            Ok(0)
        } else {
            next_boundary(self.text, self.offset)
        };
        let result = end.and_then(|end| {
            let fragment = DisplayFragmentV1 {
                schema_version: SCHEMA_VERSION,
                kind: FragmentTag::DisplayFragment,
                conversation_id: self.binding.conversation_id.clone(),
                entry_id: self.binding.entry_id.clone(),
                request_id: self.binding.request_id.clone(),
                role: self.binding.role,
                entry_sha256: self.binding.entry_sha256,
                entry_bytes: self.binding.entry_bytes,
                part_ordinal: self.ordinal,
                part_count: self.part_count,
                byte_offset: u64::try_from(self.offset).map_err(|_| LayoutError::Overflow)?,
                text: self.text[self.offset..end].to_owned(),
            };
            fragment.validate()?;
            self.offset = end;
            self.ordinal = self.ordinal.checked_add(1).ok_or(LayoutError::Overflow)?;
            Ok(fragment)
        });
        if result.is_err() {
            self.failed = true;
        }
        Some(result)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct PartRange {
    offset: usize,
    length: usize,
}

struct FragmentAssembler {
    expected: EntryBinding,
    text: String,
    ranges: [Option<PartRange>; MAX_PARTS],
    count: usize,
    expected_parts: Option<u32>,
    hasher: Sha256,
    fault: Option<LayoutError>,
}

impl FragmentAssembler {
    fn new(expected: EntryBinding) -> Result<Self, LayoutError> {
        let capacity = expected.validate()?;
        Ok(Self {
            expected,
            text: String::with_capacity(capacity),
            ranges: [None; MAX_PARTS],
            count: 0,
            expected_parts: None,
            hasher: Sha256::new(),
            fault: None,
        })
    }

    fn push(&mut self, fragment: &DisplayFragmentV1) -> Result<(), LayoutError> {
        if let Some(error) = self.fault {
            return Err(error);
        }
        let result = self.push_checked(fragment);
        if let Err(error) = result {
            self.fault = Some(error);
        }
        result
    }

    fn push_checked(&mut self, fragment: &DisplayFragmentV1) -> Result<(), LayoutError> {
        fragment.validate()?;
        if fragment.binding() != self.expected
            || usize::try_from(fragment.part_ordinal).map_err(|_| LayoutError::Overflow)?
                != self.count
            || usize::try_from(fragment.byte_offset).map_err(|_| LayoutError::Overflow)?
                != self.text.len()
            || self
                .expected_parts
                .is_some_and(|parts| parts != fragment.part_count)
        {
            return Err(LayoutError::Conflict);
        }
        if self.count >= MAX_PARTS {
            return Err(LayoutError::Limit);
        }
        let length = self
            .text
            .len()
            .checked_add(fragment.text.len())
            .ok_or(LayoutError::Overflow)?;
        if length > self.expected.validate()? {
            return Err(LayoutError::Limit);
        }
        self.ranges[self.count] = Some(PartRange {
            offset: self.text.len(),
            length: fragment.text.len(),
        });
        self.text.push_str(&fragment.text);
        self.hasher.update(fragment.text.as_bytes());
        self.count = self.count.checked_add(1).ok_or(LayoutError::Overflow)?;
        self.expected_parts = Some(fragment.part_count);
        Ok(())
    }

    fn finish(self) -> Result<String, LayoutError> {
        if let Some(error) = self.fault {
            return Err(error);
        }
        if self.expected_parts
            != Some(u32::try_from(self.count).map_err(|_| LayoutError::Overflow)?)
            || self.text.len() != self.expected.validate()?
        {
            return Err(LayoutError::Integrity);
        }
        if Digest(self.hasher.finalize().into()) != self.expected.entry_sha256 {
            return Err(LayoutError::Integrity);
        }
        validate_text(self.expected.role, &self.text)?;
        if part_count(&self.text)? != self.expected_parts.ok_or(LayoutError::Integrity)? {
            return Err(LayoutError::Integrity);
        }
        let mut offset = 0;
        for range in self.ranges.iter().take(self.count) {
            let end = if self.text.is_empty() {
                0
            } else {
                next_boundary(&self.text, offset)?
            };
            if *range
                != Some(PartRange {
                    offset,
                    length: end - offset,
                })
            {
                return Err(LayoutError::Integrity);
            }
            offset = end;
        }
        Ok(self.text)
    }
}

// Fixed classifications are metadata only, not registered PrivateStore domains.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum TranscriptDomainKind {
    Data,
    Control,
}

#[derive(Clone, Copy)]
enum InventoryDisposition {
    Referenced,
    RetainedOrphan,
}

#[derive(Clone, Copy)]
enum ObjectHealth {
    Healthy,
    Partial,
    Uncertain,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InventoryHealth {
    Healthy,
    Uncertain,
}

#[derive(Clone, Copy)]
struct ObservedLength {
    domain: TranscriptDomainKind,
    id: RecordId,
    ciphertext_bytes: u64,
    disposition: InventoryDisposition,
    health: ObjectHealth,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SuppliedInventorySnapshot {
    ciphertext_bytes: u64,
    object_count: u64,
    inventory_digest: Digest,
    health: InventoryHealth,
}

fn checked_totals(bytes: u64, count: u64, extra: u64) -> Result<(u64, u64), LayoutError> {
    let bytes = bytes.checked_add(extra).ok_or(LayoutError::Overflow)?;
    let count = count.checked_add(1).ok_or(LayoutError::Overflow)?;
    if bytes > MAX_CIPHERTEXT_TOTAL || count > MAX_INVENTORY_OBJECTS {
        return Err(LayoutError::Limit);
    }
    Ok((bytes, count))
}

struct QuotaAccumulator {
    ciphertext_bytes: u64,
    object_count: u64,
    last_key: Option<(TranscriptDomainKind, RecordId)>,
    hasher: Sha256,
    health: InventoryHealth,
    fault: Option<LayoutError>,
}

impl QuotaAccumulator {
    fn new() -> Self {
        let mut hasher = Sha256::new();
        hasher.update(b"aster-transcript-supplied-inventory-v1\0");
        Self {
            ciphertext_bytes: 0,
            object_count: 0,
            last_key: None,
            hasher,
            health: InventoryHealth::Healthy,
            fault: None,
        }
    }

    fn observe(&mut self, object: ObservedLength) -> Result<(), LayoutError> {
        if let Some(error) = self.fault {
            return Err(error);
        }
        let result = self.observe_checked(object);
        if let Err(error) = result {
            self.fault = Some(error);
        }
        result
    }

    fn observe_checked(&mut self, object: ObservedLength) -> Result<(), LayoutError> {
        let key = (object.domain, object.id);
        if self.last_key.is_some_and(|last| key <= last) {
            return Err(LayoutError::Conflict);
        }
        if object.ciphertext_bytes == 0 && !matches!(object.health, ObjectHealth::Partial) {
            return Err(LayoutError::Invalid);
        }
        let (bytes, count) = checked_totals(
            self.ciphertext_bytes,
            self.object_count,
            object.ciphertext_bytes,
        )?;
        let domain = match object.domain {
            TranscriptDomainKind::Data => 0,
            TranscriptDomainKind::Control => 1,
        };
        let disposition = match object.disposition {
            InventoryDisposition::Referenced => 0,
            InventoryDisposition::RetainedOrphan => 1,
        };
        let health = match object.health {
            ObjectHealth::Healthy => 0,
            ObjectHealth::Partial => 1,
            ObjectHealth::Uncertain => 2,
        };
        self.hasher.update([domain, disposition, health]);
        self.hasher.update(object.id.0);
        self.hasher.update(object.ciphertext_bytes.to_le_bytes());
        self.ciphertext_bytes = bytes;
        self.object_count = count;
        self.last_key = Some(key);
        if health != 0 {
            self.health = InventoryHealth::Uncertain;
        }
        Ok(())
    }

    fn finish(mut self) -> Result<SuppliedInventorySnapshot, LayoutError> {
        if let Some(error) = self.fault {
            return Err(error);
        }
        self.hasher.update(self.ciphertext_bytes.to_le_bytes());
        self.hasher.update(self.object_count.to_le_bytes());
        self.hasher.update([match self.health {
            InventoryHealth::Healthy => 0,
            InventoryHealth::Uncertain => 1,
        }]);
        Ok(SuppliedInventorySnapshot {
            ciphertext_bytes: self.ciphertext_bytes,
            object_count: self.object_count,
            inventory_digest: Digest(self.hasher.finalize().into()),
            health: self.health,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split(role: EntryTextKind, text: &str) -> Result<FragmentIter<'_>, LayoutError> {
        FragmentIter::new("conversation-1", "entry-1", "request-1", role, text)
    }

    fn fragment(text: &str) -> DisplayFragmentV1 {
        split(EntryTextKind::Assistant, text)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
    }

    fn assemble(role: EntryTextKind, text: &str) -> String {
        let mut parts = split(role, text).unwrap();
        let mut assembler = FragmentAssembler::new(parts.binding.clone()).unwrap();
        for part in &mut parts {
            let part = part.unwrap();
            let wire = encode_fragment(&part).unwrap();
            assembler.push(&decode_fragment(&wire).unwrap()).unwrap();
        }
        assembler.finish().unwrap()
    }

    fn observed(id: u128, bytes: u64) -> ObservedLength {
        ObservedLength {
            domain: TranscriptDomainKind::Data,
            id: RecordId::from_bytes(id.to_be_bytes()),
            ciphertext_bytes: bytes,
            disposition: InventoryDisposition::Referenced,
            health: ObjectHealth::Healthy,
        }
    }

    fn snapshot(objects: &[ObservedLength]) -> Result<SuppliedInventorySnapshot, LayoutError> {
        let mut accumulator = QuotaAccumulator::new();
        for object in objects {
            accumulator.observe(*object)?;
        }
        accumulator.finish()
    }

    #[test]
    fn inherited_user_byte_limit_exact_and_one_over() {
        let text = "x".repeat(MAX_USER_BYTES);
        assert_eq!(assemble(EntryTextKind::User, &text), text);
        assert!(matches!(
            split(EntryTextKind::User, &(text + "x")),
            Err(LayoutError::Limit)
        ));
    }

    #[test]
    fn inherited_assistant_byte_limit_exact_and_one_over() {
        let text = "x".repeat(MAX_ASSISTANT_BYTES);
        assert_eq!(assemble(EntryTextKind::Assistant, &text), text);
        assert!(matches!(
            split(EntryTextKind::Assistant, &(text + "x")),
            Err(LayoutError::Limit)
        ));
    }

    #[test]
    fn empty_assistant_and_whitespace_are_preserved_not_user_prompts() {
        for text in ["", " \r\n\t", "\u{2003}"] {
            assert_eq!(assemble(EntryTextKind::Assistant, text), text);
            assert!(matches!(
                split(EntryTextKind::User, text),
                Err(LayoutError::Invalid)
            ));
        }
        assert_eq!(split(EntryTextKind::Assistant, "").unwrap().count(), 1);
    }

    #[test]
    fn two_three_and_four_byte_scalars_cross_all_possible_cut_positions() {
        for scalar in ["é", "€", "🦀"] {
            for remainder in 1..scalar.len() {
                let text = "x".repeat(MAX_FRAGMENT_BYTES - remainder) + scalar + "suffix";
                let mut parts = split(EntryTextKind::Assistant, &text).unwrap();
                assert_eq!(
                    parts.next().unwrap().unwrap().text.len(),
                    MAX_FRAGMENT_BYTES - remainder
                );
                assert_eq!(assemble(EntryTextKind::Assistant, &text), text);
            }
        }
    }

    #[test]
    fn exact_utf8_bytes_line_endings_combining_and_bidi_are_not_normalised() {
        let text = "e\u{301} é\r\n\n\r\t\u{202e}אב\u{202c} 🦀 ";
        let result = assemble(EntryTextKind::Assistant, text);
        assert_eq!(result.as_bytes(), text.as_bytes());
        assert_eq!(Digest::of(result.as_bytes()), Digest::of(text.as_bytes()));
    }

    #[test]
    fn nul_is_denied_in_both_roles_and_wire() {
        for role in [EntryTextKind::User, EntryTextKind::Assistant] {
            assert!(matches!(split(role, "a\0b"), Err(LayoutError::Invalid)));
        }
        let mut part = fragment("ab");
        part.text = "a\0".to_owned();
        assert!(matches!(encode_fragment(&part), Err(LayoutError::Invalid)));
        assert!(matches!(
            decode_fragment(&serde_json::to_vec(&part).unwrap()),
            Err(LayoutError::Invalid)
        ));
    }

    #[test]
    fn strict_wire_round_trip_is_deterministic_and_has_only_closed_fields() {
        let part = fragment("synthetic");
        let wire = encode_fragment(&part).unwrap();
        assert_eq!(
            encode_fragment(&decode_fragment(&wire).unwrap()).unwrap(),
            wire
        );
        let string = String::from_utf8(wire).unwrap();
        assert!(string.contains("\"kind\":\"display_fragment\""));
        assert!(!string.contains("revision"));
        for field in [
            "profile",
            "github",
            "credential",
            "path",
            "approval",
            "originalPdf",
        ] {
            let mut hostile = string.clone();
            hostile.insert_str(hostile.len() - 1, &format!(",\"{field}\":\"synthetic\""));
            assert!(matches!(
                decode_fragment(hostile.as_bytes()),
                Err(LayoutError::Invalid)
            ));
        }
    }

    #[test]
    fn wrong_schema_tag_role_duplicate_missing_and_trailing_json_are_denied() {
        let wire = String::from_utf8(encode_fragment(&fragment("x")).unwrap()).unwrap();
        for hostile in [
            wire.replace("\"schemaVersion\":1", "\"schemaVersion\":2"),
            wire.replace("display_fragment", "commit"),
            wire.replace("assistant", "system"),
            wire.replace(
                "\"schemaVersion\":1",
                "\"schemaVersion\":1,\"schemaVersion\":1",
            ),
            wire.replace("\"schemaVersion\":1,", ""),
            wire.clone() + "{}",
        ] {
            assert!(matches!(
                decode_fragment(hostile.as_bytes()),
                Err(LayoutError::Invalid)
            ));
        }
    }

    #[test]
    fn invalid_utf8_and_unpaired_json_surrogates_are_denied() {
        assert!(matches!(
            decode_fragment(&[0xff]),
            Err(LayoutError::Invalid)
        ));
        let wire = String::from_utf8(encode_fragment(&fragment("x")).unwrap()).unwrap();
        for hostile in ["\\ud800", "\\udc00"] {
            assert!(matches!(
                decode_fragment(
                    wire.replace("\"text\":\"x\"", &format!("\"text\":\"{hostile}\""))
                        .as_bytes()
                ),
                Err(LayoutError::Invalid)
            ));
        }
    }

    #[test]
    fn identifier_and_digest_constructors_are_closed() {
        for id in ["", "..", "a/b", "a:b", "é", "a b", "\0"] {
            assert!(HistoryId::new(id).is_err());
        }
        assert!(HistoryId::new(&"x".repeat(80)).is_ok());
        assert!(HistoryId::new(&"x".repeat(81)).is_err());
        assert!(HistoryId::new("a-Z_9").is_ok());
        let digest = Digest::of(b"synthetic");
        assert_eq!(Digest::from_hex(&digest.hex()).unwrap(), digest);
        assert!(Digest::from_hex(&digest.hex().to_uppercase()).is_err());
        assert!(Digest::from_hex(&"g".repeat(64)).is_err());
        assert!(Digest::from_hex(&"a".repeat(63)).is_err());
    }

    #[test]
    fn raw_encoded_cap_is_checked_before_decode_and_whitespace_is_not_identity() {
        let wire = encode_fragment(&fragment("x")).unwrap();
        let mut padded = wire.clone();
        padded.resize(MAX_ENCODED_FRAGMENT_BYTES, b' ');
        assert_eq!(
            encode_fragment(&decode_fragment(&padded).unwrap()).unwrap(),
            wire
        );
        padded.push(b' ');
        assert!(matches!(decode_fragment(&padded), Err(LayoutError::Limit)));
        assert!(matches!(decode_fragment(&[]), Err(LayoutError::Invalid)));
    }

    #[test]
    fn worst_ascii_control_escaping_remains_bounded_and_exact() {
        let text = "\u{1}".repeat(MAX_FRAGMENT_BYTES);
        let wire = encode_fragment(&fragment(&text)).unwrap();
        assert!(wire.len() > MAX_FRAGMENT_BYTES * 6);
        assert!(wire.len() < MAX_ENCODED_FRAGMENT_BYTES);
        assert_eq!(assemble(EntryTextKind::Assistant, &text), text);
    }

    #[test]
    fn complete_expected_entry_binding_is_independent_and_required() {
        let part = fragment("synthetic");
        for field in 0..6 {
            let mut expected = part.binding();
            match field {
                0 => expected.conversation_id = HistoryId::new("other-conversation").unwrap(),
                1 => expected.entry_id = HistoryId::new("other-entry").unwrap(),
                2 => expected.request_id = HistoryId::new("other-request").unwrap(),
                3 => expected.role = EntryTextKind::User,
                4 => expected.entry_sha256 = Digest::of(b"different"),
                _ => expected.entry_bytes += 1,
            }
            let mut assembler = FragmentAssembler::new(expected).unwrap();
            assert_eq!(assembler.push(&part), Err(LayoutError::Conflict));
            assert!(assembler.finish().is_err());
        }
    }

    #[test]
    fn ordinals_offsets_counts_and_missing_parts_fail_closed() {
        let text = "x".repeat(MAX_FRAGMENT_BYTES + 1);
        let parts: Vec<_> = split(EntryTextKind::Assistant, &text)
            .unwrap()
            .map(Result::unwrap)
            .collect();
        let mut assembler = FragmentAssembler::new(parts[0].binding()).unwrap();
        assert!(assembler.push(&parts[1]).is_err());
        assert!(assembler.finish().is_err());
        let mut missing = FragmentAssembler::new(parts[0].binding()).unwrap();
        missing.push(&parts[0]).unwrap();
        assert!(matches!(missing.finish(), Err(LayoutError::Integrity)));
        for field in 0..3 {
            let mut changed = parts[1].clone();
            match field {
                0 => changed.part_ordinal = 0,
                1 => changed.byte_offset -= 1,
                _ => changed.part_count = 3,
            }
            let mut assembler = FragmentAssembler::new(parts[0].binding()).unwrap();
            assembler.push(&parts[0]).unwrap();
            assert!(assembler.push(&changed).is_err());
        }
    }

    #[test]
    fn duplicate_part_poison_is_retained_and_no_partial_text_is_returned() {
        let text = "x".repeat(MAX_FRAGMENT_BYTES + 1);
        let parts: Vec<_> = split(EntryTextKind::Assistant, &text)
            .unwrap()
            .map(Result::unwrap)
            .collect();
        let mut assembler = FragmentAssembler::new(parts[0].binding()).unwrap();
        assembler.push(&parts[0]).unwrap();
        let fault = assembler.push(&parts[0]).unwrap_err();
        assert_eq!(assembler.push(&parts[1]), Err(fault));
        assert_eq!(assembler.finish(), Err(fault));
    }

    #[test]
    fn correct_digest_but_non_greedy_partition_is_denied_at_finish() {
        let mut first = fragment("ab");
        first.text = "a".to_owned();
        first.part_count = 2;
        let mut second = first.clone();
        second.text = "b".to_owned();
        second.part_ordinal = 1;
        second.byte_offset = 1;
        let mut assembler = FragmentAssembler::new(first.binding()).unwrap();
        assembler.push(&first).unwrap();
        assembler.push(&second).unwrap();
        assert!(matches!(assembler.finish(), Err(LayoutError::Integrity)));
    }

    #[test]
    fn raw_part_length_and_metadata_part_ceiling_are_independent() {
        let mut part = fragment("x");
        part.part_count = 33;
        assert!(matches!(encode_fragment(&part), Err(LayoutError::Invalid)));
        part.part_count = 1;
        part.text = "x".repeat(MAX_FRAGMENT_BYTES + 1);
        part.entry_bytes = part.text.len() as u64;
        assert!(matches!(encode_fragment(&part), Err(LayoutError::Limit)));
        let text = "🦀".repeat(MAX_ASSISTANT_BYTES / 4);
        assert!(split(EntryTextKind::Assistant, &text).unwrap().count() <= MAX_PARTS);
    }

    #[test]
    fn altered_same_length_text_fails_whole_entry_hash() {
        let mut part = fragment("synthetic");
        part.text = "SYNTHETIC".to_owned();
        let mut assembler = FragmentAssembler::new(part.binding()).unwrap();
        assembler.push(&part).unwrap();
        assert!(matches!(assembler.finish(), Err(LayoutError::Integrity)));
    }

    #[test]
    fn empty_inventory_snapshot_is_repeatable_metadata_not_native_completeness() {
        let first = snapshot(&[]).unwrap();
        assert_eq!(snapshot(&[]).unwrap(), first);
        assert_eq!(first.ciphertext_bytes, 0);
        assert_eq!(first.object_count, 0);
        assert_eq!(first.health, InventoryHealth::Healthy);
    }

    #[test]
    fn referenced_orphan_and_control_lengths_all_consume_quota() {
        let first = observed(1, 10);
        let mut orphan = observed(2, 20);
        orphan.disposition = InventoryDisposition::RetainedOrphan;
        let mut control = observed(0, 30);
        control.domain = TranscriptDomainKind::Control;
        let value = snapshot(&[first, orphan, control]).unwrap();
        assert_eq!(value.ciphertext_bytes, 60);
        assert_eq!(value.object_count, 3);
        assert_eq!(snapshot(&[first, orphan, control]).unwrap(), value);
    }

    #[test]
    fn exact_four_gib_fits_supplied_math_and_one_more_byte_denies() {
        assert_eq!(
            snapshot(&[observed(1, MAX_CIPHERTEXT_TOTAL)])
                .unwrap()
                .ciphertext_bytes,
            MAX_CIPHERTEXT_TOTAL
        );
        assert_eq!(
            snapshot(&[observed(1, MAX_CIPHERTEXT_TOTAL), observed(2, 1)]),
            Err(LayoutError::Limit)
        );
        assert_eq!(
            snapshot(&[observed(1, MAX_CIPHERTEXT_TOTAL + 1)]),
            Err(LayoutError::Limit)
        );
    }

    #[test]
    fn supplied_length_overflow_is_distinct_from_window_limit() {
        assert_eq!(
            snapshot(&[observed(1, 1), observed(2, u64::MAX)]),
            Err(LayoutError::Overflow)
        );
        // Raw scalar helper: count overflow is unreachable through the admitted 65,536 bound.
        assert_eq!(checked_totals(0, u64::MAX, 1), Err(LayoutError::Overflow));
        assert_eq!(checked_totals(u64::MAX, 0, 1), Err(LayoutError::Overflow));
    }

    #[test]
    fn ordered_inventory_denies_duplicate_reorder_and_domain_regression() {
        assert_eq!(
            snapshot(&[observed(1, 1), observed(1, 2)]),
            Err(LayoutError::Conflict)
        );
        assert_eq!(
            snapshot(&[observed(2, 1), observed(1, 2)]),
            Err(LayoutError::Conflict)
        );
        let mut control = observed(1, 1);
        control.domain = TranscriptDomainKind::Control;
        assert_eq!(
            snapshot(&[control, observed(2, 2)]),
            Err(LayoutError::Conflict)
        );
        assert!(snapshot(&[observed(1, 1), control]).is_ok());
    }

    #[test]
    fn inventory_fault_latches_and_cannot_return_a_partial_snapshot() {
        let mut accumulator = QuotaAccumulator::new();
        accumulator.observe(observed(1, 1)).unwrap();
        assert_eq!(
            accumulator.observe(observed(1, 1)),
            Err(LayoutError::Conflict)
        );
        assert_eq!(
            accumulator.observe(observed(2, 1)),
            Err(LayoutError::Conflict)
        );
        assert_eq!(accumulator.finish(), Err(LayoutError::Conflict));
    }

    #[test]
    fn zero_partial_counts_and_marks_uncertainty_without_discounting_other_bytes() {
        assert_eq!(snapshot(&[observed(1, 0)]), Err(LayoutError::Invalid));
        let mut partial = observed(1, 0);
        partial.health = ObjectHealth::Partial;
        partial.disposition = InventoryDisposition::RetainedOrphan;
        let mut uncertain = observed(2, 7);
        uncertain.health = ObjectHealth::Uncertain;
        let value = snapshot(&[partial, uncertain, observed(3, 5)]).unwrap();
        assert_eq!(value.ciphertext_bytes, 12);
        assert_eq!(value.object_count, 3);
        assert_eq!(value.health, InventoryHealth::Uncertain);
        uncertain.ciphertext_bytes = 0;
        assert_eq!(snapshot(&[uncertain]), Err(LayoutError::Invalid));
    }

    #[test]
    fn fixed_inventory_count_limit_does_not_retain_a_global_id_set() {
        let mut accumulator = QuotaAccumulator::new();
        for id in 0..MAX_INVENTORY_OBJECTS {
            accumulator.observe(observed(u128::from(id), 1)).unwrap();
        }
        assert_eq!(accumulator.object_count, MAX_INVENTORY_OBJECTS);
        assert_eq!(
            accumulator.observe(observed(u128::from(MAX_INVENTORY_OBJECTS), 1)),
            Err(LayoutError::Limit)
        );
        assert_eq!(accumulator.finish(), Err(LayoutError::Limit));
    }

    #[test]
    fn framed_inventory_digest_binds_ids_lengths_domains_disposition_and_health() {
        let original = observed(1, 5);
        let reference = snapshot(&[original]).unwrap();
        for field in 0..5 {
            let mut changed = original;
            match field {
                0 => changed.id = RecordId::from_bytes(2u128.to_be_bytes()),
                1 => changed.ciphertext_bytes = 6,
                2 => changed.domain = TranscriptDomainKind::Control,
                3 => changed.disposition = InventoryDisposition::RetainedOrphan,
                _ => changed.health = ObjectHealth::Partial,
            }
            assert_ne!(
                snapshot(&[changed]).unwrap().inventory_digest,
                reference.inventory_digest
            );
        }
    }
}
