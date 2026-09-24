use engram_native_c2b2a_payload::contract::*;
use engram_native_c2b2a_payload::protocol::*;

fn digest(byte: u8) -> Digest32 {
    Digest32([byte; 32])
}

fn binding() -> StreamBinding {
    StreamBinding::unverified_for_codec(digest(0x11), digest(0x22))
}

fn role_frame(kind: MessageKind, sequence: u32, payload: &[u8], status: u16) -> Frame {
    Frame::new(
        kind,
        sequence,
        7,
        3,
        RoleId::Writer,
        status,
        binding(),
        payload,
    )
    .unwrap()
}

#[test]
fn header_is_exactly_160_bytes_and_big_endian() {
    let encoded = role_frame(MessageKind::Start, 0x0102_0304, &[], 0).encode();
    let bytes = encoded.as_bytes();
    assert_eq!(bytes.len(), 160);
    assert_eq!(&bytes[0..8], b"ENGC2A01");
    assert_eq!(&bytes[8..10], &[0, 1]);
    assert_eq!(&bytes[10..12], &[0, 1]);
    assert_eq!(&bytes[12..16], &[1, 2, 3, 4]);
    assert_eq!(&bytes[16..18], &[0, 7]);
    assert_eq!(&bytes[18..20], &[0, 3]);
    assert_eq!(&bytes[20..22], &[0, 1]);
    assert_eq!(&bytes[22..24], &[0, 2]);
    assert_eq!(&bytes[24..28], &[0, 0, 0, 0]);
    assert_eq!(&bytes[28..32], &[0, 0, 0, 0]);
    assert_eq!(&bytes[32..64], &[0x11; 32]);
    assert_eq!(&bytes[64..96], CONTRACT_SHA256.as_bytes());
    assert_eq!(&bytes[96..128], &[0x22; 32]);
    assert_eq!(&bytes[128..160], EMPTY_SHA256.as_bytes());
    assert_eq!(
        decode_frame(bytes).unwrap(),
        role_frame(MessageKind::Start, 0x0102_0304, &[], 0)
    );
}

#[test]
fn decoder_rejects_header_corruption_trailing_and_payload_corruption() {
    let selector = encode_host_selector(HostSelectorV1 {
        phase_code: 2,
        instance_class_code: 101,
        instance_ordinal: 1,
    });
    let frame = Frame::new(
        MessageKind::HostStart,
        1,
        0,
        0,
        RoleId::Supervisor,
        0,
        binding(),
        &selector,
    )
    .unwrap();
    let pristine = frame.encode().as_bytes().to_vec();

    let mut bytes = pristine.clone();
    bytes[0] ^= 1;
    assert_eq!(decode_frame(&bytes), Err(ProtocolError::WrongMagic));
    let mut bytes = pristine.clone();
    bytes[9] = 2;
    assert_eq!(decode_frame(&bytes), Err(ProtocolError::WrongVersion));
    let mut bytes = pristine.clone();
    bytes[10..12].copy_from_slice(&99_u16.to_be_bytes());
    assert_eq!(decode_frame(&bytes), Err(ProtocolError::UnknownKind));
    let mut bytes = pristine.clone();
    bytes[12..16].fill(0);
    assert_eq!(decode_frame(&bytes), Err(ProtocolError::ZeroSequence));
    let mut bytes = pristine.clone();
    bytes[20..22].copy_from_slice(&2_u16.to_be_bytes());
    assert_eq!(decode_frame(&bytes), Err(ProtocolError::WrongProfile));
    let mut bytes = pristine.clone();
    bytes[22..24].copy_from_slice(&99_u16.to_be_bytes());
    assert_eq!(decode_frame(&bytes), Err(ProtocolError::UnknownRole));
    let mut bytes = pristine.clone();
    bytes[26..28].copy_from_slice(&1_u16.to_be_bytes());
    assert_eq!(decode_frame(&bytes), Err(ProtocolError::NonzeroFlags));
    let mut bytes = pristine.clone();
    bytes.push(0);
    assert_eq!(
        decode_frame(&bytes),
        Err(ProtocolError::PayloadLengthMismatch)
    );
    let mut bytes = pristine.clone();
    bytes[160] ^= 1;
    assert_eq!(
        decode_frame(&bytes),
        Err(ProtocolError::PayloadDigestMismatch)
    );
}

