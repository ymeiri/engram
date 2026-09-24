#![allow(dead_code)]

use super::{
    account_virtual_input, is_credential_field, is_secret_field_string, likely_secret_in_string,
    normalize_remote, parse_id, row_schema, scope_key, snapshot_digest, strict_record_stage,
    validate_limit_and_secret_boundary, validate_semantic_snapshot, C2OneShotMacKey, C2RawSnapshot,
    C2RawTarget, C2SemanticError, C2ValidatedSnapshot, Meter, RawAccounting, RowBytes, SchemaPath,
    TableKind, MAX_EVIDENCE_TAGS_SUPERSEDES, MAX_LARGE_SCALAR_BYTES, MAX_NAME_SCALAR_BYTES,
    MAX_OBJECT_KEYS, MAX_ORDINARY_SCALAR_BYTES, MAX_PREREQUISITE_KEY_PATH, MAX_PROCEDURE_VECTOR,
    MAX_RAW_BYTES, MAX_VECTOR_ELEMENTS, TABLE_KINDS,
};
use engram_core::memory::{
    ArchiveMetadata, ClaimOrigin, CorrectionProposal, CorrectionProposalStatus, EvidenceKind,
    EvidenceRef, Harness, MemoryItem, MemoryKind, MemoryScope, MemoryStatus, ModelIdentity,
    ProcedureCard, ProcedurePrerequisite, ProcedurePrerequisiteSource, ProcedureVerification,
    WriterProvenance,
};
use engram_core::repository::{
    GitRepository, LocalCheckout, MonorepoComponent, ProjectRepositoryLink, ProjectRepositoryRole,
    RepositoryProvider,
};
use engram_core::work::{Project, ProjectStatus, Task, TaskPriority, TaskStatus};
use serde_json::{Map as JsonMap, Value as JsonValue};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::marker::PhantomData;
use surrealdb_core::dbs::{Capabilities, Session};
use surrealdb_core::kvs::Datastore;
use surrealdb_core::sql::{
    Array as NativeArray, Data, Datetime as NativeDatetime, Field, Id as NativeId,
    Number as NativeNumber, Object as NativeObject, Output, Query, Statement, Strand, Subquery,
    Thing, Value as NativeValue,
};
use time::OffsetDateTime;

const INIT_QUERY: &str = "BEGIN TRANSACTION;\nINSERT INTO memory_item $memory_item RETURN NONE;\nINSERT INTO correction_proposal $correction_proposal RETURN NONE;\nINSERT INTO memory_forget_receipt $memory_forget_receipt RETURN NONE;\nINSERT INTO work_project $work_project RETURN NONE;\nINSERT INTO work_task $work_task RETURN NONE;\nINSERT INTO git_repository $git_repository RETURN NONE;\nINSERT INTO local_checkout $local_checkout RETURN NONE;\nINSERT INTO monorepo_component $monorepo_component RETURN NONE;\nINSERT INTO project_repository_link $project_repository_link RETURN NONE;\nCOMMIT TRANSACTION;";
const INIT_QUERY_BYTES: usize = 572;
const INIT_QUERY_SHA256: &str = "9f6c41b6c7d0006db0a03f29fd951bb3c81df19946748faaac768af013c98a05";

const READ_QUERY: &str = "RETURN {\n  memory_item: (SELECT * FROM memory_item LIMIT 65),\n  correction_proposal: (SELECT * FROM correction_proposal LIMIT 33),\n  memory_forget_receipt: (SELECT * FROM memory_forget_receipt LIMIT 33),\n  work_project: (SELECT * FROM work_project LIMIT 9),\n  work_task: (SELECT * FROM work_task LIMIT 65),\n  git_repository: (SELECT * FROM git_repository LIMIT 17),\n  local_checkout: (SELECT * FROM local_checkout LIMIT 33),\n  monorepo_component: (SELECT * FROM monorepo_component LIMIT 65),\n  project_repository_link: (SELECT * FROM project_repository_link LIMIT 65)\n};";
const READ_QUERY_BYTES: usize = 570;
const READ_QUERY_SHA256: &str = "cf1034a335af240c0726e5385207acc4e4e49273dc016891138c863e8ba096f5";

#[cfg(test)]
const REPLACE_QUERY: &str = "BEGIN TRANSACTION;\nDELETE memory_item RETURN NONE;\nDELETE correction_proposal RETURN NONE;\nDELETE memory_forget_receipt RETURN NONE;\nDELETE work_project RETURN NONE;\nDELETE work_task RETURN NONE;\nDELETE git_repository RETURN NONE;\nDELETE local_checkout RETURN NONE;\nDELETE monorepo_component RETURN NONE;\nDELETE project_repository_link RETURN NONE;\nINSERT INTO memory_item $memory_item RETURN NONE;\nINSERT INTO correction_proposal $correction_proposal RETURN NONE;\nINSERT INTO memory_forget_receipt $memory_forget_receipt RETURN NONE;\nINSERT INTO work_project $work_project RETURN NONE;\nINSERT INTO work_task $work_task RETURN NONE;\nINSERT INTO git_repository $git_repository RETURN NONE;\nINSERT INTO local_checkout $local_checkout RETURN NONE;\nINSERT INTO monorepo_component $monorepo_component RETURN NONE;\nINSERT INTO project_repository_link $project_repository_link RETURN NONE;\nCOMMIT TRANSACTION;";
#[cfg(test)]
const REPLACE_QUERY_BYTES: usize = 902;
#[cfg(test)]
const REPLACE_QUERY_SHA256: &str =
    "0a30e2e0a890b16abedf2cd91282d49cd55c82885b85cbc28b6e90e2d161211d";

const COMPLETE_GENERATION_UPPER_BOUND: u64 = 16 * MAX_RAW_BYTES;

struct C2CompleteGeneration {
    target: C2RawTarget,
    memory_item: Vec<MemoryItem>,
    correction_proposal: Vec<CorrectionProposal>,
    work_project: Vec<Project>,
    work_task: Vec<Task>,
    git_repository: Vec<GitRepository>,
    local_checkout: Vec<LocalCheckout>,
    monorepo_component: Vec<MonorepoComponent>,
    project_repository_link: Vec<ProjectRepositoryLink>,
}

struct C2Empty;
struct C2Ready;

struct C2MemoryOwner<State> {
    datastore: Datastore,
    session: Session,
    target: C2RawTarget,
    initial_accounting: RawAccounting,
    state: PhantomData<State>,
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

assert_not_impl_any!(C2CompleteGeneration:
    Clone,
    Copy,
    fmt::Debug,
    fmt::Display,
    serde::Serialize,
    serde::Deserialize<'static>
);
assert_not_impl_any!(C2Empty:
    Clone,
    Copy,
    fmt::Debug,
    fmt::Display,
    serde::Serialize,
    serde::Deserialize<'static>
);
assert_not_impl_any!(C2Ready:
    Clone,
    Copy,
    fmt::Debug,
    fmt::Display,
    serde::Serialize,
    serde::Deserialize<'static>
);
assert_not_impl_any!(C2MemoryOwner<C2Empty>:
    Clone,
    Copy,
    fmt::Debug,
    fmt::Display,
    serde::Serialize,
    serde::Deserialize<'static>
);
assert_not_impl_any!(C2MemoryOwner<C2Ready>:
    Clone,
    Copy,
    fmt::Debug,
    fmt::Display,
    serde::Serialize,
    serde::Deserialize<'static>
);
assert_not_impl_any!(C2OneShotMacKey:
    Clone,
    Copy,
    fmt::Debug,
    fmt::Display,
    serde::Serialize,
    serde::Deserialize<'static>
);

#[derive(Debug, PartialEq, Eq)]
enum C2AcquisitionError {
    LimitExceeded { table_overflow_mask: Option<u16> },
    SecretMaterial,
    InvalidGeneration,
    QueryContractInvalid,
    EngineFailure,
    EngineOutcomeUncertain,
    InvalidResponse,
    UnsupportedNativeValue,
    EntropyUnavailable,
    InvalidRecord,
    InconsistentProjection,
    IncompleteDeletion,
    AmbiguousIdentity,
    ScopeMismatch,
    AppliedProvenanceUnproven,
}

impl fmt::Display for C2AcquisitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::LimitExceeded { .. } => "limit_exceeded",
            Self::SecretMaterial => "secret_material",
            Self::InvalidGeneration => "invalid_generation",
            Self::QueryContractInvalid => "query_contract_invalid",
            Self::EngineFailure => "engine_failure",
            Self::EngineOutcomeUncertain => "engine_outcome_uncertain",
            Self::InvalidResponse => "invalid_response",
            Self::UnsupportedNativeValue => "unsupported_native_value",
            Self::EntropyUnavailable => "entropy_unavailable",
            Self::InvalidRecord => "invalid_record",
            Self::InconsistentProjection => "inconsistent_projection",
            Self::IncompleteDeletion => "incomplete_deletion",
            Self::AmbiguousIdentity => "ambiguous_identity",
            Self::ScopeMismatch => "scope_mismatch",
            Self::AppliedProvenanceUnproven => "applied_provenance_unproven",
        })
    }
}

impl std::error::Error for C2AcquisitionError {}

type AcquisitionResult<T> = Result<T, C2AcquisitionError>;

impl From<C2SemanticError> for C2AcquisitionError {
    fn from(error: C2SemanticError) -> Self {
        match error {
            C2SemanticError::LimitExceeded {
                table_overflow_mask,
            } => Self::LimitExceeded {
                table_overflow_mask,
            },
            C2SemanticError::SecretMaterial => Self::SecretMaterial,
            C2SemanticError::InvalidRecord => Self::InvalidRecord,
            C2SemanticError::InconsistentProjection => Self::InconsistentProjection,
            C2SemanticError::IncompleteDeletion => Self::IncompleteDeletion,
            C2SemanticError::AmbiguousIdentity => Self::AmbiguousIdentity,
            C2SemanticError::ScopeMismatch => Self::ScopeMismatch,
            C2SemanticError::AppliedProvenanceUnproven => Self::AppliedProvenanceUnproven,
        }
    }
}

fn limit_error<T>() -> AcquisitionResult<T> {
    Err(C2AcquisitionError::LimitExceeded {
        table_overflow_mask: None,
    })
}

enum ExactContainer {
    Object {
        remaining: usize,
        awaiting_value: bool,
    },
    Array {
        remaining: usize,
    },
}

struct BorrowedExactMeter {
    meter: Meter,
    row: Option<RowBytes>,
    stack: Vec<ExactContainer>,
    root_seen: bool,
    invalid: bool,
    lower_bound: bool,
}

impl BorrowedExactMeter {
    fn new() -> Self {
        Self {
            meter: Meter::new(),
            row: None,
            stack: Vec::with_capacity(18),
            root_seen: false,
            invalid: false,
            lower_bound: false,
        }
    }

    fn add_bytes(&mut self, amount: u64) -> AcquisitionResult<()> {
        self.meter.add_bytes(amount)?;
        if let Some(row) = &mut self.row {
            row.add(amount)?;
        }
        Ok(())
    }

    fn start_value(&mut self) -> AcquisitionResult<u64> {
        let depth =
            u64::try_from(self.stack.len()).map_err(|_| C2AcquisitionError::LimitExceeded {
                table_overflow_mask: None,
            })?;
        match self.stack.last_mut() {
            Some(ExactContainer::Object {
                remaining,
                awaiting_value,
            }) => {
                if !*awaiting_value || *remaining == 0 {
                    return Err(C2AcquisitionError::InvalidGeneration);
                }
                *awaiting_value = false;
                *remaining -= 1;
            }
            Some(ExactContainer::Array { remaining }) => {
                if *remaining == 0 {
                    return Err(C2AcquisitionError::InvalidGeneration);
                }
                *remaining -= 1;
            }
            None => {
                if self.root_seen {
                    return Err(C2AcquisitionError::InvalidGeneration);
                }
                self.root_seen = true;
            }
        }
        self.meter.enter_node(depth)?;
        Ok(depth)
    }

    fn close_exhausted(&mut self) {
        while self.stack.last().is_some_and(|container| match container {
            ExactContainer::Object {
                remaining,
                awaiting_value,
            } => *remaining == 0 && !*awaiting_value,
            ExactContainer::Array { remaining } => *remaining == 0,
        }) {
            self.stack.pop();
        }
    }

    fn container(&mut self, container: ExactContainer, count: usize) -> AcquisitionResult<()> {
        let _ = self.start_value()?;
        self.add_bytes(1 + 8)?;
        if count == 0 {
            self.close_exhausted();
        } else {
            self.stack.push(container);
        }
        Ok(())
    }

    fn object(&mut self, count: usize) -> AcquisitionResult<()> {
        self.container(
            ExactContainer::Object {
                remaining: count,
                awaiting_value: false,
            },
            count,
        )
    }

    fn array(&mut self, count: usize) -> AcquisitionResult<()> {
        self.container(ExactContainer::Array { remaining: count }, count)
    }

    fn key(&mut self, length: usize) -> AcquisitionResult<()> {
        let depth =
            u64::try_from(self.stack.len()).map_err(|_| C2AcquisitionError::LimitExceeded {
                table_overflow_mask: None,
            })?;
        match self.stack.last_mut() {
            Some(ExactContainer::Object {
                remaining,
                awaiting_value,
            }) if *remaining > 0 && !*awaiting_value => *awaiting_value = true,
            _ => return Err(C2AcquisitionError::InvalidGeneration),
        }
        self.meter.enter_node(depth)?;
        self.add_bytes(
            8_u64
                .checked_add(u64::try_from(length).map_err(|_| {
                    C2AcquisitionError::LimitExceeded {
                        table_overflow_mask: None,
                    }
                })?)
                .ok_or(C2AcquisitionError::LimitExceeded {
                    table_overflow_mask: None,
                })?,
        )
    }

    fn scalar(&mut self, bytes: u64) -> AcquisitionResult<()> {
        let _ = self.start_value()?;
        self.add_bytes(bytes)?;
        self.close_exhausted();
        Ok(())
    }

    fn string(&mut self, length: usize) -> AcquisitionResult<()> {
        self.scalar(
            1_u64
                .checked_add(8)
                .and_then(|base| base.checked_add(u64::try_from(length).ok()?))
                .ok_or(C2AcquisitionError::LimitExceeded {
                    table_overflow_mask: None,
                })?,
        )
    }

    fn number(&mut self, spelling: &str) -> AcquisitionResult<()> {
        self.string(spelling.len())
    }

    fn timestamp(&mut self, value: OffsetDateTime) -> AcquisitionResult<()> {
        let year = value.year();
        let offset = value.offset();
        let valid = (0..10_000).contains(&year) && offset.seconds_past_minute() == 0;
        self.invalid |= !valid;
        self.lower_bound |= !valid;

        // RFC3339's pinned spelling is four-digit date + time (19), an optional decimal point
        // plus the nonzero nanosecond digits with trailing zeroes removed, and Z or +/-HH:MM.
        // Invalid values use the minimum 20-byte canonical-string event as a lower bound only.
        let mut length = 19_usize;
        let mut nanoseconds = value.nanosecond();
        if nanoseconds != 0 {
            let mut trailing_zeroes = 0_usize;
            while nanoseconds % 10 == 0 {
                nanoseconds /= 10;
                trailing_zeroes =
                    trailing_zeroes
                        .checked_add(1)
                        .ok_or(C2AcquisitionError::LimitExceeded {
                            table_overflow_mask: None,
                        })?;
            }
            let fraction_digits =
                9_usize
                    .checked_sub(trailing_zeroes)
                    .ok_or(C2AcquisitionError::LimitExceeded {
                        table_overflow_mask: None,
                    })?;
            length = length
                .checked_add(1)
                .and_then(|length| length.checked_add(fraction_digits))
                .ok_or(C2AcquisitionError::LimitExceeded {
                    table_overflow_mask: None,
                })?;
        }
        length = length
            .checked_add(if offset.is_utc() { 1 } else { 6 })
            .ok_or(C2AcquisitionError::LimitExceeded {
                table_overflow_mask: None,
            })?;
        if !valid {
            length = 20;
        }
        self.string(length)
    }

    fn begin_row(&mut self) -> AcquisitionResult<()> {
        if self.row.is_some() {
            return Err(C2AcquisitionError::InvalidGeneration);
        }
        self.row = Some(RowBytes::new());
        Ok(())
    }

    fn finish_row(&mut self) -> AcquisitionResult<()> {
        self.row
            .take()
            .map(|_| ())
            .ok_or(C2AcquisitionError::InvalidGeneration)
    }

    fn finish(self) -> AcquisitionResult<(RawAccounting, bool, bool)> {
        if !self.root_seen || !self.stack.is_empty() || self.row.is_some() {
            return Err(C2AcquisitionError::InvalidGeneration);
        }
        Ok((self.meter.finish(), self.invalid, self.lower_bound))
    }
}

struct BorrowedJsonUpperMeter {
    bytes: u64,
    secret: bool,
    scan_secrets: bool,
    exact: Option<BorrowedExactMeter>,
}

impl BorrowedJsonUpperMeter {
    fn new() -> Self {
        Self {
            bytes: 0,
            secret: false,
            scan_secrets: false,
            exact: None,
        }
    }

    fn with_exact_accounting() -> Self {
        Self {
            bytes: 0,
            secret: false,
            scan_secrets: true,
            exact: Some(BorrowedExactMeter::new()),
        }
    }

    fn begin_row(&mut self) -> AcquisitionResult<()> {
        if let Some(exact) = &mut self.exact {
            exact.begin_row()?;
        }
        Ok(())
    }

    fn finish_row(&mut self) -> AcquisitionResult<()> {
        if let Some(exact) = &mut self.exact {
            exact.finish_row()?;
        }
        Ok(())
    }

    fn finish_exact_accounting(mut self) -> AcquisitionResult<(RawAccounting, bool, bool)> {
        self.exact
            .take()
            .ok_or(C2AcquisitionError::InvalidGeneration)?
            .finish()
    }

    fn add(&mut self, amount: u64) -> AcquisitionResult<()> {
        self.bytes = self
            .bytes
            .checked_add(amount)
            .ok_or(C2AcquisitionError::LimitExceeded {
                table_overflow_mask: None,
            })?;
        if self.bytes > COMPLETE_GENERATION_UPPER_BOUND {
            return limit_error();
        }
        Ok(())
    }

    fn sequence_punctuation(&mut self, count: usize) -> AcquisitionResult<()> {
        let separators = count.saturating_sub(1);
        self.add(2)?;
        self.add(
            u64::try_from(separators).map_err(|_| C2AcquisitionError::LimitExceeded {
                table_overflow_mask: None,
            })?,
        )
    }

    fn object(&mut self, count: usize) -> AcquisitionResult<()> {
        if count > MAX_OBJECT_KEYS {
            return limit_error();
        }
        self.sequence_punctuation(count)?;
        if let Some(exact) = &mut self.exact {
            exact.object(count)?;
        }
        Ok(())
    }

    fn array(&mut self, count: usize, cap: usize) -> AcquisitionResult<()> {
        if count > cap {
            return limit_error();
        }
        self.sequence_punctuation(count)?;
        if let Some(exact) = &mut self.exact {
            exact.array(count)?;
        }
        Ok(())
    }

    fn upper_string(&mut self, value: &str, cap: usize) -> AcquisitionResult<()> {
        if value.len() > cap {
            return limit_error();
        }
        let length = u64::try_from(value.len()).map_err(|_| C2AcquisitionError::LimitExceeded {
            table_overflow_mask: None,
        })?;
        self.add(
            length
                .checked_mul(6)
                .and_then(|length| length.checked_add(2))
                .ok_or(C2AcquisitionError::LimitExceeded {
                    table_overflow_mask: None,
                })?,
        )?;
        if self.scan_secrets {
            self.secret |= likely_secret_in_string(value);
        }
        Ok(())
    }

    fn string(&mut self, value: &str, cap: usize) -> AcquisitionResult<()> {
        self.upper_string(value, cap)?;
        if let Some(exact) = &mut self.exact {
            exact.string(value.len())?;
        }
        Ok(())
    }

    fn key(&mut self, key: &str) -> AcquisitionResult<()> {
        self.upper_string(key, MAX_ORDINARY_SCALAR_BYTES)?;
        self.add(1)?;
        if let Some(exact) = &mut self.exact {
            exact.key(key.len())?;
        }
        Ok(())
    }

    fn member_string(&mut self, key: &str, value: &str, cap: usize) -> AcquisitionResult<()> {
        self.key(key)?;
        self.string(value, cap)?;
        if self.scan_secrets {
            self.secret |= is_credential_field(key) && is_secret_field_string(value);
        }
        Ok(())
    }

    fn member_fixed_string(&mut self, key: &str, length: usize) -> AcquisitionResult<()> {
        self.key(key)?;
        self.member_fixed_string_value(length, MAX_ORDINARY_SCALAR_BYTES)
    }

    fn member_fixed_string_value(&mut self, length: usize, cap: usize) -> AcquisitionResult<()> {
        if length > cap {
            return limit_error();
        }
        let length = u64::try_from(length).map_err(|_| C2AcquisitionError::LimitExceeded {
            table_overflow_mask: None,
        })?;
        self.add(
            length
                .checked_mul(6)
                .and_then(|length| length.checked_add(2))
                .ok_or(C2AcquisitionError::LimitExceeded {
                    table_overflow_mask: None,
                })?,
        )?;
        if let Some(exact) = &mut self.exact {
            exact.string(usize::try_from(length).map_err(|_| {
                C2AcquisitionError::LimitExceeded {
                    table_overflow_mask: None,
                }
            })?)?;
        }
        Ok(())
    }

    fn member_prefixed_string_value(
        &mut self,
        prefix: &str,
        payload_length: usize,
        cap: usize,
    ) -> AcquisitionResult<()> {
        let length =
            prefix
                .len()
                .checked_add(payload_length)
                .ok_or(C2AcquisitionError::LimitExceeded {
                    table_overflow_mask: None,
                })?;
        self.member_fixed_string_value(length, cap)
    }

    fn member_lowercase(&mut self, key: &str, value: &str) -> AcquisitionResult<()> {
        if value.len() > MAX_NAME_SCALAR_BYTES {
            return limit_error();
        }
        self.key(key)?;
        let mut length = 0_usize;
        for character in value.chars().flat_map(char::to_lowercase) {
            length = length.checked_add(character.len_utf8()).ok_or(
                C2AcquisitionError::LimitExceeded {
                    table_overflow_mask: None,
                },
            )?;
        }
        self.member_fixed_string_value(length, MAX_NAME_SCALAR_BYTES)?;
        if self.scan_secrets {
            self.secret |= likely_secret_in_string(value)
                || (is_credential_field(key) && is_secret_field_string(value));
        }
        Ok(())
    }

    fn member_optional_string(
        &mut self,
        key: &str,
        value: Option<&str>,
        cap: usize,
    ) -> AcquisitionResult<()> {
        self.key(key)?;
        match value {
            Some(value) => {
                self.string(value, cap)?;
                if self.scan_secrets {
                    self.secret |= is_credential_field(key) && is_secret_field_string(value);
                }
                Ok(())
            }
            None => self.null(),
        }
    }

    fn member_id(&mut self, key: &str) -> AcquisitionResult<()> {
        self.key(key)?;
        self.fixed_id()
    }

    fn member_optional_id(
        &mut self,
        key: &str,
        id: Option<&engram_core::id::Id>,
    ) -> AcquisitionResult<()> {
        self.key(key)?;
        match id {
            Some(_) => self.fixed_id(),
            None => self.null(),
        }
    }

    fn member_timestamp(&mut self, key: &str, value: OffsetDateTime) -> AcquisitionResult<()> {
        self.key(key)?;
        self.timestamp(value)
    }

    fn member_optional_timestamp(
        &mut self,
        key: &str,
        value: Option<OffsetDateTime>,
    ) -> AcquisitionResult<()> {
        self.key(key)?;
        match value {
            Some(value) => self.timestamp(value),
            None => self.null(),
        }
    }

    fn fixed_id(&mut self) -> AcquisitionResult<()> {
        self.add(2 + 6 * 36)?;
        if let Some(exact) = &mut self.exact {
            exact.string(36)?;
        }
        Ok(())
    }

    fn timestamp(&mut self, value: OffsetDateTime) -> AcquisitionResult<()> {
        // OffsetDateTime's pinned RFC3339 serializer cannot exceed 35 UTF-8 bytes.
        self.add(2 + 6 * 35)?;
        if let Some(exact) = &mut self.exact {
            exact.timestamp(value)?;
        }
        Ok(())
    }

    fn null(&mut self) -> AcquisitionResult<()> {
        self.add(4)?;
        if let Some(exact) = &mut self.exact {
            exact.scalar(1)?;
        }
        Ok(())
    }

    fn boolean(&mut self, value: bool) -> AcquisitionResult<()> {
        self.add(if value { 4 } else { 5 })?;
        if let Some(exact) = &mut self.exact {
            exact.scalar(1)?;
        }
        Ok(())
    }

    fn number_i32(&mut self, value: i32) -> AcquisitionResult<()> {
        self.add(32)?;
        if let Some(exact) = &mut self.exact {
            exact.number(&value.to_string())?;
        }
        Ok(())
    }

    fn number_u32(&mut self, value: u32) -> AcquisitionResult<()> {
        self.add(32)?;
        if let Some(exact) = &mut self.exact {
            exact.number(&value.to_string())?;
        }
        Ok(())
    }

    fn number_f32(&mut self, value: f32) -> AcquisitionResult<()> {
        self.add(32)?;
        if let Some(exact) = &mut self.exact {
            match JsonValue::from(value) {
                JsonValue::Number(number) => exact.number(&number.to_string())?,
                JsonValue::Null => {
                    exact.invalid = true;
                    exact.scalar(1)?;
                }
                _ => return Err(C2AcquisitionError::InvalidGeneration),
            }
        }
        Ok(())
    }

    fn number_f64(&mut self, value: f64) -> AcquisitionResult<()> {
        self.add(32)?;
        if let Some(exact) = &mut self.exact {
            let spelling = serde_json::to_string(&value).unwrap_or_else(|_| "null".to_string());
            if spelling == "null" {
                exact.scalar(1)?;
            } else {
                exact.number(&spelling)?;
            }
        }
        Ok(())
    }

    fn enum_unit(&mut self, value: &str) -> AcquisitionResult<()> {
        self.string(value, MAX_NAME_SCALAR_BYTES)
    }

    fn tagged_enum(&mut self, tag: &str, value: &str) -> AcquisitionResult<()> {
        self.object(1)?;
        self.member_string(tag, value, MAX_NAME_SCALAR_BYTES)
    }

    fn memory_kind(&mut self, value: &MemoryKind) -> AcquisitionResult<()> {
        match value {
            MemoryKind::Preference => self.enum_unit("preference"),
            MemoryKind::Rule => self.enum_unit("rule"),
            MemoryKind::Decision => self.enum_unit("decision"),
            MemoryKind::Limitation => self.enum_unit("limitation"),
            MemoryKind::ProjectFact => self.enum_unit("project_fact"),
            MemoryKind::RepositoryFact => self.enum_unit("repository_fact"),
            MemoryKind::TaskFact => self.enum_unit("task_fact"),
            MemoryKind::UserFact => self.enum_unit("user_fact"),
            MemoryKind::SessionInsight => self.enum_unit("session_insight"),
            MemoryKind::Handoff => self.enum_unit("handoff"),
            MemoryKind::Procedure => self.enum_unit("procedure"),
            MemoryKind::Custom(value) => self.tagged_enum("custom", value),
        }
    }

    fn memory_kind_display(&mut self, value: &MemoryKind) -> AcquisitionResult<()> {
        match value {
            MemoryKind::Preference => self.enum_unit("preference"),
            MemoryKind::Rule => self.enum_unit("rule"),
            MemoryKind::Decision => self.enum_unit("decision"),
            MemoryKind::Limitation => self.enum_unit("limitation"),
            MemoryKind::ProjectFact => self.enum_unit("project_fact"),
            MemoryKind::RepositoryFact => self.enum_unit("repository_fact"),
            MemoryKind::TaskFact => self.enum_unit("task_fact"),
            MemoryKind::UserFact => self.enum_unit("user_fact"),
            MemoryKind::SessionInsight => self.enum_unit("session_insight"),
            MemoryKind::Handoff => self.enum_unit("handoff"),
            MemoryKind::Procedure => self.enum_unit("procedure"),
            MemoryKind::Custom(value) => self.string(value, MAX_NAME_SCALAR_BYTES),
        }
    }

    fn memory_status(&mut self, value: &MemoryStatus) -> AcquisitionResult<()> {
        self.enum_unit(match value {
            MemoryStatus::Active => "active",
            MemoryStatus::NeedsReview => "needs_review",
            MemoryStatus::Superseded => "superseded",
            MemoryStatus::Archived => "archived",
            MemoryStatus::Rejected => "rejected",
        })
    }

    fn project_status(&mut self, value: &ProjectStatus) -> AcquisitionResult<()> {
        self.enum_unit(match value {
            ProjectStatus::Planning => "planning",
            ProjectStatus::Active => "active",
            ProjectStatus::Completed => "completed",
            ProjectStatus::Archived => "archived",
        })
    }

    fn task_status(&mut self, value: &TaskStatus) -> AcquisitionResult<()> {
        self.enum_unit(match value {
            TaskStatus::Todo => "todo",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Blocked => "blocked",
            TaskStatus::Done => "done",
        })
    }

    fn task_priority(&mut self, value: &TaskPriority) -> AcquisitionResult<()> {
        self.enum_unit(match value {
            TaskPriority::Low => "low",
            TaskPriority::Medium => "medium",
            TaskPriority::High => "high",
            TaskPriority::Critical => "critical",
        })
    }

    fn link_role(&mut self, value: &ProjectRepositoryRole) -> AcquisitionResult<()> {
        self.enum_unit(match value {
            ProjectRepositoryRole::Primary => "primary",
            ProjectRepositoryRole::Dependency => "dependency",
            ProjectRepositoryRole::Produces => "produces",
            ProjectRepositoryRole::Related => "related",
        })
    }

    fn claim_origin(&mut self, value: &ClaimOrigin) -> AcquisitionResult<()> {
        match value {
            ClaimOrigin::UserStated => self.enum_unit("user_stated"),
            ClaimOrigin::UserCorrected => self.enum_unit("user_corrected"),
            ClaimOrigin::AgentObserved => self.enum_unit("agent_observed"),
            ClaimOrigin::AgentInferred => self.enum_unit("agent_inferred"),
            ClaimOrigin::ToolResult => self.enum_unit("tool_result"),
            ClaimOrigin::Imported => self.enum_unit("imported"),
            ClaimOrigin::Migrated => self.enum_unit("migrated"),
            ClaimOrigin::GeneratedSummary => self.enum_unit("generated_summary"),
            ClaimOrigin::Custom(value) => self.tagged_enum("custom", value),
        }
    }

    fn harness(&mut self, value: &Harness) -> AcquisitionResult<()> {
        match value {
            Harness::ClaudeCode => self.enum_unit("claude_code"),
            Harness::Codex => self.enum_unit("codex"),
            Harness::ChatGpt => self.enum_unit("chat_gpt"),
            Harness::Cursor => self.enum_unit("cursor"),
            Harness::Other(value) => self.tagged_enum("other", value),
        }
    }

    fn harness_display(&mut self, value: &Harness) -> AcquisitionResult<()> {
        match value {
            Harness::ClaudeCode => self.enum_unit("claude_code"),
            Harness::Codex => self.enum_unit("codex"),
            Harness::ChatGpt => self.enum_unit("chatgpt"),
            Harness::Cursor => self.enum_unit("cursor"),
            Harness::Other(value) => self.string(value, MAX_NAME_SCALAR_BYTES),
        }
    }

    fn evidence_kind(&mut self, value: &EvidenceKind) -> AcquisitionResult<()> {
        match value {
            EvidenceKind::SessionEvent => self.enum_unit("session_event"),
            EvidenceKind::ToolCall => self.enum_unit("tool_call"),
            EvidenceKind::File => self.enum_unit("file"),
            EvidenceKind::GitCommit => self.enum_unit("git_commit"),
            EvidenceKind::Url => self.enum_unit("url"),
            EvidenceKind::Document => self.enum_unit("document"),
            EvidenceKind::Observation => self.enum_unit("observation"),
            EvidenceKind::ManualReview => self.enum_unit("manual_review"),
            EvidenceKind::Custom(value) => self.tagged_enum("custom", value),
        }
    }

    fn repository_provider(&mut self, value: &RepositoryProvider) -> AcquisitionResult<()> {
        match value {
            RepositoryProvider::GitHub => self.enum_unit("git_hub"),
            RepositoryProvider::GitLab => self.enum_unit("git_lab"),
            RepositoryProvider::Bitbucket => self.enum_unit("bitbucket"),
            RepositoryProvider::Unknown => self.enum_unit("unknown"),
            RepositoryProvider::Other(value) => self.tagged_enum("other", value),
        }
    }

    fn repository_provider_display(&mut self, value: &RepositoryProvider) -> AcquisitionResult<()> {
        match value {
            RepositoryProvider::GitHub => self.enum_unit("github"),
            RepositoryProvider::GitLab => self.enum_unit("gitlab"),
            RepositoryProvider::Bitbucket => self.enum_unit("bitbucket"),
            RepositoryProvider::Unknown => self.enum_unit("unknown"),
            RepositoryProvider::Other(value) => self.string(value, MAX_NAME_SCALAR_BYTES),
        }
    }

    fn scope_key_projection(&mut self, scope: &MemoryScope) -> AcquisitionResult<()> {
        match scope {
            MemoryScope::Global => self.enum_unit("global"),
            MemoryScope::User => self.enum_unit("user"),
            MemoryScope::Project {
                project_id,
                project_name,
            } => {
                if let Some(project_id) = project_id {
                    let _ = project_id.as_uuid();
                }
                self.member_prefixed_string_value(
                    "project:",
                    project_name.len(),
                    MAX_ORDINARY_SCALAR_BYTES,
                )
            }
            MemoryScope::Task {
                project_id,
                project_name,
                task_id,
                task_name,
            } => {
                if let Some(project_id) = project_id {
                    let _ = project_id.as_uuid();
                }
                let _ = project_name.as_deref();
                if let Some(task_id) = task_id {
                    let _ = task_id.as_uuid();
                }
                self.member_prefixed_string_value(
                    "task:",
                    task_name.len(),
                    MAX_ORDINARY_SCALAR_BYTES,
                )
            }
            MemoryScope::Entity {
                entity_id,
                entity_name,
            } => {
                if let Some(entity_id) = entity_id {
                    let _ = entity_id.as_uuid();
                }
                self.member_prefixed_string_value(
                    "entity:",
                    entity_name.len(),
                    MAX_ORDINARY_SCALAR_BYTES,
                )
            }
            MemoryScope::Repository {
                repository_id,
                remote_url,
                local_path,
            } => {
                if let Some(repository_id) = repository_id {
                    let _ = repository_id.as_uuid();
                }
                let value = remote_url
                    .as_deref()
                    .or(local_path.as_deref())
                    .unwrap_or_default();
                self.member_prefixed_string_value(
                    "repository:",
                    value.len(),
                    MAX_ORDINARY_SCALAR_BYTES,
                )
            }
            MemoryScope::Session { session_id } => {
                let _ = session_id.as_uuid();
                self.member_prefixed_string_value("session:", 36, MAX_ORDINARY_SCALAR_BYTES)
            }
            MemoryScope::Custom { name } => {
                self.member_prefixed_string_value("custom:", name.len(), MAX_ORDINARY_SCALAR_BYTES)
            }
        }
    }

    fn memory_scope(&mut self, scope: &MemoryScope) -> AcquisitionResult<()> {
        match scope {
            MemoryScope::Global => {
                self.object(1)?;
                self.member_string("type", "global", MAX_NAME_SCALAR_BYTES)
            }
            MemoryScope::User => {
                self.object(1)?;
                self.member_string("type", "user", MAX_NAME_SCALAR_BYTES)
            }
            MemoryScope::Project {
                project_id,
                project_name,
            } => {
                self.object(3)?;
                self.member_string("type", "project", MAX_NAME_SCALAR_BYTES)?;
                self.member_optional_id("project_id", project_id.as_ref())?;
                self.member_string("project_name", project_name, MAX_NAME_SCALAR_BYTES)
            }
            MemoryScope::Task {
                project_id,
                project_name,
                task_id,
                task_name,
            } => {
                self.object(5)?;
                self.member_string("type", "task", MAX_NAME_SCALAR_BYTES)?;
                self.member_optional_id("project_id", project_id.as_ref())?;
                self.member_optional_string(
                    "project_name",
                    project_name.as_deref(),
                    MAX_NAME_SCALAR_BYTES,
                )?;
                self.member_optional_id("task_id", task_id.as_ref())?;
                self.member_string("task_name", task_name, MAX_NAME_SCALAR_BYTES)
            }
            MemoryScope::Entity {
                entity_id,
                entity_name,
            } => {
                self.object(3)?;
                self.member_string("type", "entity", MAX_NAME_SCALAR_BYTES)?;
                self.member_optional_id("entity_id", entity_id.as_ref())?;
                self.member_string("entity_name", entity_name, MAX_NAME_SCALAR_BYTES)
            }
            MemoryScope::Repository {
                repository_id,
                remote_url,
                local_path,
            } => {
                self.object(4)?;
                self.member_string("type", "repository", MAX_NAME_SCALAR_BYTES)?;
                self.member_optional_id("repository_id", repository_id.as_ref())?;
                self.member_optional_string(
                    "remote_url",
                    remote_url.as_deref(),
                    MAX_ORDINARY_SCALAR_BYTES,
                )?;
                self.member_optional_string(
                    "local_path",
                    local_path.as_deref(),
                    MAX_ORDINARY_SCALAR_BYTES,
                )
            }
            MemoryScope::Session { session_id } => {
                let _ = session_id;
                self.object(2)?;
                self.member_string("type", "session", MAX_NAME_SCALAR_BYTES)?;
                self.member_id("session_id")
            }
            MemoryScope::Custom { name } => {
                self.object(2)?;
                self.member_string("type", "custom", MAX_NAME_SCALAR_BYTES)?;
                self.member_string("name", name, MAX_NAME_SCALAR_BYTES)
            }
        }
    }

    fn model(&mut self, model: &ModelIdentity) -> AcquisitionResult<()> {
        let ModelIdentity {
            provider,
            model,
            version,
        } = model;
        self.object(3)?;
        self.member_string("provider", provider, MAX_NAME_SCALAR_BYTES)?;
        self.member_string("model", model, MAX_NAME_SCALAR_BYTES)?;
        self.member_optional_string("version", version.as_deref(), MAX_NAME_SCALAR_BYTES)
    }

    fn writer(&mut self, writer: &WriterProvenance) -> AcquisitionResult<()> {
        let WriterProvenance {
            harness,
            harness_version,
            model,
            surface,
            actor,
            session_id,
            written_at,
        } = writer;
        self.object(7)?;
        self.key("harness")?;
        self.harness(harness)?;
        self.member_optional_string(
            "harness_version",
            harness_version.as_deref(),
            MAX_NAME_SCALAR_BYTES,
        )?;
        self.key("model")?;
        self.model(model)?;
        self.member_optional_string("surface", surface.as_deref(), MAX_NAME_SCALAR_BYTES)?;
        self.member_string("actor", actor, MAX_NAME_SCALAR_BYTES)?;
        self.member_optional_id("session_id", session_id.as_ref())?;
        self.member_timestamp("written_at", *written_at)
    }

