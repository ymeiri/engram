//! Strict document boundary shared by the historical native-pilot entrypoints.

use crate::native_pilot::{NativePilotProtocol, PreparedNativePilot};
use crate::native_stale::NativeStaleSafetyProtocol;
use crate::{EvalError, EvalResult};
use serde::{
    de::{DeserializeOwned, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use serde_json::{Map, Number, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs::{self, File, Metadata, OpenOptions};
use std::io::Read;
use std::path::{Path, PathBuf};

const MAX_NATIVE_DOCUMENT_BYTES: u64 = 4 * 1024 * 1024;

const HISTORICAL_PROTOCOL_KEYS: &[&str] = &[
    "schema_version",
    "pilot_id",
    "cases",
    "arms",
    "repetitions",
    "run_order",
    "codex_min_idle_hours",
    "codex_authentication_mode",
    "resource_budgets",
    "claude_budget_cents",
    "claude_prior_spend_microusd",
    "claude_authorized_ceiling_cents",
    "claude_model",
    "claude_max_turns",
    "requires_explicit_execution_approval",
];

const HISTORICAL_PLAN_KEYS: &[&str] = &[
    "protocol_schema_version",
    "pilot_id",
    "execution_approved",
    "prepared_unix_ms",
    "codex_min_idle_hours",
    "resource_budgets",
    "claude_budget_cents",
    "claude_prior_spend_microusd",
    "claude_authorized_ceiling_cents",
    "claude_model",
    "claude_max_turns",
    "native_memory_references",
    "binaries",
    "runtime_libraries",
    "engram_mcp_contract",
    "evaluation_recovery",
    "execution_recovery",
    "stale_safety",
    "lanes",
];

const HISTORICAL_STALE_PROTOCOL_KEYS: &[&str] = &[
    "schema_version",
    "pilot_id",
    "cases",
    "arms",
    "repetitions",
    "run_order",
    "codex_min_idle_hours",
    "codex_authentication_mode",
    "resource_budgets",
    "claude_budget_cents",
    "claude_prior_spend_microusd",
    "claude_authorized_ceiling_cents",
    "claude_model",
    "claude_max_turns",
    "requires_explicit_execution_approval",
    "stale_safety",
];

#[derive(Debug, Clone, PartialEq, Eq)]
enum NativeDocumentClass {
    Historical,
    Successor { family: String, schema_version: u32 },
}

/// Exact native successor families accepted by the strict document firewall.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeSuccessorFamily {
    #[serde(rename = "native_stale_isolated_v1")]
    StaleIsolatedV1,
    #[serde(rename = "native_correction_v1")]
    CorrectionV1,
}

impl NativeSuccessorFamily {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::StaleIsolatedV1 => "native_stale_isolated_v1",
            Self::CorrectionV1 => "native_correction_v1",
        }
    }

    fn parse_exact(value: &str) -> Option<Self> {
        match value {
            "native_stale_isolated_v1" => Some(Self::StaleIsolatedV1),
            "native_correction_v1" => Some(Self::CorrectionV1),
            _ => None,
        }
    }
}

/// Exact document roles accepted inside a strict native successor envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeSuccessorDocumentKind {
    Protocol,
    RunPlan,
    ExecutionIntent,
    TerminalReceipt,
    Audit,
    Report,
}

impl NativeSuccessorDocumentKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Protocol => "protocol",
            Self::RunPlan => "run_plan",
            Self::ExecutionIntent => "execution_intent",
            Self::TerminalReceipt => "terminal_receipt",
            Self::Audit => "audit",
            Self::Report => "report",
        }
    }

    fn parse_exact(value: &str) -> Option<Self> {
        match value {
            "protocol" => Some(Self::Protocol),
            "run_plan" => Some(Self::RunPlan),
            "execution_intent" => Some(Self::ExecutionIntent),
            "terminal_receipt" => Some(Self::TerminalReceipt),
            "audit" => Some(Self::Audit),
            "report" => Some(Self::Report),
            _ => None,
        }
    }
}

// The typed adapter is intentionally dormant until a closed-world successor payload is frozen.
// Stage B compiles and adversarially tests it without exposing a runnable public consumer.
#[allow(dead_code)]
pub(crate) trait NativeSuccessorPayload: DeserializeOwned {
    const FAMILY: NativeSuccessorFamily;
    const DOCUMENT_KIND: NativeSuccessorDocumentKind;
}

pub(crate) struct LoadedSuccessorPayload<T> {
    pub payload: T,
    authority: SuccessorDocumentAuthority,
}

pub(crate) struct SuccessorDocumentAuthority {
    document: BoundedOwnedDocument,
    pub family: NativeSuccessorFamily,
    #[allow(dead_code)]
    pub schema_version: u32,
    pub document_kind: NativeSuccessorDocumentKind,
    pub sha256: String,
    #[allow(dead_code)]
    pub byte_len: u64,
}

impl<T> LoadedSuccessorPayload<T> {
    pub(crate) fn sha256(&self) -> &str {
        &self.authority.sha256
    }

    pub(crate) fn into_parts(self) -> (T, SuccessorDocumentAuthority) {
        (self.payload, self.authority)
    }
}

