//! Closed structural policy for the dormant native successor C1 slice.

use crate::native_document::NativeSuccessorFamily;
use serde::{Deserialize, Deserializer};
use sha2::{Digest, Sha256};

const C1_POLICY_ID: &str = "native-phase-artifact-policy-v1";
const C1_POLICY_CANONICAL_BYTES: &[u8] = concat!(
    "engram-native-phase-artifact-policy-v1\n",
    "native_correction_v1/correction_operator|partition=required|relay=none|",
    "mutation=required|artifacts=intent,dispatch,action,terminal\n",
    "native_correction_v1/correction_proposal|partition=required|",
    "relay=synthetic_mutation_then_read_v1|mutation=required|",
    "artifacts=intent,dispatch,action,terminal\n",
    "native_correction_v1/correction_retrieval|partition=required|",
    "relay=synthetic_single_read_v1|mutation=forbidden|artifacts=intent,action,terminal\n",
    "native_stale_isolated_v1/stale_activation|partition=required|relay=none|",
    "mutation=required|artifacts=intent,dispatch,action,terminal\n",
    "native_stale_isolated_v1/stale_evaluation|partition=required|",
    "relay=synthetic_single_read_v1|mutation=forbidden|artifacts=intent,action,terminal\n",
    "native_stale_isolated_v1/stale_teaching_engram|partition=required|",
    "relay=synthetic_single_mutation_v1|mutation=required|",
    "artifacts=intent,dispatch,action,terminal\n",
    "native_stale_isolated_v1/stale_teaching_native|partition=forbidden|relay=none|",
    "mutation=forbidden|artifacts=intent,action,terminal\n",
)
.as_bytes();

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct C1Sha256 {
    spelling: String,
    digest_bytes: [u8; 32],
}

impl C1Sha256 {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        if value.len() != 64
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return None;
        }

        let mut digest_bytes = [0_u8; 32];
        for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
            digest_bytes[index] = (hex_nibble(pair[0]) << 4) | hex_nibble(pair[1]);
        }
        Some(Self {
            spelling: value.to_owned(),
            digest_bytes,
        })
    }

    pub(crate) fn from_digest_bytes(digest_bytes: [u8; 32]) -> Self {
        const LOWER_HEX: &[u8; 16] = b"0123456789abcdef";

        let mut spelling = [0_u8; 64];
        for (index, byte) in digest_bytes.iter().copied().enumerate() {
            spelling[index * 2] = LOWER_HEX[usize::from(byte >> 4)];
            spelling[index * 2 + 1] = LOWER_HEX[usize::from(byte & 0x0f)];
        }
        Self {
            spelling: String::from_utf8(spelling.to_vec())
                .expect("lowercase hexadecimal bytes are valid UTF-8"),
            digest_bytes,
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.spelling
    }

    pub(crate) fn as_digest_bytes(&self) -> &[u8; 32] {
        &self.digest_bytes
    }
}

impl<'de> Deserialize<'de> for C1Sha256 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).ok_or_else(|| {
            serde::de::Error::custom("expected exactly one lowercase SHA-256 spelling")
        })
    }
}

fn hex_nibble(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        _ => unreachable!("C1Sha256 validates lowercase hexadecimal before decoding"),
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct C1ExecutionId(String);

impl C1ExecutionId {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        let bytes = value.as_bytes();
        if bytes.is_empty()
            || bytes.len() > 64
            || !bytes.first().is_some_and(u8::is_ascii_alphanumeric)
            || !bytes.last().is_some_and(u8::is_ascii_alphanumeric)
            || !bytes
                .iter()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
        {
            return None;
        }
        Some(Self(value.to_owned()))
    }

    // Retained for the frozen typed boundary; the current classifier validates only.
    #[allow(dead_code)]
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for C1ExecutionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value)
            .ok_or_else(|| serde::de::Error::custom("expected one exact C1 execution ID"))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct C1PolicyId(String);

impl C1PolicyId {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        let bytes = value.as_bytes();
        if bytes.is_empty()
            || bytes.len() > 64
            || !bytes.first().is_some_and(u8::is_ascii_alphanumeric)
            || !bytes.last().is_some_and(u8::is_ascii_alphanumeric)
            || !bytes.iter().all(|byte| {
                byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || matches!(*byte, b'.' | b'_' | b'-')
            })
        {
            return None;
        }
        Some(Self(value.to_owned()))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for C1PolicyId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value)
            .ok_or_else(|| serde::de::Error::custom("expected one exact C1 policy ID"))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct C1RequestStringId(String);