#[test]
fn frame_shape_closes_role_host_status_and_payload_combinations() {
    assert_eq!(
        Frame::new(
            MessageKind::Start,
            1,
            0,
            1,
            RoleId::Writer,
            0,
            binding(),
            &[]
        ),
        Err(ProtocolError::WrongItemBinding)
    );
    assert_eq!(
        Frame::new(
            MessageKind::HostStart,
            1,
            1,
            0,
            RoleId::Supervisor,
            0,
            binding(),
            &[0; 8]
        ),
        Err(ProtocolError::WrongItemBinding)
    );
    assert_eq!(
        Frame::new(
            MessageKind::Start,
            1,
            1,
            1,
            RoleId::Writer,
            1,
            binding(),
            &[]
        ),
        Err(ProtocolError::WrongStatus)
    );
    assert_eq!(
        Frame::new(
            MessageKind::Start,
            1,
            1,
            1,
            RoleId::Writer,
            0,
            binding(),
            &[1]
        ),
        Err(ProtocolError::WrongPayloadLength)
    );

    let category = (ErrorCategory::Containment as u32).to_be_bytes();
    assert!(Frame::new(
        MessageKind::Abort,
        1,
        1,
        1,
        RoleId::Writer,
        ErrorCategory::Containment.header_code(),
        binding(),
        &category
    )
    .is_ok());
    assert_eq!(
        Frame::new(
            MessageKind::Abort,
            1,
            1,
            1,
            RoleId::Writer,
            ErrorCategory::LimitExceeded.header_code(),
            binding(),
            &category
        ),
        Err(ProtocolError::CategoryMismatch)
    );
}

#[test]
fn stream_verifier_binds_nonce_contract_manifest_and_sequence() {
    let first = role_frame(MessageKind::Start, 1, &[], 0).encode();
    let second = role_frame(MessageKind::Continue, 2, &[], 0).encode();
    let mut duplicate = StreamVerifier::new(binding());
    assert_eq!(duplicate.next_sequence(), 1);
    duplicate.accept(first.as_bytes()).unwrap();
    assert_eq!(duplicate.next_sequence(), 2);
    assert_eq!(
        duplicate.accept(first.as_bytes()),
        Err(ProtocolError::SequenceMismatch)
    );
    assert_eq!(
        duplicate.accept(second.as_bytes()),
        Err(ProtocolError::Poisoned)
    );

    let mut valid = StreamVerifier::new(binding());
    valid.accept(first.as_bytes()).unwrap();
    valid.accept(second.as_bytes()).unwrap();
    assert_eq!(valid.next_sequence(), 3);

    let foreign = Frame::new(
        MessageKind::Start,
        1,
        7,
        3,
        RoleId::Writer,
        0,
        StreamBinding::unverified_for_codec(digest(0x44), digest(0x22)),
        &[],
    )
    .unwrap()
    .encode();
    assert_eq!(
        StreamVerifier::new(binding()).accept(foreign.as_bytes()),
        Err(ProtocolError::BindingMismatch)
    );
}

#[test]
fn role_stream_verifier_freezes_the_manifest_selected_tuple() {
    let mut wrong = RoleStreamVerifier::new(binding(), 7, 3, RoleId::Writer).unwrap();
    let wrong_tuple = Frame::new(
        MessageKind::Start,
        1,
        8,
        3,
        RoleId::Writer,
        0,
        binding(),
        &[],
    )
    .unwrap()
    .encode();
    assert_eq!(
        wrong.accept(wrong_tuple.as_bytes()),
        Err(ProtocolError::WrongItemBinding)
    );
    assert_eq!(
        wrong.accept(
            role_frame(MessageKind::Start, 1, &[], 0)
                .encode()
                .as_bytes()
        ),
        Err(ProtocolError::Poisoned)
    );

    let mut valid = RoleStreamVerifier::new(binding(), 7, 3, RoleId::Writer).unwrap();
    assert_eq!(
        valid
            .accept(
                role_frame(MessageKind::Start, 1, &[], 0)
                    .encode()
                    .as_bytes()
            )
            .unwrap()
            .header
            .sequence,
        1
    );
    assert_eq!(
        RoleStreamVerifier::new(binding(), 0, 1, RoleId::Writer),
        Err(ProtocolError::WrongItemBinding)
    );
}

