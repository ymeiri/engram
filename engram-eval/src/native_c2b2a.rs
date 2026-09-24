//! Provider-free host protocol foundation for native strict successor Stage C2B2a.
//!
//! This module freezes only pure values, fixed-width wire codecs, schedule validation, and the
//! host-side transcript automaton. It deliberately contains no filesystem mutation, process
//! execution, Lima integration, entropy acquisition, runtime authority, or acceptance witness.

use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use thiserror::Error;

pub const ACCEPTED_C2B2A_FREEZE_SHA256: &str =
    "85312b0286080ecbbab94b5236003a42f281b1122565016d96bb20f9572063e3";
pub const C2B2A_HOST_MAGIC: [u8; 8] = *b"ENGC2A01";
pub const C2B2A_HOST_RECEIPT_MAGIC: [u8; 8] = *b"ENGC2HR1";
pub const C2B2A_HOST_RECOVERY_MAGIC: [u8; 8] = *b"ENGC2RC1";
pub const C2B2A_PROTOCOL_VERSION: u16 = 1;
pub const C2B2A_PROFILE_ID: u16 = 1;
pub const C2B2A_HOST_HEADER_BYTES: usize = 160;
pub const C2B2A_MAX_HOST_PAYLOAD_BYTES: usize = 256;
pub const C2B2A_MAX_HOST_FRAME_BYTES: usize =
    C2B2A_HOST_HEADER_BYTES + C2B2A_MAX_HOST_PAYLOAD_BYTES;
pub const C2B2A_SELECTOR_BYTES: usize = 8;
pub const C2B2A_MEASUREMENT_BYTES: usize = 256;
pub const C2B2A_TERMINAL_STATE_BYTES: usize = 32;
pub const C2B2A_CHALLENGE_BYTES: usize = 32;
pub const C2B2A_TERMINAL_BYTES: usize = 64;
pub const C2B2A_HOST_TERMINAL_RECEIPT_BYTES: usize = 256;
pub const C2B2A_CAMPAIGN_NONCE_COUNT: usize = 11;
pub const C2B2A_CAMPAIGN_CHALLENGE_COUNT: usize = 9;
pub const C2B2A_CAMPAIGN_FRESH_VALUE_COUNT: usize =
    C2B2A_CAMPAIGN_NONCE_COUNT + C2B2A_CAMPAIGN_CHALLENGE_COUNT;

pub const C2B2A_GUEST_CPUS: u32 = 2;
pub const C2B2A_GUEST_MEMORY_BYTES: u64 = 4 * 1024 * 1024 * 1024;
pub const C2B2A_PRIVATE_DATA_DISK_BYTES: u64 = 4_294_967_296;
pub const C2B2A_INPUT_BUNDLE_MAX_BYTES: u64 = 512 * 1024 * 1024;
pub const C2B2A_RETAINED_EVIDENCE_MAX_BYTES: u64 = 64 * 1024 * 1024;
pub const C2B2A_TERMINAL_STRUCTURAL_RECEIPT_MAX_BYTES: u64 = 256 * 1024;
pub const C2B2A_PER_ROLE_OUTPUT_MAX_BYTES: u64 = 64 * 1024;
pub const C2B2A_PRE_INSTANCE_AVAILABLE_KIB: u64 = 134_217_728;
pub const C2B2A_POST_CLEANUP_AVAILABLE_KIB: u64 = 19_427_004;
pub const C2B2A_FILESYSTEM_NOISE_TOLERANCE_BYTES: u64 = 1_073_741_824;
pub const C2B2A_ROLE_DEADLINE_SECONDS: u64 = 60;
pub const C2B2A_JOURNEY_DEADLINE_SECONDS: u64 = 240;
pub const C2B2A_SHORT_GUEST_DEADLINE_SECONDS: u64 = 10 * 60;
pub const C2B2A_LONG_GUEST_DEADLINE_SECONDS: u64 = 45 * 60;
pub const C2B2A_ACCEPTANCE_DEADLINE_SECONDS: u64 = 300 * 60;
pub const C2B2A_CAMPAIGN_DEADLINE_SECONDS: u64 = 310 * 60;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum C2b2aError {
    #[error("invalid C2B2a host foundation: {0}")]
    Invalid(String),
}

pub type C2b2aResult<T> = Result<T, C2b2aError>;

