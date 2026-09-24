use engram_native_c2b2a_payload::contract::*;

const STANDALONE_MANIFEST: &str = include_str!("../Cargo.toml");
const STANDALONE_LOCK: &str = include_str!("../Cargo.lock");

#[test]
fn accepted_identity_constants_are_exact() {
    assert_eq!(
        CONTRACT_SHA256,
        digest_from_hex("85312b0286080ecbbab94b5236003a42f281b1122565016d96bb20f9572063e3")
    );
    assert_eq!(INSERT_QUERY_IDENTITY.byte_len, 572);
    assert_eq!(READ_QUERY_IDENTITY.byte_len, 570);
    assert_eq!(INSERT_QUERY.len(), INSERT_QUERY_IDENTITY.byte_len);
    assert_eq!(READ_QUERY.len(), READ_QUERY_IDENTITY.byte_len);
    assert_eq!(
        sha256(INSERT_QUERY.as_bytes()),
        INSERT_QUERY_IDENTITY.sha256
    );
    assert_eq!(sha256(READ_QUERY.as_bytes()), READ_QUERY_IDENTITY.sha256);
    assert_eq!(
        INSERT_QUERY_IDENTITY.sha256,
        digest_from_hex("9f6c41b6c7d0006db0a03f29fd951bb3c81df19946748faaac768af013c98a05")
    );
    assert_eq!(
        READ_QUERY_IDENTITY.sha256,
        digest_from_hex("cf1034a335af240c0726e5385207acc4e4e49273dc016891138c863e8ba096f5")
    );
    assert_eq!(STORE_ORIGIN, "rocksdb:/data/store");
    assert_eq!(STORE_NAMESPACE, "engram");
    assert_eq!(STORE_DATABASE, "main");
    assert_eq!(TABLE_NAMES.len(), 9);
}

#[test]
fn standalone_manifest_and_lock_preserve_the_frozen_package_firewall() {
    let dependency_section = STANDALONE_MANIFEST
        .split_once("[dependencies]\n")
        .unwrap()
        .1
        .split_once("\n[")
        .unwrap()
        .0;
    let mut direct_dependencies = dependency_section
        .lines()
        .filter_map(|line| line.split_once('=').map(|(name, _)| name.trim()))
        .collect::<Vec<_>>();
    direct_dependencies.extend(STANDALONE_MANIFEST.lines().filter_map(|line| {
        line.strip_prefix("[dependencies.")
            .and_then(|name| name.strip_suffix(']'))
    }));
    assert_eq!(
        direct_dependencies,
        ["getrandom", "libc", "sha2", "surrealdb-core", "tokio"]
    );
    assert!(STANDALONE_MANIFEST.contains(
        "[dependencies.surrealdb-core]\n\
version = \"=2.6.0\"\n\
default-features = false\n\
features = [\"kv-rocksdb\"]\n\
optional = true"
    ));
    assert!(STANDALONE_MANIFEST.contains(
        "[dependencies.tokio]\n\
version = \"=1.49.0\"\n\
default-features = false\n\
features = [\"rt\", \"sync\", \"time\"]\n\
optional = true"
    ));

    for (name, version, checksum) in [
        (
            "surrealdb-core",
            "2.6.0",
            "c48e42c81713be2f9b3dae64328999eafe8b8060dd584059445a908748b39787",
        ),
        (
            "surrealdb-rocksdb",
            "0.24.0-surreal.1",
            "057727f56d48825ddbe45e4e7401cda6e99d864fbc004e7474b4689a5e72c86d",
        ),
        (
            "surrealdb-librocksdb-sys",
            "0.17.3+10.6.2",
            "db194f1cf601bb6f2d0f4cbf0931bc3e5a602bac41ef2e9a87eccdfb28b7fed2",
        ),
        (
            "getrandom",
            "0.3.4",
            "899def5c37c4fd7b2664648c28120ecec138e4d395b459e5ca34f9cce2dd77fd",
        ),
    ] {
        let matching_records = STANDALONE_LOCK
            .split("[[package]]")
            .filter(|record| {
                record
                    .lines()
                    .any(|line| line == format!("name = \"{name}\""))
                    && record
                        .lines()
                        .any(|line| line == format!("version = \"{version}\""))
                    && record
                        .lines()
                        .any(|line| line == format!("checksum = \"{checksum}\""))
            })
            .count();
        assert_eq!(
            matching_records, 1,
            "wrong or duplicate lock identity for {name}"
        );
    }

    // This is intentionally a direct-manifest assertion only. Core's unavoidable internal graph
    // contains generic crates such as `http`; their presence is not misreported as an SDK/provider
    // feature or as a direct dependency of this payload.
    for forbidden in [
        "surrealdb",
        "engram-core",
        "engram-store",
        "engram-index",
        "engram-mcp",
        "engram-embed",
        "clap",
        "anyhow",
        "reqwest",
        "anthropic",
        "openai",
    ] {
        assert!(
            !direct_dependencies.contains(&forbidden),
            "forbidden direct dependency {forbidden}"
        );
    }
}

