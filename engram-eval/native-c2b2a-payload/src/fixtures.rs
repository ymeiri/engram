//! Closed deterministic structural fixtures for the C2B2a Rocks calibration.
//!
//! This is intentionally not Engram's production projection.  The only input
//! accepted by this module is one of the twenty frozen calibration case IDs.
//! Canonical bytes stay inside the collector; the host receives only protocol
//! outcomes and digests.

use std::collections::BTreeMap;

use surrealdb_core::sql::{
    Array, Data, Field, Id, Number, Object, Output, Query, Statement, Subquery, Thing, Uuid, Value,
};

use crate::contract::{
    sha256, CalibrationCase, Digest32, FixtureShape, QueryIdentity, CALIBRATION_CASES,
    INSERT_QUERY, INSERT_QUERY_IDENTITY, PROFILE_ID, READ_QUERY, READ_QUERY_IDENTITY,
    RESOURCE_PROFILE_V1, ROCKS_ENVIRONMENT_SHA256, TABLE_NAMES,
};

pub const CANONICAL_FIXTURE_MAGIC: [u8; 8] = *b"ENGFX001";
pub const CANONICAL_FIXTURE_VERSION: u16 = 1;
pub const RAW_CANONICAL_BYTES: usize = 2_097_152;
const MAX_CANONICAL_ENCODING_DEPTH: usize = 33;
const MAX_TEMPLATE_DEPTH: usize = 35;

pub const TABLE_ROW_CAPS: [usize; 9] = [64, 32, 32, 8, 64, 16, 32, 64, 64];