    fn evidence(&mut self, evidence: &EvidenceRef) -> AcquisitionResult<()> {
        let EvidenceRef {
            kind,
            target,
            summary,
            excerpt,
            observed_at,
        } = evidence;
        self.object(5)?;
        self.key("kind")?;
        self.evidence_kind(kind)?;
        self.member_string("target", target, MAX_ORDINARY_SCALAR_BYTES)?;
        self.member_optional_string("summary", summary.as_deref(), MAX_ORDINARY_SCALAR_BYTES)?;
        self.member_optional_string("excerpt", excerpt.as_deref(), MAX_LARGE_SCALAR_BYTES)?;
        self.member_timestamp("observed_at", *observed_at)
    }

    fn archive(&mut self, archive: &ArchiveMetadata) -> AcquisitionResult<()> {
        let ArchiveMetadata {
            reason,
            archived_by,
            archived_at,
        } = archive;
        self.object(3)?;
        self.member_string("reason", reason, MAX_ORDINARY_SCALAR_BYTES)?;
        self.member_optional_string("archived_by", archived_by.as_deref(), MAX_NAME_SCALAR_BYTES)?;
        self.member_timestamp("archived_at", *archived_at)
    }

    fn prerequisite_source(
        &mut self,
        source: &ProcedurePrerequisiteSource,
    ) -> AcquisitionResult<()> {
        match source {
            ProcedurePrerequisiteSource::Toml {
                relative_path,
                key_path,
            } => {
                self.object(3)?;
                self.member_string("format", "toml", MAX_NAME_SCALAR_BYTES)?;
                self.member_string("relative_path", relative_path, MAX_ORDINARY_SCALAR_BYTES)?;
                self.key("key_path")?;
                self.array(key_path.len(), MAX_PREREQUISITE_KEY_PATH)?;
                for key in key_path {
                    self.string(key, MAX_NAME_SCALAR_BYTES)?;
                }
                Ok(())
            }
        }
    }

    fn prerequisite(&mut self, prerequisite: &ProcedurePrerequisite) -> AcquisitionResult<()> {
        let ProcedurePrerequisite {
            key,
            expected,
            source,
        } = prerequisite;
        self.object(2 + usize::from(source.is_some()))?;
        self.member_string("key", key, MAX_NAME_SCALAR_BYTES)?;
        self.member_string("expected", expected, MAX_ORDINARY_SCALAR_BYTES)?;
        if let Some(source) = source {
            self.key("source")?;
            self.prerequisite_source(source)?;
        }
        Ok(())
    }

    fn verification(&mut self, verification: &ProcedureVerification) -> AcquisitionResult<()> {
        let ProcedureVerification {
            command,
            expected_exit_code,
            expected_output_contains,
            evidence_path,
            evidence_sha256,
            verified_at,
        } = verification;
        self.object(6)?;
        self.member_string("command", command, MAX_ORDINARY_SCALAR_BYTES)?;
        self.key("expected_exit_code")?;
        self.number_i32(*expected_exit_code)?;
        self.member_string(
            "expected_output_contains",
            expected_output_contains,
            MAX_ORDINARY_SCALAR_BYTES,
        )?;
        self.member_optional_string(
            "evidence_path",
            evidence_path.as_deref(),
            MAX_ORDINARY_SCALAR_BYTES,
        )?;
        self.member_optional_string(
            "evidence_sha256",
            evidence_sha256.as_deref(),
            MAX_ORDINARY_SCALAR_BYTES,
        )?;
        self.member_optional_timestamp("verified_at", *verified_at)
    }

    fn procedure(&mut self, procedure: &ProcedureCard) -> AcquisitionResult<()> {
        let ProcedureCard {
            task,
            prerequisites,
            commands,
            failure_signatures,
            verification,
            expires_at,
        } = procedure;
        self.object(6)?;
        self.member_string("task", task, MAX_ORDINARY_SCALAR_BYTES)?;
        self.key("prerequisites")?;
        self.array(prerequisites.len(), MAX_PROCEDURE_VECTOR)?;
        for prerequisite in prerequisites {
            self.prerequisite(prerequisite)?;
        }
        self.key("commands")?;
        self.array(commands.len(), MAX_PROCEDURE_VECTOR)?;
        for command in commands {
            self.string(command, MAX_LARGE_SCALAR_BYTES)?;
        }
        self.key("failure_signatures")?;
        self.array(failure_signatures.len(), MAX_PROCEDURE_VECTOR)?;
        for signature in failure_signatures {
            self.string(signature, MAX_ORDINARY_SCALAR_BYTES)?;
        }
        self.key("verification")?;
        self.verification(verification)?;
        self.member_optional_timestamp("expires_at", *expires_at)
    }

    fn memory_item(&mut self, item: &MemoryItem) -> AcquisitionResult<()> {
        let MemoryItem {
            id,
            kind,
            title,
            content,
            scope,
            origin,
            writer,
            evidence,
            confidence,
            status,
            supersedes,
            tags,
            created_at,
            updated_at,
            last_used_at,
            review_after,
            archive,
            procedure,
            correction_proposal_id,
            pending_correction_proposal_id,
        } = item;
        let _ = id.as_uuid();
        let marker_count = usize::from(correction_proposal_id.is_some())
            + usize::from(pending_correction_proposal_id.is_some());
        self.object(18 + marker_count)?;
        self.member_id("id")?;
        self.key("kind")?;
        self.memory_kind(kind)?;
        self.member_string("title", title, MAX_NAME_SCALAR_BYTES)?;
        self.member_string("content", content, MAX_LARGE_SCALAR_BYTES)?;
        self.key("scope")?;
        self.memory_scope(scope)?;
        self.key("origin")?;
        self.claim_origin(origin)?;
        self.key("writer")?;
        self.writer(writer)?;
        self.key("evidence")?;
        self.array(evidence.len(), MAX_EVIDENCE_TAGS_SUPERSEDES)?;
        for evidence in evidence {
            self.evidence(evidence)?;
        }
        self.key("confidence")?;
        self.number_f32(confidence.value())?;
        self.key("status")?;
        self.memory_status(status)?;
        self.key("supersedes")?;
        self.array(supersedes.len(), MAX_EVIDENCE_TAGS_SUPERSEDES)?;
        for id in supersedes {
            let _ = id.as_uuid();
            self.fixed_id()?;
        }
        self.key("tags")?;
        self.array(tags.len(), MAX_EVIDENCE_TAGS_SUPERSEDES)?;
        for tag in tags {
            self.string(tag, MAX_NAME_SCALAR_BYTES)?;
        }
        self.member_timestamp("created_at", *created_at)?;
        self.member_timestamp("updated_at", *updated_at)?;
        self.member_optional_timestamp("last_used_at", *last_used_at)?;
        self.member_optional_timestamp("review_after", *review_after)?;
        self.key("archive")?;
        match archive {
            Some(archive) => self.archive(archive)?,
            None => self.null()?,
        }
        self.key("procedure")?;
        match procedure {
            Some(procedure) => self.procedure(procedure)?,
            None => self.null()?,
        }
        if let Some(id) = correction_proposal_id {
            let _ = id.as_uuid();
            self.member_id("correction_proposal_id")?;
        }
        if let Some(id) = pending_correction_proposal_id {
            let _ = id.as_uuid();
            self.member_id("pending_correction_proposal_id")?;
        }
        Ok(())
    }

    fn proposal(&mut self, proposal: &CorrectionProposal) -> AcquisitionResult<()> {
        let CorrectionProposal {
            id,
            obsolete_id,
            replacement_id,
            memory_kind,
            scope,
            canonical_digest,
            digest_schema_version,
            applied_digest,
            status,
            proposer,
            created_at,
            applied_at,
        } = proposal;
        let _ = (
            id.as_uuid(),
            obsolete_id.as_uuid(),
            replacement_id.as_uuid(),
        );
        let applied_digest_count = usize::from(applied_digest.is_some());
        self.object(11 + applied_digest_count)?;
        self.member_id("id")?;
        self.member_id("obsolete_id")?;
        self.member_id("replacement_id")?;
        self.key("memory_kind")?;
        self.memory_kind(memory_kind)?;
        self.key("scope")?;
        self.memory_scope(scope)?;
        self.member_string(
            "canonical_digest",
            canonical_digest,
            MAX_ORDINARY_SCALAR_BYTES,
        )?;
        self.key("digest_schema_version")?;
        self.number_u32(*digest_schema_version)?;
        if let Some(digest) = applied_digest {
            self.member_string("applied_digest", digest, MAX_ORDINARY_SCALAR_BYTES)?;
        }
        self.key("status")?;
        self.enum_unit(match status {
            CorrectionProposalStatus::Pending => "pending",
            CorrectionProposalStatus::Applied => "applied",
        })?;
        self.key("proposer")?;
        self.writer(proposer)?;
        self.member_timestamp("created_at", *created_at)?;
        self.member_optional_timestamp("applied_at", *applied_at)
    }

    fn project(&mut self, project: &Project) -> AcquisitionResult<()> {
        let Project {
            id,
            name,
            description,
            status,
            created_at,
            updated_at,
        } = project;
        let _ = id.as_uuid();
        self.object(6)?;
        self.member_id("record_id")?;
        self.member_string("name", name, MAX_NAME_SCALAR_BYTES)?;
        self.member_optional_string(
            "description",
            description.as_deref(),
            MAX_ORDINARY_SCALAR_BYTES,
        )?;
        self.key("status")?;
        self.project_status(status)?;
        self.member_timestamp("created_at", *created_at)?;
        self.member_timestamp("updated_at", *updated_at)
    }

    fn task(&mut self, task: &Task) -> AcquisitionResult<()> {
        let Task {
            id,
            project_id,
            name,
            description,
            status,
            priority,
            jira_key,
            blocked_by,
            created_at,
            updated_at,
        } = task;
        let _ = (id.as_uuid(), project_id.as_uuid());
        self.object(10)?;
        self.member_id("record_id")?;
        self.member_id("project_id")?;
        self.member_string("name", name, MAX_NAME_SCALAR_BYTES)?;
        self.member_optional_string(
            "description",
            description.as_deref(),
            MAX_ORDINARY_SCALAR_BYTES,
        )?;
        self.key("status")?;
        self.task_status(status)?;
        self.key("priority")?;
        self.task_priority(priority)?;
        self.member_optional_string("jira_key", jira_key.as_deref(), MAX_NAME_SCALAR_BYTES)?;
        self.key("blocked_by")?;
        self.array(blocked_by.len(), MAX_VECTOR_ELEMENTS)?;
        for id in blocked_by {
            let _ = id.as_uuid();
            self.fixed_id()?;
        }
        self.member_timestamp("created_at", *created_at)?;
        self.member_timestamp("updated_at", *updated_at)
    }

    fn repository(&mut self, repository: &GitRepository) -> AcquisitionResult<()> {
        let GitRepository {
            id,
            name,
            remote_url,
            provider,
            default_branch,
            description,
            created_at,
            updated_at,
        } = repository;
        let _ = id.as_uuid();
        self.object(8)?;
        self.member_id("id")?;
        self.member_string("name", name, MAX_NAME_SCALAR_BYTES)?;
        self.member_optional_string(
            "remote_url",
            remote_url.as_deref(),
            MAX_ORDINARY_SCALAR_BYTES,
        )?;
        self.key("provider")?;
        self.repository_provider(provider)?;
        self.member_optional_string(
            "default_branch",
            default_branch.as_deref(),
            MAX_NAME_SCALAR_BYTES,
        )?;
        self.member_optional_string(
            "description",
            description.as_deref(),
            MAX_ORDINARY_SCALAR_BYTES,
        )?;
        self.member_timestamp("created_at", *created_at)?;
        self.member_timestamp("updated_at", *updated_at)
    }

    fn checkout(&mut self, checkout: &LocalCheckout) -> AcquisitionResult<()> {
        let LocalCheckout {
            id,
            repository_id,
            local_path,
            current_branch,
            head_sha,
            is_dirty,
            created_at,
            updated_at,
            last_seen_at,
        } = checkout;
        let _ = id.as_uuid();
        if let Some(repository_id) = repository_id {
            let _ = repository_id.as_uuid();
        }
        self.object(9)?;
        self.member_id("id")?;
        self.member_optional_id("repository_id", repository_id.as_ref())?;
        self.member_string("local_path", local_path, MAX_ORDINARY_SCALAR_BYTES)?;
        self.member_optional_string(
            "current_branch",
            current_branch.as_deref(),
            MAX_NAME_SCALAR_BYTES,
        )?;
        self.member_optional_string("head_sha", head_sha.as_deref(), MAX_ORDINARY_SCALAR_BYTES)?;
        self.key("is_dirty")?;
        match is_dirty {
            Some(value) => self.boolean(*value)?,
            None => self.null()?,
        }
        self.member_timestamp("created_at", *created_at)?;
        self.member_timestamp("updated_at", *updated_at)?;
        self.member_timestamp("last_seen_at", *last_seen_at)
    }

    fn component(&mut self, component: &MonorepoComponent) -> AcquisitionResult<()> {
        let MonorepoComponent {
            id,
            repository_id,
            name,
            path,
            kind,
            description,
            source_path,
            source_sha256,
            created_at,
            updated_at,
        } = component;
        let _ = (id.as_uuid(), repository_id.as_uuid());
        let source_count =
            usize::from(source_path.is_some()) + usize::from(source_sha256.is_some());
        self.object(8 + source_count)?;
        self.member_id("id")?;
        self.member_id("repository_id")?;
        self.member_string("name", name, MAX_NAME_SCALAR_BYTES)?;
        self.member_string("path", path, MAX_ORDINARY_SCALAR_BYTES)?;
        self.member_optional_string("kind", kind.as_deref(), MAX_NAME_SCALAR_BYTES)?;
        self.member_optional_string(
            "description",
            description.as_deref(),
            MAX_ORDINARY_SCALAR_BYTES,
        )?;
        if let Some(path) = source_path {
            self.member_string("source_path", path, MAX_ORDINARY_SCALAR_BYTES)?;
        }
        if let Some(digest) = source_sha256 {
            self.member_string("source_sha256", digest, MAX_ORDINARY_SCALAR_BYTES)?;
        }
        self.member_timestamp("created_at", *created_at)?;
        self.member_timestamp("updated_at", *updated_at)
    }

    fn link(&mut self, link: &ProjectRepositoryLink) -> AcquisitionResult<()> {
        let ProjectRepositoryLink {
            id,
            project_id,
            project_name,
            repository_id,
            component_id,
            component_path,
            role,
            created_at,
            updated_at,
        } = link;
        let _ = (id.as_uuid(), repository_id.as_uuid());
        if let Some(project_id) = project_id {
            let _ = project_id.as_uuid();
        }
        if let Some(component_id) = component_id {
            let _ = component_id.as_uuid();
        }
        self.object(9)?;
        self.member_id("id")?;
        self.member_optional_id("project_id", project_id.as_ref())?;
        self.member_string("project_name", project_name, MAX_NAME_SCALAR_BYTES)?;
        self.member_id("repository_id")?;
        self.member_optional_id("component_id", component_id.as_ref())?;
        self.member_optional_string(
            "component_path",
            component_path.as_deref(),
            MAX_ORDINARY_SCALAR_BYTES,
        )?;
        self.key("role")?;
        self.link_role(role)?;
        self.member_timestamp("created_at", *created_at)?;
        self.member_timestamp("updated_at", *updated_at)
    }

    fn target(&mut self, target: &C2RawTarget) -> AcquisitionResult<()> {
        self.object(8)?;
        self.member_string("project_id", &target.project_id, MAX_ORDINARY_SCALAR_BYTES)?;
        self.member_string("project_name", &target.project_name, MAX_NAME_SCALAR_BYTES)?;
        self.member_string(
            "repository_id",
            &target.repository_id,
            MAX_ORDINARY_SCALAR_BYTES,
        )?;
        self.member_string(
            "repository_remote",
            &target.repository_remote,
            MAX_ORDINARY_SCALAR_BYTES,
        )?;
        self.member_string(
            "checkout_id",
            &target.checkout_id,
            MAX_ORDINARY_SCALAR_BYTES,
        )?;
        self.member_string(
            "checkout_path",
            &target.checkout_path,
            MAX_ORDINARY_SCALAR_BYTES,
        )?;
        self.member_optional_string(
            "task_id",
            target.task_id.as_deref(),
            MAX_ORDINARY_SCALAR_BYTES,
        )?;
        self.member_optional_string(
            "task_name",
            target.task_name.as_deref(),
            MAX_NAME_SCALAR_BYTES,
        )
    }

    fn memory_row(&mut self, item: &MemoryItem) -> AcquisitionResult<()> {
        self.object(11)?;
        self.member_id("record_id")?;
        self.key("item")?;
        self.memory_item(item)?;
        self.key("kind_key")?;
        self.memory_kind_display(&item.kind)?;
        self.key("status_key")?;
        self.memory_status(&item.status)?;
        self.key("scope_key")?;
        self.scope_key_projection(&item.scope)?;
        self.key("harness_key")?;
        self.harness_display(&item.writer.harness)?;
        self.member_string("model_key", &item.writer.model.model, MAX_NAME_SCALAR_BYTES)?;
        self.member_optional_id("session_id", item.writer.session_id.as_ref())?;
        self.member_fixed_string("snapshot_digest", 64)?;
        self.member_timestamp("created_at", item.created_at)?;
        self.member_timestamp("updated_at", item.updated_at)
    }

    fn proposal_row(&mut self, proposal: &CorrectionProposal) -> AcquisitionResult<()> {
        let pending = proposal.status == CorrectionProposalStatus::Pending;
        self.object(7 + usize::from(pending))?;
        self.member_id("record_id")?;
        self.key("proposal")?;
        self.proposal(proposal)?;
        self.key("status_key")?;
        self.enum_unit(match proposal.status {
            CorrectionProposalStatus::Pending => "pending",
            CorrectionProposalStatus::Applied => "applied",
        })?;
        self.member_id("obsolete_id")?;
        if pending {
            self.member_id("pending_obsolete_id")?;
        }
        self.member_id("replacement_id")?;
        self.member_fixed_string("snapshot_digest", 64)?;
        self.member_timestamp("created_at", proposal.created_at)
    }

    fn repository_row(&mut self, repository: &GitRepository) -> AcquisitionResult<()> {
        self.object(7)?;
        self.member_id("record_id")?;
        self.key("repository")?;
        self.repository(repository)?;
        self.member_lowercase("name_key", &repository.name)?;
        self.member_optional_string(
            "remote_url",
            repository.remote_url.as_deref(),
            MAX_ORDINARY_SCALAR_BYTES,
        )?;
        self.key("provider_key")?;
        self.repository_provider_display(&repository.provider)?;
        self.member_timestamp("created_at", repository.created_at)?;
        self.member_timestamp("updated_at", repository.updated_at)
    }

    fn checkout_row(&mut self, checkout: &LocalCheckout) -> AcquisitionResult<()> {
        self.object(10)?;
        self.member_id("record_id")?;
        self.key("checkout")?;
        self.checkout(checkout)?;
        self.member_optional_id("repository_id", checkout.repository_id.as_ref())?;
        self.member_string(
            "local_path_key",
            &checkout.local_path,
            MAX_ORDINARY_SCALAR_BYTES,
        )?;
        self.member_optional_string(
            "current_branch",
            checkout.current_branch.as_deref(),
            MAX_NAME_SCALAR_BYTES,
        )?;
        self.member_optional_string(
            "head_sha",
            checkout.head_sha.as_deref(),
            MAX_ORDINARY_SCALAR_BYTES,
        )?;
        self.key("is_dirty")?;
        match checkout.is_dirty {
            Some(value) => self.boolean(value)?,
            None => self.null()?,
        }
        self.member_timestamp("created_at", checkout.created_at)?;
        self.member_timestamp("updated_at", checkout.updated_at)?;
        self.member_timestamp("last_seen_at", checkout.last_seen_at)
    }

    fn component_row(&mut self, component: &MonorepoComponent) -> AcquisitionResult<()> {
        self.object(8)?;
        self.member_id("record_id")?;
        self.key("component")?;
        self.component(component)?;
        self.member_id("repository_id")?;
        self.member_lowercase("name_key", &component.name)?;
        self.member_string("path_key", &component.path, MAX_ORDINARY_SCALAR_BYTES)?;
        self.member_optional_string("kind", component.kind.as_deref(), MAX_NAME_SCALAR_BYTES)?;
        self.member_timestamp("created_at", component.created_at)?;
        self.member_timestamp("updated_at", component.updated_at)
    }

    fn link_row(&mut self, link: &ProjectRepositoryLink) -> AcquisitionResult<()> {
        self.object(10)?;
        self.member_id("record_id")?;
        self.key("link")?;
        self.link(link)?;
        self.member_optional_id("project_id", link.project_id.as_ref())?;
        self.member_lowercase("project_name_key", &link.project_name)?;
        self.member_id("repository_id")?;
        self.member_optional_id("component_id", link.component_id.as_ref())?;
        self.member_optional_string(
            "component_path_key",
            link.component_path.as_deref(),
            MAX_ORDINARY_SCALAR_BYTES,
        )?;
        self.key("role")?;
        self.link_role(&link.role)?;
        self.member_timestamp("created_at", link.created_at)?;
        self.member_timestamp("updated_at", link.updated_at)
    }
}

fn generation_table_overflow_mask(generation: &C2CompleteGeneration) -> u16 {
    let lengths = [
        generation.memory_item.len(),
        generation.correction_proposal.len(),
        0,
        generation.work_project.len(),
        generation.work_task.len(),
        generation.git_repository.len(),
        generation.local_checkout.len(),
        generation.monorepo_component.len(),
        generation.project_repository_link.len(),
    ];
    TABLE_KINDS
        .into_iter()
        .zip(lengths)
        .fold(0_u16, |mask, (table, length)| {
            if length > table.row_cap() {
                mask | table.overflow_bit()
            } else {
                mask
            }
        })
}

fn validate_unique_raw_ids(raw: &C2RawSnapshot) -> AcquisitionResult<()> {
    let mut ids = BTreeSet::new();
    for table in TABLE_KINDS {
        ids.clear();
        for row in super::table_rows(raw, table) {
            let record_id = row
                .as_object()
                .and_then(|row| row.get("record_id"))
                .and_then(JsonValue::as_str)
                .ok_or(C2AcquisitionError::InvalidGeneration)?;
            if !ids.insert(record_id.to_owned()) {
                return Err(C2AcquisitionError::InvalidGeneration);
            }
        }
    }
    Ok(())
}

enum BorrowedLimitAccounting {
    Exact(RawAccounting),
    InvalidExact,
    InvalidLowerBound,
}

struct BorrowedGenerationPreflight {
    upper_bytes: u64,
    secret: bool,
    accounting: BorrowedLimitAccounting,
}

fn derived_projection_contains_secret(generation: &C2CompleteGeneration) -> bool {
    generation.memory_item.iter().any(|item| {
        let projection = scope_key(&item.scope);
        likely_secret_in_string(&projection)
    }) || generation.git_repository.iter().any(|repository| {
        let projection = repository.name.to_lowercase();
        likely_secret_in_string(&projection)
    }) || generation.monorepo_component.iter().any(|component| {
        let projection = component.name.to_lowercase();
        likely_secret_in_string(&projection)
    }) || generation.project_repository_link.iter().any(|link| {
        let projection = link.project_name.to_lowercase();
        likely_secret_in_string(&projection)
    })
}

fn visit_borrowed_generation(
    meter: &mut BorrowedJsonUpperMeter,
    generation: &C2CompleteGeneration,
) -> AcquisitionResult<()> {
    meter.object(11)?;
    meter.key("target")?;
    meter.target(&generation.target)?;
    meter.key("prior_correction_bindings")?;
    meter.array(0, 0)?;

    meter.key("memory_item")?;
    meter.array(
        generation.memory_item.len(),
        TableKind::MemoryItem.row_cap(),
    )?;
    for item in &generation.memory_item {
        meter.begin_row()?;
        meter.memory_row(item)?;
        meter.finish_row()?;
    }
    meter.key("correction_proposal")?;
    meter.array(
        generation.correction_proposal.len(),
        TableKind::CorrectionProposal.row_cap(),
    )?;
    for proposal in &generation.correction_proposal {
        meter.begin_row()?;
        meter.proposal_row(proposal)?;
        meter.finish_row()?;
    }
    meter.key("memory_forget_receipt")?;
    meter.array(0, 0)?;
    meter.key("work_project")?;
    meter.array(
        generation.work_project.len(),
        TableKind::WorkProject.row_cap(),
    )?;
    for project in &generation.work_project {
        meter.begin_row()?;
        meter.project(project)?;
        meter.finish_row()?;
    }
    meter.key("work_task")?;
    meter.array(generation.work_task.len(), TableKind::WorkTask.row_cap())?;
    for task in &generation.work_task {
        meter.begin_row()?;
        meter.task(task)?;
        meter.finish_row()?;
    }
    meter.key("git_repository")?;
    meter.array(
        generation.git_repository.len(),
        TableKind::GitRepository.row_cap(),
    )?;
    for repository in &generation.git_repository {
        meter.begin_row()?;
        meter.repository_row(repository)?;
        meter.finish_row()?;
    }
    meter.key("local_checkout")?;
    meter.array(
        generation.local_checkout.len(),
        TableKind::LocalCheckout.row_cap(),
    )?;
    for checkout in &generation.local_checkout {
        meter.begin_row()?;
        meter.checkout_row(checkout)?;
        meter.finish_row()?;
    }
    meter.key("monorepo_component")?;
    meter.array(
        generation.monorepo_component.len(),
        TableKind::MonorepoComponent.row_cap(),
    )?;
    for component in &generation.monorepo_component {
        meter.begin_row()?;
        meter.component_row(component)?;
        meter.finish_row()?;
    }
    meter.key("project_repository_link")?;
    meter.array(
        generation.project_repository_link.len(),
        TableKind::ProjectRepositoryLink.row_cap(),
    )?;
    for link in &generation.project_repository_link {
        meter.begin_row()?;
        meter.link_row(link)?;
        meter.finish_row()?;
    }

    Ok(())
}

fn borrowed_generation_upper_bound(
    generation: &C2CompleteGeneration,
) -> AcquisitionResult<BorrowedGenerationPreflight> {
    let mask = generation_table_overflow_mask(generation);
    if mask != 0 {
        return Err(C2AcquisitionError::LimitExceeded {
            table_overflow_mask: Some(mask),
        });
    }

    // This first whole-generation pass is allocation-free and rejects every structural bound
    // before the bounded exact-accounting and projection scans allocate derived scalar spellings.
    let mut upper = BorrowedJsonUpperMeter::new();
    visit_borrowed_generation(&mut upper, generation)?;
    let upper_bytes = upper.bytes;

    let mut exact = BorrowedJsonUpperMeter::with_exact_accounting();
    visit_borrowed_generation(&mut exact, generation)?;
    let source_secret = exact.secret;
    let (accounting, invalid, lower_bound) = exact.finish_exact_accounting()?;

    Ok(BorrowedGenerationPreflight {
        upper_bytes,
        // The bounded exact/lower-bound pass collects source-string secret evidence, but that
        // evidence is consulted only after the complete limit pass succeeds. Derived strings are
        // allocated and scanned only now, after both structural and exact/lower-bound limits.
        // These are the exact four caller-dependent persisted transforms.
        secret: source_secret || derived_projection_contains_secret(generation),
        accounting: if lower_bound {
            BorrowedLimitAccounting::InvalidLowerBound
        } else if invalid {
            BorrowedLimitAccounting::InvalidExact
        } else {
            BorrowedLimitAccounting::Exact(accounting)
        },
    })
}

fn validate_typed_logical_uniqueness(generation: &C2CompleteGeneration) -> AcquisitionResult<()> {
    let mut family_ids = BTreeSet::new();
    for item in &generation.memory_item {
        if !family_ids.insert(*item.id.as_uuid().as_bytes()) {
            return Err(C2AcquisitionError::InvalidGeneration);
        }
    }
    family_ids.clear();
    for proposal in &generation.correction_proposal {
        if !family_ids.insert(*proposal.id.as_uuid().as_bytes()) {
            return Err(C2AcquisitionError::InvalidGeneration);
        }
    }
    family_ids.clear();
    for project in &generation.work_project {
        if !family_ids.insert(*project.id.as_uuid().as_bytes()) {
            return Err(C2AcquisitionError::InvalidGeneration);
        }
    }
    family_ids.clear();
    for task in &generation.work_task {
        if !family_ids.insert(*task.id.as_uuid().as_bytes()) {
            return Err(C2AcquisitionError::InvalidGeneration);
        }
    }
    family_ids.clear();
    for repository in &generation.git_repository {
        if !family_ids.insert(*repository.id.as_uuid().as_bytes()) {
            return Err(C2AcquisitionError::InvalidGeneration);
        }
    }
    family_ids.clear();
    for checkout in &generation.local_checkout {
        if !family_ids.insert(*checkout.id.as_uuid().as_bytes()) {
            return Err(C2AcquisitionError::InvalidGeneration);
        }
    }
    family_ids.clear();
    for component in &generation.monorepo_component {
        if !family_ids.insert(*component.id.as_uuid().as_bytes()) {
            return Err(C2AcquisitionError::InvalidGeneration);
        }
    }
    family_ids.clear();
    for link in &generation.project_repository_link {
        if !family_ids.insert(*link.id.as_uuid().as_bytes()) {
            return Err(C2AcquisitionError::InvalidGeneration);
        }
    }

    let mut project_names = BTreeSet::new();
    for project in &generation.work_project {
        if !project_names.insert(project.name.to_ascii_lowercase()) {
            return Err(C2AcquisitionError::InvalidGeneration);
        }
    }

    let mut exact_task_selectors = BTreeMap::<([u8; 16], String), [u8; 16]>::new();
    let mut folded_task_selectors = BTreeMap::<([u8; 16], String), [u8; 16]>::new();
    for task in &generation.work_task {
        let project_id = *task.project_id.as_uuid().as_bytes();
        let task_id = *task.id.as_uuid().as_bytes();
        let mut selectors = vec![task.name.as_str()];
        if let Some(jira_key) = task.jira_key.as_deref() {
            selectors.push(jira_key);
        }
        for selector in selectors {
            if exact_task_selectors
                .insert((project_id, selector.to_string()), task_id)
                .is_some_and(|existing| existing != task_id)
            {
                return Err(C2AcquisitionError::InvalidGeneration);
            }
            if folded_task_selectors
                .insert((project_id, selector.to_ascii_lowercase()), task_id)
                .is_some_and(|existing| existing != task_id)
            {
                return Err(C2AcquisitionError::InvalidGeneration);
            }
        }
    }

    let mut normalized_remotes = BTreeSet::new();
    for repository in &generation.git_repository {
        if let Some(remote) = repository.remote_url.as_deref() {
            let normalized =
                normalize_remote(remote).map_err(|_| C2AcquisitionError::InvalidGeneration)?;
            if !normalized_remotes.insert(normalized) {
                return Err(C2AcquisitionError::InvalidGeneration);
            }
        }
    }

    let mut checkout_paths = BTreeSet::new();
    for checkout in &generation.local_checkout {
        if !checkout_paths.insert(checkout.local_path.as_str()) {
            return Err(C2AcquisitionError::InvalidGeneration);
        }
    }

    let mut component_paths = BTreeSet::new();
    for component in &generation.monorepo_component {
        if !component_paths.insert((
            *component.repository_id.as_uuid().as_bytes(),
            component.path.as_str(),
        )) {
            return Err(C2AcquisitionError::InvalidGeneration);
        }
    }

    let mut link_identities = BTreeSet::new();
    for link in &generation.project_repository_link {
        let project = generation
            .work_project
            .iter()
            .find(|project| project.name == link.project_name)
            .ok_or(C2AcquisitionError::InvalidGeneration)?;
        if !link_identities.insert((
            *project.id.as_uuid().as_bytes(),
            *link.repository_id.as_uuid().as_bytes(),
            link.component_id
                .as_ref()
                .map(|component_id| *component_id.as_uuid().as_bytes()),
        )) {
            return Err(C2AcquisitionError::InvalidGeneration);
        }
    }

    let mut proposal_pairs = BTreeSet::new();
    let mut obsolete_ids = BTreeSet::new();
    let mut replacement_ids = BTreeSet::new();
    for proposal in &generation.correction_proposal {
        let obsolete_id = *proposal.obsolete_id.as_uuid().as_bytes();
        let replacement_id = *proposal.replacement_id.as_uuid().as_bytes();
        if !proposal_pairs.insert((obsolete_id, replacement_id))
            || !obsolete_ids.insert(obsolete_id)
            || !replacement_ids.insert(replacement_id)
        {
            return Err(C2AcquisitionError::InvalidGeneration);
        }
    }
    if obsolete_ids
        .iter()
        .any(|obsolete_id| replacement_ids.contains(obsolete_id))
    {
        return Err(C2AcquisitionError::InvalidGeneration);
    }
    Ok(())
}

fn json_generation(generation: C2CompleteGeneration) -> AcquisitionResult<C2RawSnapshot> {
    let C2CompleteGeneration {
        target,
        memory_item,
        correction_proposal,
        work_project,
        work_task,
        git_repository,
        local_checkout,
        monorepo_component,
        project_repository_link,
    } = generation;

    let mut memory_rows = Vec::with_capacity(memory_item.len());
    for item in memory_item {
        memory_rows.push(json_memory_row(item)?);
    }
    let mut proposal_rows = Vec::with_capacity(correction_proposal.len());
    for proposal in correction_proposal {
        proposal_rows.push(json_proposal_row(proposal)?);
    }
    let mut project_rows = Vec::with_capacity(work_project.len());
    for project in work_project {
        project_rows.push(json_project_row(project)?);
    }
    let mut task_rows = Vec::with_capacity(work_task.len());
    for task in work_task {
        task_rows.push(json_task_row(task)?);
    }
    let mut repository_rows = Vec::with_capacity(git_repository.len());
    for repository in git_repository {
        repository_rows.push(json_repository_row(repository)?);
    }
    let mut checkout_rows = Vec::with_capacity(local_checkout.len());
    for checkout in local_checkout {
        checkout_rows.push(json_checkout_row(checkout)?);
    }
    let mut component_rows = Vec::with_capacity(monorepo_component.len());
    for component in monorepo_component {
        component_rows.push(json_component_row(component)?);
    }
    let mut link_rows = Vec::with_capacity(project_repository_link.len());
    for link in project_repository_link {
        link_rows.push(json_link_row(link)?);
    }

    Ok(C2RawSnapshot {
        target,
        memory_item: memory_rows,
        correction_proposal: proposal_rows,
        memory_forget_receipt: Vec::new(),
        work_project: project_rows,
        work_task: task_rows,
        git_repository: repository_rows,
        local_checkout: checkout_rows,
        monorepo_component: component_rows,
        project_repository_link: link_rows,
    })
}

fn serialized_object(value: JsonValue) -> AcquisitionResult<JsonMap<String, JsonValue>> {
    value
        .as_object()
        .cloned()
        .ok_or(C2AcquisitionError::InvalidGeneration)
}

fn embedded_field(object: &JsonMap<String, JsonValue>, key: &str) -> AcquisitionResult<JsonValue> {
    object
        .get(key)
        .cloned()
        .ok_or(C2AcquisitionError::InvalidGeneration)
}

fn json_memory_row(item: MemoryItem) -> AcquisitionResult<JsonValue> {
    let embedded =
        serde_json::to_value(&item).map_err(|_| C2AcquisitionError::InvalidGeneration)?;
    let embedded_object = embedded
        .as_object()
        .ok_or(C2AcquisitionError::InvalidGeneration)?;
    let mut row = JsonMap::new();
    row.insert(
        "record_id".to_string(),
        JsonValue::String(item.id.to_string()),
    );
    row.insert("item".to_string(), embedded.clone());
    row.insert(
        "kind_key".to_string(),
        JsonValue::String(item.kind.to_string()),
    );
    row.insert(
        "status_key".to_string(),
        JsonValue::String(item.status.to_string()),
    );
    row.insert(
        "scope_key".to_string(),
        JsonValue::String(scope_key(&item.scope)),
    );
    row.insert(
        "harness_key".to_string(),
        JsonValue::String(item.writer.harness.to_string()),
    );
    row.insert(
        "model_key".to_string(),
        JsonValue::String(item.writer.model.model.clone()),
    );
    row.insert(
        "session_id".to_string(),
        serde_json::to_value(item.writer.session_id)
            .map_err(|_| C2AcquisitionError::InvalidGeneration)?,
    );
    row.insert(
        "snapshot_digest".to_string(),
        JsonValue::String(
            snapshot_digest(&item).map_err(|_| C2AcquisitionError::InvalidGeneration)?,
        ),
    );
    row.insert(
        "created_at".to_string(),
        embedded_field(embedded_object, "created_at")?,
    );
    row.insert(
        "updated_at".to_string(),
        embedded_field(embedded_object, "updated_at")?,
    );
    Ok(JsonValue::Object(row))
}

fn json_proposal_row(proposal: CorrectionProposal) -> AcquisitionResult<JsonValue> {
    let embedded =
        serde_json::to_value(&proposal).map_err(|_| C2AcquisitionError::InvalidGeneration)?;
    let embedded_object = embedded
        .as_object()
        .ok_or(C2AcquisitionError::InvalidGeneration)?;
    let mut row = JsonMap::new();
    row.insert(
        "record_id".to_string(),
        JsonValue::String(proposal.id.to_string()),
    );
    row.insert("proposal".to_string(), embedded.clone());
    row.insert(
        "status_key".to_string(),
        JsonValue::String(proposal.status.to_string()),
    );
    row.insert(
        "obsolete_id".to_string(),
        JsonValue::String(proposal.obsolete_id.to_string()),
    );
    if proposal.status == CorrectionProposalStatus::Pending {
        row.insert(
            "pending_obsolete_id".to_string(),
            JsonValue::String(proposal.obsolete_id.to_string()),
        );
    }
    row.insert(
        "replacement_id".to_string(),
        JsonValue::String(proposal.replacement_id.to_string()),
    );
    row.insert(
        "snapshot_digest".to_string(),
        JsonValue::String(
            snapshot_digest(&proposal).map_err(|_| C2AcquisitionError::InvalidGeneration)?,
        ),
    );
    row.insert(
        "created_at".to_string(),
        embedded_field(embedded_object, "created_at")?,
    );
    Ok(JsonValue::Object(row))
}

fn json_project_row(project: Project) -> AcquisitionResult<JsonValue> {
    let mut row = serialized_object(
        serde_json::to_value(&project).map_err(|_| C2AcquisitionError::InvalidGeneration)?,
    )?;
    if row.contains_key("record_id") || row.remove("id").is_none() {
        return Err(C2AcquisitionError::InvalidGeneration);
    }
    row.insert(
        "record_id".to_string(),
        JsonValue::String(project.id.to_string()),
    );
    Ok(JsonValue::Object(row))
}

fn json_task_row(task: Task) -> AcquisitionResult<JsonValue> {
    let mut row = serialized_object(
        serde_json::to_value(&task).map_err(|_| C2AcquisitionError::InvalidGeneration)?,
    )?;
    if row.contains_key("record_id") || row.remove("id").is_none() {
        return Err(C2AcquisitionError::InvalidGeneration);
    }
    row.insert(
        "record_id".to_string(),
        JsonValue::String(task.id.to_string()),
    );
    Ok(JsonValue::Object(row))
}