#[test]
fn environment_table_has_exact_order_count_and_digest() {
    assert_eq!(BASE_ENVIRONMENT.len(), 5);
    assert_eq!(ROCKS_ENVIRONMENT.len(), 35);
    assert_eq!(FULL_ENVIRONMENT_COUNT, 40);
    assert_eq!(sha256(&rocks_environment_bytes()), ROCKS_ENVIRONMENT_SHA256);

    let expected: Vec<_> = BASE_ENVIRONMENT
        .into_iter()
        .chain(ROCKS_ENVIRONMENT)
        .collect();
    assert!(environment_is_exact(expected.iter().copied()));

    let mut reordered = expected.clone();
    reordered.swap(0, 1);
    assert!(!environment_is_exact(reordered.iter().copied()));

    let mut extra = expected.clone();
    extra.push("PATH=/bin");
    assert!(!environment_is_exact(extra.iter().copied()));

    let raw_envp = expected
        .iter()
        .flat_map(|entry| entry.bytes().chain(core::iter::once(0)))
        .collect::<Vec<_>>();
    assert_eq!(collector_envp_bytes(), raw_envp);
    assert_eq!(full_environment_bytes(), raw_envp);
    assert!(collector_envp_is_exact(&raw_envp));
    assert_eq!(raw_envp.last(), Some(&0));

    let mut missing_final_nul = raw_envp.clone();
    missing_final_nul.pop();
    assert!(!collector_envp_is_exact(&missing_final_nul));
    let mut changed_separator = raw_envp.clone();
    let first_separator = changed_separator
        .iter()
        .position(|byte| *byte == 0)
        .unwrap();
    changed_separator[first_separator] = b'\n';
    assert!(!collector_envp_is_exact(&changed_separator));
    let mut trailing_entry = raw_envp;
    trailing_entry.extend_from_slice(b"PATH=/bin\0");
    assert!(!collector_envp_is_exact(&trailing_entry));
}

#[test]
fn manifest_digests_fail_closed_until_a_reviewed_canonical_record_exists() {
    for kind in ManifestKindV1::ALL {
        assert_eq!(
            frozen_manifest_sha256(kind),
            Err(ManifestDigestNotFrozen { kind })
        );
    }
    assert_eq!(
        host_schedule_manifest_kind(&CHARACTERIZATION_SCHEDULE[0]),
        ManifestKindV1::CharacterizationSchedule
    );
    for entry in &ACCEPTANCE_SCHEDULE {
        assert_eq!(
            host_schedule_manifest_kind(entry),
            ManifestKindV1::AcceptanceSchedule
        );
    }
}

#[test]
fn collector_mode_is_one_exact_non_data_argument() {
    let expected_roles = [
        RoleId::Boundary,
        RoleId::Writer,
        RoleId::Contender,
        RoleId::ReleaseProbe,
        RoleId::Reader,
    ];
    for (mode, expected_role) in CollectorMode::ALL.into_iter().zip(expected_roles) {
        assert_eq!(parse_collector_arguments(&[mode.argv_token()]), Ok(mode));
        assert_eq!(mode.role_id(), expected_role);
    }
    assert_eq!(
        parse_collector_arguments(&[]),
        Err(CollectorArgvError::WrongArgumentCount)
    );
    assert_eq!(
        parse_collector_arguments(&["writer", "reader"]),
        Err(CollectorArgvError::WrongArgumentCount)
    );
    assert_eq!(
        parse_collector_arguments(&["Writer"]),
        Err(CollectorArgvError::UnknownMode)
    );
}