impl SuccessorDocumentAuthority {
    pub(crate) fn canonical_path(&self) -> &Path {
        &self.document.canonical_path
    }

    pub(crate) fn revalidate(&self) -> EvalResult<()> {
        self.document
            .revalidate_retained_identity("successor native document")
    }
}

pub(crate) struct SuccessorDocumentStructure {
    pub family: NativeSuccessorFamily,
    pub schema_version: u32,
    pub document_kind: NativeSuccessorDocumentKind,
    pub sha256: String,
    pub byte_len: u64,
}

pub(crate) struct LoadedHistoricalNativePlan {
    pub plan: PreparedNativePilot,
    pub canonical_path: PathBuf,
    pub sha256: String,
}

struct BoundedOwnedDocument {
    file: File,
    bytes: Vec<u8>,
    source_path: PathBuf,
    canonical_path: PathBuf,
    opened_metadata: Metadata,
}

#[allow(dead_code)]
struct LoadedSuccessorNativeDocument {
    document: BoundedOwnedDocument,
    family: NativeSuccessorFamily,
    schema_version: u32,
    document_kind: NativeSuccessorDocumentKind,
    payload: Value,
    sha256: String,
}

#[allow(dead_code)]
impl LoadedSuccessorNativeDocument {
    fn consume<T: NativeSuccessorPayload>(self) -> EvalResult<LoadedSuccessorPayload<T>> {
        if self.family != T::FAMILY || self.document_kind != T::DOCUMENT_KIND {
            return Err(EvalError::Invalid(
                "successor native document discriminator did not match the requested typed payload"
                    .to_string(),
            ));
        }
        self.document
            .revalidate_retained_identity("successor native document")?;
        let byte_len = u64::try_from(self.document.bytes.len()).map_err(|_| {
            EvalError::Invalid(
                "successor native document length exceeded the host u64 boundary".to_string(),
            )
        })?;
        let payload = serde_json::from_value(self.payload)?;
        Ok(LoadedSuccessorPayload {
            payload,
            authority: SuccessorDocumentAuthority {
                document: self.document,
                family: self.family,
                schema_version: self.schema_version,
                document_kind: self.document_kind,
                sha256: self.sha256,
                byte_len,
            },
        })
    }

    fn into_structure(self) -> EvalResult<SuccessorDocumentStructure> {
        self.document
            .revalidate_retained_identity("successor native document")?;
        let byte_len = u64::try_from(self.document.bytes.len()).map_err(|_| {
            EvalError::Invalid(
                "successor native document length exceeded the host u64 boundary".to_string(),
            )
        })?;
        Ok(SuccessorDocumentStructure {
            family: self.family,
            schema_version: self.schema_version,
            document_kind: self.document_kind,
            sha256: self.sha256,
            byte_len,
        })
    }
}

impl BoundedOwnedDocument {
    #[cfg(not(unix))]
    fn revalidate_retained_identity(&self, _label: &str) -> EvalResult<()> {
        Err(EvalError::Invalid(
            "native document loading requires Unix owner, link-count, and no-follow checks"
                .to_string(),
        ))
    }

    #[cfg(unix)]
    fn revalidate_retained_identity(&self, label: &str) -> EvalResult<()> {
        use std::os::unix::fs::MetadataExt;

        let effective_uid = unsafe { libc::geteuid() };
        let handle_metadata = self.file.metadata()?;
        let source_metadata = fs::symlink_metadata(&self.source_path)?;
        let canonical_metadata = fs::symlink_metadata(&self.canonical_path)?;
        let canonical_now = self.source_path.canonicalize()?;
        if canonical_now != self.canonical_path
            || !same_native_document(&self.opened_metadata, &handle_metadata)
            || !same_native_document(&self.opened_metadata, &source_metadata)
            || !same_native_document(&self.opened_metadata, &canonical_metadata)
            || source_metadata.file_type().is_symlink()
            || !source_metadata.is_file()
            || source_metadata.uid() != effective_uid
            || source_metadata.nlink() != 1
            || canonical_metadata.file_type().is_symlink()
            || !canonical_metadata.is_file()
            || canonical_metadata.uid() != effective_uid
            || canonical_metadata.nlink() != 1
        {
            return Err(EvalError::Invalid(format!(
                "{label} identity or safety metadata changed before typed consumption"
            )));
        }
        require_no_extended_acl(&self.file, label)?;
        Ok(())
    }
}

/// A JSON decoder that rejects duplicate keys at every object depth. Deserializing directly into
/// `Value` would otherwise silently retain only the final value for a duplicated key.
struct DuplicateRejectingJson(Value);

impl<'de> Deserialize<'de> for DuplicateRejectingJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(DuplicateRejectingJsonVisitor)
    }
}

struct DuplicateRejectingJsonVisitor;