fn json_repository_row(repository: GitRepository) -> AcquisitionResult<JsonValue> {
    let embedded =
        serde_json::to_value(&repository).map_err(|_| C2AcquisitionError::InvalidGeneration)?;
    let embedded_object = embedded
        .as_object()
        .ok_or(C2AcquisitionError::InvalidGeneration)?;
    let mut row = JsonMap::new();
    row.insert(
        "record_id".to_string(),
        JsonValue::String(repository.id.to_string()),
    );
    row.insert("repository".to_string(), embedded.clone());
    row.insert(
        "name_key".to_string(),
        JsonValue::String(repository.name.to_lowercase()),
    );
    row.insert(
        "remote_url".to_string(),
        serde_json::to_value(&repository.remote_url)
            .map_err(|_| C2AcquisitionError::InvalidGeneration)?,
    );
    row.insert(
        "provider_key".to_string(),
        JsonValue::String(repository.provider.to_string()),
    );
    row.insert(
        "created_at".to_string(),
        embedded_field(embedded_object, "created_at")?,
    );
    row.insert(
        "updated_at".to_string(),
        embedded_field(embedded_object, "updated_at")?,
    );
    Ok(JsonValue::Object(row))
}

fn json_checkout_row(checkout: LocalCheckout) -> AcquisitionResult<JsonValue> {
    let embedded =
        serde_json::to_value(&checkout).map_err(|_| C2AcquisitionError::InvalidGeneration)?;
    let embedded_object = embedded
        .as_object()
        .ok_or(C2AcquisitionError::InvalidGeneration)?;
    let mut row = JsonMap::new();
    row.insert(
        "record_id".to_string(),
        JsonValue::String(checkout.id.to_string()),
    );
    row.insert("checkout".to_string(), embedded.clone());
    row.insert(
        "repository_id".to_string(),
        serde_json::to_value(checkout.repository_id)
            .map_err(|_| C2AcquisitionError::InvalidGeneration)?,
    );
    row.insert(
        "local_path_key".to_string(),
        JsonValue::String(checkout.local_path.clone()),
    );
    row.insert(
        "current_branch".to_string(),
        serde_json::to_value(&checkout.current_branch)
            .map_err(|_| C2AcquisitionError::InvalidGeneration)?,
    );
    row.insert(
        "head_sha".to_string(),
        serde_json::to_value(&checkout.head_sha)
            .map_err(|_| C2AcquisitionError::InvalidGeneration)?,
    );
    row.insert(
        "is_dirty".to_string(),
        serde_json::to_value(checkout.is_dirty)
            .map_err(|_| C2AcquisitionError::InvalidGeneration)?,
    );
    for key in ["created_at", "updated_at", "last_seen_at"] {
        row.insert(key.to_string(), embedded_field(embedded_object, key)?);
    }
    Ok(JsonValue::Object(row))
}

fn json_component_row(component: MonorepoComponent) -> AcquisitionResult<JsonValue> {
    let embedded =
        serde_json::to_value(&component).map_err(|_| C2AcquisitionError::InvalidGeneration)?;
    let embedded_object = embedded
        .as_object()
        .ok_or(C2AcquisitionError::InvalidGeneration)?;
    let mut row = JsonMap::new();
    row.insert(
        "record_id".to_string(),
        JsonValue::String(component.id.to_string()),
    );
    row.insert("component".to_string(), embedded.clone());
    row.insert(
        "repository_id".to_string(),
        JsonValue::String(component.repository_id.to_string()),
    );
    row.insert(
        "name_key".to_string(),
        JsonValue::String(component.name.to_lowercase()),
    );
    row.insert(
        "path_key".to_string(),
        JsonValue::String(component.path.clone()),
    );
    row.insert(
        "kind".to_string(),
        serde_json::to_value(&component.kind).map_err(|_| C2AcquisitionError::InvalidGeneration)?,
    );
    row.insert(
        "created_at".to_string(),
        embedded_field(embedded_object, "created_at")?,
    );
    row.insert(
        "updated_at".to_string(),
        embedded_field(embedded_object, "updated_at")?,
    );
    Ok(JsonValue::Object(row))
}

fn json_link_row(link: ProjectRepositoryLink) -> AcquisitionResult<JsonValue> {
    let embedded =
        serde_json::to_value(&link).map_err(|_| C2AcquisitionError::InvalidGeneration)?;
    let embedded_object = embedded
        .as_object()
        .ok_or(C2AcquisitionError::InvalidGeneration)?;
    let mut row = JsonMap::new();
    row.insert(
        "record_id".to_string(),
        JsonValue::String(link.id.to_string()),
    );
    row.insert("link".to_string(), embedded.clone());
    row.insert(
        "project_id".to_string(),
        serde_json::to_value(link.project_id).map_err(|_| C2AcquisitionError::InvalidGeneration)?,
    );
    row.insert(
        "project_name_key".to_string(),
        JsonValue::String(link.project_name.to_lowercase()),
    );
    row.insert(
        "repository_id".to_string(),
        JsonValue::String(link.repository_id.to_string()),
    );
    row.insert(
        "component_id".to_string(),
        serde_json::to_value(link.component_id)
            .map_err(|_| C2AcquisitionError::InvalidGeneration)?,
    );
    row.insert(
        "component_path_key".to_string(),
        serde_json::to_value(&link.component_path)
            .map_err(|_| C2AcquisitionError::InvalidGeneration)?,
    );
    row.insert("role".to_string(), JsonValue::String(link.role.to_string()));
    row.insert(
        "created_at".to_string(),
        embedded_field(embedded_object, "created_at")?,
    );
    row.insert(
        "updated_at".to_string(),
        embedded_field(embedded_object, "updated_at")?,
    );
    Ok(JsonValue::Object(row))
}

fn top_level_datetime(table: TableKind, key: &str) -> bool {
    match table {
        TableKind::MemoryItem => matches!(key, "created_at" | "updated_at"),
        TableKind::CorrectionProposal => key == "created_at",
        TableKind::MemoryForgetReceipt => matches!(key, "created_at" | "completed_at"),
        TableKind::WorkProject
        | TableKind::WorkTask
        | TableKind::GitRepository
        | TableKind::MonorepoComponent
        | TableKind::ProjectRepositoryLink => matches!(key, "created_at" | "updated_at"),
        TableKind::LocalCheckout => {
            matches!(key, "created_at" | "updated_at" | "last_seen_at")
        }
    }
}

fn json_value_to_native(
    value: JsonValue,
    table: TableKind,
    top_level_key: Option<&str>,
) -> AcquisitionResult<NativeValue> {
    match value {
        JsonValue::Null => Ok(NativeValue::Null),
        JsonValue::Bool(value) => Ok(NativeValue::Bool(value)),
        JsonValue::Number(value) => {
            if let Some(value) = value.as_i64() {
                Ok(NativeValue::Number(NativeNumber::Int(value)))
            } else if let Some(value) = value.as_u64() {
                let value =
                    i64::try_from(value).map_err(|_| C2AcquisitionError::InvalidGeneration)?;
                Ok(NativeValue::Number(NativeNumber::Int(value)))
            } else {
                let value = value
                    .as_f64()
                    .filter(|value| value.is_finite())
                    .ok_or(C2AcquisitionError::InvalidGeneration)?;
                Ok(NativeValue::Number(NativeNumber::Float(value)))
            }
        }
        JsonValue::String(value) => {
            if top_level_key.is_some_and(|key| top_level_datetime(table, key)) {
                super::canonical_timestamp(&JsonValue::String(value.clone()))
                    .map_err(|_| C2AcquisitionError::InvalidGeneration)?;
                let datetime = NativeDatetime::try_from(value.as_str())
                    .map_err(|_| C2AcquisitionError::InvalidGeneration)?;
                if datetime.to_raw() != value {
                    return Err(C2AcquisitionError::InvalidGeneration);
                }
                Ok(NativeValue::Datetime(datetime))
            } else {
                let strand = Strand::new(value).ok_or(C2AcquisitionError::InvalidGeneration)?;
                Ok(NativeValue::Strand(strand))
            }
        }
        JsonValue::Array(values) => {
            if values.len() > MAX_VECTOR_ELEMENTS {
                return limit_error();
            }
            let mut native = Vec::with_capacity(values.len());
            for value in values {
                native.push(json_value_to_native(value, table, None)?);
            }
            Ok(NativeValue::Array(NativeArray::from(native)))
        }
        JsonValue::Object(values) => {
            if values.len() > MAX_OBJECT_KEYS {
                return limit_error();
            }
            let mut native = BTreeMap::new();
            for (key, value) in values {
                if key.contains('\0') || key.len() > MAX_ORDINARY_SCALAR_BYTES {
                    return Err(C2AcquisitionError::InvalidGeneration);
                }
                let value = json_value_to_native(value, table, None)?;
                if native.insert(key, value).is_some() {
                    return Err(C2AcquisitionError::InvalidGeneration);
                }
            }
            Ok(NativeValue::Object(NativeObject::from(native)))
        }
    }
}

fn json_row_to_native(table: TableKind, row: JsonValue) -> AcquisitionResult<NativeValue> {
    let mut object = row
        .as_object()
        .cloned()
        .ok_or(C2AcquisitionError::InvalidGeneration)?;
    if object.contains_key("id") {
        return Err(C2AcquisitionError::InvalidGeneration);
    }
    let record_id = object
        .remove("record_id")
        .and_then(|value| value.as_str().map(str::to_owned))
        .ok_or(C2AcquisitionError::InvalidGeneration)?;
    parse_id(&record_id).map_err(|_| C2AcquisitionError::InvalidGeneration)?;

    let mut native = BTreeMap::new();
    native.insert(
        "id".to_string(),
        NativeValue::Thing(Thing::from((table.tag().to_string(), record_id))),
    );
    for (key, value) in object {
        let converted = json_value_to_native(value, table, Some(&key))?;
        if native.insert(key, converted).is_some() {
            return Err(C2AcquisitionError::InvalidGeneration);
        }
    }
    Ok(NativeValue::Object(NativeObject::from(native)))
}

fn rows_to_native(table: TableKind, rows: Vec<JsonValue>) -> AcquisitionResult<NativeValue> {
    let mut native = Vec::with_capacity(rows.len());
    for row in rows {
        native.push(json_row_to_native(table, row)?);
    }
    Ok(NativeValue::Array(NativeArray::from(native)))
}

struct C2PreparedGeneration {
    target: C2RawTarget,
    accounting: RawAccounting,
    variables: BTreeMap<String, NativeValue>,
}

fn prepare_generation(generation: C2CompleteGeneration) -> AcquisitionResult<C2PreparedGeneration> {
    let borrowed = borrowed_generation_upper_bound(&generation)?;
    if borrowed.secret {
        return Err(C2AcquisitionError::SecretMaterial);
    }
    let exact_accounting = match borrowed.accounting {
        BorrowedLimitAccounting::Exact(accounting) => accounting,
        BorrowedLimitAccounting::InvalidExact | BorrowedLimitAccounting::InvalidLowerBound => {
            return Err(C2AcquisitionError::InvalidGeneration);
        }
    };
    validate_typed_logical_uniqueness(&generation)?;
    let raw = json_generation(generation)?;
    let boundary = validate_limit_and_secret_boundary(&raw, &[])?;
    if boundary.accounting != exact_accounting {
        return Err(C2AcquisitionError::InvalidGeneration);
    }
    validate_unique_raw_ids(&raw)?;
    strict_record_stage(&raw, &[]).map_err(|_| C2AcquisitionError::InvalidGeneration)?;
    if !raw.memory_forget_receipt.is_empty() {
        return Err(C2AcquisitionError::IncompleteDeletion);
    }

    let C2RawSnapshot {
        target,
        memory_item,
        correction_proposal,
        memory_forget_receipt,
        work_project,
        work_task,
        git_repository,
        local_checkout,
        monorepo_component,
        project_repository_link,
    } = raw;
    let rows = [
        memory_item,
        correction_proposal,
        memory_forget_receipt,
        work_project,
        work_task,
        git_repository,
        local_checkout,
        monorepo_component,
        project_repository_link,
    ];
    let mut variables = BTreeMap::new();
    for (table, rows) in TABLE_KINDS.into_iter().zip(rows) {
        variables.insert(table.tag().to_string(), rows_to_native(table, rows)?);
    }

    Ok(C2PreparedGeneration {
        target,
        accounting: boundary.accounting,
        variables,
    })
}

fn variables_are_exact(variables: &BTreeMap<String, NativeValue>) -> bool {
    variables.len() == TABLE_KINDS.len()
        && TABLE_KINDS
            .into_iter()
            .all(|table| variables.contains_key(table.tag()))
}

fn literal_matches(text: &str, expected_bytes: usize, expected_sha256: &str) -> bool {
    if text.len() != expected_bytes {
        return false;
    }
    let digest = Sha256::digest(text.as_bytes());
    let mut encoded = String::with_capacity(64);
    for byte in digest {
        use fmt::Write as _;
        if write!(&mut encoded, "{byte:02x}").is_err() {
            return false;
        }
    }
    encoded == expected_sha256
}

fn exact_insert(statement: &Statement, table: TableKind) -> bool {
    let Statement::Insert(insert) = statement else {
        return false;
    };
    let into_matches = matches!(
        &insert.into,
        Some(NativeValue::Table(into)) if into.as_str() == table.tag()
    );
    let data_matches = matches!(
        &insert.data,
        Data::SingleExpression(NativeValue::Param(parameter))
            if parameter.as_str() == table.tag()
    );
    into_matches
        && data_matches
        && !insert.ignore
        && insert.update.is_none()
        && matches!(insert.output, Some(Output::None))
        && insert.timeout.is_none()
        && !insert.parallel
        && !insert.relation
        && insert.version.is_none()
}

fn init_ast_is_exact(query: &Query) -> bool {
    let statements = &query.0 .0;
    statements.len() == 11
        && matches!(statements.first(), Some(Statement::Begin(_)))
        && TABLE_KINDS
            .into_iter()
            .enumerate()
            .all(|(index, table)| exact_insert(&statements[index + 1], table))
        && matches!(statements.last(), Some(Statement::Commit(_)))
}

fn parse_init_query() -> AcquisitionResult<Query> {
    if !literal_matches(INIT_QUERY, INIT_QUERY_BYTES, INIT_QUERY_SHA256) {
        return Err(C2AcquisitionError::QueryContractInvalid);
    }
    let query = surrealdb_core::syn::parse(INIT_QUERY)
        .map_err(|_| C2AcquisitionError::QueryContractInvalid)?;
    if !init_ast_is_exact(&query) {
        return Err(C2AcquisitionError::QueryContractInvalid);
    }
    Ok(query)
}

#[cfg(test)]
fn exact_delete(statement: &Statement, table: TableKind) -> bool {
    let Statement::Delete(delete) = statement else {
        return false;
    };
    !delete.only
        && delete.what.0.len() == 1
        && matches!(
            delete.what.0.first(),
            Some(NativeValue::Table(what)) if what.as_str() == table.tag()
        )
        && delete.with.is_none()
        && delete.cond.is_none()
        && matches!(delete.output, Some(Output::None))
        && delete.timeout.is_none()
        && !delete.parallel
        && delete.explain.is_none()
}

#[cfg(test)]
fn replace_ast_is_exact(query: &Query) -> bool {
    let statements = &query.0 .0;
    statements.len() == 20
        && matches!(statements.first(), Some(Statement::Begin(_)))
        && TABLE_KINDS
            .into_iter()
            .enumerate()
            .all(|(index, table)| exact_delete(&statements[index + 1], table))
        && TABLE_KINDS
            .into_iter()
            .enumerate()
            .all(|(index, table)| exact_insert(&statements[index + 10], table))
        && matches!(statements.last(), Some(Statement::Commit(_)))
}

#[cfg(test)]
fn parse_replace_query() -> AcquisitionResult<Query> {
    if !literal_matches(REPLACE_QUERY, REPLACE_QUERY_BYTES, REPLACE_QUERY_SHA256) {
        return Err(C2AcquisitionError::QueryContractInvalid);
    }
    let query = surrealdb_core::syn::parse(REPLACE_QUERY)
        .map_err(|_| C2AcquisitionError::QueryContractInvalid)?;
    if !replace_ast_is_exact(&query) {
        return Err(C2AcquisitionError::QueryContractInvalid);
    }
    Ok(query)
}

fn exact_select(
    statement: &surrealdb_core::sql::statements::SelectStatement,
    table: TableKind,
) -> bool {
    let cap_plus_one = i64::try_from(table.row_cap() + 1).expect("fixed table cap fits i64");
    statement.expr.0.len() == 1
        && matches!(statement.expr.0.first(), Some(Field::All))
        && !statement.expr.1
        && statement.omit.is_none()
        && !statement.only
        && statement.what.0.len() == 1
        && matches!(
            statement.what.0.first(),
            Some(NativeValue::Table(from)) if from.as_str() == table.tag()
        )
        && statement.with.is_none()
        && statement.cond.is_none()
        && statement.split.is_none()
        && statement.group.is_none()
        && statement.order.is_none()
        && matches!(
            &statement.limit,
            Some(limit)
                if matches!(limit.0, NativeValue::Number(NativeNumber::Int(value)) if value == cap_plus_one)
        )
        && statement.start.is_none()
        && statement.fetch.is_none()
        && statement.version.is_none()
        && statement.timeout.is_none()
        && !statement.parallel
        && statement.explain.is_none()
        && !statement.tempfiles
}

fn read_ast_is_exact(query: &Query) -> bool {
    let [Statement::Output(output)] = query.0 .0.as_slice() else {
        return false;
    };
    if output.fetch.is_some() {
        return false;
    }
    let NativeValue::Object(object) = &output.what else {
        return false;
    };
    if object.len() != TABLE_KINDS.len() {
        return false;
    }
    TABLE_KINDS.into_iter().all(|table| {
        matches!(
            object.get(table.tag()),
            Some(NativeValue::Subquery(subquery))
                if matches!(subquery.as_ref(), Subquery::Select(select) if exact_select(select, table))
        )
    })
}

fn parse_read_query() -> AcquisitionResult<Query> {
    if !literal_matches(READ_QUERY, READ_QUERY_BYTES, READ_QUERY_SHA256) {
        return Err(C2AcquisitionError::QueryContractInvalid);
    }
    let query = surrealdb_core::syn::parse(READ_QUERY)
        .map_err(|_| C2AcquisitionError::QueryContractInvalid)?;
    if !read_ast_is_exact(&query) {
        return Err(C2AcquisitionError::QueryContractInvalid);
    }
    Ok(query)
}

fn exact_write_outcomes(responses: Vec<surrealdb_core::dbs::Response>, expected: usize) -> bool {
    let outcomes = responses
        .into_iter()
        .map(|response| response.output().map_err(|_| ()))
        .collect();
    exact_write_results(outcomes, expected)
}

fn require_exact_write_outcomes(
    responses: Vec<surrealdb_core::dbs::Response>,
    expected: usize,
) -> AcquisitionResult<()> {
    let outcomes = responses
        .into_iter()
        .map(|response| response.output().map_err(|_| ()))
        .collect();
    require_exact_write_results(outcomes, expected)
}

fn require_exact_write_results(
    outcomes: Vec<Result<NativeValue, ()>>,
    expected: usize,
) -> AcquisitionResult<()> {
    if exact_write_results(outcomes, expected) {
        Ok(())
    } else {
        Err(C2AcquisitionError::EngineOutcomeUncertain)
    }
}

fn map_engine_failure<T, E>(result: Result<T, E>) -> AcquisitionResult<T> {
    result.map_err(|_| C2AcquisitionError::EngineFailure)
}

fn map_engine_outcome_uncertain<T, E>(result: Result<T, E>) -> AcquisitionResult<T> {
    result.map_err(|_| C2AcquisitionError::EngineOutcomeUncertain)
}

fn exact_write_results(outcomes: Vec<Result<NativeValue, ()>>, expected: usize) -> bool {
    outcomes.len() == expected
        && outcomes
            .into_iter()
            .all(|outcome| outcome.is_ok_and(|value| exact_write_value(&value)))
}

fn exact_write_value(value: &NativeValue) -> bool {
    matches!(value, NativeValue::Array(values) if values.is_empty())
}

fn exact_read_output(
    responses: Vec<surrealdb_core::dbs::Response>,
) -> AcquisitionResult<NativeValue> {
    let outcomes = responses
        .into_iter()
        .map(|response| response.output().map_err(|_| ()))
        .collect();
    exact_read_results(outcomes)
}

fn exact_read_results(
    mut outcomes: Vec<Result<NativeValue, ()>>,
) -> AcquisitionResult<NativeValue> {
    if outcomes.iter().any(Result::is_err) {
        return Err(C2AcquisitionError::EngineFailure);
    }
    if outcomes.len() != 1 {
        return Err(C2AcquisitionError::InvalidResponse);
    }
    outcomes
        .pop()
        .ok_or(C2AcquisitionError::InvalidResponse)?
        .map_err(|_| C2AcquisitionError::EngineFailure)
}

#[cfg(test)]
#[derive(Clone, Copy)]
enum Fixture {
    A,
    B,
}

#[cfg(test)]
impl Fixture {
    const fn suffix(self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
        }
    }
}

#[cfg(test)]
#[derive(Clone, Copy)]
enum InitialFixture {
    A,
    Permutation,
    ReversedPermutation,
}

#[cfg(test)]
impl C2MemoryOwner<C2Empty> {
    async fn initialize_for_test(
        fixture: InitialFixture,
    ) -> AcquisitionResult<C2MemoryOwner<C2Ready>> {
        let query = parse_init_query()?;
        let _read_query = parse_read_query()?;
        let generation = tests::initial_generation(fixture);
        let prepared = prepare_generation(generation)?;
        if !variables_are_exact(&prepared.variables) {
            return Err(C2AcquisitionError::InvalidGeneration);
        }

        let datastore = map_engine_failure(Datastore::new("memory").await)?
            .with_capabilities(Capabilities::none());
        let session = Session::owner().with_ns("engram").with_db("main");
        let owner = C2MemoryOwner {
            datastore,
            session,
            target: prepared.target,
            initial_accounting: prepared.accounting,
            state: PhantomData::<C2Empty>,
        };
        let C2MemoryOwner {
            datastore,
            session,
            target,
            initial_accounting,
            state: _,
        } = owner;
        let responses = datastore
            .process(query, &session, Some(prepared.variables))
            .await
            .map_err(|_| C2AcquisitionError::EngineOutcomeUncertain)?;
        if !exact_write_outcomes(responses, TABLE_KINDS.len()) {
            return Err(C2AcquisitionError::EngineOutcomeUncertain);
        }
        Ok(C2MemoryOwner {
            datastore,
            session,
            target,
            initial_accounting,
            state: PhantomData::<C2Ready>,
        })
    }
}

struct NativeTables {
    tables: Vec<(TableKind, NativeArray)>,
}

fn native_tables(value: NativeValue) -> AcquisitionResult<NativeTables> {
    let NativeValue::Object(mut object) = value else {
        return Err(C2AcquisitionError::InvalidResponse);
    };
    if object.len() != TABLE_KINDS.len()
        || object
            .keys()
            .any(|key| !TABLE_KINDS.into_iter().any(|table| table.tag() == key))
    {
        return Err(C2AcquisitionError::InvalidResponse);
    }
    let mut tables = Vec::with_capacity(TABLE_KINDS.len());
    for table in TABLE_KINDS {
        let Some(NativeValue::Array(rows)) = object.remove(table.tag()) else {
            return Err(C2AcquisitionError::InvalidResponse);
        };
        tables.push((table, rows));
    }
    Ok(NativeTables { tables })
}

