//! Closed, provider-free constants and decision rules for the C2B2a payload.
//!
//! Nothing in this module opens a datastore. It is deliberately usable by the
//! Core-free supervisor as well as by the collector.

use core::fmt;

use sha2::{Digest as _, Sha256};

pub const PROTOCOL_MAGIC: [u8; 8] = *b"ENGC2A01";
pub const PROTOCOL_VERSION: u16 = 1;
pub const PROFILE_ID: u16 = 1;
pub const PROTOCOL_HEADER_LEN: usize = 160;
pub const MAX_PROTOCOL_PAYLOAD_LEN: usize = 256;

pub const STORE_ORIGIN: &str = "rocksdb:/data/store";
pub const STORE_NAMESPACE: &str = "engram";
pub const STORE_DATABASE: &str = "main";

pub const TABLE_NAMES: [&str; 9] = [
    "memory_item",
    "correction_proposal",
    "memory_forget_receipt",
    "work_project",
    "work_task",
    "git_repository",
    "local_checkout",
    "monorepo_component",
    "project_repository_link",
];

pub const INSERT_QUERY: &str = concat!(
    "BEGIN TRANSACTION;\n",
    "INSERT INTO memory_item $memory_item RETURN NONE;\n",
    "INSERT INTO correction_proposal $correction_proposal RETURN NONE;\n",
    "INSERT INTO memory_forget_receipt $memory_forget_receipt RETURN NONE;\n",
    "INSERT INTO work_project $work_project RETURN NONE;\n",
    "INSERT INTO work_task $work_task RETURN NONE;\n",
    "INSERT INTO git_repository $git_repository RETURN NONE;\n",
    "INSERT INTO local_checkout $local_checkout RETURN NONE;\n",
    "INSERT INTO monorepo_component $monorepo_component RETURN NONE;\n",
    "INSERT INTO project_repository_link $project_repository_link RETURN NONE;\n",
    "COMMIT TRANSACTION;",
);
pub const READ_QUERY: &str = concat!(
    "RETURN {\n",
    "  memory_item: (SELECT * FROM memory_item LIMIT 65),\n",
    "  correction_proposal: (SELECT * FROM correction_proposal LIMIT 33),\n",
    "  memory_forget_receipt: (SELECT * FROM memory_forget_receipt LIMIT 33),\n",
    "  work_project: (SELECT * FROM work_project LIMIT 9),\n",
    "  work_task: (SELECT * FROM work_task LIMIT 65),\n",
    "  git_repository: (SELECT * FROM git_repository LIMIT 17),\n",
    "  local_checkout: (SELECT * FROM local_checkout LIMIT 33),\n",
    "  monorepo_component: (SELECT * FROM monorepo_component LIMIT 65),\n",
    "  project_repository_link: (SELECT * FROM project_repository_link LIMIT 65)\n",
    "};",
);

pub const INSERT_QUERY_LEN: usize = 572;
pub const READ_QUERY_LEN: usize = 570;

const _: [(); INSERT_QUERY_LEN] = [(); INSERT_QUERY.len()];
const _: [(); READ_QUERY_LEN] = [(); READ_QUERY.len()];

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Digest32(pub [u8; 32]);

impl Digest32 {
    pub const ZERO: Self = Self([0; 32]);

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Debug for Digest32 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

const fn hex_nibble(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        b'A'..=b'F' => byte - b'A' + 10,
        _ => panic!("non-hex digest byte"),
    }
}

pub const fn digest_from_hex(value: &str) -> Digest32 {
    let input = value.as_bytes();
    assert!(input.len() == 64, "SHA-256 text must contain 64 hex digits");
    let mut output = [0_u8; 32];
    let mut index = 0;
    while index < output.len() {
        output[index] = (hex_nibble(input[index * 2]) << 4) | hex_nibble(input[index * 2 + 1]);
        index += 1;
    }
    Digest32(output)
}

pub fn sha256(bytes: &[u8]) -> Digest32 {
    Digest32(Sha256::digest(bytes).into())
}

pub const CONTRACT_SHA256: Digest32 =
    digest_from_hex("85312b0286080ecbbab94b5236003a42f281b1122565016d96bb20f9572063e3");
pub const ROCKS_ENVIRONMENT_SHA256: Digest32 =
    digest_from_hex("2e53756bdfbc02d2bd7d3dc8fe7ab13541badf8e114ecb48cd446d49d067b092");
pub const INSERT_QUERY_SHA256: Digest32 =
    digest_from_hex("9f6c41b6c7d0006db0a03f29fd951bb3c81df19946748faaac768af013c98a05");
pub const READ_QUERY_SHA256: Digest32 =
    digest_from_hex("cf1034a335af240c0726e5385207acc4e4e49273dc016891138c863e8ba096f5");

/// Canonical manifest families named by the accepted wire contract.
///
/// The accepted freeze requires these digests to be produced by a later reviewed build record,
/// but does not freeze either canonical encodings or digest values.  Keeping the slots explicit
/// and empty prevents a caller-supplied digest from accidentally becoming acceptance authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ManifestKindV1 {
    CharacterizationSchedule,
    AcceptanceSchedule,
    Characterization,
    PositiveCases,
    BoundaryProbes,
}

impl ManifestKindV1 {
    pub const ALL: [Self; 5] = [
        Self::CharacterizationSchedule,
        Self::AcceptanceSchedule,
        Self::Characterization,
        Self::PositiveCases,
        Self::BoundaryProbes,
    ];
}

pub const CHARACTERIZATION_SCHEDULE_SHA256: Option<Digest32> = None;
pub const ACCEPTANCE_SCHEDULE_SHA256: Option<Digest32> = None;
pub const CHARACTERIZATION_MANIFEST_SHA256: Option<Digest32> = None;
pub const POSITIVE_CASE_MANIFEST_SHA256: Option<Digest32> = None;
pub const BOUNDARY_PROBE_MANIFEST_SHA256: Option<Digest32> = None;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManifestDigestNotFrozen {
    pub kind: ManifestKindV1,
}