#[test]
fn case_manifest_is_closed_and_shape_exact() {
    assert_eq!(CALIBRATION_CASES.len(), 20);
    for (zero_based, case) in CALIBRATION_CASES.iter().enumerate() {
        let id = u16::try_from(zero_based + 1).unwrap();
        assert_eq!(case.case_id, id);
        assert_eq!(case.fixture_id, id);
        assert_eq!(case.profile_id, 1);
        assert_eq!(case.expected, ErrorCategory::Success);
    }
    assert_eq!(CALIBRATION_CASES[0].shape, CALIBRATION_CASES[18].shape);
    assert_eq!(CALIBRATION_CASES[1].shape, CALIBRATION_CASES[19].shape);
    assert_eq!(
        CALIBRATION_CASES[13].shape,
        FixtureShape::ScalarBytesAcrossRows {
            bytes: 1_048_576,
            rows: 16
        }
    );
    assert_eq!(
        CALIBRATION_CASES[14].shape,
        FixtureShape::ScalarElementsAcrossRows {
            elements: 4_096,
            element_bytes: 8,
            rows: 16
        }
    );
    assert_eq!(
        CALIBRATION_CASES[15].shape,
        FixtureShape::ObjectPairsAcrossRows {
            pairs: 4_096,
            rows: 16
        }
    );
    assert_eq!(
        CALIBRATION_CASES[16].shape,
        FixtureShape::ObjectDepthAcrossRows {
            depth: 32,
            rows: 64
        }
    );
    assert_eq!(
        CALIBRATION_CASES[17].shape,
        FixtureShape::RawCanonicalBytes(2_097_152)
    );
}

#[test]
fn schedule_selection_rejects_every_adjacent_mismatch() {
    let all_entries: Vec<_> = CHARACTERIZATION_SCHEDULE
        .iter()
        .chain(ACCEPTANCE_SCHEDULE.iter())
        .collect();
    assert_eq!(all_entries.len(), 11);
    for entry in all_entries {
        assert_eq!(
            select_schedule_entry(entry.phase_code, entry.class_code, entry.ordinal),
            Some(entry)
        );
        assert_eq!(
            select_schedule_entry(entry.phase_code + 1, entry.class_code, entry.ordinal),
            None
        );
        assert_eq!(
            select_schedule_entry(entry.phase_code, entry.class_code + 1, entry.ordinal),
            None
        );
        assert_eq!(
            select_schedule_entry(entry.phase_code, entry.class_code, entry.ordinal + 1),
            None
        );
    }
    assert_eq!(select_schedule_entry(0, 0, 0), None);
}

#[test]
fn frozen_cardinalities_and_terminal_sequences_are_derived() {
    let c01 = &CHARACTERIZATION_SCHEDULE[0];
    assert_eq!(measurement_count(c01), 21);
    assert_eq!(terminal_ready_sequence(c01), Some(23));
    assert_eq!(terminal_sequence(c01), Some(24));

    for (index, entry) in ACCEPTANCE_SCHEDULE.iter().enumerate() {
        let expected = match index {
            0..=2 => (140, Some(142), Some(143)),
            3 => (60, Some(62), Some(63)),
            4..=7 => (1, Some(3), Some(4)),
            8..=9 => (0, None, None),
            _ => unreachable!(),
        };
        assert_eq!(
            (
                measurement_count(entry),
                terminal_ready_sequence(entry),
                terminal_sequence(entry)
            ),
            expected
        );
    }

    assert_eq!(shared_boundary_subattempt_count(), 60);
    assert_eq!(ACCEPTANCE_BOUNDARY_MEASUREMENTS, 64);
    assert_eq!(ACCEPTANCE_BOUNDARY_SUBATTEMPTS, 66);
    assert_eq!(ACCEPTANCE_GUEST_MEASUREMENTS, 484);
    assert_eq!(LOGICAL_PROBE_COUNT, 41);
}

