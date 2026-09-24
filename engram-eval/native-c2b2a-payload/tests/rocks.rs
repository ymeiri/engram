use engram_native_c2b2a_payload::contract::{
    ErrorCategory, CALIBRATION_CASES, INSERT_QUERY_IDENTITY, READ_QUERY_IDENTITY,
    RESOURCE_PROFILE_V1, ROCKS_ENVIRONMENT_SHA256, TABLE_NAMES,
};
use engram_native_c2b2a_payload::fixtures::{
    calibration_expected_state, calibration_fixture, candidate_build_record,
    canonical_ast_identity, canonical_read_bytes, manifest_is_closed, ordered_case_manifest_bytes,
    parse_insert_query, parse_read_query, require_reviewed_build_record, CalibrationExpectedState,
    CalibrationFixture, FixtureBuildIdentityState, FixtureError, QueryKind,
    ReviewedFixtureBuildRecord, FIXTURE_BUILD_IDENTITY_STATE, RAW_CANONICAL_BYTES,
};
use engram_native_c2b2a_payload::rocks::{
    attest_runtime_environment, classify_contender_error, expected_runtime_environ_bytes,
    expected_runtime_environment, CanonicalReadVerified, LockClassifierState, ReaderOpen,
    ReleaseProbeHandlesDropped, ReleaseProbeOpen, RocksError, RuntimeAuthority, WriterCommitted,
    WriterHandlesDropped, WriterOpen, LOCK_CLASSIFIER_STATE,
};
use surrealdb_core::sql::{Array, Number, Object, Value};

#[test]
fn manifest_and_all_twenty_fixture_shapes_are_closed() {
    assert!(manifest_is_closed());
    assert_eq!(CALIBRATION_CASES.len(), 20);

    let expected_counts = [
        [1, 0, 0, 1, 0, 1, 0, 0, 0],
        [0, 1, 0, 0, 1, 0, 1, 1, 1],
        [0, 0, 0, 0, 0, 0, 0, 0, 0],
        [1, 1, 1, 1, 1, 1, 1, 1, 1],
        [64, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 32, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 32, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 8, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 64, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 16, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 32, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 64, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 64],
        [16, 0, 0, 0, 0, 0, 0, 0, 0],
        [16, 0, 0, 0, 0, 0, 0, 0, 0],
        [16, 0, 0, 0, 0, 0, 0, 0, 0],
        [64, 0, 0, 0, 0, 0, 0, 0, 0],
        [1, 0, 0, 0, 0, 0, 0, 0, 0],
        [1, 0, 0, 1, 0, 1, 0, 0, 0],
        [0, 1, 0, 0, 1, 0, 1, 1, 1],
    ];

    for (index, expected) in expected_counts.into_iter().enumerate() {
        let fixture = calibration_fixture((index + 1) as u16).expect("closed fixture");
        assert_eq!(fixture.row_counts(), expected, "case {}", index + 1);
    }
    assert_eq!(
        calibration_fixture(0).err(),
        Some(FixtureError::UnknownCase)
    );
    assert_eq!(
        calibration_fixture(21).err(),
        Some(FixtureError::UnknownCase)
    );
}

#[test]
fn opening_and_closing_fixtures_are_byte_identical() {
    let sparse_a_open = calibration_fixture(1).unwrap().canonical_bytes().unwrap();
    let sparse_b_open = calibration_fixture(2).unwrap().canonical_bytes().unwrap();
    let sparse_a_close = calibration_fixture(19).unwrap().canonical_bytes().unwrap();
    let sparse_b_close = calibration_fixture(20).unwrap().canonical_bytes().unwrap();

    assert_eq!(sparse_a_open, sparse_a_close);
    assert_eq!(sparse_b_open, sparse_b_close);
    assert_ne!(sparse_a_open, sparse_b_open);
}