impl<'de> Visitor<'de> for DuplicateRejectingJsonVisitor {
    type Value = DuplicateRejectingJson;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a JSON value without duplicate object keys")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(DuplicateRejectingJson(Value::Bool(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(DuplicateRejectingJson(Value::Number(Number::from(value))))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(DuplicateRejectingJson(Value::Number(Number::from(value))))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Number::from_f64(value)
            .map(Value::Number)
            .map(DuplicateRejectingJson)
            .ok_or_else(|| E::custom("JSON number was not finite"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_string(value.to_string())
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(DuplicateRejectingJson(Value::String(value)))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(DuplicateRejectingJson(Value::Null))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(DuplicateRejectingJson(Value::Null))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        DuplicateRejectingJson::deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(DuplicateRejectingJson(value)) = sequence.next_element()? {
            values.push(value);
        }
        Ok(DuplicateRejectingJson(Value::Array(values)))
    }

    fn visit_map<A>(self, mut object: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some(key) = object.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(serde::de::Error::custom(format!(
                    "duplicate JSON object key: {key}"
                )));
            }
            let DuplicateRejectingJson(value) = object.next_value()?;
            values.insert(key, value);
        }
        Ok(DuplicateRejectingJson(Value::Object(values)))
    }
}

pub(crate) fn load_historical_native_protocol(path: &Path) -> EvalResult<NativePilotProtocol> {
    Ok(load_historical_native_document(path, HISTORICAL_PROTOCOL_KEYS, "native-pilot protocol")?.0)
}

pub(crate) fn load_historical_native_stale_protocol(
    path: &Path,
) -> EvalResult<NativeStaleSafetyProtocol> {
    Ok(load_historical_native_document(
        path,
        HISTORICAL_STALE_PROTOCOL_KEYS,
        "native stale-safety protocol",
    )?
    .0)
}

pub(crate) fn load_historical_native_plan(path: &Path) -> EvalResult<PreparedNativePilot> {
    Ok(load_historical_native_plan_with_identity(path)?.plan)
}

pub(crate) fn load_historical_native_plan_with_identity(
    path: &Path,
) -> EvalResult<LoadedHistoricalNativePlan> {
    let (plan, document) =
        load_historical_native_document(path, HISTORICAL_PLAN_KEYS, "native-pilot run plan")?;
    Ok(LoadedHistoricalNativePlan {
        plan,
        canonical_path: document.canonical_path,
        sha256: format!("{:x}", Sha256::digest(&document.bytes)),
    })
}

pub(crate) fn read_bounded_owned_text_file(
    path: &Path,
    max_bytes: u64,
    label: &str,
) -> EvalResult<String> {
    let document = read_bounded_owned_document(path, max_bytes, label)?;
    String::from_utf8(document.bytes)
        .map_err(|_| EvalError::Invalid(format!("{label} must contain valid UTF-8")))
}

#[allow(dead_code)]
pub(crate) fn load_successor_native_document<T: NativeSuccessorPayload>(
    path: &Path,
) -> EvalResult<LoadedSuccessorPayload<T>> {
    open_successor_native_document(path)?.consume::<T>()
}

pub(crate) fn inspect_successor_native_document(
    path: &Path,
) -> EvalResult<SuccessorDocumentStructure> {
    open_successor_native_document(path)?.into_structure()
}

fn open_successor_native_document(path: &Path) -> EvalResult<LoadedSuccessorNativeDocument> {
    if !path.is_absolute() {
        return Err(EvalError::Invalid(
            "successor native document path must be absolute and canonical".to_string(),
        ));
    }
    let canonical_before = path.canonicalize()?;
    if canonical_before != path {
        return Err(EvalError::Invalid(
            "successor native document path must contain no aliases or symbolic-link components"
                .to_string(),
        ));
    }

    let document =
        read_bounded_owned_document(path, MAX_NATIVE_DOCUMENT_BYTES, "successor native document")?;
    require_no_extended_acl(&document.file, "successor native document")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        if document.opened_metadata.permissions().mode() & 0o022 != 0 {
            return Err(EvalError::Invalid(
                "successor native document must not be writable by group or other users"
                    .to_string(),
            ));
        }
    }
    if document.canonical_path != canonical_before {
        return Err(EvalError::Invalid(
            "successor native document canonical identity changed before parsing".to_string(),
        ));
    }
    let sha256 = format!("{:x}", Sha256::digest(&document.bytes));
    let mut value = strict_json_value_from_slice(&document.bytes)?;
    let (family, schema_version) = match classify_native_document(&value)? {
        NativeDocumentClass::Historical => {
            return Err(EvalError::Invalid(
                "historical native documents are not accepted by the successor loader".to_string(),
            ));
        }
        NativeDocumentClass::Successor {
            family,
            schema_version,
        } => (family, schema_version),
    };
    let family = NativeSuccessorFamily::parse_exact(&family).ok_or_else(|| {
        EvalError::Invalid("successor native document family is not accepted".to_string())
    })?;
    if schema_version != 1 {
        return Err(EvalError::Invalid(
            "successor native document schema_version must be exactly 1".to_string(),
        ));
    }
    let object = value
        .as_object_mut()
        .expect("successor classification requires a JSON object");
    let payload = object
        .remove("payload")
        .expect("successor classification requires a payload");
    let document_kind = payload
        .as_object()
        .and_then(|object| object.get("document_kind"))
        .and_then(Value::as_str)
        .and_then(NativeSuccessorDocumentKind::parse_exact)
        .ok_or_else(|| {
            EvalError::Invalid(
                "successor native document payload must contain one exact document_kind"
                    .to_string(),
            )
        })?;

    Ok(LoadedSuccessorNativeDocument {
        document,
        family,
        schema_version,
        document_kind,
        payload,
        sha256,
    })
}