#[test]
fn authoritative_protocols_fail_closed_until_manifest_digests_are_frozen() {
    for kind in ManifestKindV1::ALL {
        assert_eq!(
            StreamBinding::for_manifest(digest(0x11), kind),
            Err(ProtocolError::ManifestDigestNotFrozen(kind))
        );
    }
    assert_eq!(
        RoleProtocolV1::new(
            digest(0x11),
            RoleSelectionV1::Positive {
                case_id: 1,
                role_id: RoleId::Writer,
            },
        ),
        Err(ProtocolError::ManifestDigestNotFrozen(
            ManifestKindV1::PositiveCases
        ))
    );
    assert_eq!(
        HostProtocolV1::new(
            digest(0x11),
            HostSelectorV1::from_entry(&ACCEPTANCE_SCHEDULE[0]),
        ),
        Err(ProtocolError::ManifestDigestNotFrozen(
            ManifestKindV1::AcceptanceSchedule
        ))
    );
}

#[test]
fn role_specs_are_derived_only_from_the_closed_manifests() {
    let role_plans = [
        (RoleId::Writer, RolePlan::Writer),
        (RoleId::Contender, RolePlan::Contender),
        (RoleId::ReleaseProbe, RolePlan::ReleaseProbe),
        (RoleId::Reader, RolePlan::Reader),
    ];
    for (role_id, plan) in role_plans {
        for subattempt_id in 1..=3 {
            let spec = derive_role_protocol_spec(RoleSelectionV1::Characterization {
                subattempt_id,
                role_id,
            })
            .unwrap();
            assert_eq!(spec.manifest_kind, ManifestKindV1::Characterization);
            assert_eq!(spec.case_or_probe_id, 1);
            assert_eq!(spec.fixture_or_subattempt_id, subattempt_id);
            assert_eq!(spec.role_id, role_id);
            assert_eq!(spec.plan, plan);
            assert_eq!(spec.completion, RoleCompletionV1::RoleTerminal);
            assert_eq!(spec.output_policy, OutputPolicyV1::Empty);
        }
        for case_id in 1..=20 {
            let spec =
                derive_role_protocol_spec(RoleSelectionV1::Positive { case_id, role_id }).unwrap();
            assert_eq!(spec.manifest_kind, ManifestKindV1::PositiveCases);
            assert_eq!(spec.case_or_probe_id, case_id);
            assert_eq!(spec.fixture_or_subattempt_id, case_id);
            assert_eq!(spec.role_id, role_id);
            assert_eq!(spec.plan, plan);
            assert_eq!(spec.completion, RoleCompletionV1::RoleTerminal);
            assert_eq!(spec.output_policy, OutputPolicyV1::Empty);
        }
    }

    for invalid_role in [
        RoleId::Supervisor,
        RoleId::Boundary,
        RoleId::JourneyAggregate,
    ] {
        assert_eq!(
            derive_role_protocol_spec(RoleSelectionV1::Positive {
                case_id: 1,
                role_id: invalid_role,
            }),
            Err(ProtocolError::InvalidRoleSelection)
        );
    }
    for invalid_case in [0, 21] {
        assert_eq!(
            derive_role_protocol_spec(RoleSelectionV1::Positive {
                case_id: invalid_case,
                role_id: RoleId::Writer,
            }),
            Err(ProtocolError::InvalidRoleSelection)
        );
    }
    for invalid_subattempt in [0, 4] {
        assert_eq!(
            derive_role_protocol_spec(RoleSelectionV1::Characterization {
                subattempt_id: invalid_subattempt,
                role_id: RoleId::Writer,
            }),
            Err(ProtocolError::InvalidRoleSelection)
        );
    }
}