#[test]
fn measurement_schedule_is_complete_and_ordered() {
    let c01 = &CHARACTERIZATION_SCHEDULE[0];
    for index in 0..21 {
        let identity = expected_measurement(c01, index).unwrap();
        assert_eq!(identity.item_id, 1);
        assert_eq!(identity.subitem_id, index / 7 + 1);
        assert_eq!(identity.scope_code, index % 7 + 1);
        assert_eq!(
            identity.role_id,
            role_for_scope(identity.scope_code).unwrap()
        );
    }
    assert!(expected_measurement(c01, 21).is_none());

    let positive = &ACCEPTANCE_SCHEDULE[0];
    for index in 0..140 {
        let identity = expected_measurement(positive, index).unwrap();
        assert_eq!(identity.item_id, index / 7 + 1);
        assert_eq!(identity.subitem_id, identity.item_id);
        assert_eq!(identity.scope_code, index % 7 + 1);
    }
    assert!(expected_measurement(positive, 140).is_none());

    let shared = &ACCEPTANCE_SCHEDULE[3];
    let identities: Vec<_> = (0..60)
        .map(|index| expected_measurement(shared, index).unwrap())
        .collect();
    assert!(identities
        .iter()
        .all(|identity| identity.scope_code == 8 && identity.role_id == RoleId::Boundary));
    assert!(!identities
        .iter()
        .any(|identity| matches!(identity.item_id, 12 | 15 | 16)));
    assert_eq!(
        identities
            .iter()
            .filter(|identity| identity.item_id == 33)
            .count(),
        14
    );
    assert!(expected_measurement(shared, 60).is_none());

    assert_eq!(
        expected_measurement(&ACCEPTANCE_SCHEDULE[4], 0)
            .unwrap()
            .subitem_id,
        1
    );
    assert_eq!(
        expected_measurement(&ACCEPTANCE_SCHEDULE[5], 0)
            .unwrap()
            .subitem_id,
        2
    );
}

#[test]
fn resource_profile_is_the_frozen_calibration_ceiling() {
    let resource = RESOURCE_PROFILE_V1;
    assert_eq!(resource.guest_cpus, 2);
    assert_eq!(resource.guest_memory_bytes, 4 * 1024 * 1024 * 1024);
    assert_eq!(resource.supervisor_memory_max, 512 * 1024 * 1024);
    assert_eq!(resource.supervisor_swap_max, 0);
    assert!(resource.supervisor_oom_group);
    assert_eq!(
        (
            resource.supervisor_cpu_quota,
            resource.supervisor_cpu_period
        ),
        (100_000, 100_000)
    );
    assert_eq!(resource.supervisor_nofile, 512);
    assert_eq!(resource.supervisor_fsize, 64 * 1024 * 1024);
    assert_eq!(resource.supervisor_core, 0);
    assert_eq!(resource.workload_memory_max, 1024 * 1024 * 1024);
    assert_eq!(resource.workload_swap_max, 0);
    assert!(resource.guest_swap_absent);
    assert!(resource.workload_oom_group);
    assert_eq!(
        (resource.workload_cpu_quota, resource.workload_cpu_period),
        (100_000, 100_000)
    );
    assert_eq!(
        (resource.supervisor_pids_max, resource.workload_pids_max),
        (32, 128)
    );
    assert_eq!(
        (
            resource.normal_cpu_soft_seconds,
            resource.normal_cpu_hard_seconds
        ),
        (30, 30)
    );
    assert_eq!(
        (
            resource.probe_cpu_soft_seconds,
            resource.probe_cpu_hard_seconds
        ),
        (1, 2)
    );
    assert_eq!(resource.role_nofile, 256);
    assert_eq!(resource.role_fsize, 256 * 1024 * 1024);
    assert_eq!(resource.role_core, 0);
    assert_eq!(
        (resource.role_wall_seconds, resource.journey_seconds),
        (60, 240)
    );
    assert_eq!(resource.characterization_seconds, 600);
    assert_eq!(resource.positive_or_shared_seconds, 2_700);
    assert_eq!(resource.dedicated_seconds, 600);
    assert_eq!(resource.acceptance_campaign_seconds, 18_000);
    assert_eq!(resource.complete_campaign_seconds, 18_600);
    assert_eq!(resource.per_role_output_bytes, 65_536);
    assert_eq!(resource.retained_evidence_bytes, 64 * 1024 * 1024);
    assert_eq!(resource.terminal_receipt_bytes, 256 * 1024);
}