fn load_historical_native_document<T: DeserializeOwned>(
    path: &Path,
    allowed_keys: &[&str],
    label: &str,
) -> EvalResult<(T, BoundedOwnedDocument)> {
    let document = read_bounded_owned_document(path, MAX_NATIVE_DOCUMENT_BYTES, label)?;
    let value = strict_json_value_from_slice(&document.bytes)?;
    match classify_native_document(&value)? {
        NativeDocumentClass::Historical => {}
        NativeDocumentClass::Successor {
            family,
            schema_version,
        } => {
            return Err(EvalError::Invalid(format!(
                "successor native document family {family:?} schema {schema_version} is not accepted by the historical {label} loader"
            )))
        }
    }
    reject_unknown_historical_keys(&value, allowed_keys, label)?;
    Ok((serde_json::from_value(value)?, document))
}

fn classify_native_document(value: &Value) -> EvalResult<NativeDocumentClass> {
    let object = value.as_object().ok_or_else(|| {
        EvalError::Invalid("native document root must be a JSON object".to_string())
    })?;
    if !object.contains_key("family") && !object.contains_key("payload") {
        return Ok(NativeDocumentClass::Historical);
    }

    const SUCCESSOR_KEYS: [&str; 3] = ["family", "schema_version", "payload"];
    if object.len() != SUCCESSOR_KEYS.len()
        || SUCCESSOR_KEYS.iter().any(|key| !object.contains_key(*key))
    {
        return Err(EvalError::Invalid(
            "successor native document outer envelope must contain exactly family, schema_version, and payload"
                .to_string(),
        ));
    }
    let family = object
        .get("family")
        .and_then(Value::as_str)
        .filter(|family| !family.is_empty())
        .ok_or_else(|| {
            EvalError::Invalid(
                "successor native document family must be a non-empty string".to_string(),
            )
        })?;
    let schema_version = object
        .get("schema_version")
        .and_then(Value::as_u64)
        .and_then(|version| u32::try_from(version).ok())
        .filter(|version| *version > 0)
        .ok_or_else(|| {
            EvalError::Invalid(
                "successor native document schema_version must be a positive u32".to_string(),
            )
        })?;
    if !object.get("payload").is_some_and(Value::is_object) {
        return Err(EvalError::Invalid(
            "successor native document payload must be a JSON object".to_string(),
        ));
    }
    Ok(NativeDocumentClass::Successor {
        family: family.to_string(),
        schema_version,
    })
}