#[test]
fn boundary_specs_freeze_every_termination_and_special_output_mapping() {
    for probe_id in 1..=41 {
        let count = probe_subattempt_count(probe_id).unwrap();
        for subattempt_id in 1..=count {
            let result = derive_role_protocol_spec(RoleSelectionV1::Boundary {
                probe_id,
                subattempt_id,
            });
            if matches!(probe_id, 40 | 41) {
                assert_eq!(result, Err(ProtocolError::NoRoleChannel));
                continue;
            }
            let spec = result.unwrap();
            assert_eq!(spec.manifest_kind, ManifestKindV1::BoundaryProbes);
            assert_eq!(spec.case_or_probe_id, probe_id);
            assert_eq!(spec.fixture_or_subattempt_id, subattempt_id);
            assert_eq!(spec.role_id, RoleId::Boundary);
            let expected = match probe_id {
                1 | 5 | 7 | 12 | 37 | 38 | 39 => (
                    RolePlan::BoundaryKernelTerminal,
                    RoleCompletionV1::KernelTerminal,
                    OutputPolicyV1::Empty,
                ),
                9 => (
                    RolePlan::BoundarySupervisorAfterContinue,
                    RoleCompletionV1::SupervisorTerminal(SupervisorTerminalV1::StdoutOverflow),
                    OutputPolicyV1::StdoutOverflow,
                ),
                10 => (
                    RolePlan::BoundarySupervisorAfterRoleTerminal,
                    RoleCompletionV1::SupervisorTerminal(SupervisorTerminalV1::TrailingControl),
                    OutputPolicyV1::TrailingControlByte,
                ),
                11 => (
                    RolePlan::BoundaryRoleTerminal,
                    RoleCompletionV1::SupervisorTerminal(SupervisorTerminalV1::LateStdout),
                    OutputPolicyV1::LateStdoutByte,
                ),
                14 => (
                    RolePlan::BoundarySupervisorAfterRoleTerminal,
                    RoleCompletionV1::SupervisorTerminal(
                        SupervisorTerminalV1::PostTerminalLinkCount,
                    ),
                    OutputPolicyV1::Empty,
                ),
                15 => (
                    RolePlan::BoundaryPreStart,
                    RoleCompletionV1::SupervisorTerminal(SupervisorTerminalV1::WrongDevicePreStart),
                    OutputPolicyV1::Empty,
                ),
                16 => (
                    RolePlan::BoundaryPreStart,
                    RoleCompletionV1::SupervisorTerminal(
                        SupervisorTerminalV1::WritableMountPreStart,
                    ),
                    OutputPolicyV1::Empty,
                ),
                17 => (
                    RolePlan::BoundaryPreStart,
                    RoleCompletionV1::SupervisorTerminal(SupervisorTerminalV1::InheritedFdPostExec),
                    OutputPolicyV1::Empty,
                ),
                18 => (
                    RolePlan::BoundaryPreStart,
                    RoleCompletionV1::SupervisorTerminal(
                        SupervisorTerminalV1::InheritedSocketPostExec,
                    ),
                    OutputPolicyV1::Empty,
                ),
                _ => (
                    RolePlan::BoundaryRoleTerminal,
                    RoleCompletionV1::RoleTerminal,
                    OutputPolicyV1::Empty,
                ),
            };
            assert_eq!((spec.plan, spec.completion, spec.output_policy), expected);
        }
        assert_eq!(
            derive_role_protocol_spec(RoleSelectionV1::Boundary {
                probe_id,
                subattempt_id: count + 1,
            }),
            Err(ProtocolError::InvalidRoleSelection)
        );
    }
    assert_eq!(
        derive_role_protocol_spec(RoleSelectionV1::Boundary {
            probe_id: 0,
            subattempt_id: 1,
        }),
        Err(ProtocolError::InvalidRoleSelection)
    );
    assert_eq!(
        derive_role_protocol_spec(RoleSelectionV1::Boundary {
            probe_id: 42,
            subattempt_id: 1,
        }),
        Err(ProtocolError::InvalidRoleSelection)
    );
}

#[test]
fn fixed_payload_codecs_round_trip_and_reject_reserved_data() {
    let selector = HostSelectorV1 {
        phase_code: 2,
        instance_class_code: 241,
        instance_ordinal: 10,
    };
    let encoded = encode_host_selector(selector);
    assert_eq!(encoded, [0, 2, 0, 241, 0, 10, 0, 0]);
    assert_eq!(decode_host_selector(&encoded), Ok(selector));
    let mut bad = encoded;
    bad[7] = 1;
    assert_eq!(
        decode_host_selector(&bad),
        Err(ProtocolError::ReservedFieldNonzero)
    );

    let words = core::array::from_fn(|index| index as u64 * 0x0102_0304_0506_0708);
    let measurement = MeasurementV1::from_words(words);
    let measurement_bytes = encode_measurement(measurement);
    assert_eq!(&measurement_bytes[8..16], &words[1].to_be_bytes());
    assert_eq!(decode_measurement(&measurement_bytes), Ok(measurement));

    let terminal = TerminalStateV1 {
        primary_category: ErrorCategory::CanonicalMismatch as u32,
        active_item_code: active_item_code(20, 20),
        guest_cleanup_proven: 1,
        write_may_have_been_dispatched: 1,
        writer_acknowledged: 1,
        roles_reaped: 1,
        completed_case_count: 19,
        completed_subattempt_count: 0,
    };
    assert_eq!(
        decode_terminal_state(&encode_terminal_state(terminal)),
        Ok(terminal)
    );
    let payload = encode_terminal_payload(digest(0x77), terminal);
    assert_eq!(
        decode_terminal_payload(&payload),
        Ok((digest(0x77), terminal))
    );
    let commitment = TerminalCommitmentV1 {
        challenge: digest(0x77),
        state: terminal,
    };
    assert_eq!(commitment.verify_echo(&payload), Ok(()));
    assert_eq!(
        TerminalCommitmentV1 {
            challenge: digest(0x78),
            state: terminal
        }
        .verify_echo(&payload),
        Err(ProtocolError::ChallengeMismatch)
    );
    let other_state = TerminalStateV1 {
        completed_case_count: terminal.completed_case_count - 1,
        ..terminal
    };
    assert_eq!(
        TerminalCommitmentV1 {
            challenge: digest(0x77),
            state: other_state
        }
        .verify_echo(&payload),
        Err(ProtocolError::CommittedStateMismatch)
    );
}