pub const fn frozen_manifest_sha256(
    kind: ManifestKindV1,
) -> Result<Digest32, ManifestDigestNotFrozen> {
    let digest = match kind {
        ManifestKindV1::CharacterizationSchedule => CHARACTERIZATION_SCHEDULE_SHA256,
        ManifestKindV1::AcceptanceSchedule => ACCEPTANCE_SCHEDULE_SHA256,
        ManifestKindV1::Characterization => CHARACTERIZATION_MANIFEST_SHA256,
        ManifestKindV1::PositiveCases => POSITIVE_CASE_MANIFEST_SHA256,
        ManifestKindV1::BoundaryProbes => BOUNDARY_PROBE_MANIFEST_SHA256,
    };
    match digest {
        Some(value) => Ok(value),
        None => Err(ManifestDigestNotFrozen { kind }),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueryIdentity {
    pub byte_len: usize,
    pub sha256: Digest32,
}

pub const INSERT_QUERY_IDENTITY: QueryIdentity = QueryIdentity {
    byte_len: INSERT_QUERY_LEN,
    sha256: INSERT_QUERY_SHA256,
};
pub const READ_QUERY_IDENTITY: QueryIdentity = QueryIdentity {
    byte_len: READ_QUERY_LEN,
    sha256: READ_QUERY_SHA256,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectorMode {
    BoundaryProbe,
    Writer,
    Contender,
    ReleaseProbe,
    Reader,
}

impl CollectorMode {
    pub const ALL: [Self; 5] = [
        Self::BoundaryProbe,
        Self::Writer,
        Self::Contender,
        Self::ReleaseProbe,
        Self::Reader,
    ];

    pub const fn argv_token(self) -> &'static str {
        match self {
            Self::BoundaryProbe => "boundary-probe",
            Self::Writer => "writer",
            Self::Contender => "contender",
            Self::ReleaseProbe => "release-probe",
            Self::Reader => "reader",
        }
    }

    pub const fn role_id(self) -> RoleId {
        match self {
            Self::BoundaryProbe => RoleId::Boundary,
            Self::Writer => RoleId::Writer,
            Self::Contender => RoleId::Contender,
            Self::ReleaseProbe => RoleId::ReleaseProbe,
            Self::Reader => RoleId::Reader,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectorArgvError {
    WrongArgumentCount,
    UnknownMode,
}

pub fn parse_collector_arguments(
    arguments_after_argv0: &[&str],
) -> Result<CollectorMode, CollectorArgvError> {
    let [token] = arguments_after_argv0 else {
        return Err(CollectorArgvError::WrongArgumentCount);
    };
    CollectorMode::ALL
        .into_iter()
        .find(|mode| mode.argv_token() == *token)
        .ok_or(CollectorArgvError::UnknownMode)
}

pub const BASE_ENVIRONMENT: [&str; 5] = [
    "LANG=C",
    "LC_ALL=C",
    "TZ=UTC",
    "RUST_BACKTRACE=0",
    "TMPDIR=/tmp",
];

pub const ROCKS_ENVIRONMENT: [&str; 35] = [
    "SURREAL_SYNC_DATA=true",
    "SURREAL_ROCKSDB_BACKGROUND_FLUSH=false",
    "SURREAL_ROCKSDB_BACKGROUND_FLUSH_INTERVAL=200",
    "SURREAL_ROCKSDB_THREAD_COUNT=2",
    "SURREAL_ROCKSDB_JOBS_COUNT=2",
    "SURREAL_ROCKSDB_MAX_OPEN_FILES=128",
    "SURREAL_ROCKSDB_BLOCK_SIZE=65536",
    "SURREAL_ROCKSDB_WAL_SIZE_LIMIT=64",
    "SURREAL_ROCKSDB_MAX_WRITE_BUFFER_NUMBER=2",
    "SURREAL_ROCKSDB_WRITE_BUFFER_SIZE=33554432",
    "SURREAL_ROCKSDB_TARGET_FILE_SIZE_BASE=67108864",
    "SURREAL_ROCKSDB_TARGET_FILE_SIZE_MULTIPLIER=2",
    "SURREAL_ROCKSDB_MIN_WRITE_BUFFER_NUMBER_TO_MERGE=2",
    "SURREAL_ROCKSDB_FILE_COMPACTION_TRIGGER=4",
    "SURREAL_ROCKSDB_COMPACTION_READAHEAD_SIZE=4194304",
    "SURREAL_ROCKSDB_MAX_CONCURRENT_SUBCOMPACTIONS=2",
    "SURREAL_ROCKSDB_ENABLE_PIPELINED_WRITES=false",
    "SURREAL_ROCKSDB_ENABLE_BLOB_FILES=false",
    "SURREAL_ROCKSDB_MIN_BLOB_SIZE=4096",
    "SURREAL_ROCKSDB_BLOB_FILE_SIZE=268435456",
    "SURREAL_ROCKSDB_BLOB_COMPRESSION_TYPE=none",
    "SURREAL_ROCKSDB_ENABLE_BLOB_GC=false",
    "SURREAL_ROCKSDB_BLOB_GC_AGE_CUTOFF=0.25",
    "SURREAL_ROCKSDB_BLOB_GC_FORCE_THRESHOLD=1.0",
    "SURREAL_ROCKSDB_BLOB_COMPACTION_READAHEAD_SIZE=0",
    "SURREAL_ROCKSDB_BLOCK_CACHE_SIZE=33554432",
    "SURREAL_ROCKSDB_ENABLE_MEMORY_MAPPED_READS=false",
    "SURREAL_ROCKSDB_ENABLE_MEMORY_MAPPED_WRITES=false",
    "SURREAL_ROCKSDB_KEEP_LOG_FILE_NUM=3",
    "SURREAL_ROCKSDB_STORAGE_LOG_LEVEL=warn",
    "SURREAL_ROCKSDB_COMPACTION_STYLE=level",
    "SURREAL_ROCKSDB_DELETION_FACTORY_WINDOW_SIZE=1000",
    "SURREAL_ROCKSDB_DELETION_FACTORY_DELETE_COUNT=50",
    "SURREAL_ROCKSDB_DELETION_FACTORY_RATIO=0.5",
    "SURREAL_ROCKSDB_SST_MAX_ALLOWED_SPACE_USAGE=0",
];

pub const FULL_ENVIRONMENT_COUNT: usize = BASE_ENVIRONMENT.len() + ROCKS_ENVIRONMENT.len();

pub fn rocks_environment_bytes() -> Vec<u8> {
    newline_terminated_environment(&ROCKS_ENVIRONMENT)
}

/// Exact raw bytes visible through `/proc/<pid>/environ`: source-ordered `name=value` entries,
/// each terminated by one NUL byte.  The terminating null pointer in `envp` is not part of this
/// byte string.
pub fn collector_envp_bytes() -> Vec<u8> {
    let capacity = BASE_ENVIRONMENT
        .iter()
        .chain(ROCKS_ENVIRONMENT.iter())
        .map(|entry| entry.len() + 1)
        .sum();
    let mut bytes = Vec::with_capacity(capacity);
    for entry in BASE_ENVIRONMENT.into_iter().chain(ROCKS_ENVIRONMENT) {
        bytes.extend_from_slice(entry.as_bytes());
        bytes.push(0);
    }
    bytes
}

/// Backward-compatible name for the complete raw collector environment.  Unlike the separately
/// frozen Rocks table representation, this is NUL-separated because it models `envp`/proc bytes.
pub fn full_environment_bytes() -> Vec<u8> {
    collector_envp_bytes()
}

pub fn collector_envp_is_exact(observed: &[u8]) -> bool {
    observed == collector_envp_bytes()
}

fn newline_terminated_environment<const N: usize>(entries: &[&str; N]) -> Vec<u8> {
    let capacity = entries.iter().map(|entry| entry.len() + 1).sum();
    let mut bytes = Vec::with_capacity(capacity);
    for entry in entries {
        bytes.extend_from_slice(entry.as_bytes());
        bytes.push(b'\n');
    }
    bytes
}

pub fn environment_is_exact<'a>(entries: impl IntoIterator<Item = &'a str>) -> bool {
    let mut observed = entries.into_iter();
    for expected in BASE_ENVIRONMENT.into_iter().chain(ROCKS_ENVIRONMENT) {
        if observed.next() != Some(expected) {
            return false;
        }
    }
    observed.next().is_none()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum RoleId {
    Supervisor = 0,
    Boundary = 1,
    Writer = 2,
    Contender = 3,
    ReleaseProbe = 4,
    Reader = 5,
    JourneyAggregate = 6,
}

impl TryFrom<u16> for RoleId {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Supervisor),
            1 => Ok(Self::Boundary),
            2 => Ok(Self::Writer),
            3 => Ok(Self::Contender),
            4 => Ok(Self::ReleaseProbe),
            5 => Ok(Self::Reader),
            6 => Ok(Self::JourneyAggregate),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum MessageKind {
    Start = 1,
    Ready = 2,
    Continue = 3,
    WriterCommitted = 4,
    LockObserved = 5,
    ExpectedLockRejected = 6,
    HandlesDropped = 7,
    RoleTerminal = 8,
    Abort = 12,
    HostStart = 19,
    HostReady = 20,
    Measurement = 21,
    Challenge = 22,
    Terminal = 23,
    TerminalReady = 24,
}

impl TryFrom<u16> for MessageKind {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Start),
            2 => Ok(Self::Ready),
            3 => Ok(Self::Continue),
            4 => Ok(Self::WriterCommitted),
            5 => Ok(Self::LockObserved),
            6 => Ok(Self::ExpectedLockRejected),
            7 => Ok(Self::HandlesDropped),
            8 => Ok(Self::RoleTerminal),
            12 => Ok(Self::Abort),
            19 => Ok(Self::HostStart),
            20 => Ok(Self::HostReady),
            21 => Ok(Self::Measurement),
            22 => Ok(Self::Challenge),
            23 => Ok(Self::Terminal),
            24 => Ok(Self::TerminalReady),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ErrorCategory {
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

impl ErrorCategory {
    pub const fn header_code(self) -> u16 {
        self as u16
    }
}

impl TryFrom<u32> for ErrorCategory {
    type Error = ();

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
            _ => Err(()),
        }
    }
}

impl TryFrom<u16> for ErrorCategory {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        <Self as TryFrom<u32>>::try_from(u32::from(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixtureShape {
    SparseA,
    SparseB,
    NineEmptyTableArrays,
    OneRowPerTable,
    RowsInMemoryItem(u16),
    RowsInCorrectionProposal(u16),
    RowsInForgetReceipt(u16),
    RowsInWorkProject(u16),
    RowsInWorkTask(u16),
    RowsInGitRepository(u16),
    RowsInLocalCheckout(u16),
    RowsInMonorepoComponent(u16),
    RowsInProjectRepositoryLink(u16),
    ScalarBytesAcrossRows {
        bytes: u32,
        rows: u16,
    },
    ScalarElementsAcrossRows {
        elements: u16,
        element_bytes: u16,
        rows: u16,
    },
    ObjectPairsAcrossRows {
        pairs: u16,
        rows: u16,
    },
    ObjectDepthAcrossRows {
        depth: u16,
        rows: u16,
    },
    RawCanonicalBytes(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalibrationCase {
    pub case_id: u16,
    pub fixture_id: u16,
    pub profile_id: u16,
    pub name: &'static str,
    pub shape: FixtureShape,
    pub expected: ErrorCategory,
}

const fn calibration_case(id: u16, name: &'static str, shape: FixtureShape) -> CalibrationCase {
    CalibrationCase {
        case_id: id,
        fixture_id: id,
        profile_id: PROFILE_ID,
        name,
        shape,
        expected: ErrorCategory::Success,
    }
}

pub const CALIBRATION_CASES: [CalibrationCase; 20] = [
    calibration_case(1, "sparse_a_open", FixtureShape::SparseA),
    calibration_case(2, "sparse_b_open", FixtureShape::SparseB),
    calibration_case(3, "empty_generation", FixtureShape::NineEmptyTableArrays),
    calibration_case(4, "one_each", FixtureShape::OneRowPerTable),
    calibration_case(5, "memory_item_64", FixtureShape::RowsInMemoryItem(64)),
    calibration_case(
        6,
        "correction_proposal_32",
        FixtureShape::RowsInCorrectionProposal(32),
    ),
    calibration_case(
        7,
        "forget_receipt_32",
        FixtureShape::RowsInForgetReceipt(32),
    ),
    calibration_case(8, "work_project_8", FixtureShape::RowsInWorkProject(8)),
    calibration_case(9, "work_task_64", FixtureShape::RowsInWorkTask(64)),
    calibration_case(
        10,
        "git_repository_16",
        FixtureShape::RowsInGitRepository(16),
    ),
    calibration_case(
        11,
        "local_checkout_32",
        FixtureShape::RowsInLocalCheckout(32),
    ),
    calibration_case(
        12,
        "monorepo_component_64",
        FixtureShape::RowsInMonorepoComponent(64),
    ),
    calibration_case(
        13,
        "project_repository_link_64",
        FixtureShape::RowsInProjectRepositoryLink(64),
    ),
    calibration_case(
        14,
        "scalar_heavy",
        FixtureShape::ScalarBytesAcrossRows {
            bytes: 1_048_576,
            rows: 16,
        },
    ),
    calibration_case(
        15,
        "vector_heavy",
        FixtureShape::ScalarElementsAcrossRows {
            elements: 4_096,
            element_bytes: 8,
            rows: 16,
        },
    ),
    calibration_case(
        16,
        "object_key_heavy",
        FixtureShape::ObjectPairsAcrossRows {
            pairs: 4_096,
            rows: 16,
        },
    ),
    calibration_case(
        17,
        "depth_heavy",
        FixtureShape::ObjectDepthAcrossRows {
            depth: 32,
            rows: 64,
        },
    ),
    calibration_case(18, "raw_2mib", FixtureShape::RawCanonicalBytes(2_097_152)),
    calibration_case(19, "sparse_a_close", FixtureShape::SparseA),
    calibration_case(20, "sparse_b_close", FixtureShape::SparseB),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum InstanceAutomaton {
    Characterization = 1,
    Positive = 2,
    SharedBoundary = 3,
    DedicatedBoundary = 4,
    HostTerminal = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScheduleEntry {
    pub phase_code: u16,
    pub class_code: u16,
    pub ordinal: u16,
    pub name: &'static str,
    pub automaton: InstanceAutomaton,
}

pub static CHARACTERIZATION_SCHEDULE: [ScheduleEntry; 1] = [ScheduleEntry {
    phase_code: 1,
    class_code: 1,
    ordinal: 1,
    name: "C01",
    automaton: InstanceAutomaton::Characterization,
}];

pub static ACCEPTANCE_SCHEDULE: [ScheduleEntry; 10] = [
    ScheduleEntry {
        phase_code: 2,
        class_code: 101,
        ordinal: 1,
        name: "P01",
        automaton: InstanceAutomaton::Positive,
    },
    ScheduleEntry {
        phase_code: 2,
        class_code: 102,
        ordinal: 2,
        name: "P02",
        automaton: InstanceAutomaton::Positive,
    },
    ScheduleEntry {
        phase_code: 2,
        class_code: 103,
        ordinal: 3,
        name: "P03",
        automaton: InstanceAutomaton::Positive,
    },
    ScheduleEntry {
        phase_code: 2,
        class_code: 201,
        ordinal: 4,
        name: "B01",
        automaton: InstanceAutomaton::SharedBoundary,
    },
    ScheduleEntry {
        phase_code: 2,
        class_code: 212,
        ordinal: 5,
        name: "B12O",
        automaton: InstanceAutomaton::DedicatedBoundary,
    },
    ScheduleEntry {
        phase_code: 2,
        class_code: 213,
        ordinal: 6,
        name: "B12P",
        automaton: InstanceAutomaton::DedicatedBoundary,
    },
    ScheduleEntry {
        phase_code: 2,
        class_code: 215,
        ordinal: 7,
        name: "B15",
        automaton: InstanceAutomaton::DedicatedBoundary,
    },
    ScheduleEntry {
        phase_code: 2,
        class_code: 216,
        ordinal: 8,
        name: "B16",
        automaton: InstanceAutomaton::DedicatedBoundary,
    },
    ScheduleEntry {
        phase_code: 2,
        class_code: 240,
        ordinal: 9,
        name: "B40",
        automaton: InstanceAutomaton::HostTerminal,
    },
    ScheduleEntry {
        phase_code: 2,
        class_code: 241,
        ordinal: 10,
        name: "B41",
        automaton: InstanceAutomaton::HostTerminal,
    },
];

pub fn select_schedule_entry(
    phase_code: u16,
    class_code: u16,
    ordinal: u16,
) -> Option<&'static ScheduleEntry> {
    CHARACTERIZATION_SCHEDULE
        .iter()
        .chain(ACCEPTANCE_SCHEDULE.iter())
        .find(|entry| {
            entry.phase_code == phase_code
                && entry.class_code == class_code
                && entry.ordinal == ordinal
        })
}

pub const fn host_schedule_manifest_kind(entry: &ScheduleEntry) -> ManifestKindV1 {
    match entry.automaton {
        InstanceAutomaton::Characterization => ManifestKindV1::CharacterizationSchedule,
        InstanceAutomaton::Positive
        | InstanceAutomaton::SharedBoundary
        | InstanceAutomaton::DedicatedBoundary
        | InstanceAutomaton::HostTerminal => ManifestKindV1::AcceptanceSchedule,
    }
}

pub const CHARACTERIZATION_SUBATTEMPTS: u16 = 3;
pub const CHARACTERIZATION_MEASUREMENTS: u16 = 21;
pub const POSITIVE_CASES: u16 = 20;
pub const POSITIVE_MEASUREMENTS_PER_INSTANCE: u16 = 140;
pub const POSITIVE_INSTANCE_COUNT: u16 = 3;
pub const SHARED_BOUNDARY_MEASUREMENTS: u16 = 60;
pub const DEDICATED_BOUNDARY_INSTANCE_COUNT: u16 = 4;
pub const ACCEPTANCE_BOUNDARY_MEASUREMENTS: u16 = 64;
pub const ACCEPTANCE_GUEST_MEASUREMENTS: u16 = 484;
pub const ACCEPTANCE_BOUNDARY_SUBATTEMPTS: u16 = 66;
pub const LOGICAL_PROBE_COUNT: u16 = 41;

pub const fn probe_subattempt_count(probe_id: u16) -> Option<u16> {
    match probe_id {
        1..=41 => Some(match probe_id {
            12 | 32 => 2,
            33 => 14,
            35 => 8,
            36 => 4,
            _ => 1,
        }),
        _ => None,
    }
}

pub const fn probe_is_in_shared_boundary(probe_id: u16) -> bool {
    probe_id >= 1 && probe_id <= 39 && probe_id != 12 && probe_id != 15 && probe_id != 16
}

pub const fn shared_boundary_subattempt_count() -> u16 {
    let mut probe = 1;
    let mut count = 0;
    while probe <= 39 {
        if probe_is_in_shared_boundary(probe) {
            count += match probe_subattempt_count(probe) {
                Some(value) => value,
                None => 0,
            };
        }
        probe += 1;
    }
    count
}

pub const fn measurement_count(entry: &ScheduleEntry) -> u16 {
    match entry.automaton {
        InstanceAutomaton::Characterization => CHARACTERIZATION_MEASUREMENTS,
        InstanceAutomaton::Positive => POSITIVE_MEASUREMENTS_PER_INSTANCE,
        InstanceAutomaton::SharedBoundary => SHARED_BOUNDARY_MEASUREMENTS,
        InstanceAutomaton::DedicatedBoundary => 1,
        InstanceAutomaton::HostTerminal => 0,
    }
}

pub const fn terminal_ready_sequence(entry: &ScheduleEntry) -> Option<u32> {
    match entry.automaton {
        InstanceAutomaton::HostTerminal => None,
        _ => Some(measurement_count(entry) as u32 + 2),
    }
}

pub const fn terminal_sequence(entry: &ScheduleEntry) -> Option<u32> {
    match terminal_ready_sequence(entry) {
        Some(value) => Some(value + 1),
        None => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeasurementIdentity {
    pub item_id: u16,
    pub subitem_id: u16,
    pub scope_code: u16,
    pub role_id: RoleId,
}

pub fn expected_measurement(
    entry: &ScheduleEntry,
    zero_based_index: u16,
) -> Option<MeasurementIdentity> {
    if zero_based_index >= measurement_count(entry) {
        return None;
    }
    match entry.automaton {
        InstanceAutomaton::Characterization => {
            let subattempt_id = zero_based_index / 7 + 1;
            let scope_code = zero_based_index % 7 + 1;
            Some(MeasurementIdentity {
                item_id: 1,
                subitem_id: subattempt_id,
                scope_code,
                role_id: role_for_scope(scope_code)?,
            })
        }
        InstanceAutomaton::Positive => {
            let case_id = zero_based_index / 7 + 1;
            let scope_code = zero_based_index % 7 + 1;
            Some(MeasurementIdentity {
                item_id: case_id,
                subitem_id: case_id,
                scope_code,
                role_id: role_for_scope(scope_code)?,
            })
        }
        InstanceAutomaton::SharedBoundary => {
            let mut remaining = zero_based_index;
            let mut probe = 1;
            while probe <= 39 {
                if probe_is_in_shared_boundary(probe) {
                    let count = probe_subattempt_count(probe)?;
                    if remaining < count {
                        return Some(MeasurementIdentity {
                            item_id: probe,
                            subitem_id: remaining + 1,
                            scope_code: 8,
                            role_id: RoleId::Boundary,
                        });
                    }
                    remaining -= count;
                }
                probe += 1;
            }
            None
        }
        InstanceAutomaton::DedicatedBoundary => {
            let (probe_id, subattempt_id) = match entry.class_code {
                212 => (12, 1),
                213 => (12, 2),
                215 => (15, 1),
                216 => (16, 1),
                _ => return None,
            };
            Some(MeasurementIdentity {
                item_id: probe_id,
                subitem_id: subattempt_id,
                scope_code: 8,
                role_id: RoleId::Boundary,
            })
        }
        InstanceAutomaton::HostTerminal => None,
    }
}

pub const fn role_for_scope(scope_code: u16) -> Option<RoleId> {
    match scope_code {
        1 => Some(RoleId::Writer),
        2 => Some(RoleId::Contender),
        3 => Some(RoleId::Writer),
        4 => Some(RoleId::ReleaseProbe),
        5 => Some(RoleId::ReleaseProbe),
        6 => Some(RoleId::Reader),
        7 => Some(RoleId::JourneyAggregate),
        8 => Some(RoleId::Boundary),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceProfileV1 {
    pub guest_cpus: u16,
    pub guest_memory_bytes: u64,
    pub supervisor_memory_max: u64,
    pub supervisor_swap_max: u64,
    pub supervisor_pids_max: u32,
    pub supervisor_oom_group: bool,
    pub supervisor_cpu_quota: u32,
    pub supervisor_cpu_period: u32,
    pub supervisor_nofile: u64,
    pub supervisor_fsize: u64,
    pub supervisor_core: u64,
    pub workload_memory_max: u64,
    pub workload_swap_max: u64,
    pub guest_swap_absent: bool,
    pub workload_pids_max: u32,
    pub workload_oom_group: bool,
    pub workload_cpu_quota: u32,
    pub workload_cpu_period: u32,
    pub normal_cpu_soft_seconds: u64,
    pub normal_cpu_hard_seconds: u64,
    pub probe_cpu_soft_seconds: u64,
    pub probe_cpu_hard_seconds: u64,
    pub role_nofile: u64,
    pub role_fsize: u64,
    pub role_core: u64,
    pub role_wall_seconds: u64,
    pub journey_seconds: u64,
    pub characterization_seconds: u64,
    pub positive_or_shared_seconds: u64,
    pub dedicated_seconds: u64,
    pub acceptance_campaign_seconds: u64,
    pub complete_campaign_seconds: u64,
    pub per_role_output_bytes: u64,
    pub retained_evidence_bytes: u64,
    pub terminal_receipt_bytes: u64,
}

pub const RESOURCE_PROFILE_V1: ResourceProfileV1 = ResourceProfileV1 {
    guest_cpus: 2,
    guest_memory_bytes: 4 * 1024 * 1024 * 1024,
    supervisor_memory_max: 512 * 1024 * 1024,
    supervisor_swap_max: 0,
    supervisor_pids_max: 32,
    supervisor_oom_group: true,
    supervisor_cpu_quota: 100_000,
    supervisor_cpu_period: 100_000,
    supervisor_nofile: 512,
    supervisor_fsize: 64 * 1024 * 1024,
    supervisor_core: 0,
    workload_memory_max: 1024 * 1024 * 1024,
    workload_swap_max: 0,
    guest_swap_absent: true,
    workload_pids_max: 128,
    workload_oom_group: true,
    workload_cpu_quota: 100_000,
    workload_cpu_period: 100_000,
    normal_cpu_soft_seconds: 30,
    normal_cpu_hard_seconds: 30,
    probe_cpu_soft_seconds: 1,
    probe_cpu_hard_seconds: 2,
    role_nofile: 256,
    role_fsize: 256 * 1024 * 1024,
    role_core: 0,
    role_wall_seconds: 60,
    journey_seconds: 240,
    characterization_seconds: 600,
    positive_or_shared_seconds: 2_700,
    dedicated_seconds: 600,
    acceptance_campaign_seconds: 18_000,
    complete_campaign_seconds: 18_600,
    per_role_output_bytes: 65_536,
    retained_evidence_bytes: 64 * 1024 * 1024,
    terminal_receipt_bytes: 256 * 1024,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DescriptorType {
    DevNullCharacter = 0,
    PipeReadEnd = 1,
    PipeWriteEnd = 2,
    DataRegularFile = 3,
    DataDirectory = 4,
    TmpRegularFile = 5,
    TmpDirectory = 6,
    DiagnosticRegularFile = 7,
    DiagnosticDirectory = 8,
    EventFd = 9,
    EventPoll = 10,
    TimerFd = 11,
    ReadOnlyProcfs = 12,
    OtherAnonInode = 13,
    Socket = 14,
    BlockDevice = 15,
    OtherCharacter = 16,
    OtherRegularFile = 17,
    OtherDirectory = 18,
    Unknown = 19,
}

impl DescriptorType {
    pub const fn bit(self) -> u64 {
        1_u64 << (self as u8)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MeasurementV1 {
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

impl MeasurementV1 {
    pub const FIELD_COUNT: usize = 32;
    pub const ENCODED_LEN: usize = Self::FIELD_COUNT * 8;

    pub const fn from_words(words: [u64; Self::FIELD_COUNT]) -> Self {
        Self {
            scope_code: words[0],
            memory_current_bytes: words[1],
            memory_peak_bytes: words[2],
            memory_events_oom: words[3],
            memory_events_oom_kill: words[4],
            swap_current_bytes: words[5],
            swap_peak_bytes: words[6],
            swap_events_max: words[7],
            pids_current: words[8],
            pids_peak: words[9],
            pids_events_max: words[10],
            cpu_usage_usec: words[11],
            cpu_user_usec: words[12],
            cpu_system_usec: words[13],
            cpu_nr_periods: words[14],
            cpu_nr_throttled: words[15],
            cpu_throttled_usec: words[16],
            filesystem_logical_bytes: words[17],
            filesystem_allocated_bytes: words[18],
            filesystem_inode_count: words[19],
            filesystem_file_count: words[20],
            tmpfs_bytes: words[21],
            tmpfs_inode_count: words[22],
            diagnostic_bytes: words[23],
            diagnostic_inode_count: words[24],
            stdout_bytes: words[25],
            stderr_bytes: words[26],
            role_control_input_bytes: words[27],
            role_control_output_bytes: words[28],
            descriptor_count: words[29],
            descriptor_type_bitmap: words[30],
            monotonic_elapsed_ns: words[31],
        }
    }

    pub const fn words(self) -> [u64; Self::FIELD_COUNT] {
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

    pub fn validate_for(
        self,
        probe_id: Option<u16>,
        role_id: RoleId,
    ) -> Result<(), MeasurementError> {
        let scope = u16::try_from(self.scope_code).map_err(|_| MeasurementError::UnknownScope)?;
        if role_for_scope(scope) != Some(role_id) {
            return Err(MeasurementError::WrongRole);
        }

        if matches!(probe_id, Some(15 | 16)) {
            if scope != 8 {
                return Err(MeasurementError::WrongScopeForPreStartProbe);
            }
            let words = self.words();
            if words[1..31].iter().any(|word| *word != 0) {
                return Err(MeasurementError::InapplicableFieldWasNonzero);
            }
            return Ok(());
        }

        let allowed_bitmap = match probe_id {
            Some(18) => ((1_u64 << 13) - 1) | DescriptorType::Socket.bit(),
            _ => (1_u64 << 13) - 1,
        };
        if self.descriptor_type_bitmap & !allowed_bitmap != 0 {
            return Err(MeasurementError::ForbiddenDescriptorType);
        }
        if probe_id == Some(17)
            && self.descriptor_type_bitmap & DescriptorType::DiagnosticRegularFile.bit() == 0
        {
            return Err(MeasurementError::MissingProbeDescriptor);
        }
        if probe_id == Some(18) && self.descriptor_type_bitmap & DescriptorType::Socket.bit() == 0 {
            return Err(MeasurementError::MissingProbeDescriptor);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeasurementError {
    UnknownScope,
    WrongRole,
    WrongScopeForPreStartProbe,
    InapplicableFieldWasNonzero,
    ForbiddenDescriptorType,
    MissingProbeDescriptor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeasurementArithmeticError {
    Underflow,
    Overflow,
    OutputLimitExceeded,
}

pub fn checked_elapsed_ns(start: u64, end: u64) -> Result<u64, MeasurementArithmeticError> {
    end.checked_sub(start)
        .ok_or(MeasurementArithmeticError::Underflow)
}

pub fn checked_sum(values: &[u64]) -> Result<u64, MeasurementArithmeticError> {
    values.iter().try_fold(0_u64, |total, value| {
        total
            .checked_add(*value)
            .ok_or(MeasurementArithmeticError::Overflow)
    })
}

pub fn checked_allocated_bytes(st_blocks: u64) -> Result<u64, MeasurementArithmeticError> {
    st_blocks
        .checked_mul(512)
        .ok_or(MeasurementArithmeticError::Overflow)
}

pub fn checked_statfs_used(
    total_units: u64,
    free_units: u64,
    unit_bytes: u64,
) -> Result<u64, MeasurementArithmeticError> {
    total_units
        .checked_sub(free_units)
        .ok_or(MeasurementArithmeticError::Underflow)?
        .checked_mul(unit_bytes)
        .ok_or(MeasurementArithmeticError::Overflow)
}

pub fn checked_role_output_bytes(
    stdout_bytes: u64,
    stderr_bytes: u64,
) -> Result<u64, MeasurementArithmeticError> {
    let total = stdout_bytes
        .checked_add(stderr_bytes)
        .ok_or(MeasurementArithmeticError::Overflow)?;
    if total > RESOURCE_PROFILE_V1.per_role_output_bytes {
        return Err(MeasurementArithmeticError::OutputLimitExceeded);
    }
    Ok(total)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TerminalStateV1 {
    pub primary_category: u32,
    pub active_item_code: u32,
    pub guest_cleanup_proven: u32,
    pub write_may_have_been_dispatched: u32,
    pub writer_acknowledged: u32,
    pub roles_reaped: u32,
    pub completed_case_count: u32,
    pub completed_subattempt_count: u32,
}

impl TerminalStateV1 {
    pub const FIELD_COUNT: usize = 8;
    pub const ENCODED_LEN: usize = Self::FIELD_COUNT * 4;

    pub const fn words(self) -> [u32; Self::FIELD_COUNT] {
        [
            self.primary_category,
            self.active_item_code,
            self.guest_cleanup_proven,
            self.write_may_have_been_dispatched,
            self.writer_acknowledged,
            self.roles_reaped,
            self.completed_case_count,
            self.completed_subattempt_count,
        ]
    }

    pub const fn from_words(words: [u32; Self::FIELD_COUNT]) -> Self {
        Self {
            primary_category: words[0],
            active_item_code: words[1],
            guest_cleanup_proven: words[2],
            write_may_have_been_dispatched: words[3],
            writer_acknowledged: words[4],
            roles_reaped: words[5],
            completed_case_count: words[6],
            completed_subattempt_count: words[7],
        }
    }

    pub fn validate(self) -> Result<(), TerminalStateError> {
        ErrorCategory::try_from(self.primary_category)
            .map_err(|_| TerminalStateError::UnknownCategory)?;
        let item_id = self.active_item_code >> 16;
        let subitem_id = self.active_item_code & u32::from(u16::MAX);
        if (item_id == 0) != (subitem_id == 0) {
            return Err(TerminalStateError::HalfZeroActiveItem);
        }
        for value in [
            self.guest_cleanup_proven,
            self.write_may_have_been_dispatched,
            self.writer_acknowledged,
            self.roles_reaped,
        ] {
            if value > 1 {
                return Err(TerminalStateError::NonBoolean);
            }
        }
        if self.guest_cleanup_proven == 1 && self.roles_reaped == 0 {
            return Err(TerminalStateError::CleanupWithoutReap);
        }
        if self.writer_acknowledged == 1 && self.write_may_have_been_dispatched == 0 {
            return Err(TerminalStateError::AcknowledgementWithoutDispatch);
        }
        Ok(())
    }

    pub fn validate_success_for(self, entry: &ScheduleEntry) -> Result<(), TerminalStateError> {
        self.validate()?;
        match successful_terminal(entry) {
            Some(expected) if self == expected => Ok(()),
            _ => Err(TerminalStateError::WrongSuccessState),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalStateError {
    UnknownCategory,
    HalfZeroActiveItem,
    NonBoolean,
    CleanupWithoutReap,
    AcknowledgementWithoutDispatch,
    WrongSuccessState,
}

pub const fn active_item_code(item_id: u16, subitem_id: u16) -> u32 {
    ((item_id as u32) << 16) | subitem_id as u32
}

pub const fn successful_terminal(entry: &ScheduleEntry) -> Option<TerminalStateV1> {
    let (may_dispatch, acknowledged, cases, subattempts) = match entry.automaton {
        InstanceAutomaton::Characterization => (1, 1, 0, 3),
        InstanceAutomaton::Positive => (1, 1, 20, 0),
        InstanceAutomaton::SharedBoundary => (0, 0, 0, 60),
        InstanceAutomaton::DedicatedBoundary => (0, 0, 0, 1),
        InstanceAutomaton::HostTerminal => return None,
    };
    Some(TerminalStateV1 {
        primary_category: ErrorCategory::Success as u32,
        active_item_code: 0,
        guest_cleanup_proven: 1,
        write_may_have_been_dispatched: may_dispatch,
        writer_acknowledged: acknowledged,
        roles_reaped: 1,
        completed_case_count: cases,
        completed_subattempt_count: subattempts,
    })
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ActiveItemState {
    pub active_item_code: u32,
    pub write_may_have_been_dispatched: bool,
    pub writer_acknowledged: bool,
}

impl ActiveItemState {
    pub fn begin(&mut self, item_id: u16, subitem_id: u16) {
        self.active_item_code = active_item_code(item_id, subitem_id);
        self.write_may_have_been_dispatched = false;
        self.writer_acknowledged = false;
    }

    pub fn mark_full_dispatch(&mut self) {
        self.write_may_have_been_dispatched = true;
    }

    pub fn mark_writer_acknowledged(&mut self) -> Result<(), TerminalStateError> {
        if !self.write_may_have_been_dispatched {
            return Err(TerminalStateError::AcknowledgementWithoutDispatch);
        }
        self.writer_acknowledged = true;
        Ok(())
    }

    pub fn finish(&mut self) {
        self.active_item_code = 0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailurePhase {
    BeforeDispatch,
    DispatchOutcomeUncertain,
    AfterWriterAcknowledgement,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FailureSnapshot {
    pub protocol: bool,
    pub host_contract: bool,
    pub guest_configuration: bool,
    pub containment: bool,
    pub static_firewall: bool,
    pub limit_exceeded: bool,
    pub invalid_fixture: bool,
    pub store_open: bool,
    pub current_stage: Option<PostAcknowledgementFailure>,
    pub measurement: bool,
    pub challenge: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostAcknowledgementFailure {
    LockWitnessMissing,
    ContenderOpened,
    ContenderWrongError,
    WriterReleaseUnproven,
    ReleaseProbeFailure,
    ReaderOpenFailure,
    ReadOrDecodeFailure,
    CanonicalDisagreement,
}

impl PostAcknowledgementFailure {
    pub const fn category(self) -> ErrorCategory {
        match self {
            Self::LockWitnessMissing | Self::ContenderOpened => ErrorCategory::ExclusivityBroken,
            Self::ContenderWrongError => ErrorCategory::ContenderFailure,
            Self::WriterReleaseUnproven | Self::ReleaseProbeFailure => {
                ErrorCategory::ReleaseUnproven
            }
            Self::ReaderOpenFailure => ErrorCategory::ReopenFailure,
            Self::ReadOrDecodeFailure => ErrorCategory::EngineReadFailure,
            Self::CanonicalDisagreement => ErrorCategory::CanonicalMismatch,
        }
    }
}

pub fn classify_failure(phase: FailurePhase, snapshot: FailureSnapshot) -> Option<ErrorCategory> {
    if phase == FailurePhase::DispatchOutcomeUncertain {
        return Some(ErrorCategory::OutcomeUncertain);
    }

    for (present, category) in [
        (snapshot.protocol, ErrorCategory::ProtocolFailure),
        (snapshot.host_contract, ErrorCategory::HostContract),
        (
            snapshot.guest_configuration,
            ErrorCategory::GuestConfiguration,
        ),
        (snapshot.containment, ErrorCategory::Containment),
    ] {
        if present {
            return Some(category);
        }
    }

    match phase {
        FailurePhase::BeforeDispatch => {
            for (present, category) in [
                (snapshot.static_firewall, ErrorCategory::StaticFirewall),
                (snapshot.limit_exceeded, ErrorCategory::LimitExceeded),
                (snapshot.invalid_fixture, ErrorCategory::InvalidFixture),
                (snapshot.store_open, ErrorCategory::StoreOpenFailure),
                (snapshot.measurement, ErrorCategory::MeasurementFailure),
                (snapshot.challenge, ErrorCategory::ChallengeFailure),
            ] {
                if present {
                    return Some(category);
                }
            }
        }
        FailurePhase::AfterWriterAcknowledgement => {
            if snapshot.limit_exceeded {
                return Some(ErrorCategory::LimitExceeded);
            }
            if let Some(stage) = snapshot.current_stage {
                return Some(stage.category());
            }
            if snapshot.measurement {
                return Some(ErrorCategory::MeasurementFailure);
            }
            if snapshot.challenge {
                return Some(ErrorCategory::ChallengeFailure);
            }
        }
        FailurePhase::DispatchOutcomeUncertain => unreachable!(),
    }
    None
}