fn reject_unknown_historical_keys(
    value: &Value,
    allowed_keys: &[&str],
    label: &str,
) -> EvalResult<()> {
    let object = value
        .as_object()
        .expect("native document classification requires an object");
    let allowed = allowed_keys.iter().copied().collect::<BTreeSet<_>>();
    let unknown = object
        .keys()
        .filter(|key| !allowed.contains(key.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    if !unknown.is_empty() {
        return Err(EvalError::Invalid(format!(
            "historical {label} contains unknown top-level keys: {}",
            unknown.join(", ")
        )));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StrictJsonReadError {
    MalformedJson,
    DuplicateKey,
    TrailingData,
}

pub(crate) fn strict_json_value_from_slice_categorized(
    bytes: &[u8],
) -> Result<Value, StrictJsonReadError> {
    strict_json_value_from_slice_with_error(bytes).map_err(|(category, _)| category)
}

fn strict_json_value_from_slice(bytes: &[u8]) -> EvalResult<Value> {
    strict_json_value_from_slice_with_error(bytes).map_err(|(_, error)| EvalError::Json(error))
}

fn strict_json_value_from_slice_with_error(
    bytes: &[u8],
) -> Result<Value, (StrictJsonReadError, serde_json::Error)> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let DuplicateRejectingJson(value) = DuplicateRejectingJson::deserialize(&mut deserializer)
        .map_err(|error| {
            let category = if error.to_string().starts_with("duplicate JSON object key:") {
                StrictJsonReadError::DuplicateKey
            } else {
                StrictJsonReadError::MalformedJson
            };
            (category, error)
        })?;
    deserializer
        .end()
        .map_err(|error| (StrictJsonReadError::TrailingData, error))?;
    Ok(value)
}

#[cfg(not(unix))]
fn read_bounded_owned_document(
    _path: &Path,
    _max_bytes: u64,
    _label: &str,
) -> EvalResult<BoundedOwnedDocument> {
    Err(EvalError::Invalid(
        "native document loading requires Unix owner, link-count, and no-follow checks".to_string(),
    ))
}

#[cfg(unix)]
fn read_bounded_owned_document(
    path: &Path,
    max_bytes: u64,
    label: &str,
) -> EvalResult<BoundedOwnedDocument> {
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};

    let path_metadata = fs::symlink_metadata(path)?;
    let effective_uid = unsafe { libc::geteuid() };
    if path_metadata.file_type().is_symlink()
        || !path_metadata.is_file()
        || path_metadata.uid() != effective_uid
        || path_metadata.nlink() != 1
        || path_metadata.len() == 0
        || path_metadata.len() > max_bytes
    {
        return Err(EvalError::Invalid(format!(
            "{label} must be a non-empty, owner-controlled, single-link regular file no larger than {max_bytes} bytes"
        )));
    }

    let mut options = OpenOptions::new();
    options
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    let mut file = options.open(path)?;
    let opened_metadata = file.metadata()?;
    if !same_native_document(&path_metadata, &opened_metadata)
        || !opened_metadata.is_file()
        || opened_metadata.uid() != effective_uid
        || opened_metadata.nlink() != 1
        || opened_metadata.len() == 0
        || opened_metadata.len() > max_bytes
    {
        return Err(EvalError::Invalid(format!(
            "{label} identity or safety metadata changed before its bounded read"
        )));
    }

    let expected_len = usize::try_from(opened_metadata.len()).map_err(|_| {
        EvalError::Invalid(format!("{label} length cannot fit in memory on this host"))
    })?;
    let mut bytes = vec![0_u8; expected_len];
    file.read_exact(&mut bytes)?;
    let mut trailing = [0_u8; 1];
    if file.read(&mut trailing)? != 0 {
        return Err(EvalError::Invalid(format!(
            "{label} grew during its bounded read"
        )));
    }

    let canonical_path = path.canonicalize()?;
    let canonical_metadata = fs::symlink_metadata(&canonical_path)?;
    let final_handle_metadata = file.metadata()?;
    let final_path_metadata = fs::symlink_metadata(path)?;
    if !same_native_document(&opened_metadata, &final_handle_metadata)
        || !same_native_document(&opened_metadata, &final_path_metadata)
        || !same_native_document(&opened_metadata, &canonical_metadata)
        || final_path_metadata.file_type().is_symlink()
        || !final_path_metadata.is_file()
        || final_path_metadata.uid() != effective_uid
        || final_path_metadata.nlink() != 1
        || canonical_metadata.file_type().is_symlink()
        || !canonical_metadata.is_file()
        || canonical_metadata.uid() != effective_uid
        || canonical_metadata.nlink() != 1
    {
        return Err(EvalError::Invalid(format!(
            "{label} identity or safety metadata changed during its bounded read"
        )));
    }
    Ok(BoundedOwnedDocument {
        file,
        bytes,
        source_path: path.to_path_buf(),
        canonical_path,
        opened_metadata,
    })
}

#[cfg(target_os = "macos")]
pub(crate) fn require_no_extended_acl(file: &File, label: &str) -> EvalResult<()> {
    use std::ffi::c_void;
    use std::os::fd::AsRawFd;

    type Acl = *mut c_void;
    const ACL_TYPE_EXTENDED: libc::c_int = 0x0000_0100;
    const ACL_FIRST_ENTRY: libc::c_int = 0;

    extern "C" {
        fn acl_get_fd_np(fd: libc::c_int, acl_type: libc::c_int) -> Acl;
        fn acl_get_entry(acl: Acl, entry_id: libc::c_int, entry: *mut *mut c_void) -> libc::c_int;
        fn acl_free(value: *mut c_void) -> libc::c_int;
    }

    // SAFETY: the descriptor remains owned by `file`; the returned ACL is released exactly once.
    let acl = unsafe { acl_get_fd_np(file.as_raw_fd(), ACL_TYPE_EXTENDED) };
    if acl.is_null() {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ENOENT) {
            return Ok(());
        }
        return Err(error.into());
    }
    let mut entry = std::ptr::null_mut();
    // SAFETY: `acl` is live and `entry` points to writable storage for one borrowed entry handle.
    let entry_result = unsafe { acl_get_entry(acl, ACL_FIRST_ENTRY, &mut entry) };
    // SAFETY: `acl` was returned by acl_get_fd_np and has not previously been released.
    let free_result = unsafe { acl_free(acl) };
    if entry_result < 0 || free_result != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    if entry_result == 0 {
        return Err(EvalError::Invalid(format!(
            "{label} must not grant access through an extended ACL"
        )));
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn require_no_extended_acl(_file: &File, _label: &str) -> EvalResult<()> {
    Ok(())
}

#[cfg(unix)]
fn same_native_document(left: &std::fs::Metadata, right: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    left.dev() == right.dev()
        && left.ino() == right.ino()
        && left.len() == right.len()
        && left.mtime() == right.mtime()
        && left.mtime_nsec() == right.mtime_nsec()
        && left.ctime() == right.ctime()
        && left.ctime_nsec() == right.ctime_nsec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_pilot::prepare_native_memory_pilot;
    use crate::native_runner::{
        run_native_memory_pilot, NativePilotExecutionApproval, NativePilotRunPhase,
    };
    use std::fs;
    use std::io::Write as _;

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    struct CorrectionProtocolPayload {
        document_kind: String,
        protocol_id: String,
    }

    impl NativeSuccessorPayload for CorrectionProtocolPayload {
        const FAMILY: NativeSuccessorFamily = NativeSuccessorFamily::CorrectionV1;
        const DOCUMENT_KIND: NativeSuccessorDocumentKind = NativeSuccessorDocumentKind::Protocol;
    }

    fn successor_path(root: &Path, name: &str, value: Value) -> PathBuf {
        let canonical_root = root.canonicalize().unwrap();
        write(
            &canonical_root,
            name,
            serde_json::to_vec(&value).unwrap().as_slice(),
        )
    }

    fn write(root: &Path, name: &str, contents: &[u8]) -> std::path::PathBuf {
        let path = root.join(name);
        fs::write(&path, contents).unwrap();
        path
    }

    fn error_text<T>(result: EvalResult<T>) -> String {
        result.err().expect("expected rejection").to_string()
    }

    #[test]
    fn classifies_only_the_exact_successor_outer_envelope() {
        let value = strict_json_value_from_slice(
            br#"{"family":"native_correction_v1","schema_version":1,"payload":{}}"#,
        )
        .unwrap();
        assert_eq!(
            classify_native_document(&value).unwrap(),
            NativeDocumentClass::Successor {
                family: "native_correction_v1".to_string(),
                schema_version: 1,
            }
        );

        for malformed in [
            r#"{"family":"native_correction_v1","schema_version":1}"#,
            r#"{"payload":{},"schema_version":1}"#,
            r#"{"family":"","schema_version":1,"payload":{}}"#,
            r#"{"family":"native_correction_v1","schema_version":0,"payload":{}}"#,
            r#"{"family":"native_correction_v1","schema_version":1,"payload":[]}"#,
            r#"{"family":"native_correction_v1","schema_version":1,"payload":{},"legacy":true}"#,
        ] {
            let value = strict_json_value_from_slice(malformed.as_bytes()).unwrap();
            assert!(classify_native_document(&value).is_err(), "{malformed}");
        }
    }

    #[test]
    fn rejects_duplicate_keys_recursively_before_typed_deserialization() {
        for document in [
            r#"{"schema_version":10,"schema_version":11}"#,
            r#"{"schema_version":10,"cases":[{"id":"first","id":"second"}]}"#,
        ] {
            let error = strict_json_value_from_slice(document.as_bytes()).unwrap_err();
            assert!(error.to_string().contains("duplicate JSON object key"));
        }
    }

    #[test]
    fn categorized_strict_json_parser_returns_only_stable_categories() {
        assert_eq!(
            strict_json_value_from_slice_categorized(br#"{"key":1,"key":2}"#),
            Err(StrictJsonReadError::DuplicateKey)
        );
        assert_eq!(
            strict_json_value_from_slice_categorized(br#"{"key":"#),
            Err(StrictJsonReadError::MalformedJson)
        );
        assert_eq!(
            strict_json_value_from_slice_categorized(br#"{} {}"#),
            Err(StrictJsonReadError::TrailingData)
        );
        assert!(strict_json_value_from_slice_categorized(br#"{"key":1}"#).is_ok());
    }

    #[test]
    fn historical_loader_rejects_unknown_keys_before_missing_typed_fields() {
        let root = tempfile::tempdir().unwrap();
        let path = write(
            root.path(),
            "protocol.json",
            br#"{"schema_version":10,"unexpected_authority":true}"#,
        );
        let error = error_text(load_historical_native_protocol(&path));
        assert!(error.contains("unknown top-level keys: unexpected_authority"));
        assert!(!error.contains("missing field"));

        let plan_path = write(
            root.path(),
            "plan.json",
            br#"{"protocol_schema_version":10,"runtime_libraries":{},"unexpected_authority":true}"#,
        );
        let error = error_text(load_historical_native_plan(&plan_path));
        assert!(error.contains("unknown top-level keys: unexpected_authority"));
        assert!(!error.contains("runtime_libraries"));
        assert!(!error.contains("missing field"));
    }

    #[test]
    fn historical_loaders_reject_successors_without_fallback() {
        let root = tempfile::tempdir().unwrap();
        let path = write(
            root.path(),
            "successor.json",
            br#"{"family":"native_correction_v1","schema_version":1,"payload":{}}"#,
        );
        for error in [
            error_text(load_historical_native_protocol(&path)),
            error_text(load_historical_native_stale_protocol(&path)),
            error_text(load_historical_native_plan(&path)),
        ] {
            assert!(error.contains("is not accepted by the historical"));
            assert!(!error.contains("missing field"));
        }
    }

    #[test]
    fn runner_rejects_successor_legacy_polyglot_without_executing_payload() {
        let root = tempfile::tempdir().unwrap();
        let marker = root.path().join("provider-side-effect");
        let polyglot = serde_json::json!({
            "family": "native_correction_v1",
            "schema_version": 1,
            "payload": {
                "lanes": [{
                    "evaluation_argv": [
                        "/bin/sh",
                        "-c",
                        format!("touch {}", marker.display())
                    ]
                }]
            },
            "pilot_id": "legacy-polyglot",
            "execution_approved": false,
            "lanes": []
        });
        let path = write(
            root.path(),
            "polyglot.json",
            serde_json::to_string(&polyglot).unwrap().as_bytes(),
        );
        let error = error_text(run_native_memory_pilot(
            &path,
            NativePilotRunPhase::Evaluation,
            NativePilotExecutionApproval {
                provider_execution: true,
                claude_budget_cents: Some(0),
            },
        ));
        assert!(error.contains("outer envelope must contain exactly"));
        assert!(!marker.exists());
    }

    #[tokio::test]
    async fn preparation_rejects_malformed_successor_before_creating_output() {
        let root = tempfile::tempdir().unwrap();
        let protocol = write(
            root.path(),
            "malformed-successor.json",
            br#"{"family":"native_correction_v1","schema_version":1}"#,
        );
        let output = root.path().join("output-must-remain-absent");
        let missing_binary = root.path().join("binary-must-not-be-inspected");
        let error = error_text(
            prepare_native_memory_pilot(
                &protocol,
                &output,
                &missing_binary,
                &missing_binary,
                &missing_binary,
            )
            .await,
        );
        assert!(error.contains("outer envelope must contain exactly"));
        assert!(!output.exists());
    }

    #[cfg(unix)]
    #[test]
    fn bounded_reader_rejects_symbolic_and_hard_links() {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir().unwrap();
        let original = write(root.path(), "original.json", br#"{"schema_version":10}"#);
        let symbolic = root.path().join("symbolic.json");
        symlink(&original, &symbolic).unwrap();
        assert!(
            read_bounded_owned_document(&symbolic, MAX_NATIVE_DOCUMENT_BYTES, "test document")
                .is_err()
        );

        let hard = root.path().join("hard.json");
        fs::hard_link(&original, &hard).unwrap();
        assert!(
            read_bounded_owned_document(&original, MAX_NATIVE_DOCUMENT_BYTES, "test document")
                .is_err()
        );
        assert!(
            read_bounded_owned_document(&hard, MAX_NATIVE_DOCUMENT_BYTES, "test document").is_err()
        );
    }

    #[test]
    fn bounded_reader_rejects_empty_and_oversized_documents() {
        let root = tempfile::tempdir().unwrap();
        let empty = write(root.path(), "empty.json", b"");
        assert!(
            read_bounded_owned_document(&empty, MAX_NATIVE_DOCUMENT_BYTES, "test document")
                .is_err()
        );

        let oversized = write(
            root.path(),
            "oversized.json",
            &vec![b' '; usize::try_from(MAX_NATIVE_DOCUMENT_BYTES + 1).unwrap()],
        );
        assert!(read_bounded_owned_document(
            &oversized,
            MAX_NATIVE_DOCUMENT_BYTES,
            "test document"
        )
        .is_err());
    }

    #[test]
    fn successor_loader_consumes_exact_typed_payload_and_retains_identity() {
        let root = tempfile::tempdir().unwrap();
        let value = serde_json::json!({
            "family": "native_correction_v1",
            "schema_version": 1,
            "payload": {
                "document_kind": "protocol",
                "protocol_id": "correction-one"
            }
        });
        let bytes = serde_json::to_vec(&value).unwrap();
        let path = successor_path(root.path(), "protocol.json", value);
        let loaded = load_successor_native_document::<CorrectionProtocolPayload>(&path).unwrap();
        assert_eq!(loaded.authority.family, NativeSuccessorFamily::CorrectionV1);
        assert_eq!(loaded.authority.schema_version, 1);
        assert_eq!(
            loaded.authority.document_kind,
            NativeSuccessorDocumentKind::Protocol
        );
        assert_eq!(loaded.payload.document_kind, "protocol");
        assert_eq!(loaded.payload.protocol_id, "correction-one");
        assert_eq!(loaded.authority.canonical_path(), path);
        assert_eq!(loaded.authority.byte_len, bytes.len() as u64);
        assert_eq!(
            loaded.authority.sha256,
            format!("{:x}", Sha256::digest(bytes))
        );
    }

    #[test]
    fn successor_structure_accepts_only_two_families_one_version_and_exact_kinds() {
        let root = tempfile::tempdir().unwrap();
        for family in ["native_stale_isolated_v1", "native_correction_v1"] {
            for kind in [
                "protocol",
                "run_plan",
                "execution_intent",
                "terminal_receipt",
                "audit",
                "report",
            ] {
                let path = successor_path(
                    root.path(),
                    &format!("{}-{kind}.json", family.replace('_', "-")),
                    serde_json::json!({
                        "family": family,
                        "schema_version": 1,
                        "payload": {"document_kind": kind, "future_typed_field": true}
                    }),
                );
                let structure = inspect_successor_native_document(&path).unwrap();
                assert_eq!(structure.schema_version, 1);
                assert_eq!(structure.document_kind.as_str(), kind);
                assert!(!structure.sha256.is_empty());
                assert!(structure.byte_len > 0);
            }
        }

        for (name, value) in [
            (
                "unknown-family",
                serde_json::json!({"family":"native_other_v1","schema_version":1,"payload":{"document_kind":"protocol"}}),
            ),
            (
                "wrong-version",
                serde_json::json!({"family":"native_correction_v1","schema_version":2,"payload":{"document_kind":"protocol"}}),
            ),
            (
                "unknown-kind",
                serde_json::json!({"family":"native_correction_v1","schema_version":1,"payload":{"document_kind":"execute"}}),
            ),
            (
                "malformed-kind-marker",
                serde_json::json!({"family":"native_correction_v1","schema_version":1,"payload":{"document_kind":"protocol\n"}}),
            ),
        ] {
            let path = successor_path(root.path(), &format!("{name}.json"), value);
            assert!(inspect_successor_native_document(&path).is_err(), "{name}");
        }
    }

    #[test]
    fn successor_discriminator_rejects_before_typed_payload_errors() {
        let root = tempfile::tempdir().unwrap();
        for (name, family, kind) in [
            ("wrong-family", "native_stale_isolated_v1", "protocol"),
            ("wrong-kind", "native_correction_v1", "run_plan"),
        ] {
            let path = successor_path(
                root.path(),
                &format!("{name}.json"),
                serde_json::json!({
                    "family": family,
                    "schema_version": 1,
                    "payload": {"document_kind": kind}
                }),
            );
            let error = error_text(load_successor_native_document::<CorrectionProtocolPayload>(
                &path,
            ));
            assert!(error.contains("discriminator did not match"), "{error}");
            assert!(!error.contains("protocol_id"), "{error}");
        }
    }

    #[test]
    fn successor_loader_rejects_historical_extra_outer_and_recursive_duplicates() {
        let root = tempfile::tempdir().unwrap();
        let canonical_root = root.path().canonicalize().unwrap();
        for (name, bytes) in [
            ("historical.json", br#"{"schema_version":15,"pilot_id":"legacy"}"#.as_slice()),
            ("extra.json", br#"{"family":"native_correction_v1","schema_version":1,"payload":{"document_kind":"protocol"},"legacy":true}"#.as_slice()),
            ("duplicate.json", br#"{"family":"native_correction_v1","schema_version":1,"payload":{"document_kind":"protocol","nested":{"key":1,"key":2}}}"#.as_slice()),
            ("trailing.json", br#"{"family":"native_correction_v1","schema_version":1,"payload":{"document_kind":"protocol"}} true"#.as_slice()),
        ] {
            let path = write(&canonical_root, name, bytes);
            assert!(inspect_successor_native_document(&path).is_err(), "{name}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn successor_loader_rejects_noncanonical_parent_links_and_hardlinks() {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir().unwrap();
        let canonical_root = root.path().canonicalize().unwrap();
        let real_parent = canonical_root.join("real");
        fs::create_dir(&real_parent).unwrap();
        let original = successor_path(
            &real_parent,
            "protocol.json",
            serde_json::json!({"family":"native_correction_v1","schema_version":1,"payload":{"document_kind":"protocol"}}),
        );
        let linked_parent = canonical_root.join("linked");
        symlink(&real_parent, &linked_parent).unwrap();
        let through_parent_link = linked_parent.join("protocol.json");
        assert!(inspect_successor_native_document(&through_parent_link).is_err());

        let hard = canonical_root.join("hard.json");
        fs::hard_link(&original, &hard).unwrap();
        assert!(inspect_successor_native_document(&original).is_err());
        assert!(inspect_successor_native_document(&hard).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn successor_loader_rejects_group_or_other_writable_documents() {
        use std::os::unix::fs::PermissionsExt;

        for mode in [0o620, 0o602, 0o666] {
            let root = tempfile::tempdir().unwrap();
            let path = successor_path(
                root.path(),
                "protocol.json",
                serde_json::json!({
                    "family":"native_correction_v1",
                    "schema_version":1,
                    "payload":{"document_kind":"protocol"}
                }),
            );
            fs::set_permissions(&path, fs::Permissions::from_mode(mode)).unwrap();
            let error = inspect_successor_native_document(&path)
                .err()
                .expect("unsafe permissions must be rejected")
                .to_string();
            assert!(
                error.contains("writable by group or other"),
                "{mode:o}: {error}"
            );
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn successor_loader_rejects_extended_acl_access() {
        use std::process::Command;

        let root = tempfile::tempdir().unwrap();
        let path = successor_path(
            root.path(),
            "protocol.json",
            serde_json::json!({
                "family":"native_correction_v1",
                "schema_version":1,
                "payload":{"document_kind":"protocol"}
            }),
        );
        let status = Command::new("/bin/chmod")
            .args(["+a", "everyone allow write", path.to_str().unwrap()])
            .status()
            .unwrap();
        assert!(status.success());
        let error = inspect_successor_native_document(&path)
            .err()
            .expect("extended ACL must be rejected")
            .to_string();
        assert!(error.contains("extended ACL"), "{error}");
    }

    #[test]
    fn retained_successor_handle_rejects_replace_truncate_and_growth_before_consumption() {
        for mutation in ["replace", "truncate", "grow"] {
            let root = tempfile::tempdir().unwrap();
            let path = successor_path(
                root.path(),
                "protocol.json",
                serde_json::json!({
                    "family":"native_correction_v1",
                    "schema_version":1,
                    "payload":{"document_kind":"protocol","protocol_id":"one"}
                }),
            );
            let loaded = open_successor_native_document(&path).unwrap();
            match mutation {
                "replace" => {
                    fs::rename(&path, path.with_extension("moved")).unwrap();
                    fs::write(&path, b"replacement").unwrap();
                }
                "truncate" => {
                    OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .open(&path)
                        .unwrap();
                }
                "grow" => {
                    OpenOptions::new()
                        .append(true)
                        .open(&path)
                        .unwrap()
                        .write_all(b" ")
                        .unwrap();
                }
                _ => unreachable!(),
            }
            assert!(
                loaded.consume::<CorrectionProtocolPayload>().is_err(),
                "{mutation}"
            );
        }
    }
}