#[test]
fn heavy_fixture_cardinalities_are_exact() {
    let scalar = calibration_fixture(14).unwrap().read_output_value();
    let scalar_rows = table_rows(&scalar, 0);
    let scalar_bytes = scalar_rows
        .iter()
        .map(|row| match field(row, "scalar") {
            Value::Strand(value) => value.len(),
            _ => panic!("scalar field is not a native strand"),
        })
        .sum::<usize>();
    assert_eq!(scalar_rows.len(), 16);
    assert_eq!(scalar_bytes, 1_048_576);

    let vector = calibration_fixture(15).unwrap().read_output_value();
    let vector_rows = table_rows(&vector, 0);
    let mut element_count = 0;
    for row in vector_rows.iter() {
        let Value::Array(values) = field(row, "vector") else {
            panic!("vector field is not a native array");
        };
        assert_eq!(values.len(), 256);
        for value in values.iter() {
            assert!(matches!(value, Value::Number(Number::Int(_))));
            element_count += 1;
        }
    }
    assert_eq!(element_count, 4_096);

    let object_heavy = calibration_fixture(16).unwrap().read_output_value();
    let object_rows = table_rows(&object_heavy, 0);
    let pair_count = object_rows
        .iter()
        .map(|row| match field(row, "object") {
            Value::Object(value) => value.len(),
            _ => panic!("object field is not a native object"),
        })
        .sum::<usize>();
    assert_eq!(object_rows.len(), 16);
    assert_eq!(pair_count, 4_096);

    let depth_heavy = calibration_fixture(17).unwrap().read_output_value();
    let depth_rows = table_rows(&depth_heavy, 0);
    assert_eq!(depth_rows.len(), 64);
    for row in depth_rows.iter() {
        let mut cursor = field(row, "nested");
        for _ in 0..32 {
            let Value::Object(object) = cursor else {
                panic!("nested value ended before depth 32");
            };
            assert_eq!(object.len(), 1);
            cursor = object.values().next().unwrap();
        }
        assert!(matches!(cursor, Value::Number(Number::Int(_))));
    }
}

#[test]
fn raw_fixture_is_exactly_two_mib_of_canonical_bytes() {
    let fixture = calibration_fixture(18).unwrap();
    let first = fixture.canonical_bytes().unwrap();
    let second = calibration_fixture(18).unwrap().canonical_bytes().unwrap();
    assert_eq!(first.len(), RAW_CANONICAL_BYTES);
    assert_eq!(first, second);
}

#[test]
fn read_normalizer_requires_the_complete_exact_native_shape() {
    for case in CALIBRATION_CASES {
        let fixture = calibration_fixture(case.case_id).unwrap();
        let expected = fixture.canonical_bytes().unwrap();
        let read_value = fixture.read_output_value();
        let observed = canonical_read_bytes(&read_value).unwrap();
        assert_eq!(observed, expected, "case {}", case.case_id);
        let private_expected = calibration_expected_state(case.case_id).unwrap();
        assert_eq!(private_expected.case_id(), case.case_id);
        assert!(private_expected.compare_read_value(&read_value).unwrap());
    }

    let fixture = calibration_fixture(4).unwrap();
    let mut value = fixture.read_output_value();
    let Value::Object(outer) = &mut value else {
        unreachable!();
    };
    outer.insert("unexpected".to_owned(), Value::Array(Array::default()));
    assert_eq!(
        canonical_read_bytes(&value),
        Err(FixtureError::InvalidReadShape)
    );

    let fixture = calibration_fixture(5).unwrap();
    let mut value = fixture.read_output_value();
    let Value::Object(outer) = &mut value else {
        unreachable!();
    };
    let Some(Value::Array(rows)) = outer.get_mut(TABLE_NAMES[0]) else {
        unreachable!();
    };
    let duplicate = rows[0].clone();
    rows.push(duplicate);
    assert_eq!(
        canonical_read_bytes(&value),
        Err(FixtureError::TableLimitExceeded)
    );

    let expected = calibration_expected_state(14).unwrap();
    let mut oversized = calibration_fixture(14).unwrap().read_output_value();
    let Value::Object(outer) = &mut oversized else {
        unreachable!();
    };
    let Some(Value::Array(rows)) = outer.get_mut(TABLE_NAMES[0]) else {
        unreachable!();
    };
    let Value::Object(first_row) = &mut rows[0] else {
        unreachable!();
    };
    first_row.insert(
        "scalar".to_owned(),
        Value::from("X".repeat(RAW_CANONICAL_BYTES)),
    );
    assert_eq!(
        expected.compare_read_value(&oversized),
        Err(FixtureError::CanonicalSizeLimitExceeded)
    );

    let mut too_deep = calibration_fixture(1).unwrap().read_output_value();
    let Value::Object(outer) = &mut too_deep else {
        unreachable!();
    };
    let Some(Value::Array(rows)) = outer.get_mut(TABLE_NAMES[0]) else {
        unreachable!();
    };
    let Value::Object(first_row) = &mut rows[0] else {
        unreachable!();
    };
    let mut nested = Value::Null;
    for level in 0..=40 {
        nested = Value::Object(Object::from(std::collections::BTreeMap::from([(
            format!("depth-{level}"),
            nested,
        )])));
    }
    first_row.insert("unexpected-depth".to_owned(), nested);
    assert_eq!(
        canonical_read_bytes(&too_deep),
        Err(FixtureError::ValueDepthExceeded)
    );
}