#[test]
fn terminal_frame_commits_identical_status_and_state_category() {
    let state = successful_terminal(&ACCEPTANCE_SCHEDULE[0]).unwrap();
    let payload = encode_terminal_state(state);
    let ready = Frame::new(
        MessageKind::TerminalReady,
        142,
        0,
        0,
        RoleId::Supervisor,
        0,
        binding(),
        &payload,
    )
    .unwrap();
    assert_eq!(decode_frame(ready.encode().as_bytes()).unwrap(), ready);

    let failed = TerminalStateV1 {
        primary_category: ErrorCategory::MeasurementFailure as u32,
        active_item_code: active_item_code(7, 7),
        roles_reaped: 1,
        ..TerminalStateV1::default()
    };
    assert_eq!(
        Frame::new(
            MessageKind::TerminalReady,
            10,
            0,
            0,
            RoleId::Supervisor,
            ErrorCategory::ChallengeFailure.header_code(),
            binding(),
            &encode_terminal_state(failed)
        ),
        Err(ProtocolError::CategoryMismatch)
    );
}

fn event(direction: Direction, kind: MessageKind) -> DirectedControlEvent {
    DirectedControlEvent {
        direction,
        event: ControlEvent::Frame(kind),
    }
}

fn eof_event(direction: Direction) -> DirectedControlEvent {
    DirectedControlEvent {
        direction,
        event: ControlEvent::Eof,
    }
}