fn invalid(message: impl Into<String>) -> C2b2aError {
    C2b2aError::Invalid(message.into())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct C2b2aDigest([u8; 32]);

impl C2b2aDigest {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

pub const C2B2A_CONTRACT_SHA256: C2b2aDigest = C2b2aDigest::from_bytes([
    0x85, 0x31, 0x2b, 0x02, 0x86, 0x08, 0x0e, 0xcb, 0xba, 0xb9, 0x4b, 0x52, 0x36, 0x00, 0x3a, 0x42,
    0xf2, 0x81, 0xb1, 0x12, 0x25, 0x65, 0x01, 0x6d, 0x96, 0xbb, 0x20, 0xf9, 0x57, 0x20, 0x63, 0xe3,
]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct C2b2aNonce([u8; 32]);

impl C2b2aNonce {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct C2b2aChallenge([u8; 32]);

impl C2b2aChallenge {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct C2b2aCampaignBindings {
    pub nonce: C2b2aNonce,
    pub contract_sha256: C2b2aDigest,
    pub characterization_schedule_sha256: C2b2aDigest,
    pub acceptance_schedule_sha256: C2b2aDigest,
}

/// Fixed-capacity uniqueness ledger for caller-supplied entropy outputs.
///
/// This type does not obtain entropy. A reviewed runtime must register each nonce before
/// `HostStart` and each challenge after `TerminalReady`, in serial campaign order. The single
/// shared set rejects nonce/nonce, challenge/challenge, and nonce/challenge equality.
pub struct C2b2aCampaignFreshness {
    values: [[u8; 32]; C2B2A_CAMPAIGN_FRESH_VALUE_COUNT],
    value_count: usize,
    nonce_count: usize,
    challenge_count: usize,
    poisoned: bool,
}

impl C2b2aCampaignFreshness {
    pub const fn new() -> Self {
        Self {
            values: [[0_u8; 32]; C2B2A_CAMPAIGN_FRESH_VALUE_COUNT],
            value_count: 0,
            nonce_count: 0,
            challenge_count: 0,
            poisoned: false,
        }
    }

    pub const fn value_count(&self) -> usize {
        self.value_count
    }

    pub const fn nonce_count(&self) -> usize {
        self.nonce_count
    }

    pub const fn challenge_count(&self) -> usize {
        self.challenge_count
    }

    pub fn register_nonce(&mut self, nonce: C2b2aNonce) -> C2b2aResult<()> {
        if self.poisoned {
            return Err(invalid("campaign freshness ledger is poisoned"));
        }
        if self.nonce_count == C2B2A_CAMPAIGN_NONCE_COUNT {
            self.poisoned = true;
            return Err(invalid("campaign contains an additional nonce"));
        }
        self.register(*nonce.as_bytes())?;
        self.nonce_count += 1;
        Ok(())
    }

    pub fn register_challenge(&mut self, challenge: C2b2aChallenge) -> C2b2aResult<()> {
        if self.poisoned {
            return Err(invalid("campaign freshness ledger is poisoned"));
        }
        if self.challenge_count == C2B2A_CAMPAIGN_CHALLENGE_COUNT {
            self.poisoned = true;
            return Err(invalid("campaign contains an additional challenge"));
        }
        self.register(*challenge.as_bytes())?;
        self.challenge_count += 1;
        Ok(())
    }

    pub const fn is_complete(&self) -> bool {
        !self.poisoned
            && self.nonce_count == C2B2A_CAMPAIGN_NONCE_COUNT
            && self.challenge_count == C2B2A_CAMPAIGN_CHALLENGE_COUNT
            && self.value_count == C2B2A_CAMPAIGN_FRESH_VALUE_COUNT
    }

    fn register(&mut self, value: [u8; 32]) -> C2b2aResult<()> {
        if self.values[..self.value_count].contains(&value) {
            self.poisoned = true;
            return Err(invalid("nonce or challenge was reused within the campaign"));
        }
        let Some(slot) = self.values.get_mut(self.value_count) else {
            self.poisoned = true;
            return Err(invalid("campaign freshness ledger is full"));
        };
        *slot = value;
        self.value_count += 1;
        Ok(())
    }
}

impl Default for C2b2aCampaignFreshness {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct C2b2aFrameBindings {
    nonce: C2b2aNonce,
    contract_sha256: C2b2aDigest,
    schedule_sha256: C2b2aDigest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum C2b2aInstanceName {
    C01,
    P01,
    P02,
    P03,
    B01,
    B12O,
    B12P,
    B15,
    B16,
    B40,
    B41,
}

impl C2b2aInstanceName {
    pub const fn label(self) -> &'static str {
        match self {
            Self::C01 => "C01",
            Self::P01 => "P01",
            Self::P02 => "P02",
            Self::P03 => "P03",
            Self::B01 => "B01",
            Self::B12O => "B12O",
            Self::B12P => "B12P",
            Self::B15 => "B15",
            Self::B16 => "B16",
            Self::B40 => "B40",
            Self::B41 => "B41",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct C2b2aInstanceSelector {
    pub phase_code: u16,
    pub instance_class_code: u16,
    pub instance_ordinal: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum C2b2aEvidenceAuthority {
    NonAcceptance,
    Acceptance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum C2b2aTerminalMode {
    Challenged,
    HostTerminal { probe_id: u16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct C2b2aInstanceSpec {
    pub name: C2b2aInstanceName,
    pub selector: C2b2aInstanceSelector,
    pub authority: C2b2aEvidenceAuthority,
    pub terminal_mode: C2b2aTerminalMode,
    pub measurement_count: u32,
    pub terminal_ready_sequence: Option<u32>,
    pub terminal_sequence: Option<u32>,
}

const fn challenged_spec(
    name: C2b2aInstanceName,
    phase_code: u16,
    class_code: u16,
    ordinal: u16,
    authority: C2b2aEvidenceAuthority,
    measurement_count: u32,
) -> C2b2aInstanceSpec {
    C2b2aInstanceSpec {
        name,
        selector: C2b2aInstanceSelector {
            phase_code,
            instance_class_code: class_code,
            instance_ordinal: ordinal,
        },
        authority,
        terminal_mode: C2b2aTerminalMode::Challenged,
        measurement_count,
        terminal_ready_sequence: Some(measurement_count + 2),
        terminal_sequence: Some(measurement_count + 3),
    }
}

const fn host_terminal_spec(
    name: C2b2aInstanceName,
    class_code: u16,
    ordinal: u16,
    probe_id: u16,
) -> C2b2aInstanceSpec {
    C2b2aInstanceSpec {
        name,
        selector: C2b2aInstanceSelector {
            phase_code: 2,
            instance_class_code: class_code,
            instance_ordinal: ordinal,
        },
        authority: C2b2aEvidenceAuthority::Acceptance,
        terminal_mode: C2b2aTerminalMode::HostTerminal { probe_id },
        measurement_count: 0,
        terminal_ready_sequence: None,
        terminal_sequence: None,
    }
}

pub const CHARACTERIZATION_SCHEDULE_V1: [C2b2aInstanceSpec; 1] = [challenged_spec(
    C2b2aInstanceName::C01,
    1,
    1,
    1,
    C2b2aEvidenceAuthority::NonAcceptance,
    21,
)];

pub const ACCEPTANCE_SCHEDULE_V1: [C2b2aInstanceSpec; 10] = [
    challenged_spec(
        C2b2aInstanceName::P01,
        2,
        101,
        1,
        C2b2aEvidenceAuthority::Acceptance,
        140,
    ),
    challenged_spec(
        C2b2aInstanceName::P02,
        2,
        102,
        2,
        C2b2aEvidenceAuthority::Acceptance,
        140,
    ),
    challenged_spec(
        C2b2aInstanceName::P03,
        2,
        103,
        3,
        C2b2aEvidenceAuthority::Acceptance,
        140,
    ),
    challenged_spec(
        C2b2aInstanceName::B01,
        2,
        201,
        4,
        C2b2aEvidenceAuthority::Acceptance,
        60,
    ),
    challenged_spec(
        C2b2aInstanceName::B12O,
        2,
        212,
        5,
        C2b2aEvidenceAuthority::Acceptance,
        1,
    ),
    challenged_spec(
        C2b2aInstanceName::B12P,
        2,
        213,
        6,
        C2b2aEvidenceAuthority::Acceptance,
        1,
    ),
    challenged_spec(
        C2b2aInstanceName::B15,
        2,
        215,
        7,
        C2b2aEvidenceAuthority::Acceptance,
        1,
    ),
    challenged_spec(
        C2b2aInstanceName::B16,
        2,
        216,
        8,
        C2b2aEvidenceAuthority::Acceptance,
        1,
    ),
    host_terminal_spec(C2b2aInstanceName::B40, 240, 9, 40),
    host_terminal_spec(C2b2aInstanceName::B41, 241, 10, 41),
];

pub const C2B2A_CAMPAIGN_ORDER: [C2b2aInstanceName; 11] = [
    C2b2aInstanceName::C01,
    C2b2aInstanceName::P01,
    C2b2aInstanceName::P02,
    C2b2aInstanceName::P03,
    C2b2aInstanceName::B01,
    C2b2aInstanceName::B12O,
    C2b2aInstanceName::B12P,
    C2b2aInstanceName::B15,
    C2b2aInstanceName::B16,
    C2b2aInstanceName::B40,
    C2b2aInstanceName::B41,
];

pub fn instance_spec(name: C2b2aInstanceName) -> C2b2aInstanceSpec {
    CHARACTERIZATION_SCHEDULE_V1
        .iter()
        .chain(ACCEPTANCE_SCHEDULE_V1.iter())
        .copied()
        .find(|spec| spec.name == name)
        .expect("every closed C2B2a instance name has one compiled schedule row")
}

pub fn instance_for_selector(selector: C2b2aInstanceSelector) -> C2b2aResult<C2b2aInstanceSpec> {
    CHARACTERIZATION_SCHEDULE_V1
        .iter()
        .chain(ACCEPTANCE_SCHEDULE_V1.iter())
        .copied()
        .find(|spec| spec.selector == selector)
        .ok_or_else(|| invalid("phase/class/ordinal selector is not one compiled schedule row"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum C2b2aRoleId {
    Supervisor = 0,
    Boundary = 1,
    Writer = 2,
    Contender = 3,
    Release = 4,
    Reader = 5,
    JourneyAggregate = 6,
}

impl TryFrom<u16> for C2b2aRoleId {
    type Error = C2b2aError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Supervisor),
            1 => Ok(Self::Boundary),
            2 => Ok(Self::Writer),
            3 => Ok(Self::Contender),
            4 => Ok(Self::Release),
            5 => Ok(Self::Reader),
            6 => Ok(Self::JourneyAggregate),
            _ => Err(invalid("unknown host-control role id")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct C2b2aMeasurementKey {
    pub case_or_probe_id: u16,
    pub fixture_or_subattempt_id: u16,
    pub role_id: C2b2aRoleId,
    pub scope_code: u64,
}

fn positive_scope_role(scope_code: u64) -> Option<C2b2aRoleId> {
    match scope_code {
        1 | 3 => Some(C2b2aRoleId::Writer),
        2 => Some(C2b2aRoleId::Contender),
        4 | 5 => Some(C2b2aRoleId::Release),
        6 => Some(C2b2aRoleId::Reader),
        7 => Some(C2b2aRoleId::JourneyAggregate),
        _ => None,
    }
}

fn b01_subattempt_count(probe_id: u16) -> u16 {
    match probe_id {
        12 | 15 | 16 | 40..=u16::MAX => 0,
        32 => 2,
        33 => 14,
        35 => 8,
        36 => 4,
        1..=39 => 1,
        _ => 0,
    }
}

fn b01_measurement_at(mut index: usize) -> Option<C2b2aMeasurementKey> {
    for probe_id in 1_u16..=39 {
        let count = usize::from(b01_subattempt_count(probe_id));
        if index < count {
            return Some(C2b2aMeasurementKey {
                case_or_probe_id: probe_id,
                fixture_or_subattempt_id: u16::try_from(index + 1).ok()?,
                role_id: C2b2aRoleId::Boundary,
                scope_code: 8,
            });
        }
        index -= count;
    }
    None
}

pub fn expected_measurement_at(
    instance: C2b2aInstanceName,
    index: usize,
) -> Option<C2b2aMeasurementKey> {
    match instance {
        C2b2aInstanceName::C01 if index < 21 => {
            let subattempt_id = u16::try_from(index / 7 + 1).ok()?;
            let scope_code = u64::try_from(index % 7 + 1).ok()?;
            Some(C2b2aMeasurementKey {
                case_or_probe_id: 1,
                fixture_or_subattempt_id: subattempt_id,
                role_id: positive_scope_role(scope_code)?,
                scope_code,
            })
        }
        C2b2aInstanceName::P01 | C2b2aInstanceName::P02 | C2b2aInstanceName::P03 if index < 140 => {
            let case_id = u16::try_from(index / 7 + 1).ok()?;
            let scope_code = u64::try_from(index % 7 + 1).ok()?;
            Some(C2b2aMeasurementKey {
                case_or_probe_id: case_id,
                fixture_or_subattempt_id: case_id,
                role_id: positive_scope_role(scope_code)?,
                scope_code,
            })
        }
        C2b2aInstanceName::B01 => b01_measurement_at(index),
        C2b2aInstanceName::B12O if index == 0 => Some(C2b2aMeasurementKey {
            case_or_probe_id: 12,
            fixture_or_subattempt_id: 1,
            role_id: C2b2aRoleId::Boundary,
            scope_code: 8,
        }),
        C2b2aInstanceName::B12P if index == 0 => Some(C2b2aMeasurementKey {
            case_or_probe_id: 12,
            fixture_or_subattempt_id: 2,
            role_id: C2b2aRoleId::Boundary,
            scope_code: 8,
        }),
        C2b2aInstanceName::B15 if index == 0 => Some(C2b2aMeasurementKey {
            case_or_probe_id: 15,
            fixture_or_subattempt_id: 1,
            role_id: C2b2aRoleId::Boundary,
            scope_code: 8,
        }),
        C2b2aInstanceName::B16 if index == 0 => Some(C2b2aMeasurementKey {
            case_or_probe_id: 16,
            fixture_or_subattempt_id: 1,
            role_id: C2b2aRoleId::Boundary,
            scope_code: 8,
        }),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum C2b2aHostKind {
    Abort = 12,
    HostStart = 19,
    HostReady = 20,
    Measurement = 21,
    Challenge = 22,
    Terminal = 23,
    TerminalReady = 24,
}

impl TryFrom<u16> for C2b2aHostKind {
    type Error = C2b2aError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            12 => Ok(Self::Abort),
            19 => Ok(Self::HostStart),
            20 => Ok(Self::HostReady),
            21 => Ok(Self::Measurement),
            22 => Ok(Self::Challenge),
            23 => Ok(Self::Terminal),
            24 => Ok(Self::TerminalReady),
            _ => Err(invalid("unknown host-control frame kind")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum C2b2aPrimaryCategory {
    Success = 0,
    HostContract = 1,
    GuestConfiguration = 2,
    Containment = 3,
    StaticFirewall = 4,
    LimitExceeded = 5,
    InvalidFixture = 6,
    StoreOpenFailure = 7,
    OutcomeUncertain = 8,
    ExclusivityBroken = 9,
    ContenderFailure = 10,
    ReleaseUnproven = 11,
    ReopenFailure = 12,
    EngineReadFailure = 13,
    CanonicalMismatch = 14,
    MeasurementFailure = 15,
    ProtocolFailure = 16,
    ChallengeFailure = 17,
}

impl C2b2aPrimaryCategory {
    pub const fn code(self) -> u32 {
        self as u32
    }

    fn header_status(self) -> u16 {
        u16::try_from(self.code()).expect("all frozen category codes fit u16")
    }
}

impl TryFrom<u32> for C2b2aPrimaryCategory {
    type Error = C2b2aError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Success),
            1 => Ok(Self::HostContract),
            2 => Ok(Self::GuestConfiguration),
            3 => Ok(Self::Containment),
            4 => Ok(Self::StaticFirewall),
            5 => Ok(Self::LimitExceeded),
            6 => Ok(Self::InvalidFixture),
            7 => Ok(Self::StoreOpenFailure),
            8 => Ok(Self::OutcomeUncertain),
            9 => Ok(Self::ExclusivityBroken),
            10 => Ok(Self::ContenderFailure),
            11 => Ok(Self::ReleaseUnproven),
            12 => Ok(Self::ReopenFailure),
            13 => Ok(Self::EngineReadFailure),
            14 => Ok(Self::CanonicalMismatch),
            15 => Ok(Self::MeasurementFailure),
            16 => Ok(Self::ProtocolFailure),
            17 => Ok(Self::ChallengeFailure),
            _ => Err(invalid("unknown terminal primary category")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct C2b2aTerminalState {
    pub primary_category: C2b2aPrimaryCategory,
    pub active_item_code: u32,
    pub guest_cleanup_proven: bool,
    pub write_may_have_been_dispatched: bool,
    pub writer_acknowledged: bool,
    pub roles_reaped: bool,
    pub completed_case_count: u32,
    pub completed_subattempt_count: u32,
}

impl C2b2aTerminalState {
    pub fn success_for(instance: C2b2aInstanceName) -> C2b2aResult<Self> {
        let (dispatched, acknowledged, completed_cases, completed_subattempts) = match instance {
            C2b2aInstanceName::C01 => (true, true, 0, 3),
            C2b2aInstanceName::P01 | C2b2aInstanceName::P02 | C2b2aInstanceName::P03 => {
                (true, true, 20, 0)
            }
            C2b2aInstanceName::B01 => (false, false, 0, 60),
            C2b2aInstanceName::B12O
            | C2b2aInstanceName::B12P
            | C2b2aInstanceName::B15
            | C2b2aInstanceName::B16 => (false, false, 0, 1),
            C2b2aInstanceName::B40 | C2b2aInstanceName::B41 => {
                return Err(invalid(
                    "host-terminal instances have no guest terminal state",
                ));
            }
        };
        Ok(Self {
            primary_category: C2b2aPrimaryCategory::Success,
            active_item_code: 0,
            guest_cleanup_proven: true,
            write_may_have_been_dispatched: dispatched,
            writer_acknowledged: acknowledged,
            roles_reaped: true,
            completed_case_count: completed_cases,
            completed_subattempt_count: completed_subattempts,
        })
    }

    pub fn active_item(case_or_probe_id: u16, fixture_or_subattempt_id: u16) -> C2b2aResult<u32> {
        if case_or_probe_id == 0 || fixture_or_subattempt_id == 0 {
            return Err(invalid("active item components must both be nonzero"));
        }
        Ok((u32::from(case_or_probe_id) << 16) | u32::from(fixture_or_subattempt_id))
    }

    pub fn active_item_parts(self) -> C2b2aResult<Option<(u16, u16)>> {
        if self.active_item_code == 0 {
            return Ok(None);
        }
        let first = u16::try_from(self.active_item_code >> 16)
            .map_err(|_| invalid("active item high word does not fit u16"))?;
        let second = u16::try_from(self.active_item_code & 0xffff)
            .map_err(|_| invalid("active item low word does not fit u16"))?;
        if first == 0 || second == 0 {
            return Err(invalid("active item has a zero component"));
        }
        Ok(Some((first, second)))
    }

    pub fn encode(self) -> [u8; C2B2A_TERMINAL_STATE_BYTES] {
        let fields = [
            self.primary_category.code(),
            self.active_item_code,
            u32::from(self.guest_cleanup_proven),
            u32::from(self.write_may_have_been_dispatched),
            u32::from(self.writer_acknowledged),
            u32::from(self.roles_reaped),
            self.completed_case_count,
            self.completed_subattempt_count,
        ];
        let mut bytes = [0_u8; C2B2A_TERMINAL_STATE_BYTES];
        for (index, field) in fields.iter().enumerate() {
            put_u32(&mut bytes, index * 4, *field);
        }
        bytes
    }

    pub fn decode(bytes: &[u8; C2B2A_TERMINAL_STATE_BYTES]) -> C2b2aResult<Self> {
        let boolean = |offset| -> C2b2aResult<bool> {
            match read_u32(bytes, offset) {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(invalid("terminal boolean field is not zero or one")),
            }
        };
        let state = Self {
            primary_category: C2b2aPrimaryCategory::try_from(read_u32(bytes, 0))?,
            active_item_code: read_u32(bytes, 4),
            guest_cleanup_proven: boolean(8)?,
            write_may_have_been_dispatched: boolean(12)?,
            writer_acknowledged: boolean(16)?,
            roles_reaped: boolean(20)?,
            completed_case_count: read_u32(bytes, 24),
            completed_subattempt_count: read_u32(bytes, 28),
        };
        state.validate_common()?;
        Ok(state)
    }

    fn validate_common(self) -> C2b2aResult<()> {
        if self.guest_cleanup_proven && !self.roles_reaped {
            return Err(invalid(
                "guest cleanup cannot be proven before every role is reaped",
            ));
        }
        if self.writer_acknowledged && !self.write_may_have_been_dispatched {
            return Err(invalid(
                "writer acknowledgement cannot precede write dispatch",
            ));
        }
        let outcome_is_uncertain = self.primary_category == C2b2aPrimaryCategory::OutcomeUncertain;
        if self.write_may_have_been_dispatched && !self.writer_acknowledged {
            if !outcome_is_uncertain {
                return Err(invalid(
                    "dispatch without writer acknowledgement requires OutcomeUncertain",
                ));
            }
        } else if outcome_is_uncertain {
            return Err(invalid(
                "OutcomeUncertain requires dispatch without writer acknowledgement",
            ));
        }
        let category_is_legal_for_phase = if !self.write_may_have_been_dispatched {
            matches!(
                self.primary_category,
                C2b2aPrimaryCategory::Success
                    | C2b2aPrimaryCategory::HostContract
                    | C2b2aPrimaryCategory::GuestConfiguration
                    | C2b2aPrimaryCategory::Containment
                    | C2b2aPrimaryCategory::StaticFirewall
                    | C2b2aPrimaryCategory::LimitExceeded
                    | C2b2aPrimaryCategory::InvalidFixture
                    | C2b2aPrimaryCategory::StoreOpenFailure
                    | C2b2aPrimaryCategory::MeasurementFailure
                    | C2b2aPrimaryCategory::ProtocolFailure
                    | C2b2aPrimaryCategory::ChallengeFailure
            )
        } else if !self.writer_acknowledged {
            outcome_is_uncertain
        } else {
            matches!(
                self.primary_category,
                C2b2aPrimaryCategory::Success
                    | C2b2aPrimaryCategory::HostContract
                    | C2b2aPrimaryCategory::GuestConfiguration
                    | C2b2aPrimaryCategory::Containment
                    | C2b2aPrimaryCategory::LimitExceeded
                    | C2b2aPrimaryCategory::ExclusivityBroken
                    | C2b2aPrimaryCategory::ContenderFailure
                    | C2b2aPrimaryCategory::ReleaseUnproven
                    | C2b2aPrimaryCategory::ReopenFailure
                    | C2b2aPrimaryCategory::EngineReadFailure
                    | C2b2aPrimaryCategory::CanonicalMismatch
                    | C2b2aPrimaryCategory::MeasurementFailure
                    | C2b2aPrimaryCategory::ProtocolFailure
                    | C2b2aPrimaryCategory::ChallengeFailure
            )
        };
        if !category_is_legal_for_phase {
            return Err(invalid(
                "primary category is impossible for the recorded write phase",
            ));
        }
        self.active_item_parts()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct C2b2aMeasurement {
    pub scope_code: u64,
    pub memory_current_bytes: u64,
    pub memory_peak_bytes: u64,
    pub memory_events_oom: u64,
    pub memory_events_oom_kill: u64,
    pub swap_current_bytes: u64,
    pub swap_peak_bytes: u64,
    pub swap_events_max: u64,
    pub pids_current: u64,
    pub pids_peak: u64,
    pub pids_events_max: u64,
    pub cpu_usage_usec: u64,
    pub cpu_user_usec: u64,
    pub cpu_system_usec: u64,
    pub cpu_nr_periods: u64,
    pub cpu_nr_throttled: u64,
    pub cpu_throttled_usec: u64,
    pub filesystem_logical_bytes: u64,
    pub filesystem_allocated_bytes: u64,
    pub filesystem_inode_count: u64,
    pub filesystem_file_count: u64,
    pub tmpfs_bytes: u64,
    pub tmpfs_inode_count: u64,
    pub diagnostic_bytes: u64,
    pub diagnostic_inode_count: u64,
    pub stdout_bytes: u64,
    pub stderr_bytes: u64,
    pub role_control_input_bytes: u64,
    pub role_control_output_bytes: u64,
    pub descriptor_count: u64,
    pub descriptor_type_bitmap: u64,
    pub monotonic_elapsed_ns: u64,
}

impl C2b2aMeasurement {
    pub fn fields(self) -> [u64; 32] {
        [
            self.scope_code,
            self.memory_current_bytes,
            self.memory_peak_bytes,
            self.memory_events_oom,
            self.memory_events_oom_kill,
            self.swap_current_bytes,
            self.swap_peak_bytes,
            self.swap_events_max,
            self.pids_current,
            self.pids_peak,
            self.pids_events_max,
            self.cpu_usage_usec,
            self.cpu_user_usec,
            self.cpu_system_usec,
            self.cpu_nr_periods,
            self.cpu_nr_throttled,
            self.cpu_throttled_usec,
            self.filesystem_logical_bytes,
            self.filesystem_allocated_bytes,
            self.filesystem_inode_count,
            self.filesystem_file_count,
            self.tmpfs_bytes,
            self.tmpfs_inode_count,
            self.diagnostic_bytes,
            self.diagnostic_inode_count,
            self.stdout_bytes,
            self.stderr_bytes,
            self.role_control_input_bytes,
            self.role_control_output_bytes,
            self.descriptor_count,
            self.descriptor_type_bitmap,
            self.monotonic_elapsed_ns,
        ]
    }

    pub fn from_fields(fields: [u64; 32]) -> Self {
        Self {
            scope_code: fields[0],
            memory_current_bytes: fields[1],
            memory_peak_bytes: fields[2],
            memory_events_oom: fields[3],
            memory_events_oom_kill: fields[4],
            swap_current_bytes: fields[5],
            swap_peak_bytes: fields[6],
            swap_events_max: fields[7],
            pids_current: fields[8],
            pids_peak: fields[9],
            pids_events_max: fields[10],
            cpu_usage_usec: fields[11],
            cpu_user_usec: fields[12],
            cpu_system_usec: fields[13],
            cpu_nr_periods: fields[14],
            cpu_nr_throttled: fields[15],
            cpu_throttled_usec: fields[16],
            filesystem_logical_bytes: fields[17],
            filesystem_allocated_bytes: fields[18],
            filesystem_inode_count: fields[19],
            filesystem_file_count: fields[20],
            tmpfs_bytes: fields[21],
            tmpfs_inode_count: fields[22],
            diagnostic_bytes: fields[23],
            diagnostic_inode_count: fields[24],
            stdout_bytes: fields[25],
            stderr_bytes: fields[26],
            role_control_input_bytes: fields[27],
            role_control_output_bytes: fields[28],
            descriptor_count: fields[29],
            descriptor_type_bitmap: fields[30],
            monotonic_elapsed_ns: fields[31],
        }
    }

    pub fn encode(self) -> [u8; C2B2A_MEASUREMENT_BYTES] {
        let mut bytes = [0_u8; C2B2A_MEASUREMENT_BYTES];
        for (index, field) in self.fields().iter().enumerate() {
            put_u64(&mut bytes, index * 8, *field);
        }
        bytes
    }

    pub fn decode(bytes: &[u8; C2B2A_MEASUREMENT_BYTES]) -> C2b2aResult<Self> {
        let mut fields = [0_u64; 32];
        for (index, field) in fields.iter_mut().enumerate() {
            *field = read_u64(bytes, index * 8);
        }
        let measurement = Self::from_fields(fields);
        if !(1..=8).contains(&measurement.scope_code) {
            return Err(invalid("measurement scope is outside the frozen range"));
        }
        Ok(measurement)
    }

    fn validate_for_key(self, key: C2b2aMeasurementKey) -> C2b2aResult<()> {
        if self.scope_code != key.scope_code {
            return Err(invalid(
                "measurement scope does not match the compiled schedule",
            ));
        }
        let is_boundary_probe = key.role_id == C2b2aRoleId::Boundary && key.scope_code == 8;
        let forbidden_high_bits = self.descriptor_type_bitmap & !((1_u64 << 13) - 1);
        if is_boundary_probe
            && key.case_or_probe_id == 17
            && self.descriptor_type_bitmap & (1_u64 << 7) == 0
        {
            return Err(invalid(
                "probe 17 requires the diagnostic regular-file descriptor bit",
            ));
        }
        if is_boundary_probe && key.case_or_probe_id == 18 {
            if forbidden_high_bits != 1_u64 << 14 {
                return Err(invalid(
                    "probe 18 requires bit 14 and forbids every other high descriptor bit",
                ));
            }
        } else if forbidden_high_bits != 0 {
            return Err(invalid(
                "measurement contains a forbidden descriptor type bit",
            ));
        }
        if is_boundary_probe && matches!(key.case_or_probe_id, 15 | 16) {
            let fields = self.fields();
            if fields[1..31].iter().any(|field| *field != 0) {
                return Err(invalid(
                    "pre-Start probe measurement has a nonzero inapplicable field",
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum C2b2aDirection {
    HostToSupervisor,
    SupervisorToHost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct C2b2aHostHeader {
    pub kind: C2b2aHostKind,
    pub sequence: u32,
    pub case_or_probe_id: u16,
    pub fixture_or_subattempt_id: u16,
    pub role_id: C2b2aRoleId,
    pub status: u16,
    pub payload_len: u32,
    pub nonce: C2b2aNonce,
    pub contract_sha256: C2b2aDigest,
    pub schedule_sha256: C2b2aDigest,
    pub payload_sha256: C2b2aDigest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct C2b2aEncodedFrame {
    header: [u8; C2B2A_HOST_HEADER_BYTES],
    payload: [u8; C2B2A_MAX_HOST_PAYLOAD_BYTES],
    payload_len: usize,
}

impl C2b2aEncodedFrame {
    pub const fn header(&self) -> &[u8; C2B2A_HOST_HEADER_BYTES] {
        &self.header
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload[..self.payload_len]
    }

    pub const fn payload_len(&self) -> usize {
        self.payload_len
    }

    pub const fn wire_len(&self) -> usize {
        C2B2A_HOST_HEADER_BYTES + self.payload_len
    }

    pub fn copy_wire_into(&self, destination: &mut [u8; C2B2A_MAX_HOST_FRAME_BYTES]) -> usize {
        destination.fill(0);
        destination[..C2B2A_HOST_HEADER_BYTES].copy_from_slice(&self.header);
        destination[C2B2A_HOST_HEADER_BYTES..self.wire_len()].copy_from_slice(self.payload());
        self.wire_len()
    }
}

// Boxing the measurement would remove `Copy` and change the frozen in-process frame representation.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum C2b2aDecodedPayload {
    HostStart(C2b2aInstanceSelector),
    HostReady(C2b2aInstanceSelector),
    Measurement(C2b2aMeasurement),
    Challenge(C2b2aChallenge),
    TerminalReady(C2b2aTerminalState),
    Terminal {
        challenge: C2b2aChallenge,
        state: C2b2aTerminalState,
    },
    Abort(C2b2aTerminalState),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct C2b2aDecodedFrame {
    pub header: C2b2aHostHeader,
    pub payload: C2b2aDecodedPayload,
}

fn payload_sha256(payload: &[u8]) -> C2b2aDigest {
    let mut hasher = Sha256::new();
    hasher.update(payload);
    C2b2aDigest::from_bytes(hasher.finalize().into())
}

fn selected_bindings(
    instance: C2b2aInstanceName,
    campaign: C2b2aCampaignBindings,
) -> C2b2aFrameBindings {
    let schedule_sha256 = if instance == C2b2aInstanceName::C01 {
        campaign.characterization_schedule_sha256
    } else {
        campaign.acceptance_schedule_sha256
    };
    C2b2aFrameBindings {
        nonce: campaign.nonce,
        contract_sha256: campaign.contract_sha256,
        schedule_sha256,
    }
}

fn encode_selector(selector: C2b2aInstanceSelector) -> [u8; C2B2A_SELECTOR_BYTES] {
    let mut payload = [0_u8; C2B2A_SELECTOR_BYTES];
    put_u16(&mut payload, 0, selector.phase_code);
    put_u16(&mut payload, 2, selector.instance_class_code);
    put_u16(&mut payload, 4, selector.instance_ordinal);
    put_u16(&mut payload, 6, 0);
    payload
}

fn decode_selector(payload: &[u8; C2B2A_SELECTOR_BYTES]) -> C2b2aResult<C2b2aInstanceSelector> {
    if read_u16(payload, 6) != 0 {
        return Err(invalid(
            "HostStart/HostReady selector reserved field is nonzero",
        ));
    }
    let selector = C2b2aInstanceSelector {
        phase_code: read_u16(payload, 0),
        instance_class_code: read_u16(payload, 2),
        instance_ordinal: read_u16(payload, 4),
    };
    instance_for_selector(selector)?;
    Ok(selector)
}

fn encode_frame(
    kind: C2b2aHostKind,
    sequence: u32,
    case_or_probe_id: u16,
    fixture_or_subattempt_id: u16,
    role_id: C2b2aRoleId,
    status: u16,
    bindings: C2b2aFrameBindings,
    payload: &[u8],
) -> C2b2aResult<C2b2aEncodedFrame> {
    if sequence == 0 {
        return Err(invalid("host-control sequence must be nonzero"));
    }
    if payload.len() > C2B2A_MAX_HOST_PAYLOAD_BYTES {
        return Err(invalid("host-control payload exceeds 256 bytes"));
    }
    let payload_len = u32::try_from(payload.len())
        .map_err(|_| invalid("host-control payload length does not fit u32"))?;
    let payload_digest = payload_sha256(payload);
    let mut header = [0_u8; C2B2A_HOST_HEADER_BYTES];
    header[0..8].copy_from_slice(&C2B2A_HOST_MAGIC);
    put_u16(&mut header, 8, C2B2A_PROTOCOL_VERSION);
    put_u16(&mut header, 10, kind as u16);
    put_u32(&mut header, 12, sequence);
    put_u16(&mut header, 16, case_or_probe_id);
    put_u16(&mut header, 18, fixture_or_subattempt_id);
    put_u16(&mut header, 20, C2B2A_PROFILE_ID);
    put_u16(&mut header, 22, role_id as u16);
    put_u16(&mut header, 24, status);
    put_u16(&mut header, 26, 0);
    put_u32(&mut header, 28, payload_len);
    header[32..64].copy_from_slice(bindings.nonce.as_bytes());
    header[64..96].copy_from_slice(bindings.contract_sha256.as_bytes());
    header[96..128].copy_from_slice(bindings.schedule_sha256.as_bytes());
    header[128..160].copy_from_slice(payload_digest.as_bytes());
    let mut storage = [0_u8; C2B2A_MAX_HOST_PAYLOAD_BYTES];
    storage[..payload.len()].copy_from_slice(payload);
    Ok(C2b2aEncodedFrame {
        header,
        payload: storage,
        payload_len: payload.len(),
    })
}

pub fn encode_host_start(
    instance: C2b2aInstanceName,
    campaign: C2b2aCampaignBindings,
) -> C2b2aResult<C2b2aEncodedFrame> {
    let payload = encode_selector(instance_spec(instance).selector);
    encode_frame(
        C2b2aHostKind::HostStart,
        1,
        0,
        0,
        C2b2aRoleId::Supervisor,
        0,
        selected_bindings(instance, campaign),
        &payload,
    )
}

pub fn encode_host_ready(
    instance: C2b2aInstanceName,
    campaign: C2b2aCampaignBindings,
) -> C2b2aResult<C2b2aEncodedFrame> {
    let payload = encode_selector(instance_spec(instance).selector);
    encode_frame(
        C2b2aHostKind::HostReady,
        1,
        0,
        0,
        C2b2aRoleId::Supervisor,
        0,
        selected_bindings(instance, campaign),
        &payload,
    )
}

pub fn encode_measurement(
    instance: C2b2aInstanceName,
    campaign: C2b2aCampaignBindings,
    sequence: u32,
    key: C2b2aMeasurementKey,
    measurement: C2b2aMeasurement,
) -> C2b2aResult<C2b2aEncodedFrame> {
    let payload = measurement.encode();
    encode_frame(
        C2b2aHostKind::Measurement,
        sequence,
        key.case_or_probe_id,
        key.fixture_or_subattempt_id,
        key.role_id,
        0,
        selected_bindings(instance, campaign),
        &payload,
    )
}

pub fn encode_terminal_ready(
    instance: C2b2aInstanceName,
    campaign: C2b2aCampaignBindings,
    sequence: u32,
    state: C2b2aTerminalState,
) -> C2b2aResult<C2b2aEncodedFrame> {
    let payload = state.encode();
    encode_frame(
        C2b2aHostKind::TerminalReady,
        sequence,
        0,
        0,
        C2b2aRoleId::Supervisor,
        state.primary_category.header_status(),
        selected_bindings(instance, campaign),
        &payload,
    )
}

pub fn encode_challenge(
    instance: C2b2aInstanceName,
    campaign: C2b2aCampaignBindings,
    challenge: C2b2aChallenge,
) -> C2b2aResult<C2b2aEncodedFrame> {
    encode_frame(
        C2b2aHostKind::Challenge,
        2,
        0,
        0,
        C2b2aRoleId::Supervisor,
        0,
        selected_bindings(instance, campaign),
        challenge.as_bytes(),
    )
}

pub fn encode_terminal(
    instance: C2b2aInstanceName,
    campaign: C2b2aCampaignBindings,
    sequence: u32,
    challenge: C2b2aChallenge,
    state: C2b2aTerminalState,
) -> C2b2aResult<C2b2aEncodedFrame> {
    let mut payload = [0_u8; C2B2A_TERMINAL_BYTES];
    payload[..C2B2A_CHALLENGE_BYTES].copy_from_slice(challenge.as_bytes());
    payload[C2B2A_CHALLENGE_BYTES..].copy_from_slice(&state.encode());
    encode_frame(
        C2b2aHostKind::Terminal,
        sequence,
        0,
        0,
        C2b2aRoleId::Supervisor,
        state.primary_category.header_status(),
        selected_bindings(instance, campaign),
        &payload,
    )
}

pub fn encode_supervisor_abort(
    instance: C2b2aInstanceName,
    campaign: C2b2aCampaignBindings,
    sequence: u32,
    state: C2b2aTerminalState,
) -> C2b2aResult<C2b2aEncodedFrame> {
    if state.primary_category == C2b2aPrimaryCategory::Success {
        return Err(invalid("Abort must carry a nonzero primary category"));
    }
    let (case_or_probe_id, fixture_or_subattempt_id) = state.active_item_parts()?.unwrap_or((0, 0));
    encode_frame(
        C2b2aHostKind::Abort,
        sequence,
        case_or_probe_id,
        fixture_or_subattempt_id,
        C2b2aRoleId::Supervisor,
        state.primary_category.header_status(),
        selected_bindings(instance, campaign),
        &state.encode(),
    )
}

fn parse_header(bytes: &[u8; C2B2A_HOST_HEADER_BYTES]) -> C2b2aResult<C2b2aHostHeader> {
    if bytes[0..8] != C2B2A_HOST_MAGIC {
        return Err(invalid("host-control magic mismatch"));
    }
    if read_u16(bytes, 8) != C2B2A_PROTOCOL_VERSION {
        return Err(invalid("host-control version mismatch"));
    }
    if read_u16(bytes, 20) != C2B2A_PROFILE_ID {
        return Err(invalid("host-control profile id is not one"));
    }
    if read_u16(bytes, 26) != 0 {
        return Err(invalid("host-control flags are nonzero"));
    }
    let sequence = read_u32(bytes, 12);
    if sequence == 0 {
        return Err(invalid("host-control sequence is zero"));
    }
    let payload_len = read_u32(bytes, 28);
    if usize::try_from(payload_len)
        .map_err(|_| invalid("host-control payload length does not fit usize"))?
        > C2B2A_MAX_HOST_PAYLOAD_BYTES
    {
        return Err(invalid("host-control payload length exceeds 256 bytes"));
    }
    let header = C2b2aHostHeader {
        kind: C2b2aHostKind::try_from(read_u16(bytes, 10))?,
        sequence,
        case_or_probe_id: read_u16(bytes, 16),
        fixture_or_subattempt_id: read_u16(bytes, 18),
        role_id: C2b2aRoleId::try_from(read_u16(bytes, 22))?,
        status: read_u16(bytes, 24),
        payload_len,
        nonce: C2b2aNonce::from_bytes(array_32(bytes, 32)),
        contract_sha256: C2b2aDigest::from_bytes(array_32(bytes, 64)),
        schedule_sha256: C2b2aDigest::from_bytes(array_32(bytes, 96)),
        payload_sha256: C2b2aDigest::from_bytes(array_32(bytes, 128)),
    };
    if header.contract_sha256 != C2B2A_CONTRACT_SHA256 {
        return Err(invalid(
            "host-control contract digest is not the accepted C2B2a contract",
        ));
    }
    Ok(header)
}

pub fn decode_host_control_frame(
    direction: C2b2aDirection,
    header_bytes: &[u8; C2B2A_HOST_HEADER_BYTES],
    payload_bytes: &[u8],
) -> C2b2aResult<C2b2aDecodedFrame> {
    let header = parse_header(header_bytes)?;
    let declared = usize::try_from(header.payload_len)
        .map_err(|_| invalid("host-control payload length does not fit usize"))?;
    if declared != payload_bytes.len() {
        return Err(invalid(
            "host-control payload is missing bytes or contains trailing bytes",
        ));
    }
    if payload_sha256(payload_bytes) != header.payload_sha256 {
        return Err(invalid("host-control payload digest mismatch"));
    }
    let direction_is_valid = match direction {
        C2b2aDirection::HostToSupervisor => matches!(
            header.kind,
            C2b2aHostKind::HostStart | C2b2aHostKind::Challenge | C2b2aHostKind::Abort
        ),
        C2b2aDirection::SupervisorToHost => matches!(
            header.kind,
            C2b2aHostKind::HostReady
                | C2b2aHostKind::Measurement
                | C2b2aHostKind::TerminalReady
                | C2b2aHostKind::Terminal
                | C2b2aHostKind::Abort
        ),
    };
    if !direction_is_valid {
        return Err(invalid("host-control kind is illegal in this direction"));
    }

    let payload = match header.kind {
        C2b2aHostKind::HostStart | C2b2aHostKind::HostReady => {
            require_payload_len(payload_bytes, C2B2A_SELECTOR_BYTES)?;
            if header.sequence != 1 {
                return Err(invalid("HostStart/HostReady sequence is not one"));
            }
            let selector = decode_selector(&array_8(payload_bytes, 0))?;
            require_zero_item_shape(&header, true)?;
            if header.status != 0 {
                return Err(invalid("HostStart/HostReady status is nonzero"));
            }
            if header.kind == C2b2aHostKind::HostStart {
                C2b2aDecodedPayload::HostStart(selector)
            } else {
                C2b2aDecodedPayload::HostReady(selector)
            }
        }
        C2b2aHostKind::Measurement => {
            require_payload_len(payload_bytes, C2B2A_MEASUREMENT_BYTES)?;
            if header.case_or_probe_id == 0
                || header.fixture_or_subattempt_id == 0
                || header.role_id == C2b2aRoleId::Supervisor
                || header.status != 0
            {
                return Err(invalid(
                    "Measurement header does not carry one exact nonzero item",
                ));
            }
            C2b2aDecodedPayload::Measurement(C2b2aMeasurement::decode(&array_256(payload_bytes))?)
        }
        C2b2aHostKind::Challenge => {
            require_payload_len(payload_bytes, C2B2A_CHALLENGE_BYTES)?;
            if header.sequence != 2 {
                return Err(invalid("Challenge sequence is not two"));
            }
            require_zero_item_shape(&header, true)?;
            if header.status != 0 {
                return Err(invalid("Challenge status is nonzero"));
            }
            C2b2aDecodedPayload::Challenge(C2b2aChallenge::from_bytes(array_32(payload_bytes, 0)))
        }
        C2b2aHostKind::TerminalReady => {
            require_payload_len(payload_bytes, C2B2A_TERMINAL_STATE_BYTES)?;
            require_zero_item_shape(&header, true)?;
            let state = C2b2aTerminalState::decode(&array_32(payload_bytes, 0))?;
            require_status_matches_state(&header, state)?;
            C2b2aDecodedPayload::TerminalReady(state)
        }
        C2b2aHostKind::Terminal => {
            require_payload_len(payload_bytes, C2B2A_TERMINAL_BYTES)?;
            require_zero_item_shape(&header, true)?;
            let challenge = C2b2aChallenge::from_bytes(array_32(payload_bytes, 0));
            let state =
                C2b2aTerminalState::decode(&array_32(payload_bytes, C2B2A_CHALLENGE_BYTES))?;
            require_status_matches_state(&header, state)?;
            C2b2aDecodedPayload::Terminal { challenge, state }
        }
        C2b2aHostKind::Abort => {
            require_payload_len(payload_bytes, C2B2A_TERMINAL_STATE_BYTES)?;
            if header.role_id != C2b2aRoleId::Supervisor {
                return Err(invalid("host-stream Abort role is nonzero"));
            }
            let state = C2b2aTerminalState::decode(&array_32(payload_bytes, 0))?;
            if state.primary_category == C2b2aPrimaryCategory::Success {
                return Err(invalid("Abort carries Success"));
            }
            require_status_matches_state(&header, state)?;
            let expected_item = state.active_item_parts()?.unwrap_or((0, 0));
            if (header.case_or_probe_id, header.fixture_or_subattempt_id) != expected_item {
                return Err(invalid(
                    "Abort item tuple does not match its terminal state",
                ));
            }
            C2b2aDecodedPayload::Abort(state)
        }
    };
    Ok(C2b2aDecodedFrame { header, payload })
}

fn require_payload_len(payload: &[u8], expected: usize) -> C2b2aResult<()> {
    if payload.len() != expected {
        return Err(invalid(format!(
            "host-control kind requires exactly {expected} payload bytes"
        )));
    }
    Ok(())
}

fn require_zero_item_shape(header: &C2b2aHostHeader, zero_role: bool) -> C2b2aResult<()> {
    if header.case_or_probe_id != 0
        || header.fixture_or_subattempt_id != 0
        || zero_role && header.role_id != C2b2aRoleId::Supervisor
    {
        return Err(invalid(
            "host-control frame requires zero item ids and role",
        ));
    }
    Ok(())
}

fn require_status_matches_state(
    header: &C2b2aHostHeader,
    state: C2b2aTerminalState,
) -> C2b2aResult<()> {
    if header.status != state.primary_category.header_status() {
        return Err(invalid(
            "terminal header status does not equal primary category",
        ));
    }
    Ok(())
}

fn validate_active_item(
    instance: C2b2aInstanceName,
    item: Option<(u16, u16)>,
    completed_items: u32,
) -> C2b2aResult<()> {
    let (first, second) = item.ok_or_else(|| {
        invalid("structured failure after HostReady must name the exact active item")
    })?;
    let ordinal = match instance {
        C2b2aInstanceName::C01 if first == 1 && (1..=3).contains(&second) => u32::from(second),
        C2b2aInstanceName::P01 | C2b2aInstanceName::P02 | C2b2aInstanceName::P03
            if first == second && (1..=20).contains(&first) =>
        {
            u32::from(first)
        }
        C2b2aInstanceName::B01 => {
            let mut ordinal = 0_u32;
            let mut matched = None;
            for probe_id in 1_u16..=39 {
                for subattempt_id in 1_u16..=b01_subattempt_count(probe_id) {
                    ordinal += 1;
                    if (first, second) == (probe_id, subattempt_id) {
                        matched = Some(ordinal);
                    }
                }
            }
            matched.ok_or_else(|| invalid("active item is not in the B01 schedule"))?
        }
        C2b2aInstanceName::B12O if (first, second) == (12, 1) => 1,
        C2b2aInstanceName::B12P if (first, second) == (12, 2) => 1,
        C2b2aInstanceName::B15 if (first, second) == (15, 1) => 1,
        C2b2aInstanceName::B16 if (first, second) == (16, 1) => 1,
        _ => {
            return Err(invalid(
                "active item is not in the selected instance schedule",
            ))
        }
    };
    if ordinal != completed_items + 1 {
        return Err(invalid("active item is not the exact next scheduled item"));
    }
    Ok(())
}

fn validate_pre_ready_abort_state(state: C2b2aTerminalState) -> C2b2aResult<()> {
    state.validate_common()?;
    if state.primary_category == C2b2aPrimaryCategory::Success
        || state.active_item_parts()?.is_some()
        || state.write_may_have_been_dispatched
        || state.writer_acknowledged
        || state.completed_case_count != 0
        || state.completed_subattempt_count != 0
    {
        return Err(invalid(
            "Abort before HostReady must carry zero item, milestone, and completion state",
        ));
    }
    Ok(())
}

fn validate_terminal_state_for_instance(
    instance: C2b2aInstanceName,
    state: C2b2aTerminalState,
    measurements_seen: u32,
    allow_zero_before_item: bool,
) -> C2b2aResult<()> {
    state.validate_common()?;
    let spec = instance_spec(instance);
    if spec.terminal_mode != C2b2aTerminalMode::Challenged {
        return Err(invalid(
            "host-terminal instance cannot carry a guest terminal state",
        ));
    }
    if measurements_seen > spec.measurement_count {
        return Err(invalid(
            "measurement count exceeds the selected instance cardinality",
        ));
    }
    if state.primary_category == C2b2aPrimaryCategory::Success {
        if measurements_seen != spec.measurement_count
            || state != C2b2aTerminalState::success_for(instance)?
        {
            return Err(invalid(
                "successful terminal state or cardinality is not exact",
            ));
        }
        return Ok(());
    }

    if allow_zero_before_item
        && measurements_seen == 0
        && state.active_item_parts()?.is_none()
        && !state.write_may_have_been_dispatched
        && !state.writer_acknowledged
        && state.completed_case_count == 0
        && state.completed_subattempt_count == 0
    {
        return Ok(());
    }

    let (completed_items, completed_bound, measurements_per_item) = match instance {
        C2b2aInstanceName::C01 => {
            if state.completed_case_count != 0 || state.completed_subattempt_count >= 3 {
                return Err(invalid("characterization failure counts are inconsistent"));
            }
            (state.completed_subattempt_count, 3, 7)
        }
        C2b2aInstanceName::P01 | C2b2aInstanceName::P02 | C2b2aInstanceName::P03 => {
            if state.completed_subattempt_count != 0 || state.completed_case_count >= 20 {
                return Err(invalid("positive-instance failure counts are inconsistent"));
            }
            (state.completed_case_count, 20, 7)
        }
        C2b2aInstanceName::B01 => {
            if state.completed_case_count != 0
                || state.completed_subattempt_count >= 60
                || state.write_may_have_been_dispatched
                || state.writer_acknowledged
            {
                return Err(invalid("B01 failure state is inconsistent"));
            }
            (state.completed_subattempt_count, 60, 1)
        }
        C2b2aInstanceName::B12O
        | C2b2aInstanceName::B12P
        | C2b2aInstanceName::B15
        | C2b2aInstanceName::B16 => {
            if state.completed_case_count != 0
                || state.completed_subattempt_count >= 1
                || state.write_may_have_been_dispatched
                || state.writer_acknowledged
            {
                return Err(invalid("dedicated-boundary failure state is inconsistent"));
            }
            (state.completed_subattempt_count, 1, 1)
        }
        C2b2aInstanceName::B40 | C2b2aInstanceName::B41 => unreachable!(),
    };
    if completed_items > completed_bound {
        return Err(invalid("completed item count exceeds its frozen bound"));
    }
    let lower_measurement_bound = completed_items
        .checked_mul(measurements_per_item)
        .ok_or_else(|| invalid("failure measurement lower bound overflow"))?;
    let upper_measurement_bound = completed_items
        .checked_add(1)
        .and_then(|value| value.checked_mul(measurements_per_item))
        .ok_or_else(|| invalid("failure measurement upper bound overflow"))?;
    if !(lower_measurement_bound..=upper_measurement_bound).contains(&measurements_seen) {
        return Err(invalid(
            "failure measurement count is outside the active-item interval",
        ));
    }
    validate_active_item(instance, state.active_item_parts()?, completed_items)
}

enum C2b2aProtocolState {
    AwaitingReady,
    AwaitingHostAbortEof(C2b2aTerminalState),
    HostAbortEofAccepted(C2b2aTerminalState),
    Collecting,
    ChallengeRequired(C2b2aTerminalState),
    AwaitingHostEof {
        state: C2b2aTerminalState,
        challenge: C2b2aChallenge,
    },
    AwaitingTerminal {
        state: C2b2aTerminalState,
        challenge: C2b2aChallenge,
    },
    AwaitingChallengedEof(C2b2aTerminalState),
    AwaitingAbortEof(C2b2aTerminalState),
    AwaitingHostTerminalEof {
        probe_id: u16,
    },
    Poisoned,
}

/// A move-only validator for exactly one bidirectional host/supervisor channel.
///
/// It owns no runtime capability. An invalid transition permanently poisons the value, while EOF
/// consumes it. A separate, later-reviewed runtime must retain raw bytes and own live handles.
pub struct C2b2aHostProtocolMachine {
    instance: C2b2aInstanceName,
    bindings: C2b2aFrameBindings,
    next_supervisor_sequence: u32,
    measurements_seen: u32,
    state: C2b2aProtocolState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum C2b2aHostProtocolEvent {
    HostReady,
    MeasurementAccepted {
        one_based_index: u32,
        key: C2b2aMeasurementKey,
    },
    ChallengeRequired(C2b2aTerminalState),
    TerminalAccepted(C2b2aTerminalState),
    AbortAccepted(C2b2aTerminalState),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum C2b2aHostProtocolCompletion {
    ChallengedTerminal {
        state: C2b2aTerminalState,
        measurements_seen: u32,
    },
    UnchallengedFailure {
        state: C2b2aTerminalState,
        measurements_seen: u32,
    },
    HostTerminalEof {
        probe_id: u16,
    },
}

impl C2b2aHostProtocolMachine {
    fn begin(
        instance: C2b2aInstanceName,
        campaign: C2b2aCampaignBindings,
    ) -> C2b2aResult<(Self, C2b2aEncodedFrame)> {
        if campaign.contract_sha256 != C2B2A_CONTRACT_SHA256 {
            return Err(invalid(
                "campaign contract digest is not the accepted C2B2a contract",
            ));
        }
        let start = encode_host_start(instance, campaign)?;
        Ok((
            Self {
                instance,
                bindings: selected_bindings(instance, campaign),
                next_supervisor_sequence: 1,
                measurements_seen: 0,
                state: C2b2aProtocolState::AwaitingReady,
            },
            start,
        ))
    }

    #[cfg(test)]
    pub fn begin_for_test(
        instance: C2b2aInstanceName,
        campaign: C2b2aCampaignBindings,
    ) -> C2b2aResult<(Self, C2b2aEncodedFrame)> {
        Self::begin(instance, campaign)
    }

    pub const fn instance(&self) -> C2b2aInstanceName {
        self.instance
    }

    pub const fn measurements_seen(&self) -> u32 {
        self.measurements_seen
    }

    /// Replaces the not-yet-written initial `HostStart` with a host-originated `Abort`.
    ///
    /// The caller must discard the `HostStart` returned by [`Self::begin`] and call this before
    /// writing any bytes. As with every host-stream EOF transition, the runtime must retain
    /// separate evidence that it successfully closed or half-closed the host-to-supervisor write
    /// endpoint after writing the returned `Abort`.
    pub fn replace_initial_host_start_with_abort(
        &mut self,
        campaign: C2b2aCampaignBindings,
        state: C2b2aTerminalState,
    ) -> C2b2aResult<C2b2aEncodedFrame> {
        let prior = std::mem::replace(&mut self.state, C2b2aProtocolState::Poisoned);
        if !matches!(prior, C2b2aProtocolState::AwaitingReady) {
            return Err(invalid(
                "initial HostStart can only be replaced before any peer frame",
            ));
        }
        if selected_bindings(self.instance, campaign) != self.bindings {
            return Err(invalid(
                "host Abort campaign bindings differ from HostStart",
            ));
        }
        validate_pre_ready_abort_state(state)?;
        let frame = encode_supervisor_abort(self.instance, campaign, 1, state)?;
        self.state = C2b2aProtocolState::AwaitingHostAbortEof(state);
        Ok(frame)
    }

    pub fn accept_supervisor_frame(
        &mut self,
        header: &[u8; C2B2A_HOST_HEADER_BYTES],
        payload: &[u8],
    ) -> C2b2aResult<C2b2aHostProtocolEvent> {
        let prior = std::mem::replace(&mut self.state, C2b2aProtocolState::Poisoned);
        let result = self.advance_supervisor_frame(prior, header, payload);
        match result {
            Ok((next, event)) => {
                self.state = next;
                Ok(event)
            }
            Err(error) => Err(error),
        }
    }

    fn advance_supervisor_frame(
        &mut self,
        prior: C2b2aProtocolState,
        header: &[u8; C2B2A_HOST_HEADER_BYTES],
        payload: &[u8],
    ) -> C2b2aResult<(C2b2aProtocolState, C2b2aHostProtocolEvent)> {
        let frame = decode_host_control_frame(C2b2aDirection::SupervisorToHost, header, payload)?;
        if frame.header.sequence != self.next_supervisor_sequence {
            return Err(invalid(
                "supervisor sequence is not the exact next sequence",
            ));
        }
        if frame.header.nonce != self.bindings.nonce
            || frame.header.contract_sha256 != self.bindings.contract_sha256
            || frame.header.schedule_sha256 != self.bindings.schedule_sha256
        {
            return Err(invalid("supervisor frame bindings differ from HostStart"));
        }
        let next_sequence = self
            .next_supervisor_sequence
            .checked_add(1)
            .ok_or_else(|| invalid("supervisor sequence overflow"))?;

        let (next, event) = match (prior, frame.payload) {
            (C2b2aProtocolState::AwaitingReady, C2b2aDecodedPayload::HostReady(selector)) => {
                if selector != instance_spec(self.instance).selector {
                    return Err(invalid("HostReady selector differs from HostStart"));
                }
                let next = match instance_spec(self.instance).terminal_mode {
                    C2b2aTerminalMode::Challenged => C2b2aProtocolState::Collecting,
                    C2b2aTerminalMode::HostTerminal { probe_id } => {
                        C2b2aProtocolState::AwaitingHostTerminalEof { probe_id }
                    }
                };
                (next, C2b2aHostProtocolEvent::HostReady)
            }
            (C2b2aProtocolState::AwaitingReady, C2b2aDecodedPayload::Abort(state)) => {
                validate_pre_ready_abort_state(state)?;
                (
                    C2b2aProtocolState::AwaitingAbortEof(state),
                    C2b2aHostProtocolEvent::AbortAccepted(state),
                )
            }
            (C2b2aProtocolState::Collecting, C2b2aDecodedPayload::Measurement(measurement)) => {
                let index = usize::try_from(self.measurements_seen)
                    .map_err(|_| invalid("measurement index does not fit usize"))?;
                let key = expected_measurement_at(self.instance, index)
                    .ok_or_else(|| invalid("instance emitted an additional Measurement"))?;
                if frame.header.case_or_probe_id != key.case_or_probe_id
                    || frame.header.fixture_or_subattempt_id != key.fixture_or_subattempt_id
                    || frame.header.role_id != key.role_id
                {
                    return Err(invalid(
                        "Measurement header is out of compiled schedule order",
                    ));
                }
                measurement.validate_for_key(key)?;
                self.measurements_seen = self
                    .measurements_seen
                    .checked_add(1)
                    .ok_or_else(|| invalid("measurement count overflow"))?;
                (
                    C2b2aProtocolState::Collecting,
                    C2b2aHostProtocolEvent::MeasurementAccepted {
                        one_based_index: self.measurements_seen,
                        key,
                    },
                )
            }
            (C2b2aProtocolState::Collecting, C2b2aDecodedPayload::TerminalReady(state)) => {
                validate_terminal_state_for_instance(
                    self.instance,
                    state,
                    self.measurements_seen,
                    false,
                )?;
                (
                    C2b2aProtocolState::ChallengeRequired(state),
                    C2b2aHostProtocolEvent::ChallengeRequired(state),
                )
            }
            (C2b2aProtocolState::Collecting, C2b2aDecodedPayload::Abort(state)) => {
                validate_terminal_state_for_instance(
                    self.instance,
                    state,
                    self.measurements_seen,
                    true,
                )?;
                (
                    C2b2aProtocolState::AwaitingAbortEof(state),
                    C2b2aHostProtocolEvent::AbortAccepted(state),
                )
            }
            (
                C2b2aProtocolState::AwaitingTerminal { state, challenge },
                C2b2aDecodedPayload::Terminal {
                    challenge: echoed,
                    state: committed,
                },
            ) => {
                if challenge != echoed || state != committed {
                    return Err(invalid(
                        "Terminal did not echo the exact challenge and state",
                    ));
                }
                (
                    C2b2aProtocolState::AwaitingChallengedEof(state),
                    C2b2aHostProtocolEvent::TerminalAccepted(state),
                )
            }
            (C2b2aProtocolState::Poisoned, _) => {
                return Err(invalid("host protocol machine is poisoned"));
            }
            _ => {
                return Err(invalid(
                    "frame is not legal in the current host protocol state",
                ))
            }
        };
        self.next_supervisor_sequence = next_sequence;
        Ok((next, event))
    }

    fn issue_challenge(
        &mut self,
        campaign: C2b2aCampaignBindings,
        challenge: C2b2aChallenge,
    ) -> C2b2aResult<C2b2aEncodedFrame> {
        let prior = std::mem::replace(&mut self.state, C2b2aProtocolState::Poisoned);
        let C2b2aProtocolState::ChallengeRequired(state) = prior else {
            return Err(invalid(
                "Challenge is not legal before a valid TerminalReady",
            ));
        };
        if selected_bindings(self.instance, campaign) != self.bindings {
            return Err(invalid("Challenge campaign bindings differ from HostStart"));
        }
        if challenge.as_bytes() == self.bindings.nonce.as_bytes() {
            return Err(invalid("Challenge equals the instance run nonce"));
        }
        let frame = encode_challenge(self.instance, campaign, challenge)?;
        self.state = C2b2aProtocolState::AwaitingHostEof { state, challenge };
        Ok(frame)
    }

    #[cfg(test)]
    pub fn issue_challenge_for_test(
        &mut self,
        campaign: C2b2aCampaignBindings,
        challenge: C2b2aChallenge,
    ) -> C2b2aResult<C2b2aEncodedFrame> {
        self.issue_challenge(campaign, challenge)
    }

    /// Advances only after the runtime successfully closes or half-closes the real
    /// host-to-supervisor write endpoint.
    ///
    /// This is protocol state, not handle-closure evidence: a later reviewed runtime must retain
    /// separate evidence of that close so the supervisor can observe EOF. Calling it early, twice,
    /// or after any later transition permanently poisons this machine.
    pub fn accept_host_to_supervisor_eof(&mut self) -> C2b2aResult<()> {
        let prior = std::mem::replace(&mut self.state, C2b2aProtocolState::Poisoned);
        match prior {
            C2b2aProtocolState::AwaitingHostEof { state, challenge } => {
                self.state = C2b2aProtocolState::AwaitingTerminal { state, challenge };
                Ok(())
            }
            C2b2aProtocolState::AwaitingHostAbortEof(state) => {
                self.state = C2b2aProtocolState::HostAbortEofAccepted(state);
                Ok(())
            }
            C2b2aProtocolState::Poisoned => Err(invalid("host protocol machine is poisoned")),
            _ => Err(invalid(
                "host-to-supervisor EOF is early, duplicate, or late",
            )),
        }
    }

    /// Completes a host-originated initial `Abort` after host-to-supervisor EOF was recorded.
    pub fn finish_after_host_to_supervisor_eof(self) -> C2b2aResult<C2b2aHostProtocolCompletion> {
        match self.state {
            C2b2aProtocolState::HostAbortEofAccepted(state) => {
                Ok(C2b2aHostProtocolCompletion::UnchallengedFailure {
                    state,
                    measurements_seen: self.measurements_seen,
                })
            }
            _ => Err(invalid(
                "host-to-supervisor EOF is missing or did not follow a host Abort",
            )),
        }
    }

    pub fn finish_on_supervisor_eof(self) -> C2b2aResult<C2b2aHostProtocolCompletion> {
        match self.state {
            C2b2aProtocolState::AwaitingChallengedEof(state) => {
                Ok(C2b2aHostProtocolCompletion::ChallengedTerminal {
                    state,
                    measurements_seen: self.measurements_seen,
                })
            }
            C2b2aProtocolState::AwaitingAbortEof(state) => {
                Ok(C2b2aHostProtocolCompletion::UnchallengedFailure {
                    state,
                    measurements_seen: self.measurements_seen,
                })
            }
            C2b2aProtocolState::AwaitingHostTerminalEof { probe_id } => {
                Ok(C2b2aHostProtocolCompletion::HostTerminalEof { probe_id })
            }
            _ => Err(invalid(
                "supervisor EOF is early or Terminal has not been committed",
            )),
        }
    }
}

/// Move-only campaign gate for the one characterization instance followed by the ten acceptance
/// instances. It owns ordering and freshness; callers can neither select an instance nor issue a
/// challenge through a detached per-instance machine.
pub struct C2b2aCampaignController {
    freshness: C2b2aCampaignFreshness,
    next_instance_index: usize,
    active: Option<C2b2aHostProtocolMachine>,
    active_campaign: Option<C2b2aCampaignBindings>,
    pending_runtime_closure: Option<C2b2aHostProtocolCompletion>,
    campaign_authority: Option<(C2b2aDigest, C2b2aDigest, C2b2aDigest)>,
    terminated: bool,
    poisoned: bool,
}

impl C2b2aCampaignController {
    pub const fn new() -> Self {
        Self {
            freshness: C2b2aCampaignFreshness::new(),
            next_instance_index: 0,
            active: None,
            active_campaign: None,
            pending_runtime_closure: None,
            campaign_authority: None,
            terminated: false,
            poisoned: false,
        }
    }

    pub fn next_expected_instance(&self) -> Option<C2b2aInstanceName> {
        if self.poisoned
            || self.terminated
            || self.active.is_some()
            || self.pending_runtime_closure.is_some()
        {
            return None;
        }
        C2B2A_CAMPAIGN_ORDER.get(self.next_instance_index).copied()
    }

    pub fn active_instance(&self) -> Option<C2b2aInstanceName> {
        self.active.as_ref().map(C2b2aHostProtocolMachine::instance)
    }

    pub const fn completed_instance_count(&self) -> usize {
        self.next_instance_index
    }

    pub const fn is_complete(&self) -> bool {
        !self.poisoned
            && !self.terminated
            && self.next_instance_index == C2B2A_CAMPAIGN_ORDER.len()
            && self.active.is_none()
            && self.pending_runtime_closure.is_none()
            && self.freshness.is_complete()
    }

    pub fn begin_next(
        &mut self,
        campaign: C2b2aCampaignBindings,
    ) -> C2b2aResult<(C2b2aInstanceName, C2b2aEncodedFrame)> {
        if self.poisoned {
            return Err(invalid("campaign controller is poisoned"));
        }
        if self.terminated {
            self.poisoned = true;
            return Err(invalid("a failed campaign cannot start another instance"));
        }
        if self.active.is_some() {
            self.poisoned = true;
            return Err(invalid("campaign already has an active instance"));
        }
        if self.pending_runtime_closure.is_some() {
            self.poisoned = true;
            return Err(invalid(
                "campaign cannot advance before runtime closure is proven",
            ));
        }
        if campaign.contract_sha256 != C2B2A_CONTRACT_SHA256 {
            self.poisoned = true;
            return Err(invalid(
                "campaign contract digest is not the accepted C2B2a contract",
            ));
        }
        let Some(instance) = C2B2A_CAMPAIGN_ORDER.get(self.next_instance_index).copied() else {
            self.poisoned = true;
            return Err(invalid("campaign contains an additional instance"));
        };
        let authority = (
            campaign.contract_sha256,
            campaign.characterization_schedule_sha256,
            campaign.acceptance_schedule_sha256,
        );
        if let Some(expected) = self.campaign_authority {
            if authority != expected {
                self.poisoned = true;
                return Err(invalid(
                    "campaign authority digests changed between instances",
                ));
            }
        } else {
            self.campaign_authority = Some(authority);
        }
        if let Err(error) = self.freshness.register_nonce(campaign.nonce) {
            self.poisoned = true;
            return Err(error);
        }
        let (machine, start) = match C2b2aHostProtocolMachine::begin(instance, campaign) {
            Ok(value) => value,
            Err(error) => {
                self.poisoned = true;
                return Err(error);
            }
        };
        self.active = Some(machine);
        self.active_campaign = Some(campaign);
        Ok((instance, start))
    }

    pub fn accept_supervisor_frame(
        &mut self,
        header: &[u8; C2B2A_HOST_HEADER_BYTES],
        payload: &[u8],
    ) -> C2b2aResult<C2b2aHostProtocolEvent> {
        if self.poisoned {
            return Err(invalid("campaign controller is poisoned"));
        }
        let Some(machine) = self.active.as_mut() else {
            self.poisoned = true;
            return Err(invalid("campaign has no active instance"));
        };
        match machine.accept_supervisor_frame(header, payload) {
            Ok(event) => Ok(event),
            Err(error) => {
                self.poisoned = true;
                Err(error)
            }
        }
    }

    pub fn issue_challenge(&mut self, challenge: C2b2aChallenge) -> C2b2aResult<C2b2aEncodedFrame> {
        if self.poisoned {
            return Err(invalid("campaign controller is poisoned"));
        }
        let Some(campaign) = self.active_campaign else {
            self.poisoned = true;
            return Err(invalid("campaign has no active bindings"));
        };
        let Some(machine) = self.active.as_mut() else {
            self.poisoned = true;
            return Err(invalid("campaign has no active instance"));
        };
        let frame = match machine.issue_challenge(campaign, challenge) {
            Ok(frame) => frame,
            Err(error) => {
                self.poisoned = true;
                return Err(error);
            }
        };
        if let Err(error) = self.freshness.register_challenge(challenge) {
            self.poisoned = true;
            return Err(error);
        }
        Ok(frame)
    }

    /// Replaces the active instance's not-yet-written `HostStart` with a host-originated `Abort`.
    pub fn replace_active_initial_host_start_with_abort(
        &mut self,
        state: C2b2aTerminalState,
    ) -> C2b2aResult<C2b2aEncodedFrame> {
        if self.poisoned {
            return Err(invalid("campaign controller is poisoned"));
        }
        let Some(campaign) = self.active_campaign else {
            self.poisoned = true;
            return Err(invalid("campaign has no active bindings"));
        };
        let Some(machine) = self.active.as_mut() else {
            self.poisoned = true;
            return Err(invalid("campaign has no active instance"));
        };
        match machine.replace_initial_host_start_with_abort(campaign, state) {
            Ok(frame) => Ok(frame),
            Err(error) => {
                self.poisoned = true;
                Err(error)
            }
        }
    }

    /// Records the transition after the runtime closes the host-to-supervisor write endpoint.
    ///
    /// This does not attest that a live handle closed; retained close evidence remains a separate
    /// runtime obligation, and the supervisor is the peer that observes EOF.
    pub fn accept_active_host_to_supervisor_eof(&mut self) -> C2b2aResult<()> {
        if self.poisoned {
            return Err(invalid("campaign controller is poisoned"));
        }
        let Some(machine) = self.active.as_mut() else {
            self.poisoned = true;
            return Err(invalid("campaign has no active instance"));
        };
        match machine.accept_host_to_supervisor_eof() {
            Ok(()) => Ok(()),
            Err(error) => {
                self.poisoned = true;
                Err(error)
            }
        }
    }

    pub fn finish_active_on_supervisor_eof(&mut self) -> C2b2aResult<C2b2aHostProtocolCompletion> {
        if self.poisoned {
            return Err(invalid("campaign controller is poisoned"));
        }
        let Some(machine) = self.active.take() else {
            self.poisoned = true;
            return Err(invalid("campaign has no active instance"));
        };
        self.active_campaign = None;
        let completion = match machine.finish_on_supervisor_eof() {
            Ok(completion) => completion,
            Err(error) => {
                self.poisoned = true;
                return Err(error);
            }
        };
        self.pending_runtime_closure = Some(completion);
        Ok(completion)
    }

    pub fn finish_active_after_host_to_supervisor_eof(
        &mut self,
    ) -> C2b2aResult<C2b2aHostProtocolCompletion> {
        if self.poisoned {
            return Err(invalid("campaign controller is poisoned"));
        }
        let Some(machine) = self.active.take() else {
            self.poisoned = true;
            return Err(invalid("campaign has no active instance"));
        };
        self.active_campaign = None;
        let completion = match machine.finish_after_host_to_supervisor_eof() {
            Ok(completion) => completion,
            Err(error) => {
                self.poisoned = true;
                return Err(error);
            }
        };
        self.pending_runtime_closure = Some(completion);
        Ok(completion)
    }

    #[cfg(test)]
    pub fn attest_runtime_closure_for_test(&mut self, closure_proven: bool) -> C2b2aResult<()> {
        if self.poisoned {
            return Err(invalid("campaign controller is poisoned"));
        }
        let Some(completion) = self.pending_runtime_closure.take() else {
            self.poisoned = true;
            return Err(invalid("campaign has no pending runtime closure"));
        };
        if !closure_proven {
            self.poisoned = true;
            return Err(invalid("runtime closure was not proven"));
        }
        let may_continue = matches!(
            completion,
            C2b2aHostProtocolCompletion::ChallengedTerminal {
                state: C2b2aTerminalState {
                    primary_category: C2b2aPrimaryCategory::Success,
                    ..
                },
                ..
            } | C2b2aHostProtocolCompletion::HostTerminalEof { .. }
        );
        if may_continue {
            self.next_instance_index = self
                .next_instance_index
                .checked_add(1)
                .ok_or_else(|| invalid("campaign instance index overflow"))?;
        } else {
            self.terminated = true;
        }
        Ok(())
    }
}

impl Default for C2b2aCampaignController {
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! assert_not_impl_any {
    ($type:ty: $($forbidden:path),+ $(,)?) => {
        const _: fn() = || {
            trait AmbiguousIfImpl<Marker> {
                fn marker() {}
            }

            impl<T: ?Sized> AmbiguousIfImpl<()> for T {}

            $({
                struct Forbidden;
                impl<T: ?Sized + $forbidden> AmbiguousIfImpl<Forbidden> for T {}
            })+

            let _ = <$type as AmbiguousIfImpl<_>>::marker;
        };
    };
}

assert_not_impl_any!(C2b2aProtocolState:
    Clone,
    Copy,
    std::fmt::Debug,
    std::fmt::Display,
    serde::Serialize,
    serde::Deserialize<'static>
);
assert_not_impl_any!(C2b2aHostProtocolMachine:
    Clone,
    Copy,
    std::fmt::Debug,
    std::fmt::Display,
    serde::Serialize,
    serde::Deserialize<'static>
);
assert_not_impl_any!(C2b2aCampaignFreshness:
    Clone,
    Copy,
    std::fmt::Debug,
    std::fmt::Display,
    serde::Serialize,
    serde::Deserialize<'static>
);
assert_not_impl_any!(C2b2aCampaignController:
    Clone,
    Copy,
    std::fmt::Debug,
    std::fmt::Display,
    serde::Serialize,
    serde::Deserialize<'static>
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum C2b2aHostReceiptKind {
    Terminal,
    Recovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct C2b2aHostTerminalReceiptV1 {
    pub kind: C2b2aHostReceiptKind,
    pub probe_id: u16,
    pub host_cleanup_proven: bool,
    pub injected_cleanup_fault: bool,
    pub monotonic_elapsed_ns: u64,
    pub nonce: C2b2aNonce,
    pub contract_sha256: C2b2aDigest,
    pub acceptance_schedule_sha256: C2b2aDigest,
    pub host_executable_sha256: C2b2aDigest,
    pub instance_identity_record_sha256: C2b2aDigest,
    pub disk_identity_record_sha256: C2b2aDigest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct C2b2aHostReceiptExpectations {
    pub kind: C2b2aHostReceiptKind,
    pub probe_id: u16,
    pub nonce: C2b2aNonce,
    pub contract_sha256: C2b2aDigest,
    pub acceptance_schedule_sha256: C2b2aDigest,
    pub host_executable_sha256: C2b2aDigest,
    pub instance_identity_record_sha256: C2b2aDigest,
    pub disk_identity_record_sha256: C2b2aDigest,
}

impl C2b2aHostTerminalReceiptV1 {
    pub fn validate(self) -> C2b2aResult<()> {
        let exact = match (self.kind, self.probe_id) {
            (C2b2aHostReceiptKind::Terminal, 40) => {
                self.host_cleanup_proven && !self.injected_cleanup_fault
            }
            (C2b2aHostReceiptKind::Terminal, 41) => {
                !self.host_cleanup_proven && self.injected_cleanup_fault
            }
            (C2b2aHostReceiptKind::Recovery, 41) => {
                self.host_cleanup_proven && self.injected_cleanup_fault
            }
            _ => false,
        };
        if !exact {
            return Err(invalid(
                "host receipt kind/probe/cleanup tuple is not frozen",
            ));
        }
        Ok(())
    }

    /// Match a decoded receipt to live, externally retained campaign bindings.
    ///
    /// The elapsed value is intentionally not guessed here: the runtime must separately prove the
    /// exact checked `CLOCK_MONOTONIC_RAW` endpoints required by the frozen contract.
    pub fn validate_against(self, expected: C2b2aHostReceiptExpectations) -> C2b2aResult<()> {
        self.validate()?;
        if self.kind != expected.kind
            || self.probe_id != expected.probe_id
            || self.nonce != expected.nonce
            || self.contract_sha256 != expected.contract_sha256
            || self.acceptance_schedule_sha256 != expected.acceptance_schedule_sha256
            || self.host_executable_sha256 != expected.host_executable_sha256
            || self.instance_identity_record_sha256 != expected.instance_identity_record_sha256
            || self.disk_identity_record_sha256 != expected.disk_identity_record_sha256
        {
            return Err(invalid(
                "host receipt does not match the retained live campaign bindings",
            ));
        }
        Ok(())
    }

    pub fn encode(self) -> C2b2aResult<[u8; C2B2A_HOST_TERMINAL_RECEIPT_BYTES]> {
        self.validate()?;
        let mut bytes = [0_u8; C2B2A_HOST_TERMINAL_RECEIPT_BYTES];
        let magic = match self.kind {
            C2b2aHostReceiptKind::Terminal => C2B2A_HOST_RECEIPT_MAGIC,
            C2b2aHostReceiptKind::Recovery => C2B2A_HOST_RECOVERY_MAGIC,
        };
        bytes[0..8].copy_from_slice(&magic);
        put_u16(&mut bytes, 8, C2B2A_PROTOCOL_VERSION);
        put_u16(&mut bytes, 10, self.probe_id);
        put_u32(
            &mut bytes,
            12,
            C2b2aPrimaryCategory::GuestConfiguration.code(),
        );
        put_u32(&mut bytes, 16, u32::from(self.host_cleanup_proven));
        put_u32(&mut bytes, 20, u32::from(self.injected_cleanup_fault));
        put_u64(&mut bytes, 24, self.monotonic_elapsed_ns);
        bytes[32..64].copy_from_slice(self.nonce.as_bytes());
        bytes[64..96].copy_from_slice(self.contract_sha256.as_bytes());
        bytes[96..128].copy_from_slice(self.acceptance_schedule_sha256.as_bytes());
        bytes[128..160].copy_from_slice(self.host_executable_sha256.as_bytes());
        bytes[160..192].copy_from_slice(self.instance_identity_record_sha256.as_bytes());
        bytes[192..224].copy_from_slice(self.disk_identity_record_sha256.as_bytes());
        let digest = payload_sha256(&bytes[..224]);
        bytes[224..256].copy_from_slice(digest.as_bytes());
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8; C2B2A_HOST_TERMINAL_RECEIPT_BYTES]) -> C2b2aResult<Self> {
        let kind = if bytes[0..8] == C2B2A_HOST_RECEIPT_MAGIC {
            C2b2aHostReceiptKind::Terminal
        } else if bytes[0..8] == C2B2A_HOST_RECOVERY_MAGIC {
            C2b2aHostReceiptKind::Recovery
        } else {
            return Err(invalid("host receipt magic mismatch"));
        };
        if read_u16(bytes, 8) != C2B2A_PROTOCOL_VERSION {
            return Err(invalid("host receipt version mismatch"));
        }
        if read_u32(bytes, 12) != C2b2aPrimaryCategory::GuestConfiguration.code() {
            return Err(invalid(
                "host receipt primary category is not GuestConfiguration",
            ));
        }
        let boolean = |offset| -> C2b2aResult<bool> {
            match read_u32(bytes, offset) {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(invalid("host receipt boolean is not zero or one")),
            }
        };
        if payload_sha256(&bytes[..224]) != C2b2aDigest::from_bytes(array_32(bytes, 224)) {
            return Err(invalid("host receipt digest mismatch"));
        }
        let receipt = Self {
            kind,
            probe_id: read_u16(bytes, 10),
            host_cleanup_proven: boolean(16)?,
            injected_cleanup_fault: boolean(20)?,
            monotonic_elapsed_ns: read_u64(bytes, 24),
            nonce: C2b2aNonce::from_bytes(array_32(bytes, 32)),
            contract_sha256: C2b2aDigest::from_bytes(array_32(bytes, 64)),
            acceptance_schedule_sha256: C2b2aDigest::from_bytes(array_32(bytes, 96)),
            host_executable_sha256: C2b2aDigest::from_bytes(array_32(bytes, 128)),
            instance_identity_record_sha256: C2b2aDigest::from_bytes(array_32(bytes, 160)),
            disk_identity_record_sha256: C2b2aDigest::from_bytes(array_32(bytes, 192)),
        };
        receipt.validate()?;
        Ok(receipt)
    }
}

/// Validate the relationships among the three mandatory host-terminal records.
///
/// Live witness authority and exact monotonic endpoints remain runtime obligations; these
/// forgeable structural records do not authorize cleanup or acceptance.
pub fn validate_host_terminal_receipt_triplet(
    b40: C2b2aHostTerminalReceiptV1,
    b41: C2b2aHostTerminalReceiptV1,
    b41_recovery: C2b2aHostTerminalReceiptV1,
) -> C2b2aResult<()> {
    b40.validate()?;
    b41.validate()?;
    b41_recovery.validate()?;
    if (b40.kind, b40.probe_id) != (C2b2aHostReceiptKind::Terminal, 40)
        || (b41.kind, b41.probe_id) != (C2b2aHostReceiptKind::Terminal, 41)
        || (b41_recovery.kind, b41_recovery.probe_id) != (C2b2aHostReceiptKind::Recovery, 41)
    {
        return Err(invalid(
            "host receipt triplet does not contain exact B40/B41 records",
        ));
    }
    if b40.contract_sha256 != b41.contract_sha256
        || b40.acceptance_schedule_sha256 != b41.acceptance_schedule_sha256
        || b40.host_executable_sha256 != b41.host_executable_sha256
    {
        return Err(invalid("B40 and B41 do not share exact campaign bindings"));
    }
    if b40.nonce == b41.nonce {
        return Err(invalid("B40 and B41 reuse a run nonce"));
    }
    if b41_recovery.nonce != b41.nonce
        || b41_recovery.contract_sha256 != b41.contract_sha256
        || b41_recovery.acceptance_schedule_sha256 != b41.acceptance_schedule_sha256
        || b41_recovery.host_executable_sha256 != b41.host_executable_sha256
        || b41_recovery.instance_identity_record_sha256 != b41.instance_identity_record_sha256
        || b41_recovery.disk_identity_record_sha256 != b41.disk_identity_record_sha256
    {
        return Err(invalid(
            "B41 recovery record differs from the original receipt bindings or identities",
        ));
    }
    Ok(())
}

/// Validate only immutable, provider-free values compiled into this source file.
pub fn validate_compiled_foundation() -> C2b2aResult<()> {
    if ACCEPTED_C2B2A_FREEZE_SHA256.len() != 64
        || !ACCEPTED_C2B2A_FREEZE_SHA256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(invalid(
            "accepted freeze digest constant is not lowercase SHA-256",
        ));
    }
    if C2B2A_HOST_HEADER_BYTES != 160
        || C2B2A_MAX_HOST_PAYLOAD_BYTES != 256
        || C2B2A_MEASUREMENT_BYTES != 32 * 8
        || C2B2A_TERMINAL_STATE_BYTES != 8 * 4
        || C2B2A_TERMINAL_BYTES != C2B2A_CHALLENGE_BYTES + C2B2A_TERMINAL_STATE_BYTES
        || C2B2A_HOST_TERMINAL_RECEIPT_BYTES != 256
    {
        return Err(invalid("compiled wire sizes drifted"));
    }
    if C2B2A_CAMPAIGN_NONCE_COUNT != 11
        || C2B2A_CAMPAIGN_CHALLENGE_COUNT != 9
        || C2B2A_CAMPAIGN_FRESH_VALUE_COUNT != 20
        || C2B2A_GUEST_CPUS != 2
        || C2B2A_GUEST_MEMORY_BYTES != 4_294_967_296
        || C2B2A_PRIVATE_DATA_DISK_BYTES != 4_294_967_296
        || C2B2A_INPUT_BUNDLE_MAX_BYTES != 536_870_912
        || C2B2A_RETAINED_EVIDENCE_MAX_BYTES != 67_108_864
        || C2B2A_TERMINAL_STRUCTURAL_RECEIPT_MAX_BYTES != 262_144
        || C2B2A_PER_ROLE_OUTPUT_MAX_BYTES != 65_536
        || C2B2A_PRE_INSTANCE_AVAILABLE_KIB != 134_217_728
        || C2B2A_POST_CLEANUP_AVAILABLE_KIB != 19_427_004
        || C2B2A_FILESYSTEM_NOISE_TOLERANCE_BYTES != 1_073_741_824
        || C2B2A_ROLE_DEADLINE_SECONDS != 60
        || C2B2A_JOURNEY_DEADLINE_SECONDS != 240
        || C2B2A_SHORT_GUEST_DEADLINE_SECONDS != 600
        || C2B2A_LONG_GUEST_DEADLINE_SECONDS != 2_700
        || C2B2A_ACCEPTANCE_DEADLINE_SECONDS != 18_000
        || C2B2A_CAMPAIGN_DEADLINE_SECONDS != 18_600
    {
        return Err(invalid("compiled frozen numeric ceiling drifted"));
    }
    if CHARACTERIZATION_SCHEDULE_V1.len() != 1 || ACCEPTANCE_SCHEDULE_V1.len() != 10 {
        return Err(invalid("compiled phase schedule cardinality drifted"));
    }
    let exact_characterization = [challenged_spec(
        C2b2aInstanceName::C01,
        1,
        1,
        1,
        C2b2aEvidenceAuthority::NonAcceptance,
        21,
    )];
    let exact_acceptance = [
        challenged_spec(
            C2b2aInstanceName::P01,
            2,
            101,
            1,
            C2b2aEvidenceAuthority::Acceptance,
            140,
        ),
        challenged_spec(
            C2b2aInstanceName::P02,
            2,
            102,
            2,
            C2b2aEvidenceAuthority::Acceptance,
            140,
        ),
        challenged_spec(
            C2b2aInstanceName::P03,
            2,
            103,
            3,
            C2b2aEvidenceAuthority::Acceptance,
            140,
        ),
        challenged_spec(
            C2b2aInstanceName::B01,
            2,
            201,
            4,
            C2b2aEvidenceAuthority::Acceptance,
            60,
        ),
        challenged_spec(
            C2b2aInstanceName::B12O,
            2,
            212,
            5,
            C2b2aEvidenceAuthority::Acceptance,
            1,
        ),
        challenged_spec(
            C2b2aInstanceName::B12P,
            2,
            213,
            6,
            C2b2aEvidenceAuthority::Acceptance,
            1,
        ),
        challenged_spec(
            C2b2aInstanceName::B15,
            2,
            215,
            7,
            C2b2aEvidenceAuthority::Acceptance,
            1,
        ),
        challenged_spec(
            C2b2aInstanceName::B16,
            2,
            216,
            8,
            C2b2aEvidenceAuthority::Acceptance,
            1,
        ),
        host_terminal_spec(C2b2aInstanceName::B40, 240, 9, 40),
        host_terminal_spec(C2b2aInstanceName::B41, 241, 10, 41),
    ];
    if CHARACTERIZATION_SCHEDULE_V1 != exact_characterization
        || ACCEPTANCE_SCHEDULE_V1 != exact_acceptance
    {
        return Err(invalid("compiled phase schedule row drifted"));
    }
    if C2B2A_CAMPAIGN_ORDER[0] != CHARACTERIZATION_SCHEDULE_V1[0].name
        || C2B2A_CAMPAIGN_ORDER[1..]
            .iter()
            .copied()
            .ne(ACCEPTANCE_SCHEDULE_V1.iter().map(|spec| spec.name))
    {
        return Err(invalid(
            "campaign order differs from the two frozen schedules",
        ));
    }
    let mut selectors = BTreeSet::new();
    let mut names = BTreeSet::new();
    let mut acceptance_measurements = 0_u32;
    for spec in CHARACTERIZATION_SCHEDULE_V1
        .iter()
        .chain(ACCEPTANCE_SCHEDULE_V1.iter())
    {
        if !selectors.insert(spec.selector) || !names.insert(spec.name) {
            return Err(invalid("compiled schedule contains a duplicate row"));
        }
        if instance_for_selector(spec.selector)? != *spec {
            return Err(invalid("compiled selector lookup is not bijective"));
        }
        let generated_count = (0..)
            .take_while(|index| expected_measurement_at(spec.name, *index).is_some())
            .count();
        if generated_count
            != usize::try_from(spec.measurement_count)
                .map_err(|_| invalid("measurement count does not fit usize"))?
        {
            return Err(invalid("compiled measurement schedule cardinality drifted"));
        }
        match spec.terminal_mode {
            C2b2aTerminalMode::Challenged => {
                if spec.terminal_ready_sequence != Some(spec.measurement_count + 2)
                    || spec.terminal_sequence != Some(spec.measurement_count + 3)
                {
                    return Err(invalid("challenged terminal sequence drifted"));
                }
            }
            C2b2aTerminalMode::HostTerminal { probe_id } => {
                if !matches!(probe_id, 40 | 41)
                    || spec.measurement_count != 0
                    || spec.terminal_ready_sequence.is_some()
                    || spec.terminal_sequence.is_some()
                {
                    return Err(invalid("host-terminal schedule row drifted"));
                }
            }
        }
        if spec.authority == C2b2aEvidenceAuthority::Acceptance {
            acceptance_measurements = acceptance_measurements
                .checked_add(spec.measurement_count)
                .ok_or_else(|| invalid("acceptance measurement total overflow"))?;
        }
    }
    if CHARACTERIZATION_SCHEDULE_V1[0].measurement_count != 21
        || acceptance_measurements != 484
        || instance_spec(C2b2aInstanceName::B01).measurement_count != 60
    {
        return Err(invalid("compiled campaign measurement totals drifted"));
    }
    Ok(())
}

fn array_8(bytes: &[u8], offset: usize) -> [u8; 8] {
    let mut value = [0_u8; 8];
    value.copy_from_slice(&bytes[offset..offset + 8]);
    value
}

fn array_32(bytes: &[u8], offset: usize) -> [u8; 32] {
    let mut value = [0_u8; 32];
    value.copy_from_slice(&bytes[offset..offset + 32]);
    value
}

fn array_256(bytes: &[u8]) -> [u8; 256] {
    let mut value = [0_u8; 256];
    value.copy_from_slice(bytes);
    value
}

fn put_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}

fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
}

fn put_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_be_bytes());
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_be_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
        bytes[offset + 4],
        bytes[offset + 5],
        bytes[offset + 6],
        bytes[offset + 7],
    ])
}