fn native_overflow_mask(tables: &NativeTables) -> u16 {
    tables.tables.iter().fold(0_u16, |mask, (table, rows)| {
        if rows.len() > table.row_cap() {
            mask | table.overflow_bit()
        } else {
            mask
        }
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NoneAction {
    Null,
    Omit,
    Reject,
}

fn none_action(schema: SchemaPath, key: &str) -> NoneAction {
    use SchemaPath as S;
    match (schema, key) {
        (S::MemoryItem, "correction_proposal_id" | "pending_correction_proposal_id")
        | (S::ProcedurePrerequisite, "source")
        | (S::Proposal, "applied_digest")
        | (S::Component, "source_path" | "source_sha256")
        | (S::ForgetReceiptRow, "completed_at") => NoneAction::Omit,

        (S::MemoryRow, "session_id")
        | (S::MemoryItem, "last_used_at" | "review_after" | "archive" | "procedure")
        | (S::Scope, "project_id" | "project_name" | "task_id" | "entity_id")
        | (S::Scope, "repository_id" | "remote_url" | "local_path")
        | (S::Writer, "harness_version" | "surface" | "session_id")
        | (S::Model, "version")
        | (S::Evidence, "summary" | "excerpt")
        | (S::Archive, "archived_by")
        | (S::Procedure, "expires_at")
        | (S::ProcedureVerification, "evidence_path" | "evidence_sha256" | "verified_at")
        | (S::Proposal, "applied_at")
        | (S::ProjectRow, "description")
        | (S::TaskRow, "description" | "jira_key")
        | (S::RepositoryRow, "remote_url")
        | (S::Repository, "remote_url" | "default_branch" | "description")
        | (S::CheckoutRow, "repository_id" | "current_branch" | "head_sha" | "is_dirty")
        | (S::Checkout, "repository_id" | "current_branch" | "head_sha" | "is_dirty")
        | (S::ComponentRow, "kind")
        | (S::Component, "kind" | "description")
        | (S::LinkRow, "project_id" | "component_id" | "component_path_key")
        | (S::Link, "project_id" | "component_id" | "component_path") => NoneAction::Null,
        _ => NoneAction::Reject,
    }
}

fn object_none_action(schema: SchemaPath, key: &str, proposal_applied: bool) -> NoneAction {
    if matches!(schema, SchemaPath::ProposalRow) && key == "pending_obsolete_id" {
        return if proposal_applied {
            NoneAction::Omit
        } else {
            NoneAction::Reject
        };
    }
    none_action(schema, key)
}

struct JsonNumberCounter {
    bytes: usize,
}

impl std::io::Write for JsonNumberCounter {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        self.bytes = self
            .bytes
            .checked_add(buffer.len())
            .ok_or_else(|| std::io::Error::other("bounded number length overflow"))?;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn native_number_spelling_len(number: &NativeNumber) -> AcquisitionResult<usize> {
    let json_number = match number {
        NativeNumber::Int(value) => serde_json::Number::from(*value),
        NativeNumber::Float(value) => serde_json::Number::from_f64(*value)
            .ok_or(C2AcquisitionError::UnsupportedNativeValue)?,
        NativeNumber::Decimal(_) => return Err(C2AcquisitionError::UnsupportedNativeValue),
        _ => return Err(C2AcquisitionError::UnsupportedNativeValue),
    };
    let mut counter = JsonNumberCounter { bytes: 0 };
    serde_json::to_writer(&mut counter, &json_number)
        .map_err(|_| C2AcquisitionError::InvalidResponse)?;
    Ok(counter.bytes)
}

fn dry_add(meter: &mut Meter, row: &mut Option<RowBytes>, amount: u64) -> AcquisitionResult<()> {
    super::add_framed_bytes(meter, row, amount).map_err(C2AcquisitionError::from)
}

fn dry_enter(meter: &mut Meter, depth: u64) -> AcquisitionResult<()> {
    meter.enter_node(depth).map_err(C2AcquisitionError::from)
}

fn dry_child_depth(depth: u64) -> AcquisitionResult<u64> {
    super::child_depth(depth).map_err(C2AcquisitionError::from)
}

fn dry_len(base: u64, length: usize) -> AcquisitionResult<u64> {
    super::add_len(base, length).map_err(C2AcquisitionError::from)
}

fn canonical_thing_id(table: TableKind, thing: &Thing) -> AcquisitionResult<&str> {
    if thing.tb != table.tag() {
        return Err(C2AcquisitionError::InvalidResponse);
    }
    let NativeId::String(id) = &thing.id else {
        return Err(C2AcquisitionError::InvalidResponse);
    };
    parse_id(id).map_err(|_| C2AcquisitionError::InvalidResponse)?;
    Ok(id)
}

#[derive(Default)]
struct DryNativeStatus {
    unsupported: bool,
    invalid_response: bool,
}

fn dry_native_value(
    value: &NativeValue,
    depth: u64,
    schema: SchemaPath,
    table: TableKind,
    top_level_key: Option<&str>,
    meter: &mut Meter,
    row: &mut Option<RowBytes>,
    status: &mut DryNativeStatus,
) -> AcquisitionResult<()> {
    dry_enter(meter, depth)?;
    if top_level_key.is_some_and(|key| top_level_datetime(table, key)) {
        if let NativeValue::Datetime(value) = value {
            let canonical = value.to_raw();
            if super::canonical_timestamp(&JsonValue::String(canonical.clone())).is_err() {
                status.invalid_response = true;
            }
            if canonical.len() > schema.scalar_cap() {
                return limit_error();
            }
            return dry_add(meter, row, dry_len(1 + 8, canonical.len())?);
        }
        status.invalid_response = true;
    }
    match value {
        NativeValue::Null | NativeValue::Bool(_) => dry_add(meter, row, 1),
        NativeValue::Number(number) => match native_number_spelling_len(number) {
            Ok(length) => dry_add(meter, row, dry_len(1 + 8, length)?),
            Err(C2AcquisitionError::UnsupportedNativeValue) => {
                status.unsupported = true;
                Ok(())
            }
            Err(error) => Err(error),
        },
        NativeValue::Strand(value) => {
            if value.len() > schema.scalar_cap() {
                return limit_error();
            }
            dry_add(meter, row, dry_len(1 + 8, value.len())?)
        }
        NativeValue::Array(values) => {
            if values.len() > schema.vector_cap() {
                return limit_error();
            }
            dry_add(meter, row, 1 + 8)?;
            let depth = dry_child_depth(depth)?;
            for value in values.iter() {
                dry_native_value(
                    value,
                    depth,
                    schema.element(),
                    table,
                    None,
                    meter,
                    row,
                    status,
                )?;
            }
            Ok(())
        }
        NativeValue::Object(values) => {
            dry_native_object(values, depth, schema, table, false, meter, row, status)
        }
        NativeValue::None => {
            status.unsupported = true;
            Ok(())
        }
        _ => {
            status.unsupported = true;
            Ok(())
        }
    }
}

fn dry_native_object(
    values: &NativeObject,
    depth: u64,
    schema: SchemaPath,
    table: TableKind,
    row_top_level: bool,
    meter: &mut Meter,
    row: &mut Option<RowBytes>,
    status: &mut DryNativeStatus,
) -> AcquisitionResult<()> {
    if values.len() > MAX_OBJECT_KEYS {
        return limit_error();
    }
    let stored_record_id_collision = row_top_level && values.contains_key("record_id");
    if stored_record_id_collision {
        status.invalid_response = true;
    }
    if row_top_level {
        match values.get("id") {
            Some(NativeValue::Thing(thing)) => {
                if thing.tb != table.tag() {
                    status.invalid_response = true;
                }
                match &thing.id {
                    NativeId::String(id) => {
                        if parse_id(id).is_err() {
                            status.invalid_response = true;
                        }
                    }
                    _ => status.invalid_response = true,
                }
            }
            _ => status.invalid_response = true,
        }
    }
    let proposal_applied = matches!(
        values.get("status_key"),
        Some(NativeValue::Strand(status)) if status.as_str() == "applied"
    );

    let mut omitted = 0_usize;
    for (key, value) in values.iter() {
        if matches!(value, NativeValue::None) {
            match object_none_action(schema, key, proposal_applied) {
                NoneAction::Omit => omitted += 1,
                NoneAction::Null => {}
                NoneAction::Reject => status.unsupported = true,
            }
        }
    }
    let normalized_count = values
        .len()
        .checked_sub(omitted)
        .ok_or(C2AcquisitionError::InvalidResponse)?;
    if normalized_count > MAX_OBJECT_KEYS {
        return limit_error();
    }
    dry_add(meter, row, 1 + 8)?;
    let child_depth = dry_child_depth(depth)?;

    for (key, value) in values.iter() {
        if key.len() > MAX_ORDINARY_SCALAR_BYTES {
            return limit_error();
        }
        if matches!(value, NativeValue::None)
            && object_none_action(schema, key, proposal_applied) == NoneAction::Omit
        {
            continue;
        }
        if row_top_level && key == "id" && stored_record_id_collision {
            match value {
                NativeValue::Thing(thing) => {
                    if thing.tb.len() > MAX_ORDINARY_SCALAR_BYTES
                        || matches!(
                            &thing.id,
                            NativeId::String(id) if id.len() > MAX_ORDINARY_SCALAR_BYTES
                        )
                    {
                        return limit_error();
                    }
                }
                _ => dry_native_value(
                    value,
                    child_depth,
                    SchemaPath::OrdinaryScalar,
                    table,
                    None,
                    meter,
                    row,
                    status,
                )?,
            }
            continue;
        }
        let normalized_key = if row_top_level && key == "id" {
            "record_id"
        } else {
            key.as_str()
        };
        dry_enter(meter, child_depth)?;
        dry_add(meter, row, dry_len(8, normalized_key.len())?)?;

        if row_top_level && key == "id" {
            match value {
                NativeValue::Thing(thing) => {
                    dry_enter(meter, child_depth)?;
                    match &thing.id {
                        NativeId::String(id) => {
                            if id.len() > MAX_ORDINARY_SCALAR_BYTES {
                                return limit_error();
                            }
                            dry_add(meter, row, dry_len(1 + 8, id.len())?)?;
                        }
                        _ => dry_add(meter, row, 1)?,
                    }
                }
                _ => dry_native_value(
                    value,
                    child_depth,
                    SchemaPath::OrdinaryScalar,
                    table,
                    None,
                    meter,
                    row,
                    status,
                )?,
            }
        } else if matches!(value, NativeValue::None) {
            match object_none_action(schema, key, proposal_applied) {
                NoneAction::Null => {
                    dry_enter(meter, child_depth)?;
                    dry_add(meter, row, 1)?;
                }
                NoneAction::Omit => unreachable!("omitted member was skipped"),
                NoneAction::Reject => {
                    dry_enter(meter, child_depth)?;
                }
            }
        } else {
            dry_native_value(
                value,
                child_depth,
                schema.child(key),
                table,
                if row_top_level { Some(key) } else { None },
                meter,
                row,
                status,
            )?;
        }
    }
    Ok(())
}

struct DryNativePass {
    accounting: RawAccounting,
    unsupported: bool,
    invalid_response: bool,
}

fn dry_native_tables(
    target: &C2RawTarget,
    tables: &NativeTables,
) -> AcquisitionResult<DryNativePass> {
    let mut meter = Meter::new();
    super::account_object_header(&mut meter, 0, 11)?;
    super::account_key(&mut meter, 1, "target")?;
    super::account_target(target, &mut meter)?;
    super::account_key(&mut meter, 1, "prior_correction_bindings")?;
    super::account_array_header(&mut meter, 1, 0, 0)?;

    let mut status = DryNativeStatus::default();
    for (table, rows) in &tables.tables {
        super::account_key(&mut meter, 1, table.tag())?;
        super::account_array_header(&mut meter, 1, rows.len(), table.row_cap())?;
        for value in rows.iter() {
            let mut row = Some(RowBytes::new());
            match value {
                NativeValue::Object(object) => {
                    dry_enter(&mut meter, 2)?;
                    dry_native_object(
                        object,
                        2,
                        row_schema(*table),
                        *table,
                        true,
                        &mut meter,
                        &mut row,
                        &mut status,
                    )?;
                }
                _ => {
                    status.invalid_response = true;
                    dry_native_value(
                        value,
                        2,
                        row_schema(*table),
                        *table,
                        None,
                        &mut meter,
                        &mut row,
                        &mut status,
                    )?;
                }
            }
        }
    }
    Ok(DryNativePass {
        accounting: meter.finish(),
        unsupported: status.unsupported,
        invalid_response: status.invalid_response,
    })
}

fn secret_in_native_value(value: &NativeValue, under_credential_field: bool) -> bool {
    match value {
        NativeValue::Strand(value) => {
            likely_secret_in_string(value)
                || (under_credential_field && is_secret_field_string(value))
        }
        NativeValue::Array(values) => values
            .iter()
            .any(|value| secret_in_native_value(value, under_credential_field)),
        NativeValue::Object(values) => values.iter().any(|(key, value)| {
            likely_secret_in_string(key)
                || secret_in_native_value(value, under_credential_field || is_credential_field(key))
        }),
        NativeValue::Thing(thing) => {
            likely_secret_in_string(&thing.tb)
                || (under_credential_field && is_secret_field_string(&thing.tb))
                || matches!(
                    &thing.id,
                    NativeId::String(id)
                        if likely_secret_in_string(id)
                            || (under_credential_field && is_secret_field_string(id))
                )
        }
        NativeValue::None
        | NativeValue::Null
        | NativeValue::Bool(_)
        | NativeValue::Number(_)
        | NativeValue::Datetime(_) => false,
        _ => false,
    }
}

fn secret_in_native_tables(tables: &NativeTables) -> bool {
    tables.tables.iter().any(|(_, rows)| {
        rows.iter()
            .any(|value| secret_in_native_value(value, false))
    })
}

fn normalize_native_value(
    value: NativeValue,
    schema: SchemaPath,
    table: TableKind,
    top_level_key: Option<&str>,
) -> AcquisitionResult<JsonValue> {
    match value {
        NativeValue::Null => Ok(JsonValue::Null),
        NativeValue::Bool(value) => Ok(JsonValue::Bool(value)),
        NativeValue::Number(number) => match number {
            NativeNumber::Int(value) => Ok(JsonValue::Number(value.into())),
            NativeNumber::Float(value) => serde_json::Number::from_f64(value)
                .map(JsonValue::Number)
                .ok_or(C2AcquisitionError::UnsupportedNativeValue),
            NativeNumber::Decimal(_) => Err(C2AcquisitionError::UnsupportedNativeValue),
            _ => Err(C2AcquisitionError::UnsupportedNativeValue),
        },
        NativeValue::Strand(value) => {
            if top_level_key.is_some_and(|key| top_level_datetime(table, key)) {
                return Err(C2AcquisitionError::InvalidResponse);
            }
            if value.len() > schema.scalar_cap() {
                return limit_error();
            }
            Ok(JsonValue::String(value.0))
        }
        NativeValue::Datetime(value)
            if top_level_key.is_some_and(|key| top_level_datetime(table, key)) =>
        {
            let canonical = value.to_raw();
            super::canonical_timestamp(&JsonValue::String(canonical.clone()))
                .map_err(|_| C2AcquisitionError::InvalidResponse)?;
            Ok(JsonValue::String(canonical))
        }
        NativeValue::Array(values) => {
            if values.len() > schema.vector_cap() {
                return limit_error();
            }
            let mut normalized = Vec::with_capacity(values.len());
            for value in values.0 {
                normalized.push(normalize_native_value(
                    value,
                    schema.element(),
                    table,
                    None,
                )?);
            }
            Ok(JsonValue::Array(normalized))
        }
        NativeValue::Object(values) => normalize_native_object(values, schema, table, false),
        _ => Err(C2AcquisitionError::UnsupportedNativeValue),
    }
}

fn normalize_native_object(
    values: NativeObject,
    schema: SchemaPath,
    table: TableKind,
    row_top_level: bool,
) -> AcquisitionResult<JsonValue> {
    if values.len() > MAX_OBJECT_KEYS {
        return limit_error();
    }
    if row_top_level && values.contains_key("record_id") {
        return Err(C2AcquisitionError::InvalidResponse);
    }
    if row_top_level && !matches!(values.get("id"), Some(NativeValue::Thing(_))) {
        return Err(C2AcquisitionError::InvalidResponse);
    }
    let proposal_applied = matches!(
        values.get("status_key"),
        Some(NativeValue::Strand(status)) if status.as_str() == "applied"
    );
    let mut normalized = JsonMap::new();
    for (key, value) in values.0 {
        if key.len() > MAX_ORDINARY_SCALAR_BYTES {
            return limit_error();
        }
        if row_top_level && key == "id" {
            let NativeValue::Thing(thing) = value else {
                return Err(C2AcquisitionError::InvalidResponse);
            };
            let id = canonical_thing_id(table, &thing)?.to_owned();
            if normalized
                .insert("record_id".to_string(), JsonValue::String(id))
                .is_some()
            {
                return Err(C2AcquisitionError::InvalidResponse);
            }
            continue;
        }
        if matches!(value, NativeValue::None) {
            match object_none_action(schema, &key, proposal_applied) {
                NoneAction::Null => {
                    normalized.insert(key, JsonValue::Null);
                }
                NoneAction::Omit => {}
                NoneAction::Reject => {
                    return Err(C2AcquisitionError::UnsupportedNativeValue);
                }
            }
            continue;
        }
        let value = normalize_native_value(
            value,
            schema.child(&key),
            table,
            if row_top_level { Some(&key) } else { None },
        )?;
        if normalized.insert(key, value).is_some() {
            return Err(C2AcquisitionError::InvalidResponse);
        }
    }
    Ok(JsonValue::Object(normalized))
}

fn normalize_native_tables(
    target: C2RawTarget,
    tables: NativeTables,
) -> AcquisitionResult<C2RawSnapshot> {
    let mut normalized = C2RawSnapshot {
        target,
        memory_item: Vec::new(),
        correction_proposal: Vec::new(),
        memory_forget_receipt: Vec::new(),
        work_project: Vec::new(),
        work_task: Vec::new(),
        git_repository: Vec::new(),
        local_checkout: Vec::new(),
        monorepo_component: Vec::new(),
        project_repository_link: Vec::new(),
    };
    for (table, rows) in tables.tables {
        let destination = match table {
            TableKind::MemoryItem => &mut normalized.memory_item,
            TableKind::CorrectionProposal => &mut normalized.correction_proposal,
            TableKind::MemoryForgetReceipt => &mut normalized.memory_forget_receipt,
            TableKind::WorkProject => &mut normalized.work_project,
            TableKind::WorkTask => &mut normalized.work_task,
            TableKind::GitRepository => &mut normalized.git_repository,
            TableKind::LocalCheckout => &mut normalized.local_checkout,
            TableKind::MonorepoComponent => &mut normalized.monorepo_component,
            TableKind::ProjectRepositoryLink => &mut normalized.project_repository_link,
        };
        destination.reserve(rows.len());
        for row in rows.0 {
            let NativeValue::Object(object) = row else {
                return Err(C2AcquisitionError::InvalidResponse);
            };
            destination.push(normalize_native_object(
                object,
                row_schema(table),
                table,
                true,
            )?);
        }
    }
    Ok(normalized)
}

fn mint_mac_key() -> AcquisitionResult<C2OneShotMacKey> {
    let mut key = C2OneShotMacKey([0_u8; 32]);
    getrandom::fill(&mut key.0).map_err(|_| C2AcquisitionError::EntropyUnavailable)?;
    Ok(key)
}

#[cfg(test)]
fn mint_mac_key_with(
    fill: impl FnOnce(&mut [u8; 32]) -> Result<(), ()>,
) -> AcquisitionResult<C2OneShotMacKey> {
    let mut key = C2OneShotMacKey([0_u8; 32]);
    fill(&mut key.0).map_err(|()| C2AcquisitionError::EntropyUnavailable)?;
    Ok(key)
}

fn invoke_c2a(raw: C2RawSnapshot, key: C2OneShotMacKey) -> AcquisitionResult<C2ValidatedSnapshot> {
    #[cfg(test)]
    tests::note_c2a_invoked();
    validate_semantic_snapshot(raw, Vec::new(), key).map_err(C2AcquisitionError::from)
}

fn validate_collected_value(
    value: NativeValue,
    target: C2RawTarget,
    initial_accounting: RawAccounting,
) -> AcquisitionResult<C2RawSnapshot> {
    let tables = native_tables(value)?;
    let overflow_mask = native_overflow_mask(&tables);
    if overflow_mask != 0 {
        return Err(C2AcquisitionError::LimitExceeded {
            table_overflow_mask: Some(overflow_mask),
        });
    }
    let dry = dry_native_tables(&target, &tables)?;
    if secret_in_native_tables(&tables) {
        return Err(C2AcquisitionError::SecretMaterial);
    }
    if dry.unsupported {
        return Err(C2AcquisitionError::UnsupportedNativeValue);
    }
    if dry.invalid_response {
        return Err(C2AcquisitionError::InvalidResponse);
    }
    let raw = normalize_native_tables(target, tables)?;
    let normalized_accounting = account_virtual_input(&raw, &[])?;
    if normalized_accounting != dry.accounting || normalized_accounting != initial_accounting {
        return Err(C2AcquisitionError::InvalidResponse);
    }
    let boundary = validate_limit_and_secret_boundary(&raw, &[])?;
    if boundary.accounting != normalized_accounting {
        return Err(C2AcquisitionError::InvalidResponse);
    }
    Ok(raw)
}

#[cfg(test)]
impl C2MemoryOwner<C2Ready> {
    #[cfg(test)]
    async fn replace_for_test(self, fixture: Fixture) -> AcquisitionResult<C2MemoryOwner<C2Ready>> {
        let query = parse_replace_query()?;
        let generation = tests::fixture_generation(fixture);
        let prepared = prepare_generation(generation)?;
        if !same_target(&self.target, &prepared.target) {
            return Err(C2AcquisitionError::InvalidGeneration);
        }
        let C2MemoryOwner {
            datastore,
            session,
            target: _,
            initial_accounting: _,
            state: _,
        } = self;
        let responses = datastore
            .process(query, &session, Some(prepared.variables))
            .await
            .map_err(|_| C2AcquisitionError::EngineOutcomeUncertain)?;
        if !exact_write_outcomes(responses, 2 * TABLE_KINDS.len()) {
            return Err(C2AcquisitionError::EngineOutcomeUncertain);
        }
        Ok(C2MemoryOwner {
            datastore,
            session,
            target: prepared.target,
            initial_accounting: prepared.accounting,
            state: PhantomData::<C2Ready>,
        })
    }

    #[cfg(test)]
    async fn read_raw_for_test(&self) -> AcquisitionResult<C2RawSnapshot> {
        let query = parse_read_query()?;
        let responses = self
            .datastore
            .process(query, &self.session, None)
            .await
            .map_err(|_| C2AcquisitionError::EngineFailure)?;
        let value = exact_read_output(responses)?;
        validate_collected_value(
            value,
            copy_target_for_test(&self.target),
            RawAccounting {
                nodes: self.initial_accounting.nodes,
                bytes: self.initial_accounting.bytes,
                maximum_depth: self.initial_accounting.maximum_depth,
            },
        )
    }

    #[cfg(test)]
    async fn observe_for_test(&self) -> AcquisitionResult<C2ValidatedSnapshot> {
        let raw = self.read_raw_for_test().await?;
        invoke_c2a(
            raw,
            mint_mac_key_with(|bytes| {
                bytes.fill(0x5a);
                Ok(())
            })?,
        )
    }

    #[cfg(test)]
    async fn rollback_probe_for_test(
        self,
    ) -> AcquisitionResult<(C2RawSnapshot, C2ValidatedSnapshot)> {
        let replace_query = parse_replace_query()?;
        let read_query = parse_read_query()?;
        let mut prepared = prepare_generation(tests::fixture_generation(Fixture::B))?;
        if !same_target(&self.target, &prepared.target) {
            return Err(C2AcquisitionError::InvalidGeneration);
        }
        prepared
            .variables
            .insert(TableKind::WorkTask.tag().to_string(), NativeValue::Null);
        let C2MemoryOwner {
            datastore,
            session,
            target,
            initial_accounting,
            state: _,
        } = self;
        let responses = datastore
            .process(replace_query, &session, Some(prepared.variables))
            .await
            .map_err(|_| C2AcquisitionError::EngineOutcomeUncertain)?;
        if exact_write_outcomes(responses, 2 * TABLE_KINDS.len()) {
            return Err(C2AcquisitionError::EngineOutcomeUncertain);
        }
        let responses = datastore
            .process(read_query, &session, None)
            .await
            .map_err(|_| C2AcquisitionError::EngineFailure)?;
        let value = exact_read_output(responses)?;
        let raw = validate_collected_value(value, target, initial_accounting)?;
        let validated = invoke_c2a(
            copy_raw_for_test(&raw),
            mint_mac_key_with(|bytes| {
                bytes.fill(0x6b);
                Ok(())
            })?,
        )?;
        Ok((raw, validated))
    }
}

#[cfg(test)]
fn same_target(left: &C2RawTarget, right: &C2RawTarget) -> bool {
    left.project_id == right.project_id
        && left.project_name == right.project_name
        && left.repository_id == right.repository_id
        && left.repository_remote == right.repository_remote
        && left.checkout_id == right.checkout_id
        && left.checkout_path == right.checkout_path
        && left.task_id == right.task_id
        && left.task_name == right.task_name
}

#[cfg(test)]
fn copy_target_for_test(target: &C2RawTarget) -> C2RawTarget {
    let mut meter = BorrowedJsonUpperMeter::new();
    meter
        .target(target)
        .expect("test target must be bounded before cloning");
    C2RawTarget {
        project_id: target.project_id.clone(),
        project_name: target.project_name.clone(),
        repository_id: target.repository_id.clone(),
        repository_remote: target.repository_remote.clone(),
        checkout_id: target.checkout_id.clone(),
        checkout_path: target.checkout_path.clone(),
        task_id: target.task_id.clone(),
        task_name: target.task_name.clone(),
    }
}

#[cfg(test)]
fn copy_raw_for_test(raw: &C2RawSnapshot) -> C2RawSnapshot {
    account_virtual_input(raw, &[]).expect("test raw snapshot must be bounded before cloning");
    C2RawSnapshot {
        target: copy_target_for_test(&raw.target),
        memory_item: raw.memory_item.clone(),
        correction_proposal: raw.correction_proposal.clone(),
        memory_forget_receipt: raw.memory_forget_receipt.clone(),
        work_project: raw.work_project.clone(),
        work_task: raw.work_task.clone(),
        git_repository: raw.git_repository.clone(),
        local_checkout: raw.local_checkout.clone(),
        monorepo_component: raw.monorepo_component.clone(),
        project_repository_link: raw.project_repository_link.clone(),
    }
}

impl C2MemoryOwner<C2Empty> {
    async fn acquire_complete_generation(
        generation: C2CompleteGeneration,
    ) -> AcquisitionResult<C2ValidatedSnapshot> {
        let init_query = parse_init_query()?;
        let read_query = parse_read_query()?;
        let prepared = prepare_generation(generation)?;
        if !variables_are_exact(&prepared.variables) {
            return Err(C2AcquisitionError::InvalidGeneration);
        }

        #[cfg(test)]
        tests::controlled_await_seam(0).await;
        let datastore = map_engine_failure(Datastore::new("memory").await)?
            .with_capabilities(Capabilities::none());
        let session = Session::owner().with_ns("engram").with_db("main");
        let empty_owner = C2MemoryOwner {
            datastore,
            session,
            target: prepared.target,
            initial_accounting: prepared.accounting,
            state: PhantomData::<C2Empty>,
        };
        let C2MemoryOwner {
            datastore,
            session,
            target,
            initial_accounting,
            state: _,
        } = empty_owner;
        #[cfg(test)]
        tests::controlled_await_seam(1).await;
        let responses = map_engine_outcome_uncertain(
            datastore
                .process(init_query, &session, Some(prepared.variables))
                .await,
        )?;
        require_exact_write_outcomes(responses, TABLE_KINDS.len())?;
        let ready_owner = C2MemoryOwner {
            datastore,
            session,
            target,
            initial_accounting,
            state: PhantomData::<C2Ready>,
        };
        let C2MemoryOwner {
            datastore,
            session,
            target,
            initial_accounting,
            state: _,
        } = ready_owner;
        #[cfg(test)]
        tests::controlled_await_seam(2).await;
        let responses = map_engine_failure(datastore.process(read_query, &session, None).await)?;
        let value = exact_read_output(responses)?;
        let raw = validate_collected_value(value, target, initial_accounting)?;
        let key = mint_mac_key()?;
        invoke_c2a(raw, key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::cell::Cell;

    const PROJECT_ID: &str = "01890f5e-7b00-7000-8000-000000000010";
    const REPOSITORY_ID: &str = "01890f5e-7b00-7000-8000-000000000011";
    const CHECKOUT_ID: &str = "01890f5e-7b00-7000-8000-000000000012";
    const TIMESTAMP: &str = "2026-09-06T00:00:00Z";
    const REMOTE: &str = "https://github.com/ymeiri/engram.git";
    const CHECKOUT_PATH: &str = "/workspace/engram";
    const CHILD_NORMALIZED_SHA256: &str =
        "d96cb9e3e283c7f8195e63c81ee12726411f1dcfbb926e2373ab35d56dff81a9";

    thread_local! {
        static C2A_INVOCATIONS: Cell<usize> = const { Cell::new(0) };
        static CONTROLLED_CANCEL_AT: Cell<Option<u8>> = const { Cell::new(None) };
    }

    pub(super) fn note_c2a_invoked() {
        C2A_INVOCATIONS.with(|count| count.set(count.get() + 1));
    }

    pub(super) async fn controlled_await_seam(index: u8) {
        assert!(
            index <= 2,
            "controlled await index exceeded its fixed bound"
        );
        let should_pause = CONTROLLED_CANCEL_AT.with(|target| target.get() == Some(index));
        if should_pause {
            std::future::pending::<()>().await;
        }
    }

    fn reset_c2a_invocations() {
        C2A_INVOCATIONS.with(|count| count.set(0));
    }

    fn c2a_invocations() -> usize {
        C2A_INVOCATIONS.with(Cell::get)
    }

    fn variant_id(fixture: Fixture, offset: u64) -> String {
        assert!(offset <= 205, "fixture id offset exceeded its fixed bound");
        let base = match fixture {
            Fixture::A => 0xa000,
            Fixture::B => 0xb000,
        };
        format!("01890f5e-7b00-7000-8000-{:012x}", base + offset)
    }

    fn test_strand(value: &str) -> Strand {
        assert!(
            value.len() <= MAX_LARGE_SCALAR_BYTES + 1,
            "native string injector exceeded its fixed bound"
        );
        Strand::new(value.to_owned()).expect("bounded test string contains no NUL")
    }

    fn assert_serialized_upper<T: serde::Serialize>(
        value: &T,
        charge: impl Fn(&mut BorrowedJsonUpperMeter) -> AcquisitionResult<()>,
    ) {
        let mut meter = BorrowedJsonUpperMeter::new();
        charge(&mut meter).unwrap();
        let serialized = serde_json::to_value(value).unwrap();
        assert!(meter.bytes >= serde_json::to_vec(&serialized).unwrap().len() as u64);

        let mut exact = BorrowedJsonUpperMeter::with_exact_accounting();
        charge(&mut exact).unwrap();
        let (actual, invalid, lower_bound) = exact.finish_exact_accounting().unwrap();
        assert!(!invalid);
        assert!(!lower_bound);
        let mut expected = Meter::new();
        super::super::walk_value(
            &serialized,
            0,
            SchemaPath::Unknown,
            &mut expected,
            &mut None,
        )
        .unwrap();
        assert_eq!(actual, expected.finish());
    }

    pub(super) fn fixture_generation(fixture: Fixture) -> C2CompleteGeneration {
        let suffix = fixture.suffix();
        let proposal_id = variant_id(fixture, 1);
        let obsolete_id = variant_id(fixture, 2);
        let replacement_id = variant_id(fixture, 3);
        let task_id = variant_id(fixture, 4);
        let primary_link_id = variant_id(fixture, 5);
        let component_id = variant_id(fixture, 6);
        let component_link_id = variant_id(fixture, 7);
        let writer = json!({
            "harness": "codex",
            "harness_version": null,
            "model": {
                "provider": "openai",
                "model": format!("fixture-{suffix}"),
                "version": null
            },
            "surface": null,
            "actor": "agent",
            "session_id": null,
            "written_at": TIMESTAMP
        });
        let scope = json!({
            "type": "project",
            "project_id": PROJECT_ID,
            "project_name": "engram"
        });
        let obsolete: MemoryItem = serde_json::from_value(json!({
            "id": obsolete_id,
            "kind": "decision",
            "title": format!("obsolete {suffix}"),
            "content": format!("obsolete content {suffix}"),
            "scope": scope,
            "origin": "user_stated",
            "writer": writer,
            "evidence": [],
            "confidence": 0.8,
            "status": "active",
            "supersedes": [],
            "tags": [format!("variant-{suffix}")],
            "created_at": TIMESTAMP,
            "updated_at": TIMESTAMP,
            "last_used_at": null,
            "review_after": null,
            "archive": null,
            "procedure": null,
            "pending_correction_proposal_id": proposal_id
        }))
        .unwrap();
        let replacement: MemoryItem = serde_json::from_value(json!({
            "id": replacement_id,
            "kind": "decision",
            "title": format!("replacement {suffix}"),
            "content": format!("replacement content {suffix}"),
            "scope": scope,
            "origin": "agent_inferred",
            "writer": writer,
            "evidence": [{
                "kind": "file",
                "target": format!("fixture-{suffix}.md"),
                "summary": format!("evidence {suffix}"),
                "excerpt": null,
                "observed_at": TIMESTAMP
            }],
            "confidence": 0.8,
            "status": "needs_review",
            "supersedes": [],
            "tags": [format!("variant-{suffix}")],
            "created_at": TIMESTAMP,
            "updated_at": TIMESTAMP,
            "last_used_at": null,
            "review_after": null,
            "archive": null,
            "procedure": null,
            "correction_proposal_id": proposal_id
        }))
        .unwrap();
        let mut proposal: CorrectionProposal = serde_json::from_value(json!({
            "id": proposal_id,
            "obsolete_id": obsolete_id,
            "replacement_id": replacement_id,
            "memory_kind": "decision",
            "scope": scope,
            "canonical_digest": "0".repeat(64),
            "digest_schema_version": 1,
            "status": "pending",
            "proposer": writer,
            "created_at": TIMESTAMP,
            "applied_at": null
        }))
        .unwrap();
        proposal.canonical_digest =
            super::super::correction_digest(&proposal, &obsolete, &replacement).unwrap();
        let project: Project = serde_json::from_value(json!({
            "id": PROJECT_ID,
            "name": "engram",
            "description": format!("project {suffix}"),
            "status": "active",
            "created_at": TIMESTAMP,
            "updated_at": TIMESTAMP
        }))
        .unwrap();
        let task: Task = serde_json::from_value(json!({
            "id": task_id,
            "project_id": PROJECT_ID,
            "name": format!("task {suffix}"),
            "description": null,
            "status": "in_progress",
            "priority": "high",
            "jira_key": "ENG-42",
            "blocked_by": [],
            "created_at": TIMESTAMP,
            "updated_at": TIMESTAMP
        }))
        .unwrap();
        let repository: GitRepository = serde_json::from_value(json!({
            "id": REPOSITORY_ID,
            "name": "engram",
            "remote_url": REMOTE,
            "provider": "git_hub",
            "default_branch": "main",
            "description": format!("repository {suffix}"),
            "created_at": TIMESTAMP,
            "updated_at": TIMESTAMP
        }))
        .unwrap();
        let checkout: LocalCheckout = serde_json::from_value(json!({
            "id": CHECKOUT_ID,
            "repository_id": REPOSITORY_ID,
            "local_path": CHECKOUT_PATH,
            "current_branch": "main",
            "head_sha": if suffix == "A" {
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            } else {
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
            },
            "is_dirty": false,
            "created_at": TIMESTAMP,
            "updated_at": TIMESTAMP,
            "last_seen_at": TIMESTAMP
        }))
        .unwrap();
        let component: MonorepoComponent = serde_json::from_value(json!({
            "id": component_id,
            "repository_id": REPOSITORY_ID,
            "name": format!("component-{suffix}"),
            "path": format!("components/{suffix}"),
            "kind": "crate",
            "description": format!("component {suffix}"),
            "source_path": format!("components/{suffix}/Cargo.toml"),
            "source_sha256": if suffix == "A" { "a".repeat(64) } else { "b".repeat(64) },
            "created_at": TIMESTAMP,
            "updated_at": TIMESTAMP
        }))
        .unwrap();
        let primary_link: ProjectRepositoryLink = serde_json::from_value(json!({
            "id": primary_link_id,
            "project_id": PROJECT_ID,
            "project_name": "engram",
            "repository_id": REPOSITORY_ID,
            "component_id": null,
            "component_path": null,
            "role": "primary",
            "created_at": TIMESTAMP,
            "updated_at": TIMESTAMP
        }))
        .unwrap();
        let component_link: ProjectRepositoryLink = serde_json::from_value(json!({
            "id": component_link_id,
            "project_id": PROJECT_ID,
            "project_name": "engram",
            "repository_id": REPOSITORY_ID,
            "component_id": component_id,
            "component_path": format!("components/{suffix}"),
            "role": "related",
            "created_at": TIMESTAMP,
            "updated_at": TIMESTAMP
        }))
        .unwrap();
        C2CompleteGeneration {
            target: C2RawTarget {
                project_id: PROJECT_ID.to_string(),
                project_name: "engram".to_string(),
                repository_id: REPOSITORY_ID.to_string(),
                repository_remote: REMOTE.to_string(),
                checkout_id: CHECKOUT_ID.to_string(),
                checkout_path: CHECKOUT_PATH.to_string(),
                task_id: None,
                task_name: None,
            },
            memory_item: vec![obsolete, replacement],
            correction_proposal: vec![proposal],
            work_project: vec![project],
            work_task: vec![task],
            git_repository: vec![repository],
            local_checkout: vec![checkout],
            monorepo_component: vec![component],
            project_repository_link: vec![primary_link, component_link],
        }
    }

    fn cumulative_memory(index: usize, content_length: usize) -> MemoryItem {
        assert!(
            index <= 100,
            "cumulative fixture index exceeded its fixed bound"
        );
        assert!(
            content_length <= MAX_LARGE_SCALAR_BYTES + 1,
            "cumulative fixture content exceeded its fixed bound"
        );
        serde_json::from_value(json!({
            "id": format!("01890f5e-7b00-7000-8000-{:012x}", 0xc000_u64 + index as u64),
            "kind": "project_fact",
            "title": format!("cumulative-{index:02}"),
            "content": "x".repeat(content_length),
            "scope": {"type": "global"},
            "origin": "user_stated",
            "writer": {
                "harness": "codex",
                "harness_version": null,
                "model": {"provider": "openai", "model": "cumulative", "version": null},
                "surface": null,
                "actor": "agent",
                "session_id": null,
                "written_at": TIMESTAMP
            },
            "evidence": [],
            "confidence": 0.8,
            "status": "active",
            "supersedes": [],
            "tags": [],
            "created_at": TIMESTAMP,
            "updated_at": TIMESTAMP,
            "last_used_at": null,
            "review_after": null,
            "archive": null,
            "procedure": null
        }))
        .unwrap()
    }

    fn cumulative_generation(last_content_length: usize) -> C2CompleteGeneration {
        assert!(
            last_content_length <= MAX_LARGE_SCALAR_BYTES + 1,
            "cumulative generation exceeded its fixed bound"
        );
        let mut generation = fixture_generation(Fixture::A);
        for index in 0..62 {
            let content_length = if index == 61 {
                last_content_length
            } else {
                MAX_LARGE_SCALAR_BYTES
            };
            generation
                .memory_item
                .push(cumulative_memory(index, content_length));
        }
        generation
    }

    fn permutation_generation() -> C2CompleteGeneration {
        let mut generation = fixture_generation(Fixture::A);
        let mut second_correction = fixture_generation(Fixture::B);
        generation
            .memory_item
            .append(&mut second_correction.memory_item);
        generation
            .correction_proposal
            .append(&mut second_correction.correction_proposal);
        let project_id = variant_id(Fixture::A, 200);
        let task_id = variant_id(Fixture::A, 201);
        let repository_id = variant_id(Fixture::A, 202);
        let checkout_id = variant_id(Fixture::A, 203);
        let component_id = variant_id(Fixture::A, 204);
        let link_id = variant_id(Fixture::A, 205);
        generation.work_project.push(
            serde_json::from_value(json!({
                "id": project_id,
                "name": "secondary-project",
                "description": "secondary project",
                "status": "completed",
                "created_at": TIMESTAMP,
                "updated_at": TIMESTAMP
            }))
            .unwrap(),
        );
        generation.work_task.push(
            serde_json::from_value(json!({
                "id": task_id,
                "project_id": project_id,
                "name": "secondary task",
                "description": "secondary task description",
                "status": "todo",
                "priority": "low",
                "jira_key": null,
                "blocked_by": [],
                "created_at": TIMESTAMP,
                "updated_at": TIMESTAMP
            }))
            .unwrap(),
        );
        generation.git_repository.push(
            serde_json::from_value(json!({
                "id": repository_id,
                "name": "secondary-repository",
                "remote_url": "https://github.com/example/secondary.git",
                "provider": "git_hub",
                "default_branch": "trunk",
                "description": "secondary repository",
                "created_at": TIMESTAMP,
                "updated_at": TIMESTAMP
            }))
            .unwrap(),
        );
        generation.local_checkout.push(
            serde_json::from_value(json!({
                "id": checkout_id,
                "repository_id": repository_id,
                "local_path": "/workspace/secondary",
                "current_branch": "trunk",
                "head_sha": "cccccccccccccccccccccccccccccccccccccccc",
                "is_dirty": true,
                "created_at": TIMESTAMP,
                "updated_at": TIMESTAMP,
                "last_seen_at": TIMESTAMP
            }))
            .unwrap(),
        );
        generation.monorepo_component.push(
            serde_json::from_value(json!({
                "id": component_id,
                "repository_id": repository_id,
                "name": "secondary-component",
                "path": "components/secondary",
                "kind": "crate",
                "description": "secondary component",
                "source_path": "components/secondary/Cargo.toml",
                "source_sha256": "c".repeat(64),
                "created_at": TIMESTAMP,
                "updated_at": TIMESTAMP
            }))
            .unwrap(),
        );
        generation.project_repository_link.push(
            serde_json::from_value(json!({
                "id": link_id,
                "project_id": project_id,
                "project_name": "secondary-project",
                "repository_id": repository_id,
                "component_id": component_id,
                "component_path": "components/secondary",
                "role": "related",
                "created_at": TIMESTAMP,
                "updated_at": TIMESTAMP
            }))
            .unwrap(),
        );
        generation
    }

    pub(super) fn initial_generation(fixture: InitialFixture) -> C2CompleteGeneration {
        match fixture {
            InitialFixture::A => fixture_generation(Fixture::A),
            InitialFixture::Permutation => permutation_generation(),
            InitialFixture::ReversedPermutation => {
                let mut generation = permutation_generation();
                generation.memory_item.reverse();
                generation.correction_proposal.reverse();
                generation.work_project.reverse();
                generation.work_task.reverse();
                generation.git_repository.reverse();
                generation.local_checkout.reverse();
                generation.monorepo_component.reverse();
                generation.project_repository_link.reverse();
                generation
            }
        }
    }

    fn first_insert_mut(
        query: &mut Query,
    ) -> &mut surrealdb_core::sql::statements::InsertStatement {
        let Statement::Insert(insert) = &mut query.0 .0[1] else {
            panic!("fixed init statement must be insert");
        };
        insert
    }

    fn first_select_mut(
        query: &mut Query,
    ) -> &mut surrealdb_core::sql::statements::SelectStatement {
        let Statement::Output(output) = &mut query.0 .0[0] else {
            panic!("fixed read statement must be output");
        };
        let NativeValue::Object(object) = &mut output.what else {
            panic!("fixed output must be object");
        };
        let NativeValue::Subquery(subquery) = object.get_mut("memory_item").unwrap() else {
            panic!("fixed member must be subquery");
        };
        let Subquery::Select(select) = subquery.as_mut() else {
            panic!("fixed subquery must be select");
        };
        select
    }

    fn first_delete_mut(
        query: &mut Query,
    ) -> &mut surrealdb_core::sql::statements::DeleteStatement {
        let Statement::Delete(delete) = &mut query.0 .0[1] else {
            panic!("fixed replace statement must be delete");
        };
        delete
    }

    fn sorted_rows(rows: &[JsonValue]) -> Vec<&JsonValue> {
        let mut rows = rows.iter().collect::<Vec<_>>();
        rows.sort_unstable_by(|left, right| {
            let left_id = left
                .as_object()
                .and_then(|object| object.get("record_id"))
                .and_then(JsonValue::as_str)
                .unwrap_or_default();
            let right_id = right
                .as_object()
                .and_then(|object| object.get("record_id"))
                .and_then(JsonValue::as_str)
                .unwrap_or_default();
            left_id.as_bytes().cmp(right_id.as_bytes())
        });
        rows
    }

    fn raw_matches(left: &C2RawSnapshot, right: &C2RawSnapshot) -> bool {
        same_target(&left.target, &right.target)
            && TABLE_KINDS.into_iter().all(|table| {
                sorted_rows(super::super::table_rows(left, table))
                    == sorted_rows(super::super::table_rows(right, table))
            })
    }

    async fn shared_read(
        datastore: &Datastore,
        session: &Session,
        target: &C2RawTarget,
        accounting: &RawAccounting,
    ) -> AcquisitionResult<C2RawSnapshot> {
        let query = parse_read_query()?;
        let responses = datastore
            .process(query, session, None)
            .await
            .map_err(|_| C2AcquisitionError::EngineFailure)?;
        let value = exact_read_output(responses)?;
        validate_collected_value(
            value,
            copy_target_for_test(target),
            RawAccounting {
                nodes: accounting.nodes,
                bytes: accounting.bytes,
                maximum_depth: accounting.maximum_depth,
            },
        )
    }

    fn native_outer_from_generation(fixture: Fixture) -> (C2RawTarget, RawAccounting, NativeValue) {
        let C2PreparedGeneration {
            target,
            accounting,
            variables,
        } = prepare_generation(fixture_generation(fixture)).unwrap();
        (
            target,
            accounting,
            NativeValue::Object(NativeObject::from(variables)),
        )
    }

    fn native_row_mut(
        outer: &mut NativeValue,
        table: TableKind,
        index: usize,
    ) -> &mut NativeObject {
        assert_eq!(
            index, 0,
            "native row injector accepts only the fixed first row"
        );
        let NativeValue::Object(tables) = outer else {
            panic!("native response must be an object");
        };
        let Some(NativeValue::Array(rows)) = tables.get_mut(table.tag()) else {
            panic!("native table must be an array");
        };
        let Some(NativeValue::Object(row)) = rows.0.get_mut(index) else {
            panic!("native row must be an object");
        };
        row
    }

    fn collect_with_matching_accounting(
        outer: NativeValue,
        target: C2RawTarget,
    ) -> AcquisitionResult<C2RawSnapshot> {
        let mut bound_meter = Meter::new();
        let mut secret = None;
        let mut status = DryNativeStatus::default();
        dry_native_value(
            &outer,
            0,
            SchemaPath::Unknown,
            TableKind::MemoryItem,
            None,
            &mut bound_meter,
            &mut secret,
            &mut status,
        )?;
        assert!(!status.invalid_response);
        let normalized =
            normalize_native_tables(copy_target_for_test(&target), native_tables(outer.clone())?)?;
        let accounting = account_virtual_input(&normalized, &[])?;
        validate_collected_value(outer, target, accounting)
    }

    #[test]
    fn exact_literals_lengths_hashes_and_asts() {
        assert!(literal_matches(
            INIT_QUERY,
            INIT_QUERY_BYTES,
            INIT_QUERY_SHA256
        ));
        assert!(literal_matches(
            READ_QUERY,
            READ_QUERY_BYTES,
            READ_QUERY_SHA256
        ));
        assert!(literal_matches(
            REPLACE_QUERY,
            REPLACE_QUERY_BYTES,
            REPLACE_QUERY_SHA256
        ));
        assert!(parse_init_query().is_ok());
        assert!(parse_read_query().is_ok());
        assert!(parse_replace_query().is_ok());
        assert!(!literal_matches(
            &format!("{INIT_QUERY}\n"),
            INIT_QUERY_BYTES,
            INIT_QUERY_SHA256
        ));
        assert!(!literal_matches(
            &READ_QUERY.replacen("LIMIT 65", "LIMIT 64", 1),
            READ_QUERY_BYTES,
            READ_QUERY_SHA256
        ));
    }

    fn source_sha256(source: &str) -> String {
        let digest = Sha256::digest(source.as_bytes());
        let mut encoded = String::with_capacity(64);
        for byte in digest {
            use fmt::Write as _;
            write!(&mut encoded, "{byte:02x}").unwrap();
        }
        encoded
    }

    fn occurrence_count(source: &str, needle: &str) -> usize {
        source.match_indices(needle).count()
    }

    fn code_mask(source: &str) -> Vec<bool> {
        let bytes = source.as_bytes();
        let mut code = vec![true; bytes.len()];
        let mut index = 0;
        while index < bytes.len() {
            if bytes[index..].starts_with(b"//") {
                let start = index;
                index += 2;
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
                code[start..index].fill(false);
                continue;
            }
            if bytes[index..].starts_with(b"/*") {
                let start = index;
                index += 2;
                let mut depth = 1_usize;
                while index < bytes.len() && depth != 0 {
                    if bytes[index..].starts_with(b"/*") {
                        depth += 1;
                        index += 2;
                    } else if bytes[index..].starts_with(b"*/") {
                        depth -= 1;
                        index += 2;
                    } else {
                        index += 1;
                    }
                }
                code[start..index].fill(false);
                continue;
            }
            if bytes[index] == b'r' {
                let mut delimiter = index + 1;
                while delimiter < bytes.len() && bytes[delimiter] == b'#' {
                    delimiter += 1;
                }
                if delimiter < bytes.len() && bytes[delimiter] == b'"' {
                    let hashes = delimiter - index - 1;
                    let start = index;
                    index = delimiter + 1;
                    while index < bytes.len() {
                        if bytes[index] == b'"'
                            && index + 1 + hashes <= bytes.len()
                            && bytes[index + 1..index + 1 + hashes]
                                .iter()
                                .all(|byte| *byte == b'#')
                        {
                            index += 1 + hashes;
                            break;
                        }
                        index += 1;
                    }
                    code[start..index].fill(false);
                    continue;
                }
            }
            if bytes[index] == b'"' {
                let start = index;
                index += 1;
                while index < bytes.len() {
                    if bytes[index] == b'\\' {
                        index = usize::min(index + 2, bytes.len());
                    } else if bytes[index] == b'"' {
                        index += 1;
                        break;
                    } else {
                        index += 1;
                    }
                }
                code[start..index].fill(false);
                continue;
            }
            if bytes[index] == b'\'' {
                let start = index;
                let mut end = index + 1;
                if end < bytes.len() && bytes[end] == b'\\' {
                    end = usize::min(end + 2, bytes.len());
                } else {
                    end = usize::min(end + 1, bytes.len());
                }
                if end < bytes.len() && bytes[end] == b'\'' {
                    index = end + 1;
                    code[start..index].fill(false);
                    continue;
                }
            }
            index += 1;
        }
        code
    }

    fn cfg_test_item_end(source: &str, code: &[bool], attribute_start: usize) -> usize {
        let bytes = source.as_bytes();
        let mut cursor = attribute_start + "#[cfg(test)]".len();
        loop {
            while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            if cursor + 2 <= bytes.len() && code[cursor] && bytes[cursor..].starts_with(b"#[") {
                let mut brackets = 0_usize;
                while cursor < bytes.len() {
                    if code[cursor] && bytes[cursor] == b'[' {
                        brackets += 1;
                    } else if code[cursor] && bytes[cursor] == b']' {
                        brackets -= 1;
                        cursor += 1;
                        if brackets == 0 {
                            break;
                        }
                        continue;
                    }
                    cursor += 1;
                }
                continue;
            }
            break;
        }

        while cursor < bytes.len() {
            if code[cursor] && bytes[cursor] == b';' {
                return cursor + 1;
            }
            if code[cursor] && bytes[cursor] == b'{' {
                let mut braces = 1_usize;
                cursor += 1;
                while cursor < bytes.len() && braces != 0 {
                    if code[cursor] && bytes[cursor] == b'{' {
                        braces += 1;
                    } else if code[cursor] && bytes[cursor] == b'}' {
                        braces -= 1;
                    }
                    cursor += 1;
                }
                if cursor < bytes.len() && bytes[cursor] == b';' {
                    cursor += 1;
                }
                return cursor;
            }
            cursor += 1;
        }
        bytes.len()
    }

    fn production_projection(source: &str) -> String {
        let code = code_mask(source);
        let marker = "#[cfg(test)]";
        let mut projected = String::with_capacity(source.len());
        let mut copied_through = 0_usize;
        let mut index = 0_usize;
        while index + marker.len() <= source.len() {
            if code[index] && source[index..].starts_with(marker) {
                projected.push_str(&source[copied_through..index]);
                index = cfg_test_item_end(source, &code, index);
                copied_through = index;
            } else {
                index += 1;
            }
        }
        projected.push_str(&source[copied_through..]);
        projected
    }

    fn code_only(source: &str) -> String {
        let code = code_mask(source);
        source
            .bytes()
            .zip(code)
            .map(|(byte, is_code)| if is_code { char::from(byte) } else { ' ' })
            .collect()
    }

    #[test]
    fn source_dependency_owner_and_caller_firewalls_are_exact() {
        const CHILD: &str = include_str!("acquisition.rs");
        const PARENT: &str = include_str!("../native_successor_semantic.rs");
        const LIB: &str = include_str!("../lib.rs");
        const MAIN: &str = include_str!("../main.rs");
        const EVALUATOR_MANIFEST: &str = include_str!("../../Cargo.toml");
        const WORKSPACE_MANIFEST: &str = include_str!("../../../Cargo.toml");
        const LOCKFILE: &str = include_str!("../../../Cargo.lock");

        let self_declaration = format!(
            "const CHILD_NORMALIZED_SHA256: &str =\n        \"{}\";",
            CHILD_NORMALIZED_SHA256
        );
        assert_eq!(occurrence_count(CHILD, &self_declaration), 1);
        let normalized_declaration = format!(
            "const CHILD_NORMALIZED_SHA256: &str =\n        \"{}\";",
            "0".repeat(64)
        );
        let normalized_child = CHILD.replacen(&self_declaration, &normalized_declaration, 1);
        assert_eq!(source_sha256(&normalized_child), CHILD_NORMALIZED_SHA256);

        assert_eq!(
            source_sha256(PARENT),
            "0ae50d7abb0ab1640e03279b310c5d968439cf1b9480d9920b7a988a4dccc58a"
        );
        assert_eq!(
            source_sha256(LIB),
            "40183ec638c2baaa39629b461aa7b525d0614d7c14d539759c23bffa01584163"
        );
        assert_eq!(
            source_sha256(EVALUATOR_MANIFEST),
            "4176c6a4886492f7d5adaa0bdc3134d7e04bfc8d1536de4d5eb8d4b8985ce2bb"
        );
        assert_eq!(
            source_sha256(WORKSPACE_MANIFEST),
            "ec61531f8ffd401211f649c7b91e28a4ee5f0fa1a0fe2f199c63a841c9724f5a"
        );
        assert_eq!(
            source_sha256(LOCKFILE),
            "0eb924bb9008bb417cb9990081df8429520131aa6b3f7367866c5b805ec474de"
        );
        assert_eq!(occurrence_count(PARENT, "mod acquisition;"), 1);
        assert!(!PARENT.contains("pub mod acquisition"));
        assert!(!PARENT.contains("acquisition::"));
        assert!(!LIB.contains("acquisition"));
        assert!(!MAIN.contains("acquisition"));

        let exact_getrandom = "getrandom = \"=0.3.4\"";
        let exact_surreal = concat!(
            "surrealdb-core = { version = \"=2.6.0\", default-features = false, ",
            "features = [\"kv-mem\"] }"
        );
        assert_eq!(occurrence_count(EVALUATOR_MANIFEST, exact_getrandom), 1);
        assert_eq!(occurrence_count(EVALUATOR_MANIFEST, exact_surreal), 1);
        assert!(!EVALUATOR_MANIFEST.contains("surrealdb ="));
        let manifest: toml::Value = toml::from_str(EVALUATOR_MANIFEST).unwrap();
        let dependencies = manifest["dependencies"].as_table().unwrap();
        assert_eq!(dependencies["getrandom"].as_str(), Some("=0.3.4"));
        let surreal = dependencies["surrealdb-core"].as_table().unwrap();
        assert_eq!(surreal.len(), 3);
        assert_eq!(surreal["version"].as_str(), Some("=2.6.0"));
        assert_eq!(surreal["default-features"].as_bool(), Some(false));
        assert_eq!(
            surreal["features"].as_array().unwrap(),
            &[toml::Value::String("kv-mem".to_string())]
        );
        assert!(manifest.get("target").is_none());

        let production = production_projection(CHILD);
        assert!(!production.contains("#[cfg(test)]"));
        let production_code = code_only(&production);
        let child_code = code_only(CHILD);
        let compact_child = child_code
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();
        assert_eq!(
            occurrence_count(&production, "Datastore::new(\"memory\")"),
            1
        );
        assert_eq!(
            occurrence_count(&production, "getrandom::fill(&mut key.0)"),
            1
        );
        assert_eq!(occurrence_count(&production_code, ".process("), 2);
        assert_eq!(occurrence_count(&production_code, "macro_rules!"), 1);
        assert_eq!(
            occurrence_count(&production_code, "macro_rules! assert_not_impl_any"),
            1
        );
        assert_eq!(
            occurrence_count(&production_code, "surrealdb_core::syn::parse("),
            2
        );
        assert_eq!(
            occurrence_count(&production_code, "invoke_c2a(raw, key)"),
            1
        );
        assert_eq!(
            occurrence_count(&production, ".with_capabilities(Capabilities::none())"),
            1
        );
        assert_eq!(
            occurrence_count(
                &production,
                "Session::owner().with_ns(\"engram\").with_db(\"main\")"
            ),
            1
        );
        for forbidden in [
            "Arc<Datastore>",
            "Surreal<",
            "surrealdb::Surreal",
            "std::env",
            "std::fs",
            "std::net",
            "std::process",
            "tokio::spawn",
            "std::thread",
            "Command::new",
            "println!",
            "eprintln!",
            "dbg!",
            "tracing::",
            "log::",
            "SystemTime::now",
            "OffsetDateTime::now",
            "pub struct C2",
            "pub enum C2",
            "pub fn acquire_complete_generation",
            "pub(super) fn acquire_complete_generation",
            "pub(crate) fn acquire_complete_generation",
            "#[cfg(",
            "cfg_attr",
            "include!(",
            "#[path",
            ".bind(",
            "T: Serialize",
            "T: serde::Serialize",
            "where T: Serialize",
            "where T: serde::Serialize",
            "AcquisitionResult<Datastore>",
            "AcquisitionResult<Session>",
            "AcquisitionResult<C2MemoryOwner",
        ] {
            assert!(!production_code.contains(forbidden));
        }
        for test_only in [
            "REPLACE_QUERY",
            "Fixture",
            "InitialFixture",
            "initialize_for_test",
            "replace_for_test",
            "read_raw_for_test",
            "observe_for_test",
            "rollback_probe_for_test",
            "controlled_await_seam",
            "mint_mac_key_with",
            "copy_target_for_test",
            "copy_raw_for_test",
            "fixture_generation",
            "initial_generation",
            "variant_id",
            "test_strand",
            "cumulative_memory",
            "cumulative_generation",
            "permutation_generation",
            "first_insert_mut",
            "first_select_mut",
            "first_delete_mut",
            "sorted_rows",
            "raw_matches",
            "shared_read",
            "native_outer_from_generation",
            "native_row_mut",
            "collect_with_matching_accounting",
            "nested_native_object",
            "every_nonempty_write_value_shape",
            "Arc<Datastore>",
        ] {
            assert!(CHILD.contains(test_only));
            assert!(!production_code.contains(test_only));
        }

        for helper in [
            "initialize_for_test",
            "replace_for_test",
            "read_raw_for_test",
            "observe_for_test",
            "rollback_probe_for_test",
            "mint_mac_key_with",
            "same_target",
            "copy_target_for_test",
            "copy_raw_for_test",
            "note_c2a_invoked",
            "controlled_await_seam",
            "reset_c2a_invocations",
            "c2a_invocations",
            "variant_id",
            "test_strand",
            "fixture_generation",
            "cumulative_memory",
            "cumulative_generation",
            "permutation_generation",
            "initial_generation",
            "first_insert_mut",
            "first_select_mut",
            "first_delete_mut",
            "sorted_rows",
            "raw_matches",
            "shared_read",
            "native_outer_from_generation",
            "native_row_mut",
            "collect_with_matching_accounting",
            "nested_native_object",
            "every_nonempty_write_value_shape",
        ] {
            let declaration = format!("fn {helper}(");
            assert_eq!(occurrence_count(&child_code, &declaration), 1);
            assert_eq!(occurrence_count(&production_code, &declaration), 0);
        }

        for exact_signature in [
            "enumFixture{A,B,}",
            "enumInitialFixture{A,Permutation,ReversedPermutation,}",
            concat!(
                "asyncfninitialize_for_test(fixture:InitialFixture,)",
                "->AcquisitionResult<C2MemoryOwner<C2Ready>>"
            ),
            concat!(
                "asyncfnreplace_for_test(self,fixture:Fixture)",
                "->AcquisitionResult<C2MemoryOwner<C2Ready>>"
            ),
            concat!(
                "asyncfnrollback_probe_for_test(self,)",
                "->AcquisitionResult<(C2RawSnapshot,C2ValidatedSnapshot)>"
            ),
            "pub(super)fnfixture_generation(fixture:Fixture)->C2CompleteGeneration",
            "pub(super)fninitial_generation(fixture:InitialFixture)->C2CompleteGeneration",
            "fnvariant_id(fixture:Fixture,offset:u64)->String",
            "fntest_strand(value:&str)->Strand",
            "fnassert_serialized_upper<T:serde::Serialize>",
            "fncumulative_memory(index:usize,content_length:usize)->MemoryItem",
            "fncumulative_generation(last_content_length:usize)->C2CompleteGeneration",
            concat!(
                "asyncfnshared_read(datastore:&Datastore,session:&Session,",
                "target:&C2RawTarget,accounting:&RawAccounting,)",
                "->AcquisitionResult<C2RawSnapshot>"
            ),
            concat!(
                "fnnative_outer_from_generation(fixture:Fixture)",
                "->(C2RawTarget,RawAccounting,NativeValue)"
            ),
            concat!(
                "fnnative_row_mut(outer:&mutNativeValue,table:TableKind,index:usize,)",
                "->&mutNativeObject"
            ),
            concat!(
                "fncollect_with_matching_accounting(outer:NativeValue,target:C2RawTarget,)",
                "->AcquisitionResult<C2RawSnapshot>"
            ),
            "fnnested_native_object(levels:usize)->NativeValue",
            "fncopy_target_for_test(target:&C2RawTarget)->C2RawTarget",
            "fncopy_raw_for_test(raw:&C2RawSnapshot)->C2RawSnapshot",
        ] {
            assert!(
                compact_child.contains(exact_signature),
                "missing fixed test-seam signature {exact_signature}"
            );
        }
        for forbidden_signature in [
            "suffix:&str",
            "fixture_generation(\"",
            "native_outer_from_generation(\"",
            "replace_for_test(self,generation:C2CompleteGeneration)",
            "rollback_probe_for_test(self,generation:C2CompleteGeneration)",
            "test_strand(value:implInto<String>)",
        ] {
            assert!(!compact_child.contains(forbidden_signature));
        }

        let region = |start: &str, end: &str| {
            let start = child_code.find(start).unwrap();
            let end = child_code[start..].find(end).unwrap() + start;
            &child_code[start..end]
        };
        let variant = region("fn variant_id(", "fn test_strand(");
        assert!(variant.find("offset <= 205").unwrap() < variant.find("format!").unwrap());
        let strand = region("fn test_strand(", "fn assert_serialized_upper<");
        assert!(strand.find("value.len()").unwrap() < strand.find("value.to_owned()").unwrap());
        let cumulative = region("fn cumulative_memory(", "fn cumulative_generation(");
        assert!(
            cumulative.find("content_length <=").unwrap()
                < cumulative.find("serde_json::from_value").unwrap()
        );
        let cumulative_generation =
            region("fn cumulative_generation(", "fn permutation_generation(");
        assert!(
            cumulative_generation
                .find("last_content_length <=")
                .unwrap()
                < cumulative_generation.find("fixture_generation").unwrap()
        );
        let native_row = region("fn native_row_mut(", "fn collect_with_matching_accounting(");
        assert!(native_row.find("assert_eq!(").unwrap() < native_row.find("get_mut").unwrap());
        let native_collect = region(
            "fn collect_with_matching_accounting(",
            "fn exact_literals_lengths_hashes_and_asts(",
        );
        assert!(
            native_collect.find("dry_native_value").unwrap()
                < native_collect.find("outer.clone()").unwrap()
        );
        let target_copy = region("fn copy_target_for_test(", "fn copy_raw_for_test(");
        assert!(
            target_copy.find(".target(target)").unwrap() < target_copy.find(".clone()").unwrap()
        );
        let raw_copy = region("fn copy_raw_for_test(", "impl C2MemoryOwner<C2Empty>");
        assert!(
            raw_copy.find("account_virtual_input").unwrap() < raw_copy.find(".clone()").unwrap()
        );
        let rollback = region("fn rollback_probe_for_test(", "fn same_target(");
        assert!(!rollback.contains("Arc"));
        assert!(!rollback.contains("generation:"));
        assert!(rollback.contains("fixture_generation(Fixture::B)"));
        assert!(child_code.contains("self.object(7 + usize::from(pending))?;"));

        let exact_meter = region("enum ExactContainer", "struct BorrowedJsonUpperMeter");
        assert!(exact_meter.contains("meter: Meter"));
        assert!(exact_meter.contains("row: Option<RowBytes>"));
        assert!(exact_meter.contains("invalid: bool"));
        assert!(exact_meter.contains("lower_bound: bool"));
        assert!(!exact_meter.contains("snapshot_digest"));
        assert!(!exact_meter.contains("json_generation"));
        assert!(!exact_meter.contains(".format("));
        let borrowed_meter = region(
            "struct BorrowedJsonUpperMeter {",
            "fn generation_table_overflow_mask(",
        );
        assert!(borrowed_meter.contains("scan_secrets: bool"));
        assert_eq!(occurrence_count(borrowed_meter, "if self.scan_secrets"), 4);
        let borrowed_constructors =
            region("impl BorrowedJsonUpperMeter {", "fn begin_row(&mut self)");
        assert!(borrowed_constructors.contains(concat!(
            "fn new() -> Self {\n        Self {\n            bytes: 0,\n",
            "            secret: false,\n            scan_secrets: false,"
        )));
        assert!(borrowed_constructors.contains(concat!(
            "fn with_exact_accounting() -> Self {\n        Self {\n            bytes: 0,\n",
            "            secret: false,\n            scan_secrets: true,"
        )));
        let derived_scan = region(
            "fn derived_projection_contains_secret(",
            "fn visit_borrowed_generation(",
        );
        assert_eq!(occurrence_count(derived_scan, "scope_key("), 1);
        assert_eq!(occurrence_count(derived_scan, ".to_lowercase()"), 3);
        assert_eq!(occurrence_count(derived_scan, "likely_secret_in_string"), 4);
        let json_projection = region("fn json_memory_row(", "fn top_level_datetime(");
        assert_eq!(
            occurrence_count(json_projection, "scope_key(&item.scope)"),
            1
        );
        assert_eq!(occurrence_count(json_projection, ".to_lowercase()"), 3);
        let borrowed_visit = region(
            "fn visit_borrowed_generation(",
            "fn borrowed_generation_upper_bound(",
        );
        assert_eq!(occurrence_count(borrowed_visit, "meter.begin_row()?"), 8);
        assert_eq!(occurrence_count(borrowed_visit, "meter.finish_row()?"), 8);
        let borrowed_preflight =
            region("fn borrowed_generation_upper_bound(", "fn json_generation(");
        assert_eq!(
            occurrence_count(borrowed_preflight, "visit_borrowed_generation("),
            2
        );
        let upper_pass_start = borrowed_preflight
            .find("let mut upper = BorrowedJsonUpperMeter::new()")
            .unwrap();
        let exact_pass_start = borrowed_preflight
            .find("let mut exact = BorrowedJsonUpperMeter::with_exact_accounting()")
            .unwrap();
        assert!(upper_pass_start < exact_pass_start);
        assert!(!borrowed_preflight[upper_pass_start..exact_pass_start].contains("secret"));
        assert!(
            borrowed_preflight
                .find("visit_borrowed_generation(&mut upper")
                .unwrap()
                < borrowed_preflight
                    .find("BorrowedJsonUpperMeter::with_exact_accounting()")
                    .unwrap()
        );
        assert!(
            borrowed_preflight
                .find("visit_borrowed_generation(&mut exact")
                .unwrap()
                < borrowed_preflight
                    .find("let source_secret = exact.secret")
                    .unwrap()
        );
        assert!(
            borrowed_preflight
                .find("finish_exact_accounting()")
                .unwrap()
                < borrowed_preflight
                    .find("derived_projection_contains_secret(generation)")
                    .unwrap()
        );
        let typed_uniqueness = region(
            "fn validate_typed_logical_uniqueness(",
            "fn json_generation(",
        );
        assert_eq!(occurrence_count(typed_uniqueness, "family_ids.insert("), 8);
        assert_eq!(occurrence_count(typed_uniqueness, "family_ids.clear()"), 7);
        for required in [
            "project.name.to_ascii_lowercase()",
            "selector.to_string()",
            "selector.to_ascii_lowercase()",
            "normalize_remote(remote)",
            "checkout.local_path.as_str()",
            "component.repository_id.as_uuid().as_bytes()",
            "component.path.as_str()",
            "project.name == link.project_name",
            "link.repository_id.as_uuid().as_bytes()",
            "link.component_id",
            "proposal_pairs.insert((obsolete_id, replacement_id))",
            "replacement_ids.contains(obsolete_id)",
        ] {
            assert!(typed_uniqueness.contains(required));
        }
        let prepare = region("fn prepare_generation(", "fn variables_are_exact(");
        assert!(
            prepare.find("borrowed.secret").unwrap()
                < prepare.find("match borrowed.accounting").unwrap()
        );
        assert!(
            prepare.find("match borrowed.accounting").unwrap()
                < prepare.find("json_generation(generation)").unwrap()
        );
        assert!(
            prepare
                .find("validate_typed_logical_uniqueness(&generation)")
                .unwrap()
                < prepare.find("json_generation(generation)").unwrap()
        );
        let dry_object = region("fn dry_native_object(", "struct DryNativePass");
        assert!(dry_object.contains("let stored_record_id_collision"));
        assert!(dry_object.contains("status.invalid_response = true"));
        let collision_branch = dry_object.find("if row_top_level && key ==").unwrap();
        let replacement_charge = dry_object.find("let normalized_key =").unwrap();
        assert!(collision_branch < replacement_charge);
        assert!(dry_object[collision_branch..replacement_charge].contains("dry_native_value("));
        let native_secret_scan =
            region("fn secret_in_native_value(", "fn secret_in_native_tables(");
        assert!(native_secret_scan.contains("NativeValue::Thing(thing)"));
        assert!(native_secret_scan.contains("likely_secret_in_string(&thing.tb)"));
        assert!(native_secret_scan.contains("NativeId::String(id)"));
        let collected = region(
            "fn validate_collected_value(",
            "#[cfg(test)]\nimpl C2MemoryOwner<C2Ready>",
        );
        assert!(
            collected.find("dry_native_tables").unwrap()
                < collected.find("secret_in_native_tables").unwrap()
        );
        assert!(
            collected.find("secret_in_native_tables").unwrap()
                < collected.find("dry.unsupported").unwrap()
        );
        assert!(
            collected.find("dry.unsupported").unwrap()
                < collected.find("dry.invalid_response").unwrap()
        );
        assert!(
            collected.find("dry.invalid_response").unwrap()
                < collected.find("normalize_native_tables").unwrap()
        );
        assert!(compact_child.contains(concat!(
            "enumBorrowedLimitAccounting{Exact(RawAccounting),",
            "InvalidExact,InvalidLowerBound,}"
        )));

        let entry_signature = concat!(
            "async fn acquire_complete_generation(\n",
            "        generation: C2CompleteGeneration,"
        );
        assert_eq!(occurrence_count(CHILD, entry_signature), 1);
        let entry_start = CHILD.find(entry_signature).unwrap();
        let test_module_start = CHILD.rfind("#[cfg(test)]\nmod tests {").unwrap();
        assert!(entry_start < test_module_start);
        let entry = &CHILD[entry_start..test_module_start];
        assert_eq!(occurrence_count(entry, "Datastore::new(\"memory\")"), 1);
        assert_eq!(occurrence_count(entry, ".process(init_query"), 1);
        assert_eq!(occurrence_count(entry, ".process(read_query"), 1);
        assert_eq!(occurrence_count(entry, "mint_mac_key()?"), 1);
        assert_eq!(occurrence_count(entry, "invoke_c2a(raw, key)"), 1);
        assert!(entry.contains(") -> AcquisitionResult<C2ValidatedSnapshot>"));
        for escaped in [
            "AcquisitionResult<Datastore>",
            "AcquisitionResult<Session>",
            "AcquisitionResult<C2MemoryOwner",
            "AcquisitionResult<C2RawSnapshot>",
            "AcquisitionResult<NativeValue>",
            "AcquisitionResult<Query>",
            "AcquisitionResult<C2OneShotMacKey>",
            " loop ",
            " while ",
            "retry",
            ".clone()",
        ] {
            assert!(!entry.contains(escaped));
        }
        for forbidden in [
            "Arc<Datastore>",
            "std::env",
            "std::fs",
            "std::net",
            "tokio::spawn",
            "std::thread",
            "surrealdb::Surreal",
            "pub fn",
            "pub(super)",
            "pub(crate)",
        ] {
            assert!(!entry.contains(forbidden));
        }

        let mint_entry = ["fn mint_", "mac_key()"].concat();
        assert_eq!(occurrence_count(CHILD, &mint_entry), 1);
        assert!(!CHILD.contains(&["getrandom", "_backend"].concat()));
        assert!(!CHILD.contains(&["CARGO_ENCODED_", "RUST", "FLAGS"].concat()));
        assert!(!CHILD.contains(&["RUST", "FLAGS"].concat()));

        const EXPECTED_PEERS: &[&str] = &[
            "bin/native-c2b2a.rs",
            "comparison.rs",
            "fixture.rs",
            "lean_probe.rs",
            "lib.rs",
            "main.rs",
            "native_audit.rs",
            "native_c2b2a.rs",
            "native_correction.rs",
            "native_document.rs",
            "native_execution.rs",
            "native_instructions_control.rs",
            "native_isolation.rs",
            "native_pilot.rs",
            "native_report.rs",
            "native_runner.rs",
            "native_stale.rs",
            "native_successor.rs",
            "native_successor_artifact.rs",
            "native_successor_core.rs",
            "native_successor_policy.rs",
            "native_successor_relay.rs",
            "native_successor_semantic.rs",
            "native_vm.rs",
            "pilot.rs",
            "procedure_probe.rs",
            "retrieval_probe.rs",
            "retrieval_probe_v2.rs",
            "retrieval_probe_v3.rs",
            "retrieval_probe_v4.rs",
            "retrieval_probe_v5.rs",
            "seed.rs",
        ];
        let source_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut directories = vec![source_root];
        let mut peers = Vec::new();
        while let Some(directory) = directories.pop() {
            for entry in std::fs::read_dir(directory).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    directories.push(path);
                    continue;
                }
                if path.extension().and_then(|value| value.to_str()) != Some("rs")
                    || path.ends_with("native_successor_semantic/acquisition.rs")
                {
                    continue;
                }
                let relative = path
                    .strip_prefix(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"))
                    .unwrap()
                    .to_string_lossy()
                    .into_owned();
                peers.push(relative);
                let peer = std::fs::read_to_string(&path).unwrap();
                let peer_code = code_only(&peer);
                assert!(!peer_code.contains("acquisition::"));
                for private_name in [
                    "C2MemoryOwner",
                    "C2CompleteGeneration",
                    "acquire_complete_generation",
                    "C2AcquisitionError",
                ] {
                    assert!(!peer_code.contains(private_name));
                }
                if !path.ends_with("native_successor_semantic.rs") {
                    assert!(!peer_code.contains("mod acquisition;"));
                }
            }
        }
        peers.sort();
        assert_eq!(peers, EXPECTED_PEERS);
    }

    #[test]
    fn every_init_ast_field_mutation_rejects() {
        use surrealdb_core::sql::Table;

        let base = parse_init_query().unwrap();
        let mut query = base.clone();
        query.0 .0.pop();
        assert!(!init_ast_is_exact(&query));
        let mut query = base.clone();
        query.0 .0.push(query.0 .0[1].clone());
        assert!(!init_ast_is_exact(&query));
        let mut query = base.clone();
        query.0 .0[0] = base.0 .0.last().unwrap().clone();
        assert!(!init_ast_is_exact(&query));
        let mut query = base.clone();
        let last = query.0 .0.len() - 1;
        query.0 .0[last] = base.0 .0[0].clone();
        assert!(!init_ast_is_exact(&query));
        let mut query = base.clone();
        query.0 .0.swap(1, 2);
        assert!(!init_ast_is_exact(&query));

        let mut query = base.clone();
        first_insert_mut(&mut query).into = None;
        assert!(!init_ast_is_exact(&query));
        let mut query = base.clone();
        first_insert_mut(&mut query).into = Some(NativeValue::Table(Table::from("wrong")));
        assert!(!init_ast_is_exact(&query));
        let mut query = base.clone();
        first_insert_mut(&mut query).data = Data::EmptyExpression;
        assert!(!init_ast_is_exact(&query));
        let mut query = base.clone();
        first_insert_mut(&mut query).ignore = true;
        assert!(!init_ast_is_exact(&query));
        let mut query = base.clone();
        first_insert_mut(&mut query).update = Some(Data::EmptyExpression);
        assert!(!init_ast_is_exact(&query));
        let mut query = base.clone();
        first_insert_mut(&mut query).output = Some(Output::Null);
        assert!(!init_ast_is_exact(&query));
        let mut query = base.clone();
        first_insert_mut(&mut query).timeout = Some(Default::default());
        assert!(!init_ast_is_exact(&query));
        let mut query = base.clone();
        first_insert_mut(&mut query).parallel = true;
        assert!(!init_ast_is_exact(&query));
        let mut query = base.clone();
        first_insert_mut(&mut query).relation = true;
        assert!(!init_ast_is_exact(&query));
        let mut query = base.clone();
        first_insert_mut(&mut query).version = Some(Default::default());
        assert!(!init_ast_is_exact(&query));

        let mut query = base.clone();
        let Statement::Insert(insert) = &mut query.0 .0[2] else {
            unreachable!();
        };
        insert.data = Data::SingleExpression(NativeValue::Param("wrong".into()));
        assert!(!init_ast_is_exact(&query));

        let prepared = prepare_generation(fixture_generation(Fixture::A)).unwrap();
        assert!(variables_are_exact(&prepared.variables));
        let mut missing = prepared.variables.clone();
        missing.remove(TableKind::MemoryItem.tag());
        assert!(!variables_are_exact(&missing));
        let mut extra = prepared.variables.clone();
        extra.insert("extra".to_string(), NativeValue::Null);
        assert!(!variables_are_exact(&extra));
        let mut wrong = prepared.variables;
        let value = wrong.remove(TableKind::MemoryItem.tag()).unwrap();
        wrong.insert("wrong".to_string(), value);
        assert!(!variables_are_exact(&wrong));
    }

    #[test]
    fn every_read_ast_field_mutation_rejects() {
        use surrealdb_core::sql::{Function, Table, With};

        let base = parse_read_query().unwrap();
        let mut query = base.clone();
        query.0 .0.push(query.0 .0[0].clone());
        assert!(!read_ast_is_exact(&query));
        let mut query = base.clone();
        let Statement::Output(output) = &mut query.0 .0[0] else {
            unreachable!();
        };
        output.fetch = Some(Default::default());
        assert!(!read_ast_is_exact(&query));
        let mut query = base.clone();
        let Statement::Output(output) = &mut query.0 .0[0] else {
            unreachable!();
        };
        output.what = NativeValue::Null;
        assert!(!read_ast_is_exact(&query));
        let mut query = base.clone();
        let Statement::Output(output) = &mut query.0 .0[0] else {
            unreachable!();
        };
        let NativeValue::Object(object) = &mut output.what else {
            unreachable!();
        };
        object.insert("extra".to_string(), NativeValue::Null);
        assert!(!read_ast_is_exact(&query));
        let mut query = base.clone();
        let Statement::Output(output) = &mut query.0 .0[0] else {
            unreachable!();
        };
        let NativeValue::Object(object) = &mut output.what else {
            unreachable!();
        };
        object.remove(TableKind::MemoryItem.tag());
        assert!(!read_ast_is_exact(&query));
        let mut query = base.clone();
        let Statement::Output(output) = &mut query.0 .0[0] else {
            unreachable!();
        };
        let NativeValue::Object(object) = &mut output.what else {
            unreachable!();
        };
        let member = object.remove(TableKind::MemoryItem.tag()).unwrap();
        object.insert("wrong".to_string(), member);
        assert!(!read_ast_is_exact(&query));

        for member in [
            NativeValue::Null,
            NativeValue::Param("memory_item".into()),
            NativeValue::from(Function::Normal(
                "string::lowercase".to_string(),
                Vec::new(),
            )),
            NativeValue::from(Subquery::Value(NativeValue::Null)),
        ] {
            let mut query = base.clone();
            let Statement::Output(output) = &mut query.0 .0[0] else {
                unreachable!();
            };
            let NativeValue::Object(object) = &mut output.what else {
                unreachable!();
            };
            object.insert(TableKind::MemoryItem.tag().to_string(), member);
            assert!(!read_ast_is_exact(&query));
        }

        let mut query = base.clone();
        first_select_mut(&mut query).expr.0.push(Field::All);
        assert!(!read_ast_is_exact(&query));
        let non_all: Query =
            surrealdb_core::syn::parse("SELECT id FROM memory_item LIMIT 65;").unwrap();
        let Statement::Select(non_all) = &non_all.0 .0[0] else {
            unreachable!();
        };
        let mut query = base.clone();
        first_select_mut(&mut query).expr.0[0] = non_all.expr.0[0].clone();
        assert!(!read_ast_is_exact(&query));
        let mut query = base.clone();
        first_select_mut(&mut query).expr.1 = true;
        assert!(!read_ast_is_exact(&query));
        let mut query = base.clone();
        first_select_mut(&mut query).omit = Some(Default::default());
        assert!(!read_ast_is_exact(&query));
        let mut query = base.clone();
        first_select_mut(&mut query).only = true;
        assert!(!read_ast_is_exact(&query));
        let mut query = base.clone();
        first_select_mut(&mut query).what.0.clear();
        assert!(!read_ast_is_exact(&query));
        let mut query = base.clone();
        first_select_mut(&mut query).what.0[0] = NativeValue::Table(Table::from("wrong"));
        assert!(!read_ast_is_exact(&query));
        let mut query = base.clone();
        first_select_mut(&mut query).with = Some(With::NoIndex);
        assert!(!read_ast_is_exact(&query));
        let mut query = base.clone();
        first_select_mut(&mut query).cond = Some(Default::default());
        assert!(!read_ast_is_exact(&query));
        let mut query = base.clone();
        first_select_mut(&mut query).split = Some(Default::default());
        assert!(!read_ast_is_exact(&query));
        let mut query = base.clone();
        first_select_mut(&mut query).group = Some(Default::default());
        assert!(!read_ast_is_exact(&query));
        let mut query = base.clone();
        let ordering_source =
            surrealdb_core::syn::parse("SELECT * FROM memory_item ORDER BY id LIMIT 65;").unwrap();
        let Statement::Select(ordering_source) = &ordering_source.0 .0[0] else {
            unreachable!();
        };
        first_select_mut(&mut query).order = ordering_source.order.clone();
        assert!(!read_ast_is_exact(&query));
        let mut query = base.clone();
        first_select_mut(&mut query).limit = None;
        assert!(!read_ast_is_exact(&query));
        let mut query = base.clone();
        first_select_mut(&mut query).limit.as_mut().unwrap().0 =
            NativeValue::Number(NativeNumber::Int(64));
        assert!(!read_ast_is_exact(&query));
        let mut query = base.clone();
        first_select_mut(&mut query).limit.as_mut().unwrap().0 =
            NativeValue::Strand(test_strand("65"));
        assert!(!read_ast_is_exact(&query));
        let mut query = base.clone();
        first_select_mut(&mut query).start = Some(Default::default());
        assert!(!read_ast_is_exact(&query));
        let mut query = base.clone();
        first_select_mut(&mut query).fetch = Some(Default::default());
        assert!(!read_ast_is_exact(&query));
        let mut query = base.clone();
        first_select_mut(&mut query).version = Some(Default::default());
        assert!(!read_ast_is_exact(&query));
        let mut query = base.clone();
        first_select_mut(&mut query).timeout = Some(Default::default());
        assert!(!read_ast_is_exact(&query));
        let mut query = base.clone();
        first_select_mut(&mut query).parallel = true;
        assert!(!read_ast_is_exact(&query));
        let mut query = base.clone();
        first_select_mut(&mut query).explain = Some(Default::default());
        assert!(!read_ast_is_exact(&query));
        let mut query = base.clone();
        first_select_mut(&mut query).tempfiles = true;
        assert!(!read_ast_is_exact(&query));
    }

    #[test]
    fn every_replace_delete_ast_field_mutation_rejects() {
        use surrealdb_core::sql::{Table, With};

        let base = parse_replace_query().unwrap();
        let mut query = base.clone();
        query.0 .0.pop();
        assert!(!replace_ast_is_exact(&query));
        let mut query = base.clone();
        query.0 .0.push(query.0 .0[1].clone());
        assert!(!replace_ast_is_exact(&query));
        let mut query = base.clone();
        query.0 .0[0] = base.0 .0.last().unwrap().clone();
        assert!(!replace_ast_is_exact(&query));
        let mut query = base.clone();
        let last = query.0 .0.len() - 1;
        query.0 .0[last] = base.0 .0[0].clone();
        assert!(!replace_ast_is_exact(&query));
        let mut query = base.clone();
        query.0 .0.swap(1, 2);
        assert!(!replace_ast_is_exact(&query));
        let mut query = base.clone();
        first_delete_mut(&mut query).only = true;
        assert!(!replace_ast_is_exact(&query));
        let mut query = base.clone();
        first_delete_mut(&mut query).what.0.clear();
        assert!(!replace_ast_is_exact(&query));
        let mut query = base.clone();
        first_delete_mut(&mut query).what.0[0] = NativeValue::Table(Table::from("wrong"));
        assert!(!replace_ast_is_exact(&query));
        let mut query = base.clone();
        first_delete_mut(&mut query).with = Some(With::NoIndex);
        assert!(!replace_ast_is_exact(&query));
        let mut query = base.clone();
        first_delete_mut(&mut query).cond = Some(Default::default());
        assert!(!replace_ast_is_exact(&query));
        let mut query = base.clone();
        first_delete_mut(&mut query).output = Some(Output::Null);
        assert!(!replace_ast_is_exact(&query));
        let mut query = base.clone();
        first_delete_mut(&mut query).timeout = Some(Default::default());
        assert!(!replace_ast_is_exact(&query));
        let mut query = base.clone();
        first_delete_mut(&mut query).parallel = true;
        assert!(!replace_ast_is_exact(&query));
        let mut query = base.clone();
        first_delete_mut(&mut query).explain = Some(Default::default());
        assert!(!replace_ast_is_exact(&query));

        let mut query = base.clone();
        let Statement::Insert(insert) = &mut query.0 .0[10] else {
            unreachable!();
        };
        insert.into = Some(NativeValue::Table(Table::from("wrong")));
        assert!(!replace_ast_is_exact(&query));
        let mut query = base.clone();
        let Statement::Insert(insert) = &mut query.0 .0[10] else {
            unreachable!();
        };
        insert.data = Data::SingleExpression(NativeValue::Param("wrong".into()));
        assert!(!replace_ast_is_exact(&query));
        let mut query = base.clone();
        let Statement::Insert(insert) = &mut query.0 .0[10] else {
            unreachable!();
        };
        insert.output = Some(Output::Null);
        assert!(!replace_ast_is_exact(&query));
    }

    #[test]
    fn write_outcomes_require_exact_empty_arrays() {
        assert!(exact_write_value(&NativeValue::Array(NativeArray::from(
            Vec::<NativeValue>::new()
        ))));
        assert!(!exact_write_value(&NativeValue::None));
        assert!(!exact_write_value(&NativeValue::Null));
        assert!(!exact_write_value(&NativeValue::Array(NativeArray::from(
            vec![NativeValue::Null]
        ))));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn real_write_outcomes_cover_nine_eighteen_zero_extra_failed_and_malformed() {
        let datastore = Datastore::new("memory")
            .await
            .unwrap()
            .with_capabilities(Capabilities::none());
        let session = Session::owner().with_ns("engram").with_db("main");
        let mut prepared = prepare_generation(fixture_generation(Fixture::A)).unwrap();
        let responses = datastore
            .process(
                parse_init_query().unwrap(),
                &session,
                Some(std::mem::take(&mut prepared.variables)),
            )
            .await
            .unwrap();
        assert!(exact_write_outcomes(responses, 9));

        let mut replacement = prepare_generation(fixture_generation(Fixture::B)).unwrap();
        let responses = datastore
            .process(
                parse_replace_query().unwrap(),
                &session,
                Some(std::mem::take(&mut replacement.variables)),
            )
            .await
            .unwrap();
        assert!(exact_write_outcomes(responses, 18));

        let empty = || NativeValue::Array(NativeArray::from(Vec::<NativeValue>::new()));
        assert!(!exact_write_results(vec![], 1));
        assert!(exact_write_results(vec![Ok(empty())], 1));
        assert!(!exact_write_results(vec![Ok(empty()), Ok(empty())], 1));
        assert!(!exact_write_results(vec![Err(())], 1));
        for malformed in every_nonempty_write_value_shape() {
            assert!(!exact_write_results(vec![Ok(malformed)], 1));
        }
    }

    #[test]
    fn read_outcomes_reject_zero_extra_failed_and_malformed_shapes() {
        assert_eq!(
            exact_read_results(Vec::new()),
            Err(C2AcquisitionError::InvalidResponse)
        );
        assert_eq!(
            exact_read_results(vec![Ok(NativeValue::Null), Ok(NativeValue::Null)]),
            Err(C2AcquisitionError::InvalidResponse)
        );
        assert_eq!(
            exact_read_results(vec![Err(())]),
            Err(C2AcquisitionError::EngineFailure)
        );
        assert_eq!(
            exact_read_results(vec![Err(()), Ok(NativeValue::Null)]),
            Err(C2AcquisitionError::EngineFailure)
        );
        assert_eq!(
            exact_read_results(vec![Ok(NativeValue::Null), Err(())]),
            Err(C2AcquisitionError::EngineFailure)
        );

        let (target, accounting, valid) = native_outer_from_generation(Fixture::A);
        assert!(validate_collected_value(valid, target, accounting).is_ok());
        for malformed in [
            NativeValue::None,
            NativeValue::Null,
            NativeValue::Array(NativeArray::from(Vec::<NativeValue>::new())),
            NativeValue::Object(NativeObject::from(BTreeMap::<String, NativeValue>::new())),
            NativeValue::Strand(test_strand("not-a-table-object")),
        ] {
            let value = exact_read_results(vec![Ok(malformed)]).unwrap();
            let (target, accounting, _) = native_outer_from_generation(Fixture::A);
            assert!(matches!(
                validate_collected_value(value, target, accounting),
                Err(C2AcquisitionError::InvalidResponse)
            ));
        }
    }

    #[test]
    fn table_caps_and_combined_overflow_masks_are_exact_for_all_nine_tables() {
        for table in TABLE_KINDS {
            let mut tables = NativeTables {
                tables: TABLE_KINDS
                    .into_iter()
                    .map(|kind| {
                        let length = if kind == table { kind.row_cap() } else { 0 };
                        (kind, NativeArray::from(vec![NativeValue::Null; length]))
                    })
                    .collect(),
            };
            assert_eq!(
                native_overflow_mask(&tables),
                0,
                "{} exact cap",
                table.tag()
            );
            tables
                .tables
                .iter_mut()
                .find(|(kind, _)| *kind == table)
                .unwrap()
                .1
                 .0
                .push(NativeValue::Null);
            assert_eq!(
                native_overflow_mask(&tables),
                table.overflow_bit(),
                "{} cap plus one",
                table.tag()
            );
        }

        let every_overflow = NativeTables {
            tables: TABLE_KINDS
                .into_iter()
                .map(|table| {
                    (
                        table,
                        NativeArray::from(vec![NativeValue::Null; table.row_cap() + 1]),
                    )
                })
                .collect(),
        };
        assert_eq!(native_overflow_mask(&every_overflow), 0x01ff);

        for table in TABLE_KINDS
            .into_iter()
            .filter(|table| *table != TableKind::MemoryForgetReceipt)
        {
            let mut generation = fixture_generation(Fixture::A);
            let cap = table.row_cap();
            match table {
                TableKind::MemoryItem => {
                    generation.memory_item = vec![generation.memory_item[0].clone(); cap]
                }
                TableKind::CorrectionProposal => {
                    generation.correction_proposal =
                        vec![generation.correction_proposal[0].clone(); cap]
                }
                TableKind::WorkProject => {
                    generation.work_project = vec![generation.work_project[0].clone(); cap]
                }
                TableKind::WorkTask => {
                    generation.work_task = vec![generation.work_task[0].clone(); cap]
                }
                TableKind::GitRepository => {
                    generation.git_repository = vec![generation.git_repository[0].clone(); cap]
                }
                TableKind::LocalCheckout => {
                    generation.local_checkout = vec![generation.local_checkout[0].clone(); cap]
                }
                TableKind::MonorepoComponent => {
                    generation.monorepo_component =
                        vec![generation.monorepo_component[0].clone(); cap]
                }
                TableKind::ProjectRepositoryLink => {
                    generation.project_repository_link =
                        vec![generation.project_repository_link[0].clone(); cap]
                }
                TableKind::MemoryForgetReceipt => unreachable!(),
            }
            assert_eq!(generation_table_overflow_mask(&generation), 0);
            match table {
                TableKind::MemoryItem => generation
                    .memory_item
                    .push(generation.memory_item[0].clone()),
                TableKind::CorrectionProposal => generation
                    .correction_proposal
                    .push(generation.correction_proposal[0].clone()),
                TableKind::WorkProject => generation
                    .work_project
                    .push(generation.work_project[0].clone()),
                TableKind::WorkTask => generation.work_task.push(generation.work_task[0].clone()),
                TableKind::GitRepository => generation
                    .git_repository
                    .push(generation.git_repository[0].clone()),
                TableKind::LocalCheckout => generation
                    .local_checkout
                    .push(generation.local_checkout[0].clone()),
                TableKind::MonorepoComponent => generation
                    .monorepo_component
                    .push(generation.monorepo_component[0].clone()),
                TableKind::ProjectRepositoryLink => generation
                    .project_repository_link
                    .push(generation.project_repository_link[0].clone()),
                TableKind::MemoryForgetReceipt => unreachable!(),
            }
            let expected = table.overflow_bit();
            assert_eq!(generation_table_overflow_mask(&generation), expected);
            assert!(matches!(
                borrowed_generation_upper_bound(&generation),
                Err(C2AcquisitionError::LimitExceeded {
                    table_overflow_mask: Some(mask)
                }) if mask == expected
            ));
        }
    }

    #[test]
    fn duplicate_ids_reject_before_json_and_remain_defended_after_projection() {
        let duplicate_generation = |table: TableKind| {
            let mut generation = fixture_generation(Fixture::A);
            match table {
                TableKind::MemoryItem => generation
                    .memory_item
                    .push(generation.memory_item[0].clone()),
                TableKind::CorrectionProposal => generation
                    .correction_proposal
                    .push(generation.correction_proposal[0].clone()),
                TableKind::WorkProject => generation
                    .work_project
                    .push(generation.work_project[0].clone()),
                TableKind::WorkTask => generation.work_task.push(generation.work_task[0].clone()),
                TableKind::GitRepository => generation
                    .git_repository
                    .push(generation.git_repository[0].clone()),
                TableKind::LocalCheckout => generation
                    .local_checkout
                    .push(generation.local_checkout[0].clone()),
                TableKind::MonorepoComponent => generation
                    .monorepo_component
                    .push(generation.monorepo_component[0].clone()),
                TableKind::ProjectRepositoryLink => generation
                    .project_repository_link
                    .push(generation.project_repository_link[0].clone()),
                TableKind::MemoryForgetReceipt => unreachable!(),
            }
            generation
        };
        for table in TABLE_KINDS
            .into_iter()
            .filter(|table| *table != TableKind::MemoryForgetReceipt)
        {
            let generation = duplicate_generation(table);
            borrowed_generation_upper_bound(&generation).unwrap();
            assert_eq!(
                validate_typed_logical_uniqueness(&generation),
                Err(C2AcquisitionError::InvalidGeneration)
            );

            let raw = json_generation(duplicate_generation(table)).unwrap();
            let boundary = validate_limit_and_secret_boundary(&raw, &[]).unwrap();
            assert!(boundary.accounting.bytes > 0);
            assert_eq!(
                validate_unique_raw_ids(&raw),
                Err(C2AcquisitionError::InvalidGeneration)
            );
            assert!(matches!(
                prepare_generation(duplicate_generation(table)),
                Err(C2AcquisitionError::InvalidGeneration)
            ));
        }
    }

    #[test]
    fn typed_logical_projection_collisions_reject_before_native_ingress() {
        use engram_core::id::Id;

        let assert_collision = |generation: C2CompleteGeneration| {
            assert_eq!(
                validate_typed_logical_uniqueness(&generation),
                Err(C2AcquisitionError::InvalidGeneration)
            );
            assert!(matches!(
                prepare_generation(generation),
                Err(C2AcquisitionError::InvalidGeneration)
            ));
        };

        let mut generation = fixture_generation(Fixture::A);
        let mut project = generation.work_project[0].clone();
        project.id = Id::parse(&variant_id(Fixture::A, 100)).unwrap();
        project.name = "ENGRAM".to_string();
        generation.work_project.push(project);
        assert_collision(generation);

        for (offset, selector) in [
            (101, "task A"),
            (102, "TASK A"),
            (103, "ENG-42"),
            (104, "eng-42"),
        ] {
            let mut generation = fixture_generation(Fixture::A);
            let mut task = generation.work_task[0].clone();
            task.id = Id::parse(&variant_id(Fixture::A, offset)).unwrap();
            task.name = selector.to_string();
            task.jira_key = None;
            generation.work_task.push(task);
            assert_collision(generation);
        }

        let mut generation = fixture_generation(Fixture::A);
        let mut repository = generation.git_repository[0].clone();
        repository.id = Id::parse(&variant_id(Fixture::A, 105)).unwrap();
        repository.name = "normalized-remote-collision".to_string();
        repository.remote_url = Some("git@github.com:ymeiri/engram.git".to_string());
        generation.git_repository.push(repository);
        assert_collision(generation);

        let mut generation = fixture_generation(Fixture::A);
        let mut checkout = generation.local_checkout[0].clone();
        checkout.id = Id::parse(&variant_id(Fixture::A, 106)).unwrap();
        generation.local_checkout.push(checkout);
        assert_collision(generation);

        let mut generation = fixture_generation(Fixture::A);
        let mut component = generation.monorepo_component[0].clone();
        component.id = Id::parse(&variant_id(Fixture::A, 107)).unwrap();
        generation.monorepo_component.push(component);
        assert_collision(generation);

        let mut generation = fixture_generation(Fixture::A);
        let mut proposal = generation.correction_proposal[0].clone();
        proposal.id = Id::parse(&variant_id(Fixture::A, 108)).unwrap();
        generation.correction_proposal.push(proposal);
        assert_collision(generation);

        let mut generation = fixture_generation(Fixture::A);
        let mut proposal = generation.correction_proposal[0].clone();
        proposal.id = Id::parse(&variant_id(Fixture::A, 109)).unwrap();
        proposal.replacement_id = Id::parse(&variant_id(Fixture::A, 110)).unwrap();
        generation.correction_proposal.push(proposal);
        assert_collision(generation);

        let mut generation = fixture_generation(Fixture::A);
        let mut proposal = generation.correction_proposal[0].clone();
        proposal.id = Id::parse(&variant_id(Fixture::A, 111)).unwrap();
        proposal.obsolete_id = Id::parse(&variant_id(Fixture::A, 112)).unwrap();
        generation.correction_proposal.push(proposal);
        assert_collision(generation);

        let mut generation = fixture_generation(Fixture::A);
        let mut proposal = generation.correction_proposal[0].clone();
        proposal.id = Id::parse(&variant_id(Fixture::A, 113)).unwrap();
        proposal.obsolete_id = generation.correction_proposal[0].replacement_id;
        proposal.replacement_id = Id::parse(&variant_id(Fixture::A, 114)).unwrap();
        generation.correction_proposal.push(proposal);
        assert_collision(generation);

        let mut generation = fixture_generation(Fixture::A);
        let mut link = generation.project_repository_link[0].clone();
        link.id = Id::parse(&variant_id(Fixture::A, 117)).unwrap();
        link.project_id = None;
        link.role = ProjectRepositoryRole::Dependency;
        generation.project_repository_link.push(link);
        assert_collision(generation);

        let mut disjoint = fixture_generation(Fixture::A);
        let mut project = disjoint.work_project[0].clone();
        project.id = Id::parse(&variant_id(Fixture::A, 115)).unwrap();
        project.name = "disjoint".to_string();
        disjoint.work_project.push(project);
        let mut task = disjoint.work_task[0].clone();
        task.id = Id::parse(&variant_id(Fixture::A, 116)).unwrap();
        task.project_id = Id::parse(&variant_id(Fixture::A, 115)).unwrap();
        disjoint.work_task.push(task);
        assert!(validate_typed_logical_uniqueness(&disjoint).is_ok());
    }

    fn nested_native_object(levels: usize) -> NativeValue {
        assert!(
            levels <= super::super::MAX_DEPTH as usize + 1,
            "nested native injector exceeded its fixed bound"
        );
        let mut value = NativeValue::Null;
        for _ in 0..levels {
            value = NativeValue::Object(NativeObject::from(BTreeMap::from([(
                "x".to_string(),
                value,
            )])));
        }
        value
    }

    fn every_nonempty_write_value_shape() -> Vec<NativeValue> {
        use surrealdb_core::sql::{
            Block, Constant, Duration as NativeDuration, Expression, Function, Future, Idiom, Mock,
            Subquery, Table,
        };

        vec![
            NativeValue::None,
            NativeValue::Null,
            NativeValue::Bool(true),
            NativeValue::Number(NativeNumber::Int(7)),
            NativeValue::Number(NativeNumber::Float(1.25)),
            surrealdb_core::syn::value("1.0dec").unwrap(),
            NativeValue::Strand(test_strand("text")),
            NativeValue::from(NativeDuration::from_secs(1)),
            surrealdb_core::syn::value("d'2026-09-06T00:00:00Z'").unwrap(),
            surrealdb_core::syn::value("u'01890f5e-7b00-7000-8000-000000000099'").unwrap(),
            NativeValue::Array(NativeArray::from(vec![NativeValue::Null])),
            NativeValue::Object(NativeObject::from(BTreeMap::<String, NativeValue>::new())),
            surrealdb_core::syn::value("(1.0, 2.0)").unwrap(),
            NativeValue::from(surrealdb_core::sql::Bytes::from(vec![1_u8, 2, 3])),
            NativeValue::Thing(Thing::from((
                TableKind::MemoryItem.tag().to_string(),
                PROJECT_ID.to_string(),
            ))),
            NativeValue::Param("parameter".into()),
            NativeValue::Idiom(Idiom::default()),
            NativeValue::Table(Table::from("memory_item")),
            NativeValue::Mock(Mock::Count("memory_item".to_string(), 1)),
            surrealdb_core::syn::value("/bounded/").unwrap(),
            surrealdb_core::syn::value("<string> 7").unwrap(),
            NativeValue::from(Block::default()),
            surrealdb_core::syn::value("1..2").unwrap(),
            serde_json::from_value(json!({
                "Edges": {
                    "dir": "Out",
                    "from": {"tb": "source", "id": {"String": PROJECT_ID}},
                    "what": [{"Table": "edge"}]
                }
            }))
            .unwrap(),
            NativeValue::from(Future::from(NativeValue::Null)),
            NativeValue::Constant(Constant::MathPi),
            NativeValue::from(Function::Normal("string::lowercase".to_string(), vec![])),
            NativeValue::from(Subquery::Value(NativeValue::Null)),
            NativeValue::from(Expression::default()),
            NativeValue::Query(Query::default()),
            NativeValue::Model(Box::default()),
            surrealdb_core::syn::value("|$value| $value").unwrap(),
            serde_json::from_value(json!({"Refs": []})).unwrap(),
        ]
    }

    #[test]
    fn exact_and_plus_one_native_limits_and_checked_accounting_are_enforced() {
        let mut meter = Meter::new();
        let mut row = None;
        let mut status = DryNativeStatus::default();
        dry_native_value(
            &NativeValue::Strand(test_strand(&"x".repeat(MAX_ORDINARY_SCALAR_BYTES))),
            0,
            SchemaPath::OrdinaryScalar,
            TableKind::MemoryItem,
            None,
            &mut meter,
            &mut row,
            &mut status,
        )
        .unwrap();
        assert!(!status.invalid_response);
        assert!(matches!(
            dry_native_value(
                &NativeValue::Strand(test_strand(&"x".repeat(MAX_ORDINARY_SCALAR_BYTES + 1))),
                0,
                SchemaPath::OrdinaryScalar,
                TableKind::MemoryItem,
                None,
                &mut Meter::new(),
                &mut None,
                &mut DryNativeStatus::default(),
            ),
            Err(C2AcquisitionError::LimitExceeded { .. })
        ));
        for (schema, cap) in [
            (SchemaPath::NameScalar, MAX_NAME_SCALAR_BYTES),
            (SchemaPath::LargeScalar, MAX_LARGE_SCALAR_BYTES),
        ] {
            dry_native_value(
                &NativeValue::Strand(test_strand(&"x".repeat(cap))),
                0,
                schema,
                TableKind::MemoryItem,
                None,
                &mut Meter::new(),
                &mut None,
                &mut DryNativeStatus::default(),
            )
            .unwrap();
            assert!(matches!(
                dry_native_value(
                    &NativeValue::Strand(test_strand(&"x".repeat(cap + 1))),
                    0,
                    schema,
                    TableKind::MemoryItem,
                    None,
                    &mut Meter::new(),
                    &mut None,
                    &mut DryNativeStatus::default(),
                ),
                Err(C2AcquisitionError::LimitExceeded { .. })
            ));
        }

        let exact_vector = NativeValue::Array(NativeArray::from(vec![
            NativeValue::Null;
            MAX_VECTOR_ELEMENTS
        ]));
        dry_native_value(
            &exact_vector,
            0,
            SchemaPath::Unknown,
            TableKind::MemoryItem,
            None,
            &mut Meter::new(),
            &mut None,
            &mut DryNativeStatus::default(),
        )
        .unwrap();
        let plus_vector = NativeValue::Array(NativeArray::from(vec![
            NativeValue::Null;
            MAX_VECTOR_ELEMENTS + 1
        ]));
        assert!(matches!(
            dry_native_value(
                &plus_vector,
                0,
                SchemaPath::Unknown,
                TableKind::MemoryItem,
                None,
                &mut Meter::new(),
                &mut None,
                &mut DryNativeStatus::default(),
            ),
            Err(C2AcquisitionError::LimitExceeded { .. })
        ));

        let exact_object = NativeValue::Object(NativeObject::from(
            (0..MAX_OBJECT_KEYS)
                .map(|index| (format!("k{index:02}"), NativeValue::Null))
                .collect::<BTreeMap<_, _>>(),
        ));
        dry_native_value(
            &exact_object,
            0,
            SchemaPath::Unknown,
            TableKind::MemoryItem,
            None,
            &mut Meter::new(),
            &mut None,
            &mut DryNativeStatus::default(),
        )
        .unwrap();
        let plus_object = NativeValue::Object(NativeObject::from(
            (0..=MAX_OBJECT_KEYS)
                .map(|index| (format!("k{index:02}"), NativeValue::Null))
                .collect::<BTreeMap<_, _>>(),
        ));
        assert!(matches!(
            dry_native_value(
                &plus_object,
                0,
                SchemaPath::Unknown,
                TableKind::MemoryItem,
                None,
                &mut Meter::new(),
                &mut None,
                &mut DryNativeStatus::default(),
            ),
            Err(C2AcquisitionError::LimitExceeded { .. })
        ));

        dry_native_value(
            &nested_native_object(super::super::MAX_DEPTH as usize),
            0,
            SchemaPath::Unknown,
            TableKind::MemoryItem,
            None,
            &mut Meter::new(),
            &mut None,
            &mut DryNativeStatus::default(),
        )
        .unwrap();
        assert!(matches!(
            dry_native_value(
                &nested_native_object(super::super::MAX_DEPTH as usize + 1),
                0,
                SchemaPath::Unknown,
                TableKind::MemoryItem,
                None,
                &mut Meter::new(),
                &mut None,
                &mut DryNativeStatus::default(),
            ),
            Err(C2AcquisitionError::LimitExceeded { .. })
        ));

        let mut row = RowBytes::new();
        row.add(super::super::MAX_ROW_BYTES).unwrap();
        assert!(matches!(
            row.add(1),
            Err(C2SemanticError::LimitExceeded { .. })
        ));
        let mut byte_meter = Meter {
            nodes: 0,
            bytes: super::super::MAX_RAW_BYTES - 1,
            maximum_depth: 0,
        };
        byte_meter.add_bytes(1).unwrap();
        assert!(matches!(
            byte_meter.add_bytes(1),
            Err(C2SemanticError::LimitExceeded { .. })
        ));
        let mut node_meter = Meter {
            nodes: super::super::MAX_RAW_NODES - 1,
            bytes: 0,
            maximum_depth: 0,
        };
        node_meter.enter_node(0).unwrap();
        assert!(matches!(
            node_meter.enter_node(0),
            Err(C2SemanticError::LimitExceeded { .. })
        ));
        assert!(matches!(
            dry_len(u64::MAX, 1),
            Err(C2AcquisitionError::LimitExceeded { .. })
        ));
        let mut upper = BorrowedJsonUpperMeter {
            bytes: u64::MAX,
            secret: false,
            scan_secrets: false,
            exact: None,
        };
        assert!(matches!(
            upper.add(1),
            Err(C2AcquisitionError::LimitExceeded { .. })
        ));

        for schema in [
            SchemaPath::EvidenceVector,
            SchemaPath::SupersedesVector,
            SchemaPath::TagsVector,
        ] {
            let exact = NativeValue::Array(NativeArray::from(vec![
                NativeValue::Null;
                MAX_EVIDENCE_TAGS_SUPERSEDES
            ]));
            dry_native_value(
                &exact,
                0,
                schema,
                TableKind::MemoryItem,
                None,
                &mut Meter::new(),
                &mut None,
                &mut DryNativeStatus::default(),
            )
            .unwrap();
            let plus = NativeValue::Array(NativeArray::from(vec![
                NativeValue::Null;
                MAX_EVIDENCE_TAGS_SUPERSEDES + 1
            ]));
            assert!(matches!(
                dry_native_value(
                    &plus,
                    0,
                    schema,
                    TableKind::MemoryItem,
                    None,
                    &mut Meter::new(),
                    &mut None,
                    &mut DryNativeStatus::default(),
                ),
                Err(C2AcquisitionError::LimitExceeded { .. })
            ));
        }
        for schema in [
            SchemaPath::ProcedureCommands,
            SchemaPath::ProcedurePrerequisites,
            SchemaPath::ProcedureFailureSignatures,
        ] {
            let exact = NativeValue::Array(NativeArray::from(vec![
                NativeValue::Null;
                MAX_PROCEDURE_VECTOR
            ]));
            dry_native_value(
                &exact,
                0,
                schema,
                TableKind::MemoryItem,
                None,
                &mut Meter::new(),
                &mut None,
                &mut DryNativeStatus::default(),
            )
            .unwrap();
            let plus = NativeValue::Array(NativeArray::from(vec![
                NativeValue::Null;
                MAX_PROCEDURE_VECTOR + 1
            ]));
            assert!(matches!(
                dry_native_value(
                    &plus,
                    0,
                    schema,
                    TableKind::MemoryItem,
                    None,
                    &mut Meter::new(),
                    &mut None,
                    &mut DryNativeStatus::default(),
                ),
                Err(C2AcquisitionError::LimitExceeded { .. })
            ));
        }
        let exact_key_path = NativeValue::Array(NativeArray::from(vec![
            NativeValue::Null;
            MAX_PREREQUISITE_KEY_PATH
        ]));
        dry_native_value(
            &exact_key_path,
            0,
            SchemaPath::PrerequisiteKeyPath,
            TableKind::MemoryItem,
            None,
            &mut Meter::new(),
            &mut None,
            &mut DryNativeStatus::default(),
        )
        .unwrap();
        let plus_key_path = NativeValue::Array(NativeArray::from(vec![
            NativeValue::Null;
            MAX_PREREQUISITE_KEY_PATH
                + 1
        ]));
        assert!(matches!(
            dry_native_value(
                &plus_key_path,
                0,
                SchemaPath::PrerequisiteKeyPath,
                TableKind::MemoryItem,
                None,
                &mut Meter::new(),
                &mut None,
                &mut DryNativeStatus::default(),
            ),
            Err(C2AcquisitionError::LimitExceeded { .. })
        ));

        for key_length in [MAX_ORDINARY_SCALAR_BYTES, MAX_ORDINARY_SCALAR_BYTES + 1] {
            let value = NativeValue::Object(NativeObject::from(BTreeMap::from([(
                "k".repeat(key_length),
                NativeValue::Null,
            )])));
            let result = dry_native_value(
                &value,
                0,
                SchemaPath::Unknown,
                TableKind::MemoryItem,
                None,
                &mut Meter::new(),
                &mut None,
                &mut DryNativeStatus::default(),
            );
            assert_eq!(result.is_ok(), key_length == MAX_ORDINARY_SCALAR_BYTES);
        }
    }

    #[test]
    fn borrowed_upper_meter_covers_every_enum_control_escape_and_number_extreme() {
        use engram_core::id::Id;

        let exact_width_unicode = |cap: usize| {
            let mut value = "雪".repeat(cap / 3);
            value.push_str(&"a".repeat(cap % 3));
            assert_eq!(value.len(), cap);
            value
        };
        for (index, (value, cap, path)) in [
            (
                "\u{0}".repeat(MAX_NAME_SCALAR_BYTES),
                MAX_NAME_SCALAR_BYTES,
                SchemaPath::NameScalar,
            ),
            (
                exact_width_unicode(MAX_NAME_SCALAR_BYTES),
                MAX_NAME_SCALAR_BYTES,
                SchemaPath::NameScalar,
            ),
            (
                "\u{0}".repeat(MAX_ORDINARY_SCALAR_BYTES),
                MAX_ORDINARY_SCALAR_BYTES,
                SchemaPath::OrdinaryScalar,
            ),
            (
                exact_width_unicode(MAX_ORDINARY_SCALAR_BYTES),
                MAX_ORDINARY_SCALAR_BYTES,
                SchemaPath::OrdinaryScalar,
            ),
            (
                "\u{0}".repeat(MAX_LARGE_SCALAR_BYTES),
                MAX_LARGE_SCALAR_BYTES,
                SchemaPath::LargeScalar,
            ),
            (
                exact_width_unicode(MAX_LARGE_SCALAR_BYTES),
                MAX_LARGE_SCALAR_BYTES,
                SchemaPath::LargeScalar,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let mut probe = BorrowedJsonUpperMeter::new();
            assert!(
                probe.string(&value, cap).is_ok(),
                "scalar boundary matrix entry {index} was rejected"
            );
            let serialized = JsonValue::String(value.clone());
            assert!(probe.bytes >= serde_json::to_vec(&serialized).unwrap().len() as u64);
            let mut exact = BorrowedJsonUpperMeter::with_exact_accounting();
            exact.string(&value, cap).unwrap();
            let (actual, invalid, lower_bound) = exact.finish_exact_accounting().unwrap();
            assert!(!invalid);
            assert!(!lower_bound);
            let mut expected = Meter::new();
            super::super::walk_value(&serialized, 0, path, &mut expected, &mut None).unwrap();
            assert_eq!(actual, expected.finish());
        }
        let control = "\u{0}".repeat(MAX_NAME_SCALAR_BYTES);
        for number in [
            f64::MIN,
            f64::MAX,
            f64::MIN_POSITIVE,
            f64::EPSILON,
            -0.0,
            1.7976931348623155e308,
            5e-324,
        ] {
            let mut meter = BorrowedJsonUpperMeter::new();
            meter.number_f64(number).unwrap();
            assert!(meter.bytes >= serde_json::to_vec(&number).unwrap().len() as u64);

            let mut exact = BorrowedJsonUpperMeter::with_exact_accounting();
            exact.number_f64(number).unwrap();
            let (actual, invalid, lower_bound) = exact.finish_exact_accounting().unwrap();
            assert!(!invalid);
            assert!(!lower_bound);
            let mut expected = Meter::new();
            super::super::walk_value(
                &JsonValue::from(number),
                0,
                SchemaPath::Unknown,
                &mut expected,
                &mut None,
            )
            .unwrap();
            assert_eq!(actual, expected.finish());
        }
        for value in [i32::MIN, -1, 0, 1, i32::MAX] {
            let mut meter = BorrowedJsonUpperMeter::new();
            meter.number_i32(value).unwrap();
            assert!(meter.bytes >= serde_json::to_vec(&value).unwrap().len() as u64);

            let mut exact = BorrowedJsonUpperMeter::with_exact_accounting();
            exact.number_i32(value).unwrap();
            let (actual, invalid, lower_bound) = exact.finish_exact_accounting().unwrap();
            assert!(!invalid);
            assert!(!lower_bound);
            let mut expected = Meter::new();
            super::super::walk_value(
                &JsonValue::from(value),
                0,
                SchemaPath::Unknown,
                &mut expected,
                &mut None,
            )
            .unwrap();
            assert_eq!(actual, expected.finish());
        }
        for value in [0, 1, u32::MAX] {
            let mut meter = BorrowedJsonUpperMeter::new();
            meter.number_u32(value).unwrap();
            assert!(meter.bytes >= serde_json::to_vec(&value).unwrap().len() as u64);

            let mut exact = BorrowedJsonUpperMeter::with_exact_accounting();
            exact.number_u32(value).unwrap();
            let (actual, invalid, lower_bound) = exact.finish_exact_accounting().unwrap();
            assert!(!invalid);
            assert!(!lower_bound);
            let mut expected = Meter::new();
            super::super::walk_value(
                &JsonValue::from(value),
                0,
                SchemaPath::Unknown,
                &mut expected,
                &mut None,
            )
            .unwrap();
            assert_eq!(actual, expected.finish());
        }
        for value in [
            f32::MIN,
            f32::MAX,
            f32::MIN_POSITIVE,
            f32::EPSILON,
            -0.0,
            0.1,
            1.0,
            f32::NAN,
            f32::NEG_INFINITY,
            f32::INFINITY,
        ] {
            let mut meter = BorrowedJsonUpperMeter::new();
            meter.number_f32(value).unwrap();
            assert!(meter.bytes >= serde_json::to_vec(&value).unwrap().len() as u64);

            let mut exact = BorrowedJsonUpperMeter::with_exact_accounting();
            exact.number_f32(value).unwrap();
            let (actual, invalid, lower_bound) = exact.finish_exact_accounting().unwrap();
            assert_eq!(invalid, !value.is_finite());
            assert!(!lower_bound);
            let mut expected = Meter::new();
            super::super::walk_value(
                &JsonValue::from(value),
                0,
                SchemaPath::Unknown,
                &mut expected,
                &mut None,
            )
            .unwrap();
            assert_eq!(actual, expected.finish());
        }

        let memory_kinds = [
            MemoryKind::Preference,
            MemoryKind::Rule,
            MemoryKind::Decision,
            MemoryKind::Limitation,
            MemoryKind::ProjectFact,
            MemoryKind::RepositoryFact,
            MemoryKind::TaskFact,
            MemoryKind::UserFact,
            MemoryKind::SessionInsight,
            MemoryKind::Handoff,
            MemoryKind::Procedure,
            MemoryKind::Custom(control.clone()),
        ];
        for value in &memory_kinds {
            assert_serialized_upper(value, |meter| meter.memory_kind(value));
            assert_serialized_upper(&value.to_string(), |meter| meter.memory_kind_display(value));
        }
        for value in [
            MemoryStatus::Active,
            MemoryStatus::NeedsReview,
            MemoryStatus::Superseded,
            MemoryStatus::Archived,
            MemoryStatus::Rejected,
        ] {
            assert_serialized_upper(&value, |meter| meter.memory_status(&value));
        }
        for value in [
            ProjectStatus::Planning,
            ProjectStatus::Active,
            ProjectStatus::Completed,
            ProjectStatus::Archived,
        ] {
            assert_serialized_upper(&value, |meter| meter.project_status(&value));
        }
        for value in [
            TaskStatus::Todo,
            TaskStatus::InProgress,
            TaskStatus::Blocked,
            TaskStatus::Done,
        ] {
            assert_serialized_upper(&value, |meter| meter.task_status(&value));
        }
        for value in [
            TaskPriority::Low,
            TaskPriority::Medium,
            TaskPriority::High,
            TaskPriority::Critical,
        ] {
            assert_serialized_upper(&value, |meter| meter.task_priority(&value));
        }
        for value in [
            ProjectRepositoryRole::Primary,
            ProjectRepositoryRole::Dependency,
            ProjectRepositoryRole::Produces,
            ProjectRepositoryRole::Related,
        ] {
            assert_serialized_upper(&value, |meter| meter.link_role(&value));
        }
        let origins = [
            ClaimOrigin::UserStated,
            ClaimOrigin::UserCorrected,
            ClaimOrigin::AgentObserved,
            ClaimOrigin::AgentInferred,
            ClaimOrigin::ToolResult,
            ClaimOrigin::Imported,
            ClaimOrigin::Migrated,
            ClaimOrigin::GeneratedSummary,
            ClaimOrigin::Custom(control.clone()),
        ];
        for value in &origins {
            assert_serialized_upper(value, |meter| meter.claim_origin(value));
        }
        let harnesses = [
            Harness::ClaudeCode,
            Harness::Codex,
            Harness::ChatGpt,
            Harness::Cursor,
            Harness::Other(control.clone()),
        ];
        for value in &harnesses {
            assert_serialized_upper(value, |meter| meter.harness(value));
            assert_serialized_upper(&value.to_string(), |meter| meter.harness_display(value));
        }
        let evidence_kinds = [
            EvidenceKind::SessionEvent,
            EvidenceKind::ToolCall,
            EvidenceKind::File,
            EvidenceKind::GitCommit,
            EvidenceKind::Url,
            EvidenceKind::Document,
            EvidenceKind::Observation,
            EvidenceKind::ManualReview,
            EvidenceKind::Custom(control.clone()),
        ];
        for value in &evidence_kinds {
            assert_serialized_upper(value, |meter| meter.evidence_kind(value));
        }
        let providers = [
            RepositoryProvider::GitHub,
            RepositoryProvider::GitLab,
            RepositoryProvider::Bitbucket,
            RepositoryProvider::Unknown,
            RepositoryProvider::Other(control.clone()),
        ];
        for value in &providers {
            assert_serialized_upper(value, |meter| meter.repository_provider(value));
            assert_serialized_upper(&value.to_string(), |meter| {
                meter.repository_provider_display(value)
            });
        }

        let id = Id::parse(PROJECT_ID).unwrap();
        let scopes = [
            MemoryScope::Global,
            MemoryScope::User,
            MemoryScope::Project {
                project_id: Some(id),
                project_name: control.clone(),
            },
            MemoryScope::Project {
                project_id: None,
                project_name: "project-without-id".to_string(),
            },
            MemoryScope::Task {
                project_id: Some(id),
                project_name: Some(control.clone()),
                task_id: Some(id),
                task_name: control.clone(),
            },
            MemoryScope::Task {
                project_id: None,
                project_name: None,
                task_id: None,
                task_name: "task-without-ids".to_string(),
            },
            MemoryScope::Entity {
                entity_id: Some(id),
                entity_name: control.clone(),
            },
            MemoryScope::Entity {
                entity_id: None,
                entity_name: "entity-without-id".to_string(),
            },
            MemoryScope::Repository {
                repository_id: Some(id),
                remote_url: Some(control.clone()),
                local_path: Some(control.clone()),
            },
            MemoryScope::Repository {
                repository_id: None,
                remote_url: None,
                local_path: Some("/workspace/local-only".to_string()),
            },
            MemoryScope::Repository {
                repository_id: None,
                remote_url: None,
                local_path: None,
            },
            MemoryScope::Session { session_id: id },
            MemoryScope::Custom {
                name: control.clone(),
            },
        ];
        for value in &scopes {
            assert_serialized_upper(value, |meter| meter.memory_scope(value));
            let mut projection_meter = BorrowedJsonUpperMeter::new();
            projection_meter.scope_key_projection(value).unwrap();
            assert!(projection_meter.bytes > 0);

            let mut exact_projection = BorrowedJsonUpperMeter::with_exact_accounting();
            exact_projection.scope_key_projection(value).unwrap();
            let (actual, invalid, lower_bound) =
                exact_projection.finish_exact_accounting().unwrap();
            assert!(!invalid);
            assert!(!lower_bound);
            let mut expected = Meter::new();
            super::super::walk_value(
                &JsonValue::String(scope_key(value)),
                0,
                SchemaPath::Unknown,
                &mut expected,
                &mut None,
            )
            .unwrap();
            assert_eq!(actual, expected.finish());
        }

        for value in [
            OffsetDateTime::parse(
                "2026-09-06T12:34:56Z",
                &time::format_description::well_known::Rfc3339,
            )
            .unwrap(),
            OffsetDateTime::parse(
                "2026-09-06T12:34:56.1Z",
                &time::format_description::well_known::Rfc3339,
            )
            .unwrap(),
            OffsetDateTime::parse(
                "2026-09-06T12:34:56.12Z",
                &time::format_description::well_known::Rfc3339,
            )
            .unwrap(),
            OffsetDateTime::parse(
                "2026-09-06T12:34:56.123Z",
                &time::format_description::well_known::Rfc3339,
            )
            .unwrap(),
            OffsetDateTime::parse(
                "2026-09-06T12:34:56.1234Z",
                &time::format_description::well_known::Rfc3339,
            )
            .unwrap(),
            OffsetDateTime::parse(
                "2026-09-06T12:34:56.12345Z",
                &time::format_description::well_known::Rfc3339,
            )
            .unwrap(),
            OffsetDateTime::parse(
                "2026-09-06T12:34:56.123456Z",
                &time::format_description::well_known::Rfc3339,
            )
            .unwrap(),
            OffsetDateTime::parse(
                "2026-09-06T12:34:56.1234567Z",
                &time::format_description::well_known::Rfc3339,
            )
            .unwrap(),
            OffsetDateTime::parse(
                "2026-09-06T12:34:56.12345678Z",
                &time::format_description::well_known::Rfc3339,
            )
            .unwrap(),
            OffsetDateTime::parse(
                "2026-09-06T12:34:56.123456789Z",
                &time::format_description::well_known::Rfc3339,
            )
            .unwrap(),
            OffsetDateTime::parse(
                "2026-09-06T12:34:56.1200+05:30",
                &time::format_description::well_known::Rfc3339,
            )
            .unwrap(),
            OffsetDateTime::parse(
                "2026-09-06T12:34:56-11:45",
                &time::format_description::well_known::Rfc3339,
            )
            .unwrap(),
        ] {
            let expected_value = JsonValue::String(
                value
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            );
            let mut upper = BorrowedJsonUpperMeter::new();
            upper.timestamp(value).unwrap();
            assert!(upper.bytes >= serde_json::to_vec(&expected_value).unwrap().len() as u64);
            let mut exact = BorrowedJsonUpperMeter::with_exact_accounting();
            exact.timestamp(value).unwrap();
            let (actual, invalid, lower_bound) = exact.finish_exact_accounting().unwrap();
            assert!(!invalid);
            assert!(!lower_bound);
            let mut expected = Meter::new();
            super::super::walk_value(
                &expected_value,
                0,
                SchemaPath::Unknown,
                &mut expected,
                &mut None,
            )
            .unwrap();
            assert_eq!(actual, expected.finish());
        }

        let timestamp =
            OffsetDateTime::parse(TIMESTAMP, &time::format_description::well_known::Rfc3339)
                .unwrap();
        let base_generation = fixture_generation(Fixture::A);
        let model_none = base_generation.memory_item[0].writer.model.clone();
        let mut model_some = model_none.clone();
        model_some.version = Some("model-version".to_string());
        assert_serialized_upper(&model_none, |meter| meter.model(&model_none));
        assert_serialized_upper(&model_some, |meter| meter.model(&model_some));

        let writer_none = base_generation.memory_item[0].writer.clone();
        let mut writer_some = writer_none.clone();
        writer_some.harness_version = Some("harness-version".to_string());
        writer_some.model = model_some;
        writer_some.surface = Some("surface".to_string());
        writer_some.session_id = Some(id);
        assert_serialized_upper(&writer_none, |meter| meter.writer(&writer_none));
        assert_serialized_upper(&writer_some, |meter| meter.writer(&writer_some));

        let evidence_none = EvidenceRef {
            kind: EvidenceKind::File,
            target: "evidence-target".to_string(),
            summary: None,
            excerpt: None,
            observed_at: timestamp,
        };
        let mut evidence_some = evidence_none.clone();
        evidence_some.summary = Some("evidence summary".to_string());
        evidence_some.excerpt = Some("evidence excerpt".to_string());
        assert_serialized_upper(&evidence_none, |meter| meter.evidence(&evidence_none));
        assert_serialized_upper(&evidence_some, |meter| meter.evidence(&evidence_some));

        for archive in [
            ArchiveMetadata {
                reason: "archive reason".to_string(),
                archived_by: None,
                archived_at: timestamp,
            },
            ArchiveMetadata {
                reason: "archive reason".to_string(),
                archived_by: Some("archiver".to_string()),
                archived_at: timestamp,
            },
        ] {
            assert_serialized_upper(&archive, |meter| meter.archive(&archive));
        }

        let prerequisite_none = ProcedurePrerequisite {
            key: "cargo.version".to_string(),
            expected: "1.80".to_string(),
            source: None,
        };
        let prerequisite_some = ProcedurePrerequisite {
            key: "workspace.package.version".to_string(),
            expected: "1".to_string(),
            source: Some(ProcedurePrerequisiteSource::Toml {
                relative_path: "Cargo.toml".to_string(),
                key_path: vec![
                    "workspace".to_string(),
                    "package".to_string(),
                    "version".to_string(),
                ],
            }),
        };
        assert_serialized_upper(&prerequisite_none, |meter| {
            meter.prerequisite(&prerequisite_none)
        });
        assert_serialized_upper(&prerequisite_some, |meter| {
            meter.prerequisite(&prerequisite_some)
        });

        let verification_none = ProcedureVerification {
            command: "cargo test".to_string(),
            expected_exit_code: i32::MIN,
            expected_output_contains: "test result: ok".to_string(),
            evidence_path: None,
            evidence_sha256: None,
            verified_at: None,
        };
        let verification_some = ProcedureVerification {
            command: "cargo test".to_string(),
            expected_exit_code: i32::MAX,
            expected_output_contains: "test result: ok".to_string(),
            evidence_path: Some("evidence/test.txt".to_string()),
            evidence_sha256: Some("a".repeat(64)),
            verified_at: Some(timestamp),
        };
        assert_serialized_upper(&verification_none, |meter| {
            meter.verification(&verification_none)
        });
        assert_serialized_upper(&verification_some, |meter| {
            meter.verification(&verification_some)
        });

        let procedure_none = ProcedureCard {
            task: "reproduce".to_string(),
            prerequisites: vec![prerequisite_none.clone()],
            commands: vec!["cargo test".to_string()],
            failure_signatures: vec!["failure".to_string()],
            verification: verification_none,
            expires_at: None,
        };
        let procedure_some = ProcedureCard {
            task: "reproduce".to_string(),
            prerequisites: vec![prerequisite_none, prerequisite_some],
            commands: vec!["cargo test".to_string(), "cargo clippy".to_string()],
            failure_signatures: vec!["failure".to_string(), "timeout".to_string()],
            verification: verification_some,
            expires_at: Some(timestamp),
        };
        assert_serialized_upper(&procedure_none, |meter| meter.procedure(&procedure_none));
        assert_serialized_upper(&procedure_some, |meter| meter.procedure(&procedure_some));

        let mut rich_projection = fixture_generation(Fixture::A);
        rich_projection.work_project[0].created_at = OffsetDateTime::parse(
            "2026-09-06T12:34:56.1200+05:30",
            &time::format_description::well_known::Rfc3339,
        )
        .unwrap();
        rich_projection.work_project[0].updated_at = OffsetDateTime::parse(
            "2026-09-06T12:34:56.000000001Z",
            &time::format_description::well_known::Rfc3339,
        )
        .unwrap();
        rich_projection.memory_item[0].confidence = engram_core::memory::MemoryConfidence::new(0.1);
        rich_projection.memory_item[1].confidence = engram_core::memory::MemoryConfidence::new(1.0);
        rich_projection.target.task_name = Some("Unicode 雪\0escaped".to_string());
        rich_projection.git_repository[0].default_branch = None;
        rich_projection.local_checkout[0].current_branch = None;

        let mut branch_projection = fixture_generation(Fixture::A);
        branch_projection.target.task_id = Some(variant_id(Fixture::A, 4));
        branch_projection.target.task_name = Some("task A".to_string());
        let proposal_id = branch_projection.correction_proposal[0].id;
        branch_projection.memory_item[0].writer = writer_some;
        branch_projection.memory_item[0].evidence = vec![evidence_some, evidence_none.clone()];
        branch_projection.memory_item[0].supersedes = vec![id];
        branch_projection.memory_item[0].last_used_at = Some(timestamp);
        branch_projection.memory_item[0].review_after = Some(timestamp);
        branch_projection.memory_item[0].archive = Some(ArchiveMetadata {
            reason: "archive reason".to_string(),
            archived_by: Some("archiver".to_string()),
            archived_at: timestamp,
        });
        branch_projection.memory_item[0].procedure = Some(procedure_some);
        branch_projection.memory_item[0].correction_proposal_id = Some(proposal_id);
        branch_projection.memory_item[0].pending_correction_proposal_id = Some(proposal_id);
        branch_projection.memory_item[1].writer = writer_none;
        branch_projection.memory_item[1].evidence = vec![evidence_none];
        branch_projection.memory_item[1].correction_proposal_id = None;
        branch_projection.memory_item[1].pending_correction_proposal_id = None;
        branch_projection.correction_proposal[0].status = CorrectionProposalStatus::Applied;
        branch_projection.correction_proposal[0].applied_digest = Some("d".repeat(64));
        branch_projection.correction_proposal[0].applied_at = Some(timestamp);
        branch_projection.work_project[0].description = None;
        branch_projection.work_task[0].description = Some("task description".to_string());
        branch_projection.work_task[0].jira_key = None;
        branch_projection.work_task[0].blocked_by = vec![id];
        branch_projection.git_repository[0].remote_url = None;
        branch_projection.git_repository[0].default_branch = None;
        branch_projection.git_repository[0].description = None;
        branch_projection.local_checkout[0].repository_id = None;
        branch_projection.local_checkout[0].current_branch = None;
        branch_projection.local_checkout[0].head_sha = None;
        branch_projection.local_checkout[0].is_dirty = None;
        let mut source_path_only = branch_projection.monorepo_component[0].clone();
        source_path_only.id = Id::parse(&variant_id(Fixture::A, 8)).unwrap();
        source_path_only.name = "source-path-only".to_string();
        source_path_only.source_sha256 = None;
        let mut source_digest_only = branch_projection.monorepo_component[0].clone();
        source_digest_only.id = Id::parse(&variant_id(Fixture::A, 9)).unwrap();
        source_digest_only.name = "source-digest-only".to_string();
        source_digest_only.source_path = None;
        branch_projection.monorepo_component[0].kind = None;
        branch_projection.monorepo_component[0].description = None;
        branch_projection.monorepo_component[0].source_path = None;
        branch_projection.monorepo_component[0].source_sha256 = None;
        branch_projection
            .monorepo_component
            .extend([source_path_only, source_digest_only]);
        branch_projection.project_repository_link[0].project_id = None;

        for item in &branch_projection.memory_item {
            assert_serialized_upper(item, |meter| meter.memory_item(item));
        }
        for proposal in &branch_projection.correction_proposal {
            assert_serialized_upper(proposal, |meter| meter.proposal(proposal));
        }
        for repository in &branch_projection.git_repository {
            assert_serialized_upper(repository, |meter| meter.repository(repository));
        }
        for checkout in &branch_projection.local_checkout {
            assert_serialized_upper(checkout, |meter| meter.checkout(checkout));
        }
        for component in &branch_projection.monorepo_component {
            assert_serialized_upper(component, |meter| meter.component(component));
        }
        for link in &branch_projection.project_repository_link {
            assert_serialized_upper(link, |meter| meter.link(link));
        }

        let expanding_lowercase = "İ";
        assert!(expanding_lowercase.to_lowercase().len() > expanding_lowercase.len());
        let mut unicode_projection = fixture_generation(Fixture::A);
        unicode_projection.git_repository[0].name = expanding_lowercase.to_string();
        unicode_projection.monorepo_component[0].name = expanding_lowercase.to_string();
        unicode_projection.project_repository_link[0].project_name =
            expanding_lowercase.to_string();

        for (generation, c2a_valid) in [
            (fixture_generation(Fixture::A), true),
            (permutation_generation(), true),
            (rich_projection, false),
            (branch_projection, false),
            (unicode_projection, false),
        ] {
            let upper = borrowed_generation_upper_bound(&generation).unwrap();
            let raw = json_generation(generation).unwrap();
            let exact = validate_limit_and_secret_boundary(&raw, &[]).unwrap();
            assert!(upper.upper_bytes >= exact.accounting.bytes);
            assert!(!upper.secret);
            let BorrowedLimitAccounting::Exact(borrowed_accounting) = upper.accounting else {
                panic!("serializable fixture produced lower-bound accounting");
            };
            assert_eq!(borrowed_accounting, exact.accounting);
            if c2a_valid {
                assert!(invoke_c2a(
                    raw,
                    mint_mac_key_with(|bytes| {
                        bytes.fill(0x2c);
                        Ok(())
                    })
                    .unwrap(),
                )
                .is_ok());
            }
        }
    }

    #[test]
    fn proposal_row_member_count_and_punctuation_are_exact() {
        let mut pending = fixture_generation(Fixture::A)
            .correction_proposal
            .pop()
            .unwrap();
        let pending_row = json_proposal_row(pending.clone()).unwrap();
        assert_eq!(pending_row.as_object().unwrap().len(), 8);
        let mut punctuation = BorrowedJsonUpperMeter::new();
        punctuation.object(8).unwrap();
        assert_eq!(punctuation.bytes, 9);

        pending.status = CorrectionProposalStatus::Applied;
        pending.applied_at = Some(pending.created_at);
        pending.applied_digest = Some("f".repeat(64));
        let applied_row = json_proposal_row(pending).unwrap();
        assert_eq!(applied_row.as_object().unwrap().len(), 7);
        let mut punctuation = BorrowedJsonUpperMeter::new();
        punctuation.object(7).unwrap();
        assert_eq!(punctuation.bytes, 8);
    }

    #[test]
    fn complete_secret_scan_precedes_invalid_timestamp_and_number() {
        let invalid_timestamp = OffsetDateTime::UNIX_EPOCH.replace_year(-1).unwrap();
        let mut timestamp_class = fixture_generation(Fixture::A);
        timestamp_class.work_project[0].created_at = invalid_timestamp;
        assert!(matches!(
            borrowed_generation_upper_bound(&timestamp_class)
                .unwrap()
                .accounting,
            BorrowedLimitAccounting::InvalidLowerBound
        ));
        let mut timestamp_only = fixture_generation(Fixture::A);
        timestamp_only.work_project[0].created_at = invalid_timestamp;
        assert!(matches!(
            prepare_generation(timestamp_only),
            Err(C2AcquisitionError::InvalidGeneration)
        ));
        let mut timestamp_and_secret = fixture_generation(Fixture::A);
        timestamp_and_secret.work_project[0].created_at = invalid_timestamp;
        timestamp_and_secret.target.project_name = "sk-abcdefghijklmnopqrstuvwxyz".to_string();
        assert!(matches!(
            prepare_generation(timestamp_and_secret),
            Err(C2AcquisitionError::SecretMaterial)
        ));

        let odd_second_offset = time::UtcOffset::from_hms(0, 0, 1).unwrap();
        let invalid_offset_timestamp = OffsetDateTime::UNIX_EPOCH.to_offset(odd_second_offset);
        let mut offset_class = fixture_generation(Fixture::A);
        offset_class.work_project[0].created_at = invalid_offset_timestamp;
        assert!(matches!(
            borrowed_generation_upper_bound(&offset_class)
                .unwrap()
                .accounting,
            BorrowedLimitAccounting::InvalidLowerBound
        ));
        let mut offset_only = fixture_generation(Fixture::A);
        offset_only.work_project[0].created_at = invalid_offset_timestamp;
        assert!(matches!(
            prepare_generation(offset_only),
            Err(C2AcquisitionError::InvalidGeneration)
        ));

        let mut number_class = fixture_generation(Fixture::A);
        number_class.memory_item[0].confidence =
            engram_core::memory::MemoryConfidence::new(f32::NAN);
        assert!(matches!(
            borrowed_generation_upper_bound(&number_class)
                .unwrap()
                .accounting,
            BorrowedLimitAccounting::InvalidExact
        ));
        let mut number_only = fixture_generation(Fixture::A);
        number_only.memory_item[0].confidence =
            engram_core::memory::MemoryConfidence::new(f32::NAN);
        assert!(matches!(
            prepare_generation(number_only),
            Err(C2AcquisitionError::InvalidGeneration)
        ));
        let mut number_and_secret = fixture_generation(Fixture::A);
        number_and_secret.memory_item[0].confidence =
            engram_core::memory::MemoryConfidence::new(f32::NAN);
        number_and_secret.target.project_name = "sk-abcdefghijklmnopqrstuvwxyz".to_string();
        assert!(matches!(
            prepare_generation(number_and_secret),
            Err(C2AcquisitionError::SecretMaterial)
        ));
    }

    #[test]
    fn every_derived_projection_secret_precedes_invalid_typed_values() {
        const UPPERCASE_TOKEN: &str = "SK-ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        assert!(!likely_secret_in_string(UPPERCASE_TOKEN));
        assert!(likely_secret_in_string(&UPPERCASE_TOKEN.to_lowercase()));
        let invalid_timestamp = OffsetDateTime::UNIX_EPOCH.replace_year(-1).unwrap();

        let mut repository_name = fixture_generation(Fixture::A);
        repository_name.git_repository[0].name = UPPERCASE_TOKEN.to_string();
        repository_name.work_project[0].created_at = invalid_timestamp;
        assert!(matches!(
            prepare_generation(repository_name),
            Err(C2AcquisitionError::SecretMaterial)
        ));

        let mut component_name = fixture_generation(Fixture::A);
        component_name.monorepo_component[0].name = UPPERCASE_TOKEN.to_string();
        component_name.memory_item[0].confidence =
            engram_core::memory::MemoryConfidence::new(f32::NAN);
        assert!(matches!(
            prepare_generation(component_name),
            Err(C2AcquisitionError::SecretMaterial)
        ));

        let mut link_project_name = fixture_generation(Fixture::A);
        link_project_name.project_repository_link[0].project_name = UPPERCASE_TOKEN.to_string();
        link_project_name.work_project[0].created_at = invalid_timestamp;
        assert!(matches!(
            prepare_generation(link_project_name),
            Err(C2AcquisitionError::SecretMaterial)
        ));

        const URL_PAYLOAD: &str = "//user:password@host";
        assert!(!likely_secret_in_string(URL_PAYLOAD));
        let repository_scope = MemoryScope::Repository {
            repository_id: None,
            remote_url: Some(URL_PAYLOAD.to_string()),
            local_path: None,
        };
        assert!(likely_secret_in_string(&scope_key(&repository_scope)));
        let mut scoped = fixture_generation(Fixture::A);
        scoped.memory_item[0].scope = repository_scope;
        scoped.memory_item[0].confidence = engram_core::memory::MemoryConfidence::new(f32::NAN);
        assert!(matches!(
            prepare_generation(scoped),
            Err(C2AcquisitionError::SecretMaterial)
        ));
    }

    #[test]
    fn cumulative_multirow_generation_hits_exact_global_byte_cap_then_rejects_plus_one() {
        let mut low = 0_usize;
        let mut high = MAX_LARGE_SCALAR_BYTES + 1;
        while low + 1 < high {
            let middle = low + (high - low) / 2;
            let generation = cumulative_generation(middle);
            match borrowed_generation_upper_bound(&generation) {
                Ok(_) => low = middle,
                Err(C2AcquisitionError::LimitExceeded { .. }) => high = middle,
                Err(error) => panic!("unexpected cumulative preflight result: {error}"),
            }
        }
        let exact_generation = cumulative_generation(low);
        let exact_raw = json_generation(cumulative_generation(low)).unwrap();
        let exact_preflight = borrowed_generation_upper_bound(&cumulative_generation(low)).unwrap();
        let BorrowedLimitAccounting::Exact(exact_accounting) = exact_preflight.accounting else {
            panic!("valid cumulative fixture produced lower-bound accounting");
        };
        assert_eq!(
            account_virtual_input(&exact_raw, &[]).unwrap().bytes,
            super::super::MAX_RAW_BYTES
        );
        assert_eq!(
            exact_accounting,
            account_virtual_input(&exact_raw, &[]).unwrap()
        );
        assert!(prepare_generation(exact_generation).is_ok());
        assert!(matches!(
            prepare_generation(cumulative_generation(low + 1)),
            Err(C2AcquisitionError::LimitExceeded { .. })
        ));

        let invalid_timestamp = OffsetDateTime::UNIX_EPOCH.replace_year(-1).unwrap();
        let mut invalid = cumulative_generation(low + 1);
        invalid.work_project[0].created_at = invalid_timestamp;
        assert!(matches!(
            prepare_generation(invalid),
            Err(C2AcquisitionError::LimitExceeded { .. })
        ));

        let mut invalid_and_secret = cumulative_generation(low + 1);
        invalid_and_secret.work_project[0].created_at = invalid_timestamp;
        invalid_and_secret.target.project_name = "sk-abcdefghijklmnopqrstuvwxyz".to_string();
        assert!(matches!(
            prepare_generation(invalid_and_secret),
            Err(C2AcquisitionError::LimitExceeded { .. })
        ));
    }

    #[test]
    fn exact_row_limit_precedes_invalid_typed_value_and_secret() {
        let generation_at = |command_length: usize| {
            let mut generation = fixture_generation(Fixture::A);
            let mut procedure = cumulative_memory(90, MAX_LARGE_SCALAR_BYTES);
            procedure.kind = MemoryKind::Procedure;
            procedure.procedure = Some(ProcedureCard {
                task: "bounded row accounting".to_string(),
                prerequisites: Vec::new(),
                commands: vec!["y".repeat(command_length)],
                failure_signatures: Vec::new(),
                verification: ProcedureVerification {
                    command: "true".to_string(),
                    expected_exit_code: 0,
                    expected_output_contains: "ok".to_string(),
                    evidence_path: None,
                    evidence_sha256: None,
                    verified_at: None,
                },
                expires_at: None,
            });
            generation.memory_item.push(procedure);
            generation
        };

        let mut low = 0_usize;
        let mut high = MAX_LARGE_SCALAR_BYTES + 1;
        while low + 1 < high {
            let middle = low + (high - low) / 2;
            match borrowed_generation_upper_bound(&generation_at(middle)) {
                Ok(_) => low = middle,
                Err(C2AcquisitionError::LimitExceeded { .. }) => high = middle,
                Err(error) => panic!("unexpected row-boundary preflight result: {error}"),
            }
        }
        assert!(prepare_generation(generation_at(low)).is_ok());
        assert!(matches!(
            prepare_generation(generation_at(low + 1)),
            Err(C2AcquisitionError::LimitExceeded { .. })
        ));

        let invalid_timestamp = OffsetDateTime::UNIX_EPOCH.replace_year(-1).unwrap();
        let mut invalid = generation_at(low + 1);
        invalid.work_project[0].created_at = invalid_timestamp;
        assert!(matches!(
            prepare_generation(invalid),
            Err(C2AcquisitionError::LimitExceeded { .. })
        ));

        let mut invalid_and_secret = generation_at(low + 1);
        invalid_and_secret.work_project[0].created_at = invalid_timestamp;
        invalid_and_secret.target.project_name = "sk-abcdefghijklmnopqrstuvwxyz".to_string();
        assert!(matches!(
            prepare_generation(invalid_and_secret),
            Err(C2AcquisitionError::LimitExceeded { .. })
        ));
    }

    #[test]
    fn complete_generation_accounting_matches_native_dry_run_exactly() {
        let borrowed = borrowed_generation_upper_bound(&fixture_generation(Fixture::A)).unwrap();
        let raw = json_generation(fixture_generation(Fixture::A)).unwrap();
        let BorrowedLimitAccounting::Exact(accounting) = borrowed.accounting else {
            panic!("valid fixture produced lower-bound accounting");
        };
        assert_eq!(accounting, account_virtual_input(&raw, &[]).unwrap());
        let prepared = prepare_generation(fixture_generation(Fixture::A)).unwrap();
        let C2PreparedGeneration {
            target,
            accounting,
            mut variables,
        } = prepared;
        let tables = NativeTables {
            tables: TABLE_KINDS
                .into_iter()
                .map(|table| {
                    let NativeValue::Array(rows) = variables.remove(table.tag()).unwrap() else {
                        panic!("prepared table must be an array");
                    };
                    (table, rows)
                })
                .collect(),
        };
        assert!(variables.is_empty());
        let dry = dry_native_tables(&target, &tables).unwrap();
        assert!(!dry.unsupported);
        assert_eq!(dry.accounting, accounting);
    }

    #[test]
    fn native_converter_uses_only_thing_and_top_level_datetime_exceptions() {
        let generation = fixture_generation(Fixture::A);
        let raw = json_generation(generation).unwrap();
        let row = raw.work_project.into_iter().next().unwrap();
        let native = json_row_to_native(TableKind::WorkProject, row).unwrap();
        let NativeValue::Object(object) = native else {
            panic!("row must be an object");
        };
        assert!(matches!(object.get("id"), Some(NativeValue::Thing(_))));
        assert!(matches!(
            object.get("created_at"),
            Some(NativeValue::Datetime(_))
        ));
        assert!(matches!(
            object.get("description"),
            Some(NativeValue::Strand(_))
        ));
    }

    #[test]
    fn malformed_outer_response_and_wrong_table_id_reject() {
        assert!(matches!(
            native_tables(NativeValue::Null),
            Err(C2AcquisitionError::InvalidResponse)
        ));
        let (_, _, mut missing) = native_outer_from_generation(Fixture::A);
        let NativeValue::Object(missing) = &mut missing else {
            unreachable!();
        };
        missing.remove(TableKind::MemoryItem.tag());
        assert!(matches!(
            native_tables(NativeValue::Object(missing.clone())),
            Err(C2AcquisitionError::InvalidResponse)
        ));

        let (_, _, mut extra) = native_outer_from_generation(Fixture::A);
        let NativeValue::Object(extra) = &mut extra else {
            unreachable!();
        };
        extra.insert("extra".to_string(), NativeValue::Null);
        assert!(matches!(
            native_tables(NativeValue::Object(extra.clone())),
            Err(C2AcquisitionError::InvalidResponse)
        ));

        let (_, _, mut wrong) = native_outer_from_generation(Fixture::A);
        let NativeValue::Object(wrong) = &mut wrong else {
            unreachable!();
        };
        let rows = wrong.remove(TableKind::MemoryItem.tag()).unwrap();
        wrong.insert("wrong".to_string(), rows);
        assert!(matches!(
            native_tables(NativeValue::Object(wrong.clone())),
            Err(C2AcquisitionError::InvalidResponse)
        ));

        let (_, _, mut non_array) = native_outer_from_generation(Fixture::A);
        let NativeValue::Object(non_array) = &mut non_array else {
            unreachable!();
        };
        non_array.insert(TableKind::MemoryItem.tag().to_string(), NativeValue::Null);
        assert!(matches!(
            native_tables(NativeValue::Object(non_array.clone())),
            Err(C2AcquisitionError::InvalidResponse)
        ));

        let (target, accounting, mut non_object_row) = native_outer_from_generation(Fixture::A);
        let NativeValue::Object(tables) = &mut non_object_row else {
            unreachable!();
        };
        let NativeValue::Array(rows) = tables.get_mut(TableKind::MemoryItem.tag()).unwrap() else {
            unreachable!();
        };
        rows.0[0] = NativeValue::Null;
        assert!(matches!(
            validate_collected_value(non_object_row, target, accounting),
            Err(C2AcquisitionError::InvalidResponse)
        ));

        let thing = Thing::from(("wrong".to_string(), PROJECT_ID.to_string()));
        assert_eq!(
            canonical_thing_id(TableKind::WorkProject, &thing),
            Err(C2AcquisitionError::InvalidResponse)
        );
    }

    #[test]
    fn collector_precedence_is_overflow_then_secret_then_unsupported() {
        use surrealdb_core::sql::Future;

        let (target, accounting, mut overflow) = native_outer_from_generation(Fixture::A);
        let NativeValue::Object(tables) = &mut overflow else {
            unreachable!();
        };
        for table in TABLE_KINDS {
            let rows = vec![NativeValue::Null; table.row_cap() + 1];
            tables.insert(
                table.tag().to_string(),
                NativeValue::Array(NativeArray::from(rows)),
            );
        }
        let NativeValue::Array(rows) = tables.get_mut(TableKind::MemoryItem.tag()).unwrap() else {
            unreachable!();
        };
        rows.0[0] = NativeValue::Object(NativeObject::from(BTreeMap::from([
            (
                "credential".to_string(),
                NativeValue::Strand(test_strand("sk-abcdefghijklmnopqrstuvwxyz")),
            ),
            (
                "unsupported".to_string(),
                NativeValue::from(Future::from(NativeValue::Null)),
            ),
        ])));
        assert!(matches!(
            validate_collected_value(overflow, target, accounting),
            Err(C2AcquisitionError::LimitExceeded {
                table_overflow_mask: Some(0x01ff),
            })
        ));

        let (target, accounting, mut secret_and_unsupported) =
            native_outer_from_generation(Fixture::A);
        let row = native_row_mut(&mut secret_and_unsupported, TableKind::WorkTask, 0);
        row.insert(
            "credential".to_string(),
            NativeValue::Strand(test_strand("sk-abcdefghijklmnopqrstuvwxyz")),
        );
        row.insert(
            "unsupported".to_string(),
            NativeValue::from(Future::from(NativeValue::Null)),
        );
        assert!(matches!(
            validate_collected_value(secret_and_unsupported, target, accounting),
            Err(C2AcquisitionError::SecretMaterial)
        ));
    }

    #[test]
    fn invalid_native_rows_never_preempt_later_limit_secret_or_unsupported_evidence() {
        use surrealdb_core::sql::Future;

        let invalid_outer = |invalid_kind: u8| {
            assert!(
                invalid_kind <= 7,
                "invalid-row fixture selector escaped its bound"
            );
            let (target, accounting, mut outer) = native_outer_from_generation(Fixture::A);
            match invalid_kind {
                0 => {
                    native_row_mut(&mut outer, TableKind::MemoryItem, 0).remove("id");
                }
                1 => {
                    native_row_mut(&mut outer, TableKind::MemoryItem, 0).insert(
                        "id".to_string(),
                        NativeValue::Strand(test_strand(PROJECT_ID)),
                    );
                }
                2 => {
                    native_row_mut(&mut outer, TableKind::MemoryItem, 0).insert(
                        "id".to_string(),
                        NativeValue::Thing(Thing::from((
                            "wrong_table".to_string(),
                            PROJECT_ID.to_string(),
                        ))),
                    );
                }
                3 => {
                    native_row_mut(&mut outer, TableKind::MemoryItem, 0).insert(
                        "id".to_string(),
                        NativeValue::Thing(Thing::from((
                            TableKind::MemoryItem.tag().to_string(),
                            "not-a-canonical-id".to_string(),
                        ))),
                    );
                }
                4 => {
                    native_row_mut(&mut outer, TableKind::MemoryItem, 0).insert(
                        "record_id".to_string(),
                        NativeValue::Strand(test_strand(PROJECT_ID)),
                    );
                }
                5 => {
                    native_row_mut(&mut outer, TableKind::MemoryItem, 0).insert(
                        "created_at".to_string(),
                        NativeValue::Strand(test_strand(TIMESTAMP)),
                    );
                }
                6 => {
                    native_row_mut(&mut outer, TableKind::MemoryItem, 0).insert(
                        "created_at".to_string(),
                        NativeValue::Datetime(NativeDatetime::MAX_UTC),
                    );
                }
                7 => {
                    let NativeValue::Object(tables) = &mut outer else {
                        unreachable!();
                    };
                    let Some(NativeValue::Array(rows)) =
                        tables.get_mut(TableKind::MemoryItem.tag())
                    else {
                        unreachable!();
                    };
                    rows.0[0] = NativeValue::Null;
                }
                _ => unreachable!(),
            }
            (target, accounting, outer)
        };

        for invalid_kind in 0..=7 {
            for later_evidence in 0..=3 {
                let (target, accounting, mut outer) = invalid_outer(invalid_kind);
                if later_evidence >= 1 {
                    native_row_mut(&mut outer, TableKind::ProjectRepositoryLink, 0).insert(
                        "unsupported".to_string(),
                        NativeValue::from(Future::from(NativeValue::Null)),
                    );
                }
                if later_evidence >= 2 {
                    native_row_mut(&mut outer, TableKind::ProjectRepositoryLink, 0).insert(
                        "credential".to_string(),
                        NativeValue::Strand(test_strand("sk-abcdefghijklmnopqrstuvwxyz")),
                    );
                }
                if later_evidence >= 3 {
                    native_row_mut(&mut outer, TableKind::ProjectRepositoryLink, 0).insert(
                        "oversized".to_string(),
                        NativeValue::Strand(test_strand(
                            &"x".repeat(MAX_ORDINARY_SCALAR_BYTES + 1),
                        )),
                    );
                }
                let expected = match later_evidence {
                    0 => C2AcquisitionError::InvalidResponse,
                    1 => C2AcquisitionError::UnsupportedNativeValue,
                    2 => C2AcquisitionError::SecretMaterial,
                    3 => C2AcquisitionError::LimitExceeded {
                        table_overflow_mask: None,
                    },
                    _ => unreachable!(),
                };
                let Err(actual) = validate_collected_value(outer, target, accounting) else {
                    panic!(
                        "invalid row {invalid_kind}, later evidence {later_evidence} authorized"
                    );
                };
                assert_eq!(
                    actual, expected,
                    "invalid row {invalid_kind}, later evidence {later_evidence}"
                );
            }
        }

        for (table, id) in [
            ("sk-abcdefghijklmnopqrstuvwxyz", PROJECT_ID),
            (TableKind::MemoryItem.tag(), "sk-abcdefghijklmnopqrstuvwxyz"),
        ] {
            let (target, accounting, mut outer) = native_outer_from_generation(Fixture::A);
            native_row_mut(&mut outer, TableKind::MemoryItem, 0).insert(
                "id".to_string(),
                NativeValue::Thing(Thing::from((table.to_string(), id.to_string()))),
            );
            assert!(matches!(
                validate_collected_value(outer, target, accounting),
                Err(C2AcquisitionError::SecretMaterial)
            ));
        }

        let (target, accounting, mut collision_with_unsupported) =
            native_outer_from_generation(Fixture::A);
        let row = native_row_mut(&mut collision_with_unsupported, TableKind::MemoryItem, 0);
        row.insert(
            "record_id".to_string(),
            NativeValue::Strand(test_strand(PROJECT_ID)),
        );
        row.insert(
            "id".to_string(),
            NativeValue::from(Future::from(NativeValue::Null)),
        );
        assert!(matches!(
            validate_collected_value(collision_with_unsupported, target, accounting),
            Err(C2AcquisitionError::UnsupportedNativeValue)
        ));

        let (target, accounting, mut collision_with_limit) =
            native_outer_from_generation(Fixture::A);
        let row = native_row_mut(&mut collision_with_limit, TableKind::MemoryItem, 0);
        row.insert(
            "record_id".to_string(),
            NativeValue::Strand(test_strand(PROJECT_ID)),
        );
        row.insert(
            "id".to_string(),
            NativeValue::Strand(test_strand(&"x".repeat(MAX_ORDINARY_SCALAR_BYTES + 1))),
        );
        assert!(matches!(
            validate_collected_value(collision_with_limit, target, accounting),
            Err(C2AcquisitionError::LimitExceeded { .. })
        ));
    }

    #[test]
    fn none_null_omission_datetime_and_record_id_paths_are_exact() {
        let (_, _, mut outer) = native_outer_from_generation(Fixture::A);
        let row = native_row_mut(&mut outer, TableKind::MemoryItem, 0);
        row.insert("session_id".to_string(), NativeValue::None);
        let normalized = normalize_native_object(
            row.clone(),
            SchemaPath::MemoryRow,
            TableKind::MemoryItem,
            true,
        )
        .unwrap();
        assert!(normalized["session_id"].is_null());

        let (_, _, mut outer) = native_outer_from_generation(Fixture::A);
        let row = native_row_mut(&mut outer, TableKind::MemoryItem, 0);
        let NativeValue::Object(item) = row.get_mut("item").unwrap() else {
            panic!("embedded memory must be an object");
        };
        item.insert(
            "pending_correction_proposal_id".to_string(),
            NativeValue::None,
        );
        let normalized = normalize_native_object(
            row.clone(),
            SchemaPath::MemoryRow,
            TableKind::MemoryItem,
            true,
        )
        .unwrap();
        assert!(normalized["item"]
            .as_object()
            .unwrap()
            .get("pending_correction_proposal_id")
            .is_none());

        let (_, _, mut outer) = native_outer_from_generation(Fixture::A);
        let row = native_row_mut(&mut outer, TableKind::MemoryItem, 0);
        let NativeValue::Object(item) = row.get_mut("item").unwrap() else {
            unreachable!();
        };
        item.insert("title".to_string(), NativeValue::None);
        assert_eq!(
            normalize_native_object(
                row.clone(),
                SchemaPath::MemoryRow,
                TableKind::MemoryItem,
                true,
            ),
            Err(C2AcquisitionError::UnsupportedNativeValue)
        );

        let (_, _, mut pending_outer) = native_outer_from_generation(Fixture::A);
        let pending = native_row_mut(&mut pending_outer, TableKind::CorrectionProposal, 0);
        pending.insert("pending_obsolete_id".to_string(), NativeValue::None);
        assert_eq!(
            normalize_native_object(
                pending.clone(),
                SchemaPath::ProposalRow,
                TableKind::CorrectionProposal,
                true,
            ),
            Err(C2AcquisitionError::UnsupportedNativeValue)
        );
        pending.insert(
            "status_key".to_string(),
            NativeValue::Strand(test_strand("applied")),
        );
        let applied = normalize_native_object(
            pending.clone(),
            SchemaPath::ProposalRow,
            TableKind::CorrectionProposal,
            true,
        )
        .unwrap();
        assert!(applied
            .as_object()
            .unwrap()
            .get("pending_obsolete_id")
            .is_none());

        let (_, _, mut outer) = native_outer_from_generation(Fixture::A);
        let row = native_row_mut(&mut outer, TableKind::WorkProject, 0);
        assert!(matches!(
            row.get("created_at"),
            Some(NativeValue::Datetime(_))
        ));
        let native_datetime = row.get("created_at").unwrap().clone();
        let normalized = normalize_native_object(
            row.clone(),
            SchemaPath::ProjectRow,
            TableKind::WorkProject,
            true,
        )
        .unwrap();
        assert_eq!(normalized["created_at"], TIMESTAMP);
        row.insert(
            "created_at".to_string(),
            NativeValue::Strand(test_strand(TIMESTAMP)),
        );
        assert_eq!(
            normalize_native_object(
                row.clone(),
                SchemaPath::ProjectRow,
                TableKind::WorkProject,
                true,
            ),
            Err(C2AcquisitionError::InvalidResponse)
        );

        let (_, _, mut outer) = native_outer_from_generation(Fixture::A);
        let row = native_row_mut(&mut outer, TableKind::MemoryItem, 0);
        let NativeValue::Object(item) = row.get_mut("item").unwrap() else {
            unreachable!();
        };
        item.insert("created_at".to_string(), native_datetime);
        assert_eq!(
            normalize_native_object(
                row.clone(),
                SchemaPath::MemoryRow,
                TableKind::MemoryItem,
                true,
            ),
            Err(C2AcquisitionError::UnsupportedNativeValue)
        );

        let (_, _, mut outer) = native_outer_from_generation(Fixture::A);
        let row = native_row_mut(&mut outer, TableKind::WorkProject, 0);
        row.remove("id");
        assert_eq!(
            normalize_native_object(
                row.clone(),
                SchemaPath::ProjectRow,
                TableKind::WorkProject,
                true,
            ),
            Err(C2AcquisitionError::InvalidResponse)
        );
        row.insert("id".to_string(), NativeValue::Null);
        assert_eq!(
            normalize_native_object(
                row.clone(),
                SchemaPath::ProjectRow,
                TableKind::WorkProject,
                true,
            ),
            Err(C2AcquisitionError::InvalidResponse)
        );
        row.insert(
            "id".to_string(),
            NativeValue::Thing(Thing::from((
                TableKind::WorkTask.tag().to_string(),
                PROJECT_ID.to_string(),
            ))),
        );
        assert_eq!(
            normalize_native_object(
                row.clone(),
                SchemaPath::ProjectRow,
                TableKind::WorkProject,
                true,
            ),
            Err(C2AcquisitionError::InvalidResponse)
        );
        row.insert(
            "id".to_string(),
            NativeValue::Thing(Thing::from((
                TableKind::WorkProject.tag().to_string(),
                PROJECT_ID.to_string(),
            ))),
        );
        row.insert(
            "record_id".to_string(),
            NativeValue::Strand(test_strand(PROJECT_ID)),
        );
        assert_eq!(
            normalize_native_object(
                row.clone(),
                SchemaPath::ProjectRow,
                TableKind::WorkProject,
                true,
            ),
            Err(C2AcquisitionError::InvalidResponse)
        );

        let (target, _, mut outer) = native_outer_from_generation(Fixture::A);
        let row = native_row_mut(&mut outer, TableKind::MemoryItem, 0);
        let NativeValue::Object(item) = row.get_mut("item").unwrap() else {
            unreachable!();
        };
        item.insert("title".to_string(), NativeValue::Null);
        let raw = collect_with_matching_accounting(outer, target).unwrap();
        assert_eq!(raw.memory_item[0]["item"]["title"], JsonValue::Null);
        assert!(matches!(
            invoke_c2a(
                raw,
                mint_mac_key_with(|bytes| {
                    bytes.fill(0x31);
                    Ok(())
                })
                .unwrap(),
            ),
            Err(C2AcquisitionError::InvalidRecord)
        ));
    }

    #[test]
    fn every_none_datetime_and_thing_dispatch_path_is_pinned() {
        use NoneAction::{Null, Omit, Reject};
        use SchemaPath as S;

        let omit = [
            (S::MemoryItem, "correction_proposal_id"),
            (S::MemoryItem, "pending_correction_proposal_id"),
            (S::ProcedurePrerequisite, "source"),
            (S::Proposal, "applied_digest"),
            (S::Component, "source_path"),
            (S::Component, "source_sha256"),
            (S::ForgetReceiptRow, "completed_at"),
        ];
        for (schema, key) in omit {
            assert!(none_action(schema, key) == Omit);
        }

        let null = [
            (S::MemoryRow, "session_id"),
            (S::MemoryItem, "last_used_at"),
            (S::MemoryItem, "review_after"),
            (S::MemoryItem, "archive"),
            (S::MemoryItem, "procedure"),
            (S::Scope, "project_id"),
            (S::Scope, "project_name"),
            (S::Scope, "task_id"),
            (S::Scope, "entity_id"),
            (S::Scope, "repository_id"),
            (S::Scope, "remote_url"),
            (S::Scope, "local_path"),
            (S::Writer, "harness_version"),
            (S::Writer, "surface"),
            (S::Writer, "session_id"),
            (S::Model, "version"),
            (S::Evidence, "summary"),
            (S::Evidence, "excerpt"),
            (S::Archive, "archived_by"),
            (S::Procedure, "expires_at"),
            (S::ProcedureVerification, "evidence_path"),
            (S::ProcedureVerification, "evidence_sha256"),
            (S::ProcedureVerification, "verified_at"),
            (S::Proposal, "applied_at"),
            (S::ProjectRow, "description"),
            (S::TaskRow, "description"),
            (S::TaskRow, "jira_key"),
            (S::RepositoryRow, "remote_url"),
            (S::Repository, "remote_url"),
            (S::Repository, "default_branch"),
            (S::Repository, "description"),
            (S::CheckoutRow, "repository_id"),
            (S::CheckoutRow, "current_branch"),
            (S::CheckoutRow, "head_sha"),
            (S::CheckoutRow, "is_dirty"),
            (S::Checkout, "repository_id"),
            (S::Checkout, "current_branch"),
            (S::Checkout, "head_sha"),
            (S::Checkout, "is_dirty"),
            (S::ComponentRow, "kind"),
            (S::Component, "kind"),
            (S::Component, "description"),
            (S::LinkRow, "project_id"),
            (S::LinkRow, "component_id"),
            (S::LinkRow, "component_path_key"),
            (S::Link, "project_id"),
            (S::Link, "component_id"),
            (S::Link, "component_path"),
        ];
        for (schema, key) in null {
            assert!(none_action(schema, key) == Null);
        }
        for (schema, key) in [
            (S::MemoryItem, "title"),
            (S::Proposal, "obsolete_id"),
            (S::ProjectRow, "name_key"),
            (S::Unknown, "unknown"),
        ] {
            assert!(none_action(schema, key) == Reject);
        }
        assert!(object_none_action(S::ProposalRow, "pending_obsolete_id", true) == Omit);
        assert!(object_none_action(S::ProposalRow, "pending_obsolete_id", false) == Reject);

        let datetime_paths = [
            (TableKind::MemoryItem, &["created_at", "updated_at"][..]),
            (TableKind::CorrectionProposal, &["created_at"][..]),
            (
                TableKind::MemoryForgetReceipt,
                &["created_at", "completed_at"][..],
            ),
            (TableKind::WorkProject, &["created_at", "updated_at"][..]),
            (TableKind::WorkTask, &["created_at", "updated_at"][..]),
            (TableKind::GitRepository, &["created_at", "updated_at"][..]),
            (
                TableKind::LocalCheckout,
                &["created_at", "updated_at", "last_seen_at"][..],
            ),
            (
                TableKind::MonorepoComponent,
                &["created_at", "updated_at"][..],
            ),
            (
                TableKind::ProjectRepositoryLink,
                &["created_at", "updated_at"][..],
            ),
        ];
        for (table, keys) in datetime_paths {
            for key in keys {
                assert!(top_level_datetime(table, key));
            }
            assert!(!top_level_datetime(table, "not_a_timestamp"));
        }

        for table in TABLE_KINDS {
            let canonical = Thing::from((table.tag().to_string(), PROJECT_ID.to_string()));
            assert_eq!(canonical_thing_id(table, &canonical).unwrap(), PROJECT_ID);
            let wrong_table = Thing::from(("wrong".to_string(), PROJECT_ID.to_string()));
            assert_eq!(
                canonical_thing_id(table, &wrong_table),
                Err(C2AcquisitionError::InvalidResponse)
            );
            let noncanonical = Thing::from((table.tag().to_string(), "not-a-uuid".to_string()));
            assert_eq!(
                canonical_thing_id(table, &noncanonical),
                Err(C2AcquisitionError::InvalidResponse)
            );
            let numeric = Thing::from((table.tag().to_string(), NativeId::Number(7)));
            assert_eq!(
                canonical_thing_id(table, &numeric),
                Err(C2AcquisitionError::InvalidResponse)
            );
        }

        for table in TABLE_KINDS
            .into_iter()
            .filter(|table| *table != TableKind::MemoryForgetReceipt)
        {
            let (_, _, mut outer) = native_outer_from_generation(Fixture::A);
            let row = native_row_mut(&mut outer, table, 0);
            row.insert(
                "record_id".to_string(),
                NativeValue::Strand(test_strand(PROJECT_ID)),
            );
            assert_eq!(
                normalize_native_object(row.clone(), row_schema(table), table, true),
                Err(C2AcquisitionError::InvalidResponse)
            );
        }
        let forget_collision = NativeObject::from(BTreeMap::from([
            (
                "id".to_string(),
                NativeValue::Thing(Thing::from((
                    TableKind::MemoryForgetReceipt.tag().to_string(),
                    PROJECT_ID.to_string(),
                ))),
            ),
            (
                "record_id".to_string(),
                NativeValue::Strand(test_strand(PROJECT_ID)),
            ),
        ]));
        assert_eq!(
            normalize_native_object(
                forget_collision,
                row_schema(TableKind::MemoryForgetReceipt),
                TableKind::MemoryForgetReceipt,
                true,
            ),
            Err(C2AcquisitionError::InvalidResponse)
        );
    }

    #[test]
    fn unknown_json_native_values_are_preserved_then_rejected_by_c2a() {
        let (target, _, mut outer) = native_outer_from_generation(Fixture::A);
        let unknown = NativeValue::Object(NativeObject::from(BTreeMap::from([
            (
                "nested".to_string(),
                NativeValue::Array(NativeArray::from(vec![
                    NativeValue::Bool(true),
                    NativeValue::Null,
                    NativeValue::Number(NativeNumber::Int(7)),
                ])),
            ),
            (
                "text".to_string(),
                NativeValue::Strand(test_strand("preserved-unknown")),
            ),
        ])));
        native_row_mut(&mut outer, TableKind::WorkTask, 0).insert("unknown".to_string(), unknown);
        let raw = collect_with_matching_accounting(outer, target).unwrap();
        assert_eq!(
            raw.work_task[0]["unknown"],
            json!({
                "nested": [true, null, 7],
                "text": "preserved-unknown"
            })
        );
        assert!(matches!(
            invoke_c2a(
                raw,
                mint_mac_key_with(|bytes| {
                    bytes.fill(0x32);
                    Ok(())
                })
                .unwrap(),
            ),
            Err(C2AcquisitionError::InvalidRecord)
        ));
    }

    #[test]
    fn secret_canaries_cover_target_every_table_unknown_keys_and_nested_values() {
        const CANARY: &str = "API_TOKEN=synthetic-c2b1-canary";

        for target_field in 0..8 {
            let mut generation = fixture_generation(Fixture::A);
            match target_field {
                0 => generation.target.project_id = CANARY.to_string(),
                1 => generation.target.project_name = CANARY.to_string(),
                2 => generation.target.repository_id = CANARY.to_string(),
                3 => generation.target.repository_remote = CANARY.to_string(),
                4 => generation.target.checkout_id = CANARY.to_string(),
                5 => generation.target.checkout_path = CANARY.to_string(),
                6 => generation.target.task_id = Some(CANARY.to_string()),
                7 => generation.target.task_name = Some(CANARY.to_string()),
                _ => unreachable!(),
            }
            assert!(matches!(
                prepare_generation(generation),
                Err(C2AcquisitionError::SecretMaterial)
            ));
        }

        for table in TABLE_KINDS {
            let (target, accounting, mut outer) = native_outer_from_generation(Fixture::A);
            let NativeValue::Object(tables) = &mut outer else {
                unreachable!();
            };
            let NativeValue::Array(rows) = tables.get_mut(table.tag()).unwrap() else {
                unreachable!();
            };
            if rows.is_empty() {
                rows.0
                    .push(NativeValue::Object(NativeObject::from(BTreeMap::from([
                        (
                            "id".to_string(),
                            NativeValue::Thing(Thing::from((
                                table.tag().to_string(),
                                variant_id(Fixture::A, 90),
                            ))),
                        ),
                        (
                            "unknown".to_string(),
                            NativeValue::Strand(test_strand(CANARY)),
                        ),
                    ]))));
            } else {
                let NativeValue::Object(row) = &mut rows.0[0] else {
                    unreachable!();
                };
                row.insert(
                    "unknown".to_string(),
                    NativeValue::Strand(test_strand(CANARY)),
                );
            }
            assert!(
                matches!(
                    validate_collected_value(outer, target, accounting),
                    Err(C2AcquisitionError::SecretMaterial)
                ),
                "{}",
                table.tag()
            );
        }

        let (target, accounting, mut outer) = native_outer_from_generation(Fixture::A);
        native_row_mut(&mut outer, TableKind::WorkTask, 0)
            .insert(CANARY.to_string(), NativeValue::Strand(test_strand("safe")));
        assert!(matches!(
            validate_collected_value(outer, target, accounting),
            Err(C2AcquisitionError::SecretMaterial)
        ));

        let (target, accounting, mut outer) = native_outer_from_generation(Fixture::A);
        native_row_mut(&mut outer, TableKind::WorkTask, 0).insert(
            "unknown".to_string(),
            NativeValue::Object(NativeObject::from(BTreeMap::from([(
                "client-secret".to_string(),
                NativeValue::Strand(test_strand("synthetic-c2b1-canary")),
            )]))),
        );
        assert!(matches!(
            validate_collected_value(outer, target, accounting),
            Err(C2AcquisitionError::SecretMaterial)
        ));
    }

    #[test]
    fn every_json_native_value_is_allowed_and_extended_values_are_rejected() {
        use surrealdb_core::sql::{
            Block, Constant, Duration as NativeDuration, Expression, Function, Future, Idiom, Mock,
            Subquery, Table,
        };

        let allowed = [
            NativeValue::Null,
            NativeValue::Bool(true),
            NativeValue::Number(NativeNumber::Int(-7)),
            NativeValue::Number(NativeNumber::Float(1.25)),
            NativeValue::Strand(test_strand("bounded")),
            NativeValue::Array(NativeArray::from(vec![NativeValue::Null])),
            NativeValue::Object(NativeObject::from(BTreeMap::from([(
                "key".to_string(),
                NativeValue::Bool(false),
            )]))),
        ];
        for value in allowed {
            assert!(normalize_native_value(
                value,
                SchemaPath::Unknown,
                TableKind::MemoryItem,
                None,
            )
            .is_ok());
        }

        let parsed_uuid =
            surrealdb_core::syn::value("u'01890f5e-7b00-7000-8000-000000000099'").unwrap();
        assert!(matches!(parsed_uuid, NativeValue::Uuid(_)));
        let parsed_geometry = surrealdb_core::syn::value("(1.0, 2.0)").unwrap();
        assert!(matches!(parsed_geometry, NativeValue::Geometry(_)));
        let parsed_range = surrealdb_core::syn::value("1..2").unwrap();
        assert!(matches!(parsed_range, NativeValue::Range(_)));
        let parsed_closure = surrealdb_core::syn::value("|$value| $value").unwrap();
        assert!(matches!(parsed_closure, NativeValue::Closure(_)));
        let parsed_regex = surrealdb_core::syn::value("/bounded/").unwrap();
        assert!(matches!(parsed_regex, NativeValue::Regex(_)));
        let parsed_cast = surrealdb_core::syn::value("<string> 7").unwrap();
        assert!(matches!(parsed_cast, NativeValue::Cast(_)));
        let parsed_refs: NativeValue = serde_json::from_value(json!({"Refs": []})).unwrap();
        assert!(matches!(parsed_refs, NativeValue::Refs(_)));
        let parsed_edges: NativeValue = serde_json::from_value(json!({
            "Edges": {
                "dir": "Out",
                "from": {"tb": "source", "id": {"String": PROJECT_ID}},
                "what": [{"Table": "edge"}]
            }
        }))
        .unwrap();
        assert!(matches!(parsed_edges, NativeValue::Edges(_)));
        let forbidden = vec![
            NativeValue::None,
            NativeValue::Number(NativeNumber::Float(f64::NAN)),
            surrealdb_core::syn::value("1.0dec").unwrap(),
            NativeValue::from(NativeDuration::from_secs(1)),
            parsed_uuid,
            parsed_geometry,
            NativeValue::from(surrealdb_core::sql::Bytes::from(vec![1_u8, 2, 3])),
            NativeValue::Thing(Thing::from((
                TableKind::MemoryItem.tag().to_string(),
                PROJECT_ID.to_string(),
            ))),
            NativeValue::Param("parameter".into()),
            NativeValue::Idiom(Idiom::default()),
            NativeValue::Table(Table::from("memory_item")),
            NativeValue::Mock(Mock::Count("memory_item".to_string(), 1)),
            parsed_regex,
            parsed_cast,
            parsed_range,
            parsed_edges,
            NativeValue::from(Block::default()),
            NativeValue::from(Future::from(NativeValue::Null)),
            NativeValue::Constant(Constant::MathPi),
            NativeValue::from(Function::Normal("string::lowercase".to_string(), vec![])),
            NativeValue::from(Subquery::Value(NativeValue::Null)),
            NativeValue::from(Expression::default()),
            NativeValue::Query(Query::default()),
            NativeValue::Model(Box::default()),
            parsed_closure,
            parsed_refs,
        ];
        for value in forbidden {
            let mut meter = Meter::new();
            let mut row = None;
            let mut status = DryNativeStatus::default();
            dry_native_value(
                &value,
                0,
                SchemaPath::Unknown,
                TableKind::MemoryItem,
                None,
                &mut meter,
                &mut row,
                &mut status,
            )
            .unwrap();
            assert!(status.unsupported);
            assert!(!status.invalid_response);
            assert_eq!(
                normalize_native_value(value, SchemaPath::Unknown, TableKind::MemoryItem, None,),
                Err(C2AcquisitionError::UnsupportedNativeValue)
            );
        }

        assert_eq!(
            json_value_to_native(
                JsonValue::Number(serde_json::Number::from(u64::MAX)),
                TableKind::MemoryItem,
                None,
            ),
            Err(C2AcquisitionError::InvalidGeneration)
        );
        assert_eq!(
            json_value_to_native(
                JsonValue::String("nul\0string".to_string()),
                TableKind::MemoryItem,
                None,
            ),
            Err(C2AcquisitionError::InvalidGeneration)
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn stored_future_and_block_are_evaluated_and_function_is_denied() {
        use surrealdb_core::sql::{Function, Future};

        for executable in [NativeValue::from(Future::from(NativeValue::Strand(
            test_strand("future-evaluated"),
        )))] {
            let datastore = Datastore::new("memory")
                .await
                .unwrap()
                .with_capabilities(Capabilities::none());
            let session = Session::owner().with_ns("engram").with_db("main");
            let mut prepared = prepare_generation(fixture_generation(Fixture::A)).unwrap();
            let NativeValue::Array(projects) = prepared
                .variables
                .get_mut(TableKind::WorkProject.tag())
                .unwrap()
            else {
                unreachable!();
            };
            let NativeValue::Object(project) = &mut projects.0[0] else {
                unreachable!();
            };
            project.insert("executable_probe".to_string(), executable);
            let responses = datastore
                .process(
                    parse_init_query().unwrap(),
                    &session,
                    Some(prepared.variables),
                )
                .await
                .unwrap();
            assert!(exact_write_outcomes(responses, TABLE_KINDS.len()));
            let mut responses = datastore
                .process(parse_read_query().unwrap(), &session, None)
                .await
                .unwrap();
            let value = responses.pop().unwrap().output().unwrap();
            let tables = native_tables(value).unwrap();
            let (_, projects) = tables
                .tables
                .into_iter()
                .find(|(table, _)| *table == TableKind::WorkProject)
                .unwrap();
            let NativeValue::Object(project) = &projects.0[0] else {
                unreachable!();
            };
            assert!(matches!(
                project.get("executable_probe"),
                Some(NativeValue::Strand(value))
                    if value.as_str() == "future-evaluated"
            ));
        }

        let datastore = Datastore::new("memory")
            .await
            .unwrap()
            .with_capabilities(Capabilities::none());
        let session = Session::owner().with_ns("engram").with_db("main");
        let mut prepared = prepare_generation(fixture_generation(Fixture::A)).unwrap();
        let NativeValue::Array(projects) = prepared
            .variables
            .get_mut(TableKind::WorkProject.tag())
            .unwrap()
        else {
            unreachable!();
        };
        let NativeValue::Object(project) = &mut projects.0[0] else {
            unreachable!();
        };
        project.insert(
            "executable_probe".to_string(),
            NativeValue::from(Future::from(NativeValue::from(Function::Normal(
                "http::get".to_string(),
                vec![NativeValue::Strand(test_strand("https://example.invalid"))],
            )))),
        );
        let responses = datastore
            .process(
                parse_init_query().unwrap(),
                &session,
                Some(prepared.variables),
            )
            .await
            .unwrap();
        assert!(exact_write_outcomes(responses, TABLE_KINDS.len()));
        let mut responses = datastore
            .process(parse_read_query().unwrap(), &session, None)
            .await
            .unwrap();
        assert_eq!(responses.len(), 1);
        let error = responses.pop().unwrap().output().unwrap_err();
        assert!(matches!(
            error,
            surrealdb_core::err::Error::FunctionNotAllowed(name) if name == "http::get"
        ));
        let responses = datastore
            .process(parse_read_query().unwrap(), &session, None)
            .await
            .unwrap();
        assert_eq!(
            exact_read_output(responses),
            Err(C2AcquisitionError::EngineFailure)
        );
    }

    #[test]
    fn entropy_filler_is_once_and_partial_error_zeroizes() {
        let calls = Cell::new(0_usize);
        let first = mint_mac_key_with(|bytes| {
            calls.set(calls.get() + 1);
            bytes.fill(0x11);
            Ok(())
        })
        .unwrap();
        assert_eq!(calls.get(), 1);
        drop(first);

        super::super::DROPPED_TEST_KEY_BYTES.with(|bytes| bytes.set(None));
        let result = mint_mac_key_with(|bytes| {
            calls.set(calls.get() + 1);
            bytes[..7].fill(0x22);
            Err(())
        });
        assert!(matches!(
            result,
            Err(C2AcquisitionError::EntropyUnavailable)
        ));
        assert_eq!(calls.get(), 2);
        super::super::DROPPED_TEST_KEY_BYTES.with(|bytes| {
            assert_eq!(bytes.get(), Some([0_u8; 32]));
        });

        let left = invoke_c2a(
            json_generation(fixture_generation(Fixture::A)).unwrap(),
            mint_mac_key_with(|bytes| {
                calls.set(calls.get() + 1);
                bytes.fill(0x33);
                Ok(())
            })
            .unwrap(),
        )
        .unwrap();
        let right = invoke_c2a(
            json_generation(fixture_generation(Fixture::A)).unwrap(),
            mint_mac_key_with(|bytes| {
                calls.set(calls.get() + 1);
                bytes.fill(0x44);
                Ok(())
            })
            .unwrap(),
        )
        .unwrap();
        assert_eq!(calls.get(), 4);
        let left = serde_json::to_value(left.audit).unwrap();
        let right = serde_json::to_value(right.audit).unwrap();
        assert_ne!(left["store_state_mac"], right["store_state_mac"]);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn valid_complete_generation_round_trips_once() {
        reset_c2a_invocations();
        let validated =
            C2MemoryOwner::<C2Empty>::acquire_complete_generation(fixture_generation(Fixture::A))
                .await
                .unwrap();
        assert_eq!(validated.projects.len(), 1);
        assert_eq!(validated.tasks.len(), 1);
        assert_eq!(validated.repositories.len(), 1);
        assert_eq!(validated.checkouts.len(), 1);
        assert_eq!(validated.components.len(), 1);
        assert_eq!(validated.project_links.len(), 2);
        assert_eq!(validated.memory_items.len(), 2);
        assert_eq!(validated.correction_proposals.len(), 1);
        assert_eq!(c2a_invocations(), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn pre_c2a_failure_never_invokes_c2a() {
        reset_c2a_invocations();
        let mut generation = fixture_generation(Fixture::A);
        generation.target.project_name = "sk-abcdefghijklmnopqrstuvwxyz".to_string();
        assert!(matches!(
            C2MemoryOwner::<C2Empty>::acquire_complete_generation(generation).await,
            Err(C2AcquisitionError::SecretMaterial)
        ));
        assert_eq!(c2a_invocations(), 0);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn complete_a_b_replacement_rounds_never_observe_a_hybrid() {
        reset_c2a_invocations();
        let mut owner = C2MemoryOwner::<C2Empty>::initialize_for_test(InitialFixture::A)
            .await
            .unwrap();
        for round in 0..512 {
            let expected = if round % 2 == 0 {
                Fixture::B
            } else {
                Fixture::A
            };
            owner = owner.replace_for_test(expected).await.unwrap();
            let observed = owner.observe_for_test().await.unwrap();
            assert_eq!(observed.projects.len(), 1);
            assert_eq!(observed.tasks.len(), 1);
            assert_eq!(observed.repositories.len(), 1);
            assert_eq!(observed.checkouts.len(), 1);
            assert_eq!(observed.components.len(), 1);
            assert_eq!(observed.project_links.len(), 2);
            assert_eq!(observed.memory_items.len(), 2);
            assert_eq!(observed.correction_proposals.len(), 1);
            assert_eq!(
                observed.projects[0].description.as_deref(),
                Some(&*format!("project {}", expected.suffix()))
            );
            assert_eq!(
                observed.tasks[0].name,
                format!("task {}", expected.suffix())
            );
            assert_eq!(
                observed.repositories[0].description.as_deref(),
                Some(&*format!("repository {}", expected.suffix()))
            );
        }
        assert_eq!(c2a_invocations(), 512);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn tied_timestamps_and_arbitrary_insertion_order_are_semantically_identical() {
        let owner = C2MemoryOwner::<C2Empty>::initialize_for_test(InitialFixture::Permutation)
            .await
            .unwrap();
        let baseline = owner.read_raw_for_test().await.unwrap();
        let baseline = invoke_c2a(
            baseline,
            mint_mac_key_with(|bytes| {
                bytes.fill(0x44);
                Ok(())
            })
            .unwrap(),
        )
        .unwrap();

        let owner =
            C2MemoryOwner::<C2Empty>::initialize_for_test(InitialFixture::ReversedPermutation)
                .await
                .unwrap();
        let reordered = owner.read_raw_for_test().await.unwrap();
        let reordered = invoke_c2a(
            reordered,
            mint_mac_key_with(|bytes| {
                bytes.fill(0x44);
                Ok(())
            })
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            serde_json::to_value(baseline.audit).unwrap(),
            serde_json::to_value(reordered.audit).unwrap()
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn independent_parallel_stores_and_keys_never_cross_talk() {
        use std::sync::Arc;
        use tokio::sync::Barrier;

        let barrier = Arc::new(Barrier::new(4));
        let mut tasks = Vec::new();
        for fixture in [Fixture::A, Fixture::B, Fixture::A, Fixture::B] {
            let barrier = Arc::clone(&barrier);
            tasks.push(tokio::spawn(async move {
                barrier.wait().await;
                let validated = C2MemoryOwner::<C2Empty>::acquire_complete_generation(
                    fixture_generation(fixture),
                )
                .await
                .unwrap();
                let suffix = fixture.suffix();
                assert_eq!(
                    validated.projects[0].description.as_deref(),
                    Some(if suffix == "A" {
                        "project A"
                    } else {
                        "project B"
                    })
                );
                assert_eq!(validated.tasks[0].name, format!("task {suffix}"));
                assert_eq!(
                    validated.repositories[0].description.as_deref(),
                    Some(if suffix == "A" {
                        "repository A"
                    } else {
                        "repository B"
                    })
                );
                assert!(validated
                    .memory_items
                    .iter()
                    .all(|item| item.title.ends_with(suffix)));
                serde_json::to_value(validated.audit).unwrap()
            }));
        }
        let mut audits = Vec::new();
        for task in tasks {
            audits.push(task.await.unwrap());
        }
        for left in 0..audits.len() {
            for right in left + 1..audits.len() {
                assert_ne!(
                    audits[left]["store_state_mac"],
                    audits[right]["store_state_mac"]
                );
            }
        }
        for audit in &audits {
            assert_eq!(audit["row_counts"]["work_project"], 1);
            assert_eq!(audit["row_counts"]["memory_item"], 2);
            assert_eq!(audit["row_counts"]["project_repository_link"], 2);
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn overlapping_replacement_and_reads_match_complete_a_or_b() {
        use std::sync::Arc;

        reset_c2a_invocations();
        let expected_a = json_generation(fixture_generation(Fixture::A)).unwrap();
        let expected_b = json_generation(fixture_generation(Fixture::B)).unwrap();
        assert!(!raw_matches(&expected_a, &expected_b));
        assert_ne!(
            sorted_rows(&expected_a.memory_item),
            sorted_rows(&expected_b.memory_item)
        );
        assert_ne!(
            sorted_rows(&expected_a.correction_proposal),
            sorted_rows(&expected_b.correction_proposal)
        );
        assert_ne!(
            sorted_rows(&expected_a.work_task),
            sorted_rows(&expected_b.work_task)
        );
        assert_ne!(
            sorted_rows(&expected_a.monorepo_component),
            sorted_rows(&expected_b.monorepo_component)
        );
        assert_ne!(
            sorted_rows(&expected_a.project_repository_link),
            sorted_rows(&expected_b.project_repository_link)
        );

        let mut owner = C2MemoryOwner::<C2Empty>::initialize_for_test(InitialFixture::A)
            .await
            .unwrap();
        let observed_a = owner.read_raw_for_test().await.unwrap();
        assert!(raw_matches(&observed_a, &expected_a));
        assert!(owner.observe_for_test().await.is_ok());
        owner = owner.replace_for_test(Fixture::B).await.unwrap();
        let observed_b = owner.read_raw_for_test().await.unwrap();
        assert!(raw_matches(&observed_b, &expected_b));
        assert!(owner.observe_for_test().await.is_ok());
        owner = owner.replace_for_test(Fixture::A).await.unwrap();

        let C2MemoryOwner {
            datastore,
            session,
            target,
            initial_accounting,
            state: _,
        } = owner;
        let datastore = Arc::new(datastore);
        let session = Arc::new(session);
        let writer_datastore = Arc::clone(&datastore);
        let writer_session = Arc::clone(&session);
        let writer = async move {
            for round in 0..512 {
                let fixture = if round % 2 == 0 {
                    Fixture::B
                } else {
                    Fixture::A
                };
                let prepared = prepare_generation(fixture_generation(fixture))?;
                let responses = writer_datastore
                    .process(
                        parse_replace_query()?,
                        &writer_session,
                        Some(prepared.variables),
                    )
                    .await
                    .map_err(|_| C2AcquisitionError::EngineOutcomeUncertain)?;
                if !exact_write_outcomes(responses, 2 * TABLE_KINDS.len()) {
                    return Err(C2AcquisitionError::EngineOutcomeUncertain);
                }
                tokio::task::yield_now().await;
            }
            Ok::<(), C2AcquisitionError>(())
        };
        let reader_datastore = Arc::clone(&datastore);
        let reader_session = Arc::clone(&session);
        let reader = async {
            for _ in 0..512 {
                let observed = shared_read(
                    &reader_datastore,
                    &reader_session,
                    &target,
                    &initial_accounting,
                )
                .await?;
                if !raw_matches(&observed, &expected_a) && !raw_matches(&observed, &expected_b) {
                    return Err(C2AcquisitionError::InvalidResponse);
                }
                invoke_c2a(
                    observed,
                    mint_mac_key_with(|bytes| {
                        bytes.fill(0x7c);
                        Ok(())
                    })?,
                )?;
                tokio::task::yield_now().await;
            }
            Ok::<(), C2AcquisitionError>(())
        };
        let (writer_result, reader_result) = tokio::join!(writer, reader);
        writer_result.unwrap();
        reader_result.unwrap();
        assert_eq!(c2a_invocations(), 514);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn failed_replacement_rolls_back_complete_prior_generation() {
        reset_c2a_invocations();
        let owner = C2MemoryOwner::<C2Empty>::initialize_for_test(InitialFixture::A)
            .await
            .unwrap();
        let expected = json_generation(fixture_generation(Fixture::A)).unwrap();
        let (raw, observed) = owner.rollback_probe_for_test().await.unwrap();
        assert!(raw_matches(&raw, &expected));
        assert_eq!(
            observed.projects[0].description.as_deref(),
            Some("project A")
        );
        assert_eq!(observed.tasks[0].name, "task A");
        assert_eq!(
            observed.repositories[0].description.as_deref(),
            Some("repository A")
        );
        assert_eq!(c2a_invocations(), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cancellation_at_every_controlled_await_returns_no_authority() {
        use std::time::Duration;

        for seam in 0..=2 {
            reset_c2a_invocations();
            CONTROLLED_CANCEL_AT.with(|target| target.set(Some(seam)));
            let result = tokio::time::timeout(
                Duration::from_millis(250),
                C2MemoryOwner::<C2Empty>::acquire_complete_generation(fixture_generation(
                    Fixture::A,
                )),
            )
            .await;
            CONTROLLED_CANCEL_AT.with(|target| target.set(None));
            assert!(result.is_err(), "controlled seam {seam} returned a value");
            assert_eq!(
                c2a_invocations(),
                0,
                "controlled seam {seam} minted authority"
            );
        }
    }

    #[ignore = "invoked only by the poisoned-environment wrapper"]
    #[tokio::test(flavor = "current_thread")]
    async fn poisoned_environment_child() {
        let validated =
            C2MemoryOwner::<C2Empty>::acquire_complete_generation(fixture_generation(Fixture::A))
                .await
                .expect("poisoned environment must not widen or redirect the closed Memory path");
        assert_eq!(validated.audit.schema_version, 1);
        assert_eq!(validated.projects.len(), 1);
        assert_eq!(validated.projects[0].id.as_uuid().to_string(), PROJECT_ID);
        assert_eq!(
            validated.projects[0].description.as_deref(),
            Some("project A")
        );
        assert_eq!(validated.tasks.len(), 1);
        assert_eq!(validated.tasks[0].name, "task A");
        assert_eq!(validated.repositories.len(), 1);
        assert_eq!(
            validated.repositories[0].description.as_deref(),
            Some("repository A")
        );
        assert_eq!(validated.memory_items.len(), 2);
        assert_eq!(validated.correction_proposals.len(), 1);
        assert_eq!(validated.project_links.len(), 2);
    }

    #[test]
    fn poisoned_environment_process_is_bounded_or_non_authorizing() {
        use std::process::Command;

        let exact = concat!(
            "native_successor_semantic::acquisition::tests::",
            "poisoned_environment_child"
        );
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", exact, "--ignored", "--test-threads=1"])
            .env("SURREAL_CAPS_ALLOW_EXPERIMENTAL", "true")
            .env("SURREAL_PATH", "file:///private/poisoned-store")
            .env("SURREAL_USER", "poisoned-user")
            .env("SURREAL_PASS", "poisoned-password")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "poisoned child returned only a non-authorizing test failure"
        );
        let mut capture = output.stdout;
        capture.extend_from_slice(&output.stderr);
        let capture = String::from_utf8_lossy(&capture);
        for forbidden in [
            INIT_QUERY,
            READ_QUERY,
            "poisoned-password",
            "poisoned-store",
        ] {
            assert!(!capture.contains(forbidden));
        }
    }

    #[test]
    fn errors_are_categorical_and_payload_free_on_display() {
        let errors = [
            C2AcquisitionError::LimitExceeded {
                table_overflow_mask: Some(0x1ff),
            },
            C2AcquisitionError::SecretMaterial,
            C2AcquisitionError::InvalidGeneration,
            C2AcquisitionError::QueryContractInvalid,
            C2AcquisitionError::EngineFailure,
            C2AcquisitionError::EngineOutcomeUncertain,
            C2AcquisitionError::InvalidResponse,
            C2AcquisitionError::UnsupportedNativeValue,
            C2AcquisitionError::EntropyUnavailable,
            C2AcquisitionError::InvalidRecord,
            C2AcquisitionError::InconsistentProjection,
            C2AcquisitionError::IncompleteDeletion,
            C2AcquisitionError::AmbiguousIdentity,
            C2AcquisitionError::ScopeMismatch,
            C2AcquisitionError::AppliedProvenanceUnproven,
        ];
        let expected = [
            "limit_exceeded",
            "secret_material",
            "invalid_generation",
            "query_contract_invalid",
            "engine_failure",
            "engine_outcome_uncertain",
            "invalid_response",
            "unsupported_native_value",
            "entropy_unavailable",
            "invalid_record",
            "inconsistent_projection",
            "incomplete_deletion",
            "ambiguous_identity",
            "scope_mismatch",
            "applied_provenance_unproven",
        ];
        for (error, expected) in errors.into_iter().zip(expected) {
            assert_eq!(error.to_string(), expected);
        }

        let semantic_mappings = [
            (
                C2SemanticError::LimitExceeded {
                    table_overflow_mask: Some(7),
                },
                C2AcquisitionError::LimitExceeded {
                    table_overflow_mask: Some(7),
                },
            ),
            (
                C2SemanticError::SecretMaterial,
                C2AcquisitionError::SecretMaterial,
            ),
            (
                C2SemanticError::InvalidRecord,
                C2AcquisitionError::InvalidRecord,
            ),
            (
                C2SemanticError::InconsistentProjection,
                C2AcquisitionError::InconsistentProjection,
            ),
            (
                C2SemanticError::IncompleteDeletion,
                C2AcquisitionError::IncompleteDeletion,
            ),
            (
                C2SemanticError::AmbiguousIdentity,
                C2AcquisitionError::AmbiguousIdentity,
            ),
            (
                C2SemanticError::ScopeMismatch,
                C2AcquisitionError::ScopeMismatch,
            ),
            (
                C2SemanticError::AppliedProvenanceUnproven,
                C2AcquisitionError::AppliedProvenanceUnproven,
            ),
        ];
        for (semantic, expected) in semantic_mappings {
            assert_eq!(C2AcquisitionError::from(semantic), expected);
        }

        assert_eq!(
            map_engine_failure::<(), _>(Err("private-database-payload")),
            Err(C2AcquisitionError::EngineFailure)
        );
        assert_eq!(
            map_engine_outcome_uncertain::<(), _>(Err("private-write-payload")),
            Err(C2AcquisitionError::EngineOutcomeUncertain)
        );
        assert_eq!(
            require_exact_write_results(vec![Err(())], 1),
            Err(C2AcquisitionError::EngineOutcomeUncertain)
        );
        assert_eq!(
            exact_read_results(vec![Err(())]),
            Err(C2AcquisitionError::EngineFailure)
        );
        assert!(matches!(
            mint_mac_key_with(|bytes| {
                bytes[0] = 0x7f;
                Err(())
            }),
            Err(C2AcquisitionError::EntropyUnavailable)
        ));

        let mut invalid_typed_timestamp = fixture_generation(Fixture::A);
        invalid_typed_timestamp.work_project[0].created_at =
            OffsetDateTime::UNIX_EPOCH.replace_year(-1).unwrap();
        assert!(matches!(
            prepare_generation(invalid_typed_timestamp),
            Err(C2AcquisitionError::InvalidGeneration)
        ));

        let mut invalid_typed_number = fixture_generation(Fixture::A);
        invalid_typed_number.memory_item[0].confidence =
            engram_core::memory::MemoryConfidence::new(f32::NAN);
        assert!(matches!(
            prepare_generation(invalid_typed_number),
            Err(C2AcquisitionError::InvalidGeneration)
        ));

        let collision_generation = |kind: u8| {
            assert!(kind <= 2, "typed collision selector escaped its bound");
            let mut generation = fixture_generation(Fixture::A);
            match kind {
                0 => generation.work_task.push(generation.work_task[0].clone()),
                1 => {
                    let mut project = generation.work_project[0].clone();
                    project.id = engram_core::id::Id::parse(&variant_id(Fixture::A, 118)).unwrap();
                    project.name = "ENGRAM".to_string();
                    generation.work_project.push(project);
                }
                2 => {
                    let mut link = generation.project_repository_link[0].clone();
                    link.id = engram_core::id::Id::parse(&variant_id(Fixture::A, 119)).unwrap();
                    link.project_id = None;
                    link.role = ProjectRepositoryRole::Dependency;
                    generation.project_repository_link.push(link);
                }
                _ => unreachable!(),
            }
            generation
        };
        for kind in 0..=2 {
            let raw = json_generation(collision_generation(kind)).unwrap();
            assert!(strict_record_stage(&raw, &[]).is_ok());
            reset_c2a_invocations();
            assert!(matches!(
                invoke_c2a(
                    raw,
                    mint_mac_key_with(|bytes| {
                        bytes.fill(0x50 + kind);
                        Ok(())
                    })
                    .unwrap(),
                ),
                Err(C2AcquisitionError::InconsistentProjection)
            ));
            assert_eq!(c2a_invocations(), 1);

            reset_c2a_invocations();
            assert!(matches!(
                prepare_generation(collision_generation(kind)),
                Err(C2AcquisitionError::InvalidGeneration)
            ));
            assert_eq!(c2a_invocations(), 0);
        }

        let mut typed_row = json_generation(fixture_generation(Fixture::A))
            .unwrap()
            .work_project
            .remove(0);
        typed_row
            .as_object_mut()
            .unwrap()
            .insert("id".to_string(), JsonValue::String(PROJECT_ID.to_string()));
        assert!(matches!(
            json_row_to_native(TableKind::WorkProject, typed_row),
            Err(C2AcquisitionError::InvalidGeneration)
        ));

        let (_, _, mut invalid_timestamp) = native_outer_from_generation(Fixture::A);
        native_row_mut(&mut invalid_timestamp, TableKind::WorkProject, 0).insert(
            "created_at".to_string(),
            NativeValue::Strand(test_strand("not-a-native-datetime")),
        );
        let (target, accounting, _) = native_outer_from_generation(Fixture::A);
        assert!(matches!(
            validate_collected_value(invalid_timestamp, target, accounting),
            Err(C2AcquisitionError::InvalidResponse)
        ));

        let (_, _, mut invalid_number) = native_outer_from_generation(Fixture::A);
        native_row_mut(&mut invalid_number, TableKind::WorkProject, 0).insert(
            "unknown_decimal".to_string(),
            surrealdb_core::syn::value("1.0dec").unwrap(),
        );
        let (target, accounting, _) = native_outer_from_generation(Fixture::A);
        assert!(matches!(
            validate_collected_value(invalid_number, target, accounting),
            Err(C2AcquisitionError::UnsupportedNativeValue)
        ));

        let (_, _, mut collision) = native_outer_from_generation(Fixture::A);
        let row = native_row_mut(&mut collision, TableKind::WorkProject, 0);
        row.insert(
            "record_id".to_string(),
            NativeValue::Strand(test_strand(PROJECT_ID)),
        );
        let (target, accounting, _) = native_outer_from_generation(Fixture::A);
        assert!(matches!(
            validate_collected_value(collision, target, accounting),
            Err(C2AcquisitionError::InvalidResponse)
        ));

        let mut typed_inconsistent = fixture_generation(Fixture::A);
        typed_inconsistent.target.task_id = Some(variant_id(Fixture::A, 4));
        typed_inconsistent.target.task_name = None;
        let typed_raw = json_generation(typed_inconsistent).unwrap();
        assert!(matches!(
            strict_record_stage(&typed_raw, &[]),
            Err(C2SemanticError::InvalidRecord)
        ));
        let mut typed_inconsistent = fixture_generation(Fixture::A);
        typed_inconsistent.target.task_id = Some(variant_id(Fixture::A, 4));
        typed_inconsistent.target.task_name = None;
        assert!(matches!(
            prepare_generation(typed_inconsistent),
            Err(C2AcquisitionError::InvalidGeneration)
        ));

        let mut inconsistent = json_generation(fixture_generation(Fixture::A)).unwrap();
        inconsistent.git_repository[0]["name_key"] = JsonValue::String("wrong".to_string());
        assert!(matches!(
            invoke_c2a(
                inconsistent,
                mint_mac_key_with(|bytes| {
                    bytes.fill(0x4a);
                    Ok(())
                })
                .unwrap(),
            ),
            Err(C2AcquisitionError::InconsistentProjection)
        ));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn serialized_success_audit_and_all_errors_exclude_raw_canaries() {
        let canaries = [
            "private-c2b1-title",
            "private-c2b1-content",
            "private-c2b1-evidence",
            "private-c2b1-actor",
            "private-c2b1-project-description",
            REMOTE,
            CHECKOUT_PATH,
        ];
        let mut generation = fixture_generation(Fixture::A);
        generation.memory_item[1].title = canaries[0].to_string();
        generation.memory_item[1].content = canaries[1].to_string();
        generation.memory_item[1].evidence[0].target = canaries[2].to_string();
        let mut independent = cumulative_memory(100, 1);
        independent.writer.actor = canaries[3].to_string();
        generation.memory_item.push(independent);
        generation.work_project[0].description = Some(canaries[4].to_string());
        let canonical_digest = super::super::correction_digest(
            &generation.correction_proposal[0],
            &generation.memory_item[0],
            &generation.memory_item[1],
        )
        .unwrap();
        generation.correction_proposal[0].canonical_digest = canonical_digest;
        let validated = C2MemoryOwner::<C2Empty>::acquire_complete_generation(generation)
            .await
            .unwrap();
        let serialized = serde_json::to_string(&validated.audit).unwrap();
        for (index, canary) in canaries.into_iter().enumerate() {
            assert!(
                !serialized.contains(canary),
                "success audit contained raw canary at category {index}"
            );
        }
        assert!(!serialized.contains("_sha256"));

        for error in [
            C2AcquisitionError::LimitExceeded {
                table_overflow_mask: Some(0x01ff),
            },
            C2AcquisitionError::SecretMaterial,
            C2AcquisitionError::InvalidGeneration,
            C2AcquisitionError::QueryContractInvalid,
            C2AcquisitionError::EngineFailure,
            C2AcquisitionError::EngineOutcomeUncertain,
            C2AcquisitionError::InvalidResponse,
            C2AcquisitionError::UnsupportedNativeValue,
            C2AcquisitionError::EntropyUnavailable,
            C2AcquisitionError::InvalidRecord,
            C2AcquisitionError::InconsistentProjection,
            C2AcquisitionError::IncompleteDeletion,
            C2AcquisitionError::AmbiguousIdentity,
            C2AcquisitionError::ScopeMismatch,
            C2AcquisitionError::AppliedProvenanceUnproven,
        ] {
            let output = format!("{error} {error:?}");
            for canary in canaries {
                assert!(!output.contains(canary));
            }
        }
    }
}