#[test]
fn every_role_automaton_accepts_only_the_frozen_total_order() {
    use Direction::{CollectorToSupervisor as Cs, SupervisorToCollector as Sc};
    let plans: &[(RolePlan, &[DirectedControlEvent])] = &[
        (
            RolePlan::Writer,
            &[
                event(Sc, MessageKind::Start),
                event(Cs, MessageKind::Ready),
                event(Sc, MessageKind::Continue),
                event(Cs, MessageKind::WriterCommitted),
                event(Cs, MessageKind::LockObserved),
                event(Sc, MessageKind::Continue),
                event(Cs, MessageKind::HandlesDropped),
                event(Sc, MessageKind::Continue),
                event(Cs, MessageKind::RoleTerminal),
                eof_event(Cs),
                eof_event(Sc),
            ],
        ),
        (
            RolePlan::Contender,
            &[
                event(Sc, MessageKind::Start),
                event(Cs, MessageKind::Ready),
                event(Sc, MessageKind::Continue),
                event(Cs, MessageKind::ExpectedLockRejected),
                event(Cs, MessageKind::RoleTerminal),
                eof_event(Cs),
                eof_event(Sc),
            ],
        ),
        (
            RolePlan::ReleaseProbe,
            &[
                event(Sc, MessageKind::Start),
                event(Cs, MessageKind::Ready),
                event(Sc, MessageKind::Continue),
                event(Cs, MessageKind::LockObserved),
                event(Sc, MessageKind::Continue),
                event(Cs, MessageKind::HandlesDropped),
                event(Sc, MessageKind::Continue),
                event(Cs, MessageKind::RoleTerminal),
                eof_event(Cs),
                eof_event(Sc),
            ],
        ),
        (
            RolePlan::Reader,
            &[
                event(Sc, MessageKind::Start),
                event(Cs, MessageKind::Ready),
                event(Sc, MessageKind::Continue),
                event(Cs, MessageKind::RoleTerminal),
                eof_event(Cs),
                eof_event(Sc),
            ],
        ),
        (
            RolePlan::BoundaryKernelTerminal,
            &[
                event(Sc, MessageKind::Start),
                event(Cs, MessageKind::Ready),
                event(Sc, MessageKind::Continue),
                eof_event(Cs),
                eof_event(Sc),
            ],
        ),
        (
            RolePlan::BoundarySupervisorAfterContinue,
            &[
                event(Sc, MessageKind::Start),
                event(Cs, MessageKind::Ready),
                event(Sc, MessageKind::Continue),
                eof_event(Sc),
                eof_event(Cs),
            ],
        ),
        (
            RolePlan::BoundarySupervisorAfterRoleTerminal,
            &[
                event(Sc, MessageKind::Start),
                event(Cs, MessageKind::Ready),
                event(Sc, MessageKind::Continue),
                event(Cs, MessageKind::RoleTerminal),
                eof_event(Sc),
                eof_event(Cs),
            ],
        ),
        (RolePlan::BoundaryPreStart, &[eof_event(Sc), eof_event(Cs)]),
    ];

    for (plan, events) in plans {
        let mut automaton = RoleAutomaton::new(*plan);
        assert!(!automaton.is_complete());
        for event in *events {
            automaton.accept(*event).unwrap();
        }
        assert!(automaton.is_complete());
        assert_eq!(automaton.accept(events[0]), Err(ProtocolError::LateEvent));
        assert!(!automaton.is_complete());
        assert_eq!(automaton.accept(events[0]), Err(ProtocolError::Poisoned));
    }

    let mut wrong_frame = RoleAutomaton::new(RolePlan::Writer);
    assert_eq!(
        wrong_frame.accept(event(Cs, MessageKind::Ready)),
        Err(ProtocolError::OutOfState)
    );
    assert_eq!(
        wrong_frame.accept(eof_event(Sc)),
        Err(ProtocolError::Poisoned)
    );

    let mut early_eof = RoleAutomaton::new(RolePlan::Writer);
    assert_eq!(
        early_eof.accept(eof_event(Sc)),
        Err(ProtocolError::EarlyEof)
    );
    assert!(!early_eof.is_complete());
}

#[test]
fn role_abort_replaces_only_the_next_sender_then_closes_both_directions() {
    use Direction::{CollectorToSupervisor as Cs, SupervisorToCollector as Sc};
    let mut wrong_sender = RoleAutomaton::new(RolePlan::Writer);
    wrong_sender.accept(event(Sc, MessageKind::Start)).unwrap();
    wrong_sender.accept(event(Cs, MessageKind::Ready)).unwrap();
    assert_eq!(
        wrong_sender.accept(event(Cs, MessageKind::Abort)),
        Err(ProtocolError::WrongDirection)
    );
    assert_eq!(
        wrong_sender.accept(event(Sc, MessageKind::Abort)),
        Err(ProtocolError::Poisoned)
    );

    let mut automaton = RoleAutomaton::new(RolePlan::Writer);
    automaton.accept(event(Sc, MessageKind::Start)).unwrap();
    automaton.accept(event(Cs, MessageKind::Ready)).unwrap();
    automaton.accept(event(Sc, MessageKind::Abort)).unwrap();
    automaton.accept(eof_event(Sc)).unwrap();
    automaton.accept(eof_event(Cs)).unwrap();
    assert!(automaton.is_complete());
}

fn host_frame(
    direction: HostDirection,
    kind: MessageKind,
    measurement: Option<MeasurementIdentity>,
) -> HostEvent {
    host_frame_status(direction, kind, 0, measurement)
}

fn host_frame_status(
    direction: HostDirection,
    kind: MessageKind,
    status: u16,
    measurement: Option<MeasurementIdentity>,
) -> HostEvent {
    HostEvent::Frame {
        direction,
        kind,
        status,
        measurement,
    }
}