#[test]
fn ordered_case_manifest_binds_every_row_to_rocks_and_complete_resource_profile() {
    let manifest = ordered_case_manifest_bytes().unwrap();
    let rocks_occurrences = manifest
        .windows(ROCKS_ENVIRONMENT_SHA256.as_bytes().len())
        .filter(|window| *window == ROCKS_ENVIRONMENT_SHA256.as_bytes())
        .count();
    assert_eq!(rocks_occurrences, CALIBRATION_CASES.len());

    let resource = RESOURCE_PROFILE_V1;
    let mut tuple = Vec::new();
    tuple.extend_from_slice(&resource.guest_cpus.to_be_bytes());
    tuple.extend_from_slice(&resource.guest_memory_bytes.to_be_bytes());
    tuple.extend_from_slice(&resource.supervisor_memory_max.to_be_bytes());
    tuple.extend_from_slice(&resource.supervisor_swap_max.to_be_bytes());
    tuple.extend_from_slice(&resource.supervisor_pids_max.to_be_bytes());
    tuple.push(u8::from(resource.supervisor_oom_group));
    tuple.extend_from_slice(&resource.supervisor_cpu_quota.to_be_bytes());
    tuple.extend_from_slice(&resource.supervisor_cpu_period.to_be_bytes());
    tuple.extend_from_slice(&resource.supervisor_nofile.to_be_bytes());
    tuple.extend_from_slice(&resource.supervisor_fsize.to_be_bytes());
    tuple.extend_from_slice(&resource.supervisor_core.to_be_bytes());
    tuple.extend_from_slice(&resource.workload_memory_max.to_be_bytes());
    tuple.extend_from_slice(&resource.workload_swap_max.to_be_bytes());
    tuple.push(u8::from(resource.guest_swap_absent));
    tuple.extend_from_slice(&resource.workload_pids_max.to_be_bytes());
    tuple.push(u8::from(resource.workload_oom_group));
    tuple.extend_from_slice(&resource.workload_cpu_quota.to_be_bytes());
    tuple.extend_from_slice(&resource.workload_cpu_period.to_be_bytes());
    tuple.extend_from_slice(&resource.normal_cpu_soft_seconds.to_be_bytes());
    tuple.extend_from_slice(&resource.normal_cpu_hard_seconds.to_be_bytes());
    tuple.extend_from_slice(&resource.probe_cpu_soft_seconds.to_be_bytes());
    tuple.extend_from_slice(&resource.probe_cpu_hard_seconds.to_be_bytes());
    tuple.extend_from_slice(&resource.role_nofile.to_be_bytes());
    tuple.extend_from_slice(&resource.role_fsize.to_be_bytes());
    tuple.extend_from_slice(&resource.role_core.to_be_bytes());
    tuple.extend_from_slice(&resource.role_wall_seconds.to_be_bytes());
    tuple.extend_from_slice(&resource.journey_seconds.to_be_bytes());
    tuple.extend_from_slice(&resource.characterization_seconds.to_be_bytes());
    tuple.extend_from_slice(&resource.positive_or_shared_seconds.to_be_bytes());
    tuple.extend_from_slice(&resource.dedicated_seconds.to_be_bytes());
    tuple.extend_from_slice(&resource.acceptance_campaign_seconds.to_be_bytes());
    tuple.extend_from_slice(&resource.complete_campaign_seconds.to_be_bytes());
    tuple.extend_from_slice(&resource.per_role_output_bytes.to_be_bytes());
    tuple.extend_from_slice(&resource.retained_evidence_bytes.to_be_bytes());
    tuple.extend_from_slice(&resource.terminal_receipt_bytes.to_be_bytes());

    let resource_occurrences = manifest
        .windows(tuple.len())
        .filter(|window| *window == tuple.as_slice())
        .count();
    assert_eq!(resource_occurrences, CALIBRATION_CASES.len());
}