#[test]
fn measurement_rules_cover_roles_and_exception_probes() {
    let ordinary = MeasurementV1 {
        scope_code: 1,
        descriptor_type_bitmap: DescriptorType::PipeReadEnd.bit()
            | DescriptorType::PipeWriteEnd.bit(),
        ..MeasurementV1::default()
    };
    assert_eq!(ordinary.validate_for(None, RoleId::Writer), Ok(()));
    assert_eq!(
        ordinary.validate_for(None, RoleId::Reader),
        Err(MeasurementError::WrongRole)
    );

    let pre_start = MeasurementV1 {
        scope_code: 8,
        monotonic_elapsed_ns: 10,
        ..MeasurementV1::default()
    };
    assert_eq!(pre_start.validate_for(Some(15), RoleId::Boundary), Ok(()));
    assert_eq!(pre_start.validate_for(Some(16), RoleId::Boundary), Ok(()));
    let mut invalid_pre_start = pre_start;
    invalid_pre_start.pids_peak = 1;
    assert_eq!(
        invalid_pre_start.validate_for(Some(15), RoleId::Boundary),
        Err(MeasurementError::InapplicableFieldWasNonzero)
    );

    let p17 = MeasurementV1 {
        scope_code: 8,
        descriptor_type_bitmap: DescriptorType::DiagnosticRegularFile.bit(),
        ..MeasurementV1::default()
    };
    assert_eq!(p17.validate_for(Some(17), RoleId::Boundary), Ok(()));
    assert_eq!(
        MeasurementV1 {
            scope_code: 8,
            ..MeasurementV1::default()
        }
        .validate_for(Some(17), RoleId::Boundary),
        Err(MeasurementError::MissingProbeDescriptor)
    );

    let p18 = MeasurementV1 {
        scope_code: 8,
        descriptor_type_bitmap: DescriptorType::Socket.bit(),
        ..MeasurementV1::default()
    };
    assert_eq!(p18.validate_for(Some(18), RoleId::Boundary), Ok(()));
    assert_eq!(
        p18.validate_for(Some(19), RoleId::Boundary),
        Err(MeasurementError::ForbiddenDescriptorType)
    );
}

#[test]
fn measurement_arithmetic_never_saturates() {
    assert_eq!(checked_elapsed_ns(10, 15), Ok(5));
    assert_eq!(
        checked_elapsed_ns(15, 10),
        Err(MeasurementArithmeticError::Underflow)
    );
    assert_eq!(checked_sum(&[1, 2, 3]), Ok(6));
    assert_eq!(
        checked_sum(&[u64::MAX, 1]),
        Err(MeasurementArithmeticError::Overflow)
    );
    assert_eq!(checked_allocated_bytes(8), Ok(4_096));
    assert_eq!(
        checked_allocated_bytes(u64::MAX),
        Err(MeasurementArithmeticError::Overflow)
    );
    assert_eq!(checked_statfs_used(10, 4, 4_096), Ok(24_576));
    assert_eq!(
        checked_statfs_used(4, 10, 4_096),
        Err(MeasurementArithmeticError::Underflow)
    );
    assert_eq!(checked_role_output_bytes(32_768, 32_768), Ok(65_536));
    assert_eq!(
        checked_role_output_bytes(65_536, 1),
        Err(MeasurementArithmeticError::OutputLimitExceeded)
    );
}