impl C1RequestStringId {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        let bytes = value.as_bytes();
        if bytes.is_empty()
            || bytes.len() > 64
            || !bytes
                .iter()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'.' | b'_' | b'-'))
        {
            return None;
        }
        Some(Self(value.to_owned()))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for C1RequestStringId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value)
            .ok_or_else(|| serde::de::Error::custom("expected one exact C1 request string ID"))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum C1Phase {
    CorrectionOperator,
    CorrectionProposal,
    CorrectionRetrieval,
    StaleActivation,
    StaleEvaluation,
    StaleTeachingEngram,
    StaleTeachingNative,
}

impl C1Phase {
    pub(crate) fn parse_exact(value: &str) -> Option<Self> {
        match value {
            "correction_operator" => Some(Self::CorrectionOperator),
            "correction_proposal" => Some(Self::CorrectionProposal),
            "correction_retrieval" => Some(Self::CorrectionRetrieval),
            "stale_activation" => Some(Self::StaleActivation),
            "stale_evaluation" => Some(Self::StaleEvaluation),
            "stale_teaching_engram" => Some(Self::StaleTeachingEngram),
            "stale_teaching_native" => Some(Self::StaleTeachingNative),
            _ => None,
        }
    }

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::CorrectionOperator => "correction_operator",
            Self::CorrectionProposal => "correction_proposal",
            Self::CorrectionRetrieval => "correction_retrieval",
            Self::StaleActivation => "stale_activation",
            Self::StaleEvaluation => "stale_evaluation",
            Self::StaleTeachingEngram => "stale_teaching_engram",
            Self::StaleTeachingNative => "stale_teaching_native",
        }
    }
}