#[test]
fn fixed_queries_parse_to_the_required_ast_shapes() {
    parse_insert_query().expect("exact insert query and AST");
    parse_read_query().expect("exact read query and AST");
    let insert = canonical_ast_identity(QueryKind::Insert).unwrap();
    let read = canonical_ast_identity(QueryKind::Read).unwrap();

    assert_eq!(INSERT_QUERY_IDENTITY.byte_len, 572);
    assert_eq!(READ_QUERY_IDENTITY.byte_len, 570);
    assert_eq!(insert.kind, QueryKind::Insert);
    assert_eq!(read.kind, QueryKind::Read);
    assert_ne!(insert.sha256, read.sha256);
    assert_ne!(insert.byte_len, 0);
    assert_ne!(read.byte_len, 0);
}

#[test]
fn build_record_candidate_is_deterministic_but_not_acceptance_authority() {
    assert_eq!(
        FIXTURE_BUILD_IDENTITY_STATE,
        FixtureBuildIdentityState::Unfrozen
    );
    assert!(matches!(
        require_reviewed_build_record(),
        Err(FixtureError::ReviewedBuildRecordMissing)
    ));
    let first = candidate_build_record().unwrap();
    let second = candidate_build_record().unwrap();
    assert_eq!(first, second);
    assert_ne!(
        first.ordered_case_manifest_sha256,
        first.emitted_fixture_stream_sha256
    );
}

#[test]
fn environment_and_lock_boundaries_fail_closed() {
    let expected = expected_runtime_environment();
    attest_runtime_environment(expected.iter().copied()).unwrap();

    let mut wrong = expected.clone();
    wrong.swap(0, 1);
    assert_eq!(
        attest_runtime_environment(wrong.iter().copied()),
        Err(RocksError::EnvironmentMismatch)
    );
    assert_eq!(LOCK_CLASSIFIER_STATE, LockClassifierState::Uncalibrated);
    for bytes in [
        b"LOCK held".as_slice(),
        b" LOCK held".as_slice(),
        b"LOCK held\n".as_slice(),
        b"".as_slice(),
        &[0xff][..],
    ] {
        assert_eq!(
            classify_contender_error(bytes),
            Err(RocksError::LockClassifierUncalibrated)
        );
    }

    let expected_bytes = expected_runtime_environ_bytes();
    assert!(expected_bytes.ends_with(&[0]));
    assert_eq!(
        expected_bytes.iter().filter(|byte| **byte == 0).count(),
        expected.len()
    );
}

#[test]
fn rocks_errors_preserve_the_frozen_phase_boundaries() {
    for error in [
        FixtureError::LengthOverflow,
        FixtureError::TableLimitExceeded,
        FixtureError::ValueDepthExceeded,
        FixtureError::CanonicalSizeLimitExceeded,
    ] {
        assert_eq!(RocksError::from(error), RocksError::LimitExceeded);
    }

    let pre_dispatch = [
        (
            RocksError::ReviewedBuildRecordMissing,
            ErrorCategory::StaticFirewall,
        ),
        (
            RocksError::EnvironmentMismatch,
            ErrorCategory::GuestConfiguration,
        ),
        (RocksError::LimitExceeded, ErrorCategory::LimitExceeded),
        (RocksError::InvalidFixture, ErrorCategory::InvalidFixture),
        (
            RocksError::StoreOpenFailure,
            ErrorCategory::StoreOpenFailure,
        ),
    ];
    for (error, expected) in pre_dispatch {
        assert_eq!(error.pre_dispatch_category(), Some(expected));
    }

    for error in [
        RocksError::ReviewedBuildRecordMissing,
        RocksError::EnvironmentMismatch,
        RocksError::QueryContractInvalid,
        RocksError::LimitExceeded,
        RocksError::InvalidFixture,
        RocksError::StoreOpenFailure,
        RocksError::OutcomeUncertain,
        RocksError::LockClassifierUncalibrated,
        RocksError::ContenderOpened,
        RocksError::ContenderFailure,
        RocksError::ReleaseOpenFailure,
        RocksError::ReopenFailure,
        RocksError::EngineReadFailure,
        RocksError::CanonicalMismatch,
    ] {
        assert_eq!(
            error.dispatched_unacknowledged_category(),
            ErrorCategory::OutcomeUncertain
        );
    }

    let post_ack = [
        (RocksError::LimitExceeded, ErrorCategory::LimitExceeded),
        (
            RocksError::ContenderOpened,
            ErrorCategory::ExclusivityBroken,
        ),
        (
            RocksError::ContenderFailure,
            ErrorCategory::ContenderFailure,
        ),
        (
            RocksError::ReleaseOpenFailure,
            ErrorCategory::ReleaseUnproven,
        ),
        (RocksError::ReopenFailure, ErrorCategory::ReopenFailure),
        (
            RocksError::EngineReadFailure,
            ErrorCategory::EngineReadFailure,
        ),
        (
            RocksError::CanonicalMismatch,
            ErrorCategory::CanonicalMismatch,
        ),
    ];
    for (error, expected) in post_ack {
        assert_eq!(error.post_ack_current_stage_category(), Some(expected));
    }
}