#[test]
fn host_automata_enforce_all_cardinalities_and_measurement_identities() {
    use HostDirection::{HostToSupervisor as Hs, SupervisorToHost as Sh};
    for entry in CHARACTERIZATION_SCHEDULE
        .iter()
        .chain(ACCEPTANCE_SCHEDULE.iter())
    {
        let mut automaton = HostAutomaton::new(entry);
        automaton
            .accept(host_frame(Hs, MessageKind::HostStart, None))
            .unwrap();
        automaton
            .accept(host_frame(Sh, MessageKind::HostReady, None))
            .unwrap();
        let count = measurement_count(entry);
        if count == 0 {
            automaton.accept(HostEvent::Eof(Sh)).unwrap();
        } else {
            for index in 0..count {
                automaton
                    .accept(host_frame(
                        Sh,
                        MessageKind::Measurement,
                        expected_measurement(entry, index),
                    ))
                    .unwrap();
            }
            automaton
                .accept(host_frame(Sh, MessageKind::TerminalReady, None))
                .unwrap();
            automaton
                .accept(host_frame(Hs, MessageKind::Challenge, None))
                .unwrap();
            automaton.accept(HostEvent::Eof(Hs)).unwrap();
            automaton
                .accept(host_frame(Sh, MessageKind::Terminal, None))
                .unwrap();
            automaton.accept(HostEvent::Eof(Sh)).unwrap();
        }
        assert!(automaton.is_complete(), "{} did not complete", entry.name);
    }
}

#[test]
fn host_automaton_rejects_a_wrong_measurement_tuple() {
    use HostDirection::{HostToSupervisor as Hs, SupervisorToHost as Sh};
    let entry = &ACCEPTANCE_SCHEDULE[0];
    let mut automaton = HostAutomaton::new(entry);
    automaton
        .accept(host_frame(Hs, MessageKind::HostStart, None))
        .unwrap();
    automaton
        .accept(host_frame(Sh, MessageKind::HostReady, None))
        .unwrap();
    let mut wrong = expected_measurement(entry, 0).unwrap();
    wrong.scope_code = 2;
    assert_eq!(
        automaton.accept(host_frame(Sh, MessageKind::Measurement, Some(wrong))),
        Err(ProtocolError::OutOfState)
    );
}

#[test]
fn host_automaton_allows_only_nonzero_early_failure_terminal_status() {
    use HostDirection::{HostToSupervisor as Hs, SupervisorToHost as Sh};
    let entry = &ACCEPTANCE_SCHEDULE[0];
    let to_host_eof = || {
        let mut automaton = HostAutomaton::new(entry);
        automaton
            .accept(host_frame(Hs, MessageKind::HostStart, None))
            .unwrap();
        automaton
            .accept(host_frame(Sh, MessageKind::HostReady, None))
            .unwrap();
        automaton
            .accept(host_frame(
                Sh,
                MessageKind::Measurement,
                expected_measurement(entry, 0),
            ))
            .unwrap();
        automaton
            .accept(host_frame_status(
                Sh,
                MessageKind::TerminalReady,
                ErrorCategory::MeasurementFailure.header_code(),
                None,
            ))
            .unwrap();
        automaton
            .accept(host_frame(Hs, MessageKind::Challenge, None))
            .unwrap();
        automaton.accept(HostEvent::Eof(Hs)).unwrap();
        automaton
    };

    let mut wrong_status = to_host_eof();
    assert_eq!(
        wrong_status.accept(host_frame(Sh, MessageKind::Terminal, None)),
        Err(ProtocolError::OutOfState)
    );
    assert_eq!(
        wrong_status.accept(host_frame_status(
            Sh,
            MessageKind::Terminal,
            ErrorCategory::MeasurementFailure.header_code(),
            None,
        )),
        Err(ProtocolError::Poisoned)
    );
    assert!(!wrong_status.is_complete());

    let mut automaton = to_host_eof();
    automaton
        .accept(host_frame_status(
            Sh,
            MessageKind::Terminal,
            ErrorCategory::MeasurementFailure.header_code(),
            None,
        ))
        .unwrap();
    automaton.accept(HostEvent::Eof(Sh)).unwrap();
    assert!(automaton.is_complete());
}