#[test]
fn terminal_states_enforce_boolean_implications_and_success_table() {
    for entry in CHARACTERIZATION_SCHEDULE
        .iter()
        .chain(ACCEPTANCE_SCHEDULE[..8].iter())
    {
        let state = successful_terminal(entry).unwrap();
        assert_eq!(state.validate(), Ok(()));
        assert_eq!(state.primary_category, 0);
        assert_eq!(state.active_item_code, 0);
        assert_eq!(state.guest_cleanup_proven, 1);
        assert_eq!(state.roles_reaped, 1);
    }
    assert!(successful_terminal(&ACCEPTANCE_SCHEDULE[8]).is_none());
    assert_eq!(active_item_code(0x1234, 0xabcd), 0x1234_abcd);

    for active_item_code in [0x0001_0000, 0x0000_0001] {
        let invalid = TerminalStateV1 {
            active_item_code,
            ..TerminalStateV1::default()
        };
        assert_eq!(
            invalid.validate(),
            Err(TerminalStateError::HalfZeroActiveItem)
        );
    }

    let invalid = TerminalStateV1 {
        guest_cleanup_proven: 1,
        roles_reaped: 0,
        ..TerminalStateV1::default()
    };
    assert_eq!(
        invalid.validate(),
        Err(TerminalStateError::CleanupWithoutReap)
    );
    let invalid = TerminalStateV1 {
        write_may_have_been_dispatched: 0,
        writer_acknowledged: 1,
        ..TerminalStateV1::default()
    };
    assert_eq!(
        invalid.validate(),
        Err(TerminalStateError::AcknowledgementWithoutDispatch)
    );
}

#[test]
fn item_flags_reset_per_item_and_preserve_final_milestones() {
    let mut item = ActiveItemState::default();
    item.begin(1, 1);
    assert_eq!(item.active_item_code, 0x0001_0001);
    item.mark_full_dispatch();
    item.mark_writer_acknowledged().unwrap();
    item.finish();
    assert_eq!(item.active_item_code, 0);
    assert!(item.write_may_have_been_dispatched);
    assert!(item.writer_acknowledged);

    item.begin(2, 2);
    assert_eq!(item.active_item_code, 0x0002_0002);
    assert!(!item.write_may_have_been_dispatched);
    assert!(!item.writer_acknowledged);
    assert_eq!(
        item.mark_writer_acknowledged(),
        Err(TerminalStateError::AcknowledgementWithoutDispatch)
    );
}

#[test]
fn deterministic_failure_precedence_is_phase_sensitive() {
    let every_signal = FailureSnapshot {
        protocol: true,
        host_contract: true,
        guest_configuration: true,
        containment: true,
        static_firewall: true,
        limit_exceeded: true,
        invalid_fixture: true,
        store_open: true,
        current_stage: Some(PostAcknowledgementFailure::CanonicalDisagreement),
        measurement: true,
        challenge: true,
    };
    assert_eq!(
        classify_failure(FailurePhase::BeforeDispatch, every_signal),
        Some(ErrorCategory::ProtocolFailure)
    );
    assert_eq!(
        classify_failure(FailurePhase::DispatchOutcomeUncertain, every_signal),
        Some(ErrorCategory::OutcomeUncertain)
    );
    assert_eq!(
        classify_failure(FailurePhase::AfterWriterAcknowledgement, every_signal),
        Some(ErrorCategory::ProtocolFailure)
    );

    for (stage, category) in [
        (
            PostAcknowledgementFailure::LockWitnessMissing,
            ErrorCategory::ExclusivityBroken,
        ),
        (
            PostAcknowledgementFailure::ContenderOpened,
            ErrorCategory::ExclusivityBroken,
        ),
        (
            PostAcknowledgementFailure::ContenderWrongError,
            ErrorCategory::ContenderFailure,
        ),
        (
            PostAcknowledgementFailure::WriterReleaseUnproven,
            ErrorCategory::ReleaseUnproven,
        ),
        (
            PostAcknowledgementFailure::ReleaseProbeFailure,
            ErrorCategory::ReleaseUnproven,
        ),
        (
            PostAcknowledgementFailure::ReaderOpenFailure,
            ErrorCategory::ReopenFailure,
        ),
        (
            PostAcknowledgementFailure::ReadOrDecodeFailure,
            ErrorCategory::EngineReadFailure,
        ),
        (
            PostAcknowledgementFailure::CanonicalDisagreement,
            ErrorCategory::CanonicalMismatch,
        ),
    ] {
        assert_eq!(
            classify_failure(
                FailurePhase::AfterWriterAcknowledgement,
                FailureSnapshot {
                    current_stage: Some(stage),
                    ..FailureSnapshot::default()
                }
            ),
            Some(category)
        );
    }
}