#[test]
fn rocks_source_uses_process_and_only_observable_handle_drop() {
    let source = include_str!("../src/rocks.rs");
    assert!(!source.contains(".restart("));
    assert!(!source.contains(".shutdown("));
    assert!(!source.contains(".execute("));
    assert_eq!(source.matches("Datastore::new(STORE_ORIGIN)").count(), 2);
    assert!(source.contains(".process(insert_query"));
    assert!(source.contains(".process(read_query"));
    assert!(!source.contains("map_err(|_| RocksError::StoreOpenFailure)"));
}

fn table_rows(value: &Value, table_index: usize) -> &Array {
    let Value::Object(object) = value else {
        panic!("fixture output is not an object");
    };
    let Some(Value::Array(rows)) = object.get(TABLE_NAMES[table_index]) else {
        panic!("fixture table is not an array");
    };
    rows
}

fn field<'a>(row: &'a Value, name: &str) -> &'a Value {
    let Value::Object(object) = row else {
        panic!("fixture row is not an object");
    };
    object.get(name).expect("fixed fixture field")
}

const _: fn(Object) -> Value = Value::Object;

macro_rules! assert_not_impl {
    ($type:ty: $bound:path) => {
        const _: fn() = || {
            trait AmbiguousIfImpl<A> {
                fn marker() {}
            }
            impl<T: ?Sized> AmbiguousIfImpl<()> for T {}
            impl<T: ?Sized + $bound> AmbiguousIfImpl<u8> for T {}
            let _ = <$type as AmbiguousIfImpl<_>>::marker;
        };
    };
}

assert_not_impl!(WriterOpen: Clone);
assert_not_impl!(WriterOpen: Copy);
assert_not_impl!(WriterCommitted: Clone);
assert_not_impl!(WriterCommitted: Copy);
assert_not_impl!(WriterHandlesDropped: Clone);
assert_not_impl!(WriterHandlesDropped: Copy);
assert_not_impl!(ReleaseProbeOpen: Clone);
assert_not_impl!(ReleaseProbeOpen: Copy);
assert_not_impl!(ReleaseProbeHandlesDropped: Clone);
assert_not_impl!(ReleaseProbeHandlesDropped: Copy);
assert_not_impl!(ReaderOpen: Clone);
assert_not_impl!(ReaderOpen: Copy);
assert_not_impl!(CanonicalReadVerified: Clone);
assert_not_impl!(CanonicalReadVerified: Copy);
assert_not_impl!(ReviewedFixtureBuildRecord: Clone);
assert_not_impl!(ReviewedFixtureBuildRecord: Copy);
assert_not_impl!(RuntimeAuthority: Clone);
assert_not_impl!(RuntimeAuthority: Copy);
assert_not_impl!(CalibrationFixture: Clone);
assert_not_impl!(CalibrationFixture: Copy);
assert_not_impl!(CalibrationExpectedState: Clone);
assert_not_impl!(CalibrationExpectedState: Copy);