#[test]
fn host_abort_replaces_only_the_next_sender_frame_and_forbids_handshake() {
    use HostDirection::{HostToSupervisor as Hs, SupervisorToHost as Sh};
    let entry = &ACCEPTANCE_SCHEDULE[0];
    let after_start = || {
        let mut automaton = HostAutomaton::new(entry);
        automaton
            .accept(host_frame(Hs, MessageKind::HostStart, None))
            .unwrap();
        automaton
    };

    let mut wrong_sender = after_start();
    assert_eq!(
        wrong_sender.accept(host_frame_status(
            Hs,
            MessageKind::Abort,
            ErrorCategory::HostContract.header_code(),
            None
        )),
        Err(ProtocolError::OutOfState)
    );
    assert_eq!(
        wrong_sender.accept(host_frame_status(
            Sh,
            MessageKind::Abort,
            ErrorCategory::Containment.header_code(),
            None,
        )),
        Err(ProtocolError::Poisoned)
    );

    let mut forbidden_handshake = after_start();
    forbidden_handshake
        .accept(host_frame_status(
            Sh,
            MessageKind::Abort,
            ErrorCategory::Containment.header_code(),
            None,
        ))
        .unwrap();
    assert_eq!(
        forbidden_handshake.accept(host_frame(Hs, MessageKind::Challenge, None)),
        Err(ProtocolError::OutOfState)
    );
    assert_eq!(
        forbidden_handshake.accept(HostEvent::Eof(Sh)),
        Err(ProtocolError::Poisoned)
    );

    let mut automaton = after_start();
    automaton
        .accept(host_frame_status(
            Sh,
            MessageKind::Abort,
            ErrorCategory::Containment.header_code(),
            None,
        ))
        .unwrap();
    automaton.accept(HostEvent::Eof(Sh)).unwrap();
    assert!(automaton.is_complete());

    let after_terminal_ready = || {
        let mut automaton = HostAutomaton::new(entry);
        automaton
            .accept(host_frame(Hs, MessageKind::HostStart, None))
            .unwrap();
        automaton
            .accept(host_frame(Sh, MessageKind::HostReady, None))
            .unwrap();
        automaton
            .accept(host_frame(
                Sh,
                MessageKind::Measurement,
                expected_measurement(entry, 0),
            ))
            .unwrap();
        automaton
            .accept(host_frame_status(
                Sh,
                MessageKind::TerminalReady,
                ErrorCategory::MeasurementFailure.header_code(),
                None,
            ))
            .unwrap();
        automaton
    };
    for direction in [Hs, Sh] {
        let mut automaton = after_terminal_ready();
        assert_eq!(
            automaton.accept(host_frame_status(
                direction,
                MessageKind::Abort,
                ErrorCategory::ProtocolFailure.header_code(),
                None,
            )),
            Err(ProtocolError::OutOfState)
        );
        assert!(!automaton.is_complete());
        assert_eq!(
            automaton.accept(host_frame(Hs, MessageKind::Challenge, None)),
            Err(ProtocolError::Poisoned)
        );
    }
}

#[test]
fn probe_39_packets_are_fixed_and_reserved_bytes_are_rejected() {
    let launcher = encode_launcher_child();
    assert_eq!(launcher.len(), 24);
    assert_eq!(&launcher[0..8], b"ENGC2LC1");
    assert_eq!(&launcher[8..12], &[0, 1, 0, 39]);
    assert_eq!(&launcher[12..16], &[0, 0, 0, 2]);
    assert_eq!(&launcher[16..20], &[0, 0, 0, 3]);
    assert_eq!(decode_launcher_child(&launcher), Ok(LauncherChildV1));
    let mut bad = launcher;
    bad[23] = 1;
    assert_eq!(
        decode_launcher_child(&bad),
        Err(ProtocolError::ReservedFieldNonzero)
    );

    let trigger = encode_root_trigger();
    assert_eq!(trigger.len(), 16);
    assert_eq!(&trigger[0..8], b"ENGC2RT1");
    assert_eq!(&trigger[8..14], &[0, 1, 0, 39, 0, 1]);
    assert_eq!(decode_root_trigger(&trigger), Ok(RootTriggerV1));
    let mut bad = trigger;
    bad[15] = 1;
    assert_eq!(
        decode_root_trigger(&bad),
        Err(ProtocolError::ReservedFieldNonzero)
    );
}

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

assert_not_impl!(StreamVerifier: Clone);
assert_not_impl!(StreamVerifier: Copy);
assert_not_impl!(RoleStreamVerifier: Clone);
assert_not_impl!(RoleStreamVerifier: Copy);
assert_not_impl!(RoleAutomaton: Clone);
assert_not_impl!(RoleAutomaton: Copy);
assert_not_impl!(RoleProtocolV1: Clone);
assert_not_impl!(RoleProtocolV1: Copy);
assert_not_impl!(HostAutomaton: Clone);
assert_not_impl!(HostAutomaton: Copy);
assert_not_impl!(HostProtocolV1: Clone);
assert_not_impl!(HostProtocolV1: Copy);