impl<'de> Deserialize<'de> for C1Phase {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse_exact(&value)
            .ok_or_else(|| serde::de::Error::custom("expected one exact C1 phase"))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum C1PartitionPresence {
    Required,
    Forbidden,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum C1SyntheticRelayShape {
    None,
    SyntheticSingleReadV1,
    SyntheticSingleMutationV1,
    SyntheticMutationThenReadV1,
}

impl C1SyntheticRelayShape {
    #[allow(dead_code)] // The later relay-binding slice consumes this closed spelling.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::SyntheticSingleReadV1 => "synthetic_single_read_v1",
            Self::SyntheticSingleMutationV1 => "synthetic_single_mutation_v1",
            Self::SyntheticMutationThenReadV1 => "synthetic_mutation_then_read_v1",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum C1MutationJournalPresence {
    Required,
    Forbidden,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum C1ArtifactKind {
    Intent,
    Dispatch,
    Action,
    Terminal,
}

const C1_ARTIFACTS_WITH_DISPATCH: &[C1ArtifactKind] = &[
    C1ArtifactKind::Intent,
    C1ArtifactKind::Dispatch,
    C1ArtifactKind::Action,
    C1ArtifactKind::Terminal,
];
const C1_ARTIFACTS_WITHOUT_DISPATCH: &[C1ArtifactKind] = &[
    C1ArtifactKind::Intent,
    C1ArtifactKind::Action,
    C1ArtifactKind::Terminal,
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct C1PhaseArtifactPolicy {
    policy_id: C1PolicyId,
    policy_sha256: C1Sha256,
    partition_presence: C1PartitionPresence,
    relay_shape: C1SyntheticRelayShape,
    mutation_journal_presence: C1MutationJournalPresence,
    artifact_order: &'static [C1ArtifactKind],
}

impl C1PhaseArtifactPolicy {
    pub(crate) fn derive(family: NativeSuccessorFamily, phase: C1Phase) -> Option<Self> {
        use C1MutationJournalPresence::{Forbidden, Required};
        use C1PartitionPresence::{Forbidden as PartitionForbidden, Required as PartitionRequired};
        use C1SyntheticRelayShape::{
            None as NoRelay, SyntheticMutationThenReadV1, SyntheticSingleMutationV1,
            SyntheticSingleReadV1,
        };
        use NativeSuccessorFamily::{CorrectionV1, StaleIsolatedV1};

        let (partition_presence, relay_shape, mutation_journal_presence, artifact_order) =
            match (family, phase) {
                (CorrectionV1, C1Phase::CorrectionOperator) => (
                    PartitionRequired,
                    NoRelay,
                    Required,
                    C1_ARTIFACTS_WITH_DISPATCH,
                ),
                (CorrectionV1, C1Phase::CorrectionProposal) => (
                    PartitionRequired,
                    SyntheticMutationThenReadV1,
                    Required,
                    C1_ARTIFACTS_WITH_DISPATCH,
                ),
                (CorrectionV1, C1Phase::CorrectionRetrieval) => (
                    PartitionRequired,
                    SyntheticSingleReadV1,
                    Forbidden,
                    C1_ARTIFACTS_WITHOUT_DISPATCH,
                ),
                (StaleIsolatedV1, C1Phase::StaleActivation) => (
                    PartitionRequired,
                    NoRelay,
                    Required,
                    C1_ARTIFACTS_WITH_DISPATCH,
                ),
                (StaleIsolatedV1, C1Phase::StaleEvaluation) => (
                    PartitionRequired,
                    SyntheticSingleReadV1,
                    Forbidden,
                    C1_ARTIFACTS_WITHOUT_DISPATCH,
                ),
                (StaleIsolatedV1, C1Phase::StaleTeachingEngram) => (
                    PartitionRequired,
                    SyntheticSingleMutationV1,
                    Required,
                    C1_ARTIFACTS_WITH_DISPATCH,
                ),
                (StaleIsolatedV1, C1Phase::StaleTeachingNative) => (
                    PartitionForbidden,
                    NoRelay,
                    Forbidden,
                    C1_ARTIFACTS_WITHOUT_DISPATCH,
                ),
                (
                    CorrectionV1,
                    C1Phase::StaleActivation
                    | C1Phase::StaleEvaluation
                    | C1Phase::StaleTeachingEngram
                    | C1Phase::StaleTeachingNative,
                )
                | (
                    StaleIsolatedV1,
                    C1Phase::CorrectionOperator
                    | C1Phase::CorrectionProposal
                    | C1Phase::CorrectionRetrieval,
                ) => return None,
            };

        let policy_id = C1PolicyId::parse(C1_POLICY_ID)
            .expect("the frozen C1 policy ID must satisfy C1PolicyId");
        let policy_digest: [u8; 32] = Sha256::digest(C1_POLICY_CANONICAL_BYTES).into();
        Some(Self {
            policy_id,
            policy_sha256: C1Sha256::from_digest_bytes(policy_digest),
            partition_presence,
            relay_shape,
            mutation_journal_presence,
            artifact_order,
        })
    }

    pub(crate) fn policy_id(&self) -> &C1PolicyId {
        &self.policy_id
    }

    pub(crate) fn policy_sha256(&self) -> &C1Sha256 {
        &self.policy_sha256
    }

    #[allow(dead_code)] // Frozen output retained before its deferred consumer exists.
    pub(crate) const fn partition_presence(&self) -> C1PartitionPresence {
        self.partition_presence
    }

    #[allow(dead_code)] // Frozen output retained before its deferred consumer exists.
    pub(crate) const fn relay_shape(&self) -> C1SyntheticRelayShape {
        self.relay_shape
    }

    #[allow(dead_code)] // Frozen output retained before its deferred consumer exists.
    pub(crate) const fn mutation_journal_presence(&self) -> C1MutationJournalPresence {
        self.mutation_journal_presence
    }

    #[allow(dead_code)] // Frozen output retained before its deferred consumer exists.
    pub(crate) const fn artifact_order(&self) -> &'static [C1ArtifactKind] {
        self.artifact_order
    }

    #[allow(dead_code)] // Frozen output retained before its deferred consumer exists.
    pub(crate) const fn requires_partition(&self) -> bool {
        matches!(self.partition_presence, C1PartitionPresence::Required)
    }

    pub(crate) const fn requires_mutation_journal(&self) -> bool {
        matches!(
            self.mutation_journal_presence,
            C1MutationJournalPresence::Required
        )
    }

    pub(crate) const fn requires_dispatch(&self) -> bool {
        self.requires_mutation_journal()
    }
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct C1StaticToken(String);

#[cfg(test)]
impl C1StaticToken {
    pub(crate) fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() || bytes.len() > 4096 || bytes.iter().any(u8::is_ascii_control) {
            return None;
        }
        std::str::from_utf8(bytes)
            .ok()
            .map(|value| Self(value.to_owned()))
    }

    fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct C1EnvironmentKey(String);

#[cfg(test)]
impl C1EnvironmentKey {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        let mut bytes = value.bytes();
        let first = bytes.next()?;
        if value.len() > 64
            || !(first.is_ascii_uppercase() || first == b'_')
            || !bytes.all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return None;
        }
        Some(Self(value.to_owned()))
    }

    fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum C1SyntheticSlot {
    Daemon,
    Relay,
}

#[cfg(test)]
impl C1SyntheticSlot {
    const fn as_bytes(self) -> &'static [u8] {
        match self {
            Self::Daemon => b"c1_synthetic_daemon_slot",
            Self::Relay => b"c1_synthetic_relay_slot",
        }
    }
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum C1StaticEnvironmentValue {
    Literal(C1StaticToken),
    Slot(C1SyntheticSlot),
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct C1StaticLaunchTemplate {
    executable_binding_sha256: C1Sha256,
    argv: Vec<C1StaticToken>,
    working_directory_binding_sha256: C1Sha256,
    environment: Vec<(C1EnvironmentKey, C1StaticEnvironmentValue)>,
}

#[cfg(test)]
impl C1StaticLaunchTemplate {
    pub(crate) fn new(
        executable_binding_sha256: C1Sha256,
        argv: Vec<C1StaticToken>,
        working_directory_binding_sha256: C1Sha256,
        mut environment: Vec<(C1EnvironmentKey, C1StaticEnvironmentValue)>,
    ) -> Option<Self> {
        if argv.len() > 64 || environment.len() > 64 {
            return None;
        }
        environment.sort_by(|left, right| left.0.cmp(&right.0));
        if environment.windows(2).any(|pair| pair[0].0 == pair[1].0)
            || environment
                .iter()
                .any(|(key, value)| !static_environment_binding_is_valid(key, value))
        {
            return None;
        }
        Some(Self {
            executable_binding_sha256,
            argv,
            working_directory_binding_sha256,
            environment,
        })
    }

    pub(crate) fn framed_bytes(&self) -> Vec<u8> {
        let mut framed = Vec::new();
        framed.extend_from_slice(b"engram-native-static-launch-v1\0");
        framed.extend_from_slice(self.executable_binding_sha256.as_digest_bytes());
        push_u64(&mut framed, self.argv.len());
        for argument in &self.argv {
            push_len_prefixed(&mut framed, argument.as_bytes());
        }
        framed.extend_from_slice(self.working_directory_binding_sha256.as_digest_bytes());
        push_u64(&mut framed, self.environment.len());
        for (key, value) in &self.environment {
            push_len_prefixed(&mut framed, key.as_bytes());
            match value {
                C1StaticEnvironmentValue::Literal(token) => {
                    framed.push(0);
                    push_len_prefixed(&mut framed, token.as_bytes());
                }
                C1StaticEnvironmentValue::Slot(slot) => {
                    framed.push(1);
                    push_len_prefixed(&mut framed, slot.as_bytes());
                }
            }
        }
        framed
    }

    pub(crate) fn sha256(&self) -> C1Sha256 {
        let digest: [u8; 32] = Sha256::digest(self.framed_bytes()).into();
        C1Sha256::from_digest_bytes(digest)
    }
}

#[cfg(test)]
fn static_environment_binding_is_valid(
    key: &C1EnvironmentKey,
    value: &C1StaticEnvironmentValue,
) -> bool {
    match (key.as_str(), value) {
        ("C1_SYNTHETIC_DAEMON_SLOT", C1StaticEnvironmentValue::Slot(C1SyntheticSlot::Daemon))
        | ("C1_SYNTHETIC_RELAY_SLOT", C1StaticEnvironmentValue::Slot(C1SyntheticSlot::Relay)) => {
            true
        }
        ("C1_SYNTHETIC_DAEMON_SLOT" | "C1_SYNTHETIC_RELAY_SLOT", _) => false,
        (_, C1StaticEnvironmentValue::Literal(_)) => true,
        (_, C1StaticEnvironmentValue::Slot(_)) => false,
    }
}

#[cfg(test)]
fn push_u64(output: &mut Vec<u8>, value: usize) {
    output.extend_from_slice(
        &u64::try_from(value)
            .expect("bounded C1 collection length must fit u64")
            .to_be_bytes(),
    );
}

#[cfg(test)]
fn push_len_prefixed(output: &mut Vec<u8>, value: &[u8]) {
    push_u64(output, value.len());
    output.extend_from_slice(value);
}

#[cfg(test)]
mod tests {
    use super::*;

    const POLICY_SHA256: &str = "70a8ddb1722e69009c05964df6f641b219f42c580180bd59dd8f9294f6533abb";
    const STATIC_SHA256: &str = "2ce018e48ec3c82b9ff3855fa072815785adc3f200fcef1624d0a058f4477e41";

    fn token(value: &str) -> C1StaticToken {
        C1StaticToken::parse(value.as_bytes()).expect("test token must be valid")
    }

    fn key(value: &str) -> C1EnvironmentKey {
        C1EnvironmentKey::parse(value).expect("test environment key must be valid")
    }

    fn golden_template(reverse_environment: bool) -> C1StaticLaunchTemplate {
        let mut environment = vec![
            (
                key("C1_LITERAL"),
                C1StaticEnvironmentValue::Literal(token("fixture")),
            ),
            (
                key("C1_SYNTHETIC_DAEMON_SLOT"),
                C1StaticEnvironmentValue::Slot(C1SyntheticSlot::Daemon),
            ),
        ];
        if reverse_environment {
            environment.reverse();
        }
        C1StaticLaunchTemplate::new(
            C1Sha256::from_digest_bytes([0_u8; 32]),
            vec![token("c1"), token("--fixture")],
            C1Sha256::from_digest_bytes([0x11_u8; 32]),
            environment,
        )
        .expect("golden template must be valid")
    }

    #[test]
    fn sha256_newtype_enforces_exact_lowercase_spelling_and_decoding() {
        let spelling = "0123456789abcdef".repeat(4);
        let digest = C1Sha256::parse(&spelling).expect("valid digest");
        assert_eq!(digest.as_str(), spelling);
        assert_eq!(
            digest.as_digest_bytes()[0..8],
            [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]
        );

        let from_bytes = C1Sha256::from_digest_bytes([0xab; 32]);
        assert_eq!(from_bytes.as_str(), "ab".repeat(32));
        assert_eq!(C1Sha256::parse(from_bytes.as_str()), Some(from_bytes));

        for invalid in [
            "0".repeat(63),
            "0".repeat(65),
            format!("{}A", "0".repeat(63)),
            format!("{}g", "0".repeat(63)),
            format!(" {}", "0".repeat(64)),
        ] {
            assert!(C1Sha256::parse(&invalid).is_none(), "{invalid}");
            let json = serde_json::to_string(&invalid).unwrap();
            assert!(serde_json::from_str::<C1Sha256>(&json).is_err());
        }
    }

    #[test]
    fn execution_id_enforces_all_boundaries_without_normalizing() {
        for valid in ["a", "0", "a-b", "a--b", &"a".repeat(64)] {
            let parsed = C1ExecutionId::parse(valid).expect("valid execution ID");
            assert_eq!(parsed.as_str(), valid);
        }
        for invalid in [
            "",
            "-a",
            "a-",
            "A",
            "a_b",
            "a.b",
            " a",
            "a ",
            &"a".repeat(65),
        ] {
            assert!(C1ExecutionId::parse(invalid).is_none(), "{invalid:?}");
        }
        assert!(serde_json::from_str::<C1ExecutionId>(r#""A""#).is_err());
    }

    #[test]
    fn policy_id_enforces_all_boundaries_without_normalizing() {
        for valid in ["a", "a.b", "a_b", "a-b", "a._--_.b", &"a".repeat(64)] {
            let parsed = C1PolicyId::parse(valid).expect("valid policy ID");
            assert_eq!(parsed.as_str(), valid);
        }
        for invalid in [
            "",
            ".a",
            "a.",
            "_a",
            "a_",
            "-a",
            "a-",
            "A",
            "a/b",
            " a",
            &"a".repeat(65),
        ] {
            assert!(C1PolicyId::parse(invalid).is_none(), "{invalid:?}");
        }
        assert!(serde_json::from_str::<C1PolicyId>(r#""A""#).is_err());
    }

    #[test]
    fn request_string_id_enforces_exact_ascii_set_and_byte_bounds() {
        for valid in ["a", "A", "0", ".", "_", "-", "A.b_0-x", &"z".repeat(64)] {
            let parsed = C1RequestStringId::parse(valid).expect("valid request string ID");
            assert_eq!(parsed.as_str(), valid);
        }
        for invalid in ["", "a/b", "a b", "é", "\n", &"z".repeat(65)] {
            assert!(C1RequestStringId::parse(invalid).is_none(), "{invalid:?}");
        }
        assert!(serde_json::from_str::<C1RequestStringId>(r#""a b""#).is_err());
    }

    #[test]
    fn phase_parser_accepts_only_the_seven_exact_spellings() {
        let phases = [
            C1Phase::CorrectionOperator,
            C1Phase::CorrectionProposal,
            C1Phase::CorrectionRetrieval,
            C1Phase::StaleActivation,
            C1Phase::StaleEvaluation,
            C1Phase::StaleTeachingEngram,
            C1Phase::StaleTeachingNative,
        ];
        for phase in phases {
            assert_eq!(C1Phase::parse_exact(phase.as_str()), Some(phase));
            let json = format!("\"{}\"", phase.as_str());
            assert_eq!(
                serde_json::from_str::<C1Phase>(&json).expect("exact phase must deserialize"),
                phase
            );
        }
        for invalid in [
            "",
            "CorrectionOperator",
            "correction-operator",
            "correction_operator ",
            "stale_teaching",
        ] {
            assert!(C1Phase::parse_exact(invalid).is_none());
        }
    }

    #[test]
    fn policy_canonical_bytes_and_digest_match_the_frozen_golden() {
        assert_eq!(C1_POLICY_CANONICAL_BYTES.len(), 1039);
        assert_eq!(
            format!("{:x}", Sha256::digest(C1_POLICY_CANONICAL_BYTES)),
            POLICY_SHA256
        );
        assert!(C1_POLICY_CANONICAL_BYTES.starts_with(b"engram-native-phase-artifact-policy-v1\n"));
        assert_eq!(C1_POLICY_CANONICAL_BYTES.last(), Some(&b'\n'));
    }

    #[test]
    fn phase_policy_match_is_exhaustive_and_rejects_all_substitutions() {
        use C1ArtifactKind::{Action, Dispatch, Intent, Terminal};
        use C1MutationJournalPresence::{Forbidden, Required};
        use C1PartitionPresence::{Forbidden as PartitionForbidden, Required as PartitionRequired};
        use C1SyntheticRelayShape::{
            None as NoRelay, SyntheticMutationThenReadV1, SyntheticSingleMutationV1,
            SyntheticSingleReadV1,
        };
        use NativeSuccessorFamily::{CorrectionV1, StaleIsolatedV1};

        let with_dispatch = [Intent, Dispatch, Action, Terminal];
        let without_dispatch = [Intent, Action, Terminal];
        let expected = [
            (
                CorrectionV1,
                C1Phase::CorrectionOperator,
                PartitionRequired,
                NoRelay,
                Required,
                with_dispatch.as_slice(),
            ),
            (
                CorrectionV1,
                C1Phase::CorrectionProposal,
                PartitionRequired,
                SyntheticMutationThenReadV1,
                Required,
                with_dispatch.as_slice(),
            ),
            (
                CorrectionV1,
                C1Phase::CorrectionRetrieval,
                PartitionRequired,
                SyntheticSingleReadV1,
                Forbidden,
                without_dispatch.as_slice(),
            ),
            (
                StaleIsolatedV1,
                C1Phase::StaleActivation,
                PartitionRequired,
                NoRelay,
                Required,
                with_dispatch.as_slice(),
            ),
            (
                StaleIsolatedV1,
                C1Phase::StaleEvaluation,
                PartitionRequired,
                SyntheticSingleReadV1,
                Forbidden,
                without_dispatch.as_slice(),
            ),
            (
                StaleIsolatedV1,
                C1Phase::StaleTeachingEngram,
                PartitionRequired,
                SyntheticSingleMutationV1,
                Required,
                with_dispatch.as_slice(),
            ),
            (
                StaleIsolatedV1,
                C1Phase::StaleTeachingNative,
                PartitionForbidden,
                NoRelay,
                Forbidden,
                without_dispatch.as_slice(),
            ),
        ];

        for (family, phase, partition, relay, mutation, artifacts) in expected {
            let policy = C1PhaseArtifactPolicy::derive(family, phase).expect("valid pair");
            assert_eq!(policy.policy_id().as_str(), C1_POLICY_ID);
            assert_eq!(policy.policy_sha256().as_str(), POLICY_SHA256);
            assert_eq!(policy.partition_presence(), partition);
            assert_eq!(policy.relay_shape(), relay);
            assert_eq!(policy.mutation_journal_presence(), mutation);
            assert_eq!(policy.artifact_order(), artifacts);
            assert_eq!(policy.requires_partition(), partition == PartitionRequired);
            assert_eq!(policy.requires_mutation_journal(), mutation == Required);
            assert_eq!(policy.requires_dispatch(), mutation == Required);
        }

        let families = [CorrectionV1, StaleIsolatedV1];
        let phases = [
            C1Phase::CorrectionOperator,
            C1Phase::CorrectionProposal,
            C1Phase::CorrectionRetrieval,
            C1Phase::StaleActivation,
            C1Phase::StaleEvaluation,
            C1Phase::StaleTeachingEngram,
            C1Phase::StaleTeachingNative,
        ];
        for family in families {
            for phase in phases {
                let should_exist = expected.iter().any(|(valid_family, valid_phase, ..)| {
                    *valid_family == family && *valid_phase == phase
                });
                assert_eq!(
                    C1PhaseArtifactPolicy::derive(family, phase).is_some(),
                    should_exist,
                    "{family:?}/{phase:?}"
                );
            }
        }
        assert_eq!(NoRelay.as_str(), "none");
        assert_eq!(SyntheticSingleReadV1.as_str(), "synthetic_single_read_v1");
        assert_eq!(
            SyntheticSingleMutationV1.as_str(),
            "synthetic_single_mutation_v1"
        );
        assert_eq!(
            SyntheticMutationThenReadV1.as_str(),
            "synthetic_mutation_then_read_v1"
        );
    }

    #[test]
    fn static_token_enforces_utf8_byte_and_control_boundaries() {
        assert_eq!(C1StaticToken::parse(b"a").unwrap().as_bytes(), b"a");
        assert!(C1StaticToken::parse(&vec![b'a'; 4096]).is_some());
        assert!(C1StaticToken::parse("é".repeat(2048).as_bytes()).is_some());
        assert!(C1StaticToken::parse("\u{85}".as_bytes()).is_some());

        assert!(C1StaticToken::parse(b"").is_none());
        assert!(C1StaticToken::parse(&vec![b'a'; 4097]).is_none());
        assert!(C1StaticToken::parse(&[0xff]).is_none());
        for invalid in [b"a\0b".as_slice(), b"a\nb", b"a\tb", b"a\x7fb"] {
            assert!(C1StaticToken::parse(invalid).is_none());
        }
    }

    #[test]
    fn environment_key_enforces_the_exact_ascii_grammar() {
        for valid in ["A", "_", "A0", "A_B", "_A0", &"A".repeat(64)] {
            assert_eq!(C1EnvironmentKey::parse(valid).unwrap().as_str(), valid);
        }
        for invalid in ["", "0A", "a", "A-", "A.", "A B", "É", &"A".repeat(65)] {
            assert!(C1EnvironmentKey::parse(invalid).is_none(), "{invalid:?}");
        }
    }

    #[test]
    fn static_template_matches_exact_framing_and_golden_digest() {
        let template = golden_template(true);
        let framed = template.framed_bytes();

        let mut expected = Vec::new();
        expected.extend_from_slice(b"engram-native-static-launch-v1\0");
        expected.extend_from_slice(&[0_u8; 32]);
        expected.extend_from_slice(&2_u64.to_be_bytes());
        expected.extend_from_slice(&2_u64.to_be_bytes());
        expected.extend_from_slice(b"c1");
        expected.extend_from_slice(&9_u64.to_be_bytes());
        expected.extend_from_slice(b"--fixture");
        expected.extend_from_slice(&[0x11_u8; 32]);
        expected.extend_from_slice(&2_u64.to_be_bytes());
        expected.extend_from_slice(&10_u64.to_be_bytes());
        expected.extend_from_slice(b"C1_LITERAL");
        expected.push(0);
        expected.extend_from_slice(&7_u64.to_be_bytes());
        expected.extend_from_slice(b"fixture");
        expected.extend_from_slice(&24_u64.to_be_bytes());
        expected.extend_from_slice(b"C1_SYNTHETIC_DAEMON_SLOT");
        expected.push(1);
        expected.extend_from_slice(&24_u64.to_be_bytes());
        expected.extend_from_slice(b"c1_synthetic_daemon_slot");

        assert_eq!(framed, expected);
        assert_eq!(framed.len(), 237);
        assert_eq!(template.sha256().as_str(), STATIC_SHA256);
    }

    #[test]
    fn static_template_sorts_environment_and_rejects_invalid_bindings() {
        assert_eq!(
            golden_template(false).framed_bytes(),
            golden_template(true).framed_bytes()
        );

        let digest = C1Sha256::from_digest_bytes([0_u8; 32]);
        let make = |environment| {
            C1StaticLaunchTemplate::new(digest.clone(), Vec::new(), digest.clone(), environment)
        };
        assert!(make(vec![
            (
                key("DUPLICATE"),
                C1StaticEnvironmentValue::Literal(token("one")),
            ),
            (
                key("DUPLICATE"),
                C1StaticEnvironmentValue::Literal(token("two")),
            ),
        ])
        .is_none());
        assert!(make(vec![(
            key("C1_SYNTHETIC_DAEMON_SLOT"),
            C1StaticEnvironmentValue::Literal(token("fixture")),
        )])
        .is_none());
        assert!(make(vec![(
            key("C1_SYNTHETIC_DAEMON_SLOT"),
            C1StaticEnvironmentValue::Slot(C1SyntheticSlot::Relay),
        )])
        .is_none());
        assert!(make(vec![(
            key("C1_SYNTHETIC_RELAY_SLOT"),
            C1StaticEnvironmentValue::Slot(C1SyntheticSlot::Daemon),
        )])
        .is_none());
        assert!(make(vec![(
            key("ORDINARY"),
            C1StaticEnvironmentValue::Slot(C1SyntheticSlot::Relay),
        )])
        .is_none());
        assert!(make(vec![(
            key("C1_SYNTHETIC_RELAY_SLOT"),
            C1StaticEnvironmentValue::Slot(C1SyntheticSlot::Relay),
        )])
        .is_some());
    }

    #[test]
    fn static_template_enforces_collection_cardinality() {
        let digest = C1Sha256::from_digest_bytes([0_u8; 32]);
        let argv_64 = (0..64).map(|_| token("x")).collect::<Vec<_>>();
        let argv_65 = (0..65).map(|_| token("x")).collect::<Vec<_>>();
        assert!(
            C1StaticLaunchTemplate::new(digest.clone(), argv_64, digest.clone(), Vec::new(),)
                .is_some()
        );
        assert!(
            C1StaticLaunchTemplate::new(digest.clone(), argv_65, digest.clone(), Vec::new(),)
                .is_none()
        );

        let environment_64 = (0..64)
            .map(|index| {
                (
                    key(&format!("K{index}")),
                    C1StaticEnvironmentValue::Literal(token("x")),
                )
            })
            .collect::<Vec<_>>();
        let environment_65 = (0..65)
            .map(|index| {
                (
                    key(&format!("K{index}")),
                    C1StaticEnvironmentValue::Literal(token("x")),
                )
            })
            .collect::<Vec<_>>();
        assert!(C1StaticLaunchTemplate::new(
            digest.clone(),
            Vec::new(),
            digest.clone(),
            environment_64,
        )
        .is_some());
        assert!(
            C1StaticLaunchTemplate::new(digest.clone(), Vec::new(), digest, environment_65,)
                .is_none()
        );
    }

    #[test]
    fn production_region_has_no_launch_or_runtime_input_surface() {
        let source = include_str!("native_successor_policy.rs");
        let production = source
            .split_once("#[cfg(test)]")
            .expect("test-only section marker must exist")
            .0;
        for forbidden in [
            "C1StaticLaunchTemplate",
            "C1StaticToken",
            "C1EnvironmentKey",
            "C1SyntheticSlot",
            "std::env",
            "std::process",
            "std::net",
            "tokio",
            "reqwest",
            "impl From<String>",
            "PathBuf",
        ] {
            assert!(!production.contains(forbidden), "{forbidden}");
        }
    }

    #[test]
    fn c1_modules_are_private_and_reverse_isolated_from_existing_code() {
        use std::fs;
        use std::path::Path;

        let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let library = fs::read_to_string(source_root.join("lib.rs")).unwrap();
        for module in [
            "native_successor_artifact",
            "native_successor_policy",
            "native_successor_relay",
        ] {
            let private_declaration = format!("mod {module};");
            assert_eq!(library.matches(&private_declaration).count(), 1, "{module}");
            assert!(!library.contains(&format!("pub mod {module};")), "{module}");
        }

        let forbidden_reverse_references = [
            "native_successor_artifact",
            "native_successor_policy",
            "native_successor_relay",
            "C1OfflineState",
            "C1ArtifactReadError",
            "C1PhaseArtifactPolicy",
            "C1SyntheticRelay",
        ];
        for entry in fs::read_dir(&source_root).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|value| value.to_str()) != Some("rs") {
                continue;
            }
            let name = path.file_name().and_then(|value| value.to_str()).unwrap();
            if matches!(
                name,
                "lib.rs"
                    | "native_successor_artifact.rs"
                    | "native_successor_policy.rs"
                    | "native_successor_relay.rs"
            ) {
                continue;
            }
            let source = fs::read_to_string(&path).unwrap();
            for forbidden in forbidden_reverse_references {
                assert!(
                    !source.contains(forbidden),
                    "{name} contains reverse C1 reference {forbidden}"
                );
            }
        }
    }

    #[test]
    fn serialized_golden_fixtures_do_not_contain_the_runtime_canary() {
        const CANARY: &[u8] = b"C1_RUNTIME_SECRET_CANARY_7f14bff5";
        let template = golden_template(false);
        let policy = C1PhaseArtifactPolicy::derive(
            NativeSuccessorFamily::CorrectionV1,
            C1Phase::CorrectionProposal,
        )
        .unwrap();
        let framed = template.framed_bytes();
        let template_sha256 = template.sha256();
        let values = [
            C1_POLICY_CANONICAL_BYTES,
            framed.as_slice(),
            policy.policy_id().as_str().as_bytes(),
            policy.policy_sha256().as_str().as_bytes(),
            template_sha256.as_str().as_bytes(),
        ];
        for value in values {
            assert!(!value.windows(CANARY.len()).any(|window| window == CANARY));
        }
    }
}
