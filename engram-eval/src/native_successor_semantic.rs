use engram_core::id::Id;
use engram_core::memory::{
    ClaimOrigin, CorrectionProposal, CorrectionProposalStatus, EvidenceKind, EvidenceRef, Harness,
    MemoryItem, MemoryKind, MemoryScope, MemoryStatus, ProcedurePrerequisiteSource,
};
use engram_core::repository::{
    GitRepository, LocalCheckout, MonorepoComponent, ProjectRepositoryLink, ProjectRepositoryRole,
    RepositoryProvider,
};
use engram_core::work::{Project, Task};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use time::{OffsetDateTime, UtcOffset};
use unicode_normalization::UnicodeNormalization;
use zeroize::Zeroize;

mod acquisition;

#[allow(dead_code)]
struct C2RawSnapshot {
    target: C2RawTarget,
    memory_item: Vec<Value>,
    correction_proposal: Vec<Value>,
    memory_forget_receipt: Vec<Value>,
    work_project: Vec<Value>,
    work_task: Vec<Value>,
    git_repository: Vec<Value>,
    local_checkout: Vec<Value>,
    monorepo_component: Vec<Value>,
    project_repository_link: Vec<Value>,
}

#[allow(dead_code)]
struct C2RawTarget {
    project_id: String,
    project_name: String,
    repository_id: String,
    repository_remote: String,
    checkout_id: String,
    checkout_path: String,
    task_id: Option<String>,
    task_name: Option<String>,
}

#[allow(dead_code)]
struct C2ValidatedSnapshot {
    memory_items: Vec<MemoryItem>,
    correction_proposals: Vec<CorrectionProposal>,
    projects: Vec<Project>,
    tasks: Vec<Task>,
    repositories: Vec<GitRepository>,
    checkouts: Vec<LocalCheckout>,
    components: Vec<MonorepoComponent>,
    project_links: Vec<ProjectRepositoryLink>,
    target: C2ResolvedTarget,
    applicable_memory_ids: Vec<Id>,
    target_affecting_correction_ids: Vec<Id>,
    pending_bindings: Vec<C2PriorCorrectionBinding>,
    audit: C2SanitizedAudit,
}

#[allow(dead_code)]
struct C2ResolvedTarget {
    project_id: Id,
    repository_id: Id,
    checkout_id: Id,
    task_id: Option<Id>,
}

#[allow(dead_code)]
struct C2PriorCorrectionBinding {
    proposal_id: Id,
    proposal_row: Value,
    obsolete_row: Value,
    replacement_row: Value,
    proposal: CorrectionProposal,
    obsolete: MemoryItem,
    replacement: MemoryItem,
    proposal_record_mac: [u8; 32],
    obsolete_record_mac: [u8; 32],
    replacement_record_mac: [u8; 32],
}

#[allow(dead_code)]
struct C2OneShotMacKey([u8; 32]);

#[cfg(test)]
thread_local! {
    static DROPPED_TEST_KEY_BYTES: std::cell::Cell<Option<[u8; 32]>> = const {
        std::cell::Cell::new(None)
    };
    static DROPPED_TEST_PAD_BYTES: std::cell::Cell<Option<[u8; 64]>> = const {
        std::cell::Cell::new(None)
    };
}

impl C2OneShotMacKey {
    #[cfg(test)]
    fn from_test_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl Drop for C2OneShotMacKey {
    fn drop(&mut self) {
        self.0.zeroize();
        #[cfg(test)]
        DROPPED_TEST_KEY_BYTES.with(|bytes| bytes.set(Some(self.0)));
    }
}

#[allow(dead_code)]
struct C2MacPad([u8; 64]);

impl Drop for C2MacPad {
    fn drop(&mut self) {
        self.0.zeroize();
        #[cfg(test)]
        DROPPED_TEST_PAD_BYTES.with(|bytes| bytes.set(Some(self.0)));
    }
}

#[derive(Debug, Eq, PartialEq)]
enum C2FramingError {
    Capacity,
    LengthOverflow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct C2Mac([u8; 32]);

impl C2Mac {
    fn to_lower_hex(self) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut encoded = String::with_capacity(64);
        for byte in self.0 {
            encoded.push(HEX[usize::from(byte >> 4)] as char);
            encoded.push(HEX[usize::from(byte & 0x0f)] as char);
        }
        encoded
    }
}

struct C2MacEngine {
    key: C2OneShotMacKey,
}

impl C2MacEngine {
    fn consume(key: C2OneShotMacKey) -> Self {
        Self { key }
    }

    #[allow(clippy::needless_borrows_for_generic_args)]
    fn authenticate(&self, message: &[u8]) -> C2Mac {
        let mut inner_pad = C2MacPad([0x36; 64]);
        let mut outer_pad = C2MacPad([0x5c; 64]);
        for ((inner, outer), key_byte) in inner_pad
            .0
            .iter_mut()
            .zip(outer_pad.0.iter_mut())
            .zip(self.key.0.iter())
        {
            *inner ^= *key_byte;
            *outer ^= *key_byte;
        }

        let mut inner_hasher = Sha256::new();
        inner_hasher.update(&inner_pad.0);
        inner_hasher.update(message);
        let mut inner_output: [u8; 32] = inner_hasher.finalize().into();

        let mut outer_hasher = Sha256::new();
        outer_hasher.update(&outer_pad.0);
        outer_hasher.update(&inner_output);
        let output = C2Mac(outer_hasher.finalize().into());

        inner_output.zeroize();
        inner_pad.0.zeroize();
        outer_pad.0.zeroize();
        output
    }
}

fn extend_checked(output: &mut Vec<u8>, bytes: &[u8]) -> Result<(), C2FramingError> {
    output
        .len()
        .checked_add(bytes.len())
        .ok_or(C2FramingError::LengthOverflow)?;
    output
        .try_reserve(bytes.len())
        .map_err(|_| C2FramingError::Capacity)?;
    output.extend_from_slice(bytes);
    Ok(())
}

fn push_checked(output: &mut Vec<u8>, byte: u8) -> Result<(), C2FramingError> {
    output
        .len()
        .checked_add(1)
        .ok_or(C2FramingError::LengthOverflow)?;
    output
        .try_reserve(1)
        .map_err(|_| C2FramingError::Capacity)?;
    output.push(byte);
    Ok(())
}

fn append_u64(output: &mut Vec<u8>, value: u64) -> Result<(), C2FramingError> {
    extend_checked(output, &value.to_be_bytes())
}

fn append_len(output: &mut Vec<u8>, value: usize) -> Result<(), C2FramingError> {
    let value = u64::try_from(value).map_err(|_| C2FramingError::LengthOverflow)?;
    append_u64(output, value)
}

fn canonical_json_value(value: &Value) -> Result<Vec<u8>, C2FramingError> {
    let mut output = Vec::new();
    canonical_json_value_into(value, &mut output)?;
    Ok(output)
}

fn canonical_json_value_into(value: &Value, output: &mut Vec<u8>) -> Result<(), C2FramingError> {
    match value {
        Value::Null => push_checked(output, 0x00),
        Value::Bool(false) => push_checked(output, 0x01),
        Value::Bool(true) => push_checked(output, 0x02),
        Value::Number(number) => {
            push_checked(output, 0x03)?;
            let spelling = number.to_string();
            append_len(output, spelling.len())?;
            extend_checked(output, spelling.as_bytes())
        }
        Value::String(string) => {
            push_checked(output, 0x04)?;
            append_len(output, string.len())?;
            extend_checked(output, string.as_bytes())
        }
        Value::Array(values) => {
            push_checked(output, 0x05)?;
            append_len(output, values.len())?;
            for value in values {
                canonical_json_value_into(value, output)?;
            }
            Ok(())
        }
        Value::Object(object) => {
            push_checked(output, 0x06)?;
            append_len(output, object.len())?;
            let mut members: Vec<_> = object.iter().collect();
            members.sort_unstable_by(|(left, _), (right, _)| left.as_bytes().cmp(right.as_bytes()));
            for (key, value) in members {
                append_len(output, key.len())?;
                extend_checked(output, key.as_bytes())?;
                canonical_json_value_into(value, output)?;
            }
            Ok(())
        }
    }
}

const RECORD_DOMAIN: &[u8] = b"engram-native-semantic-record-v1\0";
const TABLE_DOMAIN: &[u8] = b"engram-native-semantic-table-v1\0";
const STORE_DOMAIN: &[u8] = b"engram-native-semantic-store-v1\0";
const PROJECT_VIEW_DOMAIN: &[u8] = b"engram-native-semantic-project-view-v1\0";
const RELAY_CANDIDATES_DOMAIN: &[u8] = b"engram-native-semantic-relay-candidates-v1\0";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum C2TableTag {
    MemoryItem,
    CorrectionProposal,
    MemoryForgetReceipt,
    WorkProject,
    WorkTask,
    GitRepository,
    LocalCheckout,
    MonorepoComponent,
    ProjectRepositoryLink,
}

impl C2TableTag {
    const ALL: [Self; 9] = [
        Self::MemoryItem,
        Self::CorrectionProposal,
        Self::MemoryForgetReceipt,
        Self::WorkProject,
        Self::WorkTask,
        Self::GitRepository,
        Self::LocalCheckout,
        Self::MonorepoComponent,
        Self::ProjectRepositoryLink,
    ];

    fn as_bytes(self) -> &'static [u8] {
        match self {
            Self::MemoryItem => b"memory_item",
            Self::CorrectionProposal => b"correction_proposal",
            Self::MemoryForgetReceipt => b"memory_forget_receipt",
            Self::WorkProject => b"work_project",
            Self::WorkTask => b"work_task",
            Self::GitRepository => b"git_repository",
            Self::LocalCheckout => b"local_checkout",
            Self::MonorepoComponent => b"monorepo_component",
            Self::ProjectRepositoryLink => b"project_repository_link",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct C2RecordMac {
    id: [u8; 16],
    mac: C2Mac,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum C2CorrectionStatus {
    Pending,
    Applied,
}

impl C2CorrectionStatus {
    fn as_bytes(self) -> &'static [u8] {
        match self {
            Self::Pending => b"pending",
            Self::Applied => b"applied",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct C2CorrectionEdgeMac {
    proposal_id: [u8; 16],
    obsolete_id: [u8; 16],
    replacement_id: [u8; 16],
    status: C2CorrectionStatus,
    proposal_mac: C2Mac,
    obsolete_mac: C2Mac,
    replacement_mac: C2Mac,
}

#[derive(Clone, Copy)]
struct C2ProjectViewFrame<'a> {
    target_project: C2RecordMac,
    target_repository: C2RecordMac,
    target_checkout: C2RecordMac,
    target_task: Option<C2RecordMac>,
    competing_links: &'a [C2RecordMac],
    linked_components: &'a [C2RecordMac],
    correction_edges: &'a [C2CorrectionEdgeMac],
    applicable_memories: &'a [C2RecordMac],
}

fn append_frame(output: &mut Vec<u8>, bytes: &[u8]) -> Result<(), C2FramingError> {
    append_len(output, bytes.len())?;
    extend_checked(output, bytes)
}

fn append_record_mac(output: &mut Vec<u8>, record: C2RecordMac) -> Result<(), C2FramingError> {
    append_frame(output, &record.id)?;
    append_frame(output, &record.mac.0)
}

fn append_sorted_record_macs(
    output: &mut Vec<u8>,
    records: &[C2RecordMac],
) -> Result<(), C2FramingError> {
    append_len(output, records.len())?;
    let mut sorted = records.to_vec();
    sorted.sort_unstable_by_key(|record| record.id);
    for record in sorted {
        append_record_mac(output, record)?;
    }
    Ok(())
}

fn record_preimage(
    table: C2TableTag,
    id: [u8; 16],
    row: &Value,
) -> Result<Vec<u8>, C2FramingError> {
    let encoded_row = canonical_json_value(row)?;
    let mut output = Vec::new();
    extend_checked(&mut output, RECORD_DOMAIN)?;
    append_frame(&mut output, table.as_bytes())?;
    append_frame(&mut output, &id)?;
    append_frame(&mut output, &encoded_row)?;
    Ok(output)
}

fn table_preimage(table: C2TableTag, records: &[C2RecordMac]) -> Result<Vec<u8>, C2FramingError> {
    let mut output = Vec::new();
    extend_checked(&mut output, TABLE_DOMAIN)?;
    append_frame(&mut output, table.as_bytes())?;
    append_sorted_record_macs(&mut output, records)?;
    Ok(output)
}

fn store_preimage(table_macs: &[C2Mac; 9]) -> Result<Vec<u8>, C2FramingError> {
    let mut output = Vec::new();
    extend_checked(&mut output, STORE_DOMAIN)?;
    for (table, mac) in C2TableTag::ALL.into_iter().zip(table_macs) {
        append_frame(&mut output, table.as_bytes())?;
        append_frame(&mut output, &mac.0)?;
    }
    Ok(output)
}

fn project_view_preimage(frame: &C2ProjectViewFrame<'_>) -> Result<Vec<u8>, C2FramingError> {
    let mut output = Vec::new();
    extend_checked(&mut output, PROJECT_VIEW_DOMAIN)?;
    append_record_mac(&mut output, frame.target_project)?;
    append_record_mac(&mut output, frame.target_repository)?;
    append_record_mac(&mut output, frame.target_checkout)?;
    match frame.target_task {
        None => push_checked(&mut output, 0x00)?,
        Some(task) => {
            push_checked(&mut output, 0x01)?;
            append_record_mac(&mut output, task)?;
        }
    }
    append_sorted_record_macs(&mut output, frame.competing_links)?;
    append_sorted_record_macs(&mut output, frame.linked_components)?;

    append_len(&mut output, frame.correction_edges.len())?;
    let mut edges = frame.correction_edges.to_vec();
    edges.sort_unstable_by_key(|edge| edge.proposal_id);
    for edge in edges {
        append_frame(&mut output, &edge.proposal_id)?;
        append_frame(&mut output, &edge.obsolete_id)?;
        append_frame(&mut output, &edge.replacement_id)?;
        append_frame(&mut output, edge.status.as_bytes())?;
        append_frame(&mut output, &edge.proposal_mac.0)?;
        append_frame(&mut output, &edge.obsolete_mac.0)?;
        append_frame(&mut output, &edge.replacement_mac.0)?;
    }

    append_sorted_record_macs(&mut output, frame.applicable_memories)?;
    Ok(output)
}

fn relay_candidates_preimage(records: &[C2RecordMac]) -> Result<Vec<u8>, C2FramingError> {
    let mut output = Vec::new();
    extend_checked(&mut output, RELAY_CANDIDATES_DOMAIN)?;
    append_sorted_record_macs(&mut output, records)?;
    Ok(output)
}

impl C2MacEngine {
    fn record_mac(
        &self,
        table: C2TableTag,
        id: [u8; 16],
        row: &Value,
    ) -> Result<C2Mac, C2FramingError> {
        Ok(self.authenticate(&record_preimage(table, id, row)?))
    }

    fn table_mac(
        &self,
        table: C2TableTag,
        records: &[C2RecordMac],
    ) -> Result<C2Mac, C2FramingError> {
        Ok(self.authenticate(&table_preimage(table, records)?))
    }

    fn store_mac(&self, table_macs: &[C2Mac; 9]) -> Result<C2Mac, C2FramingError> {
        Ok(self.authenticate(&store_preimage(table_macs)?))
    }

    fn project_view_mac(&self, frame: &C2ProjectViewFrame<'_>) -> Result<C2Mac, C2FramingError> {
        Ok(self.authenticate(&project_view_preimage(frame)?))
    }

    fn relay_candidates_mac(&self, records: &[C2RecordMac]) -> Result<C2Mac, C2FramingError> {
        Ok(self.authenticate(&relay_candidates_preimage(records)?))
    }
}

#[derive(Serialize)]
struct C2SanitizedAudit {
    schema_version: u64,
    target: C2AuditTarget,
    row_counts: C2RowCounts,
    memory_status_counts: C2MemoryStatusCounts,
    proposal_status_counts: C2ProposalStatusCounts,
    table_macs: C2TableMacs,
    store_state_mac: String,
    project_view_mac: String,
    relay_candidate_mac: String,
    correction_edges: Vec<C2AuditCorrectionEdge>,
    relay_candidates: Vec<C2AuditRelayCandidate>,
}

#[derive(Serialize)]
struct C2AuditTarget {
    project_id: String,
    repository_id: String,
    checkout_id: String,
    task_id: Option<String>,
}

#[derive(Serialize)]
struct C2RowCounts {
    memory_item: u64,
    correction_proposal: u64,
    memory_forget_receipt: u64,
    work_project: u64,
    work_task: u64,
    git_repository: u64,
    local_checkout: u64,
    monorepo_component: u64,
    project_repository_link: u64,
}

#[derive(Serialize)]
struct C2MemoryStatusCounts {
    active: u64,
    needs_review: u64,
    superseded: u64,
    archived: u64,
    rejected: u64,
}

#[derive(Serialize)]
struct C2ProposalStatusCounts {
    pending: u64,
    applied: u64,
}

#[derive(Serialize)]
struct C2TableMacs {
    memory_item: String,
    correction_proposal: String,
    memory_forget_receipt: String,
    work_project: String,
    work_task: String,
    git_repository: String,
    local_checkout: String,
    monorepo_component: String,
    project_repository_link: String,
}

#[derive(Serialize)]
struct C2AuditCorrectionEdge {
    proposal_id: String,
    obsolete_id: String,
    replacement_id: String,
    status: String,
    proposal_record_mac: String,
    obsolete_record_mac: String,
    replacement_record_mac: String,
}

#[derive(Serialize)]
struct C2AuditRelayCandidate {
    memory_id: String,
    status: String,
    record_mac: String,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
enum C2SemanticError {
    LimitExceeded { table_overflow_mask: Option<u16> },
    SecretMaterial,
    InvalidRecord,
    InconsistentProjection,
    IncompleteDeletion,
    AmbiguousIdentity,
    ScopeMismatch,
    AppliedProvenanceUnproven,
}

impl fmt::Display for C2SemanticError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::LimitExceeded { .. } => "limit_exceeded",
            Self::SecretMaterial => "secret_material",
            Self::InvalidRecord => "invalid_record",
            Self::InconsistentProjection => "inconsistent_projection",
            Self::IncompleteDeletion => "incomplete_deletion",
            Self::AmbiguousIdentity => "ambiguous_identity",
            Self::ScopeMismatch => "scope_mismatch",
            Self::AppliedProvenanceUnproven => "applied_provenance_unproven",
        })
    }
}

impl std::error::Error for C2SemanticError {}

type C2Result<T> = Result<T, C2SemanticError>;

impl From<C2FramingError> for C2SemanticError {
    fn from(_: C2FramingError) -> Self {
        Self::LimitExceeded {
            table_overflow_mask: None,
        }
    }
}

const MAX_RAW_NODES: u64 = 131_072;
const MAX_RAW_BYTES: u64 = 2_097_152;
const MAX_ROW_BYTES: u64 = 65_536;
const MAX_DEPTH: u64 = 16;
const MAX_OBJECT_KEYS: usize = 64;
const MAX_VECTOR_ELEMENTS: usize = 64;
const MAX_ORDINARY_SCALAR_BYTES: usize = 4_096;
const MAX_LARGE_SCALAR_BYTES: usize = 32_768;
const MAX_NAME_SCALAR_BYTES: usize = 256;
const MAX_BINDINGS: usize = 32;
const MAX_EVIDENCE_TAGS_SUPERSEDES: usize = 32;
const MAX_PROCEDURE_VECTOR: usize = 16;
const MAX_PREREQUISITE_KEY_PATH: usize = 16;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum TableKind {
    MemoryItem = 0,
    CorrectionProposal = 1,
    MemoryForgetReceipt = 2,
    WorkProject = 3,
    WorkTask = 4,
    GitRepository = 5,
    LocalCheckout = 6,
    MonorepoComponent = 7,
    ProjectRepositoryLink = 8,
}

const TABLE_KINDS: [TableKind; 9] = [
    TableKind::MemoryItem,
    TableKind::CorrectionProposal,
    TableKind::MemoryForgetReceipt,
    TableKind::WorkProject,
    TableKind::WorkTask,
    TableKind::GitRepository,
    TableKind::LocalCheckout,
    TableKind::MonorepoComponent,
    TableKind::ProjectRepositoryLink,
];

impl TableKind {
    const fn tag(self) -> &'static str {
        match self {
            Self::MemoryItem => "memory_item",
            Self::CorrectionProposal => "correction_proposal",
            Self::MemoryForgetReceipt => "memory_forget_receipt",
            Self::WorkProject => "work_project",
            Self::WorkTask => "work_task",
            Self::GitRepository => "git_repository",
            Self::LocalCheckout => "local_checkout",
            Self::MonorepoComponent => "monorepo_component",
            Self::ProjectRepositoryLink => "project_repository_link",
        }
    }

    const fn row_cap(self) -> usize {
        match self {
            Self::MemoryItem => 64,
            Self::CorrectionProposal | Self::MemoryForgetReceipt => 32,
            Self::WorkProject => 8,
            Self::WorkTask => 64,
            Self::GitRepository => 16,
            Self::LocalCheckout => 32,
            Self::MonorepoComponent | Self::ProjectRepositoryLink => 64,
        }
    }

    const fn overflow_bit(self) -> u16 {
        1_u16 << (self as u8)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct RawAccounting {
    nodes: u64,
    bytes: u64,
    maximum_depth: u64,
}

struct Meter {
    nodes: u64,
    bytes: u64,
    maximum_depth: u64,
}

impl Meter {
    const fn new() -> Self {
        Self {
            nodes: 0,
            bytes: 0,
            maximum_depth: 0,
        }
    }

    fn enter_node(&mut self, depth: u64) -> C2Result<()> {
        if depth > MAX_DEPTH {
            return limit_error();
        }
        self.nodes = self
            .nodes
            .checked_add(1)
            .ok_or(C2SemanticError::LimitExceeded {
                table_overflow_mask: None,
            })?;
        if self.nodes > MAX_RAW_NODES {
            return limit_error();
        }
        self.maximum_depth = self.maximum_depth.max(depth);
        Ok(())
    }

    fn add_bytes(&mut self, amount: u64) -> C2Result<()> {
        self.bytes = self
            .bytes
            .checked_add(amount)
            .ok_or(C2SemanticError::LimitExceeded {
                table_overflow_mask: None,
            })?;
        if self.bytes > MAX_RAW_BYTES {
            return limit_error();
        }
        Ok(())
    }

    fn finish(self) -> RawAccounting {
        RawAccounting {
            nodes: self.nodes,
            bytes: self.bytes,
            maximum_depth: self.maximum_depth,
        }
    }
}

fn limit_error<T>() -> C2Result<T> {
    Err(C2SemanticError::LimitExceeded {
        table_overflow_mask: None,
    })
}

fn usize_as_u64(value: usize) -> C2Result<u64> {
    u64::try_from(value).map_err(|_| C2SemanticError::LimitExceeded {
        table_overflow_mask: None,
    })
}

fn table_rows(raw: &C2RawSnapshot, table: TableKind) -> &[Value] {
    match table {
        TableKind::MemoryItem => &raw.memory_item,
        TableKind::CorrectionProposal => &raw.correction_proposal,
        TableKind::MemoryForgetReceipt => &raw.memory_forget_receipt,
        TableKind::WorkProject => &raw.work_project,
        TableKind::WorkTask => &raw.work_task,
        TableKind::GitRepository => &raw.git_repository,
        TableKind::LocalCheckout => &raw.local_checkout,
        TableKind::MonorepoComponent => &raw.monorepo_component,
        TableKind::ProjectRepositoryLink => &raw.project_repository_link,
    }
}

fn precheck_table_counts(raw: &C2RawSnapshot) -> C2Result<()> {
    let mut mask = 0_u16;
    for table in TABLE_KINDS {
        if table_rows(raw, table).len() > table.row_cap() {
            mask |= table.overflow_bit();
        }
    }
    if mask == 0 {
        Ok(())
    } else {
        Err(C2SemanticError::LimitExceeded {
            table_overflow_mask: Some(mask),
        })
    }
}

#[derive(Clone, Copy)]
enum SchemaPath {
    Unknown,
    OrdinaryScalar,
    NameScalar,
    LargeScalar,
    EnumScalar,
    MemoryRow,
    MemoryItem,
    Scope,
    Writer,
    Model,
    EvidenceVector,
    Evidence,
    SupersedesVector,
    TagsVector,
    Archive,
    Procedure,
    ProcedureCommands,
    ProcedurePrerequisites,
    ProcedurePrerequisite,
    ProcedurePrerequisiteSource,
    PrerequisiteKeyPath,
    ProcedureFailureSignatures,
    ProcedureVerification,
    ProposalRow,
    Proposal,
    ForgetReceiptRow,
    ProjectRow,
    TaskRow,
    TaskBlockedBy,
    RepositoryRow,
    Repository,
    CheckoutRow,
    Checkout,
    ComponentRow,
    Component,
    LinkRow,
    Link,
}

impl SchemaPath {
    const fn scalar_cap(self) -> usize {
        match self {
            Self::LargeScalar => MAX_LARGE_SCALAR_BYTES,
            Self::NameScalar | Self::EnumScalar => MAX_NAME_SCALAR_BYTES,
            _ => MAX_ORDINARY_SCALAR_BYTES,
        }
    }

    const fn vector_cap(self) -> usize {
        match self {
            Self::EvidenceVector | Self::SupersedesVector | Self::TagsVector => {
                MAX_EVIDENCE_TAGS_SUPERSEDES
            }
            Self::ProcedureCommands
            | Self::ProcedurePrerequisites
            | Self::ProcedureFailureSignatures => MAX_PROCEDURE_VECTOR,
            Self::PrerequisiteKeyPath => MAX_PREREQUISITE_KEY_PATH,
            _ => MAX_VECTOR_ELEMENTS,
        }
    }

    const fn element(self) -> Self {
        match self {
            Self::EvidenceVector => Self::Evidence,
            Self::SupersedesVector | Self::TaskBlockedBy => Self::OrdinaryScalar,
            Self::TagsVector | Self::PrerequisiteKeyPath => Self::NameScalar,
            Self::ProcedureCommands => Self::LargeScalar,
            Self::ProcedurePrerequisites => Self::ProcedurePrerequisite,
            Self::ProcedureFailureSignatures => Self::OrdinaryScalar,
            _ => Self::Unknown,
        }
    }

    fn child(self, key: &str) -> Self {
        use SchemaPath as S;
        match self {
            S::MemoryRow => match key {
                "item" => S::MemoryItem,
                "kind_key" | "status_key" | "harness_key" => S::EnumScalar,
                "model_key" => S::NameScalar,
                "record_id" | "scope_key" | "session_id" | "snapshot_digest" | "created_at"
                | "updated_at" => S::OrdinaryScalar,
                _ => S::Unknown,
            },
            S::MemoryItem => match key {
                "kind" | "origin" | "status" => S::EnumScalar,
                "title" => S::NameScalar,
                "content" => S::LargeScalar,
                "scope" => S::Scope,
                "writer" => S::Writer,
                "evidence" => S::EvidenceVector,
                "supersedes" => S::SupersedesVector,
                "tags" => S::TagsVector,
                "archive" => S::Archive,
                "procedure" => S::Procedure,
                "id"
                | "confidence"
                | "created_at"
                | "updated_at"
                | "last_used_at"
                | "review_after"
                | "correction_proposal_id"
                | "pending_correction_proposal_id" => S::OrdinaryScalar,
                _ => S::Unknown,
            },
            S::Scope => match key {
                "type" => S::EnumScalar,
                "project_name" | "task_name" | "entity_name" | "name" => S::NameScalar,
                "project_id" | "task_id" | "entity_id" | "repository_id" | "remote_url"
                | "local_path" | "session_id" => S::OrdinaryScalar,
                _ => S::Unknown,
            },
            S::Writer => match key {
                "harness" => S::EnumScalar,
                "harness_version" | "surface" | "actor" => S::NameScalar,
                "model" => S::Model,
                "session_id" | "written_at" => S::OrdinaryScalar,
                _ => S::Unknown,
            },
            S::Model => match key {
                "provider" | "model" | "version" => S::NameScalar,
                _ => S::Unknown,
            },
            S::Evidence => match key {
                "kind" => S::EnumScalar,
                "excerpt" => S::LargeScalar,
                "target" | "summary" | "observed_at" => S::OrdinaryScalar,
                _ => S::Unknown,
            },
            S::Archive => match key {
                "archived_by" => S::NameScalar,
                "reason" | "archived_at" => S::OrdinaryScalar,
                _ => S::Unknown,
            },
            S::Procedure => match key {
                "prerequisites" => S::ProcedurePrerequisites,
                "commands" => S::ProcedureCommands,
                "failure_signatures" => S::ProcedureFailureSignatures,
                "verification" => S::ProcedureVerification,
                "task" | "expires_at" => S::OrdinaryScalar,
                _ => S::Unknown,
            },
            S::ProcedurePrerequisite => match key {
                "key" => S::NameScalar,
                "expected" => S::OrdinaryScalar,
                "source" => S::ProcedurePrerequisiteSource,
                _ => S::Unknown,
            },
            S::ProcedurePrerequisiteSource => match key {
                "format" => S::EnumScalar,
                "key_path" => S::PrerequisiteKeyPath,
                "relative_path" => S::OrdinaryScalar,
                _ => S::Unknown,
            },
            S::ProcedureVerification => match key {
                "command"
                | "expected_exit_code"
                | "expected_output_contains"
                | "evidence_path"
                | "evidence_sha256"
                | "verified_at" => S::OrdinaryScalar,
                _ => S::Unknown,
            },
            S::ProposalRow => match key {
                "proposal" => S::Proposal,
                "status_key" => S::EnumScalar,
                "record_id"
                | "obsolete_id"
                | "pending_obsolete_id"
                | "replacement_id"
                | "snapshot_digest"
                | "created_at" => S::OrdinaryScalar,
                _ => S::Unknown,
            },
            S::Proposal => match key {
                "memory_kind" | "status" => S::EnumScalar,
                "scope" => S::Scope,
                "proposer" => S::Writer,
                "id"
                | "obsolete_id"
                | "replacement_id"
                | "canonical_digest"
                | "digest_schema_version"
                | "applied_digest"
                | "created_at"
                | "applied_at" => S::OrdinaryScalar,
                _ => S::Unknown,
            },
            S::ForgetReceiptRow => match key {
                "deleted"
                | "proposal_ids"
                | "pending_replacement_ids"
                | "unlocked_obsolete_ids" => S::TaskBlockedBy,
                "record_id" | "cleanup_complete" | "created_at" | "completed_at" => {
                    S::OrdinaryScalar
                }
                _ => S::Unknown,
            },
            S::ProjectRow => match key {
                "name" => S::NameScalar,
                "status" => S::EnumScalar,
                "record_id" | "description" | "created_at" | "updated_at" => S::OrdinaryScalar,
                _ => S::Unknown,
            },
            S::TaskRow => match key {
                "name" | "jira_key" => S::NameScalar,
                "status" | "priority" => S::EnumScalar,
                "blocked_by" => S::TaskBlockedBy,
                "record_id" | "project_id" | "description" | "created_at" | "updated_at" => {
                    S::OrdinaryScalar
                }
                _ => S::Unknown,
            },
            S::RepositoryRow => match key {
                "repository" => S::Repository,
                "name_key" => S::NameScalar,
                "provider_key" => S::EnumScalar,
                "record_id" | "remote_url" | "created_at" | "updated_at" => S::OrdinaryScalar,
                _ => S::Unknown,
            },
            S::Repository => match key {
                "name" | "default_branch" => S::NameScalar,
                "provider" => S::EnumScalar,
                "id" | "remote_url" | "description" | "created_at" | "updated_at" => {
                    S::OrdinaryScalar
                }
                _ => S::Unknown,
            },
            S::CheckoutRow => match key {
                "checkout" => S::Checkout,
                "current_branch" => S::NameScalar,
                "record_id" | "repository_id" | "local_path_key" | "head_sha" | "is_dirty"
                | "created_at" | "updated_at" | "last_seen_at" => S::OrdinaryScalar,
                _ => S::Unknown,
            },
            S::Checkout => match key {
                "current_branch" => S::NameScalar,
                "id" | "repository_id" | "local_path" | "head_sha" | "is_dirty" | "created_at"
                | "updated_at" | "last_seen_at" => S::OrdinaryScalar,
                _ => S::Unknown,
            },
            S::ComponentRow => match key {
                "component" => S::Component,
                "name_key" | "kind" => S::NameScalar,
                "record_id" | "repository_id" | "path_key" | "created_at" | "updated_at" => {
                    S::OrdinaryScalar
                }
                _ => S::Unknown,
            },
            S::Component => match key {
                "name" | "kind" => S::NameScalar,
                "id" | "repository_id" | "path" | "description" | "source_path"
                | "source_sha256" | "created_at" | "updated_at" => S::OrdinaryScalar,
                _ => S::Unknown,
            },
            S::LinkRow => match key {
                "link" => S::Link,
                "project_name_key" => S::NameScalar,
                "role" => S::EnumScalar,
                "record_id" | "project_id" | "repository_id" | "component_id"
                | "component_path_key" | "created_at" | "updated_at" => S::OrdinaryScalar,
                _ => S::Unknown,
            },
            S::Link => match key {
                "project_name" => S::NameScalar,
                "role" => S::EnumScalar,
                "id" | "project_id" | "repository_id" | "component_id" | "component_path"
                | "created_at" | "updated_at" => S::OrdinaryScalar,
                _ => S::Unknown,
            },
            S::EnumScalar if matches!(key, "custom" | "other") => S::NameScalar,
            _ => S::Unknown,
        }
    }
}

fn row_schema(table: TableKind) -> SchemaPath {
    match table {
        TableKind::MemoryItem => SchemaPath::MemoryRow,
        TableKind::CorrectionProposal => SchemaPath::ProposalRow,
        TableKind::MemoryForgetReceipt => SchemaPath::ForgetReceiptRow,
        TableKind::WorkProject => SchemaPath::ProjectRow,
        TableKind::WorkTask => SchemaPath::TaskRow,
        TableKind::GitRepository => SchemaPath::RepositoryRow,
        TableKind::LocalCheckout => SchemaPath::CheckoutRow,
        TableKind::MonorepoComponent => SchemaPath::ComponentRow,
        TableKind::ProjectRepositoryLink => SchemaPath::LinkRow,
    }
}

struct RowBytes(u64);

impl RowBytes {
    const fn new() -> Self {
        Self(0)
    }

    fn add(&mut self, amount: u64) -> C2Result<()> {
        self.0 = self
            .0
            .checked_add(amount)
            .ok_or(C2SemanticError::LimitExceeded {
                table_overflow_mask: None,
            })?;
        if self.0 > MAX_ROW_BYTES {
            return limit_error();
        }
        Ok(())
    }
}

fn add_framed_bytes(meter: &mut Meter, row: &mut Option<RowBytes>, amount: u64) -> C2Result<()> {
    meter.add_bytes(amount)?;
    if let Some(row) = row {
        row.add(amount)?;
    }
    Ok(())
}

fn add_len(base: u64, length: usize) -> C2Result<u64> {
    base.checked_add(usize_as_u64(length)?)
        .ok_or(C2SemanticError::LimitExceeded {
            table_overflow_mask: None,
        })
}

fn child_depth(depth: u64) -> C2Result<u64> {
    depth.checked_add(1).ok_or(C2SemanticError::LimitExceeded {
        table_overflow_mask: None,
    })
}

fn walk_value(
    value: &Value,
    depth: u64,
    schema: SchemaPath,
    meter: &mut Meter,
    row: &mut Option<RowBytes>,
) -> C2Result<()> {
    meter.enter_node(depth)?;
    match value {
        Value::Null | Value::Bool(_) => add_framed_bytes(meter, row, 1),
        Value::Number(number) => {
            let spelling = number.to_string();
            add_framed_bytes(meter, row, add_len(1 + 8, spelling.len())?)
        }
        Value::String(string) => {
            if string.len() > schema.scalar_cap() {
                return limit_error();
            }
            add_framed_bytes(meter, row, add_len(1 + 8, string.len())?)
        }
        Value::Array(values) => {
            if values.len() > schema.vector_cap() {
                return limit_error();
            }
            let _ = usize_as_u64(values.len())?;
            add_framed_bytes(meter, row, 1 + 8)?;
            let depth = child_depth(depth)?;
            for value in values {
                walk_value(value, depth, schema.element(), meter, row)?;
            }
            Ok(())
        }
        Value::Object(values) => {
            if values.len() > MAX_OBJECT_KEYS {
                return limit_error();
            }
            let _ = usize_as_u64(values.len())?;
            add_framed_bytes(meter, row, 1 + 8)?;
            let depth = child_depth(depth)?;
            let mut members: Vec<_> = values.iter().collect();
            members.sort_unstable_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
            for (key, value) in members {
                meter.enter_node(depth)?;
                if key.len() > MAX_ORDINARY_SCALAR_BYTES {
                    return limit_error();
                }
                add_framed_bytes(meter, row, add_len(8, key.len())?)?;
                walk_value(value, depth, schema.child(key), meter, row)?;
            }
            Ok(())
        }
    }
}

fn account_key(meter: &mut Meter, depth: u64, key: &str) -> C2Result<()> {
    meter.enter_node(depth)?;
    if key.len() > MAX_ORDINARY_SCALAR_BYTES {
        return limit_error();
    }
    meter.add_bytes(add_len(8, key.len())?)
}

fn account_string(meter: &mut Meter, depth: u64, string: &str, cap: usize) -> C2Result<()> {
    meter.enter_node(depth)?;
    if string.len() > cap {
        return limit_error();
    }
    meter.add_bytes(add_len(1 + 8, string.len())?)
}

fn account_optional_string(
    meter: &mut Meter,
    depth: u64,
    string: Option<&str>,
    cap: usize,
) -> C2Result<()> {
    match string {
        Some(string) => account_string(meter, depth, string, cap),
        None => {
            meter.enter_node(depth)?;
            meter.add_bytes(1)
        }
    }
}

fn account_object_header(meter: &mut Meter, depth: u64, count: usize) -> C2Result<()> {
    if count > MAX_OBJECT_KEYS {
        return limit_error();
    }
    let _ = usize_as_u64(count)?;
    meter.enter_node(depth)?;
    meter.add_bytes(1 + 8)
}

fn account_array_header(meter: &mut Meter, depth: u64, count: usize, cap: usize) -> C2Result<()> {
    if count > cap {
        return limit_error();
    }
    let _ = usize_as_u64(count)?;
    meter.enter_node(depth)?;
    meter.add_bytes(1 + 8)
}

fn account_target(target: &C2RawTarget, meter: &mut Meter) -> C2Result<()> {
    const FIELDS: [&str; 8] = [
        "project_id",
        "project_name",
        "repository_id",
        "repository_remote",
        "checkout_id",
        "checkout_path",
        "task_id",
        "task_name",
    ];
    account_object_header(meter, 1, FIELDS.len())?;
    for key in FIELDS {
        account_key(meter, 2, key)?;
        match key {
            "project_id" => {
                account_string(meter, 2, &target.project_id, MAX_ORDINARY_SCALAR_BYTES)?
            }
            "project_name" => {
                account_string(meter, 2, &target.project_name, MAX_NAME_SCALAR_BYTES)?
            }
            "repository_id" => {
                account_string(meter, 2, &target.repository_id, MAX_ORDINARY_SCALAR_BYTES)?
            }
            "repository_remote" => account_string(
                meter,
                2,
                &target.repository_remote,
                MAX_ORDINARY_SCALAR_BYTES,
            )?,
            "checkout_id" => {
                account_string(meter, 2, &target.checkout_id, MAX_ORDINARY_SCALAR_BYTES)?
            }
            "checkout_path" => {
                account_string(meter, 2, &target.checkout_path, MAX_ORDINARY_SCALAR_BYTES)?
            }
            "task_id" => account_optional_string(
                meter,
                2,
                target.task_id.as_deref(),
                MAX_ORDINARY_SCALAR_BYTES,
            )?,
            "task_name" => account_optional_string(
                meter,
                2,
                target.task_name.as_deref(),
                MAX_NAME_SCALAR_BYTES,
            )?,
            _ => unreachable!("fixed target field"),
        }
    }
    Ok(())
}

fn account_row(
    row_value: &Value,
    depth: u64,
    schema: SchemaPath,
    meter: &mut Meter,
) -> C2Result<()> {
    let mut row = Some(RowBytes::new());
    walk_value(row_value, depth, schema, meter, &mut row)
}

fn account_binding(binding: &C2PriorCorrectionBinding, meter: &mut Meter) -> C2Result<()> {
    account_object_header(meter, 2, 4)?;
    account_key(meter, 3, "proposal_id")?;
    account_string(
        meter,
        3,
        &binding.proposal_id.to_string(),
        MAX_ORDINARY_SCALAR_BYTES,
    )?;
    account_key(meter, 3, "proposal_row")?;
    account_row(&binding.proposal_row, 3, SchemaPath::ProposalRow, meter)?;
    account_key(meter, 3, "obsolete_row")?;
    account_row(&binding.obsolete_row, 3, SchemaPath::MemoryRow, meter)?;
    account_key(meter, 3, "replacement_row")?;
    account_row(&binding.replacement_row, 3, SchemaPath::MemoryRow, meter)
}

fn account_virtual_input(
    raw: &C2RawSnapshot,
    prior_bindings: &[C2PriorCorrectionBinding],
) -> C2Result<RawAccounting> {
    precheck_table_counts(raw)?;
    if prior_bindings.len() > MAX_BINDINGS {
        return limit_error();
    }

    let mut meter = Meter::new();
    account_object_header(&mut meter, 0, 11)?;

    account_key(&mut meter, 1, "target")?;
    account_target(&raw.target, &mut meter)?;

    account_key(&mut meter, 1, "prior_correction_bindings")?;
    account_array_header(&mut meter, 1, prior_bindings.len(), MAX_BINDINGS)?;
    for binding in prior_bindings {
        account_binding(binding, &mut meter)?;
    }

    for table in TABLE_KINDS {
        account_key(&mut meter, 1, table.tag())?;
        let rows = table_rows(raw, table);
        account_array_header(&mut meter, 1, rows.len(), table.row_cap())?;
        for row in rows {
            account_row(row, 2, row_schema(table), &mut meter)?;
        }
    }

    Ok(meter.finish())
}

const SECRET_ASSIGNMENT_NAMES: [&str; 8] = [
    "API_KEY",
    "API_TOKEN",
    "ACCESS_TOKEN",
    "AUTH_TOKEN",
    "PASSWORD",
    "PASSWD",
    "CLIENT_SECRET",
    "PRIVATE_KEY",
];

fn likely_secret_in_string(value: &str) -> bool {
    let upper = value.to_ascii_uppercase();
    if upper.contains("-----BEGIN PRIVATE KEY-----")
        || upper.contains("-----BEGIN RSA PRIVATE KEY-----")
        || upper.contains("-----BEGIN OPENSSH PRIVATE KEY-----")
        || upper.contains("AUTHORIZATION: BEARER ")
    {
        return true;
    }
    if contains_credential_url(value) || contains_secret_assignment(value) {
        return true;
    }

    value
        .split(|character: char| {
            character.is_whitespace()
                || matches!(
                    character,
                    '\'' | '"' | '`' | ',' | ';' | '(' | ')' | '[' | ']'
                )
        })
        .map(|token| token.trim_matches(|character: char| matches!(character, ':' | '=')))
        .any(|token| {
            (token.starts_with("AKIA")
                && token.len() == 20
                && token
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric()))
                || (token.starts_with("github_pat_") && token.len() >= 30)
                || ["ghp_", "gho_", "ghu_", "ghs_", "ghr_"]
                    .iter()
                    .any(|prefix| token.starts_with(prefix) && token.len() >= 20)
                || ["xoxb-", "xoxp-", "xoxa-", "xoxr-", "xoxs-"]
                    .iter()
                    .any(|prefix| token.starts_with(prefix) && token.len() >= 20)
                || (token.starts_with("sk-") && token.len() >= 24)
                || looks_like_jwt(token)
        })
}

fn contains_credential_url(value: &str) -> bool {
    value.split_whitespace().any(|token| {
        let Some((_, authority_and_path)) = token.split_once("://") else {
            return false;
        };
        let authority = authority_and_path.split('/').next().unwrap_or_default();
        authority
            .split_once('@')
            .is_some_and(|(user_info, _)| user_info.contains(':'))
    })
}

fn assignment_candidate_contains_secret(candidate: &str) -> bool {
    let candidate = candidate.trim_matches(|character: char| {
        matches!(
            character,
            '\'' | '"' | '`' | ',' | ';' | '(' | ')' | '[' | ']'
        )
    });
    let candidate = candidate.strip_prefix("export").unwrap_or(candidate).trim();
    let Some((name, assigned)) = candidate.split_once('=') else {
        return false;
    };
    let name = name.trim().to_ascii_uppercase();
    let assigned = assigned.trim().trim_matches(['\'', '"', '`']);
    SECRET_ASSIGNMENT_NAMES.contains(&name.as_str()) && assigned.len() >= 8
}

fn contains_secret_assignment(value: &str) -> bool {
    value.lines().any(|line| {
        assignment_candidate_contains_secret(line)
            || line
                .split_whitespace()
                .any(assignment_candidate_contains_secret)
    })
}

fn looks_like_jwt(token: &str) -> bool {
    let mut segments = token.split('.');
    let Some(first) = segments.next() else {
        return false;
    };
    let Some(second) = segments.next() else {
        return false;
    };
    let Some(third) = segments.next() else {
        return false;
    };
    segments.next().is_none()
        && first.starts_with("eyJ")
        && [first, second, third].iter().all(|segment| {
            segment.len() >= 8
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'='))
        })
}

fn is_credential_field(name: &str) -> bool {
    let normalized = name.trim().replace(['-', ' '], "_").to_ascii_uppercase();
    matches!(
        normalized.as_str(),
        "API_KEY"
            | "APIKEY"
            | "API_TOKEN"
            | "ACCESS_TOKEN"
            | "AUTH_TOKEN"
            | "PASSWORD"
            | "PASSWD"
            | "CLIENT_SECRET"
            | "PRIVATE_KEY"
            | "AUTHORIZATION"
    )
}

fn is_secret_field_string(value: &str) -> bool {
    let trimmed = value.trim();
    let normalized = trimmed.to_ascii_lowercase();
    trimmed.len() >= 8
        && !matches!(
            normalized.as_str(),
            "[redacted]"
                | "changeme"
                | "example"
                | "not-set"
                | "placeholder"
                | "redacted"
                | "your_token_here"
        )
}

fn secret_in_value_inner(value: &Value, under_credential_field: bool) -> bool {
    match value {
        Value::String(value) => {
            likely_secret_in_string(value)
                || (under_credential_field && is_secret_field_string(value))
        }
        Value::Array(values) => values
            .iter()
            .any(|value| secret_in_value_inner(value, under_credential_field)),
        Value::Object(values) => {
            let mut members: Vec<_> = values.iter().collect();
            members.sort_unstable_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
            members.into_iter().any(|(key, value)| {
                likely_secret_in_string(key)
                    || secret_in_value_inner(
                        value,
                        under_credential_field || is_credential_field(key),
                    )
            })
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => false,
    }
}

fn secret_in_value(value: &Value) -> bool {
    secret_in_value_inner(value, false)
}

fn secret_in_optional_string(value: Option<&str>) -> bool {
    value.is_some_and(likely_secret_in_string)
}

fn secret_in_virtual_input(
    raw: &C2RawSnapshot,
    prior_bindings: &[C2PriorCorrectionBinding],
) -> bool {
    let target = &raw.target;
    if [
        target.project_id.as_str(),
        target.project_name.as_str(),
        target.repository_id.as_str(),
        target.repository_remote.as_str(),
        target.checkout_id.as_str(),
        target.checkout_path.as_str(),
    ]
    .into_iter()
    .any(likely_secret_in_string)
        || secret_in_optional_string(target.task_id.as_deref())
        || secret_in_optional_string(target.task_name.as_deref())
    {
        return true;
    }

    for binding in prior_bindings {
        if likely_secret_in_string(&binding.proposal_id.to_string())
            || secret_in_value(&binding.proposal_row)
            || secret_in_value(&binding.obsolete_row)
            || secret_in_value(&binding.replacement_row)
        {
            return true;
        }
    }

    TABLE_KINDS
        .into_iter()
        .any(|table| table_rows(raw, table).iter().any(secret_in_value))
}

struct C2BoundaryPass {
    accounting: RawAccounting,
    forget_receipts_present: bool,
}

fn validate_limit_and_secret_boundary(
    raw: &C2RawSnapshot,
    prior_bindings: &[C2PriorCorrectionBinding],
) -> C2Result<C2BoundaryPass> {
    let accounting = account_virtual_input(raw, prior_bindings)?;
    if secret_in_virtual_input(raw, prior_bindings) {
        return Err(C2SemanticError::SecretMaterial);
    }
    Ok(C2BoundaryPass {
        accounting,
        forget_receipts_present: !raw.memory_forget_receipt.is_empty(),
    })
}

fn enforce_receipt_precedence_after_strict_records(
    boundary: &C2BoundaryPass,
    strict_records: C2Result<()>,
) -> C2Result<()> {
    strict_records?;
    if boundary.forget_receipts_present {
        Err(C2SemanticError::IncompleteDeletion)
    } else {
        Ok(())
    }
}

fn exact_object<'a>(
    value: &'a Value,
    required: &[&str],
    optional: &[&str],
) -> C2Result<&'a Map<String, Value>> {
    let object = value.as_object().ok_or(C2SemanticError::InvalidRecord)?;
    if required.iter().any(|key| !object.contains_key(*key))
        || object
            .keys()
            .any(|key| !required.contains(&key.as_str()) && !optional.contains(&key.as_str()))
    {
        return Err(C2SemanticError::InvalidRecord);
    }
    Ok(object)
}

fn exact_keys<'a>(value: &'a Value, required: &[&str]) -> C2Result<&'a Map<String, Value>> {
    let object = exact_object(value, required, &[])?;
    if object.len() != required.len() {
        return Err(C2SemanticError::InvalidRecord);
    }
    Ok(object)
}

fn field<'a>(object: &'a Map<String, Value>, key: &str) -> C2Result<&'a Value> {
    object.get(key).ok_or(C2SemanticError::InvalidRecord)
}

fn string_field<'a>(object: &'a Map<String, Value>, key: &str) -> C2Result<&'a str> {
    field(object, key)?
        .as_str()
        .ok_or(C2SemanticError::InvalidRecord)
}

fn optional_string_field<'a>(
    object: &'a Map<String, Value>,
    key: &str,
) -> C2Result<Option<&'a str>> {
    match field(object, key)? {
        Value::Null => Ok(None),
        Value::String(value) => Ok(Some(value)),
        _ => Err(C2SemanticError::InvalidRecord),
    }
}

fn canonical_timestamp(value: &Value) -> C2Result<OffsetDateTime> {
    let text = value.as_str().ok_or(C2SemanticError::InvalidRecord)?;
    let parsed = OffsetDateTime::parse(text, &time::format_description::well_known::Rfc3339)
        .map_err(|_| C2SemanticError::InvalidRecord)?;
    let canonical = parsed
        .to_offset(UtcOffset::UTC)
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|_| C2SemanticError::InvalidRecord)?;
    if canonical != text {
        return Err(C2SemanticError::InvalidRecord);
    }
    Ok(parsed)
}

fn canonical_optional_timestamp(value: &Value) -> C2Result<Option<OffsetDateTime>> {
    if value.is_null() {
        Ok(None)
    } else {
        canonical_timestamp(value).map(Some)
    }
}

fn validate_timestamp_fields(object: &Map<String, Value>, keys: &[&str]) -> C2Result<()> {
    for key in keys {
        canonical_timestamp(field(object, key)?)?;
    }
    Ok(())
}

fn validate_optional_timestamp_fields(object: &Map<String, Value>, keys: &[&str]) -> C2Result<()> {
    for key in keys {
        canonical_optional_timestamp(field(object, key)?)?;
    }
    Ok(())
}

fn validate_unit_or_data_enum(
    value: &Value,
    unit_variants: &[&str],
    data_variant: Option<&str>,
) -> C2Result<()> {
    match value {
        Value::String(value) if unit_variants.contains(&value.as_str()) => Ok(()),
        Value::Object(object) => {
            let Some(data_variant) = data_variant else {
                return Err(C2SemanticError::InvalidRecord);
            };
            if object.len() != 1 {
                return Err(C2SemanticError::InvalidRecord);
            }
            let payload = object
                .get(data_variant)
                .and_then(Value::as_str)
                .ok_or(C2SemanticError::InvalidRecord)?;
            validate_identity_text(payload, true)
        }
        _ => Err(C2SemanticError::InvalidRecord),
    }
}

fn validate_identity_text(value: &str, require_nonempty: bool) -> C2Result<()> {
    if (require_nonempty && value.trim().is_empty())
        || value.nfc().ne(value.chars())
        || value.bytes().any(|byte| byte.is_ascii_control())
    {
        return Err(C2SemanticError::InvalidRecord);
    }
    Ok(())
}

fn validate_prose(value: &str, require_nonempty: bool) -> C2Result<()> {
    if (require_nonempty && value.trim().is_empty())
        || value
            .bytes()
            .any(|byte| byte == 0 || (byte.is_ascii_control() && byte != b'\n' && byte != b'\t'))
    {
        return Err(C2SemanticError::InvalidRecord);
    }
    Ok(())
}

fn validate_digest(value: &str) -> C2Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(C2SemanticError::InvalidRecord);
    }
    Ok(())
}

fn validate_id(id: &Id) -> C2Result<()> {
    if id.as_uuid().get_version_num() != 7 || id.as_uuid().as_bytes()[8] & 0xc0 != 0x80 {
        return Err(C2SemanticError::InvalidRecord);
    }
    Ok(())
}

fn parse_id(value: &str) -> C2Result<Id> {
    let id = Id::parse(value).map_err(|_| C2SemanticError::InvalidRecord)?;
    if id.to_string() != value {
        return Err(C2SemanticError::InvalidRecord);
    }
    validate_id(&id)?;
    Ok(id)
}

fn validate_memory_scope_schema(value: &Value) -> C2Result<()> {
    let object = value.as_object().ok_or(C2SemanticError::InvalidRecord)?;
    match string_field(object, "type")? {
        "global" | "user" => {
            exact_keys(value, &["type"])?;
        }
        "project" => {
            exact_keys(value, &["type", "project_id", "project_name"])?;
            optional_string_field(object, "project_id")?;
            string_field(object, "project_name")?;
        }
        "task" => {
            exact_keys(
                value,
                &["type", "project_id", "project_name", "task_id", "task_name"],
            )?;
            optional_string_field(object, "project_id")?;
            optional_string_field(object, "project_name")?;
            optional_string_field(object, "task_id")?;
            string_field(object, "task_name")?;
        }
        "entity" => {
            exact_keys(value, &["type", "entity_id", "entity_name"])?;
            optional_string_field(object, "entity_id")?;
            string_field(object, "entity_name")?;
        }
        "repository" => {
            exact_keys(
                value,
                &["type", "repository_id", "remote_url", "local_path"],
            )?;
            optional_string_field(object, "repository_id")?;
            optional_string_field(object, "remote_url")?;
            optional_string_field(object, "local_path")?;
        }
        "session" => {
            exact_keys(value, &["type", "session_id"])?;
            string_field(object, "session_id")?;
        }
        "custom" => {
            exact_keys(value, &["type", "name"])?;
            string_field(object, "name")?;
        }
        _ => return Err(C2SemanticError::InvalidRecord),
    }
    Ok(())
}

fn validate_model_schema(value: &Value) -> C2Result<()> {
    let object = exact_keys(value, &["provider", "model", "version"])?;
    string_field(object, "provider")?;
    string_field(object, "model")?;
    optional_string_field(object, "version")?;
    Ok(())
}

fn validate_writer_schema(value: &Value) -> C2Result<()> {
    let object = exact_keys(
        value,
        &[
            "harness",
            "harness_version",
            "model",
            "surface",
            "actor",
            "session_id",
            "written_at",
        ],
    )?;
    validate_unit_or_data_enum(
        field(object, "harness")?,
        &["claude_code", "codex", "chat_gpt", "cursor"],
        Some("other"),
    )?;
    optional_string_field(object, "harness_version")?;
    validate_model_schema(field(object, "model")?)?;
    optional_string_field(object, "surface")?;
    string_field(object, "actor")?;
    optional_string_field(object, "session_id")?;
    canonical_timestamp(field(object, "written_at")?)?;
    Ok(())
}

fn validate_evidence_schema(value: &Value) -> C2Result<()> {
    let object = exact_keys(
        value,
        &["kind", "target", "summary", "excerpt", "observed_at"],
    )?;
    validate_unit_or_data_enum(
        field(object, "kind")?,
        &[
            "session_event",
            "tool_call",
            "file",
            "git_commit",
            "url",
            "document",
            "observation",
            "manual_review",
        ],
        Some("custom"),
    )?;
    string_field(object, "target")?;
    optional_string_field(object, "summary")?;
    optional_string_field(object, "excerpt")?;
    canonical_timestamp(field(object, "observed_at")?)?;
    Ok(())
}

fn validate_archive_schema(value: &Value) -> C2Result<()> {
    let object = exact_keys(value, &["reason", "archived_by", "archived_at"])?;
    string_field(object, "reason")?;
    optional_string_field(object, "archived_by")?;
    canonical_timestamp(field(object, "archived_at")?)?;
    Ok(())
}

fn validate_prerequisite_source_schema(value: &Value) -> C2Result<()> {
    let object = exact_keys(value, &["format", "relative_path", "key_path"])?;
    if string_field(object, "format")? != "toml" {
        return Err(C2SemanticError::InvalidRecord);
    }
    string_field(object, "relative_path")?;
    let key_path = field(object, "key_path")?
        .as_array()
        .ok_or(C2SemanticError::InvalidRecord)?;
    if key_path.iter().any(|key| key.as_str().is_none()) {
        return Err(C2SemanticError::InvalidRecord);
    }
    Ok(())
}

fn validate_prerequisite_schema(value: &Value) -> C2Result<()> {
    let object = exact_object(value, &["key", "expected"], &["source"])?;
    string_field(object, "key")?;
    string_field(object, "expected")?;
    if let Some(source) = object.get("source") {
        if source.is_null() {
            return Err(C2SemanticError::InvalidRecord);
        }
        validate_prerequisite_source_schema(source)?;
    }
    Ok(())
}

fn validate_verification_schema(value: &Value) -> C2Result<()> {
    let object = exact_keys(
        value,
        &[
            "command",
            "expected_exit_code",
            "expected_output_contains",
            "evidence_path",
            "evidence_sha256",
            "verified_at",
        ],
    )?;
    string_field(object, "command")?;
    if field(object, "expected_exit_code")?.as_i64().is_none() {
        return Err(C2SemanticError::InvalidRecord);
    }
    string_field(object, "expected_output_contains")?;
    optional_string_field(object, "evidence_path")?;
    if let Some(digest) = optional_string_field(object, "evidence_sha256")? {
        validate_digest(digest)?;
    }
    canonical_optional_timestamp(field(object, "verified_at")?)?;
    Ok(())
}

fn validate_procedure_schema(value: &Value) -> C2Result<()> {
    let object = exact_keys(
        value,
        &[
            "task",
            "prerequisites",
            "commands",
            "failure_signatures",
            "verification",
            "expires_at",
        ],
    )?;
    string_field(object, "task")?;
    let prerequisites = field(object, "prerequisites")?
        .as_array()
        .ok_or(C2SemanticError::InvalidRecord)?;
    for prerequisite in prerequisites {
        validate_prerequisite_schema(prerequisite)?;
    }
    for key in ["commands", "failure_signatures"] {
        if field(object, key)?
            .as_array()
            .ok_or(C2SemanticError::InvalidRecord)?
            .iter()
            .any(|value| value.as_str().is_none())
        {
            return Err(C2SemanticError::InvalidRecord);
        }
    }
    validate_verification_schema(field(object, "verification")?)?;
    canonical_optional_timestamp(field(object, "expires_at")?)?;
    Ok(())
}

fn validate_memory_item_schema(value: &Value) -> C2Result<()> {
    let object = exact_object(
        value,
        &[
            "id",
            "kind",
            "title",
            "content",
            "scope",
            "origin",
            "writer",
            "evidence",
            "confidence",
            "status",
            "supersedes",
            "tags",
            "created_at",
            "updated_at",
            "last_used_at",
            "review_after",
            "archive",
            "procedure",
        ],
        &["correction_proposal_id", "pending_correction_proposal_id"],
    )?;
    string_field(object, "id")?;
    validate_unit_or_data_enum(
        field(object, "kind")?,
        &[
            "preference",
            "rule",
            "decision",
            "limitation",
            "project_fact",
            "repository_fact",
            "task_fact",
            "user_fact",
            "session_insight",
            "handoff",
            "procedure",
        ],
        Some("custom"),
    )?;
    string_field(object, "title")?;
    string_field(object, "content")?;
    validate_memory_scope_schema(field(object, "scope")?)?;
    validate_unit_or_data_enum(
        field(object, "origin")?,
        &[
            "user_stated",
            "user_corrected",
            "agent_observed",
            "agent_inferred",
            "tool_result",
            "imported",
            "migrated",
            "generated_summary",
        ],
        Some("custom"),
    )?;
    validate_writer_schema(field(object, "writer")?)?;
    for evidence in field(object, "evidence")?
        .as_array()
        .ok_or(C2SemanticError::InvalidRecord)?
    {
        validate_evidence_schema(evidence)?;
    }
    let confidence = field(object, "confidence")?
        .as_number()
        .ok_or(C2SemanticError::InvalidRecord)?;
    let confidence_value = confidence.as_f64().ok_or(C2SemanticError::InvalidRecord)?;
    if !confidence_value.is_finite()
        || !(0.0..=1.0).contains(&confidence_value)
        || (confidence_value == 0.0 && confidence.to_string().starts_with('-'))
    {
        return Err(C2SemanticError::InvalidRecord);
    }
    validate_unit_or_data_enum(
        field(object, "status")?,
        &[
            "active",
            "needs_review",
            "superseded",
            "archived",
            "rejected",
        ],
        None,
    )?;
    for key in ["supersedes", "tags"] {
        if field(object, key)?
            .as_array()
            .ok_or(C2SemanticError::InvalidRecord)?
            .iter()
            .any(|value| value.as_str().is_none())
        {
            return Err(C2SemanticError::InvalidRecord);
        }
    }
    validate_timestamp_fields(object, &["created_at", "updated_at"])?;
    validate_optional_timestamp_fields(object, &["last_used_at", "review_after"])?;
    if let Some(archive) = field(object, "archive")?.as_object() {
        let _ = archive;
        validate_archive_schema(field(object, "archive")?)?;
    } else if !field(object, "archive")?.is_null() {
        return Err(C2SemanticError::InvalidRecord);
    }
    if let Some(procedure) = field(object, "procedure")?.as_object() {
        let _ = procedure;
        validate_procedure_schema(field(object, "procedure")?)?;
    } else if !field(object, "procedure")?.is_null() {
        return Err(C2SemanticError::InvalidRecord);
    }
    for marker in ["correction_proposal_id", "pending_correction_proposal_id"] {
        if let Some(value) = object.get(marker) {
            if value.as_str().is_none() {
                return Err(C2SemanticError::InvalidRecord);
            }
        }
    }
    Ok(())
}

fn validate_proposal_schema(value: &Value) -> C2Result<CorrectionProposalStatus> {
    let object = value.as_object().ok_or(C2SemanticError::InvalidRecord)?;
    let status = match string_field(object, "status")? {
        "pending" => CorrectionProposalStatus::Pending,
        "applied" => CorrectionProposalStatus::Applied,
        _ => return Err(C2SemanticError::InvalidRecord),
    };
    let base = [
        "id",
        "obsolete_id",
        "replacement_id",
        "memory_kind",
        "scope",
        "canonical_digest",
        "digest_schema_version",
        "status",
        "proposer",
        "created_at",
        "applied_at",
    ];
    match status {
        CorrectionProposalStatus::Pending => {
            exact_keys(value, &base)?;
            if !field(object, "applied_at")?.is_null() {
                return Err(C2SemanticError::InvalidRecord);
            }
        }
        CorrectionProposalStatus::Applied => {
            let mut applied = base.to_vec();
            applied.insert(7, "applied_digest");
            exact_keys(value, &applied)?;
            validate_digest(string_field(object, "applied_digest")?)?;
            canonical_timestamp(field(object, "applied_at")?)?;
        }
    }
    for key in ["id", "obsolete_id", "replacement_id"] {
        string_field(object, key)?;
    }
    validate_unit_or_data_enum(
        field(object, "memory_kind")?,
        &[
            "preference",
            "rule",
            "decision",
            "limitation",
            "project_fact",
            "repository_fact",
            "task_fact",
            "user_fact",
            "session_insight",
            "handoff",
            "procedure",
        ],
        Some("custom"),
    )?;
    validate_memory_scope_schema(field(object, "scope")?)?;
    validate_digest(string_field(object, "canonical_digest")?)?;
    if field(object, "digest_schema_version")?.as_u64() != Some(1) {
        return Err(C2SemanticError::InvalidRecord);
    }
    validate_writer_schema(field(object, "proposer")?)?;
    canonical_timestamp(field(object, "created_at")?)?;
    Ok(status)
}

fn validate_repository_schema(value: &Value) -> C2Result<()> {
    let object = exact_keys(
        value,
        &[
            "id",
            "name",
            "remote_url",
            "provider",
            "default_branch",
            "description",
            "created_at",
            "updated_at",
        ],
    )?;
    string_field(object, "id")?;
    string_field(object, "name")?;
    optional_string_field(object, "remote_url")?;
    validate_unit_or_data_enum(
        field(object, "provider")?,
        &["git_hub", "git_lab", "bitbucket", "unknown"],
        Some("other"),
    )?;
    optional_string_field(object, "default_branch")?;
    optional_string_field(object, "description")?;
    validate_timestamp_fields(object, &["created_at", "updated_at"])
}

fn validate_checkout_schema(value: &Value) -> C2Result<()> {
    let object = exact_keys(
        value,
        &[
            "id",
            "repository_id",
            "local_path",
            "current_branch",
            "head_sha",
            "is_dirty",
            "created_at",
            "updated_at",
            "last_seen_at",
        ],
    )?;
    string_field(object, "id")?;
    optional_string_field(object, "repository_id")?;
    string_field(object, "local_path")?;
    optional_string_field(object, "current_branch")?;
    optional_string_field(object, "head_sha")?;
    if !matches!(field(object, "is_dirty")?, Value::Null | Value::Bool(_)) {
        return Err(C2SemanticError::InvalidRecord);
    }
    validate_timestamp_fields(object, &["created_at", "updated_at", "last_seen_at"])
}

fn validate_component_schema(value: &Value) -> C2Result<()> {
    let base = [
        "id",
        "repository_id",
        "name",
        "path",
        "kind",
        "description",
        "created_at",
        "updated_at",
    ];
    let object = value.as_object().ok_or(C2SemanticError::InvalidRecord)?;
    let has_source_path = object.contains_key("source_path");
    let has_source_digest = object.contains_key("source_sha256");
    if has_source_path != has_source_digest {
        return Err(C2SemanticError::InvalidRecord);
    }
    if has_source_path {
        let mut keys = base.to_vec();
        keys.splice(6..6, ["source_path", "source_sha256"]);
        exact_keys(value, &keys)?;
        string_field(object, "source_path")?;
        validate_digest(string_field(object, "source_sha256")?)?;
    } else {
        exact_keys(value, &base)?;
    }
    string_field(object, "id")?;
    string_field(object, "repository_id")?;
    string_field(object, "name")?;
    string_field(object, "path")?;
    optional_string_field(object, "kind")?;
    optional_string_field(object, "description")?;
    validate_timestamp_fields(object, &["created_at", "updated_at"])
}

fn validate_link_schema(value: &Value) -> C2Result<()> {
    let object = exact_keys(
        value,
        &[
            "id",
            "project_id",
            "project_name",
            "repository_id",
            "component_id",
            "component_path",
            "role",
            "created_at",
            "updated_at",
        ],
    )?;
    string_field(object, "id")?;
    optional_string_field(object, "project_id")?;
    string_field(object, "project_name")?;
    string_field(object, "repository_id")?;
    optional_string_field(object, "component_id")?;
    optional_string_field(object, "component_path")?;
    validate_unit_or_data_enum(
        field(object, "role")?,
        &["primary", "dependency", "produces", "related"],
        None,
    )?;
    validate_timestamp_fields(object, &["created_at", "updated_at"])
}

fn validate_forget_receipt_schema(value: &Value) -> C2Result<()> {
    let object = exact_object(
        value,
        &[
            "record_id",
            "deleted",
            "proposal_ids",
            "pending_replacement_ids",
            "unlocked_obsolete_ids",
            "cleanup_complete",
            "created_at",
        ],
        &["completed_at"],
    )?;
    parse_id(string_field(object, "record_id")?)?;
    if field(object, "deleted")?.as_bool().is_none()
        || field(object, "cleanup_complete")?.as_bool().is_none()
    {
        return Err(C2SemanticError::InvalidRecord);
    }
    for key in [
        "proposal_ids",
        "pending_replacement_ids",
        "unlocked_obsolete_ids",
    ] {
        for id in field(object, key)?
            .as_array()
            .ok_or(C2SemanticError::InvalidRecord)?
        {
            parse_id(id.as_str().ok_or(C2SemanticError::InvalidRecord)?)?;
        }
    }
    canonical_timestamp(field(object, "created_at")?)?;
    if let Some(completed_at) = object.get("completed_at") {
        canonical_timestamp(completed_at)?;
    }
    Ok(())
}

fn validate_project_row_schema(value: &Value) -> C2Result<()> {
    let object = exact_keys(
        value,
        &[
            "record_id",
            "name",
            "description",
            "status",
            "created_at",
            "updated_at",
        ],
    )?;
    string_field(object, "record_id")?;
    string_field(object, "name")?;
    optional_string_field(object, "description")?;
    validate_unit_or_data_enum(
        field(object, "status")?,
        &["planning", "active", "completed", "archived"],
        None,
    )?;
    validate_timestamp_fields(object, &["created_at", "updated_at"])
}

fn validate_task_row_schema(value: &Value) -> C2Result<()> {
    let object = exact_keys(
        value,
        &[
            "record_id",
            "project_id",
            "name",
            "description",
            "status",
            "priority",
            "jira_key",
            "blocked_by",
            "created_at",
            "updated_at",
        ],
    )?;
    string_field(object, "record_id")?;
    string_field(object, "project_id")?;
    string_field(object, "name")?;
    optional_string_field(object, "description")?;
    validate_unit_or_data_enum(
        field(object, "status")?,
        &["todo", "in_progress", "blocked", "done"],
        None,
    )?;
    validate_unit_or_data_enum(
        field(object, "priority")?,
        &["low", "medium", "high", "critical"],
        None,
    )?;
    optional_string_field(object, "jira_key")?;
    if field(object, "blocked_by")?
        .as_array()
        .ok_or(C2SemanticError::InvalidRecord)?
        .iter()
        .any(|id| id.as_str().is_none())
    {
        return Err(C2SemanticError::InvalidRecord);
    }
    validate_timestamp_fields(object, &["created_at", "updated_at"])
}

fn validate_absolute_path(value: &str) -> C2Result<()> {
    validate_identity_text(value, true)?;
    if value == "/"
        || !value.starts_with('/')
        || value.starts_with("//")
        || value.ends_with('/')
        || value.contains("//")
        || value[1..]
            .split('/')
            .any(|component| component.is_empty() || matches!(component, "." | ".."))
    {
        return Err(C2SemanticError::InvalidRecord);
    }
    Ok(())
}

fn validate_relative_path(value: &str, allow_dot: bool) -> C2Result<()> {
    validate_identity_text(value, true)?;
    if allow_dot && value == "." {
        return Ok(());
    }
    if value.starts_with('/')
        || value.ends_with('/')
        || value.contains("//")
        || value
            .split('/')
            .any(|component| component.is_empty() || matches!(component, "." | ".."))
    {
        return Err(C2SemanticError::InvalidRecord);
    }
    Ok(())
}

fn normalize_remote(value: &str) -> C2Result<String> {
    validate_identity_text(value, true)?;
    if value.contains(['?', '#']) {
        return Err(C2SemanticError::InvalidRecord);
    }
    let raw = value.trim().trim_end_matches('/');
    let explicit_git_suffix = raw.ends_with(".git");
    let uses_git_ssh = raw.starts_with("git@") || raw.starts_with("ssh://git@");
    let trimmed = raw.trim_end_matches(".git").trim_end_matches('/');
    let remote = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
        .or_else(|| trimmed.strip_prefix("ssh://git@"))
        .or_else(|| trimmed.strip_prefix("git@"))
        .or_else(|| {
            (trimmed.starts_with("github.com/")
                || trimmed.starts_with("gitlab.com/")
                || trimmed.starts_with("bitbucket.org/"))
            .then_some(trimmed)
        })
        .ok_or(C2SemanticError::InvalidRecord)?;
    let (host, path) = remote
        .split_once(':')
        .or_else(|| remote.split_once('/'))
        .ok_or(C2SemanticError::InvalidRecord)?;
    let lower_host = host.to_ascii_lowercase();
    if !host.contains('.')
        || host.contains('@')
        || path.contains('@')
        || (!matches!(
            lower_host.as_str(),
            "github.com" | "gitlab.com" | "bitbucket.org"
        ) && !lower_host.contains(".git.")
            && !explicit_git_suffix
            && !uses_git_ssh)
    {
        return Err(C2SemanticError::InvalidRecord);
    }
    let segments = path
        .trim_start_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    if segments.len() != 2 {
        return Err(C2SemanticError::InvalidRecord);
    }
    let owner = segments[0].trim_end_matches(".git");
    let repository = segments[1].trim_end_matches(".git");
    if owner.is_empty() || repository.is_empty() {
        return Err(C2SemanticError::InvalidRecord);
    }
    Ok(format!("{lower_host}/{owner}/{repository}"))
}

fn validate_head_sha(value: &str) -> C2Result<()> {
    if !matches!(value.len(), 40 | 64)
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(C2SemanticError::InvalidRecord);
    }
    Ok(())
}

fn decode<T: DeserializeOwned>(value: Value) -> C2Result<T> {
    serde_json::from_value(value).map_err(|_| C2SemanticError::InvalidRecord)
}

struct C2DecodedRow<T> {
    id: Id,
    raw: Value,
    value: T,
    record_mac: Option<[u8; 32]>,
}

fn deserialize_direct_row<T: DeserializeOwned>(raw: &Value) -> C2Result<T> {
    let mut object = raw
        .as_object()
        .ok_or(C2SemanticError::InvalidRecord)?
        .clone();
    let id = object
        .remove("record_id")
        .ok_or(C2SemanticError::InvalidRecord)?;
    object.insert("id".to_string(), id);
    decode(Value::Object(object))
}

fn strict_memory_row(raw: &Value) -> C2Result<MemoryItem> {
    let row = exact_keys(
        raw,
        &[
            "record_id",
            "item",
            "kind_key",
            "status_key",
            "scope_key",
            "harness_key",
            "model_key",
            "session_id",
            "snapshot_digest",
            "created_at",
            "updated_at",
        ],
    )?;
    for key in [
        "record_id",
        "kind_key",
        "status_key",
        "scope_key",
        "harness_key",
        "model_key",
        "snapshot_digest",
    ] {
        string_field(row, key)?;
    }
    for key in ["kind_key", "scope_key", "harness_key", "model_key"] {
        validate_identity_text(string_field(row, key)?, true)?;
    }
    validate_unit_or_data_enum(
        field(row, "status_key")?,
        &[
            "active",
            "needs_review",
            "superseded",
            "archived",
            "rejected",
        ],
        None,
    )?;
    optional_string_field(row, "session_id")?;
    validate_digest(string_field(row, "snapshot_digest")?)?;
    validate_timestamp_fields(row, &["created_at", "updated_at"])?;
    validate_memory_item_schema(field(row, "item")?)?;
    validate_canonical_ids(raw)?;
    let id = parse_id(string_field(row, "record_id")?)?;
    let item: MemoryItem = decode(field(row, "item")?.clone())?;
    validate_memory_domain(&item)?;
    if id != item.id {
        return Err(C2SemanticError::InvalidRecord);
    }
    Ok(item)
}

fn strict_proposal_row(raw: &Value) -> C2Result<CorrectionProposal> {
    let row = raw.as_object().ok_or(C2SemanticError::InvalidRecord)?;
    let status = validate_proposal_schema(field(row, "proposal")?)?;
    match status {
        CorrectionProposalStatus::Pending => {
            exact_keys(
                raw,
                &[
                    "record_id",
                    "proposal",
                    "status_key",
                    "obsolete_id",
                    "pending_obsolete_id",
                    "replacement_id",
                    "snapshot_digest",
                    "created_at",
                ],
            )?;
            string_field(row, "pending_obsolete_id")?;
        }
        CorrectionProposalStatus::Applied => {
            exact_keys(
                raw,
                &[
                    "record_id",
                    "proposal",
                    "status_key",
                    "obsolete_id",
                    "replacement_id",
                    "snapshot_digest",
                    "created_at",
                ],
            )?;
        }
    }
    for key in [
        "record_id",
        "status_key",
        "obsolete_id",
        "replacement_id",
        "snapshot_digest",
    ] {
        string_field(row, key)?;
    }
    validate_unit_or_data_enum(field(row, "status_key")?, &["pending", "applied"], None)?;
    validate_digest(string_field(row, "snapshot_digest")?)?;
    canonical_timestamp(field(row, "created_at")?)?;
    validate_canonical_ids(raw)?;
    let id = parse_id(string_field(row, "record_id")?)?;
    let proposal: CorrectionProposal = decode(field(row, "proposal")?.clone())?;
    validate_proposal_domain(&proposal)?;
    if id != proposal.id {
        return Err(C2SemanticError::InvalidRecord);
    }
    Ok(proposal)
}

fn strict_repository_row(raw: &Value) -> C2Result<GitRepository> {
    let row = exact_keys(
        raw,
        &[
            "record_id",
            "repository",
            "name_key",
            "remote_url",
            "provider_key",
            "created_at",
            "updated_at",
        ],
    )?;
    string_field(row, "record_id")?;
    validate_identity_text(string_field(row, "name_key")?, true)?;
    if let Some(remote) = optional_string_field(row, "remote_url")? {
        normalize_remote(remote)?;
    }
    validate_identity_text(string_field(row, "provider_key")?, true)?;
    validate_timestamp_fields(row, &["created_at", "updated_at"])?;
    validate_repository_schema(field(row, "repository")?)?;
    validate_canonical_ids(raw)?;
    let id = parse_id(string_field(row, "record_id")?)?;
    let repository: GitRepository = decode(field(row, "repository")?.clone())?;
    validate_repository_domain(&repository)?;
    if id != repository.id {
        return Err(C2SemanticError::InvalidRecord);
    }
    Ok(repository)
}

fn strict_checkout_row(raw: &Value) -> C2Result<LocalCheckout> {
    let row = exact_keys(
        raw,
        &[
            "record_id",
            "checkout",
            "repository_id",
            "local_path_key",
            "current_branch",
            "head_sha",
            "is_dirty",
            "created_at",
            "updated_at",
            "last_seen_at",
        ],
    )?;
    string_field(row, "record_id")?;
    optional_string_field(row, "repository_id")?;
    validate_absolute_path(string_field(row, "local_path_key")?)?;
    if let Some(branch) = optional_string_field(row, "current_branch")? {
        validate_identity_text(branch, true)?;
    }
    if let Some(head) = optional_string_field(row, "head_sha")? {
        validate_head_sha(head)?;
    }
    if !matches!(field(row, "is_dirty")?, Value::Null | Value::Bool(_)) {
        return Err(C2SemanticError::InvalidRecord);
    }
    validate_timestamp_fields(row, &["created_at", "updated_at", "last_seen_at"])?;
    validate_checkout_schema(field(row, "checkout")?)?;
    validate_canonical_ids(raw)?;
    let id = parse_id(string_field(row, "record_id")?)?;
    let checkout: LocalCheckout = decode(field(row, "checkout")?.clone())?;
    validate_checkout_domain(&checkout)?;
    if id != checkout.id {
        return Err(C2SemanticError::InvalidRecord);
    }
    Ok(checkout)
}

fn strict_component_row(raw: &Value) -> C2Result<MonorepoComponent> {
    let row = exact_keys(
        raw,
        &[
            "record_id",
            "component",
            "repository_id",
            "name_key",
            "path_key",
            "kind",
            "created_at",
            "updated_at",
        ],
    )?;
    string_field(row, "record_id")?;
    string_field(row, "repository_id")?;
    validate_identity_text(string_field(row, "name_key")?, true)?;
    validate_relative_path(string_field(row, "path_key")?, true)?;
    if let Some(kind) = optional_string_field(row, "kind")? {
        validate_identity_text(kind, true)?;
    }
    validate_timestamp_fields(row, &["created_at", "updated_at"])?;
    validate_component_schema(field(row, "component")?)?;
    validate_canonical_ids(raw)?;
    let id = parse_id(string_field(row, "record_id")?)?;
    let component: MonorepoComponent = decode(field(row, "component")?.clone())?;
    validate_component_domain(&component)?;
    if id != component.id {
        return Err(C2SemanticError::InvalidRecord);
    }
    Ok(component)
}

fn strict_link_row(raw: &Value) -> C2Result<ProjectRepositoryLink> {
    let row = exact_keys(
        raw,
        &[
            "record_id",
            "link",
            "project_id",
            "project_name_key",
            "repository_id",
            "component_id",
            "component_path_key",
            "role",
            "created_at",
            "updated_at",
        ],
    )?;
    string_field(row, "record_id")?;
    optional_string_field(row, "project_id")?;
    validate_identity_text(string_field(row, "project_name_key")?, true)?;
    string_field(row, "repository_id")?;
    optional_string_field(row, "component_id")?;
    if let Some(path) = optional_string_field(row, "component_path_key")? {
        validate_relative_path(path, true)?;
    }
    validate_unit_or_data_enum(
        field(row, "role")?,
        &["primary", "dependency", "produces", "related"],
        None,
    )?;
    validate_timestamp_fields(row, &["created_at", "updated_at"])?;
    validate_link_schema(field(row, "link")?)?;
    validate_canonical_ids(raw)?;
    let id = parse_id(string_field(row, "record_id")?)?;
    let link: ProjectRepositoryLink = decode(field(row, "link")?.clone())?;
    validate_link_domain(&link)?;
    if id != link.id {
        return Err(C2SemanticError::InvalidRecord);
    }
    Ok(link)
}

fn strict_project_row(raw: &Value) -> C2Result<Project> {
    validate_project_row_schema(raw)?;
    validate_canonical_ids(raw)?;
    let project: Project = deserialize_direct_row(raw)?;
    validate_project_domain(&project)?;
    Ok(project)
}

fn strict_task_row(raw: &Value) -> C2Result<Task> {
    validate_task_row_schema(raw)?;
    validate_canonical_ids(raw)?;
    let task: Task = deserialize_direct_row(raw)?;
    validate_task_domain(&task)?;
    Ok(task)
}

fn validate_scope_domain(scope: &MemoryScope) -> C2Result<()> {
    match scope {
        MemoryScope::Global | MemoryScope::User => {}
        MemoryScope::Project {
            project_id,
            project_name,
        } => {
            if let Some(id) = project_id {
                validate_id(id)?;
            }
            validate_identity_text(project_name, true)?;
        }
        MemoryScope::Task {
            project_id,
            project_name,
            task_id,
            task_name,
        } => {
            if let Some(id) = project_id {
                validate_id(id)?;
            }
            if let Some(name) = project_name {
                validate_identity_text(name, true)?;
            }
            if let Some(id) = task_id {
                validate_id(id)?;
            }
            validate_identity_text(task_name, true)?;
        }
        MemoryScope::Entity {
            entity_id,
            entity_name,
        } => {
            if let Some(id) = entity_id {
                validate_id(id)?;
            }
            validate_identity_text(entity_name, true)?;
        }
        MemoryScope::Repository {
            repository_id,
            remote_url,
            local_path,
        } => {
            if let Some(id) = repository_id {
                validate_id(id)?;
            }
            if let Some(remote) = remote_url {
                normalize_remote(remote)?;
            }
            if let Some(path) = local_path {
                validate_absolute_path(path)?;
            }
        }
        MemoryScope::Session { session_id } => validate_id(session_id)?,
        MemoryScope::Custom { name } => validate_identity_text(name, true)?,
    }
    Ok(())
}

fn validate_writer_domain(writer: &engram_core::memory::WriterProvenance) -> C2Result<()> {
    if let Harness::Other(value) = &writer.harness {
        validate_identity_text(value, true)?;
    }
    if let Some(version) = &writer.harness_version {
        validate_identity_text(version, false)?;
    }
    validate_identity_text(&writer.model.provider, true)?;
    validate_identity_text(&writer.model.model, true)?;
    if let Some(version) = &writer.model.version {
        validate_identity_text(version, false)?;
    }
    if let Some(surface) = &writer.surface {
        validate_identity_text(surface, false)?;
    }
    validate_identity_text(&writer.actor, true)?;
    if let Some(session_id) = &writer.session_id {
        validate_id(session_id)?;
    }
    Ok(())
}

fn validate_evidence_domain(evidence: &EvidenceRef) -> C2Result<()> {
    if let EvidenceKind::Custom(value) = &evidence.kind {
        validate_identity_text(value, true)?;
    }
    validate_identity_text(&evidence.target, true)?;
    if let Some(summary) = &evidence.summary {
        validate_prose(summary, false)?;
    }
    if let Some(excerpt) = &evidence.excerpt {
        validate_prose(excerpt, false)?;
    }
    Ok(())
}

fn validate_memory_domain(item: &MemoryItem) -> C2Result<()> {
    validate_id(&item.id)?;
    if let MemoryKind::Custom(value) = &item.kind {
        validate_identity_text(value, true)?;
    }
    validate_prose(&item.title, true)?;
    validate_prose(&item.content, true)?;
    validate_scope_domain(&item.scope)?;
    if let ClaimOrigin::Custom(value) = &item.origin {
        validate_identity_text(value, true)?;
    }
    validate_writer_domain(&item.writer)?;
    for evidence in &item.evidence {
        validate_evidence_domain(evidence)?;
    }
    let confidence = item.confidence.value();
    if !confidence.is_finite()
        || !(0.0..=1.0).contains(&confidence)
        || confidence.is_sign_negative()
    {
        return Err(C2SemanticError::InvalidRecord);
    }
    for id in &item.supersedes {
        validate_id(id)?;
    }
    for tag in &item.tags {
        validate_identity_text(tag, true)?;
    }
    if let Some(archive) = &item.archive {
        validate_prose(&archive.reason, true)?;
        if let Some(actor) = &archive.archived_by {
            validate_identity_text(actor, false)?;
        }
    }
    if item.status == MemoryStatus::Archived {
        if item.archive.is_none() {
            return Err(C2SemanticError::InvalidRecord);
        }
    } else if item.archive.is_some() {
        return Err(C2SemanticError::InvalidRecord);
    }
    match (&item.kind, &item.procedure) {
        (MemoryKind::Procedure, Some(procedure)) => {
            validate_prose(&procedure.task, true)?;
            if procedure.commands.is_empty()
                || procedure
                    .commands
                    .iter()
                    .any(|command| validate_prose(command, true).is_err())
            {
                return Err(C2SemanticError::InvalidRecord);
            }
            if procedure
                .failure_signatures
                .iter()
                .any(|signature| validate_prose(signature, true).is_err())
            {
                return Err(C2SemanticError::InvalidRecord);
            }
            let mut keys = BTreeSet::new();
            for prerequisite in &procedure.prerequisites {
                validate_identity_text(&prerequisite.key, true)?;
                validate_identity_text(&prerequisite.expected, true)?;
                if !keys.insert(prerequisite.key.to_ascii_lowercase()) {
                    return Err(C2SemanticError::InvalidRecord);
                }
                if let Some(ProcedurePrerequisiteSource::Toml {
                    relative_path,
                    key_path,
                }) = &prerequisite.source
                {
                    validate_relative_path(relative_path, false)?;
                    if key_path.is_empty() {
                        return Err(C2SemanticError::InvalidRecord);
                    }
                    for key in key_path {
                        validate_identity_text(key, true)?;
                    }
                }
            }
            validate_prose(&procedure.verification.command, true)?;
            validate_prose(&procedure.verification.expected_output_contains, true)?;
            if let Some(path) = &procedure.verification.evidence_path {
                validate_identity_text(path, true)?;
            }
            if let Some(digest) = &procedure.verification.evidence_sha256 {
                validate_digest(digest)?;
            }
            let proof_parts = [
                procedure.verification.evidence_path.is_some(),
                procedure.verification.evidence_sha256.is_some(),
                procedure.verification.verified_at.is_some(),
            ];
            if proof_parts.iter().any(|present| *present)
                && proof_parts.iter().any(|present| !*present)
            {
                return Err(C2SemanticError::InvalidRecord);
            }
        }
        (MemoryKind::Procedure, None) | (_, Some(_)) => {
            return Err(C2SemanticError::InvalidRecord);
        }
        (_, None) => {}
    }
    if let Some(id) = &item.correction_proposal_id {
        validate_id(id)?;
    }
    if let Some(id) = &item.pending_correction_proposal_id {
        validate_id(id)?;
    }
    Ok(())
}

fn validate_proposal_domain(proposal: &CorrectionProposal) -> C2Result<()> {
    validate_id(&proposal.id)?;
    validate_id(&proposal.obsolete_id)?;
    validate_id(&proposal.replacement_id)?;
    if let MemoryKind::Custom(value) = &proposal.memory_kind {
        validate_identity_text(value, true)?;
    }
    validate_scope_domain(&proposal.scope)?;
    validate_digest(&proposal.canonical_digest)?;
    if proposal.digest_schema_version != 1 {
        return Err(C2SemanticError::InvalidRecord);
    }
    if let Some(digest) = &proposal.applied_digest {
        validate_digest(digest)?;
    }
    validate_writer_domain(&proposal.proposer)?;
    match proposal.status {
        CorrectionProposalStatus::Pending
            if proposal.applied_digest.is_none() && proposal.applied_at.is_none() => {}
        CorrectionProposalStatus::Applied
            if proposal.applied_digest.is_some() && proposal.applied_at.is_some() => {}
        _ => return Err(C2SemanticError::InvalidRecord),
    }
    Ok(())
}

fn validate_project_domain(project: &Project) -> C2Result<()> {
    validate_id(&project.id)?;
    validate_identity_text(&project.name, true)?;
    if let Some(description) = &project.description {
        validate_prose(description, false)?;
    }
    Ok(())
}

fn validate_task_domain(task: &Task) -> C2Result<()> {
    validate_id(&task.id)?;
    validate_id(&task.project_id)?;
    validate_identity_text(&task.name, true)?;
    if let Some(description) = &task.description {
        validate_prose(description, false)?;
    }
    if let Some(jira_key) = &task.jira_key {
        validate_identity_text(jira_key, true)?;
    }
    for blocker in &task.blocked_by {
        validate_id(blocker)?;
    }
    Ok(())
}

fn validate_repository_domain(repository: &GitRepository) -> C2Result<()> {
    validate_id(&repository.id)?;
    validate_identity_text(&repository.name, true)?;
    if let Some(remote) = &repository.remote_url {
        normalize_remote(remote)?;
    }
    if let RepositoryProvider::Other(value) = &repository.provider {
        validate_identity_text(value, true)?;
    }
    if let Some(branch) = &repository.default_branch {
        validate_identity_text(branch, true)?;
    }
    if let Some(description) = &repository.description {
        validate_prose(description, false)?;
    }
    Ok(())
}

fn validate_checkout_domain(checkout: &LocalCheckout) -> C2Result<()> {
    validate_id(&checkout.id)?;
    if let Some(repository_id) = &checkout.repository_id {
        validate_id(repository_id)?;
    }
    validate_absolute_path(&checkout.local_path)?;
    if let Some(branch) = &checkout.current_branch {
        validate_identity_text(branch, true)?;
    }
    if let Some(head) = &checkout.head_sha {
        validate_head_sha(head)?;
    }
    Ok(())
}

fn validate_component_domain(component: &MonorepoComponent) -> C2Result<()> {
    validate_id(&component.id)?;
    validate_id(&component.repository_id)?;
    validate_identity_text(&component.name, true)?;
    validate_relative_path(&component.path, true)?;
    if let Some(kind) = &component.kind {
        validate_identity_text(kind, true)?;
    }
    if let Some(description) = &component.description {
        validate_prose(description, false)?;
    }
    match (&component.source_path, &component.source_sha256) {
        (Some(path), Some(digest)) => {
            validate_relative_path(path, true)?;
            validate_digest(digest)?;
        }
        (None, None) => {}
        _ => return Err(C2SemanticError::InvalidRecord),
    }
    Ok(())
}

fn validate_link_domain(link: &ProjectRepositoryLink) -> C2Result<()> {
    validate_id(&link.id)?;
    if let Some(project_id) = &link.project_id {
        validate_id(project_id)?;
    }
    validate_identity_text(&link.project_name, true)?;
    validate_id(&link.repository_id)?;
    if let Some(component_id) = &link.component_id {
        validate_id(component_id)?;
    }
    if let Some(path) = &link.component_path {
        validate_relative_path(path, true)?;
    }
    if link.component_id.is_some() != link.component_path.is_some() {
        return Err(C2SemanticError::InvalidRecord);
    }
    Ok(())
}

fn validate_canonical_ids(value: &Value) -> C2Result<()> {
    fn walk(value: &Value, key: Option<&str>) -> C2Result<()> {
        if let Some(key) = key {
            if key == "id" || key == "record_id" || key.ends_with("_id") {
                return match value {
                    Value::Null => Ok(()),
                    Value::String(value) => parse_id(value).map(|_| ()),
                    _ => Err(C2SemanticError::InvalidRecord),
                };
            }
            if key == "supersedes" || key == "blocked_by" || key.ends_with("_ids") {
                let values = value.as_array().ok_or(C2SemanticError::InvalidRecord)?;
                for value in values {
                    parse_id(value.as_str().ok_or(C2SemanticError::InvalidRecord)?)?;
                }
                return Ok(());
            }
        }
        match value {
            Value::Array(values) => {
                for value in values {
                    walk(value, None)?;
                }
            }
            Value::Object(object) => {
                for (key, value) in object {
                    walk(value, Some(key))?;
                }
            }
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
        }
        Ok(())
    }
    walk(value, None)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(64);
    for byte in digest {
        use fmt::Write as _;
        let _ = write!(encoded, "{byte:02x}");
    }
    encoded
}

fn snapshot_digest<T: Serialize>(value: &T) -> C2Result<String> {
    let bytes = serde_json::to_vec(value).map_err(|_| C2SemanticError::InvalidRecord)?;
    Ok(sha256_hex(&bytes))
}

fn constant_time_equal(left: &str, right: &str) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.bytes()
        .zip(right.bytes())
        .fold(0_u8, |difference, (left, right)| {
            difference | (left ^ right)
        })
        == 0
}

fn value_matches<T: Serialize>(actual: &Value, expected: &T) -> C2Result<bool> {
    let expected = serde_json::to_value(expected).map_err(|_| C2SemanticError::InvalidRecord)?;
    Ok(actual == &expected)
}

fn scope_key(scope: &MemoryScope) -> String {
    match scope {
        MemoryScope::Global => "global".to_string(),
        MemoryScope::User => "user".to_string(),
        MemoryScope::Project { project_name, .. } => format!("project:{project_name}"),
        MemoryScope::Task { task_name, .. } => format!("task:{task_name}"),
        MemoryScope::Entity { entity_name, .. } => format!("entity:{entity_name}"),
        MemoryScope::Repository {
            remote_url,
            local_path,
            ..
        } => format!(
            "repository:{}",
            remote_url
                .as_deref()
                .or(local_path.as_deref())
                .unwrap_or_default()
        ),
        MemoryScope::Session { session_id } => format!("session:{session_id}"),
        MemoryScope::Custom { name } => format!("custom:{name}"),
    }
}

fn decode_memory_row(raw: Value) -> C2Result<C2DecodedRow<MemoryItem>> {
    let row = exact_keys(
        &raw,
        &[
            "record_id",
            "item",
            "kind_key",
            "status_key",
            "scope_key",
            "harness_key",
            "model_key",
            "session_id",
            "snapshot_digest",
            "created_at",
            "updated_at",
        ],
    )?;
    validate_memory_item_schema(field(row, "item")?)?;
    validate_canonical_ids(&raw)?;
    validate_timestamp_fields(row, &["created_at", "updated_at"])?;
    let id = parse_id(string_field(row, "record_id")?)?;
    let item: MemoryItem = decode(field(row, "item")?.clone())?;
    validate_memory_domain(&item)?;
    if id != item.id
        || string_field(row, "kind_key")? != item.kind.to_string()
        || string_field(row, "status_key")? != item.status.to_string()
        || string_field(row, "scope_key")? != scope_key(&item.scope)
        || string_field(row, "harness_key")? != item.writer.harness.to_string()
        || string_field(row, "model_key")? != item.writer.model.model
        || !value_matches(field(row, "session_id")?, &item.writer.session_id)?
        || field(row, "created_at")?
            != field(
                field(row, "item")?
                    .as_object()
                    .ok_or(C2SemanticError::InvalidRecord)?,
                "created_at",
            )?
        || field(row, "updated_at")?
            != field(
                field(row, "item")?
                    .as_object()
                    .ok_or(C2SemanticError::InvalidRecord)?,
                "updated_at",
            )?
    {
        return Err(C2SemanticError::InconsistentProjection);
    }
    let expected_digest = snapshot_digest(&item)?;
    let persisted_digest = string_field(row, "snapshot_digest")?;
    validate_digest(persisted_digest)?;
    if !constant_time_equal(persisted_digest, &expected_digest) {
        return Err(C2SemanticError::InconsistentProjection);
    }
    Ok(C2DecodedRow {
        id,
        raw,
        value: item,
        record_mac: None,
    })
}

fn decode_proposal_row(raw: Value) -> C2Result<C2DecodedRow<CorrectionProposal>> {
    let row = raw.as_object().ok_or(C2SemanticError::InvalidRecord)?;
    let proposal_status = validate_proposal_schema(field(row, "proposal")?)?;
    let keys = match proposal_status {
        CorrectionProposalStatus::Pending => vec![
            "record_id",
            "proposal",
            "status_key",
            "obsolete_id",
            "pending_obsolete_id",
            "replacement_id",
            "snapshot_digest",
            "created_at",
        ],
        CorrectionProposalStatus::Applied => vec![
            "record_id",
            "proposal",
            "status_key",
            "obsolete_id",
            "replacement_id",
            "snapshot_digest",
            "created_at",
        ],
    };
    exact_keys(&raw, &keys)?;
    validate_canonical_ids(&raw)?;
    canonical_timestamp(field(row, "created_at")?)?;
    let id = parse_id(string_field(row, "record_id")?)?;
    let proposal: CorrectionProposal = decode(field(row, "proposal")?.clone())?;
    validate_proposal_domain(&proposal)?;
    if id != proposal.id
        || string_field(row, "status_key")? != proposal.status.to_string()
        || !value_matches(field(row, "obsolete_id")?, &proposal.obsolete_id)?
        || !value_matches(field(row, "replacement_id")?, &proposal.replacement_id)?
        || field(row, "created_at")?
            != field(
                field(row, "proposal")?
                    .as_object()
                    .ok_or(C2SemanticError::InvalidRecord)?,
                "created_at",
            )?
        || (proposal.status == CorrectionProposalStatus::Pending
            && !value_matches(field(row, "pending_obsolete_id")?, &proposal.obsolete_id)?)
    {
        return Err(C2SemanticError::InconsistentProjection);
    }
    let expected_digest = snapshot_digest(&proposal)?;
    let persisted_digest = string_field(row, "snapshot_digest")?;
    validate_digest(persisted_digest)?;
    if !constant_time_equal(persisted_digest, &expected_digest) {
        return Err(C2SemanticError::InconsistentProjection);
    }
    Ok(C2DecodedRow {
        id,
        raw,
        value: proposal,
        record_mac: None,
    })
}

fn decode_project_row(raw: Value) -> C2Result<C2DecodedRow<Project>> {
    validate_project_row_schema(&raw)?;
    validate_canonical_ids(&raw)?;
    let row = raw.as_object().ok_or(C2SemanticError::InvalidRecord)?;
    let id = parse_id(string_field(row, "record_id")?)?;
    let project: Project = deserialize_direct_row(&raw)?;
    validate_project_domain(&project)?;
    if id != project.id {
        return Err(C2SemanticError::InconsistentProjection);
    }
    Ok(C2DecodedRow {
        id,
        raw,
        value: project,
        record_mac: None,
    })
}

fn decode_task_row(raw: Value) -> C2Result<C2DecodedRow<Task>> {
    validate_task_row_schema(&raw)?;
    validate_canonical_ids(&raw)?;
    let row = raw.as_object().ok_or(C2SemanticError::InvalidRecord)?;
    let id = parse_id(string_field(row, "record_id")?)?;
    let task: Task = deserialize_direct_row(&raw)?;
    validate_task_domain(&task)?;
    if id != task.id {
        return Err(C2SemanticError::InconsistentProjection);
    }
    Ok(C2DecodedRow {
        id,
        raw,
        value: task,
        record_mac: None,
    })
}

fn decode_repository_row(raw: Value) -> C2Result<C2DecodedRow<GitRepository>> {
    let row = exact_keys(
        &raw,
        &[
            "record_id",
            "repository",
            "name_key",
            "remote_url",
            "provider_key",
            "created_at",
            "updated_at",
        ],
    )?;
    validate_repository_schema(field(row, "repository")?)?;
    validate_canonical_ids(&raw)?;
    validate_timestamp_fields(row, &["created_at", "updated_at"])?;
    let id = parse_id(string_field(row, "record_id")?)?;
    let repository: GitRepository = decode(field(row, "repository")?.clone())?;
    validate_repository_domain(&repository)?;
    let embedded = field(row, "repository")?
        .as_object()
        .ok_or(C2SemanticError::InvalidRecord)?;
    if id != repository.id
        || string_field(row, "name_key")? != repository.name.to_lowercase()
        || !value_matches(field(row, "remote_url")?, &repository.remote_url)?
        || string_field(row, "provider_key")? != repository.provider.to_string()
        || field(row, "created_at")? != field(embedded, "created_at")?
        || field(row, "updated_at")? != field(embedded, "updated_at")?
    {
        return Err(C2SemanticError::InconsistentProjection);
    }
    Ok(C2DecodedRow {
        id,
        raw,
        value: repository,
        record_mac: None,
    })
}

fn decode_checkout_row(raw: Value) -> C2Result<C2DecodedRow<LocalCheckout>> {
    let row = exact_keys(
        &raw,
        &[
            "record_id",
            "checkout",
            "repository_id",
            "local_path_key",
            "current_branch",
            "head_sha",
            "is_dirty",
            "created_at",
            "updated_at",
            "last_seen_at",
        ],
    )?;
    validate_checkout_schema(field(row, "checkout")?)?;
    validate_canonical_ids(&raw)?;
    validate_timestamp_fields(row, &["created_at", "updated_at", "last_seen_at"])?;
    let id = parse_id(string_field(row, "record_id")?)?;
    let checkout: LocalCheckout = decode(field(row, "checkout")?.clone())?;
    validate_checkout_domain(&checkout)?;
    let embedded = field(row, "checkout")?
        .as_object()
        .ok_or(C2SemanticError::InvalidRecord)?;
    if id != checkout.id
        || !value_matches(field(row, "repository_id")?, &checkout.repository_id)?
        || string_field(row, "local_path_key")? != checkout.local_path
        || !value_matches(field(row, "current_branch")?, &checkout.current_branch)?
        || !value_matches(field(row, "head_sha")?, &checkout.head_sha)?
        || !value_matches(field(row, "is_dirty")?, &checkout.is_dirty)?
        || field(row, "created_at")? != field(embedded, "created_at")?
        || field(row, "updated_at")? != field(embedded, "updated_at")?
        || field(row, "last_seen_at")? != field(embedded, "last_seen_at")?
    {
        return Err(C2SemanticError::InconsistentProjection);
    }
    Ok(C2DecodedRow {
        id,
        raw,
        value: checkout,
        record_mac: None,
    })
}

fn decode_component_row(raw: Value) -> C2Result<C2DecodedRow<MonorepoComponent>> {
    let row = exact_keys(
        &raw,
        &[
            "record_id",
            "component",
            "repository_id",
            "name_key",
            "path_key",
            "kind",
            "created_at",
            "updated_at",
        ],
    )?;
    validate_component_schema(field(row, "component")?)?;
    validate_canonical_ids(&raw)?;
    validate_timestamp_fields(row, &["created_at", "updated_at"])?;
    let id = parse_id(string_field(row, "record_id")?)?;
    let component: MonorepoComponent = decode(field(row, "component")?.clone())?;
    validate_component_domain(&component)?;
    let embedded = field(row, "component")?
        .as_object()
        .ok_or(C2SemanticError::InvalidRecord)?;
    if id != component.id
        || !value_matches(field(row, "repository_id")?, &component.repository_id)?
        || string_field(row, "name_key")? != component.name.to_lowercase()
        || string_field(row, "path_key")? != component.path
        || !value_matches(field(row, "kind")?, &component.kind)?
        || field(row, "created_at")? != field(embedded, "created_at")?
        || field(row, "updated_at")? != field(embedded, "updated_at")?
    {
        return Err(C2SemanticError::InconsistentProjection);
    }
    Ok(C2DecodedRow {
        id,
        raw,
        value: component,
        record_mac: None,
    })
}

fn decode_link_row(raw: Value) -> C2Result<C2DecodedRow<ProjectRepositoryLink>> {
    let row = exact_keys(
        &raw,
        &[
            "record_id",
            "link",
            "project_id",
            "project_name_key",
            "repository_id",
            "component_id",
            "component_path_key",
            "role",
            "created_at",
            "updated_at",
        ],
    )?;
    validate_link_schema(field(row, "link")?)?;
    validate_canonical_ids(&raw)?;
    validate_timestamp_fields(row, &["created_at", "updated_at"])?;
    let id = parse_id(string_field(row, "record_id")?)?;
    let link: ProjectRepositoryLink = decode(field(row, "link")?.clone())?;
    validate_link_domain(&link)?;
    let embedded = field(row, "link")?
        .as_object()
        .ok_or(C2SemanticError::InvalidRecord)?;
    if id != link.id
        || !value_matches(field(row, "project_id")?, &link.project_id)?
        || string_field(row, "project_name_key")? != link.project_name.to_lowercase()
        || !value_matches(field(row, "repository_id")?, &link.repository_id)?
        || !value_matches(field(row, "component_id")?, &link.component_id)?
        || !value_matches(field(row, "component_path_key")?, &link.component_path)?
        || string_field(row, "role")? != link.role.to_string()
        || field(row, "created_at")? != field(embedded, "created_at")?
        || field(row, "updated_at")? != field(embedded, "updated_at")?
    {
        return Err(C2SemanticError::InconsistentProjection);
    }
    Ok(C2DecodedRow {
        id,
        raw,
        value: link,
        record_mac: None,
    })
}

struct C2TargetSelector {
    project_id: Id,
    project_name: String,
    repository_id: Id,
    normalized_repository_remote: String,
    checkout_id: Id,
    checkout_path: String,
    task_id: Option<Id>,
    task_name: Option<String>,
}

struct C2DecodedSnapshot {
    memory_items: Vec<C2DecodedRow<MemoryItem>>,
    correction_proposals: Vec<C2DecodedRow<CorrectionProposal>>,
    projects: Vec<C2DecodedRow<Project>>,
    tasks: Vec<C2DecodedRow<Task>>,
    repositories: Vec<C2DecodedRow<GitRepository>>,
    checkouts: Vec<C2DecodedRow<LocalCheckout>>,
    components: Vec<C2DecodedRow<MonorepoComponent>>,
    project_links: Vec<C2DecodedRow<ProjectRepositoryLink>>,
}

#[derive(Serialize)]
struct C2CorrectionDigestItem<'a> {
    id: &'a Id,
    kind: &'a MemoryKind,
    title: &'a str,
    content: &'a str,
    scope: &'a MemoryScope,
    origin: &'a ClaimOrigin,
    writer: &'a engram_core::memory::WriterProvenance,
    evidence: &'a [EvidenceRef],
    confidence: f32,
    status: MemoryStatus,
    supersedes: &'a [Id],
    tags: &'a [String],
    review_after: Option<OffsetDateTime>,
    archive: &'a Option<engram_core::memory::ArchiveMetadata>,
    procedure: &'a Option<engram_core::memory::ProcedureCard>,
    correction_proposal_id: Option<Id>,
    pending_correction_proposal_id: Option<Id>,
}

impl<'a> From<&'a MemoryItem> for C2CorrectionDigestItem<'a> {
    fn from(item: &'a MemoryItem) -> Self {
        Self {
            id: &item.id,
            kind: &item.kind,
            title: &item.title,
            content: &item.content,
            scope: &item.scope,
            origin: &item.origin,
            writer: &item.writer,
            evidence: &item.evidence,
            confidence: item.confidence.value(),
            status: item.status,
            supersedes: &item.supersedes,
            tags: &item.tags,
            review_after: item.review_after,
            archive: &item.archive,
            procedure: &item.procedure,
            correction_proposal_id: item.correction_proposal_id,
            pending_correction_proposal_id: item.pending_correction_proposal_id,
        }
    }
}

#[derive(Serialize)]
struct C2CorrectionDigestPayload<'a> {
    schema_version: u32,
    proposal_id: &'a Id,
    obsolete_id: &'a Id,
    replacement_id: &'a Id,
    memory_kind: &'a MemoryKind,
    scope: &'a MemoryScope,
    obsolete: C2CorrectionDigestItem<'a>,
    replacement: C2CorrectionDigestItem<'a>,
}

fn correction_digest(
    proposal: &CorrectionProposal,
    obsolete: &MemoryItem,
    replacement: &MemoryItem,
) -> C2Result<String> {
    let payload = C2CorrectionDigestPayload {
        schema_version: proposal.digest_schema_version,
        proposal_id: &proposal.id,
        obsolete_id: &proposal.obsolete_id,
        replacement_id: &proposal.replacement_id,
        memory_kind: &proposal.memory_kind,
        scope: &proposal.scope,
        obsolete: obsolete.into(),
        replacement: replacement.into(),
    };
    snapshot_digest(&payload)
}

fn strict_target(target: &C2RawTarget) -> C2Result<C2TargetSelector> {
    for value in [
        target.project_id.as_str(),
        target.project_name.as_str(),
        target.repository_id.as_str(),
        target.repository_remote.as_str(),
        target.checkout_id.as_str(),
        target.checkout_path.as_str(),
    ] {
        validate_identity_text(value, true)?;
    }
    if target.task_id.is_some() != target.task_name.is_some() {
        return Err(C2SemanticError::InvalidRecord);
    }
    if let Some(task_id) = &target.task_id {
        validate_identity_text(task_id, true)?;
    }
    if let Some(task_name) = &target.task_name {
        validate_identity_text(task_name, true)?;
    }
    let normalized_repository_remote = normalize_remote(&target.repository_remote)?;
    validate_absolute_path(&target.checkout_path)?;
    Ok(C2TargetSelector {
        project_id: parse_id(&target.project_id)?,
        project_name: target.project_name.clone(),
        repository_id: parse_id(&target.repository_id)?,
        normalized_repository_remote,
        checkout_id: parse_id(&target.checkout_id)?,
        checkout_path: target.checkout_path.clone(),
        task_id: target.task_id.as_deref().map(parse_id).transpose()?,
        task_name: target.task_name.clone(),
    })
}

fn serialized_equal<T: Serialize>(left: &T, right: &T) -> C2Result<bool> {
    let left = serde_json::to_vec(left).map_err(|_| C2SemanticError::InvalidRecord)?;
    let right = serde_json::to_vec(right).map_err(|_| C2SemanticError::InvalidRecord)?;
    Ok(left == right)
}

fn strict_prior_bindings(prior_bindings: &[C2PriorCorrectionBinding]) -> C2Result<()> {
    for binding in prior_bindings {
        validate_id(&binding.proposal_id)?;
        strict_proposal_row(&binding.proposal_row)?;
        strict_memory_row(&binding.obsolete_row)?;
        strict_memory_row(&binding.replacement_row)?;
    }
    Ok(())
}

fn strict_record_stage(
    raw: &C2RawSnapshot,
    prior_bindings: &[C2PriorCorrectionBinding],
) -> C2Result<C2TargetSelector> {
    let target = strict_target(&raw.target)?;
    strict_prior_bindings(prior_bindings)?;
    for row in &raw.memory_item {
        strict_memory_row(row)?;
    }
    for row in &raw.correction_proposal {
        strict_proposal_row(row)?;
    }
    for row in &raw.memory_forget_receipt {
        validate_forget_receipt_schema(row)?;
    }
    for row in &raw.work_project {
        strict_project_row(row)?;
    }
    for row in &raw.work_task {
        strict_task_row(row)?;
    }
    for row in &raw.git_repository {
        strict_repository_row(row)?;
    }
    for row in &raw.local_checkout {
        strict_checkout_row(row)?;
    }
    for row in &raw.monorepo_component {
        strict_component_row(row)?;
    }
    for row in &raw.project_repository_link {
        strict_link_row(row)?;
    }
    Ok(target)
}

fn sort_unique_rows<T>(rows: &mut [C2DecodedRow<T>]) -> C2Result<()> {
    rows.sort_unstable_by_key(|row| *row.id.as_uuid().as_bytes());
    if rows.windows(2).any(|pair| pair[0].id == pair[1].id) {
        return Err(C2SemanticError::InconsistentProjection);
    }
    Ok(())
}

fn decode_snapshot(raw: C2RawSnapshot) -> C2Result<C2DecodedSnapshot> {
    let mut memory_items = raw
        .memory_item
        .into_iter()
        .map(decode_memory_row)
        .collect::<C2Result<Vec<_>>>()?;
    let mut correction_proposals = raw
        .correction_proposal
        .into_iter()
        .map(decode_proposal_row)
        .collect::<C2Result<Vec<_>>>()?;
    let mut projects = raw
        .work_project
        .into_iter()
        .map(decode_project_row)
        .collect::<C2Result<Vec<_>>>()?;
    let mut tasks = raw
        .work_task
        .into_iter()
        .map(decode_task_row)
        .collect::<C2Result<Vec<_>>>()?;
    let mut repositories = raw
        .git_repository
        .into_iter()
        .map(decode_repository_row)
        .collect::<C2Result<Vec<_>>>()?;
    let mut checkouts = raw
        .local_checkout
        .into_iter()
        .map(decode_checkout_row)
        .collect::<C2Result<Vec<_>>>()?;
    let mut components = raw
        .monorepo_component
        .into_iter()
        .map(decode_component_row)
        .collect::<C2Result<Vec<_>>>()?;
    let mut project_links = raw
        .project_repository_link
        .into_iter()
        .map(decode_link_row)
        .collect::<C2Result<Vec<_>>>()?;

    sort_unique_rows(&mut memory_items)?;
    sort_unique_rows(&mut correction_proposals)?;
    sort_unique_rows(&mut projects)?;
    sort_unique_rows(&mut tasks)?;
    sort_unique_rows(&mut repositories)?;
    sort_unique_rows(&mut checkouts)?;
    sort_unique_rows(&mut components)?;
    sort_unique_rows(&mut project_links)?;

    Ok(C2DecodedSnapshot {
        memory_items,
        correction_proposals,
        projects,
        tasks,
        repositories,
        checkouts,
        components,
        project_links,
    })
}

fn id_bytes(id: &Id) -> [u8; 16] {
    *id.as_uuid().as_bytes()
}

fn row_by_id<'a, T>(rows: &'a [C2DecodedRow<T>], id: &Id) -> Option<&'a C2DecodedRow<T>> {
    let key = id_bytes(id);
    rows.binary_search_by_key(&key, |row| id_bytes(&row.id))
        .ok()
        .map(|index| &rows[index])
}

fn validate_logical_uniqueness(snapshot: &C2DecodedSnapshot) -> C2Result<()> {
    let mut project_names = BTreeSet::new();
    for project in &snapshot.projects {
        if !project_names.insert(project.value.name.to_ascii_lowercase()) {
            return Err(C2SemanticError::InconsistentProjection);
        }
    }

    let mut task_selectors = BTreeMap::<([u8; 16], String), [u8; 16]>::new();
    let mut folded_task_selectors = BTreeMap::<([u8; 16], String), [u8; 16]>::new();
    for task in &snapshot.tasks {
        let project_id = id_bytes(&task.value.project_id);
        let task_id = id_bytes(&task.id);
        let mut selectors = vec![task.value.name.as_str()];
        if let Some(jira_key) = task.value.jira_key.as_deref() {
            selectors.push(jira_key);
        }
        for selector in selectors {
            let exact_key = (project_id, selector.to_string());
            if task_selectors
                .insert(exact_key, task_id)
                .is_some_and(|existing| existing != task_id)
            {
                return Err(C2SemanticError::InconsistentProjection);
            }
            let folded_key = (project_id, selector.to_ascii_lowercase());
            if folded_task_selectors
                .insert(folded_key, task_id)
                .is_some_and(|existing| existing != task_id)
            {
                return Err(C2SemanticError::InconsistentProjection);
            }
        }
    }

    let mut remotes = BTreeSet::new();
    for repository in &snapshot.repositories {
        if let Some(remote) = repository.value.remote_url.as_deref() {
            if !remotes.insert(normalize_remote(remote)?) {
                return Err(C2SemanticError::InconsistentProjection);
            }
        }
    }

    let mut checkout_paths = BTreeSet::new();
    for checkout in &snapshot.checkouts {
        if !checkout_paths.insert(checkout.value.local_path.as_str()) {
            return Err(C2SemanticError::InconsistentProjection);
        }
    }

    let mut component_paths = BTreeSet::new();
    for component in &snapshot.components {
        if !component_paths.insert((
            id_bytes(&component.value.repository_id),
            component.value.path.as_str(),
        )) {
            return Err(C2SemanticError::InconsistentProjection);
        }
    }

    let mut proposal_pairs = BTreeSet::new();
    let mut obsolete_ids = BTreeSet::new();
    let mut replacement_ids = BTreeSet::new();
    for proposal in &snapshot.correction_proposals {
        let obsolete_id = id_bytes(&proposal.value.obsolete_id);
        let replacement_id = id_bytes(&proposal.value.replacement_id);
        if !proposal_pairs.insert((obsolete_id, replacement_id))
            || !obsolete_ids.insert(obsolete_id)
            || !replacement_ids.insert(replacement_id)
        {
            return Err(C2SemanticError::InconsistentProjection);
        }
    }
    if obsolete_ids
        .iter()
        .any(|obsolete_id| replacement_ids.contains(obsolete_id))
    {
        return Err(C2SemanticError::InconsistentProjection);
    }
    Ok(())
}

fn graph_is_acyclic(
    nodes: impl Iterator<Item = [u8; 16]>,
    edges: &BTreeMap<[u8; 16], Vec<[u8; 16]>>,
) -> bool {
    let mut indegree = BTreeMap::new();
    for node in nodes {
        indegree.insert(node, 0_usize);
    }
    for targets in edges.values() {
        for target in targets {
            let Some(degree) = indegree.get_mut(target) else {
                return false;
            };
            *degree += 1;
        }
    }
    let mut ready = indegree
        .iter()
        .filter_map(|(node, degree)| (*degree == 0).then_some(*node))
        .collect::<Vec<_>>();
    let mut visited = 0_usize;
    while let Some(node) = ready.pop() {
        visited += 1;
        if let Some(targets) = edges.get(&node) {
            for target in targets {
                let degree = indegree
                    .get_mut(target)
                    .expect("validated graph target must have an indegree entry");
                *degree -= 1;
                if *degree == 0 {
                    ready.push(*target);
                }
            }
        }
    }
    visited == indegree.len()
}

fn correction_application_target(proposal: &CorrectionProposal) -> String {
    format!("memory.apply_correction:{}", proposal.id)
}

fn correction_application_summary(proposal: &CorrectionProposal) -> String {
    format!(
        "Operator-selected correction proposal {} applied replacement {} over {}",
        proposal.id, proposal.replacement_id, proposal.obsolete_id
    )
}

fn has_manual_review_evidence(item: &MemoryItem) -> bool {
    item.evidence
        .iter()
        .any(|evidence| evidence.kind == EvidenceKind::ManualReview)
}

fn validate_pending_pair(
    proposal: &CorrectionProposal,
    obsolete: &MemoryItem,
    replacement: &MemoryItem,
) -> C2Result<()> {
    let application_target = correction_application_target(proposal);
    if proposal.status != CorrectionProposalStatus::Pending
        || proposal.obsolete_id != obsolete.id
        || proposal.replacement_id != replacement.id
        || proposal.memory_kind != obsolete.kind
        || proposal.memory_kind != replacement.kind
        || proposal.scope != obsolete.scope
        || proposal.scope != replacement.scope
        || !serialized_equal(&proposal.proposer, &replacement.writer)?
        || obsolete.status != MemoryStatus::Active
        || obsolete.pending_correction_proposal_id != Some(proposal.id)
        || obsolete.correction_proposal_id.is_some()
        || replacement.status != MemoryStatus::NeedsReview
        || replacement.origin != ClaimOrigin::AgentInferred
        || replacement.kind == MemoryKind::Procedure
        || replacement.procedure.is_some()
        || replacement.correction_proposal_id != Some(proposal.id)
        || replacement.pending_correction_proposal_id.is_some()
        || replacement.writer.actor != "agent"
        || replacement.confidence.value() != obsolete.confidence.value()
        || replacement.tags != obsolete.tags
        || replacement.review_after != obsolete.review_after
        || replacement.archive.is_some()
        || replacement.last_used_at.is_some()
        || !replacement.supersedes.is_empty()
        || replacement.evidence.is_empty()
        || has_manual_review_evidence(replacement)
        || obsolete
            .evidence
            .iter()
            .chain(&replacement.evidence)
            .any(|evidence| evidence.target == application_target)
    {
        return Err(C2SemanticError::InconsistentProjection);
    }
    let digest = correction_digest(proposal, obsolete, replacement)?;
    if !constant_time_equal(&digest, &proposal.canonical_digest) {
        return Err(C2SemanticError::InconsistentProjection);
    }
    Ok(())
}

fn applied_evidence<'a>(
    proposal: &CorrectionProposal,
    obsolete: &'a MemoryItem,
    replacement: &'a MemoryItem,
) -> C2Result<&'a EvidenceRef> {
    let obsolete_evidence = obsolete
        .evidence
        .last()
        .ok_or(C2SemanticError::InconsistentProjection)?;
    let replacement_evidence = replacement
        .evidence
        .last()
        .ok_or(C2SemanticError::InconsistentProjection)?;
    let target = correction_application_target(proposal);
    if obsolete_evidence != replacement_evidence
        || obsolete_evidence.kind != EvidenceKind::ToolCall
        || obsolete_evidence.target != target
        || obsolete_evidence.summary.as_deref()
            != Some(correction_application_summary(proposal).as_str())
        || obsolete_evidence.excerpt.is_some()
        || obsolete.evidence[..obsolete.evidence.len() - 1]
            .iter()
            .chain(&replacement.evidence[..replacement.evidence.len() - 1])
            .any(|evidence| evidence.target == target)
    {
        return Err(C2SemanticError::InconsistentProjection);
    }
    Ok(obsolete_evidence)
}

fn inverse_pending_pair(
    proposal: &CorrectionProposal,
    obsolete: &MemoryItem,
    replacement: &MemoryItem,
) -> (CorrectionProposal, MemoryItem, MemoryItem) {
    let mut pending_proposal = proposal.clone();
    pending_proposal.status = CorrectionProposalStatus::Pending;
    pending_proposal.applied_at = None;
    pending_proposal.applied_digest = None;

    let mut pending_obsolete = obsolete.clone();
    pending_obsolete.status = MemoryStatus::Active;
    pending_obsolete.pending_correction_proposal_id = Some(proposal.id);
    pending_obsolete.evidence.pop();

    let mut pending_replacement = replacement.clone();
    pending_replacement.status = MemoryStatus::NeedsReview;
    pending_replacement.supersedes.clear();
    pending_replacement.correction_proposal_id = Some(proposal.id);
    pending_replacement.evidence.pop();

    (pending_proposal, pending_obsolete, pending_replacement)
}

fn validate_applied_pair(
    proposal: &CorrectionProposal,
    obsolete: &MemoryItem,
    replacement: &MemoryItem,
) -> C2Result<()> {
    if proposal.status != CorrectionProposalStatus::Applied
        || proposal.obsolete_id != obsolete.id
        || proposal.replacement_id != replacement.id
        || proposal.memory_kind != obsolete.kind
        || proposal.memory_kind != replacement.kind
        || proposal.scope != obsolete.scope
        || proposal.scope != replacement.scope
        || !serialized_equal(&proposal.proposer, &replacement.writer)?
        || replacement.status != MemoryStatus::Active
        || replacement.origin != ClaimOrigin::AgentInferred
        || replacement.correction_proposal_id.is_some()
        || replacement.pending_correction_proposal_id.is_some()
        || replacement.supersedes.as_slice() != [proposal.obsolete_id]
        || has_manual_review_evidence(replacement)
        || obsolete.status != MemoryStatus::Superseded
        || obsolete.correction_proposal_id.is_some()
        || obsolete.pending_correction_proposal_id.is_some()
    {
        return Err(C2SemanticError::InconsistentProjection);
    }
    applied_evidence(proposal, obsolete, replacement)?;
    let digest = correction_digest(proposal, obsolete, replacement)?;
    if !proposal
        .applied_digest
        .as_deref()
        .is_some_and(|expected| constant_time_equal(&digest, expected))
    {
        return Err(C2SemanticError::InconsistentProjection);
    }

    let (pending_proposal, pending_obsolete, pending_replacement) =
        inverse_pending_pair(proposal, obsolete, replacement);
    validate_pending_pair(&pending_proposal, &pending_obsolete, &pending_replacement)
}

fn validate_correction_graph(snapshot: &C2DecodedSnapshot) -> C2Result<()> {
    for proposal in &snapshot.correction_proposals {
        let obsolete = row_by_id(&snapshot.memory_items, &proposal.value.obsolete_id)
            .ok_or(C2SemanticError::InconsistentProjection)?;
        let replacement = row_by_id(&snapshot.memory_items, &proposal.value.replacement_id)
            .ok_or(C2SemanticError::InconsistentProjection)?;
        match proposal.value.status {
            CorrectionProposalStatus::Pending => {
                validate_pending_pair(&proposal.value, &obsolete.value, &replacement.value)?;
            }
            CorrectionProposalStatus::Applied => {
                validate_applied_pair(&proposal.value, &obsolete.value, &replacement.value)?;
            }
        }
    }

    for item in &snapshot.memory_items {
        if let Some(proposal_id) = &item.value.correction_proposal_id {
            let proposal = row_by_id(&snapshot.correction_proposals, proposal_id)
                .ok_or(C2SemanticError::InconsistentProjection)?;
            if proposal.value.status != CorrectionProposalStatus::Pending
                || proposal.value.replacement_id != item.id
            {
                return Err(C2SemanticError::InconsistentProjection);
            }
        }
        if let Some(proposal_id) = &item.value.pending_correction_proposal_id {
            let proposal = row_by_id(&snapshot.correction_proposals, proposal_id)
                .ok_or(C2SemanticError::InconsistentProjection)?;
            if proposal.value.status != CorrectionProposalStatus::Pending
                || proposal.value.obsolete_id != item.id
            {
                return Err(C2SemanticError::InconsistentProjection);
            }
        }
    }
    Ok(())
}

fn project_by_name<'a>(
    projects: &'a [C2DecodedRow<Project>],
    name: &str,
) -> Option<&'a C2DecodedRow<Project>> {
    let mut matches = projects.iter().filter(|project| project.value.name == name);
    let project = matches.next()?;
    matches.next().is_none().then_some(project)
}

fn validate_task_references(snapshot: &C2DecodedSnapshot) -> C2Result<()> {
    let mut edges = BTreeMap::new();
    for task in &snapshot.tasks {
        if row_by_id(&snapshot.projects, &task.value.project_id).is_none() {
            return Err(C2SemanticError::InconsistentProjection);
        }
        let mut blockers = BTreeSet::new();
        let mut task_edges = Vec::new();
        for blocker_id in &task.value.blocked_by {
            if blocker_id == &task.id || !blockers.insert(id_bytes(blocker_id)) {
                return Err(C2SemanticError::InconsistentProjection);
            }
            let blocker = row_by_id(&snapshot.tasks, blocker_id)
                .ok_or(C2SemanticError::InconsistentProjection)?;
            if blocker.value.project_id != task.value.project_id {
                return Err(C2SemanticError::InconsistentProjection);
            }
            task_edges.push(id_bytes(blocker_id));
        }
        edges.insert(id_bytes(&task.id), task_edges);
    }
    if !graph_is_acyclic(snapshot.tasks.iter().map(|task| id_bytes(&task.id)), &edges) {
        return Err(C2SemanticError::InconsistentProjection);
    }
    Ok(())
}

fn validate_topology_references(snapshot: &C2DecodedSnapshot) -> C2Result<()> {
    for checkout in &snapshot.checkouts {
        if checkout
            .value
            .repository_id
            .as_ref()
            .is_some_and(|id| row_by_id(&snapshot.repositories, id).is_none())
        {
            return Err(C2SemanticError::InconsistentProjection);
        }
    }
    for component in &snapshot.components {
        if row_by_id(&snapshot.repositories, &component.value.repository_id).is_none() {
            return Err(C2SemanticError::InconsistentProjection);
        }
    }

    let mut link_identities = BTreeSet::new();
    for link in &snapshot.project_links {
        let project = project_by_name(&snapshot.projects, &link.value.project_name)
            .ok_or(C2SemanticError::InconsistentProjection)?;
        if link
            .value
            .project_id
            .as_ref()
            .is_some_and(|project_id| project_id != &project.id)
            || row_by_id(&snapshot.repositories, &link.value.repository_id).is_none()
        {
            return Err(C2SemanticError::InconsistentProjection);
        }
        if let (Some(component_id), Some(component_path)) =
            (&link.value.component_id, &link.value.component_path)
        {
            let component = row_by_id(&snapshot.components, component_id)
                .ok_or(C2SemanticError::InconsistentProjection)?;
            if component.value.repository_id != link.value.repository_id
                || component.value.path != *component_path
            {
                return Err(C2SemanticError::InconsistentProjection);
            }
        }
        if !link_identities.insert((
            id_bytes(&project.id),
            id_bytes(&link.value.repository_id),
            link.value.component_id.as_ref().map(id_bytes),
        )) {
            return Err(C2SemanticError::InconsistentProjection);
        }
    }
    Ok(())
}

fn validate_supersedes_references(snapshot: &C2DecodedSnapshot) -> C2Result<()> {
    let mut incoming = BTreeSet::new();
    let mut edges = BTreeMap::new();
    for item in &snapshot.memory_items {
        let mut item_targets = BTreeSet::new();
        let mut item_edges = Vec::new();
        for target_id in &item.value.supersedes {
            let target_key = id_bytes(target_id);
            if target_id == &item.id
                || !item_targets.insert(target_key)
                || !incoming.insert(target_key)
            {
                return Err(C2SemanticError::InconsistentProjection);
            }
            let target = row_by_id(&snapshot.memory_items, target_id)
                .ok_or(C2SemanticError::InconsistentProjection)?;
            if target.value.status != MemoryStatus::Superseded {
                return Err(C2SemanticError::InconsistentProjection);
            }
            item_edges.push(target_key);
        }
        edges.insert(id_bytes(&item.id), item_edges);
    }
    if !graph_is_acyclic(
        snapshot.memory_items.iter().map(|item| id_bytes(&item.id)),
        &edges,
    ) {
        return Err(C2SemanticError::InconsistentProjection);
    }
    Ok(())
}

fn validate_whole_store_references(snapshot: &C2DecodedSnapshot) -> C2Result<()> {
    validate_task_references(snapshot)?;
    validate_topology_references(snapshot)?;
    validate_supersedes_references(snapshot)
}

enum C2ScopeReferencePreflight {
    Resolved,
    DeferredScopeMismatch,
}

fn scope_touches_raw_project_target(
    project_id: Option<&Id>,
    project_name: Option<&str>,
    selector: &C2TargetSelector,
) -> bool {
    project_id == Some(&selector.project_id) || project_name == Some(selector.project_name.as_str())
}

fn preflight_project_scope(
    snapshot: &C2DecodedSnapshot,
    project_id: Option<&Id>,
    project_name: &str,
    selector: &C2TargetSelector,
) -> C2Result<C2ScopeReferencePreflight> {
    let named = project_by_name(&snapshot.projects, project_name);
    let identified = project_id.and_then(|id| row_by_id(&snapshot.projects, id));
    if named.is_some()
        && (project_id.is_none() || identified.map(|row| row.id) == named.map(|row| row.id))
    {
        return Ok(C2ScopeReferencePreflight::Resolved);
    }
    if scope_touches_raw_project_target(project_id, Some(project_name), selector) {
        Ok(C2ScopeReferencePreflight::DeferredScopeMismatch)
    } else {
        Err(C2SemanticError::InconsistentProjection)
    }
}

fn preflight_task_scope(
    snapshot: &C2DecodedSnapshot,
    project_id: Option<&Id>,
    project_name: Option<&str>,
    task_id: Option<&Id>,
    task_name: &str,
    selector: &C2TargetSelector,
) -> C2Result<C2ScopeReferencePreflight> {
    let named_project = project_name.and_then(|name| project_by_name(&snapshot.projects, name));
    let identified_project = project_id.and_then(|id| row_by_id(&snapshot.projects, id));
    let project_selectors_resolve = project_name.map_or(true, |_| named_project.is_some())
        && project_id.map_or(true, |_| identified_project.is_some())
        && match (named_project, identified_project) {
            (Some(named), Some(identified)) => named.id == identified.id,
            _ => true,
        };
    let candidate_count = snapshot
        .tasks
        .iter()
        .filter(|task| {
            task_matches_selector(&task.value, task_name)
                && task_id.map_or(true, |id| task.id == *id)
                && project_id.map_or(true, |id| task.value.project_id == *id)
                && named_project.map_or(true, |project| task.value.project_id == project.id)
        })
        .take(2)
        .count();
    if candidate_count > 1 {
        return Err(C2SemanticError::InconsistentProjection);
    }
    if candidate_count == 1 && project_selectors_resolve {
        return Ok(C2ScopeReferencePreflight::Resolved);
    }

    let target_task_is_present = selector.task_id.is_some() && selector.task_name.is_some();
    let selected_task_matches_name = selector
        .task_id
        .as_ref()
        .and_then(|id| row_by_id(&snapshot.tasks, id))
        .is_some_and(|task| task_matches_selector(&task.value, task_name));
    let touches_target = target_task_is_present
        && (scope_touches_raw_project_target(project_id, project_name, selector)
            || task_id == selector.task_id.as_ref()
            || selector.task_name.as_deref() == Some(task_name)
            || selected_task_matches_name);
    if touches_target {
        Ok(C2ScopeReferencePreflight::DeferredScopeMismatch)
    } else {
        Err(C2SemanticError::InconsistentProjection)
    }
}

fn unique_repository_for_normalized_remote(
    snapshot: &C2DecodedSnapshot,
    normalized_remote: &str,
) -> C2Result<Option<Id>> {
    let mut found = None;
    for repository in &snapshot.repositories {
        let Some(remote) = repository.value.remote_url.as_deref() else {
            continue;
        };
        if normalize_remote(remote)? != normalized_remote {
            continue;
        }
        if found.replace(repository.id).is_some() {
            return Err(C2SemanticError::InconsistentProjection);
        }
    }
    Ok(found)
}

fn unique_checkout_repository_for_path(
    snapshot: &C2DecodedSnapshot,
    path: &str,
) -> C2Result<Option<Id>> {
    let mut found = None;
    for checkout in &snapshot.checkouts {
        if checkout.value.local_path != path {
            continue;
        }
        let Some(repository_id) = checkout.value.repository_id else {
            return Ok(None);
        };
        if found.replace(repository_id).is_some() {
            return Err(C2SemanticError::InconsistentProjection);
        }
    }
    Ok(found)
}

fn preflight_repository_scope(
    snapshot: &C2DecodedSnapshot,
    repository_id: Option<&Id>,
    remote_url: Option<&str>,
    local_path: Option<&str>,
    selector: &C2TargetSelector,
) -> C2Result<C2ScopeReferencePreflight> {
    if repository_id.is_none() && remote_url.is_none() && local_path.is_none() {
        return Ok(C2ScopeReferencePreflight::DeferredScopeMismatch);
    }
    let normalized_remote = remote_url.map(normalize_remote).transpose()?;
    let mut resolved_repository_ids = BTreeSet::new();
    let mut unresolved = false;
    if let Some(repository_id) = repository_id {
        if row_by_id(&snapshot.repositories, repository_id).is_some() {
            resolved_repository_ids.insert(id_bytes(repository_id));
        } else {
            unresolved = true;
        }
    }
    if let Some(remote) = normalized_remote.as_deref() {
        if let Some(repository_id) = unique_repository_for_normalized_remote(snapshot, remote)? {
            resolved_repository_ids.insert(id_bytes(&repository_id));
        } else {
            unresolved = true;
        }
    }
    if let Some(path) = local_path {
        if let Some(repository_id) = unique_checkout_repository_for_path(snapshot, path)? {
            resolved_repository_ids.insert(id_bytes(&repository_id));
        } else {
            unresolved = true;
        }
    }
    let touches_target = repository_id == Some(&selector.repository_id)
        || normalized_remote.as_deref() == Some(selector.normalized_repository_remote.as_str())
        || local_path == Some(selector.checkout_path.as_str());
    if resolved_repository_ids.len() > 1 {
        return Ok(C2ScopeReferencePreflight::DeferredScopeMismatch);
    }
    if unresolved {
        return if touches_target {
            Ok(C2ScopeReferencePreflight::DeferredScopeMismatch)
        } else {
            Err(C2SemanticError::InconsistentProjection)
        };
    }
    if resolved_repository_ids.len() == 1 {
        Ok(C2ScopeReferencePreflight::Resolved)
    } else {
        Err(C2SemanticError::InconsistentProjection)
    }
}

fn preflight_scope_reference(
    snapshot: &C2DecodedSnapshot,
    scope: &MemoryScope,
    selector: &C2TargetSelector,
) -> C2Result<C2ScopeReferencePreflight> {
    match scope {
        MemoryScope::Project {
            project_id,
            project_name,
        } => preflight_project_scope(snapshot, project_id.as_ref(), project_name, selector),
        MemoryScope::Task {
            project_id,
            project_name,
            task_id,
            task_name,
        } => preflight_task_scope(
            snapshot,
            project_id.as_ref(),
            project_name.as_deref(),
            task_id.as_ref(),
            task_name,
            selector,
        ),
        MemoryScope::Repository {
            repository_id,
            remote_url,
            local_path,
        } => preflight_repository_scope(
            snapshot,
            repository_id.as_ref(),
            remote_url.as_deref(),
            local_path.as_deref(),
            selector,
        ),
        MemoryScope::Global
        | MemoryScope::User
        | MemoryScope::Entity { .. }
        | MemoryScope::Session { .. }
        | MemoryScope::Custom { .. } => Ok(C2ScopeReferencePreflight::Resolved),
    }
}

fn validate_whole_store_scope_references(
    snapshot: &C2DecodedSnapshot,
    selector: &C2TargetSelector,
) -> C2Result<()> {
    for item in &snapshot.memory_items {
        let _ = preflight_scope_reference(snapshot, &item.value.scope, selector)?;
    }
    for proposal in &snapshot.correction_proposals {
        let _ = preflight_scope_reference(snapshot, &proposal.value.scope, selector)?;
    }
    Ok(())
}

fn task_matches_selector(task: &Task, selector: &str) -> bool {
    task.name == selector || task.jira_key.as_deref() == Some(selector)
}

fn resolve_target(
    snapshot: &C2DecodedSnapshot,
    selector: &C2TargetSelector,
) -> C2Result<C2ResolvedTarget> {
    let project = row_by_id(&snapshot.projects, &selector.project_id)
        .filter(|project| project.value.name == selector.project_name)
        .ok_or(C2SemanticError::AmbiguousIdentity)?;
    let mut primary_links = Vec::new();
    for link in &snapshot.project_links {
        let touches_id = link.value.project_id == Some(selector.project_id);
        let touches_name = link.value.project_name == selector.project_name;
        if touches_id || touches_name {
            if !touches_id || !touches_name {
                return Err(C2SemanticError::AmbiguousIdentity);
            }
            if link.value.role == ProjectRepositoryRole::Primary {
                primary_links.push(link);
            }
        }
    }
    if primary_links.len() != 1 {
        return Err(C2SemanticError::AmbiguousIdentity);
    }
    let primary = primary_links[0];
    if primary.value.component_id.is_some()
        || primary.value.component_path.is_some()
        || primary.value.repository_id != selector.repository_id
    {
        return Err(C2SemanticError::AmbiguousIdentity);
    }

    let repository = row_by_id(&snapshot.repositories, &selector.repository_id)
        .ok_or(C2SemanticError::AmbiguousIdentity)?;
    let remote_matches = repository
        .value
        .remote_url
        .as_deref()
        .map(normalize_remote)
        .transpose()?
        .as_deref()
        == Some(selector.normalized_repository_remote.as_str());
    if !remote_matches {
        return Err(C2SemanticError::AmbiguousIdentity);
    }
    let checkout = row_by_id(&snapshot.checkouts, &selector.checkout_id)
        .filter(|checkout| {
            checkout.value.repository_id == Some(repository.id)
                && checkout.value.local_path == selector.checkout_path
        })
        .ok_or(C2SemanticError::AmbiguousIdentity)?;

    let task_id = match (&selector.task_id, &selector.task_name) {
        (Some(task_id), Some(task_name)) => Some(
            row_by_id(&snapshot.tasks, task_id)
                .filter(|task| {
                    task.value.project_id == project.id
                        && task_matches_selector(&task.value, task_name)
                })
                .ok_or(C2SemanticError::AmbiguousIdentity)?
                .id,
        ),
        (None, None) => None,
        _ => return Err(C2SemanticError::AmbiguousIdentity),
    };
    Ok(C2ResolvedTarget {
        project_id: project.id,
        repository_id: repository.id,
        checkout_id: checkout.id,
        task_id,
    })
}

fn scope_project<'a>(
    snapshot: &'a C2DecodedSnapshot,
    project_id: Option<&Id>,
    project_name: &str,
    target: &C2ResolvedTarget,
) -> C2Result<&'a C2DecodedRow<Project>> {
    let named = project_by_name(&snapshot.projects, project_name);
    let identified = project_id.and_then(|id| row_by_id(&snapshot.projects, id));
    let touches_target = project_id == Some(&target.project_id)
        || named.is_some_and(|project| project.id == target.project_id);
    let Some(project) = named else {
        return Err(if touches_target {
            C2SemanticError::ScopeMismatch
        } else {
            C2SemanticError::InconsistentProjection
        });
    };
    if project_id.is_some() && identified.map(|row| row.id) != Some(project.id) {
        return Err(if touches_target {
            C2SemanticError::ScopeMismatch
        } else {
            C2SemanticError::InconsistentProjection
        });
    }
    Ok(project)
}

fn scope_task<'a>(
    snapshot: &'a C2DecodedSnapshot,
    project_id: Option<&Id>,
    project_name: Option<&str>,
    task_id: Option<&Id>,
    task_name: &str,
    target: &C2ResolvedTarget,
) -> C2Result<&'a C2DecodedRow<Task>> {
    let named_project = project_name.and_then(|name| project_by_name(&snapshot.projects, name));
    let touches_target = project_id == Some(&target.project_id)
        || target
            .task_id
            .as_ref()
            .is_some_and(|target_task_id| task_id == Some(target_task_id))
        || named_project.is_some_and(|project| project.id == target.project_id)
        || target
            .task_id
            .as_ref()
            .and_then(|target_task_id| row_by_id(&snapshot.tasks, target_task_id))
            .is_some_and(|task| task_matches_selector(&task.value, task_name));
    if project_name.is_some() && named_project.is_none() {
        return Err(if touches_target {
            C2SemanticError::ScopeMismatch
        } else {
            C2SemanticError::InconsistentProjection
        });
    }
    let mut candidates = snapshot.tasks.iter().filter(|task| {
        task_matches_selector(&task.value, task_name)
            && task_id.map_or(true, |id| task.id == *id)
            && project_id.map_or(true, |id| task.value.project_id == *id)
            && named_project.map_or(true, |project| task.value.project_id == project.id)
    });
    let task = candidates.next();
    let second = candidates.next();
    if task.is_none() || second.is_some() {
        return Err(if touches_target {
            C2SemanticError::ScopeMismatch
        } else {
            C2SemanticError::InconsistentProjection
        });
    }
    let task = task.expect("unique task candidate was checked");
    let explicit_target_context = project_id == Some(&target.project_id)
        || named_project.is_some_and(|project| project.id == target.project_id)
        || target
            .task_id
            .as_ref()
            .is_some_and(|target_task_id| task_id == Some(target_task_id));
    if target
        .task_id
        .as_ref()
        .is_some_and(|target_task_id| explicit_target_context && task.id != *target_task_id)
    {
        return Err(C2SemanticError::ScopeMismatch);
    }
    Ok(task)
}

fn scope_repository<'a>(
    snapshot: &'a C2DecodedSnapshot,
    repository_id: Option<&Id>,
    remote_url: Option<&str>,
    local_path: Option<&str>,
    target: &C2ResolvedTarget,
    selector: &C2TargetSelector,
) -> C2Result<&'a C2DecodedRow<GitRepository>> {
    if repository_id.is_none() && remote_url.is_none() && local_path.is_none() {
        return Err(C2SemanticError::ScopeMismatch);
    }
    let normalized_remote = remote_url.map(normalize_remote).transpose()?;
    let touches_target = repository_id == Some(&target.repository_id)
        || normalized_remote.as_deref() == Some(selector.normalized_repository_remote.as_str())
        || local_path == Some(selector.checkout_path.as_str());
    let by_id = repository_id.and_then(|id| row_by_id(&snapshot.repositories, id));
    let by_remote = normalized_remote.as_deref().and_then(|remote| {
        let mut matches = snapshot.repositories.iter().filter(|repository| {
            repository
                .value
                .remote_url
                .as_deref()
                .and_then(|stored| normalize_remote(stored).ok())
                .as_deref()
                == Some(remote)
        });
        let repository = matches.next();
        (matches.next().is_none()).then_some(repository).flatten()
    });
    let by_path = local_path.and_then(|path| {
        snapshot
            .checkouts
            .iter()
            .find(|checkout| checkout.value.local_path == path)
            .and_then(|checkout| checkout.value.repository_id.as_ref())
            .and_then(|id| row_by_id(&snapshot.repositories, id))
    });
    let resolved_ids = [by_id, by_remote, by_path]
        .into_iter()
        .flatten()
        .map(|repository| id_bytes(&repository.id))
        .collect::<BTreeSet<_>>();
    if resolved_ids.len() > 1 {
        return Err(C2SemanticError::ScopeMismatch);
    }
    if (repository_id.is_some() && by_id.is_none())
        || (remote_url.is_some() && by_remote.is_none())
        || (local_path.is_some() && by_path.is_none())
    {
        return Err(if touches_target {
            C2SemanticError::ScopeMismatch
        } else {
            C2SemanticError::InconsistentProjection
        });
    }
    if resolved_ids.len() != 1 {
        return Err(C2SemanticError::ScopeMismatch);
    }
    let resolved_id = *resolved_ids
        .iter()
        .next()
        .expect("at least one repository selector is present");
    snapshot
        .repositories
        .binary_search_by_key(&resolved_id, |row| id_bytes(&row.id))
        .ok()
        .map(|index| &snapshot.repositories[index])
        .ok_or(C2SemanticError::InconsistentProjection)
}

fn scope_is_applicable(
    scope: &MemoryScope,
    snapshot: &C2DecodedSnapshot,
    target: &C2ResolvedTarget,
    selector: &C2TargetSelector,
) -> C2Result<bool> {
    match scope {
        MemoryScope::Global | MemoryScope::User => Ok(true),
        MemoryScope::Project {
            project_id,
            project_name,
        } => Ok(
            scope_project(snapshot, project_id.as_ref(), project_name, target)?.id
                == target.project_id,
        ),
        MemoryScope::Task {
            project_id,
            project_name,
            task_id,
            task_name,
        } => {
            let task = scope_task(
                snapshot,
                project_id.as_ref(),
                project_name.as_deref(),
                task_id.as_ref(),
                task_name,
                target,
            )?;
            let identifies_target_project = project_id == &Some(target.project_id)
                || project_name
                    .as_deref()
                    .and_then(|name| project_by_name(&snapshot.projects, name))
                    .is_some_and(|project| project.id == target.project_id);
            Ok(target.task_id == Some(task.id)
                && (task_id == &Some(task.id) || identifies_target_project))
        }
        MemoryScope::Repository {
            repository_id,
            remote_url,
            local_path,
        } => {
            let repository = scope_repository(
                snapshot,
                repository_id.as_ref(),
                remote_url.as_deref(),
                local_path.as_deref(),
                target,
                selector,
            )?;
            let path_matches = local_path
                .as_deref()
                .map_or(true, |path| path == selector.checkout_path);
            if repository.id == target.repository_id && !path_matches {
                return Err(C2SemanticError::ScopeMismatch);
            }
            Ok(repository.id == target.repository_id && path_matches)
        }
        MemoryScope::Entity { .. } | MemoryScope::Session { .. } | MemoryScope::Custom { .. } => {
            Ok(false)
        }
    }
}

struct C2ScopeProjection {
    applicable_memory_ids: Vec<Id>,
    target_affecting_correction_ids: Vec<Id>,
}

fn project_scopes(
    snapshot: &C2DecodedSnapshot,
    target: &C2ResolvedTarget,
    selector: &C2TargetSelector,
) -> C2Result<C2ScopeProjection> {
    let mut applicability = BTreeMap::new();
    let mut applicable_memory_ids = Vec::new();
    for item in &snapshot.memory_items {
        let applies = scope_is_applicable(&item.value.scope, snapshot, target, selector)?;
        applicability.insert(id_bytes(&item.id), applies);
        if applies && item.value.status == MemoryStatus::Active {
            applicable_memory_ids.push(item.id);
        }
    }

    let mut target_affecting_correction_ids = Vec::new();
    for proposal in &snapshot.correction_proposals {
        let obsolete_applies = applicability
            .get(&id_bytes(&proposal.value.obsolete_id))
            .copied()
            .ok_or(C2SemanticError::InconsistentProjection)?;
        let replacement_applies = applicability
            .get(&id_bytes(&proposal.value.replacement_id))
            .copied()
            .ok_or(C2SemanticError::InconsistentProjection)?;
        if obsolete_applies
            && replacement_applies
            && scope_is_applicable(&proposal.value.scope, snapshot, target, selector)?
        {
            target_affecting_correction_ids.push(proposal.id);
        }
    }
    Ok(C2ScopeProjection {
        applicable_memory_ids,
        target_affecting_correction_ids,
    })
}

fn authenticate_rows<T>(
    engine: &C2MacEngine,
    table: C2TableTag,
    rows: &mut [C2DecodedRow<T>],
) -> C2Result<Vec<C2RecordMac>> {
    let mut records = Vec::with_capacity(rows.len());
    for row in rows {
        let mac = engine.record_mac(table, id_bytes(&row.id), &row.raw)?;
        row.record_mac = Some(mac.0);
        records.push(C2RecordMac {
            id: id_bytes(&row.id),
            mac,
        });
    }
    Ok(records)
}

fn decoded_record<T>(row: &C2DecodedRow<T>) -> C2RecordMac {
    C2RecordMac {
        id: id_bytes(&row.id),
        mac: C2Mac(
            row.record_mac
                .expect("record MAC must be populated before view framing"),
        ),
    }
}

fn count_rows(rows: usize) -> C2Result<u64> {
    u64::try_from(rows).map_err(|_| C2SemanticError::LimitExceeded {
        table_overflow_mask: None,
    })
}

fn build_pending_bindings(
    snapshot: &C2DecodedSnapshot,
    target_affecting_ids: &[Id],
) -> C2Result<Vec<C2PriorCorrectionBinding>> {
    let target_affecting = target_affecting_ids
        .iter()
        .map(id_bytes)
        .collect::<BTreeSet<_>>();
    let mut bindings = Vec::new();
    for proposal in &snapshot.correction_proposals {
        if proposal.value.status != CorrectionProposalStatus::Pending
            || !target_affecting.contains(&id_bytes(&proposal.id))
        {
            continue;
        }
        let obsolete = row_by_id(&snapshot.memory_items, &proposal.value.obsolete_id)
            .ok_or(C2SemanticError::InconsistentProjection)?;
        let replacement = row_by_id(&snapshot.memory_items, &proposal.value.replacement_id)
            .ok_or(C2SemanticError::InconsistentProjection)?;
        bindings.push(C2PriorCorrectionBinding {
            proposal_id: proposal.id,
            proposal_row: proposal.raw.clone(),
            obsolete_row: obsolete.raw.clone(),
            replacement_row: replacement.raw.clone(),
            proposal: proposal.value.clone(),
            obsolete: obsolete.value.clone(),
            replacement: replacement.value.clone(),
            proposal_record_mac: proposal
                .record_mac
                .expect("proposal MAC must be populated before binding creation"),
            obsolete_record_mac: obsolete
                .record_mac
                .expect("memory MAC must be populated before binding creation"),
            replacement_record_mac: replacement
                .record_mac
                .expect("memory MAC must be populated before binding creation"),
        });
    }
    Ok(bindings)
}

fn build_audit(
    snapshot: &mut C2DecodedSnapshot,
    target: &C2ResolvedTarget,
    selector: &C2TargetSelector,
    scope: &C2ScopeProjection,
    mac_key: C2OneShotMacKey,
) -> C2Result<C2SanitizedAudit> {
    let engine = C2MacEngine::consume(mac_key);
    let memory_records =
        authenticate_rows(&engine, C2TableTag::MemoryItem, &mut snapshot.memory_items)?;
    let proposal_records = authenticate_rows(
        &engine,
        C2TableTag::CorrectionProposal,
        &mut snapshot.correction_proposals,
    )?;
    let project_records =
        authenticate_rows(&engine, C2TableTag::WorkProject, &mut snapshot.projects)?;
    let task_records = authenticate_rows(&engine, C2TableTag::WorkTask, &mut snapshot.tasks)?;
    let repository_records = authenticate_rows(
        &engine,
        C2TableTag::GitRepository,
        &mut snapshot.repositories,
    )?;
    let checkout_records =
        authenticate_rows(&engine, C2TableTag::LocalCheckout, &mut snapshot.checkouts)?;
    let component_records = authenticate_rows(
        &engine,
        C2TableTag::MonorepoComponent,
        &mut snapshot.components,
    )?;
    let link_records = authenticate_rows(
        &engine,
        C2TableTag::ProjectRepositoryLink,
        &mut snapshot.project_links,
    )?;
    let table_mac_values = [
        engine.table_mac(C2TableTag::MemoryItem, &memory_records)?,
        engine.table_mac(C2TableTag::CorrectionProposal, &proposal_records)?,
        engine.table_mac(C2TableTag::MemoryForgetReceipt, &[])?,
        engine.table_mac(C2TableTag::WorkProject, &project_records)?,
        engine.table_mac(C2TableTag::WorkTask, &task_records)?,
        engine.table_mac(C2TableTag::GitRepository, &repository_records)?,
        engine.table_mac(C2TableTag::LocalCheckout, &checkout_records)?,
        engine.table_mac(C2TableTag::MonorepoComponent, &component_records)?,
        engine.table_mac(C2TableTag::ProjectRepositoryLink, &link_records)?,
    ];
    let store_state_mac = engine.store_mac(&table_mac_values)?;

    let competing_links = snapshot
        .project_links
        .iter()
        .filter(|link| {
            link.value.project_id == Some(target.project_id)
                || link.value.project_name == selector.project_name
        })
        .map(decoded_record)
        .collect::<Vec<_>>();
    let mut linked_component_ids = BTreeSet::new();
    for link in &snapshot.project_links {
        if link.value.project_id == Some(target.project_id)
            || link.value.project_name == selector.project_name
        {
            if let Some(component_id) = &link.value.component_id {
                linked_component_ids.insert(id_bytes(component_id));
            }
        }
    }
    let linked_components = snapshot
        .components
        .iter()
        .filter(|component| linked_component_ids.contains(&id_bytes(&component.id)))
        .map(decoded_record)
        .collect::<Vec<_>>();

    let affecting_ids = scope
        .target_affecting_correction_ids
        .iter()
        .map(id_bytes)
        .collect::<BTreeSet<_>>();
    let mut correction_edges = Vec::new();
    let mut audit_edges = Vec::new();
    for proposal in &snapshot.correction_proposals {
        if !affecting_ids.contains(&id_bytes(&proposal.id)) {
            continue;
        }
        let obsolete = row_by_id(&snapshot.memory_items, &proposal.value.obsolete_id)
            .ok_or(C2SemanticError::InconsistentProjection)?;
        let replacement = row_by_id(&snapshot.memory_items, &proposal.value.replacement_id)
            .ok_or(C2SemanticError::InconsistentProjection)?;
        let proposal_record = decoded_record(proposal);
        let obsolete_record = decoded_record(obsolete);
        let replacement_record = decoded_record(replacement);
        let (status, status_text) = match proposal.value.status {
            CorrectionProposalStatus::Pending => (C2CorrectionStatus::Pending, "pending"),
            CorrectionProposalStatus::Applied => (C2CorrectionStatus::Applied, "applied"),
        };
        correction_edges.push(C2CorrectionEdgeMac {
            proposal_id: proposal_record.id,
            obsolete_id: obsolete_record.id,
            replacement_id: replacement_record.id,
            status,
            proposal_mac: proposal_record.mac,
            obsolete_mac: obsolete_record.mac,
            replacement_mac: replacement_record.mac,
        });
        audit_edges.push(C2AuditCorrectionEdge {
            proposal_id: proposal.id.to_string(),
            obsolete_id: obsolete.id.to_string(),
            replacement_id: replacement.id.to_string(),
            status: status_text.to_string(),
            proposal_record_mac: proposal_record.mac.to_lower_hex(),
            obsolete_record_mac: obsolete_record.mac.to_lower_hex(),
            replacement_record_mac: replacement_record.mac.to_lower_hex(),
        });
    }

    let applicable_ids = scope
        .applicable_memory_ids
        .iter()
        .map(id_bytes)
        .collect::<BTreeSet<_>>();
    let applicable_memories = snapshot
        .memory_items
        .iter()
        .filter(|item| applicable_ids.contains(&id_bytes(&item.id)))
        .map(decoded_record)
        .collect::<Vec<_>>();
    let relay_candidate_mac = engine.relay_candidates_mac(&applicable_memories)?;
    let project_view_mac = engine.project_view_mac(&C2ProjectViewFrame {
        target_project: decoded_record(
            row_by_id(&snapshot.projects, &target.project_id)
                .expect("resolved target project must remain present"),
        ),
        target_repository: decoded_record(
            row_by_id(&snapshot.repositories, &target.repository_id)
                .expect("resolved target repository must remain present"),
        ),
        target_checkout: decoded_record(
            row_by_id(&snapshot.checkouts, &target.checkout_id)
                .expect("resolved target checkout must remain present"),
        ),
        target_task: target.task_id.as_ref().map(|task_id| {
            decoded_record(
                row_by_id(&snapshot.tasks, task_id)
                    .expect("resolved target task must remain present"),
            )
        }),
        competing_links: &competing_links,
        linked_components: &linked_components,
        correction_edges: &correction_edges,
        applicable_memories: &applicable_memories,
    })?;

    let mut memory_status_counts = C2MemoryStatusCounts {
        active: 0,
        needs_review: 0,
        superseded: 0,
        archived: 0,
        rejected: 0,
    };
    for item in &snapshot.memory_items {
        let count = match item.value.status {
            MemoryStatus::Active => &mut memory_status_counts.active,
            MemoryStatus::NeedsReview => &mut memory_status_counts.needs_review,
            MemoryStatus::Superseded => &mut memory_status_counts.superseded,
            MemoryStatus::Archived => &mut memory_status_counts.archived,
            MemoryStatus::Rejected => &mut memory_status_counts.rejected,
        };
        *count = count.checked_add(1).ok_or(C2SemanticError::LimitExceeded {
            table_overflow_mask: None,
        })?;
    }
    let mut proposal_status_counts = C2ProposalStatusCounts {
        pending: 0,
        applied: 0,
    };
    for proposal in &snapshot.correction_proposals {
        let count = match proposal.value.status {
            CorrectionProposalStatus::Pending => &mut proposal_status_counts.pending,
            CorrectionProposalStatus::Applied => &mut proposal_status_counts.applied,
        };
        *count = count.checked_add(1).ok_or(C2SemanticError::LimitExceeded {
            table_overflow_mask: None,
        })?;
    }
    let relay_candidates = snapshot
        .memory_items
        .iter()
        .filter(|item| applicable_ids.contains(&id_bytes(&item.id)))
        .map(|item| C2AuditRelayCandidate {
            memory_id: item.id.to_string(),
            status: item.value.status.to_string(),
            record_mac: decoded_record(item).mac.to_lower_hex(),
        })
        .collect();
    Ok(C2SanitizedAudit {
        schema_version: 1,
        target: C2AuditTarget {
            project_id: target.project_id.to_string(),
            repository_id: target.repository_id.to_string(),
            checkout_id: target.checkout_id.to_string(),
            task_id: target.task_id.map(|id| id.to_string()),
        },
        row_counts: C2RowCounts {
            memory_item: count_rows(snapshot.memory_items.len())?,
            correction_proposal: count_rows(snapshot.correction_proposals.len())?,
            memory_forget_receipt: 0,
            work_project: count_rows(snapshot.projects.len())?,
            work_task: count_rows(snapshot.tasks.len())?,
            git_repository: count_rows(snapshot.repositories.len())?,
            local_checkout: count_rows(snapshot.checkouts.len())?,
            monorepo_component: count_rows(snapshot.components.len())?,
            project_repository_link: count_rows(snapshot.project_links.len())?,
        },
        memory_status_counts,
        proposal_status_counts,
        table_macs: C2TableMacs {
            memory_item: table_mac_values[0].to_lower_hex(),
            correction_proposal: table_mac_values[1].to_lower_hex(),
            memory_forget_receipt: table_mac_values[2].to_lower_hex(),
            work_project: table_mac_values[3].to_lower_hex(),
            work_task: table_mac_values[4].to_lower_hex(),
            git_repository: table_mac_values[5].to_lower_hex(),
            local_checkout: table_mac_values[6].to_lower_hex(),
            monorepo_component: table_mac_values[7].to_lower_hex(),
            project_repository_link: table_mac_values[8].to_lower_hex(),
        },
        store_state_mac: store_state_mac.to_lower_hex(),
        project_view_mac: project_view_mac.to_lower_hex(),
        relay_candidate_mac: relay_candidate_mac.to_lower_hex(),
        correction_edges: audit_edges,
        relay_candidates,
    })
}

fn binding_failure<T>() -> C2Result<T> {
    Err(C2SemanticError::AppliedProvenanceUnproven)
}

fn reverse_proposal_row(
    current_row: &Value,
    pending_proposal: &CorrectionProposal,
) -> C2Result<Value> {
    let mut reversed = current_row.clone();
    let row = reversed
        .as_object_mut()
        .ok_or(C2SemanticError::AppliedProvenanceUnproven)?;
    let proposal = row
        .get_mut("proposal")
        .and_then(Value::as_object_mut)
        .ok_or(C2SemanticError::AppliedProvenanceUnproven)?;
    proposal.remove("applied_digest");
    proposal.insert("status".to_string(), Value::String("pending".to_string()));
    proposal.insert("applied_at".to_string(), Value::Null);
    row.insert(
        "status_key".to_string(),
        Value::String("pending".to_string()),
    );
    row.insert(
        "pending_obsolete_id".to_string(),
        Value::String(pending_proposal.obsolete_id.to_string()),
    );
    row.insert(
        "snapshot_digest".to_string(),
        Value::String(
            snapshot_digest(pending_proposal)
                .map_err(|_| C2SemanticError::AppliedProvenanceUnproven)?,
        ),
    );
    Ok(reversed)
}

fn reverse_memory_row(
    current_row: &Value,
    pending_item: &MemoryItem,
    pending_status: MemoryStatus,
    proposal_id: &Id,
    replacement: bool,
    bound_row: &Value,
) -> C2Result<Value> {
    let bound_updated_at = bound_row
        .as_object()
        .and_then(|row| row.get("updated_at"))
        .cloned()
        .ok_or(C2SemanticError::AppliedProvenanceUnproven)?;
    let mut reversed = current_row.clone();
    let row = reversed
        .as_object_mut()
        .ok_or(C2SemanticError::AppliedProvenanceUnproven)?;
    let item = row
        .get_mut("item")
        .and_then(Value::as_object_mut)
        .ok_or(C2SemanticError::AppliedProvenanceUnproven)?;
    let evidence = item
        .get_mut("evidence")
        .and_then(Value::as_array_mut)
        .ok_or(C2SemanticError::AppliedProvenanceUnproven)?;
    evidence
        .pop()
        .ok_or(C2SemanticError::AppliedProvenanceUnproven)?;
    item.insert(
        "status".to_string(),
        Value::String(pending_status.to_string()),
    );
    if replacement {
        item.insert("supersedes".to_string(), Value::Array(Vec::new()));
        item.insert(
            "correction_proposal_id".to_string(),
            Value::String(proposal_id.to_string()),
        );
    } else {
        item.insert(
            "pending_correction_proposal_id".to_string(),
            Value::String(proposal_id.to_string()),
        );
    }
    item.insert("updated_at".to_string(), bound_updated_at.clone());
    row.insert(
        "status_key".to_string(),
        Value::String(pending_status.to_string()),
    );
    row.insert("updated_at".to_string(), bound_updated_at);
    row.insert(
        "snapshot_digest".to_string(),
        Value::String(
            snapshot_digest(pending_item)
                .map_err(|_| C2SemanticError::AppliedProvenanceUnproven)?,
        ),
    );
    Ok(reversed)
}

fn binding_rows_equal(left: &Value, right: &Value) -> C2Result<bool> {
    let left =
        canonical_json_value(left).map_err(|_| C2SemanticError::AppliedProvenanceUnproven)?;
    let right =
        canonical_json_value(right).map_err(|_| C2SemanticError::AppliedProvenanceUnproven)?;
    Ok(left == right)
}

fn validate_applied_transition(
    proposal: &C2DecodedRow<CorrectionProposal>,
    obsolete: &C2DecodedRow<MemoryItem>,
    replacement: &C2DecodedRow<MemoryItem>,
    binding: &C2PriorCorrectionBinding,
) -> C2Result<()> {
    let evidence = applied_evidence(&proposal.value, &obsolete.value, &replacement.value)
        .map_err(|_| C2SemanticError::AppliedProvenanceUnproven)?;
    let applied_at = proposal
        .value
        .applied_at
        .ok_or(C2SemanticError::AppliedProvenanceUnproven)?;
    if binding.proposal_id != proposal.id
        || binding.proposal.id != proposal.id
        || binding.obsolete.id != obsolete.id
        || binding.replacement.id != replacement.id
        || binding.replacement.updated_at > evidence.observed_at
        || binding.obsolete.updated_at > evidence.observed_at
        || evidence.observed_at > replacement.value.updated_at
        || evidence.observed_at > obsolete.value.updated_at
        || replacement.value.updated_at > applied_at
        || obsolete.value.updated_at > applied_at
    {
        return binding_failure();
    }

    let mut pending_proposal = proposal.value.clone();
    pending_proposal.status = CorrectionProposalStatus::Pending;
    pending_proposal.applied_digest = None;
    pending_proposal.applied_at = None;
    let mut pending_replacement = replacement.value.clone();
    pending_replacement.status = MemoryStatus::NeedsReview;
    pending_replacement.supersedes.clear();
    pending_replacement.correction_proposal_id = Some(proposal.id);
    pending_replacement.evidence.pop();
    pending_replacement.updated_at = binding.replacement.updated_at;
    let mut pending_obsolete = obsolete.value.clone();
    pending_obsolete.status = MemoryStatus::Active;
    pending_obsolete.pending_correction_proposal_id = Some(proposal.id);
    pending_obsolete.evidence.pop();
    pending_obsolete.updated_at = binding.obsolete.updated_at;

    if !serialized_equal(&pending_proposal, &binding.proposal)
        .map_err(|_| C2SemanticError::AppliedProvenanceUnproven)?
        || !serialized_equal(&pending_replacement, &binding.replacement)
            .map_err(|_| C2SemanticError::AppliedProvenanceUnproven)?
        || !serialized_equal(&pending_obsolete, &binding.obsolete)
            .map_err(|_| C2SemanticError::AppliedProvenanceUnproven)?
    {
        return binding_failure();
    }

    let reversed_proposal = reverse_proposal_row(&proposal.raw, &pending_proposal)?;
    let reversed_replacement = reverse_memory_row(
        &replacement.raw,
        &pending_replacement,
        MemoryStatus::NeedsReview,
        &proposal.id,
        true,
        &binding.replacement_row,
    )?;
    let reversed_obsolete = reverse_memory_row(
        &obsolete.raw,
        &pending_obsolete,
        MemoryStatus::Active,
        &proposal.id,
        false,
        &binding.obsolete_row,
    )?;
    if !binding_rows_equal(&reversed_proposal, &binding.proposal_row)?
        || !binding_rows_equal(&reversed_replacement, &binding.replacement_row)?
        || !binding_rows_equal(&reversed_obsolete, &binding.obsolete_row)?
    {
        return binding_failure();
    }

    let inverse_digest =
        correction_digest(&pending_proposal, &pending_obsolete, &pending_replacement)
            .map_err(|_| C2SemanticError::AppliedProvenanceUnproven)?;
    let applied_digest = correction_digest(&proposal.value, &obsolete.value, &replacement.value)
        .map_err(|_| C2SemanticError::AppliedProvenanceUnproven)?;
    if !constant_time_equal(&inverse_digest, &proposal.value.canonical_digest)
        || !constant_time_equal(&inverse_digest, &binding.proposal.canonical_digest)
        || !proposal
            .value
            .applied_digest
            .as_deref()
            .is_some_and(|expected| constant_time_equal(&applied_digest, expected))
    {
        return binding_failure();
    }
    Ok(())
}

fn validate_applied_bindings(
    snapshot: &C2DecodedSnapshot,
    target_affecting_ids: &[Id],
    prior_bindings: &[C2PriorCorrectionBinding],
) -> C2Result<()> {
    let mut previous_binding_id = None;
    for binding in prior_bindings {
        let binding_id = id_bytes(&binding.proposal_id);
        if previous_binding_id.is_some_and(|previous| previous >= binding_id) {
            return binding_failure();
        }
        previous_binding_id = Some(binding_id);

        let proposal = strict_proposal_row(&binding.proposal_row)
            .map_err(|_| C2SemanticError::AppliedProvenanceUnproven)?;
        let obsolete = strict_memory_row(&binding.obsolete_row)
            .map_err(|_| C2SemanticError::AppliedProvenanceUnproven)?;
        let replacement = strict_memory_row(&binding.replacement_row)
            .map_err(|_| C2SemanticError::AppliedProvenanceUnproven)?;
        if proposal.id != binding.proposal_id
            || proposal.status != CorrectionProposalStatus::Pending
            || !serialized_equal(&proposal, &binding.proposal)
                .map_err(|_| C2SemanticError::AppliedProvenanceUnproven)?
            || !serialized_equal(&obsolete, &binding.obsolete)
                .map_err(|_| C2SemanticError::AppliedProvenanceUnproven)?
            || !serialized_equal(&replacement, &binding.replacement)
                .map_err(|_| C2SemanticError::AppliedProvenanceUnproven)?
        {
            return binding_failure();
        }
    }

    let applied_ids = target_affecting_ids
        .iter()
        .filter(|proposal_id| {
            row_by_id(&snapshot.correction_proposals, proposal_id)
                .is_some_and(|proposal| proposal.value.status == CorrectionProposalStatus::Applied)
        })
        .map(id_bytes)
        .collect::<Vec<_>>();
    let binding_ids = prior_bindings
        .iter()
        .map(|binding| id_bytes(&binding.proposal_id))
        .collect::<Vec<_>>();
    if applied_ids != binding_ids {
        return binding_failure();
    }
    for (proposal_id, binding) in applied_ids.iter().zip(prior_bindings) {
        let proposal = snapshot
            .correction_proposals
            .binary_search_by_key(proposal_id, |row| id_bytes(&row.id))
            .ok()
            .map(|index| &snapshot.correction_proposals[index])
            .ok_or(C2SemanticError::AppliedProvenanceUnproven)?;
        let obsolete = row_by_id(&snapshot.memory_items, &proposal.value.obsolete_id)
            .ok_or(C2SemanticError::AppliedProvenanceUnproven)?;
        let replacement = row_by_id(&snapshot.memory_items, &proposal.value.replacement_id)
            .ok_or(C2SemanticError::AppliedProvenanceUnproven)?;
        validate_applied_transition(proposal, obsolete, replacement, binding)?;
    }
    Ok(())
}

#[allow(dead_code)]
fn validate_semantic_snapshot(
    raw: C2RawSnapshot,
    prior_bindings: Vec<C2PriorCorrectionBinding>,
    mac_key: C2OneShotMacKey,
) -> Result<C2ValidatedSnapshot, C2SemanticError> {
    let boundary = validate_limit_and_secret_boundary(&raw, &prior_bindings)?;
    let selector = strict_record_stage(&raw, &prior_bindings)?;
    enforce_receipt_precedence_after_strict_records(&boundary, Ok(()))?;
    let _accounting = boundary.accounting;
    let mut snapshot = decode_snapshot(raw)?;
    validate_logical_uniqueness(&snapshot)?;
    validate_correction_graph(&snapshot)?;
    validate_whole_store_references(&snapshot)?;
    validate_whole_store_scope_references(&snapshot, &selector)?;
    let target = resolve_target(&snapshot, &selector)?;
    let scope = project_scopes(&snapshot, &target, &selector)?;
    validate_applied_bindings(
        &snapshot,
        &scope.target_affecting_correction_ids,
        &prior_bindings,
    )?;
    let audit = build_audit(&mut snapshot, &target, &selector, &scope, mac_key)?;
    let pending_bindings =
        build_pending_bindings(&snapshot, &scope.target_affecting_correction_ids)?;

    Ok(C2ValidatedSnapshot {
        memory_items: snapshot
            .memory_items
            .into_iter()
            .map(|row| row.value)
            .collect(),
        correction_proposals: snapshot
            .correction_proposals
            .into_iter()
            .map(|row| row.value)
            .collect(),
        projects: snapshot.projects.into_iter().map(|row| row.value).collect(),
        tasks: snapshot.tasks.into_iter().map(|row| row.value).collect(),
        repositories: snapshot
            .repositories
            .into_iter()
            .map(|row| row.value)
            .collect(),
        checkouts: snapshot
            .checkouts
            .into_iter()
            .map(|row| row.value)
            .collect(),
        components: snapshot
            .components
            .into_iter()
            .map(|row| row.value)
            .collect(),
        project_links: snapshot
            .project_links
            .into_iter()
            .map(|row| row.value)
            .collect(),
        target,
        applicable_memory_ids: scope.applicable_memory_ids,
        target_affecting_correction_ids: scope.target_affecting_correction_ids,
        pending_bindings,
        audit,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const OLD_ITEM_ID: &str = "01890f5e-7b00-7000-8000-000000000002";
    const PROPOSAL_ID: &str = "01890f5e-7b00-7000-8000-000000000001";
    const REPLACEMENT_ITEM_ID: &str = "01890f5e-7b00-7000-8000-000000000003";
    const PROJECT_ID: &str = "01890f5e-7b00-7000-8000-000000000010";
    const REPOSITORY_ID: &str = "01890f5e-7b00-7000-8000-000000000011";
    const CHECKOUT_ID: &str = "01890f5e-7b00-7000-8000-000000000012";
    const TASK_ID: &str = "01890f5e-7b00-7000-8000-000000000013";
    const PRIMARY_LINK_ID: &str = "01890f5e-7b00-7000-8000-000000000014";
    const GLOBAL_MEMORY_ID: &str = "01890f5e-7b00-7000-8000-000000000021";
    const PROJECT_MEMORY_ID: &str = "01890f5e-7b00-7000-8000-000000000022";
    const TASK_MEMORY_ID: &str = "01890f5e-7b00-7000-8000-000000000023";
    const OUT_OF_SCOPE_MEMORY_ID: &str = "01890f5e-7b00-7000-8000-000000000024";
    const FIXTURE_TIMESTAMP: &str = "2026-09-06T00:00:00Z";
    const FIXTURE_REMOTE: &str = "https://github.com/ymeiri/engram.git";
    const FIXTURE_CHECKOUT_PATH: &str = "/workspace/engram";
    const OLD_ITEM_SNAPSHOT_DIGEST: &str =
        "c5ad92ecdd3a719b61cfca6ef91430a354132c8761c793e5c34e01b6578e3e51";
    const MEMORY_ITEM_GOLDEN: &str = r#"{"id":"01890f5e-7b00-7000-8000-000000000002","kind":"decision","title":"old decision","content":"old content","scope":{"type":"project","project_id":null,"project_name":"engram"},"origin":"user_stated","writer":{"harness":"codex","harness_version":null,"model":{"provider":"openai","model":"fixture","version":null},"surface":null,"actor":"agent","session_id":null,"written_at":"2026-09-06T00:00:00Z"},"evidence":[],"confidence":0.8,"status":"active","supersedes":[],"tags":[],"created_at":"2026-09-06T00:00:00Z","updated_at":"2026-09-06T00:00:00Z","last_used_at":null,"review_after":null,"archive":null,"procedure":null,"pending_correction_proposal_id":"01890f5e-7b00-7000-8000-000000000001"}"#;
    const PENDING_PROPOSAL_GOLDEN: &str = r#"{"id":"01890f5e-7b00-7000-8000-000000000001","obsolete_id":"01890f5e-7b00-7000-8000-000000000002","replacement_id":"01890f5e-7b00-7000-8000-000000000003","memory_kind":"decision","scope":{"type":"project","project_id":null,"project_name":"engram"},"canonical_digest":"ebd471b14169c74083751c4b8ce6fbf44e27f1613c6c34c9b7fb6cf926232a01","digest_schema_version":1,"status":"pending","proposer":{"harness":"codex","harness_version":null,"model":{"provider":"openai","model":"fixture","version":null},"surface":null,"actor":"agent","session_id":null,"written_at":"2026-09-06T00:00:00Z"},"created_at":"2026-09-06T00:00:00Z","applied_at":null}"#;
    const PENDING_REPLACEMENT_GOLDEN: &str = r#"{"id":"01890f5e-7b00-7000-8000-000000000003","kind":"decision","title":"new decision","content":"new content","scope":{"type":"project","project_id":null,"project_name":"engram"},"origin":"agent_inferred","writer":{"harness":"codex","harness_version":null,"model":{"provider":"openai","model":"fixture","version":null},"surface":null,"actor":"agent","session_id":null,"written_at":"2026-09-06T00:00:00Z"},"evidence":[{"kind":"file","target":"fixture.md","summary":"fixture","excerpt":null,"observed_at":"2026-09-06T00:00:00Z"}],"confidence":0.8,"status":"needs_review","supersedes":[],"tags":[],"created_at":"2026-09-06T00:00:00Z","updated_at":"2026-09-06T00:00:00Z","last_used_at":null,"review_after":null,"archive":null,"procedure":null,"correction_proposal_id":"01890f5e-7b00-7000-8000-000000000001"}"#;
    const CORRECTION_DIGEST_GOLDEN: &str = r#"{"schema_version":1,"proposal_id":"01890f5e-7b00-7000-8000-000000000001","obsolete_id":"01890f5e-7b00-7000-8000-000000000002","replacement_id":"01890f5e-7b00-7000-8000-000000000003","memory_kind":"decision","scope":{"type":"project","project_id":null,"project_name":"engram"},"obsolete":{"id":"01890f5e-7b00-7000-8000-000000000002","kind":"decision","title":"old decision","content":"old content","scope":{"type":"project","project_id":null,"project_name":"engram"},"origin":"user_stated","writer":{"harness":"codex","harness_version":null,"model":{"provider":"openai","model":"fixture","version":null},"surface":null,"actor":"agent","session_id":null,"written_at":"2026-09-06T00:00:00Z"},"evidence":[],"confidence":0.8,"status":"active","supersedes":[],"tags":[],"review_after":null,"archive":null,"procedure":null,"correction_proposal_id":null,"pending_correction_proposal_id":"01890f5e-7b00-7000-8000-000000000001"},"replacement":{"id":"01890f5e-7b00-7000-8000-000000000003","kind":"decision","title":"new decision","content":"new content","scope":{"type":"project","project_id":null,"project_name":"engram"},"origin":"agent_inferred","writer":{"harness":"codex","harness_version":null,"model":{"provider":"openai","model":"fixture","version":null},"surface":null,"actor":"agent","session_id":null,"written_at":"2026-09-06T00:00:00Z"},"evidence":[{"kind":"file","target":"fixture.md","summary":"fixture","excerpt":null,"observed_at":"2026-09-06T00:00:00Z"}],"confidence":0.8,"status":"needs_review","supersedes":[],"tags":[],"review_after":null,"archive":null,"procedure":null,"correction_proposal_id":"01890f5e-7b00-7000-8000-000000000001","pending_correction_proposal_id":null}}"#;
    const VALID_AUDIT_GOLDEN: &str = r#"{"schema_version":1,"target":{"project_id":"01890f5e-7b00-7000-8000-000000000010","repository_id":"01890f5e-7b00-7000-8000-000000000011","checkout_id":"01890f5e-7b00-7000-8000-000000000012","task_id":"01890f5e-7b00-7000-8000-000000000013"},"row_counts":{"memory_item":4,"correction_proposal":0,"memory_forget_receipt":0,"work_project":1,"work_task":1,"git_repository":1,"local_checkout":1,"monorepo_component":0,"project_repository_link":1},"memory_status_counts":{"active":4,"needs_review":0,"superseded":0,"archived":0,"rejected":0},"proposal_status_counts":{"pending":0,"applied":0},"table_macs":{"memory_item":"9d24931f7602e4f8b1faaa573ebd9d62f633b4c4a53b8190cad21b5adb031057","correction_proposal":"6f285f96b56926177576ba06d1b50e3dbf0887f5da9e31f00edabf763b7320d3","memory_forget_receipt":"303e29c9478974a053f2840074a62ed943ee5dd6e545956d5635b553a4fa4281","work_project":"e71fce0a10c74d3029a69c936ebbe78c110fbd0df184e74ee2894fcefaa31446","work_task":"3afa28ec6127d11bf5508675939c45dc4707b9d46e0a0819e65f6822c83eb5f4","git_repository":"fdd33660b9343636e61f7ce3e3ce6cf93f232655fda7128b94e8973946e03b92","local_checkout":"16a2c31e2b9bcd8da144a182c832d8d1a4089fbc42be5058c654120fffdf4324","monorepo_component":"c805857d8c1f5a6ec8d8076e2c3e30d3aa08928390feb5957b96af1aefe476f6","project_repository_link":"b96d20123fc5da2d74c3632e5bb72f481351880d49e3b2c2b48c60d1d717f6cc"},"store_state_mac":"34a6da35873c5cfc1c040590a362f35db5b776a5af8c628e4bdf67ca7b1fb1d8","project_view_mac":"9debea1176b4b90a8316e2e5c508c8efda53f7d479fc3755ac6bbdede3cbd5b5","relay_candidate_mac":"583cf3b69c9a59f8d3656516bf9821421abd3be8ded685b5a643a1bf75eedb82","correction_edges":[],"relay_candidates":[{"memory_id":"01890f5e-7b00-7000-8000-000000000021","status":"active","record_mac":"08952e0c51182aaae200bb7a1e666cdf2224e4a44e4683266220e0495e699b0b"},{"memory_id":"01890f5e-7b00-7000-8000-000000000022","status":"active","record_mac":"73898ddb2c77189856bce288e21e8d52243b9f9476b3fd6364d86dae2f3d490d"},{"memory_id":"01890f5e-7b00-7000-8000-000000000023","status":"active","record_mac":"9bf6c9a81bf3bbf1c13a6030f579f9192c693fd57d1ad972234e4be9b18c8800"}]}"#;

    fn lower_hex(bytes: &[u8]) -> String {
        let mut encoded = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            use std::fmt::Write as _;
            write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
        }
        encoded
    }

    fn uuid_bytes(value: &str) -> [u8; 16] {
        let id = parse_id(value).expect("fixture ID must be a canonical UUIDv7");
        *id.as_uuid().as_bytes()
    }

    fn repeated_record(id: &str, byte: u8) -> C2RecordMac {
        C2RecordMac {
            id: uuid_bytes(id),
            mac: C2Mac([byte; 32]),
        }
    }

    fn memory_item_row() -> Value {
        let item: Value = serde_json::from_str(MEMORY_ITEM_GOLDEN).unwrap();
        json!({
            "record_id": OLD_ITEM_ID,
            "item": item,
            "kind_key": "decision",
            "status_key": "active",
            "scope_key": "project:engram",
            "harness_key": "codex",
            "model_key": "fixture",
            "session_id": null,
            "snapshot_digest": OLD_ITEM_SNAPSHOT_DIGEST,
            "created_at": "2026-09-06T00:00:00Z",
            "updated_at": "2026-09-06T00:00:00Z"
        })
    }

    fn empty_snapshot() -> C2RawSnapshot {
        C2RawSnapshot {
            target: C2RawTarget {
                project_id: PROJECT_ID.to_owned(),
                project_name: "engram".to_owned(),
                repository_id: REPOSITORY_ID.to_owned(),
                repository_remote: "https://github.com/ymeiri/engram.git".to_owned(),
                checkout_id: CHECKOUT_ID.to_owned(),
                checkout_path: "/workspace/engram".to_owned(),
                task_id: None,
                task_name: None,
            },
            memory_item: Vec::new(),
            correction_proposal: Vec::new(),
            memory_forget_receipt: Vec::new(),
            work_project: Vec::new(),
            work_task: Vec::new(),
            git_repository: Vec::new(),
            local_checkout: Vec::new(),
            monorepo_component: Vec::new(),
            project_repository_link: Vec::new(),
        }
    }

    fn rows_mut(raw: &mut C2RawSnapshot, table: TableKind) -> &mut Vec<Value> {
        match table {
            TableKind::MemoryItem => &mut raw.memory_item,
            TableKind::CorrectionProposal => &mut raw.correction_proposal,
            TableKind::MemoryForgetReceipt => &mut raw.memory_forget_receipt,
            TableKind::WorkProject => &mut raw.work_project,
            TableKind::WorkTask => &mut raw.work_task,
            TableKind::GitRepository => &mut raw.git_repository,
            TableKind::LocalCheckout => &mut raw.local_checkout,
            TableKind::MonorepoComponent => &mut raw.monorepo_component,
            TableKind::ProjectRepositoryLink => &mut raw.project_repository_link,
        }
    }

    fn assert_limit_none<T>(result: C2Result<T>) {
        assert!(matches!(
            result,
            Err(C2SemanticError::LimitExceeded {
                table_overflow_mask: None
            })
        ));
    }

    fn memory_row_with_item_field(name: &str, value: Value) -> Value {
        let mut item = Map::new();
        item.insert(name.to_owned(), value);
        let mut row = Map::new();
        row.insert("item".to_owned(), Value::Object(item));
        Value::Object(row)
    }

    fn nested_array(levels: usize) -> Value {
        let mut value = Value::Null;
        for _ in 0..levels {
            value = Value::Array(vec![value]);
        }
        value
    }

    fn fixture_writer_value() -> Value {
        json!({
            "harness": "codex",
            "harness_version": null,
            "model": {"provider": "openai", "model": "fixture", "version": null},
            "surface": null,
            "actor": "agent",
            "session_id": null,
            "written_at": FIXTURE_TIMESTAMP
        })
    }

    fn fixture_memory_item(id: &str, title: &str, scope: Value) -> Value {
        json!({
            "id": id,
            "kind": "decision",
            "title": title,
            "content": format!("content for {title}"),
            "scope": scope,
            "origin": "user_stated",
            "writer": fixture_writer_value(),
            "evidence": [],
            "confidence": 0.8,
            "status": "active",
            "supersedes": [],
            "tags": [],
            "created_at": FIXTURE_TIMESTAMP,
            "updated_at": FIXTURE_TIMESTAMP,
            "last_used_at": null,
            "review_after": null,
            "archive": null,
            "procedure": null
        })
    }

    fn fixture_memory_row(item: Value) -> Value {
        let typed: MemoryItem = serde_json::from_value(item.clone()).unwrap();
        let embedded = item.as_object().unwrap();
        let created_at = embedded.get("created_at").unwrap().clone();
        let updated_at = embedded.get("updated_at").unwrap().clone();
        json!({
            "record_id": typed.id.to_string(),
            "item": item,
            "kind_key": typed.kind.to_string(),
            "status_key": typed.status.to_string(),
            "scope_key": scope_key(&typed.scope),
            "harness_key": typed.writer.harness.to_string(),
            "model_key": typed.writer.model.model,
            "session_id": typed.writer.session_id,
            "snapshot_digest": snapshot_digest(&typed).unwrap(),
            "created_at": created_at,
            "updated_at": updated_at
        })
    }

    fn fixture_memory_rows() -> Vec<Value> {
        vec![
            fixture_memory_row(fixture_memory_item(
                GLOBAL_MEMORY_ID,
                "global fixture",
                json!({"type": "global"}),
            )),
            fixture_memory_row(fixture_memory_item(
                PROJECT_MEMORY_ID,
                "project fixture",
                json!({
                    "type": "project",
                    "project_id": PROJECT_ID,
                    "project_name": "engram"
                }),
            )),
            fixture_memory_row(fixture_memory_item(
                TASK_MEMORY_ID,
                "task fixture",
                json!({
                    "type": "task",
                    "project_id": PROJECT_ID,
                    "project_name": "engram",
                    "task_id": TASK_ID,
                    "task_name": "ENG-42"
                }),
            )),
            fixture_memory_row(fixture_memory_item(
                OUT_OF_SCOPE_MEMORY_ID,
                "entity fixture",
                json!({
                    "type": "entity",
                    "entity_id": null,
                    "entity_name": "unrelated entity"
                }),
            )),
        ]
    }

    fn valid_snapshot() -> C2RawSnapshot {
        let repository = json!({
            "id": REPOSITORY_ID,
            "name": "engram",
            "remote_url": FIXTURE_REMOTE,
            "provider": "git_hub",
            "default_branch": "main",
            "description": null,
            "created_at": FIXTURE_TIMESTAMP,
            "updated_at": FIXTURE_TIMESTAMP
        });
        let checkout = json!({
            "id": CHECKOUT_ID,
            "repository_id": REPOSITORY_ID,
            "local_path": FIXTURE_CHECKOUT_PATH,
            "current_branch": "main",
            "head_sha": "0123456789abcdef0123456789abcdef01234567",
            "is_dirty": false,
            "created_at": FIXTURE_TIMESTAMP,
            "updated_at": FIXTURE_TIMESTAMP,
            "last_seen_at": FIXTURE_TIMESTAMP
        });
        let link = json!({
            "id": PRIMARY_LINK_ID,
            "project_id": PROJECT_ID,
            "project_name": "engram",
            "repository_id": REPOSITORY_ID,
            "component_id": null,
            "component_path": null,
            "role": "primary",
            "created_at": FIXTURE_TIMESTAMP,
            "updated_at": FIXTURE_TIMESTAMP
        });
        C2RawSnapshot {
            target: C2RawTarget {
                project_id: PROJECT_ID.to_string(),
                project_name: "engram".to_string(),
                repository_id: REPOSITORY_ID.to_string(),
                repository_remote: FIXTURE_REMOTE.to_string(),
                checkout_id: CHECKOUT_ID.to_string(),
                checkout_path: FIXTURE_CHECKOUT_PATH.to_string(),
                task_id: Some(TASK_ID.to_string()),
                task_name: Some("ENG-42".to_string()),
            },
            memory_item: fixture_memory_rows(),
            correction_proposal: Vec::new(),
            memory_forget_receipt: Vec::new(),
            work_project: vec![json!({
                "record_id": PROJECT_ID,
                "name": "engram",
                "description": null,
                "status": "active",
                "created_at": FIXTURE_TIMESTAMP,
                "updated_at": FIXTURE_TIMESTAMP
            })],
            work_task: vec![json!({
                "record_id": TASK_ID,
                "project_id": PROJECT_ID,
                "name": "C2A semantic validator",
                "description": null,
                "status": "in_progress",
                "priority": "high",
                "jira_key": "ENG-42",
                "blocked_by": [],
                "created_at": FIXTURE_TIMESTAMP,
                "updated_at": FIXTURE_TIMESTAMP
            })],
            git_repository: vec![json!({
                "record_id": REPOSITORY_ID,
                "repository": repository,
                "name_key": "engram",
                "remote_url": FIXTURE_REMOTE,
                "provider_key": "github",
                "created_at": FIXTURE_TIMESTAMP,
                "updated_at": FIXTURE_TIMESTAMP
            })],
            local_checkout: vec![json!({
                "record_id": CHECKOUT_ID,
                "checkout": checkout,
                "repository_id": REPOSITORY_ID,
                "local_path_key": FIXTURE_CHECKOUT_PATH,
                "current_branch": "main",
                "head_sha": "0123456789abcdef0123456789abcdef01234567",
                "is_dirty": false,
                "created_at": FIXTURE_TIMESTAMP,
                "updated_at": FIXTURE_TIMESTAMP,
                "last_seen_at": FIXTURE_TIMESTAMP
            })],
            monorepo_component: Vec::new(),
            project_repository_link: vec![json!({
                "record_id": PRIMARY_LINK_ID,
                "link": link,
                "project_id": PROJECT_ID,
                "project_name_key": "engram",
                "repository_id": REPOSITORY_ID,
                "component_id": null,
                "component_path_key": null,
                "role": "primary",
                "created_at": FIXTURE_TIMESTAMP,
                "updated_at": FIXTURE_TIMESTAMP
            })],
        }
    }

    fn validate_fixture(raw: C2RawSnapshot) -> C2ValidatedSnapshot {
        validate_semantic_snapshot(
            raw,
            Vec::new(),
            C2OneShotMacKey::from_test_bytes([0x42; 32]),
        )
        .unwrap()
    }

    fn fixture_timestamp(value: &str) -> OffsetDateTime {
        OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339).unwrap()
    }

    fn fixture_proposal_row(proposal: Value) -> Value {
        let typed: CorrectionProposal = serde_json::from_value(proposal.clone()).unwrap();
        let created_at = proposal
            .as_object()
            .unwrap()
            .get("created_at")
            .unwrap()
            .clone();
        let mut row = json!({
            "record_id": typed.id.to_string(),
            "proposal": proposal,
            "status_key": typed.status.to_string(),
            "obsolete_id": typed.obsolete_id,
            "replacement_id": typed.replacement_id,
            "snapshot_digest": snapshot_digest(&typed).unwrap(),
            "created_at": created_at
        });
        if typed.status == CorrectionProposalStatus::Pending {
            row.as_object_mut().unwrap().insert(
                "pending_obsolete_id".to_string(),
                Value::String(typed.obsolete_id.to_string()),
            );
        }
        row
    }

    fn pending_correction_values(
        obsolete_updated_at: &str,
        replacement_updated_at: &str,
    ) -> (CorrectionProposal, MemoryItem, MemoryItem) {
        let proposal: CorrectionProposal = serde_json::from_str(PENDING_PROPOSAL_GOLDEN).unwrap();
        let mut obsolete: MemoryItem = serde_json::from_str(MEMORY_ITEM_GOLDEN).unwrap();
        let mut replacement: MemoryItem = serde_json::from_str(PENDING_REPLACEMENT_GOLDEN).unwrap();
        obsolete.updated_at = fixture_timestamp(obsolete_updated_at);
        replacement.updated_at = fixture_timestamp(replacement_updated_at);
        (proposal, obsolete, replacement)
    }

    fn correction_snapshot(
        proposal: &CorrectionProposal,
        obsolete: &MemoryItem,
        replacement: &MemoryItem,
    ) -> C2RawSnapshot {
        let mut raw = valid_snapshot();
        raw.memory_item = vec![
            fixture_memory_row(serde_json::to_value(obsolete).unwrap()),
            fixture_memory_row(serde_json::to_value(replacement).unwrap()),
        ];
        raw.correction_proposal = vec![fixture_proposal_row(
            serde_json::to_value(proposal).unwrap(),
        )];
        raw
    }

    fn pending_correction_snapshot(
        obsolete_updated_at: &str,
        replacement_updated_at: &str,
    ) -> C2RawSnapshot {
        let (proposal, obsolete, replacement) =
            pending_correction_values(obsolete_updated_at, replacement_updated_at);
        correction_snapshot(&proposal, &obsolete, &replacement)
    }

    fn applied_correction_snapshot(
        obsolete_prior_updated_at: &str,
        replacement_prior_updated_at: &str,
        evidence_at: &str,
        obsolete_updated_at: &str,
        replacement_updated_at: &str,
        applied_at: &str,
    ) -> C2RawSnapshot {
        let (mut proposal, mut obsolete, mut replacement) =
            pending_correction_values(obsolete_prior_updated_at, replacement_prior_updated_at);
        let evidence = EvidenceRef {
            kind: EvidenceKind::ToolCall,
            target: correction_application_target(&proposal),
            summary: Some(correction_application_summary(&proposal)),
            excerpt: None,
            observed_at: fixture_timestamp(evidence_at),
        };
        obsolete.status = MemoryStatus::Superseded;
        obsolete.pending_correction_proposal_id = None;
        obsolete.evidence.push(evidence.clone());
        obsolete.updated_at = fixture_timestamp(obsolete_updated_at);
        replacement.status = MemoryStatus::Active;
        replacement.correction_proposal_id = None;
        replacement.supersedes.push(obsolete.id);
        replacement.evidence.push(evidence);
        replacement.updated_at = fixture_timestamp(replacement_updated_at);
        proposal.status = CorrectionProposalStatus::Applied;
        proposal.applied_at = Some(fixture_timestamp(applied_at));
        proposal.applied_digest =
            Some(correction_digest(&proposal, &obsolete, &replacement).unwrap());
        correction_snapshot(&proposal, &obsolete, &replacement)
    }

    fn default_applied_correction_snapshot() -> C2RawSnapshot {
        applied_correction_snapshot(
            FIXTURE_TIMESTAMP,
            FIXTURE_TIMESTAMP,
            "2026-09-06T00:00:01Z",
            "2026-09-06T00:00:02Z",
            "2026-09-06T00:00:02Z",
            "2026-09-06T00:00:03Z",
        )
    }

    fn minted_pending_binding(
        obsolete_updated_at: &str,
        replacement_updated_at: &str,
    ) -> C2PriorCorrectionBinding {
        let validated = validate_semantic_snapshot(
            pending_correction_snapshot(obsolete_updated_at, replacement_updated_at),
            Vec::new(),
            C2OneShotMacKey::from_test_bytes([0x31; 32]),
        )
        .unwrap();
        let mut bindings = validated.pending_bindings.into_iter();
        let binding = bindings.next().unwrap();
        assert!(bindings.next().is_none());
        binding
    }

    fn applied_snapshot_with_immutable_title_change() -> C2RawSnapshot {
        let raw = default_applied_correction_snapshot();
        let mut proposal = strict_proposal_row(&raw.correction_proposal[0]).unwrap();
        let obsolete = strict_memory_row(&raw.memory_item[0]).unwrap();
        let mut replacement = strict_memory_row(&raw.memory_item[1]).unwrap();
        replacement.title = "changed after the pending binding".to_string();
        let (pending_proposal, pending_obsolete, pending_replacement) =
            inverse_pending_pair(&proposal, &obsolete, &replacement);
        proposal.canonical_digest =
            correction_digest(&pending_proposal, &pending_obsolete, &pending_replacement).unwrap();
        proposal.applied_digest =
            Some(correction_digest(&proposal, &obsolete, &replacement).unwrap());
        correction_snapshot(&proposal, &obsolete, &replacement)
    }

    fn push_scoped_memory(raw: &mut C2RawSnapshot, id: &str, scope: Value) {
        raw.memory_item.push(fixture_memory_row(fixture_memory_item(
            id,
            "scope precedence fixture",
            scope,
        )));
    }

    fn push_valid_receipt(raw: &mut C2RawSnapshot) {
        raw.memory_forget_receipt.push(json!({
            "record_id": "01890f5e-7b00-7000-8000-000000000070",
            "deleted": false,
            "proposal_ids": [],
            "pending_replacement_ids": [],
            "unlocked_obsolete_ids": [],
            "cleanup_complete": false,
            "created_at": FIXTURE_TIMESTAMP
        }));
    }

    fn push_fixture_project(raw: &mut C2RawSnapshot, id: &str, name: &str) {
        raw.work_project.push(json!({
            "record_id": id,
            "name": name,
            "description": null,
            "status": "active",
            "created_at": FIXTURE_TIMESTAMP,
            "updated_at": FIXTURE_TIMESTAMP
        }));
    }

    fn push_fixture_repository(raw: &mut C2RawSnapshot, id: &str, name: &str, remote: &str) {
        let repository = json!({
            "id": id,
            "name": name,
            "remote_url": remote,
            "provider": "git_hub",
            "default_branch": "main",
            "description": null,
            "created_at": FIXTURE_TIMESTAMP,
            "updated_at": FIXTURE_TIMESTAMP
        });
        raw.git_repository.push(json!({
            "record_id": id,
            "repository": repository,
            "name_key": name.to_lowercase(),
            "remote_url": remote,
            "provider_key": "github",
            "created_at": FIXTURE_TIMESTAMP,
            "updated_at": FIXTURE_TIMESTAMP
        }));
    }

    fn push_fixture_task(
        raw: &mut C2RawSnapshot,
        id: &str,
        project_id: &str,
        name: &str,
        jira_key: Option<&str>,
    ) {
        raw.work_task.push(json!({
            "record_id": id,
            "project_id": project_id,
            "name": name,
            "description": null,
            "status": "todo",
            "priority": "medium",
            "jira_key": jira_key,
            "blocked_by": [],
            "created_at": FIXTURE_TIMESTAMP,
            "updated_at": FIXTURE_TIMESTAMP
        }));
    }

    fn push_fixture_checkout(
        raw: &mut C2RawSnapshot,
        id: &str,
        repository_id: &str,
        local_path: &str,
    ) {
        let checkout = json!({
            "id": id,
            "repository_id": repository_id,
            "local_path": local_path,
            "current_branch": "main",
            "head_sha": "0123456789abcdef0123456789abcdef01234567",
            "is_dirty": false,
            "created_at": FIXTURE_TIMESTAMP,
            "updated_at": FIXTURE_TIMESTAMP,
            "last_seen_at": FIXTURE_TIMESTAMP
        });
        raw.local_checkout.push(json!({
            "record_id": id,
            "checkout": checkout,
            "repository_id": repository_id,
            "local_path_key": local_path,
            "current_branch": "main",
            "head_sha": "0123456789abcdef0123456789abcdef01234567",
            "is_dirty": false,
            "created_at": FIXTURE_TIMESTAMP,
            "updated_at": FIXTURE_TIMESTAMP,
            "last_seen_at": FIXTURE_TIMESTAMP
        }));
    }

    fn push_fixture_component(
        raw: &mut C2RawSnapshot,
        id: &str,
        repository_id: &str,
        name: &str,
        path: &str,
    ) {
        let component = json!({
            "id": id,
            "repository_id": repository_id,
            "name": name,
            "path": path,
            "kind": "crate",
            "description": null,
            "created_at": FIXTURE_TIMESTAMP,
            "updated_at": FIXTURE_TIMESTAMP
        });
        raw.monorepo_component.push(json!({
            "record_id": id,
            "component": component,
            "repository_id": repository_id,
            "name_key": name.to_lowercase(),
            "path_key": path,
            "kind": "crate",
            "created_at": FIXTURE_TIMESTAMP,
            "updated_at": FIXTURE_TIMESTAMP
        }));
    }

    fn push_fixture_link(
        raw: &mut C2RawSnapshot,
        id: &str,
        component_id: &str,
        component_path: &str,
        role: &str,
    ) {
        let link = json!({
            "id": id,
            "project_id": PROJECT_ID,
            "project_name": "engram",
            "repository_id": REPOSITORY_ID,
            "component_id": component_id,
            "component_path": component_path,
            "role": role,
            "created_at": FIXTURE_TIMESTAMP,
            "updated_at": FIXTURE_TIMESTAMP
        });
        raw.project_repository_link.push(json!({
            "record_id": id,
            "link": link,
            "project_id": PROJECT_ID,
            "project_name_key": "engram",
            "repository_id": REPOSITORY_ID,
            "component_id": component_id,
            "component_path_key": component_path,
            "role": role,
            "created_at": FIXTURE_TIMESTAMP,
            "updated_at": FIXTURE_TIMESTAMP
        }));
    }

    fn push_pending_correction(
        raw: &mut C2RawSnapshot,
        proposal_id: &str,
        obsolete_id: &str,
        replacement_id: &str,
    ) {
        let (mut proposal, mut obsolete, mut replacement) =
            pending_correction_values(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
        proposal.id = parse_id(proposal_id).unwrap();
        proposal.obsolete_id = parse_id(obsolete_id).unwrap();
        proposal.replacement_id = parse_id(replacement_id).unwrap();
        obsolete.id = proposal.obsolete_id;
        obsolete.pending_correction_proposal_id = Some(proposal.id);
        replacement.id = proposal.replacement_id;
        replacement.correction_proposal_id = Some(proposal.id);
        proposal.canonical_digest = correction_digest(&proposal, &obsolete, &replacement).unwrap();
        raw.memory_item
            .push(fixture_memory_row(serde_json::to_value(obsolete).unwrap()));
        raw.memory_item.push(fixture_memory_row(
            serde_json::to_value(replacement).unwrap(),
        ));
        raw.correction_proposal.push(fixture_proposal_row(
            serde_json::to_value(proposal).unwrap(),
        ));
    }

    fn rich_permutation_snapshot() -> C2RawSnapshot {
        const OTHER_PROJECT_ID: &str = "01890f5e-7b00-7000-8000-000000000060";
        const OTHER_REPOSITORY_ID: &str = "01890f5e-7b00-7000-8000-000000000061";
        const OTHER_TASK_ID: &str = "01890f5e-7b00-7000-8000-000000000063";
        const OTHER_CHECKOUT_ID: &str = "01890f5e-7b00-7000-8000-000000000064";
        const COMPONENT_A_ID: &str = "01890f5e-7b00-7000-8000-000000000065";
        const COMPONENT_B_ID: &str = "01890f5e-7b00-7000-8000-000000000066";
        let mut raw = valid_snapshot();
        push_fixture_project(&mut raw, OTHER_PROJECT_ID, "other");
        push_fixture_task(
            &mut raw,
            OTHER_TASK_ID,
            OTHER_PROJECT_ID,
            "other task",
            Some("OTHER-1"),
        );
        push_fixture_repository(
            &mut raw,
            OTHER_REPOSITORY_ID,
            "other-repository",
            "https://github.com/example/other-repository.git",
        );
        push_fixture_checkout(
            &mut raw,
            OTHER_CHECKOUT_ID,
            OTHER_REPOSITORY_ID,
            "/workspace/other-repository",
        );
        push_fixture_component(
            &mut raw,
            COMPONENT_A_ID,
            REPOSITORY_ID,
            "component-a",
            "crates/component-a",
        );
        push_fixture_component(
            &mut raw,
            COMPONENT_B_ID,
            REPOSITORY_ID,
            "component-b",
            "crates/component-b",
        );
        push_fixture_link(
            &mut raw,
            "01890f5e-7b00-7000-8000-000000000067",
            COMPONENT_A_ID,
            "crates/component-a",
            "dependency",
        );
        push_fixture_link(
            &mut raw,
            "01890f5e-7b00-7000-8000-000000000068",
            COMPONENT_B_ID,
            "crates/component-b",
            "related",
        );
        push_pending_correction(&mut raw, PROPOSAL_ID, OLD_ITEM_ID, REPLACEMENT_ITEM_ID);
        push_pending_correction(
            &mut raw,
            "01890f5e-7b00-7000-8000-000000000004",
            "01890f5e-7b00-7000-8000-000000000005",
            "01890f5e-7b00-7000-8000-000000000006",
        );
        raw
    }

    fn mutate_fixture_memory(raw: &mut C2RawSnapshot, id: &str, mutate: impl FnOnce(&mut Value)) {
        let row = raw
            .memory_item
            .iter_mut()
            .find(|row| row["record_id"].as_str() == Some(id))
            .unwrap();
        let mut item = row["item"].clone();
        mutate(&mut item);
        *row = fixture_memory_row(item);
    }

    fn audit_value(raw: C2RawSnapshot) -> Value {
        serde_json::to_value(validate_fixture(raw).audit).unwrap()
    }

    fn assert_mac_domain_changes(
        before: &Value,
        after: &Value,
        changed_table: &str,
        project_changed: bool,
        relay_changed: bool,
    ) {
        let before_tables = before["table_macs"].as_object().unwrap();
        let after_tables = after["table_macs"].as_object().unwrap();
        for (table, before_mac) in before_tables {
            if table == changed_table {
                assert_ne!(before_mac, &after_tables[table]);
            } else {
                assert_eq!(before_mac, &after_tables[table]);
            }
        }
        assert_ne!(before["store_state_mac"], after["store_state_mac"]);
        if project_changed {
            assert_ne!(before["project_view_mac"], after["project_view_mac"]);
        } else {
            assert_eq!(before["project_view_mac"], after["project_view_mac"]);
        }
        if relay_changed {
            assert_ne!(before["relay_candidate_mac"], after["relay_candidate_mac"]);
        } else {
            assert_eq!(before["relay_candidate_mac"], after["relay_candidate_mac"]);
        }
        for field in [
            "schema_version",
            "target",
            "row_counts",
            "memory_status_counts",
            "proposal_status_counts",
            "correction_edges",
        ] {
            assert_eq!(before[field], after[field]);
        }
    }

    #[test]
    fn empty_virtual_input_matches_frozen_accounting_golden() {
        let accounting = account_virtual_input(&empty_snapshot(), &[]).unwrap();
        assert_eq!(
            accounting,
            RawAccounting {
                nodes: 39,
                bytes: 747,
                maximum_depth: 2,
            }
        );
    }

    #[test]
    fn every_table_accepts_exact_cap_and_mask_reports_every_plus_one() {
        for table in TABLE_KINDS {
            let mut exact = empty_snapshot();
            *rows_mut(&mut exact, table) = vec![Value::Null; table.row_cap()];
            account_virtual_input(&exact, &[]).unwrap();

            let mut overflow = empty_snapshot();
            *rows_mut(&mut overflow, table) = vec![Value::Null; table.row_cap() + 1];
            assert_eq!(
                account_virtual_input(&overflow, &[]).unwrap_err(),
                C2SemanticError::LimitExceeded {
                    table_overflow_mask: Some(table.overflow_bit()),
                }
            );
        }

        let mut every = empty_snapshot();
        for table in TABLE_KINDS {
            *rows_mut(&mut every, table) = vec![Value::Null; table.row_cap() + 1];
        }
        assert_eq!(
            account_virtual_input(&every, &[]).unwrap_err(),
            C2SemanticError::LimitExceeded {
                table_overflow_mask: Some(0x01ff),
            }
        );
    }

    #[test]
    fn meter_limits_are_inclusive_plus_one_and_checked_overflow_rejects() {
        let mut node_meter = Meter {
            nodes: MAX_RAW_NODES - 1,
            bytes: 0,
            maximum_depth: 0,
        };
        node_meter.enter_node(MAX_DEPTH).unwrap();
        assert_eq!(node_meter.nodes, MAX_RAW_NODES);
        assert_limit_none(node_meter.enter_node(0));

        let mut byte_meter = Meter {
            nodes: 0,
            bytes: MAX_RAW_BYTES - 1,
            maximum_depth: 0,
        };
        byte_meter.add_bytes(1).unwrap();
        assert_eq!(byte_meter.bytes, MAX_RAW_BYTES);
        assert_limit_none(byte_meter.add_bytes(1));

        let mut row = RowBytes(MAX_ROW_BYTES - 1);
        row.add(1).unwrap();
        assert_eq!(row.0, MAX_ROW_BYTES);
        assert_limit_none(row.add(1));

        let mut node_overflow = Meter {
            nodes: u64::MAX,
            bytes: 0,
            maximum_depth: 0,
        };
        assert_limit_none(node_overflow.enter_node(0));
        let mut byte_overflow = Meter {
            nodes: 0,
            bytes: u64::MAX,
            maximum_depth: 0,
        };
        assert_limit_none(byte_overflow.add_bytes(1));
        let mut row_overflow = RowBytes(u64::MAX);
        assert_limit_none(row_overflow.add(1));
        assert_limit_none(add_len(u64::MAX, 1));
        assert_limit_none(child_depth(u64::MAX));

        let mut depth_meter = Meter::new();
        depth_meter.enter_node(MAX_DEPTH).unwrap();
        assert_limit_none(depth_meter.enter_node(MAX_DEPTH + 1));
    }

    #[test]
    fn row_byte_limit_accepts_exact_encoding_and_rejects_plus_one() {
        let exact = json!({
            "item": {
                "content": "x".repeat(32_768),
                "evidence": [{"excerpt": "y".repeat(32_656)}]
            }
        });
        let mut raw = empty_snapshot();
        raw.memory_item.push(exact);
        account_virtual_input(&raw, &[]).unwrap();

        raw.memory_item[0]["item"]["evidence"][0]["excerpt"] = Value::String("y".repeat(32_657));
        assert_limit_none(account_virtual_input(&raw, &[]));
    }

    #[test]
    fn depth_object_key_and_generic_vector_caps_are_exact() {
        let mut depth_ok = empty_snapshot();
        depth_ok
            .memory_item
            .push(json!({"unknown": nested_array(13)}));
        assert_eq!(
            account_virtual_input(&depth_ok, &[]).unwrap().maximum_depth,
            MAX_DEPTH
        );
        depth_ok.memory_item[0] = json!({"unknown": nested_array(14)});
        assert_limit_none(account_virtual_input(&depth_ok, &[]));

        let mut keys = Map::new();
        for index in 0..MAX_OBJECT_KEYS {
            keys.insert(format!("k{index}"), Value::Null);
        }
        let mut raw = empty_snapshot();
        raw.memory_item.push(Value::Object(keys.clone()));
        account_virtual_input(&raw, &[]).unwrap();
        keys.insert("overflow".to_owned(), Value::Null);
        raw.memory_item[0] = Value::Object(keys);
        assert_limit_none(account_virtual_input(&raw, &[]));

        raw.memory_item[0] = json!({"unknown": vec![Value::Null; 64]});
        account_virtual_input(&raw, &[]).unwrap();
        raw.memory_item[0] = json!({"unknown": vec![Value::Null; 65]});
        assert_limit_none(account_virtual_input(&raw, &[]));
    }

    #[test]
    fn scalar_caps_are_path_specific_and_inclusive() {
        let mut raw = empty_snapshot();
        raw.memory_item
            .push(json!({"unknown": "x".repeat(MAX_ORDINARY_SCALAR_BYTES)}));
        account_virtual_input(&raw, &[]).unwrap();
        raw.memory_item[0]["unknown"] = Value::String("x".repeat(MAX_ORDINARY_SCALAR_BYTES + 1));
        assert_limit_none(account_virtual_input(&raw, &[]));

        raw.memory_item[0] =
            memory_row_with_item_field("title", Value::String("n".repeat(MAX_NAME_SCALAR_BYTES)));
        account_virtual_input(&raw, &[]).unwrap();
        raw.memory_item[0]["item"]["title"] = Value::String("n".repeat(MAX_NAME_SCALAR_BYTES + 1));
        assert_limit_none(account_virtual_input(&raw, &[]));

        for (exact, overflow) in [
            (
                memory_row_with_item_field(
                    "content",
                    Value::String("c".repeat(MAX_LARGE_SCALAR_BYTES)),
                ),
                memory_row_with_item_field(
                    "content",
                    Value::String("c".repeat(MAX_LARGE_SCALAR_BYTES + 1)),
                ),
            ),
            (
                json!({"item": {"procedure": {"commands": ["c".repeat(32_768)]}}}),
                json!({"item": {"procedure": {"commands": ["c".repeat(32_769)]}}}),
            ),
            (
                json!({"item": {"evidence": [{"excerpt": "e".repeat(32_768)}]}}),
                json!({"item": {"evidence": [{"excerpt": "e".repeat(32_769)}]}}),
            ),
        ] {
            raw.memory_item[0] = exact;
            account_virtual_input(&raw, &[]).unwrap();
            raw.memory_item[0] = overflow;
            assert_limit_none(account_virtual_input(&raw, &[]));
        }

        raw.memory_item[0] = json!({"item": {"procedure": {
            "verification": {"command": "v".repeat(4_096)}
        }}});
        account_virtual_input(&raw, &[]).unwrap();
        raw.memory_item[0]["item"]["procedure"]["verification"]["command"] =
            Value::String("v".repeat(4_097));
        assert_limit_none(account_virtual_input(&raw, &[]));

        let mut exact_key = Map::new();
        exact_key.insert("k".repeat(4_096), Value::Null);
        raw.memory_item[0] = Value::Object(exact_key);
        account_virtual_input(&raw, &[]).unwrap();
        let mut overflow_key = Map::new();
        overflow_key.insert("k".repeat(4_097), Value::Null);
        raw.memory_item[0] = Value::Object(overflow_key);
        assert_limit_none(account_virtual_input(&raw, &[]));
    }

    #[test]
    fn every_path_specific_vector_cap_is_exact() {
        let cases = [
            (
                json!({"item": {"evidence": vec![Value::Null; 32]}}),
                json!({"item": {"evidence": vec![Value::Null; 33]}}),
            ),
            (
                json!({"item": {"tags": vec![Value::Null; 32]}}),
                json!({"item": {"tags": vec![Value::Null; 33]}}),
            ),
            (
                json!({"item": {"supersedes": vec![Value::Null; 32]}}),
                json!({"item": {"supersedes": vec![Value::Null; 33]}}),
            ),
            (
                json!({"item": {"procedure": {"commands": vec![Value::Null; 16]}}}),
                json!({"item": {"procedure": {"commands": vec![Value::Null; 17]}}}),
            ),
            (
                json!({"item": {"procedure": {"prerequisites": vec![Value::Null; 16]}}}),
                json!({"item": {"procedure": {"prerequisites": vec![Value::Null; 17]}}}),
            ),
            (
                json!({"item": {"procedure": {"failure_signatures": vec![Value::Null; 16]}}}),
                json!({"item": {"procedure": {"failure_signatures": vec![Value::Null; 17]}}}),
            ),
            (
                json!({"item": {"procedure": {"prerequisites": [{"source": {
                    "key_path": vec![Value::Null; 16]
                }}]}}}),
                json!({"item": {"procedure": {"prerequisites": [{"source": {
                    "key_path": vec![Value::Null; 17]
                }}]}}}),
            ),
        ];

        for (exact, overflow) in cases {
            let mut raw = empty_snapshot();
            raw.memory_item.push(exact);
            account_virtual_input(&raw, &[]).unwrap();
            raw.memory_item[0] = overflow;
            assert_limit_none(account_virtual_input(&raw, &[]));
        }
    }

    #[test]
    fn secret_rule_one_literals_each_have_a_near_miss() {
        let cases = [
            (
                "prefix -----begin private key----- suffix",
                "prefix ----begin private key----- suffix",
            ),
            (
                "prefix -----BEGIN RSA PRIVATE KEY----- suffix",
                "prefix -----BEGIN RSA PRIVATE KE----- suffix",
            ),
            (
                "prefix -----BEGIN OPENSSH PRIVATE KEY----- suffix",
                "prefix -----BEGIN OPENSSH PRIVATE KE----- suffix",
            ),
            (
                "prefix authorization: bearer synthetic suffix",
                "prefix authorization:bearer synthetic suffix",
            ),
        ];
        for (positive, near_miss) in cases {
            assert!(likely_secret_in_string(positive), "positive: {positive}");
            assert!(
                !likely_secret_in_string(near_miss),
                "near miss: {near_miss}"
            );
        }
    }

    #[test]
    fn secret_rule_two_url_credentials_and_remote_edge_cases() {
        for positive in [
            "https://synthetic:credential@example.test/repository",
            "https://synthetic:credential@example.test?query=one",
            "https://synthetic:credential@example.test#fragment",
            "prefix ssh://synthetic:credential@example.test/repository suffix",
        ] {
            assert!(likely_secret_in_string(positive), "positive: {positive}");
        }
        for near_miss in [
            "https://synthetic@example.test/repository",
            "git@example.test:owner/repository.git",
            "example.test/synthetic:credential@example.test/repository",
        ] {
            assert!(
                !likely_secret_in_string(near_miss),
                "near miss: {near_miss}"
            );
        }
    }

    #[test]
    fn secret_rule_three_all_assignments_and_delimiters_have_near_misses() {
        for name in SECRET_ASSIGNMENT_NAMES {
            let positive = format!("export{name}=12345678");
            let near_miss = format!("export{name}=1234567");
            assert!(likely_secret_in_string(&positive), "positive: {positive}");
            assert!(
                !likely_secret_in_string(&near_miss),
                "near miss: {near_miss}"
            );
        }
        assert!(likely_secret_in_string("('[api_key=\"12345678\";]')"));
        assert!(likely_secret_in_string("export API_TOKEN=`12345678`"));
        assert!(likely_secret_in_string("API_KEY = 12345678"));
        assert!(likely_secret_in_string("export API_TOKEN = '12345678'"));
        assert!(!likely_secret_in_string("API_KEY = 1234567"));
        assert!(!likely_secret_in_string("NOT_A_TOKEN=12345678"));
        assert!(!likely_secret_in_string("EXPORTAPI_TOKEN=12345678"));
    }

    #[test]
    fn secret_rule_four_every_token_prefix_has_an_exact_threshold_near_miss() {
        let aws = format!("AKIA{}", "A".repeat(16));
        assert!(likely_secret_in_string(&aws));
        assert!(!likely_secret_in_string(&format!("AKIA{}", "A".repeat(15))));
        assert!(!likely_secret_in_string(&format!("AKIA{}", "A".repeat(17))));
        assert!(!likely_secret_in_string("AKIAAAAAAAAAAAAAAAA!"));

        let github_pat = format!("github_pat_{}", "a".repeat(19));
        assert_eq!(github_pat.len(), 30);
        assert!(likely_secret_in_string(&github_pat));
        assert!(!likely_secret_in_string(&format!(
            "github_pat_{}",
            "a".repeat(18)
        )));

        for prefix in ["ghp_", "gho_", "ghu_", "ghs_", "ghr_"] {
            let positive = format!("{prefix}{}", "a".repeat(20 - prefix.len()));
            let near_miss = format!("{prefix}{}", "a".repeat(19 - prefix.len()));
            assert!(likely_secret_in_string(&positive), "positive: {prefix}");
            assert!(!likely_secret_in_string(&near_miss), "near miss: {prefix}");
        }

        for prefix in ["xoxb-", "xoxp-", "xoxa-", "xoxr-", "xoxs-"] {
            let positive = format!("{prefix}{}", "a".repeat(20 - prefix.len()));
            let near_miss = format!("{prefix}{}", "a".repeat(19 - prefix.len()));
            assert!(likely_secret_in_string(&positive), "positive: {prefix}");
            assert!(!likely_secret_in_string(&near_miss), "near miss: {prefix}");
        }

        let sk = format!("sk-{}", "a".repeat(21));
        assert!(likely_secret_in_string(&format!("([{sk}=])")));
        assert!(!likely_secret_in_string(&format!("sk-{}", "a".repeat(20))));
    }

    #[test]
    fn secret_rule_five_jwt_exact_shape_and_near_misses() {
        let positive = "eyJaaaaa.bbbbbbbb.cccccccc";
        assert!(likely_secret_in_string(positive));
        for near_miss in [
            "eyJaaaaa.bbbbbbb.cccccccc",
            "eyJaaaaa.bbbbbbbb.cccccccc.dddddddd",
            "eyIaaaaa.bbbbbbbb.cccccccc",
            "eyJaaaaa.bbbbbbbb.ccccccc!",
        ] {
            assert!(
                !likely_secret_in_string(near_miss),
                "near miss: {near_miss}"
            );
        }
    }

    #[test]
    fn secret_rule_six_all_credential_keys_nested_values_and_placeholders() {
        let names = [
            "API_KEY",
            "APIKEY",
            "API_TOKEN",
            "ACCESS_TOKEN",
            "AUTH_TOKEN",
            "PASSWORD",
            "PASSWD",
            "CLIENT_SECRET",
            "PRIVATE_KEY",
            "AUTHORIZATION",
        ];
        for name in names {
            let mut positive = Map::new();
            positive.insert(name.to_owned(), Value::String("12345678".to_owned()));
            assert!(
                secret_in_value(&Value::Object(positive)),
                "positive: {name}"
            );

            let mut near_miss = Map::new();
            near_miss.insert(name.to_owned(), Value::String("1234567".to_owned()));
            assert!(
                !secret_in_value(&Value::Object(near_miss)),
                "near miss: {name}"
            );
        }

        assert!(secret_in_value(&json!({
            " api-key ": {"nested": [null, "  12345678  "]}
        })));
        assert!(secret_in_value(&json!({"api token": [false, "12345678"]})));
        assert!(!secret_in_value(&json!({"password": 12345678})));

        for placeholder in [
            "[redacted]",
            "changeme",
            "example",
            "not-set",
            "placeholder",
            "redacted",
            "your_token_here",
            " YOUR_TOKEN_HERE ",
        ] {
            assert!(!secret_in_value(&json!({"authorization": placeholder})));
        }

        assert!(secret_in_value(&json!({"API_TOKEN=12345678": "safe"})));
        assert!(!secret_in_value(&json!({"API_TOKEN=1234567": "safe"})));
    }

    #[test]
    fn secret_scan_covers_every_target_and_table_string_family() {
        const CANARY: &str = "API_TOKEN=synthetic-canary";
        const NEAR_MISS: &str = "API_TOKEN=1234567";

        let target_mutations: [fn(&mut C2RawTarget, &str); 8] = [
            |target, value| target.project_id = value.to_owned(),
            |target, value| target.project_name = value.to_owned(),
            |target, value| target.repository_id = value.to_owned(),
            |target, value| target.repository_remote = value.to_owned(),
            |target, value| target.checkout_id = value.to_owned(),
            |target, value| target.checkout_path = value.to_owned(),
            |target, value| target.task_id = Some(value.to_owned()),
            |target, value| target.task_name = Some(value.to_owned()),
        ];
        for mutate in target_mutations {
            let mut positive = empty_snapshot();
            mutate(&mut positive.target, CANARY);
            assert!(matches!(
                validate_limit_and_secret_boundary(&positive, &[]),
                Err(C2SemanticError::SecretMaterial)
            ));

            let mut near_miss = empty_snapshot();
            mutate(&mut near_miss.target, NEAR_MISS);
            validate_limit_and_secret_boundary(&near_miss, &[]).unwrap();
        }

        for table in TABLE_KINDS {
            let mut positive = empty_snapshot();
            rows_mut(&mut positive, table).push(json!({"probe": CANARY}));
            assert!(matches!(
                validate_limit_and_secret_boundary(&positive, &[]),
                Err(C2SemanticError::SecretMaterial)
            ));

            let mut near_miss = empty_snapshot();
            rows_mut(&mut near_miss, table).push(json!({"probe": NEAR_MISS}));
            validate_limit_and_secret_boundary(&near_miss, &[]).unwrap();
        }
    }

    #[test]
    fn limit_secret_strict_record_and_receipt_precedence_is_frozen() {
        const CANARY: &str = "PASSWORD=synthetic-canary";

        let mut table_overflow = empty_snapshot();
        table_overflow.memory_forget_receipt =
            vec![json!({"probe": CANARY}); TableKind::MemoryForgetReceipt.row_cap() + 1];
        assert_eq!(
            validate_limit_and_secret_boundary(&table_overflow, &[]).err(),
            Some(C2SemanticError::LimitExceeded {
                table_overflow_mask: Some(TableKind::MemoryForgetReceipt.overflow_bit()),
            })
        );

        let mut later_limit = empty_snapshot();
        later_limit.target.project_id = CANARY.to_owned();
        later_limit
            .memory_item
            .push(json!({"unknown": "x".repeat(4_097)}));
        assert_limit_none(validate_limit_and_secret_boundary(&later_limit, &[]));

        let mut secret_receipt = empty_snapshot();
        secret_receipt
            .memory_forget_receipt
            .push(json!({"malformed": CANARY}));
        assert_eq!(
            validate_limit_and_secret_boundary(&secret_receipt, &[]).err(),
            Some(C2SemanticError::SecretMaterial)
        );

        let mut malformed_receipt = empty_snapshot();
        malformed_receipt
            .memory_forget_receipt
            .push(json!({"malformed": "safe"}));
        let boundary = validate_limit_and_secret_boundary(&malformed_receipt, &[]).unwrap();
        assert_eq!(
            enforce_receipt_precedence_after_strict_records(
                &boundary,
                Err(C2SemanticError::InvalidRecord),
            ),
            Err(C2SemanticError::InvalidRecord)
        );

        let mut strictly_valid_receipt = empty_snapshot();
        strictly_valid_receipt.memory_forget_receipt.push(json!({}));
        let boundary = validate_limit_and_secret_boundary(&strictly_valid_receipt, &[]).unwrap();
        assert_eq!(
            enforce_receipt_precedence_after_strict_records(&boundary, Ok(())),
            Err(C2SemanticError::IncompleteDeletion)
        );

        let no_receipt = validate_limit_and_secret_boundary(&empty_snapshot(), &[]).unwrap();
        enforce_receipt_precedence_after_strict_records(&no_receipt, Ok(())).unwrap();
        assert_eq!(no_receipt.accounting.nodes, 39);
    }

    #[test]
    fn error_display_never_contains_caller_material_or_table_mask() {
        let canary = "synthetic-caller-canary";
        for error in [
            C2SemanticError::LimitExceeded {
                table_overflow_mask: Some(0x01ff),
            },
            C2SemanticError::SecretMaterial,
            C2SemanticError::InvalidRecord,
            C2SemanticError::InconsistentProjection,
            C2SemanticError::IncompleteDeletion,
            C2SemanticError::AmbiguousIdentity,
            C2SemanticError::ScopeMismatch,
            C2SemanticError::AppliedProvenanceUnproven,
        ] {
            let display = error.to_string();
            assert!(!display.contains(canary));
            assert!(!display.contains("511"));
        }
    }

    #[test]
    fn canonical_json_value_and_hmac_golden() {
        let encoded = canonical_json_value(&json!({"b": false, "a": 1})).unwrap();
        assert_eq!(
            lower_hex(&encoded),
            "0600000000000000020000000000000001610300000000000000013100000000000000016201"
        );

        let engine = C2MacEngine::consume(C2OneShotMacKey::from_test_bytes([0; 32]));
        assert_eq!(
            engine.authenticate(&encoded).to_lower_hex(),
            "a2751529cd3929ac5ff03c72909a40ad401f9cc9cbd1f0ea6f3428a0f0c5841c"
        );

        let other_engine = C2MacEngine::consume(C2OneShotMacKey::from_test_bytes([0x01; 32]));
        assert_ne!(
            other_engine.authenticate(&encoded),
            engine.authenticate(&encoded)
        );
    }

    #[test]
    fn key_and_pad_types_zeroize_their_storage() {
        assert!(std::mem::needs_drop::<C2OneShotMacKey>());
        assert!(std::mem::needs_drop::<C2MacPad>());

        DROPPED_TEST_KEY_BYTES.with(|bytes| bytes.set(None));
        DROPPED_TEST_PAD_BYTES.with(|bytes| bytes.set(None));
        drop(C2OneShotMacKey::from_test_bytes([0xa5; 32]));
        drop(C2MacPad([0x5a; 64]));
        DROPPED_TEST_KEY_BYTES.with(|bytes| assert_eq!(bytes.get(), Some([0; 32])));
        DROPPED_TEST_PAD_BYTES.with(|bytes| assert_eq!(bytes.get(), Some([0; 64])));
    }

    #[test]
    fn empty_virtual_input_encoding_length_golden() {
        let virtual_input = json!({
            "target": {
                "project_id": PROJECT_ID,
                "project_name": "engram",
                "repository_id": REPOSITORY_ID,
                "repository_remote": "https://github.com/ymeiri/engram.git",
                "checkout_id": CHECKOUT_ID,
                "checkout_path": "/workspace/engram",
                "task_id": null,
                "task_name": null
            },
            "prior_correction_bindings": [],
            "memory_item": [],
            "correction_proposal": [],
            "memory_forget_receipt": [],
            "work_project": [],
            "work_task": [],
            "git_repository": [],
            "local_checkout": [],
            "monorepo_component": [],
            "project_repository_link": []
        });
        assert_eq!(canonical_json_value(&virtual_input).unwrap().len(), 747);
    }

    #[test]
    fn record_table_and_store_framing_goldens() {
        assert_eq!(MEMORY_ITEM_GOLDEN.len(), 693);
        let row = memory_item_row();
        let encoded_row = canonical_json_value(&row).unwrap();
        assert_eq!(encoded_row.len(), 1_432);

        let old_item_id = uuid_bytes(OLD_ITEM_ID);
        let record_preimage = record_preimage(C2TableTag::MemoryItem, old_item_id, &row).unwrap();
        assert_eq!(record_preimage.len(), 1_516);

        let engine = C2MacEngine::consume(C2OneShotMacKey::from_test_bytes([0; 32]));
        let record_mac = engine
            .record_mac(C2TableTag::MemoryItem, old_item_id, &row)
            .unwrap();
        assert_eq!(
            record_mac.to_lower_hex(),
            "a9e0669d85e989b1abb3d77ba797f940792fa58ffffaeda18cd91c2c72104e24"
        );

        let record = C2RecordMac {
            id: old_item_id,
            mac: record_mac,
        };
        let table_preimage = table_preimage(C2TableTag::MemoryItem, &[record]).unwrap();
        assert_eq!(table_preimage.len(), 123);
        let memory_table_mac = engine.table_mac(C2TableTag::MemoryItem, &[record]).unwrap();
        assert_eq!(
            memory_table_mac.to_lower_hex(),
            "ef7c82bb7927a73db9152a0f935654c54c372428ca4c78e8a8940bc40cae0632"
        );

        let mut table_macs = [C2Mac([0; 32]); 9];
        for (index, table) in C2TableTag::ALL.into_iter().enumerate() {
            table_macs[index] = if table == C2TableTag::MemoryItem {
                memory_table_mac
            } else {
                engine.table_mac(table, &[]).unwrap()
            };
        }
        let store_preimage = store_preimage(&table_macs).unwrap();
        assert_eq!(store_preimage.len(), 605);
        assert_eq!(
            engine.store_mac(&table_macs).unwrap().to_lower_hex(),
            "a99a636011b9fdca4b709edbe60e5b53b45c5c7a19f441e28cf19a40749a084f"
        );
    }

    #[test]
    fn project_view_and_relay_framing_goldens() {
        let empty: [C2RecordMac; 0] = [];
        let no_edges: [C2CorrectionEdgeMac; 0] = [];
        let frame = C2ProjectViewFrame {
            target_project: repeated_record(PROJECT_ID, 0x11),
            target_repository: repeated_record(REPOSITORY_ID, 0x22),
            target_checkout: repeated_record(CHECKOUT_ID, 0x33),
            target_task: None,
            competing_links: &empty,
            linked_components: &empty,
            correction_edges: &no_edges,
            applicable_memories: &empty,
        };
        let project_preimage = project_view_preimage(&frame).unwrap();
        assert_eq!(project_preimage.len(), 264);

        let engine = C2MacEngine::consume(C2OneShotMacKey::from_test_bytes([0; 32]));
        assert_eq!(
            engine.project_view_mac(&frame).unwrap().to_lower_hex(),
            "18b2228f9af7cc94b603117bb220f1d4148ad34225e1ed11e8a8880aed5e44a8"
        );

        let relay_preimage = relay_candidates_preimage(&empty).unwrap();
        assert_eq!(relay_preimage.len(), 51);
        assert_eq!(
            engine.relay_candidates_mac(&empty).unwrap().to_lower_hex(),
            "f81ca811ea6d1bcbc9ae94fb1315d3f3841ab07c55083f9e3baad480089abfec"
        );
    }

    #[test]
    fn variable_collections_are_uuid_byte_sorted_before_framing() {
        let first = repeated_record(PROJECT_ID, 0x44);
        let second = repeated_record(REPOSITORY_ID, 0x55);
        let forward = [first, second];
        let reverse = [second, first];
        assert_eq!(
            table_preimage(C2TableTag::WorkProject, &forward).unwrap(),
            table_preimage(C2TableTag::WorkProject, &reverse).unwrap()
        );
        assert_eq!(
            relay_candidates_preimage(&forward).unwrap(),
            relay_candidates_preimage(&reverse).unwrap()
        );

        let no_edges: [C2CorrectionEdgeMac; 0] = [];
        let left = C2ProjectViewFrame {
            target_project: repeated_record(PROJECT_ID, 0x11),
            target_repository: repeated_record(REPOSITORY_ID, 0x22),
            target_checkout: repeated_record(CHECKOUT_ID, 0x33),
            target_task: None,
            competing_links: &forward,
            linked_components: &reverse,
            correction_edges: &no_edges,
            applicable_memories: &forward,
        };
        let right = C2ProjectViewFrame {
            competing_links: &reverse,
            linked_components: &forward,
            applicable_memories: &reverse,
            ..left
        };
        assert_eq!(
            project_view_preimage(&left).unwrap(),
            project_view_preimage(&right).unwrap()
        );
    }

    #[test]
    fn task_and_correction_edge_frames_use_raw_ids_and_macs() {
        let task = repeated_record("01890f5e-7b00-7000-8000-000000000013", 0x66);
        let link = repeated_record("01890f5e-7b00-7000-8000-000000000014", 0x77);
        let component = repeated_record("01890f5e-7b00-7000-8000-000000000015", 0x88);
        let memory = repeated_record(OLD_ITEM_ID, 0x99);
        let pending_edge = C2CorrectionEdgeMac {
            proposal_id: uuid_bytes("01890f5e-7b00-7000-8000-000000000001"),
            obsolete_id: uuid_bytes(OLD_ITEM_ID),
            replacement_id: uuid_bytes("01890f5e-7b00-7000-8000-000000000003"),
            status: C2CorrectionStatus::Pending,
            proposal_mac: C2Mac([0xaa; 32]),
            obsolete_mac: C2Mac([0xbb; 32]),
            replacement_mac: C2Mac([0xcc; 32]),
        };
        let applied_edge = C2CorrectionEdgeMac {
            status: C2CorrectionStatus::Applied,
            ..pending_edge
        };
        let pending = C2ProjectViewFrame {
            target_project: repeated_record(PROJECT_ID, 0x11),
            target_repository: repeated_record(REPOSITORY_ID, 0x22),
            target_checkout: repeated_record(CHECKOUT_ID, 0x33),
            target_task: Some(task),
            competing_links: &[link],
            linked_components: &[component],
            correction_edges: &[pending_edge],
            applicable_memories: &[memory],
        };
        let applied = C2ProjectViewFrame {
            correction_edges: &[applied_edge],
            ..pending
        };
        let pending_preimage = project_view_preimage(&pending).unwrap();
        let applied_preimage = project_view_preimage(&applied).unwrap();
        assert_eq!(pending_preimage.len(), 727);
        assert_eq!(applied_preimage.len(), 727);
        assert_ne!(pending_preimage, applied_preimage);

        let task_id_offset = PROJECT_VIEW_DOMAIN.len() + (24 + 40) * 3 + 1;
        assert_eq!(
            &project_view_preimage(&pending).unwrap()[task_id_offset..task_id_offset + 8],
            &16_u64.to_be_bytes()
        );
        assert_eq!(
            &project_view_preimage(&pending).unwrap()[task_id_offset + 8..task_id_offset + 24],
            &task.id
        );
    }

    #[test]
    fn uuid_v7_requires_the_rfc_variant() {
        assert_eq!(
            parse_id("01890f5e-7b00-7000-0000-000000000099"),
            Err(C2SemanticError::InvalidRecord)
        );
    }

    #[test]
    fn correction_digest_schema_one_matches_exact_frozen_preimage() {
        let proposal: CorrectionProposal = serde_json::from_str(PENDING_PROPOSAL_GOLDEN).unwrap();
        let obsolete: MemoryItem = serde_json::from_str(MEMORY_ITEM_GOLDEN).unwrap();
        let replacement: MemoryItem = serde_json::from_str(PENDING_REPLACEMENT_GOLDEN).unwrap();
        let payload = C2CorrectionDigestPayload {
            schema_version: proposal.digest_schema_version,
            proposal_id: &proposal.id,
            obsolete_id: &proposal.obsolete_id,
            replacement_id: &proposal.replacement_id,
            memory_kind: &proposal.memory_kind,
            scope: &proposal.scope,
            obsolete: (&obsolete).into(),
            replacement: (&replacement).into(),
        };
        let bytes = serde_json::to_vec(&payload).unwrap();
        assert_eq!(bytes, CORRECTION_DIGEST_GOLDEN.as_bytes());
        assert_eq!(bytes.len(), 1_683);
        assert_eq!(
            sha256_hex(&bytes),
            "ebd471b14169c74083751c4b8ce6fbf44e27f1613c6c34c9b7fb6cf926232a01"
        );
        assert_eq!(
            correction_digest(&proposal, &obsolete, &replacement).unwrap(),
            proposal.canonical_digest
        );

        let proposal_bytes = serde_json::to_vec(&proposal).unwrap();
        assert_eq!(proposal_bytes, PENDING_PROPOSAL_GOLDEN.as_bytes());
        assert_eq!(proposal_bytes.len(), 635);
        assert_eq!(
            sha256_hex(&proposal_bytes),
            "3b0a8f366d4c7478edcbe6c778f3f4d4c006916ad107706d24c47ca328865563"
        );
        for item in [&obsolete, &replacement] {
            let digest_item = serde_json::to_value(C2CorrectionDigestItem::from(item)).unwrap();
            let object = digest_item.as_object().unwrap();
            assert!(object.contains_key("correction_proposal_id"));
            assert!(object.contains_key("pending_correction_proposal_id"));
            assert!(!object.contains_key("created_at"));
            assert!(!object.contains_key("updated_at"));
            assert!(!object.contains_key("last_used_at"));
        }
    }

    #[test]
    fn valid_snapshot_yields_sanitized_audit_and_exact_scope_projection() {
        let validated = validate_fixture(valid_snapshot());
        assert_eq!(validated.target.project_id.to_string(), PROJECT_ID);
        assert_eq!(validated.target.repository_id.to_string(), REPOSITORY_ID);
        assert_eq!(validated.target.checkout_id.to_string(), CHECKOUT_ID);
        assert_eq!(
            validated.target.task_id.map(|id| id.to_string()).as_deref(),
            Some(TASK_ID)
        );
        assert!(validated.pending_bindings.is_empty());
        assert!(validated.target_affecting_correction_ids.is_empty());
        assert_eq!(validated.memory_items.len(), 4);
        assert_eq!(validated.projects.len(), 1);
        assert_eq!(validated.tasks.len(), 1);
        assert_eq!(validated.repositories.len(), 1);
        assert_eq!(validated.checkouts.len(), 1);
        assert!(validated.components.is_empty());
        assert_eq!(validated.project_links.len(), 1);
        assert!(validated.correction_proposals.is_empty());

        let applicable = validated
            .applicable_memory_ids
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        assert_eq!(
            applicable,
            [GLOBAL_MEMORY_ID, PROJECT_MEMORY_ID, TASK_MEMORY_ID]
        );
        let audit = serde_json::to_value(&validated.audit).unwrap();
        assert_eq!(audit["schema_version"], 1);
        assert_eq!(audit["row_counts"]["memory_item"], 4);
        assert_eq!(audit["row_counts"]["work_project"], 1);
        assert_eq!(audit["row_counts"]["work_task"], 1);
        assert_eq!(audit["row_counts"]["git_repository"], 1);
        assert_eq!(audit["row_counts"]["local_checkout"], 1);
        assert_eq!(audit["row_counts"]["project_repository_link"], 1);
        assert_eq!(audit["memory_status_counts"]["active"], 4);
        assert_eq!(audit["proposal_status_counts"]["pending"], 0);
        assert_eq!(audit["proposal_status_counts"]["applied"], 0);
        assert_eq!(audit["correction_edges"], json!([]));
        assert_eq!(audit["relay_candidates"].as_array().unwrap().len(), 3);
        for name in ["store_state_mac", "project_view_mac", "relay_candidate_mac"] {
            let mac = audit[name].as_str().unwrap();
            assert_eq!(mac.len(), 64);
            assert!(mac
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
        }
        for mac in audit["table_macs"].as_object().unwrap().values() {
            let mac = mac.as_str().unwrap();
            assert_eq!(mac.len(), 64);
            assert!(mac
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
        }
        let serialized = serde_json::to_string(&validated.audit).unwrap();
        assert_eq!(serialized, VALID_AUDIT_GOLDEN);
        assert!(!serialized.contains("content for"));
        assert!(!serialized.contains(FIXTURE_REMOTE));
        assert!(!serialized.contains(FIXTURE_CHECKOUT_PATH));
        assert!(!serialized.contains("_sha256"));
    }

    #[test]
    fn every_table_and_edge_permutation_with_tied_timestamps_has_the_same_audit() {
        let forward =
            serde_json::to_value(validate_fixture(rich_permutation_snapshot()).audit).unwrap();
        for table in TABLE_KINDS {
            let mut reversed = rich_permutation_snapshot();
            if table != TableKind::MemoryForgetReceipt {
                assert!(table_rows(&reversed, table).len() >= 2);
            }
            rows_mut(&mut reversed, table).reverse();
            assert_eq!(
                serde_json::to_value(validate_fixture(reversed).audit).unwrap(),
                forward,
                "{} row order changed the audit",
                table.tag()
            );
        }
    }

    #[test]
    fn valid_mutations_change_only_the_authenticated_mac_domains() {
        let baseline = audit_value(valid_snapshot());

        let mut out_of_scope = valid_snapshot();
        mutate_fixture_memory(&mut out_of_scope, OUT_OF_SCOPE_MEMORY_ID, |item| {
            item["content"] = json!("changed out-of-scope content");
        });
        let out_of_scope = audit_value(out_of_scope);
        assert_mac_domain_changes(&baseline, &out_of_scope, "memory_item", false, false);
        assert_eq!(
            baseline["relay_candidates"],
            out_of_scope["relay_candidates"]
        );

        let mut target_project = valid_snapshot();
        target_project.work_project[0]["description"] = json!("changed target description");
        let target_project = audit_value(target_project);
        assert_mac_domain_changes(&baseline, &target_project, "work_project", true, false);

        let rich_baseline = audit_value(rich_permutation_snapshot());
        let mut linked_component = rich_permutation_snapshot();
        linked_component.monorepo_component[0]["component"]["description"] =
            json!("changed linked component description");
        let linked_component = audit_value(linked_component);
        assert_mac_domain_changes(
            &rich_baseline,
            &linked_component,
            "monorepo_component",
            true,
            false,
        );

        let mut applicable = valid_snapshot();
        mutate_fixture_memory(&mut applicable, GLOBAL_MEMORY_ID, |item| {
            item["content"] = json!("changed applicable content");
        });
        let applicable = audit_value(applicable);
        assert_mac_domain_changes(&baseline, &applicable, "memory_item", true, true);
        assert_ne!(baseline["relay_candidates"], applicable["relay_candidates"]);

        let mut ordered = valid_snapshot();
        mutate_fixture_memory(&mut ordered, GLOBAL_MEMORY_ID, |item| {
            item["tags"] = json!(["alpha", "beta"]);
        });
        let ordered_audit = audit_value(ordered);
        let mut reversed = valid_snapshot();
        mutate_fixture_memory(&mut reversed, GLOBAL_MEMORY_ID, |item| {
            item["tags"] = json!(["beta", "alpha"]);
        });
        let reversed_audit = audit_value(reversed);
        assert_mac_domain_changes(&ordered_audit, &reversed_audit, "memory_item", true, true);
    }

    #[test]
    fn target_id_and_name_are_conjunctive() {
        let mut raw = valid_snapshot();
        raw.target.project_name = "different-project".to_string();
        assert!(matches!(
            validate_semantic_snapshot(
                raw,
                Vec::new(),
                C2OneShotMacKey::from_test_bytes([0x42; 32])
            ),
            Err(C2SemanticError::AmbiguousIdentity)
        ));
    }

    #[test]
    fn pending_to_applied_requires_and_accepts_one_validator_minted_binding() {
        let binding = minted_pending_binding(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
        assert!(matches!(
            validate_semantic_snapshot(
                default_applied_correction_snapshot(),
                Vec::new(),
                C2OneShotMacKey::from_test_bytes([0x32; 32])
            ),
            Err(C2SemanticError::AppliedProvenanceUnproven)
        ));
        let validated = validate_semantic_snapshot(
            default_applied_correction_snapshot(),
            vec![binding],
            C2OneShotMacKey::from_test_bytes([0x33; 32]),
        )
        .unwrap();
        assert!(validated.pending_bindings.is_empty());
        assert_eq!(
            validated.target_affecting_correction_ids,
            vec![parse_id(PROPOSAL_ID).unwrap()]
        );
        assert_eq!(validated.audit.correction_edges.len(), 1);
        assert_eq!(validated.audit.correction_edges[0].status, "applied");
    }

    #[test]
    fn every_applied_timestamp_inequality_fails_closed() {
        let cases = [
            (
                FIXTURE_TIMESTAMP,
                "2026-09-06T00:00:04Z",
                "2026-09-06T00:00:01Z",
                "2026-09-06T00:00:02Z",
                "2026-09-06T00:00:04Z",
                "2026-09-06T00:00:05Z",
            ),
            (
                "2026-09-06T00:00:04Z",
                FIXTURE_TIMESTAMP,
                "2026-09-06T00:00:01Z",
                "2026-09-06T00:00:04Z",
                "2026-09-06T00:00:02Z",
                "2026-09-06T00:00:05Z",
            ),
            (
                FIXTURE_TIMESTAMP,
                FIXTURE_TIMESTAMP,
                "2026-09-06T00:00:03Z",
                "2026-09-06T00:00:04Z",
                "2026-09-06T00:00:02Z",
                "2026-09-06T00:00:05Z",
            ),
            (
                FIXTURE_TIMESTAMP,
                FIXTURE_TIMESTAMP,
                "2026-09-06T00:00:03Z",
                "2026-09-06T00:00:02Z",
                "2026-09-06T00:00:04Z",
                "2026-09-06T00:00:05Z",
            ),
            (
                FIXTURE_TIMESTAMP,
                FIXTURE_TIMESTAMP,
                "2026-09-06T00:00:01Z",
                "2026-09-06T00:00:02Z",
                "2026-09-06T00:00:04Z",
                "2026-09-06T00:00:03Z",
            ),
            (
                FIXTURE_TIMESTAMP,
                FIXTURE_TIMESTAMP,
                "2026-09-06T00:00:01Z",
                "2026-09-06T00:00:04Z",
                "2026-09-06T00:00:02Z",
                "2026-09-06T00:00:03Z",
            ),
        ];
        for (index, case) in cases.into_iter().enumerate() {
            let (
                obsolete_prior,
                replacement_prior,
                evidence_at,
                obsolete_current,
                replacement_current,
                applied_at,
            ) = case;
            let binding = minted_pending_binding(obsolete_prior, replacement_prior);
            let result = validate_semantic_snapshot(
                applied_correction_snapshot(
                    obsolete_prior,
                    replacement_prior,
                    evidence_at,
                    obsolete_current,
                    replacement_current,
                    applied_at,
                ),
                vec![binding],
                C2OneShotMacKey::from_test_bytes([0x40 + index as u8; 32]),
            );
            assert!(
                matches!(result, Err(C2SemanticError::AppliedProvenanceUnproven)),
                "timestamp case {index}"
            );
        }
    }

    #[test]
    fn immutable_applied_change_is_rejected_against_minted_binding() {
        let binding = minted_pending_binding(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
        assert!(matches!(
            validate_semantic_snapshot(
                applied_snapshot_with_immutable_title_change(),
                vec![binding],
                C2OneShotMacKey::from_test_bytes([0x34; 32])
            ),
            Err(C2SemanticError::AppliedProvenanceUnproven)
        ));
    }

    #[test]
    fn memory_title_is_prose_but_rejects_other_ascii_controls() {
        let mut item: MemoryItem = serde_json::from_value(fixture_memory_item(
            GLOBAL_MEMORY_ID,
            "line one\n\tline two",
            json!({"type": "global"}),
        ))
        .unwrap();
        assert_eq!(validate_memory_domain(&item), Ok(()));
        item.title = "bad\u{000b}title".to_string();
        assert_eq!(
            validate_memory_domain(&item),
            Err(C2SemanticError::InvalidRecord)
        );
    }

    #[test]
    fn unrelated_scope_reference_failure_precedes_ambiguous_target() {
        let scopes = [
            json!({
                "type": "project",
                "project_id": null,
                "project_name": "missing-project"
            }),
            json!({
                "type": "task",
                "project_id": null,
                "project_name": null,
                "task_id": null,
                "task_name": "missing-task"
            }),
            json!({
                "type": "repository",
                "repository_id": "01890f5e-7b00-7000-8000-000000000099",
                "remote_url": null,
                "local_path": null
            }),
        ];
        for (offset, scope) in scopes.into_iter().enumerate() {
            let mut raw = valid_snapshot();
            push_scoped_memory(
                &mut raw,
                &format!("01890f5e-7b00-7000-8000-00000000002{}", offset + 5),
                scope,
            );
            raw.target.checkout_path = "/workspace/not-the-target".to_string();
            assert!(matches!(
                validate_semantic_snapshot(
                    raw,
                    Vec::new(),
                    C2OneShotMacKey::from_test_bytes([0x42; 32])
                ),
                Err(C2SemanticError::InconsistentProjection)
            ));
        }
    }

    #[test]
    fn ambiguous_target_precedes_deferred_scope_mismatch() {
        let mut raw = valid_snapshot();
        push_scoped_memory(
            &mut raw,
            "01890f5e-7b00-7000-8000-000000000025",
            json!({
                "type": "project",
                "project_id": PROJECT_ID,
                "project_name": "missing-project"
            }),
        );
        raw.target.checkout_path = "/workspace/not-the-target".to_string();
        assert!(matches!(
            validate_semantic_snapshot(
                raw,
                Vec::new(),
                C2OneShotMacKey::from_test_bytes([0x42; 32])
            ),
            Err(C2SemanticError::AmbiguousIdentity)
        ));
    }

    #[test]
    fn same_task_selector_in_another_project_is_out_of_scope() {
        const OTHER_PROJECT_ID: &str = "01890f5e-7b00-7000-8000-000000000060";
        const OTHER_TASK_ID: &str = "01890f5e-7b00-7000-8000-000000000063";
        const OTHER_MEMORY_ID: &str = "01890f5e-7b00-7000-8000-000000000025";
        const UNQUALIFIED_MEMORY_ID: &str = "01890f5e-7b00-7000-8000-000000000026";
        let mut raw = valid_snapshot();
        push_fixture_project(&mut raw, OTHER_PROJECT_ID, "other");
        push_fixture_task(&mut raw, OTHER_TASK_ID, OTHER_PROJECT_ID, "ENG-42", None);
        push_scoped_memory(
            &mut raw,
            OTHER_MEMORY_ID,
            json!({
                "type": "task",
                "project_id": OTHER_PROJECT_ID,
                "project_name": "other",
                "task_id": OTHER_TASK_ID,
                "task_name": "ENG-42"
            }),
        );
        push_scoped_memory(
            &mut raw,
            UNQUALIFIED_MEMORY_ID,
            json!({
                "type": "task",
                "project_id": null,
                "project_name": null,
                "task_id": null,
                "task_name": "C2A semantic validator"
            }),
        );

        let validated = validate_fixture(raw);
        for excluded in [OTHER_MEMORY_ID, UNQUALIFIED_MEMORY_ID] {
            assert!(!validated
                .applicable_memory_ids
                .contains(&parse_id(excluded).unwrap()));
        }
    }

    #[test]
    fn ambiguous_unqualified_task_scope_precedes_target_resolution() {
        const OTHER_PROJECT_ID: &str = "01890f5e-7b00-7000-8000-000000000060";
        const OTHER_TASK_ID: &str = "01890f5e-7b00-7000-8000-000000000063";
        const SCOPE_MEMORY_ID: &str = "01890f5e-7b00-7000-8000-000000000025";
        let cases = [
            (
                "cross-project exact-name collision",
                "C2A semantic validator",
                "C2A semantic validator",
                Some("OTHER-1"),
            ),
            (
                "target-name/other-Jira collision",
                "C2A semantic validator",
                "other task",
                Some("C2A semantic validator"),
            ),
            (
                "target-Jira/other-name collision",
                "ENG-42",
                "ENG-42",
                Some("OTHER-2"),
            ),
        ];
        for (label, task_selector, other_name, other_jira) in cases {
            for break_target in [false, true] {
                let mut raw = valid_snapshot();
                push_fixture_project(&mut raw, OTHER_PROJECT_ID, "other");
                push_fixture_task(
                    &mut raw,
                    OTHER_TASK_ID,
                    OTHER_PROJECT_ID,
                    other_name,
                    other_jira,
                );
                push_scoped_memory(
                    &mut raw,
                    SCOPE_MEMORY_ID,
                    json!({
                        "type": "task",
                        "project_id": null,
                        "project_name": null,
                        "task_id": null,
                        "task_name": task_selector
                    }),
                );
                if break_target {
                    raw.target.checkout_path = "/workspace/not-the-target".to_string();
                }
                let actual = validate_semantic_snapshot(
                    raw,
                    Vec::new(),
                    C2OneShotMacKey::from_test_bytes([0x42; 32]),
                )
                .err()
                .expect("ambiguous task scope must fail closed");
                assert_eq!(
                    actual,
                    C2SemanticError::InconsistentProjection,
                    "{label}, break_target={break_target}"
                );
            }
        }
    }

    #[test]
    fn single_target_task_with_incoherent_project_name_defers_scope_mismatch() {
        for (break_target, expected) in [
            (false, C2SemanticError::ScopeMismatch),
            (true, C2SemanticError::AmbiguousIdentity),
        ] {
            let mut raw = valid_snapshot();
            push_scoped_memory(
                &mut raw,
                "01890f5e-7b00-7000-8000-000000000025",
                json!({
                    "type": "task",
                    "project_id": PROJECT_ID,
                    "project_name": "missing-project",
                    "task_id": TASK_ID,
                    "task_name": "ENG-42"
                }),
            );
            if break_target {
                raw.target.checkout_path = "/workspace/not-the-target".to_string();
            }
            let actual = validate_semantic_snapshot(
                raw,
                Vec::new(),
                C2OneShotMacKey::from_test_bytes([0x42; 32]),
            )
            .err()
            .expect("incoherent task scope must fail closed");
            assert_eq!(actual, expected, "break_target={break_target}");
        }
    }

    #[test]
    fn split_non_target_repository_scope_is_scope_mismatch() {
        const REPOSITORY_A: &str = "01890f5e-7b00-7000-8000-000000000061";
        const REPOSITORY_B: &str = "01890f5e-7b00-7000-8000-000000000062";
        const REMOTE_A: &str = "https://github.com/example/repository-a.git";
        const REMOTE_B: &str = "https://github.com/example/repository-b.git";
        let mut raw = valid_snapshot();
        push_fixture_repository(&mut raw, REPOSITORY_A, "repository-a", REMOTE_A);
        push_fixture_repository(&mut raw, REPOSITORY_B, "repository-b", REMOTE_B);
        push_scoped_memory(
            &mut raw,
            "01890f5e-7b00-7000-8000-000000000025",
            json!({
                "type": "repository",
                "repository_id": REPOSITORY_A,
                "remote_url": REMOTE_B,
                "local_path": null
            }),
        );
        assert!(matches!(
            validate_semantic_snapshot(
                raw,
                Vec::new(),
                C2OneShotMacKey::from_test_bytes([0x42; 32])
            ),
            Err(C2SemanticError::ScopeMismatch)
        ));
    }

    #[test]
    fn repository_scope_split_precedes_dangling_selector() {
        const REPOSITORY_A: &str = "01890f5e-7b00-7000-8000-000000000061";
        const REPOSITORY_B: &str = "01890f5e-7b00-7000-8000-000000000062";
        const REMOTE_A: &str = "https://github.com/example/repository-a.git";
        const REMOTE_B: &str = "https://github.com/example/repository-b.git";
        const MISSING_PATH: &str = "/workspace/missing-repository";
        let base = || {
            let mut raw = valid_snapshot();
            push_fixture_repository(&mut raw, REPOSITORY_A, "repository-a", REMOTE_A);
            push_fixture_repository(&mut raw, REPOSITORY_B, "repository-b", REMOTE_B);
            raw
        };

        let mut split = base();
        push_scoped_memory(
            &mut split,
            "01890f5e-7b00-7000-8000-000000000025",
            json!({
                "type": "repository",
                "repository_id": REPOSITORY_A,
                "remote_url": REMOTE_B,
                "local_path": MISSING_PATH
            }),
        );
        assert!(matches!(
            validate_semantic_snapshot(
                split,
                Vec::new(),
                C2OneShotMacKey::from_test_bytes([0x42; 32])
            ),
            Err(C2SemanticError::ScopeMismatch)
        ));

        let mut dangling = base();
        push_scoped_memory(
            &mut dangling,
            "01890f5e-7b00-7000-8000-000000000025",
            json!({
                "type": "repository",
                "repository_id": null,
                "remote_url": null,
                "local_path": MISSING_PATH
            }),
        );
        assert!(matches!(
            validate_semantic_snapshot(
                dangling,
                Vec::new(),
                C2OneShotMacKey::from_test_bytes([0x42; 32])
            ),
            Err(C2SemanticError::InconsistentProjection)
        ));

        let mut target_touch = base();
        push_scoped_memory(
            &mut target_touch,
            "01890f5e-7b00-7000-8000-000000000025",
            json!({
                "type": "repository",
                "repository_id": REPOSITORY_ID,
                "remote_url": null,
                "local_path": MISSING_PATH
            }),
        );
        assert!(matches!(
            validate_semantic_snapshot(
                target_touch,
                Vec::new(),
                C2OneShotMacKey::from_test_bytes([0x42; 32])
            ),
            Err(C2SemanticError::ScopeMismatch)
        ));
    }

    #[test]
    fn valid_receipt_precedes_logical_identity_duplicate() {
        let mut raw = valid_snapshot();
        raw.memory_forget_receipt.push(json!({
            "record_id": "01890f5e-7b00-7000-8000-000000000070",
            "deleted": false,
            "proposal_ids": [],
            "pending_replacement_ids": [],
            "unlocked_obsolete_ids": [],
            "cleanup_complete": false,
            "created_at": FIXTURE_TIMESTAMP
        }));
        push_fixture_project(&mut raw, "01890f5e-7b00-7000-8000-000000000060", "ENGRAM");
        assert!(matches!(
            validate_semantic_snapshot(
                raw,
                Vec::new(),
                C2OneShotMacKey::from_test_bytes([0x42; 32])
            ),
            Err(C2SemanticError::IncompleteDeletion)
        ));
    }

    #[test]
    fn receipt_precedence_is_end_to_end_before_row_projection() {
        for completed in [false, true] {
            let mut raw = valid_snapshot();
            let mut receipt = json!({
                "record_id": "01890f5e-7b00-7000-8000-000000000070",
                "deleted": completed,
                "proposal_ids": [],
                "pending_replacement_ids": [],
                "unlocked_obsolete_ids": [],
                "cleanup_complete": completed,
                "created_at": FIXTURE_TIMESTAMP
            });
            if completed {
                receipt
                    .as_object_mut()
                    .unwrap()
                    .insert("completed_at".to_string(), json!(FIXTURE_TIMESTAMP));
            }
            raw.memory_forget_receipt.push(receipt);
            raw.memory_item[0]["snapshot_digest"] = json!("0".repeat(64));
            assert!(matches!(
                validate_semantic_snapshot(
                    raw,
                    Vec::new(),
                    C2OneShotMacKey::from_test_bytes([0x42; 32])
                ),
                Err(C2SemanticError::IncompleteDeletion)
            ));
        }

        let mut malformed = valid_snapshot();
        malformed.memory_forget_receipt.push(json!({}));
        assert!(matches!(
            validate_semantic_snapshot(
                malformed,
                Vec::new(),
                C2OneShotMacKey::from_test_bytes([0x42; 32])
            ),
            Err(C2SemanticError::InvalidRecord)
        ));

        let mut secret = valid_snapshot();
        secret
            .memory_forget_receipt
            .push(json!({"unknown": "PASSWORD=synthetic-canary"}));
        assert!(matches!(
            validate_semantic_snapshot(
                secret,
                Vec::new(),
                C2OneShotMacKey::from_test_bytes([0x42; 32])
            ),
            Err(C2SemanticError::SecretMaterial)
        ));
    }

    #[test]
    fn malformed_outer_projection_domains_precede_valid_receipts() {
        type OuterProjectionMutation = fn(&mut C2RawSnapshot);

        let cases: &[(&str, OuterProjectionMutation)] = &[
            ("memory_item.kind_key", |raw| {
                raw.memory_item[0]["kind_key"] = json!("decisio\u{301}n");
            }),
            ("memory_item.status_key", |raw| {
                raw.memory_item[0]["status_key"] = json!("unknown");
            }),
            ("memory_item.scope_key", |raw| {
                raw.memory_item[0]["scope_key"] = json!("bad\u{000b}scope");
            }),
            ("memory_item.harness_key", |raw| {
                raw.memory_item[0]["harness_key"] = json!("bad\u{000b}harness");
            }),
            ("memory_item.model_key", |raw| {
                raw.memory_item[0]["model_key"] = json!("bad\u{000b}model");
            }),
            ("correction_proposal.status_key", |raw| {
                *raw = pending_correction_snapshot(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
                raw.correction_proposal[0]["status_key"] = json!("unknown");
            }),
            ("git_repository.name_key", |raw| {
                raw.git_repository[0]["name_key"] = json!("bad\u{000b}name");
            }),
            ("git_repository.remote_url", |raw| {
                raw.git_repository[0]["remote_url"] =
                    json!("https://github.com/ymeiri/engram.git?query");
            }),
            ("git_repository.provider_key", |raw| {
                raw.git_repository[0]["provider_key"] = json!("bad\u{000b}provider");
            }),
            ("local_checkout.local_path_key", |raw| {
                raw.local_checkout[0]["local_path_key"] = json!("/workspace/../engram");
            }),
            ("local_checkout.current_branch", |raw| {
                raw.local_checkout[0]["current_branch"] = json!("bad\u{000b}branch");
            }),
            ("local_checkout.head_sha", |raw| {
                raw.local_checkout[0]["head_sha"] = json!("A".repeat(40));
            }),
            ("monorepo_component.name_key", |raw| {
                raw.monorepo_component.push(matrix_component_row(
                    "01890f5e-7b00-7000-8000-000000000063",
                    REPOSITORY_ID,
                    "engram-eval",
                ));
                raw.monorepo_component[0]["name_key"] = json!("bad\u{000b}name");
            }),
            ("monorepo_component.path_key", |raw| {
                raw.monorepo_component.push(matrix_component_row(
                    "01890f5e-7b00-7000-8000-000000000063",
                    REPOSITORY_ID,
                    "engram-eval",
                ));
                raw.monorepo_component[0]["path_key"] = json!("../escape");
            }),
            ("monorepo_component.kind", |raw| {
                raw.monorepo_component.push(matrix_component_row(
                    "01890f5e-7b00-7000-8000-000000000063",
                    REPOSITORY_ID,
                    "engram-eval",
                ));
                raw.monorepo_component[0]["kind"] = json!("bad\u{000b}kind");
            }),
            ("project_repository_link.project_name_key", |raw| {
                raw.project_repository_link[0]["project_name_key"] = json!("bad\u{000b}name");
            }),
            ("project_repository_link.component_path_key", |raw| {
                raw.project_repository_link[0]["component_path_key"] = json!("../escape");
            }),
            ("project_repository_link.role", |raw| {
                raw.project_repository_link[0]["role"] = json!("unknown");
            }),
        ];

        for (label, mutate) in cases {
            let mut raw = valid_snapshot();
            mutate(&mut raw);
            push_valid_receipt(&mut raw);
            let actual = validate_semantic_snapshot(
                raw,
                Vec::new(),
                C2OneShotMacKey::from_test_bytes([0x42; 32]),
            )
            .err()
            .expect("malformed outer projection must fail closed");
            assert_eq!(actual, C2SemanticError::InvalidRecord, "{label}");
        }

        let mut stale = valid_snapshot();
        stale.local_checkout[0]["local_path_key"] = json!("/workspace/stale");
        push_valid_receipt(&mut stale);
        assert!(matches!(
            validate_semantic_snapshot(
                stale,
                Vec::new(),
                C2OneShotMacKey::from_test_bytes([0x42; 32])
            ),
            Err(C2SemanticError::IncompleteDeletion)
        ));
    }

    // Integrate inside native_successor_semantic.rs's existing `tests` module.

    fn matrix_assert_invalid<T>(result: C2Result<T>) {
        assert!(matches!(result, Err(C2SemanticError::InvalidRecord)));
    }

    fn matrix_assert_inconsistent<T>(result: C2Result<T>) {
        assert!(matches!(
            result,
            Err(C2SemanticError::InconsistentProjection)
        ));
    }

    fn matrix_assert_snapshot_error(raw: C2RawSnapshot, expected: C2SemanticError) {
        let actual = match validate_semantic_snapshot(
            raw,
            Vec::new(),
            C2OneShotMacKey::from_test_bytes([0x9a; 32]),
        ) {
            Ok(_) => panic!("snapshot unexpectedly validated"),
            Err(error) => error,
        };
        assert_eq!(actual, expected);
    }

    fn matrix_component_row(id: &str, repository_id: &str, path: &str) -> Value {
        let component = json!({
            "id": id,
            "repository_id": repository_id,
            "name": format!("component-{id}"),
            "path": path,
            "kind": "crate",
            "description": null,
            "created_at": FIXTURE_TIMESTAMP,
            "updated_at": FIXTURE_TIMESTAMP
        });
        json!({
            "record_id": id,
            "component": component,
            "repository_id": repository_id,
            "name_key": format!("component-{id}").to_lowercase(),
            "path_key": path,
            "kind": "crate",
            "created_at": FIXTURE_TIMESTAMP,
            "updated_at": FIXTURE_TIMESTAMP
        })
    }

    fn matrix_component_row_from_value(component: Value) -> Value {
        let typed: MonorepoComponent = serde_json::from_value(component.clone()).unwrap();
        let embedded = component.as_object().unwrap();
        json!({
            "record_id": typed.id,
            "component": component,
            "repository_id": typed.repository_id,
            "name_key": typed.name.to_lowercase(),
            "path_key": typed.path,
            "kind": typed.kind,
            "created_at": embedded["created_at"],
            "updated_at": embedded["updated_at"]
        })
    }

    fn matrix_checkout_row(id: &str, repository_id: Option<&str>, path: &str) -> Value {
        let checkout = json!({
            "id": id,
            "repository_id": repository_id,
            "local_path": path,
            "current_branch": null,
            "head_sha": null,
            "is_dirty": null,
            "created_at": FIXTURE_TIMESTAMP,
            "updated_at": FIXTURE_TIMESTAMP,
            "last_seen_at": FIXTURE_TIMESTAMP
        });
        json!({
            "record_id": id,
            "checkout": checkout,
            "repository_id": repository_id,
            "local_path_key": path,
            "current_branch": null,
            "head_sha": null,
            "is_dirty": null,
            "created_at": FIXTURE_TIMESTAMP,
            "updated_at": FIXTURE_TIMESTAMP,
            "last_seen_at": FIXTURE_TIMESTAMP
        })
    }

    fn matrix_link_row(
        id: &str,
        project_id: Option<&str>,
        project_name: &str,
        repository_id: &str,
        component: Option<(&str, &str)>,
        role: &str,
    ) -> Value {
        let component_id = component.map(|(id, _)| id);
        let component_path = component.map(|(_, path)| path);
        let link = json!({
            "id": id,
            "project_id": project_id,
            "project_name": project_name,
            "repository_id": repository_id,
            "component_id": component_id,
            "component_path": component_path,
            "role": role,
            "created_at": FIXTURE_TIMESTAMP,
            "updated_at": FIXTURE_TIMESTAMP
        });
        json!({
            "record_id": id,
            "link": link,
            "project_id": project_id,
            "project_name_key": project_name.to_lowercase(),
            "repository_id": repository_id,
            "component_id": component_id,
            "component_path_key": component_path,
            "role": role,
            "created_at": FIXTURE_TIMESTAMP,
            "updated_at": FIXTURE_TIMESTAMP
        })
    }

    fn matrix_rewrite_repository_id(raw: &mut C2RawSnapshot, id: &str) {
        raw.target.repository_id = id.to_string();
        raw.git_repository[0]["record_id"] = json!(id);
        raw.git_repository[0]["repository"]["id"] = json!(id);
        raw.local_checkout[0]["repository_id"] = json!(id);
        raw.local_checkout[0]["checkout"]["repository_id"] = json!(id);
        raw.project_repository_link[0]["repository_id"] = json!(id);
        raw.project_repository_link[0]["link"]["repository_id"] = json!(id);
    }

    fn matrix_pending_row_with(
        proposal_id: &str,
        obsolete_id: &str,
        replacement_id: &str,
    ) -> Value {
        let mut proposal: Value = serde_json::from_str(PENDING_PROPOSAL_GOLDEN).unwrap();
        proposal["id"] = json!(proposal_id);
        proposal["obsolete_id"] = json!(obsolete_id);
        proposal["replacement_id"] = json!(replacement_id);
        fixture_proposal_row(proposal)
    }

    #[test]
    fn strict_schema_rejects_unknown_and_missing_fields_at_every_row_family() {
        let pending = pending_correction_snapshot(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
        let component = matrix_component_row(
            "01890f5e-7b00-7000-8000-000000000063",
            REPOSITORY_ID,
            "engram-eval",
        );
        let receipt = json!({
            "record_id": "01890f5e-7b00-7000-8000-000000000070",
            "deleted": false,
            "proposal_ids": [],
            "pending_replacement_ids": [],
            "unlocked_obsolete_ids": [],
            "cleanup_complete": false,
            "created_at": FIXTURE_TIMESTAMP
        });

        let mut memory = valid_snapshot().memory_item.remove(0);
        memory["unknown"] = json!(true);
        matrix_assert_invalid(strict_memory_row(&memory));
        memory.as_object_mut().unwrap().remove("unknown");
        memory.as_object_mut().unwrap().remove("kind_key");
        matrix_assert_invalid(strict_memory_row(&memory));

        let mut nested_memory = valid_snapshot().memory_item.remove(0);
        nested_memory["item"]["writer"]["model"]["unknown"] = json!(true);
        matrix_assert_invalid(strict_memory_row(&nested_memory));
        nested_memory["item"]["writer"]["model"]
            .as_object_mut()
            .unwrap()
            .remove("unknown");
        nested_memory["item"]["writer"]["model"]
            .as_object_mut()
            .unwrap()
            .remove("provider");
        matrix_assert_invalid(strict_memory_row(&nested_memory));

        let mut proposal = pending.correction_proposal[0].clone();
        proposal["proposal"]["unknown"] = json!(true);
        matrix_assert_invalid(strict_proposal_row(&proposal));
        proposal["proposal"]
            .as_object_mut()
            .unwrap()
            .remove("unknown");
        proposal["proposal"]
            .as_object_mut()
            .unwrap()
            .remove("scope");
        matrix_assert_invalid(strict_proposal_row(&proposal));

        type MatrixRowValidator = fn(&Value) -> C2Result<()>;
        let row_mutations: Vec<(Value, MatrixRowValidator)> = vec![
            (valid_snapshot().work_project.remove(0), |row| {
                strict_project_row(row).map(|_| ())
            }),
            (valid_snapshot().work_task.remove(0), |row| {
                strict_task_row(row).map(|_| ())
            }),
            (valid_snapshot().git_repository.remove(0), |row| {
                strict_repository_row(row).map(|_| ())
            }),
            (valid_snapshot().local_checkout.remove(0), |row| {
                strict_checkout_row(row).map(|_| ())
            }),
            (component, |row| strict_component_row(row).map(|_| ())),
            (valid_snapshot().project_repository_link.remove(0), |row| {
                strict_link_row(row).map(|_| ())
            }),
            (receipt, validate_forget_receipt_schema),
        ];
        for (index, (row, validator)) in row_mutations.into_iter().enumerate() {
            let mut unknown = row.clone();
            unknown["unknown"] = json!(index);
            matrix_assert_invalid(validator(&unknown));
            let mut missing = row;
            missing.as_object_mut().unwrap().remove("created_at");
            matrix_assert_invalid(validator(&missing));
        }
    }

    #[test]
    fn none_null_and_omission_contract_is_exact() {
        let mut memory = valid_snapshot().memory_item.remove(0);
        assert!(strict_memory_row(&memory).is_ok());
        memory.as_object_mut().unwrap().remove("session_id");
        matrix_assert_invalid(strict_memory_row(&memory));

        let mut project = valid_snapshot().work_project.remove(0);
        assert!(strict_project_row(&project).is_ok());
        project.as_object_mut().unwrap().remove("description");
        matrix_assert_invalid(strict_project_row(&project));

        let mut task = valid_snapshot().work_task.remove(0);
        task["description"] = Value::Null;
        task["jira_key"] = Value::Null;
        assert!(strict_task_row(&task).is_ok());
        for field in ["description", "jira_key"] {
            let mut missing = task.clone();
            missing.as_object_mut().unwrap().remove(field);
            matrix_assert_invalid(strict_task_row(&missing));
        }

        let mut repository = valid_snapshot().git_repository.remove(0);
        repository["repository"]["remote_url"] = Value::Null;
        repository["remote_url"] = Value::Null;
        assert!(strict_repository_row(&repository).is_ok());
        repository.as_object_mut().unwrap().remove("remote_url");
        matrix_assert_invalid(strict_repository_row(&repository));

        let checkout = matrix_checkout_row(
            "01890f5e-7b00-7000-8000-000000000062",
            None,
            "/workspace/detached",
        );
        assert!(strict_checkout_row(&checkout).is_ok());
        for field in ["repository_id", "current_branch", "head_sha", "is_dirty"] {
            let mut missing = checkout.clone();
            missing.as_object_mut().unwrap().remove(field);
            matrix_assert_invalid(strict_checkout_row(&missing));
        }

        let mut component = matrix_component_row(
            "01890f5e-7b00-7000-8000-000000000063",
            REPOSITORY_ID,
            "engram-eval",
        );
        component["component"]["kind"] = Value::Null;
        component["kind"] = Value::Null;
        assert!(strict_component_row(&component).is_ok());
        component.as_object_mut().unwrap().remove("kind");
        matrix_assert_invalid(strict_component_row(&component));

        let link = matrix_link_row(
            "01890f5e-7b00-7000-8000-000000000065",
            None,
            "engram",
            REPOSITORY_ID,
            None,
            "related",
        );
        assert!(strict_link_row(&link).is_ok());
        for field in ["project_id", "component_id", "component_path_key"] {
            let mut missing = link.clone();
            missing.as_object_mut().unwrap().remove(field);
            matrix_assert_invalid(strict_link_row(&missing));
        }

        let mut marker_null = valid_snapshot().memory_item.remove(0);
        marker_null["item"]["correction_proposal_id"] = Value::Null;
        matrix_assert_invalid(strict_memory_row(&marker_null));

        let no_source = matrix_component_row(
            "01890f5e-7b00-7000-8000-000000000063",
            REPOSITORY_ID,
            "engram-eval",
        );
        assert!(strict_component_row(&no_source).is_ok());
        for (source_path, source_digest) in [
            (Some(Value::Null), None),
            (None, Some(Value::Null)),
            (Some(Value::Null), Some(Value::Null)),
        ] {
            let mut invalid = no_source.clone();
            if let Some(value) = source_path {
                invalid["component"]["source_path"] = value;
            }
            if let Some(value) = source_digest {
                invalid["component"]["source_sha256"] = value;
            }
            matrix_assert_invalid(strict_component_row(&invalid));
        }

        let pending = pending_correction_snapshot(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
        let mut pending_row = pending.correction_proposal[0].clone();
        pending_row["proposal"]["applied_digest"] = Value::Null;
        matrix_assert_invalid(strict_proposal_row(&pending_row));
        let mut pending_missing = pending.correction_proposal[0].clone();
        pending_missing
            .as_object_mut()
            .unwrap()
            .remove("pending_obsolete_id");
        matrix_assert_invalid(strict_proposal_row(&pending_missing));

        let applied = default_applied_correction_snapshot();
        let mut applied_extra = applied.correction_proposal[0].clone();
        applied_extra["pending_obsolete_id"] = json!(OLD_ITEM_ID);
        matrix_assert_invalid(strict_proposal_row(&applied_extra));
        let mut applied_missing = applied.correction_proposal[0].clone();
        applied_missing["proposal"]
            .as_object_mut()
            .unwrap()
            .remove("applied_digest");
        matrix_assert_invalid(strict_proposal_row(&applied_missing));

        assert!(validate_prerequisite_schema(&json!({
            "key": "cargo.profile",
            "expected": "release"
        }))
        .is_ok());
        matrix_assert_invalid(validate_prerequisite_schema(&json!({
            "key": "cargo.profile",
            "expected": "release",
            "source": null
        })));
    }

    #[test]
    fn strict_ids_enums_timestamps_numbers_and_digests_fail_closed() {
        let memory_base = valid_snapshot().memory_item.remove(0);
        for row in [
            {
                let mut row = memory_base.clone();
                row["record_id"] = json!(GLOBAL_MEMORY_ID.to_ascii_uppercase());
                row
            },
            {
                let mut row = memory_base.clone();
                row["item"]["id"] = json!(PROJECT_MEMORY_ID);
                row
            },
            {
                let mut row = memory_base.clone();
                row["item"]["kind"] = json!("Decision");
                row
            },
            {
                let mut row = memory_base.clone();
                row["item"]["confidence"] = json!("0.8");
                row
            },
            {
                let mut row = memory_base.clone();
                row["item"]["confidence"] = json!(1.000_001);
                row
            },
            {
                let mut row = memory_base.clone();
                row["item"]["confidence"] = json!(-0.0);
                row
            },
            {
                let mut row = memory_base.clone();
                row["created_at"] = json!("2026-09-06T03:00:00+03:00");
                row
            },
            {
                let mut row = memory_base.clone();
                row["snapshot_digest"] = json!("A".repeat(64));
                row
            },
        ] {
            matrix_assert_invalid(strict_memory_row(&row));
        }

        let mut proposal = pending_correction_snapshot(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP)
            .correction_proposal
            .remove(0);
        proposal["proposal"]["digest_schema_version"] = json!(1.0);
        matrix_assert_invalid(strict_proposal_row(&proposal));

        let mut repository = valid_snapshot().git_repository.remove(0);
        repository["repository"]["provider"] = json!("GitHub");
        matrix_assert_invalid(strict_repository_row(&repository));

        let mut checkout = valid_snapshot().local_checkout.remove(0);
        checkout["checkout"]["head_sha"] = json!("A".repeat(40));
        matrix_assert_invalid(strict_checkout_row(&checkout));

        let mut task = valid_snapshot().work_task.remove(0);
        task["priority"] = json!(1);
        matrix_assert_invalid(strict_task_row(&task));
    }

    #[test]
    fn every_denormalized_projection_and_snapshot_digest_is_verified() {
        let memory = valid_snapshot().memory_item.remove(0);
        for (field, stale) in [
            ("kind_key", json!("rule")),
            ("status_key", json!("archived")),
            ("scope_key", json!("user")),
            ("harness_key", json!("claude_code")),
            ("model_key", json!("different-model")),
            ("session_id", json!("01890f5e-7b00-7000-8000-000000000099")),
            ("snapshot_digest", json!("0".repeat(64))),
            ("created_at", json!("2026-09-06T00:00:01Z")),
            ("updated_at", json!("2026-09-06T00:00:01Z")),
        ] {
            let mut row = memory.clone();
            row[field] = stale;
            assert_eq!(
                decode_memory_row(row).err(),
                Some(C2SemanticError::InconsistentProjection),
                "memory_item.{field}"
            );
        }

        let proposal = pending_correction_snapshot(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP)
            .correction_proposal
            .remove(0);
        for (field, stale) in [
            ("status_key", json!("applied")),
            ("obsolete_id", json!(PROJECT_MEMORY_ID)),
            ("pending_obsolete_id", json!(PROJECT_MEMORY_ID)),
            ("replacement_id", json!(PROJECT_MEMORY_ID)),
            ("snapshot_digest", json!("0".repeat(64))),
            ("created_at", json!("2026-09-06T00:00:01Z")),
        ] {
            let mut row = proposal.clone();
            row[field] = stale;
            assert_eq!(
                decode_proposal_row(row).err(),
                Some(C2SemanticError::InconsistentProjection),
                "correction_proposal.{field}"
            );
        }

        let repository = valid_snapshot().git_repository.remove(0);
        for (field, stale) in [
            ("name_key", json!("different")),
            ("remote_url", Value::Null),
            ("provider_key", json!("gitlab")),
            ("created_at", json!("2026-09-06T00:00:01Z")),
            ("updated_at", json!("2026-09-06T00:00:01Z")),
        ] {
            let mut row = repository.clone();
            row[field] = stale;
            assert_eq!(
                decode_repository_row(row).err(),
                Some(C2SemanticError::InconsistentProjection),
                "git_repository.{field}"
            );
        }

        let checkout = valid_snapshot().local_checkout.remove(0);
        for (field, stale) in [
            ("repository_id", Value::Null),
            ("local_path_key", json!("/workspace/different")),
            ("current_branch", Value::Null),
            ("head_sha", Value::Null),
            ("is_dirty", json!(true)),
            ("created_at", json!("2026-09-06T00:00:01Z")),
            ("updated_at", json!("2026-09-06T00:00:01Z")),
            ("last_seen_at", json!("2026-09-06T00:00:01Z")),
        ] {
            let mut row = checkout.clone();
            row[field] = stale;
            assert_eq!(
                decode_checkout_row(row).err(),
                Some(C2SemanticError::InconsistentProjection),
                "local_checkout.{field}"
            );
        }

        let component = matrix_component_row(
            "01890f5e-7b00-7000-8000-000000000063",
            REPOSITORY_ID,
            "engram-eval",
        );
        for (field, stale) in [
            ("repository_id", json!(PROJECT_ID)),
            ("name_key", json!("different")),
            ("path_key", json!("different")),
            ("kind", Value::Null),
            ("created_at", json!("2026-09-06T00:00:01Z")),
            ("updated_at", json!("2026-09-06T00:00:01Z")),
        ] {
            let mut row = component.clone();
            row[field] = stale;
            assert_eq!(
                decode_component_row(row).err(),
                Some(C2SemanticError::InconsistentProjection),
                "monorepo_component.{field}"
            );
        }

        let link = valid_snapshot().project_repository_link.remove(0);
        for (field, stale) in [
            ("project_id", Value::Null),
            ("project_name_key", json!("different")),
            ("repository_id", json!(PROJECT_ID)),
            (
                "component_id",
                json!("01890f5e-7b00-7000-8000-000000000063"),
            ),
            ("component_path_key", json!("engram-eval")),
            ("role", json!("dependency")),
            ("created_at", json!("2026-09-06T00:00:01Z")),
            ("updated_at", json!("2026-09-06T00:00:01Z")),
        ] {
            let mut row = link.clone();
            row[field] = stale;
            assert_eq!(
                decode_link_row(row).err(),
                Some(C2SemanticError::InconsistentProjection),
                "project_repository_link.{field}"
            );
        }
    }

    #[test]
    fn forged_and_duplicate_record_ids_are_rejected_but_table_tags_are_namespaces() {
        let mut forged_repository = valid_snapshot().git_repository.remove(0);
        forged_repository["repository"]["id"] = json!(PROJECT_ID);
        matrix_assert_invalid(strict_repository_row(&forged_repository));

        let mut forged_checkout = valid_snapshot().local_checkout.remove(0);
        forged_checkout["checkout"]["id"] = json!(PROJECT_ID);
        matrix_assert_invalid(strict_checkout_row(&forged_checkout));

        let mut forged_component = matrix_component_row(
            "01890f5e-7b00-7000-8000-000000000063",
            REPOSITORY_ID,
            "engram-eval",
        );
        forged_component["component"]["id"] = json!(PROJECT_ID);
        matrix_assert_invalid(strict_component_row(&forged_component));

        let mut forged_link = valid_snapshot().project_repository_link.remove(0);
        forged_link["link"]["id"] = json!(PROJECT_ID);
        matrix_assert_invalid(strict_link_row(&forged_link));

        let mut duplicated = valid_snapshot();
        duplicated
            .memory_item
            .push(duplicated.memory_item[0].clone());
        matrix_assert_inconsistent(decode_snapshot(duplicated));
        let mut duplicated = valid_snapshot();
        duplicated
            .work_project
            .push(duplicated.work_project[0].clone());
        matrix_assert_inconsistent(decode_snapshot(duplicated));
        let mut duplicated = valid_snapshot();
        duplicated.work_task.push(duplicated.work_task[0].clone());
        matrix_assert_inconsistent(decode_snapshot(duplicated));
        let mut duplicated = valid_snapshot();
        duplicated
            .git_repository
            .push(duplicated.git_repository[0].clone());
        matrix_assert_inconsistent(decode_snapshot(duplicated));
        let mut duplicated = valid_snapshot();
        duplicated
            .local_checkout
            .push(duplicated.local_checkout[0].clone());
        matrix_assert_inconsistent(decode_snapshot(duplicated));
        let mut duplicated = valid_snapshot();
        duplicated
            .project_repository_link
            .push(duplicated.project_repository_link[0].clone());
        matrix_assert_inconsistent(decode_snapshot(duplicated));
        let mut duplicated = valid_snapshot();
        let component = matrix_component_row(
            "01890f5e-7b00-7000-8000-000000000063",
            REPOSITORY_ID,
            "engram-eval",
        );
        duplicated.monorepo_component = vec![component.clone(), component];
        matrix_assert_inconsistent(decode_snapshot(duplicated));
        let mut duplicated = pending_correction_snapshot(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
        duplicated
            .correction_proposal
            .push(duplicated.correction_proposal[0].clone());
        matrix_assert_inconsistent(decode_snapshot(duplicated));

        let mut cross_table_same_id = valid_snapshot();
        matrix_rewrite_repository_id(&mut cross_table_same_id, PROJECT_ID);
        assert!(validate_semantic_snapshot(
            cross_table_same_id,
            Vec::new(),
            C2OneShotMacKey::from_test_bytes([0x9b; 32]),
        )
        .is_ok());
    }

    #[test]
    fn target_identity_split_brain_and_missing_or_competing_primary_fail_exactly() {
        const OTHER_PROJECT: &str = "01890f5e-7b00-7000-8000-000000000060";
        const OTHER_REPOSITORY: &str = "01890f5e-7b00-7000-8000-000000000061";
        const OTHER_LINK: &str = "01890f5e-7b00-7000-8000-000000000065";
        const OTHER_REMOTE: &str = "https://github.com/example/other.git";

        let mut split = valid_snapshot();
        push_fixture_project(&mut split, OTHER_PROJECT, "other");
        split.target.project_id = OTHER_PROJECT.to_string();
        matrix_assert_snapshot_error(split, C2SemanticError::AmbiguousIdentity);

        let mut folded_twin = valid_snapshot();
        push_fixture_project(&mut folded_twin, OTHER_PROJECT, "ENGRAM");
        matrix_assert_snapshot_error(folded_twin, C2SemanticError::InconsistentProjection);

        let mut no_primary = valid_snapshot();
        no_primary.project_repository_link.clear();
        matrix_assert_snapshot_error(no_primary, C2SemanticError::AmbiguousIdentity);

        let mut duplicate_primary = valid_snapshot();
        duplicate_primary
            .project_repository_link
            .push(matrix_link_row(
                OTHER_LINK,
                Some(PROJECT_ID),
                "engram",
                REPOSITORY_ID,
                None,
                "primary",
            ));
        matrix_assert_snapshot_error(duplicate_primary, C2SemanticError::InconsistentProjection);

        let mut competing_primary = valid_snapshot();
        push_fixture_repository(
            &mut competing_primary,
            OTHER_REPOSITORY,
            "other",
            OTHER_REMOTE,
        );
        competing_primary
            .project_repository_link
            .push(matrix_link_row(
                OTHER_LINK,
                Some(PROJECT_ID),
                "engram",
                OTHER_REPOSITORY,
                None,
                "primary",
            ));
        matrix_assert_snapshot_error(competing_primary, C2SemanticError::AmbiguousIdentity);

        let mut component_primary = valid_snapshot();
        component_primary
            .monorepo_component
            .push(matrix_component_row(
                "01890f5e-7b00-7000-8000-000000000063",
                REPOSITORY_ID,
                "engram-eval",
            ));
        component_primary.project_repository_link[0] = matrix_link_row(
            PRIMARY_LINK_ID,
            Some(PROJECT_ID),
            "engram",
            REPOSITORY_ID,
            Some(("01890f5e-7b00-7000-8000-000000000063", "engram-eval")),
            "primary",
        );
        matrix_assert_snapshot_error(component_primary, C2SemanticError::AmbiguousIdentity);
    }

    #[test]
    fn every_target_repository_checkout_and_optional_task_selector_is_conjunctive() {
        const OTHER_REPOSITORY: &str = "01890f5e-7b00-7000-8000-000000000061";
        const OTHER_CHECKOUT: &str = "01890f5e-7b00-7000-8000-000000000062";
        const OTHER_TASK: &str = "01890f5e-7b00-7000-8000-000000000066";
        const OTHER_REMOTE: &str = "https://github.com/example/other.git";

        let mut repository_id = valid_snapshot();
        push_fixture_repository(&mut repository_id, OTHER_REPOSITORY, "other", OTHER_REMOTE);
        repository_id.target.repository_id = OTHER_REPOSITORY.to_string();
        matrix_assert_snapshot_error(repository_id, C2SemanticError::AmbiguousIdentity);

        let mut remote = valid_snapshot();
        remote.target.repository_remote = OTHER_REMOTE.to_string();
        matrix_assert_snapshot_error(remote, C2SemanticError::AmbiguousIdentity);

        let mut checkout_id = valid_snapshot();
        checkout_id.local_checkout.push(matrix_checkout_row(
            OTHER_CHECKOUT,
            Some(REPOSITORY_ID),
            "/workspace/other",
        ));
        checkout_id.target.checkout_id = OTHER_CHECKOUT.to_string();
        matrix_assert_snapshot_error(checkout_id, C2SemanticError::AmbiguousIdentity);

        let mut checkout_path = valid_snapshot();
        checkout_path.target.checkout_path = "/workspace/other".to_string();
        matrix_assert_snapshot_error(checkout_path, C2SemanticError::AmbiguousIdentity);

        let mut half_task = valid_snapshot();
        half_task.target.task_name = None;
        matrix_assert_snapshot_error(half_task, C2SemanticError::InvalidRecord);

        let mut wrong_task_id = valid_snapshot();
        push_fixture_task(
            &mut wrong_task_id,
            OTHER_TASK,
            PROJECT_ID,
            "different task",
            Some("ENG-43"),
        );
        wrong_task_id.target.task_id = Some(OTHER_TASK.to_string());
        matrix_assert_snapshot_error(wrong_task_id, C2SemanticError::AmbiguousIdentity);

        let mut wrong_task_name = valid_snapshot();
        wrong_task_name.target.task_name = Some("ENG-404".to_string());
        matrix_assert_snapshot_error(wrong_task_name, C2SemanticError::AmbiguousIdentity);

        let mut detached_target_checkout = valid_snapshot();
        detached_target_checkout.local_checkout[0] =
            matrix_checkout_row(CHECKOUT_ID, None, FIXTURE_CHECKOUT_PATH);
        matrix_assert_snapshot_error(detached_target_checkout, C2SemanticError::AmbiguousIdentity);
    }

    #[test]
    fn task_foreign_keys_duplicates_and_cycles_are_rejected() {
        const OTHER_PROJECT: &str = "01890f5e-7b00-7000-8000-000000000060";
        const TASK_B: &str = "01890f5e-7b00-7000-8000-000000000066";
        const MISSING: &str = "01890f5e-7b00-7000-8000-000000000099";

        let mut missing_project = valid_snapshot();
        missing_project.work_task[0]["project_id"] = json!(MISSING);
        matrix_assert_snapshot_error(missing_project, C2SemanticError::InconsistentProjection);

        let mut missing_blocker = valid_snapshot();
        missing_blocker.work_task[0]["blocked_by"] = json!([MISSING]);
        matrix_assert_snapshot_error(missing_blocker, C2SemanticError::InconsistentProjection);

        let mut self_blocker = valid_snapshot();
        self_blocker.work_task[0]["blocked_by"] = json!([TASK_ID]);
        matrix_assert_snapshot_error(self_blocker, C2SemanticError::InconsistentProjection);

        let mut duplicate_blocker = valid_snapshot();
        push_fixture_task(
            &mut duplicate_blocker,
            TASK_B,
            PROJECT_ID,
            "blocking task",
            None,
        );
        duplicate_blocker.work_task[0]["blocked_by"] = json!([TASK_B, TASK_B]);
        matrix_assert_snapshot_error(duplicate_blocker, C2SemanticError::InconsistentProjection);

        let mut cross_project = valid_snapshot();
        push_fixture_project(&mut cross_project, OTHER_PROJECT, "other");
        push_fixture_task(
            &mut cross_project,
            TASK_B,
            OTHER_PROJECT,
            "blocking task",
            None,
        );
        cross_project.work_task[0]["blocked_by"] = json!([TASK_B]);
        matrix_assert_snapshot_error(cross_project, C2SemanticError::InconsistentProjection);

        let mut cycle = valid_snapshot();
        push_fixture_task(&mut cycle, TASK_B, PROJECT_ID, "blocking task", None);
        cycle.work_task[0]["blocked_by"] = json!([TASK_B]);
        cycle.work_task[1]["blocked_by"] = json!([TASK_ID]);
        matrix_assert_snapshot_error(cycle, C2SemanticError::InconsistentProjection);

        let mut selector_collision = valid_snapshot();
        push_fixture_task(&mut selector_collision, TASK_B, PROJECT_ID, "eng-42", None);
        matrix_assert_snapshot_error(selector_collision, C2SemanticError::InconsistentProjection);
    }

    #[test]
    fn topology_orphans_and_cross_repository_component_links_are_rejected() {
        const OTHER_PROJECT: &str = "01890f5e-7b00-7000-8000-000000000060";
        const OTHER_REPOSITORY: &str = "01890f5e-7b00-7000-8000-000000000061";
        const OTHER_CHECKOUT: &str = "01890f5e-7b00-7000-8000-000000000062";
        const COMPONENT: &str = "01890f5e-7b00-7000-8000-000000000063";
        const LINK: &str = "01890f5e-7b00-7000-8000-000000000065";
        const MISSING: &str = "01890f5e-7b00-7000-8000-000000000099";
        const OTHER_REMOTE: &str = "https://github.com/example/other.git";

        let mut orphan_checkout = valid_snapshot();
        orphan_checkout.local_checkout.push(matrix_checkout_row(
            OTHER_CHECKOUT,
            Some(MISSING),
            "/workspace/orphan",
        ));
        matrix_assert_snapshot_error(orphan_checkout, C2SemanticError::InconsistentProjection);

        let mut orphan_component = valid_snapshot();
        orphan_component
            .monorepo_component
            .push(matrix_component_row(COMPONENT, MISSING, "orphan"));
        matrix_assert_snapshot_error(orphan_component, C2SemanticError::InconsistentProjection);

        let mut missing_link_project = valid_snapshot();
        missing_link_project
            .project_repository_link
            .push(matrix_link_row(
                LINK,
                None,
                "missing",
                REPOSITORY_ID,
                None,
                "related",
            ));
        matrix_assert_snapshot_error(
            missing_link_project,
            C2SemanticError::InconsistentProjection,
        );

        let mut split_link_project = valid_snapshot();
        push_fixture_project(&mut split_link_project, OTHER_PROJECT, "other");
        split_link_project
            .project_repository_link
            .push(matrix_link_row(
                LINK,
                Some(OTHER_PROJECT),
                "engram",
                REPOSITORY_ID,
                None,
                "related",
            ));
        matrix_assert_snapshot_error(split_link_project, C2SemanticError::InconsistentProjection);

        let mut missing_link_repository = valid_snapshot();
        missing_link_repository
            .project_repository_link
            .push(matrix_link_row(
                LINK,
                Some(PROJECT_ID),
                "engram",
                MISSING,
                None,
                "related",
            ));
        matrix_assert_snapshot_error(
            missing_link_repository,
            C2SemanticError::InconsistentProjection,
        );

        let mut missing_link_component = valid_snapshot();
        missing_link_component
            .project_repository_link
            .push(matrix_link_row(
                LINK,
                Some(PROJECT_ID),
                "engram",
                REPOSITORY_ID,
                Some((MISSING, "engram-eval")),
                "related",
            ));
        matrix_assert_snapshot_error(
            missing_link_component,
            C2SemanticError::InconsistentProjection,
        );

        let mut cross_repository = valid_snapshot();
        push_fixture_repository(
            &mut cross_repository,
            OTHER_REPOSITORY,
            "other",
            OTHER_REMOTE,
        );
        cross_repository
            .monorepo_component
            .push(matrix_component_row(
                COMPONENT,
                REPOSITORY_ID,
                "engram-eval",
            ));
        cross_repository
            .project_repository_link
            .push(matrix_link_row(
                LINK,
                Some(PROJECT_ID),
                "engram",
                OTHER_REPOSITORY,
                Some((COMPONENT, "engram-eval")),
                "dependency",
            ));
        matrix_assert_snapshot_error(cross_repository, C2SemanticError::InconsistentProjection);

        let mut wrong_component_path = valid_snapshot();
        wrong_component_path
            .monorepo_component
            .push(matrix_component_row(
                COMPONENT,
                REPOSITORY_ID,
                "engram-eval",
            ));
        wrong_component_path
            .project_repository_link
            .push(matrix_link_row(
                LINK,
                Some(PROJECT_ID),
                "engram",
                REPOSITORY_ID,
                Some((COMPONENT, "different")),
                "dependency",
            ));
        matrix_assert_snapshot_error(
            wrong_component_path,
            C2SemanticError::InconsistentProjection,
        );
    }

    #[test]
    fn lexical_path_traversal_is_rejected_for_checkout_component_source_and_link() {
        let checkout = valid_snapshot().local_checkout.remove(0);
        for path in [
            "/",
            "//workspace/x",
            "/workspace/",
            "/a//b",
            "/a/./b",
            "/a/../b",
        ] {
            let mut invalid = checkout.clone();
            invalid["checkout"]["local_path"] = json!(path);
            invalid["local_path_key"] = json!(path);
            matrix_assert_invalid(strict_checkout_row(&invalid));
        }

        let component = matrix_component_row(
            "01890f5e-7b00-7000-8000-000000000063",
            REPOSITORY_ID,
            "engram-eval",
        );
        for path in [
            "/absolute",
            "trailing/",
            "a//b",
            ".",
            "a/./b",
            "a/../b",
            "..",
        ] {
            let mut invalid = component.clone();
            invalid["component"]["path"] = json!(path);
            invalid["path_key"] = json!(path);
            if path == "." {
                assert!(strict_component_row(&invalid).is_ok());
            } else {
                matrix_assert_invalid(strict_component_row(&invalid));
            }
        }

        let mut bad_source = component.clone();
        bad_source["component"]["source_path"] = json!("../Cargo.toml");
        bad_source["component"]["source_sha256"] = json!("0".repeat(64));
        matrix_assert_invalid(strict_component_row(&bad_source));

        let mut link = matrix_link_row(
            "01890f5e-7b00-7000-8000-000000000065",
            Some(PROJECT_ID),
            "engram",
            REPOSITORY_ID,
            Some(("01890f5e-7b00-7000-8000-000000000063", "../engram-eval")),
            "related",
        );
        matrix_assert_invalid(strict_link_row(&link));
        link["link"]["component_path"] = json!("engram-eval");
        matrix_assert_inconsistent(decode_link_row(link));
    }

    #[test]
    fn every_logical_identity_tuple_and_correction_role_overlap_is_rejected() {
        const REPOSITORY_B: &str = "01890f5e-7b00-7000-8000-000000000061";
        const CHECKOUT_B: &str = "01890f5e-7b00-7000-8000-000000000062";
        const COMPONENT_A: &str = "01890f5e-7b00-7000-8000-000000000063";
        const COMPONENT_B: &str = "01890f5e-7b00-7000-8000-000000000064";
        const LINK_B: &str = "01890f5e-7b00-7000-8000-000000000065";
        const TASK_B: &str = "01890f5e-7b00-7000-8000-000000000066";

        let mut duplicate_task_token = valid_snapshot();
        push_fixture_task(
            &mut duplicate_task_token,
            TASK_B,
            PROJECT_ID,
            "different name",
            Some("ENG-42"),
        );
        matrix_assert_snapshot_error(
            duplicate_task_token,
            C2SemanticError::InconsistentProjection,
        );

        let mut duplicate_remote = valid_snapshot();
        push_fixture_repository(
            &mut duplicate_remote,
            REPOSITORY_B,
            "same remote",
            "git@github.com:ymeiri/engram.git",
        );
        matrix_assert_snapshot_error(duplicate_remote, C2SemanticError::InconsistentProjection);

        let mut duplicate_checkout = valid_snapshot();
        duplicate_checkout.local_checkout.push(matrix_checkout_row(
            CHECKOUT_B,
            Some(REPOSITORY_ID),
            FIXTURE_CHECKOUT_PATH,
        ));
        matrix_assert_snapshot_error(duplicate_checkout, C2SemanticError::InconsistentProjection);

        let mut duplicate_component = valid_snapshot();
        duplicate_component
            .monorepo_component
            .push(matrix_component_row(
                COMPONENT_A,
                REPOSITORY_ID,
                "engram-eval",
            ));
        duplicate_component
            .monorepo_component
            .push(matrix_component_row(
                COMPONENT_B,
                REPOSITORY_ID,
                "engram-eval",
            ));
        matrix_assert_snapshot_error(duplicate_component, C2SemanticError::InconsistentProjection);

        let mut duplicate_link = valid_snapshot();
        duplicate_link.project_repository_link.push(matrix_link_row(
            LINK_B,
            Some(PROJECT_ID),
            "engram",
            REPOSITORY_ID,
            None,
            "dependency",
        ));
        matrix_assert_snapshot_error(duplicate_link, C2SemanticError::InconsistentProjection);

        let mut duplicate_pair = pending_correction_snapshot(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
        duplicate_pair
            .correction_proposal
            .push(matrix_pending_row_with(
                "01890f5e-7b00-7000-8000-000000000004",
                OLD_ITEM_ID,
                "01890f5e-7b00-7000-8000-000000000003",
            ));
        matrix_assert_snapshot_error(duplicate_pair, C2SemanticError::InconsistentProjection);

        let mut correction_chain =
            pending_correction_snapshot(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
        correction_chain
            .correction_proposal
            .push(matrix_pending_row_with(
                "01890f5e-7b00-7000-8000-000000000004",
                "01890f5e-7b00-7000-8000-000000000003",
                "01890f5e-7b00-7000-8000-000000000025",
            ));
        matrix_assert_snapshot_error(correction_chain, C2SemanticError::InconsistentProjection);
    }

    #[test]
    fn supersedes_cycle_is_rejected_and_linked_component_mutation_changes_project_mac() {
        const MEMORY_A: &str = "01890f5e-7b00-7000-8000-000000000025";
        const MEMORY_B: &str = "01890f5e-7b00-7000-8000-000000000026";
        const COMPONENT: &str = "01890f5e-7b00-7000-8000-000000000063";
        const LINK: &str = "01890f5e-7b00-7000-8000-000000000065";

        let mut cycle = valid_snapshot();
        cycle.memory_item.clear();
        let mut item_a = fixture_memory_item(MEMORY_A, "cycle a", json!({"type": "global"}));
        item_a["status"] = json!("superseded");
        item_a["supersedes"] = json!([MEMORY_B]);
        let mut item_b = fixture_memory_item(MEMORY_B, "cycle b", json!({"type": "global"}));
        item_b["status"] = json!("superseded");
        item_b["supersedes"] = json!([MEMORY_A]);
        cycle.memory_item = vec![fixture_memory_row(item_a), fixture_memory_row(item_b)];
        matrix_assert_snapshot_error(cycle, C2SemanticError::InconsistentProjection);

        let mut base = valid_snapshot();
        base.monorepo_component.push(matrix_component_row(
            COMPONENT,
            REPOSITORY_ID,
            "engram-eval",
        ));
        base.project_repository_link.push(matrix_link_row(
            LINK,
            Some(PROJECT_ID),
            "engram",
            REPOSITORY_ID,
            Some((COMPONENT, "engram-eval")),
            "dependency",
        ));
        let before = validate_fixture(base);

        let mut changed = valid_snapshot();
        let mut component = matrix_component_row(COMPONENT, REPOSITORY_ID, "engram-eval");
        let mut embedded = component["component"].clone();
        embedded["description"] = json!("authenticated linked-component change");
        component = matrix_component_row_from_value(embedded);
        changed.monorepo_component.push(component);
        changed.project_repository_link.push(matrix_link_row(
            LINK,
            Some(PROJECT_ID),
            "engram",
            REPOSITORY_ID,
            Some((COMPONENT, "engram-eval")),
            "dependency",
        ));
        let after = validate_fixture(changed);

        assert_ne!(
            before.audit.table_macs.monorepo_component,
            after.audit.table_macs.monorepo_component
        );
        assert_ne!(before.audit.store_state_mac, after.audit.store_state_mac);
        assert_ne!(before.audit.project_view_mac, after.audit.project_view_mac);
        assert_eq!(
            before.audit.relay_candidate_mac,
            after.audit.relay_candidate_mac
        );
    }

    // Paste inside `native_successor_semantic::tests`. This nested module deliberately uses only the
    // deterministic parent fixtures and the private C2A entry point.
    mod correction_matrix_tests {
        use super::*;

        const OTHER_PROPOSAL_ID: &str = "01890f5e-7b00-7000-8000-000000000031";
        const OTHER_OBSOLETE_ID: &str = "01890f5e-7b00-7000-8000-000000000032";
        const OTHER_REPLACEMENT_ID: &str = "01890f5e-7b00-7000-8000-000000000033";
        const THIRD_MEMORY_ID: &str = "01890f5e-7b00-7000-8000-000000000034";
        const DANGLING_ID: &str = "01890f5e-7b00-7000-8000-000000000099";

        fn assert_error(
            raw: C2RawSnapshot,
            bindings: Vec<C2PriorCorrectionBinding>,
            expected: C2SemanticError,
        ) {
            let actual = validate_semantic_snapshot(
                raw,
                bindings,
                C2OneShotMacKey::from_test_bytes([0xa5; 32]),
            )
            .err()
            .expect("fixture must fail closed");
            assert_eq!(actual, expected);
        }

        fn manual_evidence() -> EvidenceRef {
            serde_json::from_value(json!({
                "kind": "manual_review",
                "target": "fixture-review",
                "summary": "fixture review",
                "excerpt": null,
                "observed_at": FIXTURE_TIMESTAMP
            }))
            .unwrap()
        }

        fn unrelated_evidence() -> EvidenceRef {
            serde_json::from_value(json!({
                "kind": "file",
                "target": "second-fixture.md",
                "summary": "second fixture",
                "excerpt": null,
                "observed_at": FIXTURE_TIMESTAMP
            }))
            .unwrap()
        }

        #[derive(Clone, Copy, Debug)]
        enum PendingMutation {
            ProposalObsoleteId,
            ProposalReplacementId,
            ProposalKind,
            ProposalScope,
            ProposerMismatch,
            ObsoleteStatus,
            ObsoleteLockMissing,
            ObsoleteReplacementMarker,
            ReplacementStatus,
            ReplacementOrigin,
            ReplacementProcedureKind,
            ReplacementMarkerMissing,
            ReplacementPendingMarker,
            ReplacementWriterActor,
            ReplacementConfidence,
            ReplacementTags,
            ReplacementReviewAfter,
            ReplacementArchive,
            ReplacementLastUsed,
            ReplacementSupersedes,
            ReplacementEvidenceEmpty,
            ReplacementManualEvidence,
            PreexistingApplicationEvidence,
            CanonicalDigest,
        }

        impl PendingMutation {
            const ALL: [Self; 24] = [
                Self::ProposalObsoleteId,
                Self::ProposalReplacementId,
                Self::ProposalKind,
                Self::ProposalScope,
                Self::ProposerMismatch,
                Self::ObsoleteStatus,
                Self::ObsoleteLockMissing,
                Self::ObsoleteReplacementMarker,
                Self::ReplacementStatus,
                Self::ReplacementOrigin,
                Self::ReplacementProcedureKind,
                Self::ReplacementMarkerMissing,
                Self::ReplacementPendingMarker,
                Self::ReplacementWriterActor,
                Self::ReplacementConfidence,
                Self::ReplacementTags,
                Self::ReplacementReviewAfter,
                Self::ReplacementArchive,
                Self::ReplacementLastUsed,
                Self::ReplacementSupersedes,
                Self::ReplacementEvidenceEmpty,
                Self::ReplacementManualEvidence,
                Self::PreexistingApplicationEvidence,
                Self::CanonicalDigest,
            ];

            fn apply(
                self,
                proposal: &mut CorrectionProposal,
                obsolete: &mut MemoryItem,
                replacement: &mut MemoryItem,
            ) {
                match self {
                    Self::ProposalObsoleteId => proposal.obsolete_id = replacement.id,
                    Self::ProposalReplacementId => proposal.replacement_id = obsolete.id,
                    Self::ProposalKind => proposal.memory_kind = MemoryKind::Rule,
                    Self::ProposalScope => proposal.scope = MemoryScope::Global,
                    Self::ProposerMismatch => proposal.proposer.actor = "other-agent".to_string(),
                    Self::ObsoleteStatus => obsolete.status = MemoryStatus::Superseded,
                    Self::ObsoleteLockMissing => obsolete.pending_correction_proposal_id = None,
                    Self::ObsoleteReplacementMarker => {
                        obsolete.correction_proposal_id = Some(proposal.id)
                    }
                    Self::ReplacementStatus => replacement.status = MemoryStatus::Active,
                    Self::ReplacementOrigin => replacement.origin = ClaimOrigin::UserStated,
                    Self::ReplacementProcedureKind => {
                        proposal.memory_kind = MemoryKind::Procedure;
                        obsolete.kind = MemoryKind::Procedure;
                        replacement.kind = MemoryKind::Procedure;
                    }
                    Self::ReplacementMarkerMissing => replacement.correction_proposal_id = None,
                    Self::ReplacementPendingMarker => {
                        replacement.pending_correction_proposal_id = Some(proposal.id)
                    }
                    Self::ReplacementWriterActor => {
                        replacement.writer.actor = "operator".to_string();
                        proposal.proposer.actor = "operator".to_string();
                    }
                    Self::ReplacementConfidence => {
                        replacement.confidence = engram_core::memory::MemoryConfidence::new(0.7)
                    }
                    Self::ReplacementTags => replacement.tags.push("changed".to_string()),
                    Self::ReplacementReviewAfter => {
                        replacement.review_after = Some(fixture_timestamp("2026-09-07T00:00:00Z"))
                    }
                    Self::ReplacementArchive => {
                        replacement.archive = Some(
                            serde_json::from_value(json!({
                                "reason": "fixture",
                                "archived_by": "fixture",
                                "archived_at": FIXTURE_TIMESTAMP
                            }))
                            .unwrap(),
                        )
                    }
                    Self::ReplacementLastUsed => {
                        replacement.last_used_at = Some(fixture_timestamp(FIXTURE_TIMESTAMP))
                    }
                    Self::ReplacementSupersedes => replacement.supersedes.push(obsolete.id),
                    Self::ReplacementEvidenceEmpty => replacement.evidence.clear(),
                    Self::ReplacementManualEvidence => replacement.evidence.push(manual_evidence()),
                    Self::PreexistingApplicationEvidence => {
                        replacement.evidence.push(EvidenceRef {
                            kind: EvidenceKind::ToolCall,
                            target: correction_application_target(proposal),
                            summary: Some(correction_application_summary(proposal)),
                            excerpt: None,
                            observed_at: fixture_timestamp(FIXTURE_TIMESTAMP),
                        })
                    }
                    Self::CanonicalDigest => proposal.canonical_digest = "0".repeat(64),
                }
            }
        }

        #[test]
        fn pending_pair_rule_matrix_fails_as_current_snapshot_inconsistency() {
            let (base_proposal, base_obsolete, base_replacement) =
                pending_correction_values(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
            assert_eq!(
                validate_pending_pair(&base_proposal, &base_obsolete, &base_replacement),
                Ok(())
            );
            for mutation in PendingMutation::ALL {
                let (mut proposal, mut obsolete, mut replacement) =
                    pending_correction_values(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
                mutation.apply(&mut proposal, &mut obsolete, &mut replacement);
                if !matches!(mutation, PendingMutation::CanonicalDigest) {
                    proposal.canonical_digest =
                        correction_digest(&proposal, &obsolete, &replacement).unwrap();
                }
                assert_eq!(
                    validate_pending_pair(&proposal, &obsolete, &replacement),
                    Err(C2SemanticError::InconsistentProjection),
                    "{mutation:?}"
                );
            }
        }

        fn applied_values() -> (CorrectionProposal, MemoryItem, MemoryItem) {
            let raw = default_applied_correction_snapshot();
            (
                strict_proposal_row(&raw.correction_proposal[0]).unwrap(),
                strict_memory_row(&raw.memory_item[0]).unwrap(),
                strict_memory_row(&raw.memory_item[1]).unwrap(),
            )
        }

        #[derive(Clone, Copy, Debug)]
        enum AppliedMutation {
            ProposalStatus,
            ProposalObsoleteId,
            ProposalReplacementId,
            ProposalKind,
            ProposalScope,
            ProposerMismatch,
            ReplacementStatus,
            ReplacementOrigin,
            ReplacementMarker,
            ReplacementPendingMarker,
            ReplacementSupersedesEmpty,
            ReplacementSupersedesWrong,
            ReplacementManualEvidence,
            ObsoleteStatus,
            ObsoleteMarker,
            ObsoletePendingMarker,
            EvidenceKind,
            EvidenceTarget,
            EvidenceSummary,
            EvidenceExcerpt,
            EvidenceTimestampSplit,
            EvidenceNotLast,
            EarlierApplicationEvidence,
            CanonicalDigest,
            AppliedDigest,
        }

        impl AppliedMutation {
            const ALL: [Self; 25] = [
                Self::ProposalStatus,
                Self::ProposalObsoleteId,
                Self::ProposalReplacementId,
                Self::ProposalKind,
                Self::ProposalScope,
                Self::ProposerMismatch,
                Self::ReplacementStatus,
                Self::ReplacementOrigin,
                Self::ReplacementMarker,
                Self::ReplacementPendingMarker,
                Self::ReplacementSupersedesEmpty,
                Self::ReplacementSupersedesWrong,
                Self::ReplacementManualEvidence,
                Self::ObsoleteStatus,
                Self::ObsoleteMarker,
                Self::ObsoletePendingMarker,
                Self::EvidenceKind,
                Self::EvidenceTarget,
                Self::EvidenceSummary,
                Self::EvidenceExcerpt,
                Self::EvidenceTimestampSplit,
                Self::EvidenceNotLast,
                Self::EarlierApplicationEvidence,
                Self::CanonicalDigest,
                Self::AppliedDigest,
            ];

            fn apply(
                self,
                proposal: &mut CorrectionProposal,
                obsolete: &mut MemoryItem,
                replacement: &mut MemoryItem,
            ) {
                match self {
                    Self::ProposalStatus => proposal.status = CorrectionProposalStatus::Pending,
                    Self::ProposalObsoleteId => proposal.obsolete_id = replacement.id,
                    Self::ProposalReplacementId => proposal.replacement_id = obsolete.id,
                    Self::ProposalKind => proposal.memory_kind = MemoryKind::Rule,
                    Self::ProposalScope => proposal.scope = MemoryScope::Global,
                    Self::ProposerMismatch => proposal.proposer.actor = "other-agent".to_string(),
                    Self::ReplacementStatus => replacement.status = MemoryStatus::NeedsReview,
                    Self::ReplacementOrigin => replacement.origin = ClaimOrigin::UserStated,
                    Self::ReplacementMarker => {
                        replacement.correction_proposal_id = Some(proposal.id)
                    }
                    Self::ReplacementPendingMarker => {
                        replacement.pending_correction_proposal_id = Some(proposal.id)
                    }
                    Self::ReplacementSupersedesEmpty => replacement.supersedes.clear(),
                    Self::ReplacementSupersedesWrong => {
                        replacement.supersedes = vec![replacement.id]
                    }
                    Self::ReplacementManualEvidence => {
                        let final_evidence = replacement.evidence.pop().unwrap();
                        replacement.evidence.push(manual_evidence());
                        replacement.evidence.push(final_evidence);
                    }
                    Self::ObsoleteStatus => obsolete.status = MemoryStatus::Active,
                    Self::ObsoleteMarker => obsolete.correction_proposal_id = Some(proposal.id),
                    Self::ObsoletePendingMarker => {
                        obsolete.pending_correction_proposal_id = Some(proposal.id)
                    }
                    Self::EvidenceKind => {
                        replacement.evidence.last_mut().unwrap().kind = EvidenceKind::File
                    }
                    Self::EvidenceTarget => {
                        replacement.evidence.last_mut().unwrap().target = "wrong-target".to_string()
                    }
                    Self::EvidenceSummary => {
                        replacement.evidence.last_mut().unwrap().summary = Some("wrong".to_string())
                    }
                    Self::EvidenceExcerpt => {
                        replacement.evidence.last_mut().unwrap().excerpt = Some("wrong".to_string())
                    }
                    Self::EvidenceTimestampSplit => {
                        replacement.evidence.last_mut().unwrap().observed_at =
                            fixture_timestamp("2026-09-06T00:00:01.5Z")
                    }
                    Self::EvidenceNotLast => replacement.evidence.push(unrelated_evidence()),
                    Self::EarlierApplicationEvidence => {
                        let application = replacement.evidence.last().unwrap().clone();
                        replacement.evidence.insert(0, application);
                    }
                    Self::CanonicalDigest => proposal.canonical_digest = "0".repeat(64),
                    Self::AppliedDigest => proposal.applied_digest = Some("0".repeat(64)),
                }
            }
        }

        #[test]
        fn applied_pair_rule_matrix_fails_as_current_snapshot_inconsistency() {
            let (base_proposal, base_obsolete, base_replacement) = applied_values();
            assert_eq!(
                validate_applied_pair(&base_proposal, &base_obsolete, &base_replacement),
                Ok(())
            );
            for mutation in AppliedMutation::ALL {
                let (mut proposal, mut obsolete, mut replacement) = applied_values();
                mutation.apply(&mut proposal, &mut obsolete, &mut replacement);
                if !matches!(mutation, AppliedMutation::AppliedDigest) {
                    proposal.applied_digest =
                        Some(correction_digest(&proposal, &obsolete, &replacement).unwrap());
                }
                assert_eq!(
                    validate_applied_pair(&proposal, &obsolete, &replacement),
                    Err(C2SemanticError::InconsistentProjection),
                    "{mutation:?}"
                );
            }
        }

        fn pending_values_with_ids(
            proposal_id: &str,
            obsolete_id: &str,
            replacement_id: &str,
        ) -> (CorrectionProposal, MemoryItem, MemoryItem) {
            let (mut proposal, mut obsolete, mut replacement) =
                pending_correction_values(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
            proposal.id = parse_id(proposal_id).unwrap();
            proposal.obsolete_id = parse_id(obsolete_id).unwrap();
            proposal.replacement_id = parse_id(replacement_id).unwrap();
            obsolete.id = proposal.obsolete_id;
            obsolete.pending_correction_proposal_id = Some(proposal.id);
            replacement.id = proposal.replacement_id;
            replacement.correction_proposal_id = Some(proposal.id);
            proposal.canonical_digest =
                correction_digest(&proposal, &obsolete, &replacement).unwrap();
            (proposal, obsolete, replacement)
        }

        fn two_pending_snapshot() -> C2RawSnapshot {
            let first = pending_values_with_ids(PROPOSAL_ID, OLD_ITEM_ID, OTHER_REPLACEMENT_ID);
            let second =
                pending_values_with_ids(OTHER_PROPOSAL_ID, OTHER_OBSOLETE_ID, THIRD_MEMORY_ID);
            let mut raw = valid_snapshot();
            raw.memory_item.clear();
            raw.correction_proposal.clear();
            for (proposal, obsolete, replacement) in [first, second] {
                raw.memory_item
                    .push(fixture_memory_row(serde_json::to_value(obsolete).unwrap()));
                raw.memory_item.push(fixture_memory_row(
                    serde_json::to_value(replacement).unwrap(),
                ));
                raw.correction_proposal.push(fixture_proposal_row(
                    serde_json::to_value(proposal).unwrap(),
                ));
            }
            raw
        }

        fn minted_distinct_bindings() -> Vec<C2PriorCorrectionBinding> {
            validate_semantic_snapshot(
                two_pending_snapshot(),
                Vec::new(),
                C2OneShotMacKey::from_test_bytes([0xb1; 32]),
            )
            .unwrap()
            .pending_bindings
        }

        #[test]
        fn binding_cardinality_order_limit_and_private_state_fail_closed() {
            let extra = minted_pending_binding(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
            assert_error(
                valid_snapshot(),
                vec![extra],
                C2SemanticError::AppliedProvenanceUnproven,
            );

            let duplicate_a = minted_pending_binding(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
            let duplicate_b = minted_pending_binding(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
            assert_error(
                default_applied_correction_snapshot(),
                vec![duplicate_a, duplicate_b],
                C2SemanticError::AppliedProvenanceUnproven,
            );

            let mut reversed = minted_distinct_bindings();
            assert_eq!(reversed.len(), 2);
            let mut matching_applied = valid_snapshot();
            matching_applied.memory_item.clear();
            matching_applied.correction_proposal.clear();
            for binding in &reversed {
                let mut proposal = binding.proposal.clone();
                let mut obsolete = binding.obsolete.clone();
                let mut replacement = binding.replacement.clone();
                let evidence = EvidenceRef {
                    kind: EvidenceKind::ToolCall,
                    target: correction_application_target(&proposal),
                    summary: Some(correction_application_summary(&proposal)),
                    excerpt: None,
                    observed_at: fixture_timestamp("2026-09-06T00:00:01Z"),
                };
                obsolete.status = MemoryStatus::Superseded;
                obsolete.pending_correction_proposal_id = None;
                obsolete.evidence.push(evidence.clone());
                obsolete.updated_at = fixture_timestamp("2026-09-06T00:00:02Z");
                replacement.status = MemoryStatus::Active;
                replacement.correction_proposal_id = None;
                replacement.supersedes.push(obsolete.id);
                replacement.evidence.push(evidence);
                replacement.updated_at = fixture_timestamp("2026-09-06T00:00:02Z");
                proposal.status = CorrectionProposalStatus::Applied;
                proposal.applied_at = Some(fixture_timestamp("2026-09-06T00:00:03Z"));
                proposal.applied_digest =
                    Some(correction_digest(&proposal, &obsolete, &replacement).unwrap());
                matching_applied
                    .memory_item
                    .push(fixture_memory_row(serde_json::to_value(obsolete).unwrap()));
                matching_applied.memory_item.push(fixture_memory_row(
                    serde_json::to_value(replacement).unwrap(),
                ));
                matching_applied
                    .correction_proposal
                    .push(fixture_proposal_row(
                        serde_json::to_value(proposal).unwrap(),
                    ));
            }
            reversed.reverse();
            assert_error(
                matching_applied,
                reversed,
                C2SemanticError::AppliedProvenanceUnproven,
            );

            let mut too_many = Vec::new();
            for _ in 0..=MAX_BINDINGS {
                too_many.push(minted_pending_binding(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP));
            }
            assert_error(
                valid_snapshot(),
                too_many,
                C2SemanticError::LimitExceeded {
                    table_overflow_mask: None,
                },
            );

            let mut malformed_raw = minted_pending_binding(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
            malformed_raw
                .proposal_row
                .as_object_mut()
                .unwrap()
                .insert("unknown".to_string(), Value::Null);
            assert_error(
                default_applied_correction_snapshot(),
                vec![malformed_raw],
                C2SemanticError::InvalidRecord,
            );

            let mut stale_projection = minted_pending_binding(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
            stale_projection.proposal_row["status_key"] = json!("applied");
            assert_error(
                default_applied_correction_snapshot(),
                vec![stale_projection],
                C2SemanticError::AppliedProvenanceUnproven,
            );

            let mut changed_typed = minted_pending_binding(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
            changed_typed.replacement.title = "changed private typed value".to_string();
            assert_error(
                default_applied_correction_snapshot(),
                vec![changed_typed],
                C2SemanticError::AppliedProvenanceUnproven,
            );
        }

        #[test]
        fn semantic_binding_faults_obey_record_and_receipt_precedence() {
            type BindingMutation = fn(&mut C2PriorCorrectionBinding);

            let cases: &[(&str, BindingMutation)] = &[
                ("proposal ID mismatch", |binding| {
                    binding.proposal_id = parse_id(OTHER_PROPOSAL_ID).unwrap();
                }),
                ("non-pending typed proposal", |binding| {
                    binding.proposal.status = CorrectionProposalStatus::Applied;
                }),
                ("typed proposal field mismatch", |binding| {
                    binding.proposal.memory_kind = MemoryKind::Rule;
                }),
                ("typed obsolete field mismatch", |binding| {
                    binding.obsolete.title = "changed private typed value".to_string();
                }),
                ("typed/raw replacement mismatch", |binding| {
                    binding.replacement.title = "changed private typed value".to_string();
                }),
                ("stale raw proposal status projection", |binding| {
                    binding.proposal_row["status_key"] = json!("applied");
                }),
            ];

            for (label, mutate) in cases {
                let faulty_binding = || {
                    let mut binding = minted_pending_binding(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
                    mutate(&mut binding);
                    binding
                };

                assert_error(
                    default_applied_correction_snapshot(),
                    vec![faulty_binding()],
                    C2SemanticError::AppliedProvenanceUnproven,
                );

                let mut malformed_current = default_applied_correction_snapshot();
                malformed_current.memory_item[0]
                    .as_object_mut()
                    .unwrap()
                    .remove("kind_key");
                assert_error(
                    malformed_current,
                    vec![faulty_binding()],
                    C2SemanticError::InvalidRecord,
                );

                let mut receipt = default_applied_correction_snapshot();
                push_valid_receipt(&mut receipt);
                let actual = validate_semantic_snapshot(
                    receipt,
                    vec![faulty_binding()],
                    C2OneShotMacKey::from_test_bytes([0xa5; 32]),
                )
                .err()
                .expect("receipt fixture must fail closed");
                assert_eq!(actual, C2SemanticError::IncompleteDeletion, "{label}");
            }

            type BindingSetFactory = fn() -> Vec<C2PriorCorrectionBinding>;
            let set_cases: &[(&str, BindingSetFactory)] = &[
                ("unsorted binding IDs", || {
                    let mut bindings = minted_distinct_bindings();
                    bindings.reverse();
                    bindings
                }),
                ("duplicate binding IDs", || {
                    vec![
                        minted_pending_binding(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP),
                        minted_pending_binding(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP),
                    ]
                }),
            ];
            for (label, bindings) in set_cases {
                assert_error(
                    valid_snapshot(),
                    bindings(),
                    C2SemanticError::AppliedProvenanceUnproven,
                );

                let mut malformed_current = valid_snapshot();
                malformed_current.memory_item[0]
                    .as_object_mut()
                    .unwrap()
                    .remove("kind_key");
                assert_error(
                    malformed_current,
                    bindings(),
                    C2SemanticError::InvalidRecord,
                );

                let mut receipt = valid_snapshot();
                push_valid_receipt(&mut receipt);
                let actual = validate_semantic_snapshot(
                    receipt,
                    bindings(),
                    C2OneShotMacKey::from_test_bytes([0xa5; 32]),
                )
                .err()
                .expect("receipt fixture must fail closed");
                assert_eq!(actual, C2SemanticError::IncompleteDeletion, "{label}");
            }

            let mut malformed_binding =
                minted_pending_binding(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
            malformed_binding
                .proposal_row
                .as_object_mut()
                .unwrap()
                .remove("status_key");
            let mut receipt = default_applied_correction_snapshot();
            push_valid_receipt(&mut receipt);
            assert_error(
                receipt,
                vec![malformed_binding],
                C2SemanticError::InvalidRecord,
            );
        }

        type AppliedValueMutation = fn(&mut CorrectionProposal, &mut MemoryItem, &mut MemoryItem);

        fn recomputed_applied_snapshot(mutate: AppliedValueMutation) -> C2RawSnapshot {
            let (mut proposal, mut obsolete, mut replacement) = applied_values();
            mutate(&mut proposal, &mut obsolete, &mut replacement);
            let (pending_proposal, pending_obsolete, pending_replacement) =
                inverse_pending_pair(&proposal, &obsolete, &replacement);
            proposal.canonical_digest =
                correction_digest(&pending_proposal, &pending_obsolete, &pending_replacement)
                    .unwrap();
            proposal.applied_digest =
                Some(correction_digest(&proposal, &obsolete, &replacement).unwrap());
            correction_snapshot(&proposal, &obsolete, &replacement)
        }

        #[test]
        fn binding_rejects_every_recomputed_immutable_transition_family() {
            let mutations: [(&str, AppliedValueMutation); 9] = [
                ("replacement title", |_, _, replacement| {
                    replacement.title = "changed title".to_string()
                }),
                ("obsolete content", |_, obsolete, _| {
                    obsolete.content = "changed content".to_string()
                }),
                ("proposal created_at", |proposal, _, _| {
                    proposal.created_at = fixture_timestamp("2026-09-06T00:00:00.5Z")
                }),
                ("writer and proposer", |proposal, _, replacement| {
                    replacement.writer.model.model = "changed-model".to_string();
                    proposal.proposer = replacement.writer.clone();
                }),
                ("pair tags", |_, obsolete, replacement| {
                    obsolete.tags.push("changed".to_string());
                    replacement.tags.push("changed".to_string());
                }),
                ("pair review_after", |_, obsolete, replacement| {
                    let changed = Some(fixture_timestamp("2026-09-07T00:00:00Z"));
                    obsolete.review_after = changed;
                    replacement.review_after = changed;
                }),
                ("pair confidence", |_, obsolete, replacement| {
                    let changed = engram_core::memory::MemoryConfidence::new(0.7);
                    obsolete.confidence = changed;
                    replacement.confidence = changed;
                }),
                ("obsolete last_used_at", |_, obsolete, _| {
                    obsolete.last_used_at = Some(fixture_timestamp(FIXTURE_TIMESTAMP))
                }),
                ("prior evidence", |_, _, replacement| {
                    replacement.evidence.insert(0, unrelated_evidence())
                }),
            ];
            for (name, mutate) in mutations {
                let binding = minted_pending_binding(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
                assert_error(
                    recomputed_applied_snapshot(mutate),
                    vec![binding],
                    C2SemanticError::AppliedProvenanceUnproven,
                );
                let _ = name;
            }
        }

        #[test]
        fn allowed_transition_accepts_equality_and_advance_boundaries() {
            for (key, evidence, update, applied) in [
                (
                    0xc1,
                    FIXTURE_TIMESTAMP,
                    FIXTURE_TIMESTAMP,
                    FIXTURE_TIMESTAMP,
                ),
                (
                    0xc2,
                    "2026-09-06T00:00:01Z",
                    "2026-09-06T00:00:02Z",
                    "2026-09-06T00:00:03Z",
                ),
            ] {
                let binding = minted_pending_binding(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
                validate_semantic_snapshot(
                    applied_correction_snapshot(
                        FIXTURE_TIMESTAMP,
                        FIXTURE_TIMESTAMP,
                        evidence,
                        update,
                        update,
                        applied,
                    ),
                    vec![binding],
                    C2OneShotMacKey::from_test_bytes([key; 32]),
                )
                .unwrap();
            }
        }

        #[test]
        fn current_applied_failures_precede_binding_provenance() {
            type SnapshotMutation = fn(&mut C2RawSnapshot);
            let cases: [(&str, SnapshotMutation); 2] = [
                ("applied digest", |raw: &mut C2RawSnapshot| {
                    raw.correction_proposal[0]["proposal"]["applied_digest"] =
                        json!("0".repeat(64));
                    raw.correction_proposal[0]["snapshot_digest"] = json!("0".repeat(64));
                }),
                ("application summary", |raw: &mut C2RawSnapshot| {
                    raw.memory_item[1]["item"]["evidence"][1]["summary"] = json!("wrong");
                    raw.memory_item[1]["snapshot_digest"] = json!("0".repeat(64));
                }),
            ];
            for (name, mutate) in cases {
                let binding = minted_pending_binding(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
                let mut raw = default_applied_correction_snapshot();
                mutate(&mut raw);
                assert_error(raw, vec![binding], C2SemanticError::InconsistentProjection);
                let _ = name;
            }
        }

        #[test]
        fn proposal_role_collisions_and_attempted_chain_fail_closed() {
            let (base_proposal, base_obsolete, base_replacement) =
                pending_correction_values(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);

            let mut duplicate_pair =
                pending_correction_snapshot(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
            let mut second = base_proposal.clone();
            second.id = parse_id(OTHER_PROPOSAL_ID).unwrap();
            second.canonical_digest =
                correction_digest(&second, &base_obsolete, &base_replacement).unwrap();
            duplicate_pair
                .correction_proposal
                .push(fixture_proposal_row(serde_json::to_value(second).unwrap()));
            assert_error(
                duplicate_pair,
                Vec::new(),
                C2SemanticError::InconsistentProjection,
            );

            let mut repeated_obsolete =
                pending_correction_snapshot(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
            let (mut second, _, second_replacement) =
                pending_values_with_ids(OTHER_PROPOSAL_ID, OLD_ITEM_ID, OTHER_REPLACEMENT_ID);
            second.canonical_digest =
                correction_digest(&second, &base_obsolete, &second_replacement).unwrap();
            repeated_obsolete.memory_item.push(fixture_memory_row(
                serde_json::to_value(second_replacement).unwrap(),
            ));
            repeated_obsolete
                .correction_proposal
                .push(fixture_proposal_row(serde_json::to_value(second).unwrap()));
            assert_error(
                repeated_obsolete,
                Vec::new(),
                C2SemanticError::InconsistentProjection,
            );

            let mut chain = pending_correction_snapshot(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
            let (second, _, second_replacement) = pending_values_with_ids(
                OTHER_PROPOSAL_ID,
                base_replacement.id.to_string().as_str(),
                THIRD_MEMORY_ID,
            );
            chain.memory_item.push(fixture_memory_row(
                serde_json::to_value(second_replacement).unwrap(),
            ));
            chain
                .correction_proposal
                .push(fixture_proposal_row(serde_json::to_value(second).unwrap()));
            assert_error(chain, Vec::new(), C2SemanticError::InconsistentProjection);
        }

        fn replace_memory(raw: &mut C2RawSnapshot, index: usize, item: MemoryItem) {
            raw.memory_item[index] = fixture_memory_row(serde_json::to_value(item).unwrap());
        }

        fn fixture_memories(raw: &C2RawSnapshot) -> Vec<MemoryItem> {
            raw.memory_item
                .iter()
                .map(|row| strict_memory_row(row).unwrap())
                .collect()
        }

        #[test]
        fn supersedes_reference_duplicate_status_incoming_and_cycle_matrix() {
            let mut dangling = valid_snapshot();
            let mut items = fixture_memories(&dangling);
            items[0].supersedes.push(parse_id(DANGLING_ID).unwrap());
            replace_memory(&mut dangling, 0, items.remove(0));
            assert_error(
                dangling,
                Vec::new(),
                C2SemanticError::InconsistentProjection,
            );

            let mut duplicate = valid_snapshot();
            let mut items = fixture_memories(&duplicate);
            items[0].status = MemoryStatus::Superseded;
            items[1].supersedes = vec![items[0].id, items[0].id];
            replace_memory(&mut duplicate, 0, items[0].clone());
            replace_memory(&mut duplicate, 1, items[1].clone());
            assert_error(
                duplicate,
                Vec::new(),
                C2SemanticError::InconsistentProjection,
            );

            let mut wrong_status = valid_snapshot();
            let mut items = fixture_memories(&wrong_status);
            items[1].supersedes = vec![items[0].id];
            replace_memory(&mut wrong_status, 1, items[1].clone());
            assert_error(
                wrong_status,
                Vec::new(),
                C2SemanticError::InconsistentProjection,
            );

            let mut competing = valid_snapshot();
            let mut items = fixture_memories(&competing);
            items[0].status = MemoryStatus::Superseded;
            items[1].supersedes = vec![items[0].id];
            items[2].supersedes = vec![items[0].id];
            for (index, item) in items.iter().take(3).enumerate() {
                replace_memory(&mut competing, index, item.clone());
            }
            assert_error(
                competing,
                Vec::new(),
                C2SemanticError::InconsistentProjection,
            );

            let mut cycle = valid_snapshot();
            let mut items = fixture_memories(&cycle);
            items[0].status = MemoryStatus::Superseded;
            items[1].status = MemoryStatus::Superseded;
            items[0].supersedes = vec![items[1].id];
            items[1].supersedes = vec![items[0].id];
            replace_memory(&mut cycle, 0, items[0].clone());
            replace_memory(&mut cycle, 1, items[1].clone());
            assert_error(cycle, Vec::new(), C2SemanticError::InconsistentProjection);
        }

        #[test]
        fn missing_pair_and_dangling_markers_fail_closed() {
            let mut missing = pending_correction_snapshot(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
            missing.memory_item.pop();
            assert_error(missing, Vec::new(), C2SemanticError::InconsistentProjection);

            let mut dangling = valid_snapshot();
            let mut item = strict_memory_row(&dangling.memory_item[0]).unwrap();
            item.correction_proposal_id = Some(parse_id(DANGLING_ID).unwrap());
            replace_memory(&mut dangling, 0, item);
            assert_error(
                dangling,
                Vec::new(),
                C2SemanticError::InconsistentProjection,
            );
        }

        #[test]
        fn receipt_terminal_and_earlier_error_precedence_matrix() {
            for completed in [false, true] {
                let mut raw = valid_snapshot();
                let mut receipt = json!({
                    "record_id": "01890f5e-7b00-7000-8000-000000000070",
                    "deleted": completed,
                    "proposal_ids": [],
                    "pending_replacement_ids": [],
                    "unlocked_obsolete_ids": [],
                    "cleanup_complete": completed,
                    "created_at": FIXTURE_TIMESTAMP
                });
                if completed {
                    receipt["completed_at"] = json!(FIXTURE_TIMESTAMP);
                }
                raw.memory_forget_receipt.push(receipt);
                raw.memory_item[0]["snapshot_digest"] = json!("0".repeat(64));
                assert_error(raw, Vec::new(), C2SemanticError::IncompleteDeletion);
            }

            let mut malformed = valid_snapshot();
            malformed.memory_forget_receipt.push(json!({}));
            assert_error(malformed, Vec::new(), C2SemanticError::InvalidRecord);

            let mut secret = valid_snapshot();
            secret
                .memory_forget_receipt
                .push(json!({"unknown": "API_TOKEN=synthetic-secret"}));
            assert_error(secret, Vec::new(), C2SemanticError::SecretMaterial);
        }
    }

    // Paste inside `native_successor_semantic::tests`. All source inspection uses `include_str!`, so
    // these tests perform no runtime filesystem/process/network access.
    mod security_firewall_tests {
        use super::*;

        const SECRET_CANARY: &str = "API_TOKEN=synthetic-security-canary";
        const SECRET_VALUE: &str = "synthetic-security-canary";

        fn assert_error(
            raw: C2RawSnapshot,
            bindings: Vec<C2PriorCorrectionBinding>,
            expected: C2SemanticError,
        ) {
            assert_eq!(
                validate_semantic_snapshot(
                    raw,
                    bindings,
                    C2OneShotMacKey::from_test_bytes([0xd1; 32]),
                )
                .err(),
                Some(expected)
            );
        }

        #[test]
        fn secret_scan_covers_moved_binding_rows_unknown_keys_and_nested_credential_fields() {
            let mut proposal = minted_pending_binding(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
            proposal.proposal_row["unknown"] = json!(SECRET_CANARY);
            assert_error(
                valid_snapshot(),
                vec![proposal],
                C2SemanticError::SecretMaterial,
            );

            let mut obsolete = minted_pending_binding(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
            obsolete.obsolete_row["item"]
                .as_object_mut()
                .unwrap()
                .insert(SECRET_CANARY.to_string(), json!("safe"));
            assert_error(
                valid_snapshot(),
                vec![obsolete],
                C2SemanticError::SecretMaterial,
            );

            let mut replacement = minted_pending_binding(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
            replacement.replacement_row["item"]
                .as_object_mut()
                .unwrap()
                .insert("api-token".to_string(), json!({"nested": [SECRET_VALUE]}));
            assert_error(
                valid_snapshot(),
                vec![replacement],
                C2SemanticError::SecretMaterial,
            );
        }

        #[test]
        fn secret_scan_covers_representative_known_strings_in_all_nine_tables() {
            type SecretMutation = fn(&mut C2RawSnapshot);
            let mutations: [(&str, SecretMutation); 12] = [
                ("target", |raw| {
                    raw.target.project_name = SECRET_CANARY.to_string()
                }),
                ("memory projection", |raw| {
                    raw.memory_item[0]["model_key"] = json!(SECRET_CANARY)
                }),
                ("memory nested", |raw| {
                    raw.memory_item[0]["item"]["content"] = json!(SECRET_CANARY)
                }),
                ("proposal", |raw| {
                    raw.correction_proposal[0]["proposal"]["proposer"]["actor"] =
                        json!(SECRET_CANARY)
                }),
                ("receipt", |raw| {
                    raw.memory_forget_receipt
                        .push(json!({"unknown": SECRET_CANARY}))
                }),
                ("project", |raw| {
                    raw.work_project[0]["description"] = json!(SECRET_CANARY)
                }),
                ("task", |raw| {
                    raw.work_task[0]["jira_key"] = json!(SECRET_CANARY)
                }),
                ("repository", |raw| {
                    raw.git_repository[0]["repository"]["remote_url"] = json!(SECRET_CANARY)
                }),
                ("checkout", |raw| {
                    raw.local_checkout[0]["checkout"]["local_path"] = json!(SECRET_CANARY)
                }),
                ("component", |raw| {
                    raw.monorepo_component[0]["component"]["description"] = json!(SECRET_CANARY)
                }),
                ("link", |raw| {
                    raw.project_repository_link[0]["link"]["project_name"] = json!(SECRET_CANARY)
                }),
                ("unknown nested key", |raw| {
                    raw.work_task[0].as_object_mut().unwrap().insert(
                        "unknown".to_string(),
                        json!({"client-secret": SECRET_VALUE}),
                    );
                }),
            ];
            for (name, mutate) in mutations {
                let mut raw = rich_permutation_snapshot();
                mutate(&mut raw);
                let result = validate_semantic_snapshot(
                    raw,
                    Vec::new(),
                    C2OneShotMacKey::from_test_bytes([0xd2; 32]),
                );
                assert!(
                    matches!(result, Err(C2SemanticError::SecretMaterial)),
                    "{name}"
                );
            }
        }

        #[test]
        fn credential_url_query_fragment_and_colonless_userinfo_categories_are_exact() {
            for remote in [
                "https://synthetic:credential@example.test/repository?query=one",
                "https://synthetic:credential@example.test/repository#fragment",
            ] {
                let mut raw = valid_snapshot();
                raw.target.repository_remote = remote.to_string();
                assert_error(raw, Vec::new(), C2SemanticError::SecretMaterial);
            }

            for remote in [
                "https://synthetic@example.test/owner/repository.git",
                "https://github.com/ymeiri/engram.git?query=one",
                "https://github.com/ymeiri/engram.git#fragment",
            ] {
                let mut raw = valid_snapshot();
                raw.target.repository_remote = remote.to_string();
                assert_eq!(
                    validate_limit_and_secret_boundary(&raw, &[]).err(),
                    None,
                    "exact detector near-miss: {remote}"
                );
                assert_error(raw, Vec::new(), C2SemanticError::InvalidRecord);
            }
        }

        fn scrub_mac_values(value: &mut Value, parent: Option<&str>) {
            match value {
                Value::Object(object) => {
                    for (key, value) in object {
                        if key.ends_with("_mac") || parent == Some("table_macs") {
                            *value = json!("<authenticated>");
                        } else {
                            scrub_mac_values(value, Some(key));
                        }
                    }
                }
                Value::Array(values) => {
                    for value in values {
                        scrub_mac_values(value, parent);
                    }
                }
                _ => {}
            }
        }

        fn assert_mac_changed(left: &Value, right: &Value, label: &str) {
            let left = left.as_str().unwrap();
            let right = right.as_str().unwrap();
            assert_eq!(left.len(), 64, "{label}");
            assert_eq!(right.len(), 64, "{label}");
            assert_ne!(left, right, "{label}");
        }

        #[test]
        fn a_fresh_key_changes_every_exposed_mac_and_nothing_else() {
            let left = serde_json::to_value(
                validate_semantic_snapshot(
                    rich_permutation_snapshot(),
                    Vec::new(),
                    C2OneShotMacKey::from_test_bytes([0x11; 32]),
                )
                .unwrap()
                .audit,
            )
            .unwrap();
            let right = serde_json::to_value(
                validate_semantic_snapshot(
                    rich_permutation_snapshot(),
                    Vec::new(),
                    C2OneShotMacKey::from_test_bytes([0x22; 32]),
                )
                .unwrap()
                .audit,
            )
            .unwrap();

            for name in ["store_state_mac", "project_view_mac", "relay_candidate_mac"] {
                assert_mac_changed(&left[name], &right[name], name);
            }
            for table in C2TableTag::ALL {
                let name = std::str::from_utf8(table.as_bytes()).unwrap();
                assert_mac_changed(&left["table_macs"][name], &right["table_macs"][name], name);
            }
            for (index, (left, right)) in left["correction_edges"]
                .as_array()
                .unwrap()
                .iter()
                .zip(right["correction_edges"].as_array().unwrap())
                .enumerate()
            {
                for field in [
                    "proposal_record_mac",
                    "obsolete_record_mac",
                    "replacement_record_mac",
                ] {
                    assert_mac_changed(
                        &left[field],
                        &right[field],
                        &format!("edge {index} {field}"),
                    );
                }
            }
            for (index, (left, right)) in left["relay_candidates"]
                .as_array()
                .unwrap()
                .iter()
                .zip(right["relay_candidates"].as_array().unwrap())
                .enumerate()
            {
                assert_mac_changed(
                    &left["record_mac"],
                    &right["record_mac"],
                    &format!("relay {index}"),
                );
            }
            let mut left_without_macs = left;
            let mut right_without_macs = right;
            scrub_mac_values(&mut left_without_macs, None);
            scrub_mac_values(&mut right_without_macs, None);
            assert_eq!(left_without_macs, right_without_macs);
        }

        #[test]
        fn sanitized_success_and_all_error_formats_exclude_raw_canaries() {
            let canaries = [
                "private-title-canary",
                "private-content-canary",
                "private-evidence-canary",
                "private-writer-canary",
                "private-project-description-canary",
                FIXTURE_REMOTE,
                FIXTURE_CHECKOUT_PATH,
            ];
            let mut raw = valid_snapshot();
            mutate_fixture_memory(&mut raw, GLOBAL_MEMORY_ID, |item| {
                item["title"] = json!(canaries[0]);
                item["content"] = json!(canaries[1]);
                item["writer"]["actor"] = json!(canaries[3]);
                item["evidence"] = json!([{
                    "kind": "file", "target": canaries[2], "summary": canaries[2],
                    "excerpt": canaries[2], "observed_at": FIXTURE_TIMESTAMP
                }]);
            });
            raw.work_project[0]["description"] = json!(canaries[4]);
            let serialized = serde_json::to_string(
                &validate_semantic_snapshot(
                    raw,
                    Vec::new(),
                    C2OneShotMacKey::from_test_bytes([0xd3; 32]),
                )
                .unwrap()
                .audit,
            )
            .unwrap();
            for canary in canaries {
                assert!(!serialized.contains(canary), "leaked {canary}");
            }
            assert!(!serialized.contains("_sha256"));

            for error in [
                C2SemanticError::LimitExceeded {
                    table_overflow_mask: Some(0x01ff),
                },
                C2SemanticError::SecretMaterial,
                C2SemanticError::InvalidRecord,
                C2SemanticError::InconsistentProjection,
                C2SemanticError::IncompleteDeletion,
                C2SemanticError::AmbiguousIdentity,
                C2SemanticError::ScopeMismatch,
                C2SemanticError::AppliedProvenanceUnproven,
            ] {
                let formatted = format!("{error} {error:?}");
                for canary in canaries {
                    assert!(!formatted.contains(canary));
                }
            }
        }

        const SEMANTIC_SOURCE: &str = include_str!("native_successor_semantic.rs");
        const LIB_SOURCE: &str = include_str!("lib.rs");
        const PEER_SOURCES: &[(&str, &str)] = &[
            ("bin/native-c2b2a.rs", include_str!("bin/native-c2b2a.rs")),
            ("comparison.rs", include_str!("comparison.rs")),
            ("fixture.rs", include_str!("fixture.rs")),
            ("lean_probe.rs", include_str!("lean_probe.rs")),
            ("main.rs", include_str!("main.rs")),
            ("native_audit.rs", include_str!("native_audit.rs")),
            ("native_c2b2a.rs", include_str!("native_c2b2a.rs")),
            ("native_correction.rs", include_str!("native_correction.rs")),
            ("native_document.rs", include_str!("native_document.rs")),
            ("native_execution.rs", include_str!("native_execution.rs")),
            (
                "native_instructions_control.rs",
                include_str!("native_instructions_control.rs"),
            ),
            ("native_isolation.rs", include_str!("native_isolation.rs")),
            ("native_pilot.rs", include_str!("native_pilot.rs")),
            ("native_report.rs", include_str!("native_report.rs")),
            ("native_runner.rs", include_str!("native_runner.rs")),
            ("native_stale.rs", include_str!("native_stale.rs")),
            ("native_successor.rs", include_str!("native_successor.rs")),
            (
                concat!("native_successor_", "artifact.rs"),
                include_str!(concat!("native_successor_", "artifact.rs")),
            ),
            (
                "native_successor_core.rs",
                include_str!("native_successor_core.rs"),
            ),
            (
                concat!("native_successor_", "policy.rs"),
                include_str!(concat!("native_successor_", "policy.rs")),
            ),
            (
                concat!("native_successor_", "relay.rs"),
                include_str!(concat!("native_successor_", "relay.rs")),
            ),
            ("native_vm.rs", include_str!("native_vm.rs")),
            ("pilot.rs", include_str!("pilot.rs")),
            ("procedure_probe.rs", include_str!("procedure_probe.rs")),
            ("retrieval_probe.rs", include_str!("retrieval_probe.rs")),
            (
                "retrieval_probe_v2.rs",
                include_str!("retrieval_probe_v2.rs"),
            ),
            (
                "retrieval_probe_v3.rs",
                include_str!("retrieval_probe_v3.rs"),
            ),
            (
                "retrieval_probe_v4.rs",
                include_str!("retrieval_probe_v4.rs"),
            ),
            (
                "retrieval_probe_v5.rs",
                include_str!("retrieval_probe_v5.rs"),
            ),
            ("seed.rs", include_str!("seed.rs")),
        ];

        fn production_source() -> &'static str {
            SEMANTIC_SOURCE
                .split_once("#[cfg(test)]\nmod tests {")
                .expect("one outer test-module sentinel")
                .0
        }

        #[test]
        fn production_source_has_no_forbidden_capability_or_runnable_surface() {
            let production = production_source();
            let production_sha256 = lower_hex(&Sha256::digest(production.as_bytes()));
            assert_eq!(
                production_sha256,
                "c67ec74b936ce861e53e52d110b4635e4ba6fdfc9a3b0d3a93357315ca6d4b34",
                "reviewed C2A production prefix changed"
            );

            const SOURCE_SEAL_PLACEHOLDER: &str = concat!(
                "................................",
                "................................"
            );
            const REVIEWED_NORMALIZED_SOURCE_SHA256: &str =
                "89ee90fa325e41ab2d558e8dbb0bf66831611d5c52386872418ac9d66588686c";
            assert_eq!(
                SEMANTIC_SOURCE
                    .matches(REVIEWED_NORMALIZED_SOURCE_SHA256)
                    .count(),
                1,
                "reviewed C2A normalized source digest must occur exactly once"
            );
            let normalized_source = SEMANTIC_SOURCE.replacen(
                REVIEWED_NORMALIZED_SOURCE_SHA256,
                SOURCE_SEAL_PLACEHOLDER,
                1,
            );
            assert_eq!(
                lower_hex(&Sha256::digest(normalized_source.as_bytes())),
                REVIEWED_NORMALIZED_SOURCE_SHA256,
                "reviewed C2A complete source changed"
            );
            for forbidden in [
                "engram_store",
                "engram_index",
                "engram_mcp",
                "native_execution",
                "native_successor_core",
                concat!("native_successor_", "artifact"),
                concat!("native_successor_", "relay"),
                "std::fs",
                "std::net",
                "std::process",
                "std::env",
                "std::sync",
                "std::thread",
                "tokio::",
                "async fn",
                ".await",
                "Surreal<",
                "File::open",
                "OpenOptions",
                "TcpStream",
                "UdpSocket",
                "Command::",
                "rand::",
                "getrandom",
                "OsRng",
                "SystemTime",
                "Instant::now",
                "OffsetDateTime::now",
                "println!",
                "eprintln!",
                "tracing::",
                "log::",
            ] {
                assert!(
                    !production.contains(forbidden),
                    "forbidden production token: {forbidden}"
                );
            }
            assert_eq!(production.matches("validate_semantic_snapshot(").count(), 1);
            for declaration in [
                "struct C2RawSnapshot",
                "struct C2RawTarget",
                "struct C2ValidatedSnapshot",
                "struct C2ResolvedTarget",
                "struct C2PriorCorrectionBinding",
                "struct C2OneShotMacKey",
                "fn validate_semantic_snapshot",
            ] {
                let line = production
                    .lines()
                    .find(|line| line.contains(declaration))
                    .unwrap();
                assert!(
                    !line.trim_start().starts_with("pub"),
                    "public declaration: {line}"
                );
            }
        }

        #[test]
        fn key_and_success_types_remain_no_trait_move_only_and_by_value() {
            let production = production_source();
            for name in [
                "C2RawSnapshot",
                "C2RawTarget",
                "C2ValidatedSnapshot",
                "C2ResolvedTarget",
                "C2PriorCorrectionBinding",
                "C2OneShotMacKey",
            ] {
                let marker = format!("struct {name}");
                let position = production.find(&marker).unwrap();
                let block_start = production[..position]
                    .rfind("\n\n")
                    .map_or(0, |index| index + 2);
                assert!(
                    !production[block_start..position].contains("#[derive"),
                    "derive on {name}"
                );
                for trait_name in [
                    "Debug",
                    "Display",
                    "Clone",
                    "Copy",
                    "Serialize",
                    "Deserialize",
                ] {
                    assert!(!production.contains(&format!("impl {trait_name} for {name}")));
                    assert!(!production.contains(&format!("impl fmt::{trait_name} for {name}")));
                }
            }
            assert_eq!(production.matches("C2OneShotMacKey(").count(), 1);
            assert!(production
                .contains("#[cfg(test)]\n    fn from_test_bytes(bytes: [u8; 32]) -> Self"));
            assert_eq!(production.matches("Self(bytes)").count(), 1);
            assert!(production.contains("impl Drop for C2OneShotMacKey"));
            assert!(production.contains("impl Drop for C2MacPad"));
            assert!(production.matches("self.0.zeroize();").count() >= 2);

            let compact = production.split_whitespace().collect::<String>();
            assert!(compact.contains(
            "fnvalidate_semantic_snapshot(raw:C2RawSnapshot,prior_bindings:Vec<C2PriorCorrectionBinding>,mac_key:C2OneShotMacKey,)"
        ));

            fn consume_once(key: C2OneShotMacKey) {
                let _engine = C2MacEngine::consume(key);
            }
            consume_once(C2OneShotMacKey::from_test_bytes([0xd4; 32]));
        }

        #[test]
        fn module_is_private_and_no_peer_source_calls_or_names_c2a() {
            assert!(LIB_SOURCE.contains("\nmod native_successor_semantic;\n"));
            assert!(!LIB_SOURCE.contains("pub mod native_successor_semantic"));
            assert!(!LIB_SOURCE.contains("pub(crate) mod native_successor_semantic"));
            assert_eq!(LIB_SOURCE.matches("native_successor_semantic").count(), 1);
            for symbol in [
                "C2RawSnapshot",
                "C2ValidatedSnapshot",
                "C2PriorCorrectionBinding",
                "C2OneShotMacKey",
                "validate_semantic_snapshot",
            ] {
                assert!(
                    !LIB_SOURCE.contains(symbol),
                    "lib.rs names private symbol {symbol}"
                );
            }
            for (path, source) in PEER_SOURCES {
                for forbidden in [
                    "native_successor_semantic",
                    "C2RawSnapshot",
                    "C2ValidatedSnapshot",
                    "C2PriorCorrectionBinding",
                    "C2OneShotMacKey",
                    "validate_semantic_snapshot",
                ] {
                    assert!(!source.contains(forbidden), "{path} names {forbidden}");
                }
            }
        }

        #[test]
        fn key_and_pad_drop_probes_observe_zeroized_storage() {
            DROPPED_TEST_KEY_BYTES.with(|bytes| bytes.set(None));
            DROPPED_TEST_PAD_BYTES.with(|bytes| bytes.set(None));
            drop(C2OneShotMacKey::from_test_bytes([0x7f; 32]));
            drop(C2MacPad([0x7f; 64]));
            assert_eq!(
                DROPPED_TEST_KEY_BYTES.with(|bytes| bytes.get()),
                Some([0; 32])
            );
            assert_eq!(
                DROPPED_TEST_PAD_BYTES.with(|bytes| bytes.get()),
                Some([0; 64])
            );
        }
    }

    mod limits_scope_tests {
        use super::*;

        // Integrate inside native_successor_semantic.rs's existing `tests` module.

        fn limit_scope_filler_row(encoded_len: usize) -> Value {
            const MEMBER_COUNT: usize = 16;
            const FIXED_BYTES: usize = 9 + MEMBER_COUNT * (8 + 3 + 1 + 8);
            assert!((FIXED_BYTES..=FIXED_BYTES + MEMBER_COUNT * 4_096).contains(&encoded_len));
            let mut remaining = encoded_len - FIXED_BYTES;
            let mut object = Map::new();
            for index in 0..MEMBER_COUNT {
                let length = remaining.min(4_096);
                object.insert(format!("k{index:02}"), Value::String("x".repeat(length)));
                remaining -= length;
            }
            assert_eq!(remaining, 0);
            let row = Value::Object(object);
            assert_eq!(canonical_json_value(&row).unwrap().len(), encoded_len);
            row
        }

        fn limit_scope_dense_node_row() -> Value {
            let mut object = Map::new();
            for index in 0..64 {
                object.insert(format!("k{index:02}"), Value::Array(vec![Value::Null; 64]));
            }
            Value::Object(object)
        }

        fn limit_scope_tail_node_row(nulls: usize) -> Value {
            json!({"tail": vec![Value::Null; nulls]})
        }

        fn limit_scope_byte_cap_snapshot() -> C2RawSnapshot {
            let mut raw = empty_snapshot();
            raw.memory_item = (0..31)
                .map(|_| limit_scope_filler_row(MAX_ROW_BYTES as usize))
                .collect();
            raw.memory_item.push(limit_scope_filler_row(64_789));
            raw
        }

        fn limit_scope_mint_bindings(count: usize) -> Vec<C2PriorCorrectionBinding> {
            assert!(count <= MAX_BINDINGS);
            let mut raw = valid_snapshot();
            raw.memory_item.clear();
            raw.correction_proposal.clear();
            for index in 0..count {
                let proposal_id =
                    format!("01890f5e-7b00-7000-8000-{:012x}", 0x100_u64 + index as u64);
                let obsolete_id =
                    format!("01890f5e-7b00-7000-8000-{:012x}", 0x200_u64 + index as u64);
                let replacement_id =
                    format!("01890f5e-7b00-7000-8000-{:012x}", 0x300_u64 + index as u64);
                push_pending_correction(&mut raw, &proposal_id, &obsolete_id, &replacement_id);
            }
            validate_semantic_snapshot(
                raw,
                Vec::new(),
                C2OneShotMacKey::from_test_bytes([0xa1; 32]),
            )
            .unwrap()
            .pending_bindings
        }

        fn limit_scope_push_memory(raw: &mut C2RawSnapshot, id: &str, scope: Value, status: &str) {
            let mut item = fixture_memory_item(id, &format!("scope {id}"), scope);
            item["status"] = json!(status);
            if status == "archived" {
                item["archive"] = json!({
                    "reason": "scope matrix archive",
                    "archived_by": null,
                    "archived_at": FIXTURE_TIMESTAMP
                });
            }
            raw.memory_item.push(fixture_memory_row(item));
        }

        fn limit_scope_snapshot() -> C2RawSnapshot {
            const OTHER_PROJECT: &str = "01890f5e-7b00-7000-8000-000000000090";
            const OTHER_REPOSITORY: &str = "01890f5e-7b00-7000-8000-000000000091";
            const OTHER_CHECKOUT: &str = "01890f5e-7b00-7000-8000-000000000092";
            const OTHER_TASK: &str = "01890f5e-7b00-7000-8000-000000000093";
            const OTHER_REMOTE: &str = "https://github.com/example/scope-other.git";
            let mut raw = valid_snapshot();
            raw.memory_item.clear();
            push_fixture_project(&mut raw, OTHER_PROJECT, "scope-other");
            push_fixture_repository(&mut raw, OTHER_REPOSITORY, "scope-other", OTHER_REMOTE);
            push_fixture_checkout(
                &mut raw,
                OTHER_CHECKOUT,
                OTHER_REPOSITORY,
                "/workspace/scope-other",
            );
            push_fixture_task(
                &mut raw,
                OTHER_TASK,
                OTHER_PROJECT,
                "scope other task",
                Some("OTHER-42"),
            );

            let active = [
                (
                    "01890f5e-7b00-7000-8000-000000000080",
                    json!({"type": "global"}),
                ),
                (
                    "01890f5e-7b00-7000-8000-000000000081",
                    json!({"type": "user"}),
                ),
                (
                    "01890f5e-7b00-7000-8000-000000000082",
                    json!({"type": "project", "project_id": PROJECT_ID, "project_name": "engram"}),
                ),
                (
                    "01890f5e-7b00-7000-8000-000000000083",
                    json!({"type": "project", "project_id": null, "project_name": "engram"}),
                ),
                (
                    "01890f5e-7b00-7000-8000-000000000084",
                    json!({
                        "type": "task", "project_id": null, "project_name": null,
                        "task_id": TASK_ID, "task_name": "ENG-42"
                    }),
                ),
                (
                    "01890f5e-7b00-7000-8000-000000000085",
                    json!({
                        "type": "task", "project_id": PROJECT_ID, "project_name": null,
                        "task_id": null, "task_name": "ENG-42"
                    }),
                ),
                (
                    "01890f5e-7b00-7000-8000-000000000086",
                    json!({
                        "type": "task", "project_id": null, "project_name": null,
                        "task_id": null, "task_name": "ENG-42"
                    }),
                ),
                (
                    "01890f5e-7b00-7000-8000-000000000087",
                    json!({
                        "type": "repository", "repository_id": REPOSITORY_ID,
                        "remote_url": null, "local_path": null
                    }),
                ),
                (
                    "01890f5e-7b00-7000-8000-000000000088",
                    json!({
                        "type": "repository", "repository_id": null,
                        "remote_url": FIXTURE_REMOTE, "local_path": null
                    }),
                ),
                (
                    "01890f5e-7b00-7000-8000-000000000089",
                    json!({
                        "type": "repository", "repository_id": null,
                        "remote_url": null, "local_path": FIXTURE_CHECKOUT_PATH
                    }),
                ),
                (
                    "01890f5e-7b00-7000-8000-00000000008a",
                    json!({
                        "type": "repository", "repository_id": REPOSITORY_ID,
                        "remote_url": FIXTURE_REMOTE, "local_path": FIXTURE_CHECKOUT_PATH
                    }),
                ),
                (
                    "01890f5e-7b00-7000-8000-00000000008b",
                    json!({
                        "type": "project", "project_id": OTHER_PROJECT,
                        "project_name": "scope-other"
                    }),
                ),
                (
                    "01890f5e-7b00-7000-8000-00000000008c",
                    json!({
                        "type": "task", "project_id": OTHER_PROJECT,
                        "project_name": "scope-other", "task_id": OTHER_TASK,
                        "task_name": "OTHER-42"
                    }),
                ),
                (
                    "01890f5e-7b00-7000-8000-00000000008d",
                    json!({
                        "type": "repository", "repository_id": OTHER_REPOSITORY,
                        "remote_url": OTHER_REMOTE, "local_path": "/workspace/scope-other"
                    }),
                ),
                (
                    "01890f5e-7b00-7000-8000-00000000008e",
                    json!({"type": "entity", "entity_id": null, "entity_name": "scope entity"}),
                ),
                (
                    "01890f5e-7b00-7000-8000-00000000008f",
                    json!({"type": "session", "session_id": "01890f5e-7b00-7000-8000-000000000094"}),
                ),
                (
                    "01890f5e-7b00-7000-8000-0000000000a0",
                    json!({"type": "custom", "name": "scope custom"}),
                ),
            ];
            for (id, scope) in active {
                limit_scope_push_memory(&mut raw, id, scope, "active");
            }
            for (id, status) in [
                ("01890f5e-7b00-7000-8000-0000000000a1", "needs_review"),
                ("01890f5e-7b00-7000-8000-0000000000a2", "superseded"),
                ("01890f5e-7b00-7000-8000-0000000000a3", "archived"),
                ("01890f5e-7b00-7000-8000-0000000000a4", "rejected"),
            ] {
                limit_scope_push_memory(&mut raw, id, json!({"type": "global"}), status);
            }
            push_pending_correction(
                &mut raw,
                "01890f5e-7b00-7000-8000-0000000000b0",
                "01890f5e-7b00-7000-8000-0000000000b1",
                "01890f5e-7b00-7000-8000-0000000000b2",
            );
            raw
        }

        fn limit_scope_ids(validated: &C2ValidatedSnapshot) -> Vec<String> {
            validated
                .applicable_memory_ids
                .iter()
                .map(ToString::to_string)
                .collect()
        }

        #[test]
        fn actual_virtual_input_global_byte_and_node_caps_are_inclusive() {
            let mut bytes = limit_scope_byte_cap_snapshot();
            assert_eq!(
                account_virtual_input(&bytes, &[]).unwrap().bytes,
                MAX_RAW_BYTES
            );
            bytes.memory_item[31] = limit_scope_filler_row(64_790);
            assert_limit_none(account_virtual_input(&bytes, &[]));

            let mut nodes = empty_snapshot();
            nodes.memory_item = (0..31).map(|_| limit_scope_dense_node_row()).collect();
            nodes.memory_item.push(limit_scope_tail_node_row(55));
            let accounting = account_virtual_input(&nodes, &[]).unwrap();
            assert_eq!(accounting.nodes, MAX_RAW_NODES);
            assert!(accounting.bytes < MAX_RAW_BYTES);
            nodes.memory_item[31] = limit_scope_tail_node_row(56);
            assert_limit_none(account_virtual_input(&nodes, &[]));
        }

        #[test]
        fn prior_binding_count_row_limit_and_global_recharge_are_real() {
            let mut bindings = limit_scope_mint_bindings(MAX_BINDINGS);
            assert_eq!(bindings.len(), MAX_BINDINGS);
            account_virtual_input(&empty_snapshot(), &bindings).unwrap();
            bindings.push(minted_pending_binding(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP));
            assert_limit_none(account_virtual_input(&empty_snapshot(), &bindings));

            for row_index in 0..3 {
                let mut exact = minted_pending_binding(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
                match row_index {
                    0 => exact.proposal_row = limit_scope_filler_row(MAX_ROW_BYTES as usize),
                    1 => exact.obsolete_row = limit_scope_filler_row(MAX_ROW_BYTES as usize),
                    2 => exact.replacement_row = limit_scope_filler_row(MAX_ROW_BYTES as usize),
                    _ => unreachable!("fixed retained-row index"),
                }
                account_virtual_input(&empty_snapshot(), std::slice::from_ref(&exact)).unwrap();
                match row_index {
                    0 => exact.proposal_row = limit_scope_filler_row(MAX_ROW_BYTES as usize + 1),
                    1 => exact.obsolete_row = limit_scope_filler_row(MAX_ROW_BYTES as usize + 1),
                    2 => exact.replacement_row = limit_scope_filler_row(MAX_ROW_BYTES as usize + 1),
                    _ => unreachable!("fixed retained-row index"),
                }
                assert_limit_none(account_virtual_input(
                    &empty_snapshot(),
                    std::slice::from_ref(&exact),
                ));
            }

            let raw_at_cap = limit_scope_byte_cap_snapshot();
            let binding = minted_pending_binding(FIXTURE_TIMESTAMP, FIXTURE_TIMESTAMP);
            assert_limit_none(account_virtual_input(
                &raw_at_cap,
                std::slice::from_ref(&binding),
            ));
        }

        #[test]
        fn every_scope_form_and_lifecycle_has_exact_candidate_membership() {
            let validated = validate_fixture(limit_scope_snapshot());
            assert_eq!(
                limit_scope_ids(&validated),
                [
                    "01890f5e-7b00-7000-8000-000000000080",
                    "01890f5e-7b00-7000-8000-000000000081",
                    "01890f5e-7b00-7000-8000-000000000082",
                    "01890f5e-7b00-7000-8000-000000000083",
                    "01890f5e-7b00-7000-8000-000000000084",
                    "01890f5e-7b00-7000-8000-000000000085",
                    "01890f5e-7b00-7000-8000-000000000087",
                    "01890f5e-7b00-7000-8000-000000000088",
                    "01890f5e-7b00-7000-8000-000000000089",
                    "01890f5e-7b00-7000-8000-00000000008a",
                    "01890f5e-7b00-7000-8000-0000000000b1",
                ]
            );
            assert_eq!(
                validated
                    .audit
                    .relay_candidates
                    .iter()
                    .map(|candidate| candidate.memory_id.clone())
                    .collect::<Vec<_>>(),
                limit_scope_ids(&validated)
            );
            assert_eq!(validated.target_affecting_correction_ids.len(), 1);
            assert_eq!(validated.audit.correction_edges.len(), 1);
            assert_eq!(validated.audit.correction_edges[0].status, "pending");
            assert!(!limit_scope_ids(&validated)
                .iter()
                .any(|id| id == "01890f5e-7b00-7000-8000-0000000000b2"));
        }

        #[test]
        fn absent_target_task_and_task_name_only_are_never_adopted() {
            let mut raw = limit_scope_snapshot();
            raw.target.task_id = None;
            raw.target.task_name = None;
            let validated = validate_fixture(raw);
            let ids = limit_scope_ids(&validated);
            for excluded in [
                "01890f5e-7b00-7000-8000-000000000084",
                "01890f5e-7b00-7000-8000-000000000085",
                "01890f5e-7b00-7000-8000-000000000086",
                "01890f5e-7b00-7000-8000-00000000008c",
            ] {
                assert!(!ids.iter().any(|id| id == excluded));
            }

            let with_task = validate_fixture(limit_scope_snapshot());
            assert!(!limit_scope_ids(&with_task)
                .iter()
                .any(|id| id == "01890f5e-7b00-7000-8000-000000000086"));
        }

        #[test]
        fn applicable_and_excluded_rows_change_only_their_authenticated_domains() {
            let baseline = audit_value(limit_scope_snapshot());
            for (id, project_changed, relay_changed) in [
                ("01890f5e-7b00-7000-8000-000000000081", true, true),
                ("01890f5e-7b00-7000-8000-000000000087", true, true),
                ("01890f5e-7b00-7000-8000-000000000086", false, false),
                ("01890f5e-7b00-7000-8000-00000000008b", false, false),
                ("01890f5e-7b00-7000-8000-00000000008c", false, false),
                ("01890f5e-7b00-7000-8000-00000000008d", false, false),
                ("01890f5e-7b00-7000-8000-0000000000a1", false, false),
                ("01890f5e-7b00-7000-8000-0000000000a2", false, false),
                ("01890f5e-7b00-7000-8000-0000000000a3", false, false),
                ("01890f5e-7b00-7000-8000-0000000000a4", false, false),
            ] {
                let mut changed = limit_scope_snapshot();
                mutate_fixture_memory(&mut changed, id, |item| {
                    item["content"] = json!(format!("authenticated mutation {id}"));
                });
                assert_mac_domain_changes(
                    &baseline,
                    &audit_value(changed),
                    "memory_item",
                    project_changed,
                    relay_changed,
                );
            }
        }

        #[test]
        fn pending_replacement_is_not_a_candidate_but_is_bound_into_its_project_edge() {
            const OBSOLETE: &str = "01890f5e-7b00-7000-8000-0000000000b1";
            const REPLACEMENT: &str = "01890f5e-7b00-7000-8000-0000000000b2";
            let baseline = audit_value(limit_scope_snapshot());
            let mut changed = limit_scope_snapshot();
            let proposal_index = changed
                .correction_proposal
                .iter()
                .position(|row| {
                    row["record_id"].as_str() == Some("01890f5e-7b00-7000-8000-0000000000b0")
                })
                .unwrap();
            let obsolete = changed
                .memory_item
                .iter()
                .find(|row| row["record_id"].as_str() == Some(OBSOLETE))
                .map(strict_memory_row)
                .unwrap()
                .unwrap();
            let replacement_index = changed
                .memory_item
                .iter()
                .position(|row| row["record_id"].as_str() == Some(REPLACEMENT))
                .unwrap();
            let mut replacement =
                strict_memory_row(&changed.memory_item[replacement_index]).unwrap();
            replacement.content = "changed pending replacement".to_string();
            changed.memory_item[replacement_index] =
                fixture_memory_row(serde_json::to_value(&replacement).unwrap());
            let mut proposal =
                strict_proposal_row(&changed.correction_proposal[proposal_index]).unwrap();
            proposal.canonical_digest =
                correction_digest(&proposal, &obsolete, &replacement).unwrap();
            changed.correction_proposal[proposal_index] =
                fixture_proposal_row(serde_json::to_value(proposal).unwrap());
            let changed = audit_value(changed);

            assert_ne!(
                baseline["table_macs"]["memory_item"],
                changed["table_macs"]["memory_item"]
            );
            assert_ne!(
                baseline["table_macs"]["correction_proposal"],
                changed["table_macs"]["correction_proposal"]
            );
            assert_ne!(baseline["store_state_mac"], changed["store_state_mac"]);
            assert_ne!(baseline["project_view_mac"], changed["project_view_mac"]);
            assert_eq!(
                baseline["relay_candidate_mac"],
                changed["relay_candidate_mac"]
            );
            assert_eq!(baseline["relay_candidates"], changed["relay_candidates"]);
        }
    }
}