const CANONICAL_AST_MAGIC: [u8; 8] = *b"ENGAST01";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixtureError {
    UnknownCase,
    QueryLiteralMismatch,
    QueryParseFailure,
    QueryAstMismatch,
    InvalidUuid,
    LengthOverflow,
    UnsupportedNativeValue,
    InvalidReadShape,
    InvalidRecordId,
    TableLimitExceeded,
    ValueDepthExceeded,
    CanonicalSizeLimitExceeded,
    RawCanonicalSizeMismatch,
    ReviewedBuildRecordMissing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryKind {
    Insert,
    Read,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanonicalAstIdentity {
    pub kind: QueryKind,
    pub byte_len: usize,
    pub sha256: Digest32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixtureBuildRecordCandidate {
    pub fixture_source_sha256: Digest32,
    pub ordered_case_manifest_sha256: Digest32,
    pub emitted_fixture_stream_sha256: Digest32,
    pub insert_ast: CanonicalAstIdentity,
    pub read_ast: CanonicalAstIdentity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixtureBuildIdentityState {
    Unfrozen,
}

pub const FIXTURE_BUILD_IDENTITY_STATE: FixtureBuildIdentityState =
    FixtureBuildIdentityState::Unfrozen;

/// Capability required by the datastore layer before it may touch Core.
///
/// The accepted design requires an external reviewed build record containing
/// the source, fixture stream, manifest, and parsed-AST identities.  Those
/// values were not present in the design freeze, so this type deliberately has
/// no public constructor yet.
pub struct ReviewedFixtureBuildRecord {
    _private: (),
}

pub fn require_reviewed_build_record() -> Result<ReviewedFixtureBuildRecord, FixtureError> {
    Err(FixtureError::ReviewedBuildRecordMissing)
}

pub struct CalibrationFixture {
    case: &'static CalibrationCase,
    tables: [Array; TABLE_NAMES.len()],
}

/// Bounded, collector-private expected state for the fresh-reader comparison.
/// It is deliberately neither cloneable nor serializable and exposes no byte
/// accessor, so the datastore boundary can consume it without making the
/// canonical fixture a protocol payload.
pub struct CalibrationExpectedState {
    case_id: u16,
    canonical_bytes: Vec<u8>,
    template: Value,
}

impl CalibrationExpectedState {
    pub const fn case_id(&self) -> u16 {
        self.case_id
    }

    pub fn compare_read_value(&self, observed: &Value) -> Result<bool, FixtureError> {
        validate_value_shape(observed, &self.template, 0)?;
        Ok(
            canonical_read_bytes_with_limit(observed, self.canonical_bytes.len())?
                == self.canonical_bytes,
        )
    }
}

pub fn calibration_expected_state(case_id: u16) -> Result<CalibrationExpectedState, FixtureError> {
    let fixture = calibration_fixture(case_id)?;
    let canonical_bytes = fixture.canonical_bytes()?;
    let template = fixture.read_output_value();
    Ok(CalibrationExpectedState {
        case_id,
        canonical_bytes,
        template,
    })
}

impl CalibrationFixture {
    pub const fn case(&self) -> &'static CalibrationCase {
        self.case
    }

    pub fn rows(&self, table_index: usize) -> Option<&Array> {
        self.tables.get(table_index)
    }

    pub fn row_counts(&self) -> [usize; TABLE_NAMES.len()] {
        std::array::from_fn(|index| self.tables[index].len())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, FixtureError> {
        canonical_table_bytes(&self.tables)
    }

    pub fn read_output_value(&self) -> Value {
        let object = TABLE_NAMES
            .into_iter()
            .enumerate()
            .map(|(index, table)| (table.to_owned(), Value::Array(self.tables[index].clone())))
            .collect::<BTreeMap<_, _>>();
        Value::Object(Object::from(object))
    }

    pub fn into_variables(self) -> BTreeMap<String, Value> {
        TABLE_NAMES
            .into_iter()
            .zip(self.tables)
            .map(|(table, rows)| (table.to_owned(), Value::Array(rows)))
            .collect()
    }
}

pub fn calibration_fixture(case_id: u16) -> Result<CalibrationFixture, FixtureError> {
    let case = CALIBRATION_CASES
        .iter()
        .find(|case| case.case_id == case_id)
        .ok_or(FixtureError::UnknownCase)?;
    let seed = match case_id {
        19 => 1,
        20 => 2,
        value => value,
    };
    let mut tables = empty_tables();

    match case.shape {
        FixtureShape::SparseA => build_sparse_a(&mut tables)?,
        FixtureShape::SparseB => build_sparse_b(&mut tables)?,
        FixtureShape::NineEmptyTableArrays => {}
        FixtureShape::OneRowPerTable => {
            for (table_index, table_rows) in tables.iter_mut().enumerate() {
                table_rows.push(fixed_row(seed, table_index, 0, RowFlavor::Ordinary)?);
            }
        }
        FixtureShape::RowsInMemoryItem(count) => {
            fill_ordinary(&mut tables, seed, 0, usize::from(count))?
        }
        FixtureShape::RowsInCorrectionProposal(count) => {
            fill_ordinary(&mut tables, seed, 1, usize::from(count))?
        }
        FixtureShape::RowsInForgetReceipt(count) => {
            fill_ordinary(&mut tables, seed, 2, usize::from(count))?
        }
        FixtureShape::RowsInWorkProject(count) => {
            fill_ordinary(&mut tables, seed, 3, usize::from(count))?
        }
        FixtureShape::RowsInWorkTask(count) => {
            fill_ordinary(&mut tables, seed, 4, usize::from(count))?
        }
        FixtureShape::RowsInGitRepository(count) => {
            fill_ordinary(&mut tables, seed, 5, usize::from(count))?
        }
        FixtureShape::RowsInLocalCheckout(count) => {
            fill_ordinary(&mut tables, seed, 6, usize::from(count))?
        }
        FixtureShape::RowsInMonorepoComponent(count) => {
            fill_ordinary(&mut tables, seed, 7, usize::from(count))?
        }
        FixtureShape::RowsInProjectRepositoryLink(count) => {
            fill_ordinary(&mut tables, seed, 8, usize::from(count))?
        }
        FixtureShape::ScalarBytesAcrossRows { bytes, rows } => build_scalar_heavy(
            &mut tables,
            usize::try_from(bytes).unwrap(),
            usize::from(rows),
        )?,
        FixtureShape::ScalarElementsAcrossRows {
            elements,
            element_bytes,
            rows,
        } => build_vector_heavy(
            &mut tables,
            usize::from(elements),
            usize::from(element_bytes),
            usize::from(rows),
        )?,
        FixtureShape::ObjectPairsAcrossRows { pairs, rows } => {
            build_object_key_heavy(&mut tables, usize::from(pairs), usize::from(rows))?
        }
        FixtureShape::ObjectDepthAcrossRows { depth, rows } => {
            build_depth_heavy(&mut tables, usize::from(depth), usize::from(rows))?
        }
        FixtureShape::RawCanonicalBytes(bytes) => {
            build_raw_canonical(&mut tables, usize::try_from(bytes).unwrap())?
        }
    }

    validate_table_rows(&tables)?;
    let fixture = CalibrationFixture { case, tables };
    if case_id == 18 && fixture.canonical_bytes()?.len() != RAW_CANONICAL_BYTES {
        return Err(FixtureError::RawCanonicalSizeMismatch);
    }
    Ok(fixture)
}

fn empty_tables() -> [Array; TABLE_NAMES.len()] {
    std::array::from_fn(|_| Array::from(Vec::<Value>::new()))
}

#[derive(Clone, Copy)]
enum RowFlavor {
    SparseA,
    SparseB,
    Ordinary,
}

fn build_sparse_a(tables: &mut [Array; TABLE_NAMES.len()]) -> Result<(), FixtureError> {
    tables[0].push(fixed_row(1, 0, 0, RowFlavor::SparseA)?);
    tables[3].push(fixed_row(1, 3, 0, RowFlavor::SparseA)?);
    tables[5].push(fixed_row(1, 5, 0, RowFlavor::SparseA)?);
    Ok(())
}

fn build_sparse_b(tables: &mut [Array; TABLE_NAMES.len()]) -> Result<(), FixtureError> {
    tables[1].push(fixed_row(2, 1, 0, RowFlavor::SparseB)?);
    tables[4].push(fixed_row(2, 4, 0, RowFlavor::SparseB)?);
    tables[6].push(fixed_row(2, 6, 0, RowFlavor::SparseB)?);
    tables[7].push(fixed_row(2, 7, 0, RowFlavor::SparseB)?);
    tables[8].push(fixed_row(2, 8, 0, RowFlavor::SparseB)?);
    Ok(())
}

fn fill_ordinary(
    tables: &mut [Array; TABLE_NAMES.len()],
    seed: u16,
    table_index: usize,
    count: usize,
) -> Result<(), FixtureError> {
    for row_index in 0..count {
        tables[table_index].push(fixed_row(
            seed,
            table_index,
            row_index,
            RowFlavor::Ordinary,
        )?);
    }
    Ok(())
}

fn fixed_row(
    seed: u16,
    table_index: usize,
    row_index: usize,
    flavor: RowFlavor,
) -> Result<Value, FixtureError> {
    let table = TABLE_NAMES[table_index];
    let record_id = format!("c{seed:02}-t{table_index:02}-r{row_index:04}");
    let uuid_text = format!(
        "02b2{seed:04x}-{table_index:04x}-7{row_index:03x}-8{table_index:03x}-{row_index:012x}"
    );
    let uuid = Uuid::try_from(uuid_text.as_str()).map_err(|_| FixtureError::InvalidUuid)?;
    let flavor_text = match flavor {
        RowFlavor::SparseA => "sparse-a",
        RowFlavor::SparseB => "sparse-b",
        RowFlavor::Ordinary => "ordinary",
    };
    let mut row = BTreeMap::from([
        (
            "id".to_owned(),
            Value::Thing(Thing::from((table.to_owned(), record_id))),
        ),
        ("fixture_uuid".to_owned(), Value::Uuid(uuid)),
        ("ordinal".to_owned(), Value::from(row_index as i64)),
        ("flavor".to_owned(), Value::from(flavor_text)),
    ]);
    match flavor {
        RowFlavor::SparseA => {
            row.insert("optional_a".to_owned(), Value::Null);
        }
        RowFlavor::SparseB => {
            row.insert("optional_b".to_owned(), Value::Bool(true));
        }
        RowFlavor::Ordinary => {}
    }
    Ok(Value::Object(Object::from(row)))
}

fn build_scalar_heavy(
    tables: &mut [Array; TABLE_NAMES.len()],
    bytes: usize,
    rows: usize,
) -> Result<(), FixtureError> {
    if rows == 0 || bytes % rows != 0 {
        return Err(FixtureError::RawCanonicalSizeMismatch);
    }
    let bytes_per_row = bytes / rows;
    for row_index in 0..rows {
        let mut row = object_from_row(fixed_row(14, 0, row_index, RowFlavor::Ordinary)?)?;
        let byte = b'A' + u8::try_from(row_index % 26).unwrap();
        let scalar = String::from_utf8(vec![byte; bytes_per_row])
            .map_err(|_| FixtureError::UnsupportedNativeValue)?;
        row.insert("scalar".to_owned(), Value::from(scalar));
        tables[0].push(Value::Object(Object::from(row)));
    }
    Ok(())
}

fn build_vector_heavy(
    tables: &mut [Array; TABLE_NAMES.len()],
    elements: usize,
    element_bytes: usize,
    rows: usize,
) -> Result<(), FixtureError> {
    if rows == 0 || elements % rows != 0 || element_bytes != 8 {
        return Err(FixtureError::UnsupportedNativeValue);
    }
    let elements_per_row = elements / rows;
    for row_index in 0..rows {
        let mut row = object_from_row(fixed_row(15, 0, row_index, RowFlavor::Ordinary)?)?;
        let vector = (0..elements_per_row)
            .map(|element| {
                let ordinal = row_index
                    .checked_mul(elements_per_row)
                    .and_then(|base| base.checked_add(element))
                    .ok_or(FixtureError::LengthOverflow)?;
                Ok(Value::Number(Number::Int(ordinal as i64)))
            })
            .collect::<Result<Vec<_>, FixtureError>>()?;
        row.insert("vector".to_owned(), Value::Array(Array::from(vector)));
        tables[0].push(Value::Object(Object::from(row)));
    }
    Ok(())
}

fn build_object_key_heavy(
    tables: &mut [Array; TABLE_NAMES.len()],
    pairs: usize,
    rows: usize,
) -> Result<(), FixtureError> {
    if rows == 0 || pairs % rows != 0 {
        return Err(FixtureError::UnsupportedNativeValue);
    }
    let pairs_per_row = pairs / rows;
    for row_index in 0..rows {
        let mut row = object_from_row(fixed_row(16, 0, row_index, RowFlavor::Ordinary)?)?;
        let mut pairs_object = BTreeMap::new();
        for pair in 0..pairs_per_row {
            let ordinal = row_index
                .checked_mul(pairs_per_row)
                .and_then(|base| base.checked_add(pair))
                .ok_or(FixtureError::LengthOverflow)?;
            pairs_object.insert(format!("key-{ordinal:04}"), Value::from(ordinal as i64));
        }
        row.insert(
            "object".to_owned(),
            Value::Object(Object::from(pairs_object)),
        );
        tables[0].push(Value::Object(Object::from(row)));
    }
    Ok(())
}

fn build_depth_heavy(
    tables: &mut [Array; TABLE_NAMES.len()],
    depth: usize,
    rows: usize,
) -> Result<(), FixtureError> {
    for row_index in 0..rows {
        let mut nested = Value::from(row_index as i64);
        for level in (0..depth).rev() {
            nested = Value::Object(Object::from(BTreeMap::from([(
                format!("level-{level:02}"),
                nested,
            )])));
        }
        let mut row = object_from_row(fixed_row(17, 0, row_index, RowFlavor::Ordinary)?)?;
        row.insert("nested".to_owned(), nested);
        tables[0].push(Value::Object(Object::from(row)));
    }
    Ok(())
}

fn build_raw_canonical(
    tables: &mut [Array; TABLE_NAMES.len()],
    target_bytes: usize,
) -> Result<(), FixtureError> {
    let mut row = object_from_row(fixed_row(18, 0, 0, RowFlavor::Ordinary)?)?;
    row.insert("raw".to_owned(), Value::from(String::new()));
    tables[0].push(Value::Object(Object::from(row)));
    let base = canonical_table_bytes(tables)?.len();
    let payload_bytes = target_bytes
        .checked_sub(base)
        .ok_or(FixtureError::RawCanonicalSizeMismatch)?;
    let Value::Object(row) = &mut tables[0][0] else {
        return Err(FixtureError::UnsupportedNativeValue);
    };
    row.insert(
        "raw".to_owned(),
        Value::from(
            String::from_utf8(vec![b'R'; payload_bytes])
                .map_err(|_| FixtureError::UnsupportedNativeValue)?,
        ),
    );
    if canonical_table_bytes(tables)?.len() != target_bytes {
        return Err(FixtureError::RawCanonicalSizeMismatch);
    }
    Ok(())
}

fn object_from_row(value: Value) -> Result<BTreeMap<String, Value>, FixtureError> {
    let Value::Object(object) = value else {
        return Err(FixtureError::UnsupportedNativeValue);
    };
    Ok(object.into_iter().collect())
}

fn validate_table_rows(tables: &[Array; TABLE_NAMES.len()]) -> Result<(), FixtureError> {
    for (table_index, rows) in tables.iter().enumerate() {
        validate_table(table_index, rows)?;
    }
    Ok(())
}

fn validate_table(table_index: usize, rows: &Array) -> Result<(), FixtureError> {
    if rows.len() > TABLE_ROW_CAPS[table_index] {
        return Err(FixtureError::TableLimitExceeded);
    }
    for row in rows.iter() {
        let Value::Object(object) = row else {
            return Err(FixtureError::InvalidReadShape);
        };
        match object.get("id") {
            Some(Value::Thing(thing)) if thing.tb == TABLE_NAMES[table_index] => {}
            _ => return Err(FixtureError::InvalidRecordId),
        }
    }
    Ok(())
}

fn validate_value_shape(
    observed: &Value,
    expected: &Value,
    depth: usize,
) -> Result<(), FixtureError> {
    if depth > MAX_TEMPLATE_DEPTH {
        return Err(FixtureError::ValueDepthExceeded);
    }
    match (observed, expected) {
        (Value::Null, Value::Null)
        | (Value::Bool(_), Value::Bool(_))
        | (Value::Number(Number::Int(_)), Value::Number(Number::Int(_)))
        | (Value::Strand(_), Value::Strand(_))
        | (Value::Uuid(_), Value::Uuid(_))
        | (Value::Bytes(_), Value::Bytes(_)) => Ok(()),
        (Value::Array(observed), Value::Array(expected)) => {
            if observed.len() != expected.len() {
                return Err(FixtureError::InvalidReadShape);
            }
            for (observed, expected) in observed.iter().zip(expected.iter()) {
                validate_value_shape(observed, expected, depth + 1)?;
            }
            Ok(())
        }
        (Value::Object(observed), Value::Object(expected)) => {
            if observed.len() != expected.len() {
                return Err(FixtureError::InvalidReadShape);
            }
            for (key, expected_value) in expected.iter() {
                let observed_value = observed.get(key).ok_or(FixtureError::InvalidReadShape)?;
                validate_value_shape(observed_value, expected_value, depth + 1)?;
            }
            Ok(())
        }
        (Value::Thing(observed), Value::Thing(expected)) if observed.tb == expected.tb => {
            match (&observed.id, &expected.id) {
                (Id::Number(_), Id::Number(_))
                | (Id::String(_), Id::String(_))
                | (Id::Uuid(_), Id::Uuid(_)) => Ok(()),
                _ => Err(FixtureError::InvalidRecordId),
            }
        }
        (Value::Thing(_), Value::Thing(_)) => Err(FixtureError::InvalidRecordId),
        (_, Value::Null)
        | (_, Value::Bool(_))
        | (_, Value::Number(Number::Int(_)))
        | (_, Value::Strand(_))
        | (_, Value::Uuid(_))
        | (_, Value::Array(_))
        | (_, Value::Object(_))
        | (_, Value::Thing(_))
        | (_, Value::Bytes(_)) => Err(FixtureError::InvalidReadShape),
        _ => Err(FixtureError::UnsupportedNativeValue),
    }
}

pub fn canonical_read_bytes(value: &Value) -> Result<Vec<u8>, FixtureError> {
    canonical_read_bytes_with_limit(value, RAW_CANONICAL_BYTES)
}

fn canonical_read_bytes_with_limit(
    value: &Value,
    byte_limit: usize,
) -> Result<Vec<u8>, FixtureError> {
    let Value::Object(object) = value else {
        return Err(FixtureError::InvalidReadShape);
    };
    if object.len() != TABLE_NAMES.len()
        || object
            .keys()
            .any(|key| !TABLE_NAMES.into_iter().any(|table| table == key))
    {
        return Err(FixtureError::InvalidReadShape);
    }
    let mut output = Vec::new();
    output.extend_from_slice(&CANONICAL_FIXTURE_MAGIC);
    push_u16(&mut output, CANONICAL_FIXTURE_VERSION)?;
    push_u16(&mut output, TABLE_NAMES.len())?;
    for (table_index, table) in TABLE_NAMES.into_iter().enumerate() {
        let Some(Value::Array(rows)) = object.get(table) else {
            return Err(FixtureError::InvalidReadShape);
        };
        validate_table(table_index, rows)?;
        push_text_u16(&mut output, table)?;
        push_u32(&mut output, rows.len())?;
        for row in rows.iter() {
            encode_value(row, &mut output, 0, byte_limit)?;
        }
    }
    if output.len() > byte_limit {
        return Err(FixtureError::CanonicalSizeLimitExceeded);
    }
    Ok(output)
}

fn canonical_table_bytes(tables: &[Array; TABLE_NAMES.len()]) -> Result<Vec<u8>, FixtureError> {
    validate_table_rows(tables)?;
    let mut output = Vec::new();
    output.extend_from_slice(&CANONICAL_FIXTURE_MAGIC);
    push_u16(&mut output, CANONICAL_FIXTURE_VERSION)?;
    push_u16(&mut output, TABLE_NAMES.len())?;
    for (table_index, table) in TABLE_NAMES.into_iter().enumerate() {
        push_text_u16(&mut output, table)?;
        push_u32(&mut output, tables[table_index].len())?;
        for row in tables[table_index].iter() {
            encode_value(row, &mut output, 0, RAW_CANONICAL_BYTES)?;
        }
    }
    if output.len() > RAW_CANONICAL_BYTES {
        return Err(FixtureError::CanonicalSizeLimitExceeded);
    }
    Ok(output)
}

fn encode_value(
    value: &Value,
    output: &mut Vec<u8>,
    depth: usize,
    byte_limit: usize,
) -> Result<(), FixtureError> {
    if depth > MAX_CANONICAL_ENCODING_DEPTH {
        return Err(FixtureError::ValueDepthExceeded);
    }
    match value {
        Value::Null => push_canonical_bytes(output, &[0], byte_limit)?,
        Value::Bool(false) => push_canonical_bytes(output, &[1], byte_limit)?,
        Value::Bool(true) => push_canonical_bytes(output, &[2], byte_limit)?,
        Value::Number(Number::Int(number)) => {
            push_canonical_bytes(output, &[3], byte_limit)?;
            push_canonical_bytes(output, &number.to_be_bytes(), byte_limit)?;
        }
        Value::Strand(text) => {
            push_canonical_bytes(output, &[4], byte_limit)?;
            push_canonical_u32(output, text.len(), byte_limit)?;
            push_canonical_bytes(output, text.as_bytes(), byte_limit)?;
        }
        Value::Uuid(uuid) => {
            push_canonical_bytes(output, &[5], byte_limit)?;
            push_canonical_bytes(output, uuid.as_bytes(), byte_limit)?;
        }
        Value::Array(values) => {
            push_canonical_bytes(output, &[6], byte_limit)?;
            push_canonical_u32(output, values.len(), byte_limit)?;
            for item in values.iter() {
                encode_value(item, output, depth + 1, byte_limit)?;
            }
        }
        Value::Object(object) => {
            push_canonical_bytes(output, &[7], byte_limit)?;
            push_canonical_u32(output, object.len(), byte_limit)?;
            for (key, item) in object.iter() {
                push_canonical_u16(output, key.len(), byte_limit)?;
                push_canonical_bytes(output, key.as_bytes(), byte_limit)?;
                encode_value(item, output, depth + 1, byte_limit)?;
            }
        }
        Value::Thing(thing) => {
            push_canonical_bytes(output, &[8], byte_limit)?;
            push_canonical_u16(output, thing.tb.len(), byte_limit)?;
            push_canonical_bytes(output, thing.tb.as_bytes(), byte_limit)?;
            match &thing.id {
                Id::Number(number) => {
                    push_canonical_bytes(output, &[0], byte_limit)?;
                    push_canonical_bytes(output, &number.to_be_bytes(), byte_limit)?;
                }
                Id::String(text) => {
                    push_canonical_bytes(output, &[1], byte_limit)?;
                    push_canonical_u32(output, text.len(), byte_limit)?;
                    push_canonical_bytes(output, text.as_bytes(), byte_limit)?;
                }
                Id::Uuid(uuid) => {
                    push_canonical_bytes(output, &[2], byte_limit)?;
                    push_canonical_bytes(output, uuid.as_bytes(), byte_limit)?;
                }
                _ => return Err(FixtureError::UnsupportedNativeValue),
            }
        }
        Value::Bytes(bytes) => {
            push_canonical_bytes(output, &[9], byte_limit)?;
            push_canonical_u32(output, bytes.len(), byte_limit)?;
            push_canonical_bytes(output, bytes, byte_limit)?;
        }
        _ => return Err(FixtureError::UnsupportedNativeValue),
    }
    Ok(())
}

fn push_canonical_u16(
    output: &mut Vec<u8>,
    value: impl TryInto<u16>,
    byte_limit: usize,
) -> Result<(), FixtureError> {
    let value = value.try_into().map_err(|_| FixtureError::LengthOverflow)?;
    push_canonical_bytes(output, &value.to_be_bytes(), byte_limit)
}

fn push_canonical_u32(
    output: &mut Vec<u8>,
    value: impl TryInto<u32>,
    byte_limit: usize,
) -> Result<(), FixtureError> {
    let value = value.try_into().map_err(|_| FixtureError::LengthOverflow)?;
    push_canonical_bytes(output, &value.to_be_bytes(), byte_limit)
}

fn push_canonical_bytes(
    output: &mut Vec<u8>,
    bytes: &[u8],
    byte_limit: usize,
) -> Result<(), FixtureError> {
    let next = output
        .len()
        .checked_add(bytes.len())
        .ok_or(FixtureError::LengthOverflow)?;
    if next > byte_limit {
        return Err(FixtureError::CanonicalSizeLimitExceeded);
    }
    output.extend_from_slice(bytes);
    Ok(())
}

fn push_u16(output: &mut Vec<u8>, value: impl TryInto<u16>) -> Result<(), FixtureError> {
    let value = value.try_into().map_err(|_| FixtureError::LengthOverflow)?;
    output.extend_from_slice(&value.to_be_bytes());
    Ok(())
}

fn push_u32(output: &mut Vec<u8>, value: impl TryInto<u32>) -> Result<(), FixtureError> {
    let value = value.try_into().map_err(|_| FixtureError::LengthOverflow)?;
    output.extend_from_slice(&value.to_be_bytes());
    Ok(())
}

fn push_text_u16(output: &mut Vec<u8>, value: &str) -> Result<(), FixtureError> {
    push_u16(output, value.len())?;
    output.extend_from_slice(value.as_bytes());
    Ok(())
}

fn literal_matches(text: &str, identity: QueryIdentity) -> bool {
    text.len() == identity.byte_len && sha256(text.as_bytes()) == identity.sha256
}

fn exact_insert(statement: &Statement, table: &str) -> bool {
    let Statement::Insert(insert) = statement else {
        return false;
    };
    matches!(&insert.into, Some(Value::Table(into)) if into.as_str() == table)
        && matches!(
            &insert.data,
            Data::SingleExpression(Value::Param(parameter)) if parameter.as_str() == table
        )
        && !insert.ignore
        && insert.update.is_none()
        && matches!(insert.output, Some(Output::None))
        && insert.timeout.is_none()
        && !insert.parallel
        && !insert.relation
        && insert.version.is_none()
}

fn insert_ast_is_exact(query: &Query) -> bool {
    let statements = &query.0 .0;
    statements.len() == TABLE_NAMES.len() + 2
        && matches!(statements.first(), Some(Statement::Begin(_)))
        && TABLE_NAMES
            .into_iter()
            .enumerate()
            .all(|(index, table)| exact_insert(&statements[index + 1], table))
        && matches!(statements.last(), Some(Statement::Commit(_)))
}

fn exact_select(
    statement: &surrealdb_core::sql::statements::SelectStatement,
    table_index: usize,
) -> bool {
    let expected_limit = i64::try_from(TABLE_ROW_CAPS[table_index] + 1).unwrap();
    statement.expr.0.len() == 1
        && matches!(statement.expr.0.first(), Some(Field::All))
        && !statement.expr.1
        && statement.omit.is_none()
        && !statement.only
        && statement.what.0.len() == 1
        && matches!(
            statement.what.0.first(),
            Some(Value::Table(table)) if table.as_str() == TABLE_NAMES[table_index]
        )
        && statement.with.is_none()
        && statement.cond.is_none()
        && statement.split.is_none()
        && statement.group.is_none()
        && statement.order.is_none()
        && matches!(
            &statement.limit,
            Some(limit)
                if matches!(limit.0, Value::Number(Number::Int(value)) if value == expected_limit)
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
    let Value::Object(object) = &output.what else {
        return false;
    };
    object.len() == TABLE_NAMES.len()
        && TABLE_NAMES.into_iter().enumerate().all(|(index, table)| {
            matches!(
                object.get(table),
                Some(Value::Subquery(subquery)) if matches!(
                    subquery.as_ref(),
                    Subquery::Select(select) if exact_select(select, index)
                )
            )
        })
}

pub fn parse_insert_query() -> Result<Query, FixtureError> {
    if !literal_matches(INSERT_QUERY, INSERT_QUERY_IDENTITY) {
        return Err(FixtureError::QueryLiteralMismatch);
    }
    let query =
        surrealdb_core::syn::parse(INSERT_QUERY).map_err(|_| FixtureError::QueryParseFailure)?;
    if !insert_ast_is_exact(&query) {
        return Err(FixtureError::QueryAstMismatch);
    }
    Ok(query)
}

pub fn parse_read_query() -> Result<Query, FixtureError> {
    if !literal_matches(READ_QUERY, READ_QUERY_IDENTITY) {
        return Err(FixtureError::QueryLiteralMismatch);
    }
    let query =
        surrealdb_core::syn::parse(READ_QUERY).map_err(|_| FixtureError::QueryParseFailure)?;
    if !read_ast_is_exact(&query) {
        return Err(FixtureError::QueryAstMismatch);
    }
    Ok(query)
}

pub fn canonical_ast_identity(kind: QueryKind) -> Result<CanonicalAstIdentity, FixtureError> {
    let bytes = match kind {
        QueryKind::Insert => {
            let query = parse_insert_query()?;
            canonical_insert_ast_bytes(&query)?
        }
        QueryKind::Read => {
            let query = parse_read_query()?;
            canonical_read_ast_bytes(&query)?
        }
    };
    Ok(CanonicalAstIdentity {
        kind,
        byte_len: bytes.len(),
        sha256: sha256(&bytes),
    })
}

fn canonical_insert_ast_bytes(query: &Query) -> Result<Vec<u8>, FixtureError> {
    if !insert_ast_is_exact(query) {
        return Err(FixtureError::QueryAstMismatch);
    }
    let mut output = Vec::new();
    output.extend_from_slice(&CANONICAL_AST_MAGIC);
    push_u16(&mut output, 1_u16)?;
    push_u16(&mut output, query.0 .0.len())?;
    for statement in query.0 .0.iter() {
        match statement {
            Statement::Begin(_) => output.push(1),
            Statement::Insert(insert) => {
                let (Some(Value::Table(table)), Data::SingleExpression(Value::Param(parameter))) =
                    (&insert.into, &insert.data)
                else {
                    return Err(FixtureError::QueryAstMismatch);
                };
                output.push(2);
                push_text_u16(&mut output, table.as_str())?;
                push_text_u16(&mut output, parameter.as_str())?;
                output.push(1); // RETURN NONE
            }
            Statement::Commit(_) => output.push(3),
            _ => return Err(FixtureError::QueryAstMismatch),
        }
    }
    Ok(output)
}

fn canonical_read_ast_bytes(query: &Query) -> Result<Vec<u8>, FixtureError> {
    if !read_ast_is_exact(query) {
        return Err(FixtureError::QueryAstMismatch);
    }
    let [Statement::Output(statement)] = query.0 .0.as_slice() else {
        return Err(FixtureError::QueryAstMismatch);
    };
    let Value::Object(object) = &statement.what else {
        return Err(FixtureError::QueryAstMismatch);
    };
    let mut output = Vec::new();
    output.extend_from_slice(&CANONICAL_AST_MAGIC);
    push_u16(&mut output, 2_u16)?;
    push_u16(&mut output, object.len())?;
    for table in TABLE_NAMES {
        let Some(Value::Subquery(subquery)) = object.get(table) else {
            return Err(FixtureError::QueryAstMismatch);
        };
        let Subquery::Select(select) = subquery.as_ref() else {
            return Err(FixtureError::QueryAstMismatch);
        };
        let Some(Value::Table(from)) = select.what.0.first() else {
            return Err(FixtureError::QueryAstMismatch);
        };
        let Some(limit) = &select.limit else {
            return Err(FixtureError::QueryAstMismatch);
        };
        let Value::Number(Number::Int(limit_value)) = &limit.0 else {
            return Err(FixtureError::QueryAstMismatch);
        };
        output.push(4); // SELECT ALL
        push_text_u16(&mut output, table)?;
        push_text_u16(&mut output, from.as_str())?;
        output.extend_from_slice(&limit_value.to_be_bytes());
    }
    Ok(output)
}

pub fn ordered_case_manifest_bytes() -> Result<Vec<u8>, FixtureError> {
    let mut output = Vec::new();
    output.extend_from_slice(b"ENGC2CASE1");
    push_u16(&mut output, CALIBRATION_CASES.len())?;
    for case in CALIBRATION_CASES {
        push_u16(&mut output, case.case_id)?;
        push_u16(&mut output, case.fixture_id)?;
        push_u16(&mut output, case.profile_id)?;
        push_text_u16(&mut output, case.name)?;
        encode_shape(case.shape, &mut output)?;
        output.extend_from_slice(&(case.expected as u32).to_be_bytes());
        output.extend_from_slice(ROCKS_ENVIRONMENT_SHA256.as_bytes());
        encode_resource_profile(&mut output);
    }
    Ok(output)
}

fn encode_resource_profile(output: &mut Vec<u8>) {
    let resource = RESOURCE_PROFILE_V1;
    output.extend_from_slice(&resource.guest_cpus.to_be_bytes());
    output.extend_from_slice(&resource.guest_memory_bytes.to_be_bytes());
    output.extend_from_slice(&resource.supervisor_memory_max.to_be_bytes());
    output.extend_from_slice(&resource.supervisor_swap_max.to_be_bytes());
    output.extend_from_slice(&resource.supervisor_pids_max.to_be_bytes());
    output.push(u8::from(resource.supervisor_oom_group));
    output.extend_from_slice(&resource.supervisor_cpu_quota.to_be_bytes());
    output.extend_from_slice(&resource.supervisor_cpu_period.to_be_bytes());
    output.extend_from_slice(&resource.supervisor_nofile.to_be_bytes());
    output.extend_from_slice(&resource.supervisor_fsize.to_be_bytes());
    output.extend_from_slice(&resource.supervisor_core.to_be_bytes());
    output.extend_from_slice(&resource.workload_memory_max.to_be_bytes());
    output.extend_from_slice(&resource.workload_swap_max.to_be_bytes());
    output.push(u8::from(resource.guest_swap_absent));
    output.extend_from_slice(&resource.workload_pids_max.to_be_bytes());
    output.push(u8::from(resource.workload_oom_group));
    output.extend_from_slice(&resource.workload_cpu_quota.to_be_bytes());
    output.extend_from_slice(&resource.workload_cpu_period.to_be_bytes());
    output.extend_from_slice(&resource.normal_cpu_soft_seconds.to_be_bytes());
    output.extend_from_slice(&resource.normal_cpu_hard_seconds.to_be_bytes());
    output.extend_from_slice(&resource.probe_cpu_soft_seconds.to_be_bytes());
    output.extend_from_slice(&resource.probe_cpu_hard_seconds.to_be_bytes());
    output.extend_from_slice(&resource.role_nofile.to_be_bytes());
    output.extend_from_slice(&resource.role_fsize.to_be_bytes());
    output.extend_from_slice(&resource.role_core.to_be_bytes());
    output.extend_from_slice(&resource.role_wall_seconds.to_be_bytes());
    output.extend_from_slice(&resource.journey_seconds.to_be_bytes());
    output.extend_from_slice(&resource.characterization_seconds.to_be_bytes());
    output.extend_from_slice(&resource.positive_or_shared_seconds.to_be_bytes());
    output.extend_from_slice(&resource.dedicated_seconds.to_be_bytes());
    output.extend_from_slice(&resource.acceptance_campaign_seconds.to_be_bytes());
    output.extend_from_slice(&resource.complete_campaign_seconds.to_be_bytes());
    output.extend_from_slice(&resource.per_role_output_bytes.to_be_bytes());
    output.extend_from_slice(&resource.retained_evidence_bytes.to_be_bytes());
    output.extend_from_slice(&resource.terminal_receipt_bytes.to_be_bytes());
}

fn encode_shape(shape: FixtureShape, output: &mut Vec<u8>) -> Result<(), FixtureError> {
    let (tag, values): (u16, [u32; 3]) = match shape {
        FixtureShape::SparseA => (1, [0, 0, 0]),
        FixtureShape::SparseB => (2, [0, 0, 0]),
        FixtureShape::NineEmptyTableArrays => (3, [0, 0, 0]),
        FixtureShape::OneRowPerTable => (4, [0, 0, 0]),
        FixtureShape::RowsInMemoryItem(rows) => (5, [u32::from(rows), 0, 0]),
        FixtureShape::RowsInCorrectionProposal(rows) => (6, [u32::from(rows), 0, 0]),
        FixtureShape::RowsInForgetReceipt(rows) => (7, [u32::from(rows), 0, 0]),
        FixtureShape::RowsInWorkProject(rows) => (8, [u32::from(rows), 0, 0]),
        FixtureShape::RowsInWorkTask(rows) => (9, [u32::from(rows), 0, 0]),
        FixtureShape::RowsInGitRepository(rows) => (10, [u32::from(rows), 0, 0]),
        FixtureShape::RowsInLocalCheckout(rows) => (11, [u32::from(rows), 0, 0]),
        FixtureShape::RowsInMonorepoComponent(rows) => (12, [u32::from(rows), 0, 0]),
        FixtureShape::RowsInProjectRepositoryLink(rows) => (13, [u32::from(rows), 0, 0]),
        FixtureShape::ScalarBytesAcrossRows { bytes, rows } => (14, [bytes, u32::from(rows), 0]),
        FixtureShape::ScalarElementsAcrossRows {
            elements,
            element_bytes,
            rows,
        } => (
            15,
            [
                u32::from(elements),
                u32::from(element_bytes),
                u32::from(rows),
            ],
        ),
        FixtureShape::ObjectPairsAcrossRows { pairs, rows } => {
            (16, [u32::from(pairs), u32::from(rows), 0])
        }
        FixtureShape::ObjectDepthAcrossRows { depth, rows } => {
            (17, [u32::from(depth), u32::from(rows), 0])
        }
        FixtureShape::RawCanonicalBytes(bytes) => (18, [bytes, 0, 0]),
    };
    push_u16(output, tag)?;
    for value in values {
        output.extend_from_slice(&value.to_be_bytes());
    }
    Ok(())
}

pub fn candidate_build_record() -> Result<FixtureBuildRecordCandidate, FixtureError> {
    let mut fixture_stream = Vec::new();
    fixture_stream.extend_from_slice(b"ENGC2FIX1");
    push_u16(&mut fixture_stream, CALIBRATION_CASES.len())?;
    for case in CALIBRATION_CASES {
        let bytes = calibration_fixture(case.case_id)?.canonical_bytes()?;
        push_u16(&mut fixture_stream, case.case_id)?;
        push_u32(&mut fixture_stream, bytes.len())?;
        fixture_stream.extend_from_slice(&bytes);
    }
    let manifest = ordered_case_manifest_bytes()?;
    Ok(FixtureBuildRecordCandidate {
        fixture_source_sha256: sha256(include_bytes!("fixtures.rs")),
        ordered_case_manifest_sha256: sha256(&manifest),
        emitted_fixture_stream_sha256: sha256(&fixture_stream),
        insert_ast: canonical_ast_identity(QueryKind::Insert)?,
        read_ast: canonical_ast_identity(QueryKind::Read)?,
    })
}

pub fn manifest_is_closed() -> bool {
    CALIBRATION_CASES.len() == 20
        && CALIBRATION_CASES.iter().enumerate().all(|(index, case)| {
            let id = u16::try_from(index + 1).unwrap();
            case.case_id == id && case.fixture_id == id && case.profile_id == PROFILE_ID
        })
}
