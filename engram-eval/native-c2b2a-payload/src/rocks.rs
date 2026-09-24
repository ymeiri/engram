//! Exact stock-Core/Rocks lifecycle boundary for C2B2a.
//!
//! This module intentionally has no configurable datastore path, namespace,
//! database, query, or fixture input.  Kernel lock attribution, process
//! liveness, descriptor scans, reaping, and protocol sequencing remain the
//! supervisor's responsibility.

use std::collections::BTreeMap;

use surrealdb_core::dbs::{Capabilities, Response, Session};
use surrealdb_core::kvs::Datastore;
use surrealdb_core::sql::{Array, Query, Value};

use crate::contract::{
    environment_is_exact, ErrorCategory, BASE_ENVIRONMENT, ROCKS_ENVIRONMENT, STORE_DATABASE,
    STORE_NAMESPACE, STORE_ORIGIN, TABLE_NAMES,
};
use crate::fixtures::{
    parse_insert_query, parse_read_query, require_reviewed_build_record, CalibrationExpectedState,
    CalibrationFixture, FixtureError, ReviewedFixtureBuildRecord,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RocksError {
    ReviewedBuildRecordMissing,
    EnvironmentMismatch,
    QueryContractInvalid,
    LimitExceeded,
    InvalidFixture,
    StoreOpenFailure,
    OutcomeUncertain,
    LockClassifierUncalibrated,
    ContenderOpened,
    ContenderFailure,
    ReleaseOpenFailure,
    ReopenFailure,
    EngineReadFailure,
    CanonicalMismatch,
}

impl RocksError {
    pub const fn pre_dispatch_category(self) -> Option<ErrorCategory> {
        match self {
            Self::ReviewedBuildRecordMissing | Self::QueryContractInvalid => {
                Some(ErrorCategory::StaticFirewall)
            }
            Self::EnvironmentMismatch => Some(ErrorCategory::GuestConfiguration),
            Self::LimitExceeded => Some(ErrorCategory::LimitExceeded),
            Self::InvalidFixture => Some(ErrorCategory::InvalidFixture),
            Self::StoreOpenFailure => Some(ErrorCategory::StoreOpenFailure),
            _ => None,
        }
    }

    /// Once the writer Continue is fully delivered and until its exact commit
    /// acknowledgement, the frozen classifier ignores the apparent cause.
    pub const fn dispatched_unacknowledged_category(self) -> ErrorCategory {
        let _ = self;
        ErrorCategory::OutcomeUncertain
    }

    pub const fn post_ack_current_stage_category(self) -> Option<ErrorCategory> {
        match self {
            Self::LimitExceeded => Some(ErrorCategory::LimitExceeded),
            Self::ContenderOpened => Some(ErrorCategory::ExclusivityBroken),
            Self::LockClassifierUncalibrated | Self::ContenderFailure => {
                Some(ErrorCategory::ContenderFailure)
            }
            Self::ReleaseOpenFailure => Some(ErrorCategory::ReleaseUnproven),
            Self::ReopenFailure => Some(ErrorCategory::ReopenFailure),
            Self::EngineReadFailure => Some(ErrorCategory::EngineReadFailure),
            Self::CanonicalMismatch => Some(ErrorCategory::CanonicalMismatch),
            _ => None,
        }
    }
}

impl From<FixtureError> for RocksError {
    fn from(error: FixtureError) -> Self {
        match error {
            FixtureError::ReviewedBuildRecordMissing => Self::ReviewedBuildRecordMissing,
            FixtureError::QueryLiteralMismatch
            | FixtureError::QueryParseFailure
            | FixtureError::QueryAstMismatch => Self::QueryContractInvalid,
            FixtureError::LengthOverflow
            | FixtureError::TableLimitExceeded
            | FixtureError::ValueDepthExceeded
            | FixtureError::CanonicalSizeLimitExceeded => Self::LimitExceeded,
            _ => Self::InvalidFixture,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockClassifierState {
    Uncalibrated,
}

/// The accepted freeze supplies no target/binary-specific contender bytes.
/// No substring, case-folded, normalized, or path-substituted fallback exists.
pub const LOCK_CLASSIFIER_STATE: LockClassifierState = LockClassifierState::Uncalibrated;

pub fn classify_contender_error(_complete_utf8_bytes: &[u8]) -> Result<(), RocksError> {
    Err(RocksError::LockClassifierUncalibrated)
}

/// Capability proving that the complete environment and reviewed fixture build
/// record were checked before the first Core/Rocks operation.
pub struct RuntimeAuthority {
    _reviewed_build: ReviewedFixtureBuildRecord,
}

pub fn attest_runtime_environment<'a>(
    entries_in_environ_order: impl IntoIterator<Item = &'a str>,
) -> Result<(), RocksError> {
    if environment_is_exact(entries_in_environ_order) {
        Ok(())
    } else {
        Err(RocksError::EnvironmentMismatch)
    }
}

pub fn attest_current_runtime_environment() -> Result<(), RocksError> {
    let observed =
        std::fs::read("/proc/self/environ").map_err(|_| RocksError::EnvironmentMismatch)?;
    if observed == expected_runtime_environ_bytes() {
        Ok(())
    } else {
        Err(RocksError::EnvironmentMismatch)
    }
}

pub fn establish_runtime_authority() -> Result<RuntimeAuthority, RocksError> {
    // The frozen pre-dispatch precedence evaluates guest configuration before
    // the static source/build firewall.  Neither check references Core.
    attest_current_runtime_environment()?;
    let reviewed_build = require_reviewed_build_record()?;
    Ok(RuntimeAuthority {
        _reviewed_build: reviewed_build,
    })
}

pub fn expected_runtime_environment() -> Vec<&'static str> {
    BASE_ENVIRONMENT
        .into_iter()
        .chain(ROCKS_ENVIRONMENT)
        .collect()
}

pub fn expected_runtime_environ_bytes() -> Vec<u8> {
    let entries = expected_runtime_environment();
    let mut bytes = Vec::with_capacity(entries.iter().map(|entry| entry.len() + 1).sum());
    for entry in entries {
        bytes.extend_from_slice(entry.as_bytes());
        bytes.push(0);
    }
    bytes
}

struct StoreHandles {
    datastore: Datastore,
    session: Session,
}

impl StoreHandles {
    async fn open() -> Result<Self, ()> {
        let datastore = Datastore::new(STORE_ORIGIN)
            .await
            .map_err(|_| ())?
            .with_capabilities(Capabilities::none());
        let session = Session::owner()
            .with_ns(STORE_NAMESPACE)
            .with_db(STORE_DATABASE);
        Ok(Self { datastore, session })
    }
}

pub struct WriterOpen {
    handles: StoreHandles,
    insert_query: Query,
}

pub struct WriterCommitted {
    _handles: StoreHandles,
    case_id: u16,
}

pub struct WriterHandlesDropped {
    case_id: u16,
}

impl WriterHandlesDropped {
    pub const fn case_id(&self) -> u16 {
        self.case_id
    }
}

pub async fn open_writer(_authority: &RuntimeAuthority) -> Result<WriterOpen, RocksError> {
    // This entrypoint is called only after full delivery of the writer's
    // dispatch-authorizing Continue.  Every failure until the exact commit
    // outcome is acknowledged is therefore uncertain.
    let insert_query = parse_insert_query().map_err(|_| RocksError::OutcomeUncertain)?;
    let handles = StoreHandles::open()
        .await
        .map_err(|_| RocksError::OutcomeUncertain)?;
    Ok(WriterOpen {
        handles,
        insert_query,
    })
}

impl WriterOpen {
    pub async fn commit(self, fixture: CalibrationFixture) -> Result<WriterCommitted, RocksError> {
        let case_id = fixture.case().case_id;
        let variables = fixture.into_variables();
        if !variables_are_exact(&variables) {
            return Err(RocksError::OutcomeUncertain);
        }
        let Self {
            handles,
            insert_query,
        } = self;
        let responses = handles
            .datastore
            .process(insert_query, &handles.session, Some(variables))
            .await
            .map_err(|_| RocksError::OutcomeUncertain)?;
        require_exact_write_outcomes(responses)?;
        Ok(WriterCommitted {
            _handles: handles,
            case_id,
        })
    }
}

impl WriterCommitted {
    pub const fn case_id(&self) -> u16 {
        self.case_id
    }

    /// Drops every datastore-owning handle.  The collector process remains
    /// alive so the supervisor can prove descriptor and kernel-lock release.
    pub fn drop_handles(self) -> WriterHandlesDropped {
        let case_id = self.case_id;
        drop(self);
        WriterHandlesDropped { case_id }
    }
}

pub struct ReleaseProbeOpen {
    _handles: StoreHandles,
}

pub struct ReleaseProbeHandlesDropped {
    _private: (),
}

pub async fn open_release_probe(
    _authority: &RuntimeAuthority,
) -> Result<ReleaseProbeOpen, RocksError> {
    let handles = StoreHandles::open()
        .await
        .map_err(|_| RocksError::ReleaseOpenFailure)?;
    Ok(ReleaseProbeOpen { _handles: handles })
}

impl ReleaseProbeOpen {
    /// As with the writer, this is only a handle-release operation.  It makes
    /// no claim about hidden close/flush errors.
    pub fn drop_handles(self) -> ReleaseProbeHandlesDropped {
        drop(self);
        ReleaseProbeHandlesDropped { _private: () }
    }
}

pub struct ReaderOpen {
    handles: StoreHandles,
    read_query: Query,
}

pub struct CanonicalReadVerified {
    case_id: u16,
}

impl CanonicalReadVerified {
    pub const fn case_id(&self) -> u16 {
        self.case_id
    }
}

pub async fn open_reader(_authority: &RuntimeAuthority) -> Result<ReaderOpen, RocksError> {
    let read_query = parse_read_query().map_err(|_| RocksError::EngineReadFailure)?;
    let handles = StoreHandles::open()
        .await
        .map_err(|_| RocksError::ReopenFailure)?;
    Ok(ReaderOpen {
        handles,
        read_query,
    })
}

impl ReaderOpen {
    pub async fn read_and_validate(
        self,
        expected_state: CalibrationExpectedState,
    ) -> Result<CanonicalReadVerified, RocksError> {
        let case_id = expected_state.case_id();
        let Self {
            handles,
            read_query,
        } = self;
        let responses = handles
            .datastore
            .process(read_query, &handles.session, None)
            .await
            .map_err(|_| RocksError::EngineReadFailure)?;
        let output = exact_read_output(responses)?;
        let matches = expected_state
            .compare_read_value(&output)
            .map_err(|error| match error {
                FixtureError::TableLimitExceeded
                | FixtureError::ValueDepthExceeded
                | FixtureError::CanonicalSizeLimitExceeded => RocksError::LimitExceeded,
                _ => RocksError::EngineReadFailure,
            })?;
        if !matches {
            return Err(RocksError::CanonicalMismatch);
        }
        Ok(CanonicalReadVerified { case_id })
    }
}

pub async fn expect_contender_lock_rejection(
    _authority: &RuntimeAuthority,
) -> Result<(), RocksError> {
    match Datastore::new(STORE_ORIGIN).await {
        Ok(datastore) => {
            drop(datastore);
            Err(RocksError::ContenderOpened)
        }
        Err(error) => {
            let rendered = error.to_string();
            classify_contender_error(rendered.as_bytes())
        }
    }
}

fn variables_are_exact(variables: &BTreeMap<String, Value>) -> bool {
    variables.len() == TABLE_NAMES.len()
        && TABLE_NAMES
            .into_iter()
            .all(|table| matches!(variables.get(table), Some(Value::Array(_))))
}

fn require_exact_write_outcomes(responses: Vec<Response>) -> Result<(), RocksError> {
    if responses.len() != TABLE_NAMES.len() {
        return Err(RocksError::OutcomeUncertain);
    }
    for response in responses {
        match response.output() {
            Ok(Value::Array(values)) if values.is_empty() => {}
            _ => return Err(RocksError::OutcomeUncertain),
        }
    }
    Ok(())
}

fn exact_read_output(mut responses: Vec<Response>) -> Result<Value, RocksError> {
    if responses.len() != 1 {
        return Err(RocksError::EngineReadFailure);
    }
    responses
        .pop()
        .ok_or(RocksError::EngineReadFailure)?
        .output()
        .map_err(|_| RocksError::EngineReadFailure)
}

// Keep imports and type checks tied to the exact native array representation.
const _: fn(Array) -> Value = Value::Array;
