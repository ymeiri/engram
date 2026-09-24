#[allow(dead_code)]
#[path = "../src/native_c2b2a.rs"]
mod native_c2b2a;

#[allow(dead_code)]
#[path = "../native-c2b2a-payload/src/contract.rs"]
mod contract;

#[allow(dead_code)]
#[path = "../native-c2b2a-payload/src/protocol.rs"]
mod payload_protocol;

use native_c2b2a::*;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

const FREEZE: &[u8] = include_bytes!(
    "../../evals/native_memory_pilot_v1/\
NATIVE_STRICT_SUCCESSOR_STAGE_C2B2A_CALIBRATION_CONTAINMENT_FROZEN_2026-09-07.md"
);
const C2B2_RESEARCH: &[u8] = include_bytes!(
    "../../evals/native_memory_pilot_v1/\
NATIVE_STRICT_SUCCESSOR_STAGE_C2B2_PERSISTENT_CONTAINMENT_RESEARCH_2026-09-06.md"
);
const C2B2_RESEARCH_REVIEW: &[u8] = include_bytes!(
    "../../evals/native_memory_pilot_v1/\
NATIVE_STRICT_SUCCESSOR_STAGE_C2B2_PERSISTENT_CONTAINMENT_REVIEW_2026-09-06.md"
);
const ACCEPTED_C2B1_REPORT: &[u8] = include_bytes!(
    "../../evals/native_memory_pilot_v1/\
NATIVE_STRICT_SUCCESSOR_STAGE_C2B1_ACCEPTED_2026-09-06.md"
);
const C2B1_FREEZE: &[u8] = include_bytes!(
    "../../evals/native_memory_pilot_v1/\
NATIVE_STRICT_SUCCESSOR_STAGE_C2B1_MEMORY_ACQUISITION_FROZEN_2026-09-06.md"
);
const C2B1_ACQUISITION_SOURCE: &[u8] =
    include_bytes!("../src/native_successor_semantic/acquisition.rs");
const NATIVE_VM_SOURCE: &[u8] = include_bytes!("../src/native_vm.rs");
const NATIVE_VM_FOUNDATION_RECORD: &[u8] = include_bytes!(
    "../../evals/native_memory_pilot_v1/NATIVE_VM_COLLECTOR_FOUNDATION_2026-09-05.md"
);
const EVALUATOR_MANIFEST: &[u8] = include_bytes!("../Cargo.toml");
const WORKSPACE_MANIFEST: &[u8] = include_bytes!("../../Cargo.toml");
const WORKSPACE_LOCK: &[u8] = include_bytes!("../../Cargo.lock");

const CROSS_INSTANCES: [C2b2aInstanceName; 11] = C2B2A_CAMPAIGN_ORDER;
const SELECTOR_CORPUS_SHA256: &str =
    "2b3f3d9eb7553c7c93c40291a298d63d284ab5871f5cb9f807fd72c6070db9de";
const IDENTITY_CORPUS_SHA256: &str =
    "90c60ed8a9766c6b18fa6705836e2e2c76f10c1dd8bba2a8280a18dd3603cba9";
const SUCCESS_TERMINAL_CORPUS_SHA256: &str =
    "93846841325321bc99ddd780a4f06a4080876652e62f1b51a64a8863def6477f";
const FRAME_CORPUS_SHA256: &str =
    "489ac97d9a5d5a1c69504e922bb6b6ea1fbb287b11ca26427f17b0cdf1ffa78e";
const FAILURE_FRAME_CORPUS_SHA256: &str =
    "7df34eb96c6cdd2a53ca3fc60e347fded1db49d76cc6c2b78ef31c396cd8f31b";

fn digest(byte: u8) -> C2b2aDigest {
    C2b2aDigest::from_bytes([byte; 32])
}

fn nonce(byte: u8) -> C2b2aNonce {
    C2b2aNonce::from_bytes([byte; 32])
}

fn challenge(byte: u8) -> C2b2aChallenge {
    C2b2aChallenge::from_bytes([byte; 32])
}

fn parity_campaign(index: usize) -> C2b2aCampaignBindings {
    let nonce_byte = u8::try_from(index + 1).unwrap();
    C2b2aCampaignBindings {
        nonce: nonce(nonce_byte),
        contract_sha256: C2B2A_CONTRACT_SHA256,
        characterization_schedule_sha256: digest(0x33),
        acceptance_schedule_sha256: digest(0x44),
    }
}

fn parity_challenge(index: usize) -> C2b2aChallenge {
    challenge(0x80 + u8::try_from(index + 1).unwrap())
}

fn payload_entry(instance: C2b2aInstanceName) -> &'static contract::ScheduleEntry {
    let selector = instance_spec(instance).selector;
    contract::select_schedule_entry(
        selector.phase_code,
        selector.instance_class_code,
        selector.instance_ordinal,
    )
    .unwrap()
}

fn payload_selector(entry: &contract::ScheduleEntry) -> payload_protocol::HostSelectorV1 {
    payload_protocol::HostSelectorV1::from_entry(entry)
}

fn payload_binding(
    instance: C2b2aInstanceName,
    campaign: C2b2aCampaignBindings,
) -> payload_protocol::StreamBinding {
    let manifest = if instance == C2b2aInstanceName::C01 {
        campaign.characterization_schedule_sha256
    } else {
        campaign.acceptance_schedule_sha256
    };
    payload_protocol::StreamBinding::unverified_for_codec(
        contract::Digest32(*campaign.nonce.as_bytes()),
        contract::Digest32(*manifest.as_bytes()),
    )
}

fn payload_role(role: C2b2aRoleId) -> contract::RoleId {
    match role {
        C2b2aRoleId::Supervisor => contract::RoleId::Supervisor,
        C2b2aRoleId::Boundary => contract::RoleId::Boundary,
        C2b2aRoleId::Writer => contract::RoleId::Writer,
        C2b2aRoleId::Contender => contract::RoleId::Contender,
        C2b2aRoleId::Release => contract::RoleId::ReleaseProbe,
        C2b2aRoleId::Reader => contract::RoleId::Reader,
        C2b2aRoleId::JourneyAggregate => contract::RoleId::JourneyAggregate,
    }
}

fn payload_identity(key: C2b2aMeasurementKey) -> contract::MeasurementIdentity {
    contract::MeasurementIdentity {
        item_id: key.case_or_probe_id,
        subitem_id: key.fixture_or_subattempt_id,
        scope_code: u16::try_from(key.scope_code).unwrap(),
        role_id: payload_role(key.role_id),
    }
}

fn payload_measurement(identity: contract::MeasurementIdentity) -> contract::MeasurementV1 {
    let mut measurement = contract::MeasurementV1 {
        scope_code: u64::from(identity.scope_code),
        monotonic_elapsed_ns: 1,
        ..contract::MeasurementV1::default()
    };
    if identity.role_id == contract::RoleId::Boundary && identity.item_id == 17 {
        measurement.descriptor_type_bitmap = 1 << 7;
    }
    if identity.role_id == contract::RoleId::Boundary && identity.item_id == 18 {
        measurement.descriptor_type_bitmap = 1 << 14;
    }
    measurement
}

fn host_wire(frame: &C2b2aEncodedFrame) -> Vec<u8> {
    let mut bytes = [0_u8; C2B2A_MAX_HOST_FRAME_BYTES];
    let len = frame.copy_wire_into(&mut bytes);
    bytes[..len].to_vec()
}

fn payload_frame(
    binding: payload_protocol::StreamBinding,
    kind: contract::MessageKind,
    sequence: u32,
    status: u16,
    payload: &[u8],
    identity: Option<contract::MeasurementIdentity>,
) -> Vec<u8> {
    let (item_id, subitem_id, role_id) = identity
        .map(|value| (value.item_id, value.subitem_id, value.role_id))
        .unwrap_or((0, 0, contract::RoleId::Supervisor));
    payload_protocol::Frame::new(
        kind, sequence, item_id, subitem_id, role_id, status, binding, payload,
    )
    .unwrap()
    .encode()
    .as_bytes()
    .to_vec()
}

fn accept_payload_wire_by_host(
    machine: &mut C2b2aHostProtocolMachine,
    wire: &[u8],
) -> C2b2aHostProtocolEvent {
    let header: &[u8; C2B2A_HOST_HEADER_BYTES] =
        wire[..C2B2A_HOST_HEADER_BYTES].try_into().unwrap();
    machine
        .accept_supervisor_frame(header, &wire[C2B2A_HOST_HEADER_BYTES..])
        .unwrap()
}

fn append_len_prefixed(corpus: &mut Vec<u8>, frame: &[u8]) {
    corpus.extend_from_slice(&u32::try_from(frame.len()).unwrap().to_be_bytes());
    corpus.extend_from_slice(frame);
}

fn append_cross_frame(
    host_corpus: &mut Vec<u8>,
    payload_corpus: &mut Vec<u8>,
    frame_count: &mut usize,
    host: &[u8],
    payload: &[u8],
) {
    assert_eq!(host, payload);
    append_len_prefixed(host_corpus, host);
    append_len_prefixed(payload_corpus, payload);
    *frame_count += 1;
}

fn append_identity(
    corpus: &mut Vec<u8>,
    selector: &[u8; C2B2A_SELECTOR_BYTES],
    index: u16,
    item_id: u16,
    subitem_id: u16,
    scope_code: u16,
    role_id: u16,
) {
    corpus.extend_from_slice(selector);
    for field in [index, item_id, subitem_id, scope_code, role_id] {
        corpus.extend_from_slice(&field.to_be_bytes());
    }
}

fn mutate_wire(mut wire: Vec<u8>, edit: impl FnOnce(&mut Vec<u8>)) -> Vec<u8> {
    edit(&mut wire);
    wire
}

fn reseal_wire_payload(wire: &mut [u8]) {
    let payload_digest = sha256_bytes(&wire[C2B2A_HOST_HEADER_BYTES..]);
    wire[128..160].copy_from_slice(&payload_digest);
}

fn host_decode_wire(direction: C2b2aDirection, wire: &[u8]) -> C2b2aResult<C2b2aDecodedFrame> {
    if wire.len() < C2B2A_HOST_HEADER_BYTES {
        return Err(C2b2aError::Invalid(
            "wire is shorter than the header".into(),
        ));
    }
    let header: &[u8; C2B2A_HOST_HEADER_BYTES] =
        wire[..C2B2A_HOST_HEADER_BYTES].try_into().unwrap();
    decode_host_control_frame(direction, header, &wire[C2B2A_HOST_HEADER_BYTES..])
}

fn payload_decode_host_wire(wire: &[u8]) -> Result<(), payload_protocol::ProtocolError> {
    let frame = payload_protocol::decode_frame(wire)?;
    if matches!(
        frame.header.kind,
        contract::MessageKind::HostStart | contract::MessageKind::HostReady
    ) {
        let selector = payload_protocol::decode_host_selector(frame.payload())?;
        contract::select_schedule_entry(
            selector.phase_code,
            selector.instance_class_code,
            selector.instance_ordinal,
        )
        .ok_or(payload_protocol::ProtocolError::WrongItemBinding)?;
    }
    Ok(())
}

fn crossed_protocols_after_measurements(
    instance: C2b2aInstanceName,
    measurement_count: usize,
) -> (
    C2b2aHostProtocolMachine,
    payload_protocol::HostProtocolV1,
    C2b2aCampaignBindings,
    payload_protocol::StreamBinding,
) {
    let instance_index = CROSS_INSTANCES
        .into_iter()
        .position(|candidate| candidate == instance)
        .unwrap();
    let campaign = parity_campaign(instance_index);
    let binding = payload_binding(instance, campaign);
    let entry = payload_entry(instance);
    let selector = payload_selector(entry);
    let selector_bytes = payload_protocol::encode_host_selector(selector);
    let mut payload_machine =
        payload_protocol::HostProtocolV1::new_bound_for_test(entry, selector, binding).unwrap();
    let (mut host_machine, host_start) =
        C2b2aHostProtocolMachine::begin_for_test(instance, campaign).unwrap();
    payload_machine
        .accept_frame(
            payload_protocol::HostDirection::HostToSupervisor,
            &host_wire(&host_start),
        )
        .unwrap();

    let host_ready = host_wire(&encode_host_ready(instance, campaign).unwrap());
    let nested_ready = payload_frame(
        binding,
        contract::MessageKind::HostReady,
        1,
        0,
        &selector_bytes,
        None,
    );
    payload_machine
        .accept_frame(
            payload_protocol::HostDirection::SupervisorToHost,
            &host_ready,
        )
        .unwrap();
    assert_eq!(
        accept_payload_wire_by_host(&mut host_machine, &nested_ready),
        C2b2aHostProtocolEvent::HostReady
    );

    for index in 0..measurement_count {
        let host_key = expected_measurement_at(instance, index).unwrap();
        let nested_identity =
            contract::expected_measurement(entry, u16::try_from(index).unwrap()).unwrap();
        let sequence = u32::try_from(index).unwrap() + 2;
        let host_measurement = host_wire(
            &encode_measurement(
                instance,
                campaign,
                sequence,
                host_key,
                valid_measurement(host_key),
            )
            .unwrap(),
        );
        let nested_measurement = payload_frame(
            binding,
            contract::MessageKind::Measurement,
            sequence,
            0,
            &payload_protocol::encode_measurement(payload_measurement(nested_identity)),
            Some(nested_identity),
        );
        payload_machine
            .accept_frame(
                payload_protocol::HostDirection::SupervisorToHost,
                &host_measurement,
            )
            .unwrap();
        accept_payload_wire_by_host(&mut host_machine, &nested_measurement);
    }
    (host_machine, payload_machine, campaign, binding)
}

fn assert_measurement_word_mutation_rejected(
    instance: C2b2aInstanceName,
    measurement_index: usize,
    word_index: usize,
    replacement: u64,
) {
    let (mut host_machine, mut payload_machine, campaign, binding) =
        crossed_protocols_after_measurements(instance, measurement_index);
    let entry = payload_entry(instance);
    let host_key = expected_measurement_at(instance, measurement_index).unwrap();
    let nested_identity =
        contract::expected_measurement(entry, u16::try_from(measurement_index).unwrap()).unwrap();
    let sequence = u32::try_from(measurement_index).unwrap() + 2;
    let valid = payload_frame(
        binding,
        contract::MessageKind::Measurement,
        sequence,
        0,
        &payload_protocol::encode_measurement(payload_measurement(nested_identity)),
        Some(nested_identity),
    );
    let mutated = mutate_wire(valid.clone(), |wire| {
        let offset = C2B2A_HOST_HEADER_BYTES + word_index * 8;
        wire[offset..offset + 8].copy_from_slice(&replacement.to_be_bytes());
        reseal_wire_payload(wire);
    });
    let header: &[u8; C2B2A_HOST_HEADER_BYTES] =
        mutated[..C2B2A_HOST_HEADER_BYTES].try_into().unwrap();
    assert!(host_machine
        .accept_supervisor_frame(header, &mutated[C2B2A_HOST_HEADER_BYTES..])
        .is_err());
    assert!(payload_machine
        .accept_frame(payload_protocol::HostDirection::SupervisorToHost, &mutated,)
        .is_err());

    let valid_host = host_wire(
        &encode_measurement(
            instance,
            campaign,
            sequence,
            host_key,
            valid_measurement(host_key),
        )
        .unwrap(),
    );
    let retry_header: &[u8; C2B2A_HOST_HEADER_BYTES] =
        valid_host[..C2B2A_HOST_HEADER_BYTES].try_into().unwrap();
    assert!(host_machine
        .accept_supervisor_frame(retry_header, &valid_host[C2B2A_HOST_HEADER_BYTES..],)
        .is_err());
    assert_eq!(
        payload_machine.accept_frame(payload_protocol::HostDirection::SupervisorToHost, &valid,),
        Err(payload_protocol::ProtocolError::Poisoned)
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CrossFailurePath {
    Challenged,
    HostAbortBeforeStart,
    AbortBeforeReady,
    AbortAfterReady,
}

#[derive(Debug, Clone, Copy)]
struct CrossFailureCase {
    label: &'static str,
    instance: C2b2aInstanceName,
    category: u32,
    measurements: usize,
    active_item: Option<(u16, u16)>,
    cleanup: bool,
    dispatched: bool,
    acknowledged: bool,
    reaped: bool,
    completed_cases: u32,
    completed_subattempts: u32,
    path: CrossFailurePath,
}

fn cross_failure_cases() -> Vec<CrossFailureCase> {
    use C2b2aInstanceName::{B01, B12O, B12P, B15, B16, B40, B41, C01, P01, P02, P03};
    use CrossFailurePath::{AbortAfterReady, AbortBeforeReady, Challenged, HostAbortBeforeStart};

    vec![
        CrossFailureCase {
            label: "c01_lower_first_no_dispatch",
            instance: C01,
            category: 1,
            measurements: 0,
            active_item: Some((1, 1)),
            cleanup: false,
            dispatched: false,
            acknowledged: false,
            reaped: false,
            completed_cases: 0,
            completed_subattempts: 0,
            path: Challenged,
        },
        CrossFailureCase {
            label: "c01_upper_first_dispatched",
            instance: C01,
            category: 8,
            measurements: 7,
            active_item: Some((1, 1)),
            cleanup: false,
            dispatched: true,
            acknowledged: false,
            reaped: true,
            completed_cases: 0,
            completed_subattempts: 0,
            path: Challenged,
        },
        CrossFailureCase {
            label: "c01_lower_second_acknowledged",
            instance: C01,
            category: 9,
            measurements: 7,
            active_item: Some((1, 2)),
            cleanup: true,
            dispatched: true,
            acknowledged: true,
            reaped: true,
            completed_cases: 0,
            completed_subattempts: 1,
            path: Challenged,
        },
        CrossFailureCase {
            label: "c01_upper_last_acknowledged",
            instance: C01,
            category: 10,
            measurements: 21,
            active_item: Some((1, 3)),
            cleanup: true,
            dispatched: true,
            acknowledged: true,
            reaped: true,
            completed_cases: 0,
            completed_subattempts: 2,
            path: Challenged,
        },
        CrossFailureCase {
            label: "positive_lower_first_no_dispatch",
            instance: P01,
            category: 2,
            measurements: 0,
            active_item: Some((1, 1)),
            cleanup: false,
            dispatched: false,
            acknowledged: false,
            reaped: false,
            completed_cases: 0,
            completed_subattempts: 0,
            path: Challenged,
        },
        CrossFailureCase {
            label: "positive_upper_first_dispatched",
            instance: P01,
            category: 8,
            measurements: 7,
            active_item: Some((1, 1)),
            cleanup: false,
            dispatched: true,
            acknowledged: false,
            reaped: true,
            completed_cases: 0,
            completed_subattempts: 0,
            path: Challenged,
        },
        CrossFailureCase {
            label: "positive_lower_second_acknowledged",
            instance: P02,
            category: 11,
            measurements: 7,
            active_item: Some((2, 2)),
            cleanup: true,
            dispatched: true,
            acknowledged: true,
            reaped: true,
            completed_cases: 1,
            completed_subattempts: 0,
            path: Challenged,
        },
        CrossFailureCase {
            label: "positive_upper_last_acknowledged",
            instance: P03,
            category: 12,
            measurements: 140,
            active_item: Some((20, 20)),
            cleanup: true,
            dispatched: true,
            acknowledged: true,
            reaped: true,
            completed_cases: 19,
            completed_subattempts: 0,
            path: Challenged,
        },
        CrossFailureCase {
            label: "positive_second_acknowledged_engine_read",
            instance: P01,
            category: 13,
            measurements: 7,
            active_item: Some((2, 2)),
            cleanup: true,
            dispatched: true,
            acknowledged: true,
            reaped: true,
            completed_cases: 1,
            completed_subattempts: 0,
            path: Challenged,
        },
        CrossFailureCase {
            label: "positive_second_acknowledged_canonical",
            instance: P03,
            category: 14,
            measurements: 7,
            active_item: Some((2, 2)),
            cleanup: true,
            dispatched: true,
            acknowledged: true,
            reaped: true,
            completed_cases: 1,
            completed_subattempts: 0,
            path: Challenged,
        },
        CrossFailureCase {
            label: "shared_lower_first",
            instance: B01,
            category: 3,
            measurements: 0,
            active_item: Some((1, 1)),
            cleanup: false,
            dispatched: false,
            acknowledged: false,
            reaped: false,
            completed_cases: 0,
            completed_subattempts: 0,
            path: Challenged,
        },
        CrossFailureCase {
            label: "shared_upper_first",
            instance: B01,
            category: 4,
            measurements: 1,
            active_item: Some((1, 1)),
            cleanup: false,
            dispatched: false,
            acknowledged: false,
            reaped: true,
            completed_cases: 0,
            completed_subattempts: 0,
            path: Challenged,
        },
        CrossFailureCase {
            label: "shared_lower_second",
            instance: B01,
            category: 5,
            measurements: 1,
            active_item: Some((2, 1)),
            cleanup: true,
            dispatched: false,
            acknowledged: false,
            reaped: true,
            completed_cases: 0,
            completed_subattempts: 1,
            path: Challenged,
        },
        CrossFailureCase {
            label: "shared_upper_last",
            instance: B01,
            category: 6,
            measurements: 60,
            active_item: Some((39, 1)),
            cleanup: true,
            dispatched: false,
            acknowledged: false,
            reaped: true,
            completed_cases: 0,
            completed_subattempts: 59,
            path: Challenged,
        },
        CrossFailureCase {
            label: "dedicated_lower",
            instance: B12O,
            category: 7,
            measurements: 0,
            active_item: Some((12, 1)),
            cleanup: false,
            dispatched: false,
            acknowledged: false,
            reaped: false,
            completed_cases: 0,
            completed_subattempts: 0,
            path: Challenged,
        },
        CrossFailureCase {
            label: "dedicated_upper",
            instance: B12P,
            category: 15,
            measurements: 1,
            active_item: Some((12, 2)),
            cleanup: true,
            dispatched: false,
            acknowledged: false,
            reaped: true,
            completed_cases: 0,
            completed_subattempts: 0,
            path: Challenged,
        },
        CrossFailureCase {
            label: "c01_abort_before_ready",
            instance: C01,
            category: 15,
            measurements: 0,
            active_item: None,
            cleanup: false,
            dispatched: false,
            acknowledged: false,
            reaped: false,
            completed_cases: 0,
            completed_subattempts: 0,
            path: AbortBeforeReady,
        },
        CrossFailureCase {
            label: "b40_abort_before_ready",
            instance: B40,
            category: 16,
            measurements: 0,
            active_item: None,
            cleanup: true,
            dispatched: false,
            acknowledged: false,
            reaped: true,
            completed_cases: 0,
            completed_subattempts: 0,
            path: AbortBeforeReady,
        },
        CrossFailureCase {
            label: "b41_abort_before_ready",
            instance: B41,
            category: 17,
            measurements: 0,
            active_item: None,
            cleanup: true,
            dispatched: false,
            acknowledged: false,
            reaped: true,
            completed_cases: 0,
            completed_subattempts: 0,
            path: AbortBeforeReady,
        },
        CrossFailureCase {
            label: "positive_abort_before_ready",
            instance: P01,
            category: 1,
            measurements: 0,
            active_item: None,
            cleanup: true,
            dispatched: false,
            acknowledged: false,
            reaped: true,
            completed_cases: 0,
            completed_subattempts: 0,
            path: AbortBeforeReady,
        },
        CrossFailureCase {
            label: "shared_abort_before_ready",
            instance: B01,
            category: 2,
            measurements: 0,
            active_item: None,
            cleanup: true,
            dispatched: false,
            acknowledged: false,
            reaped: true,
            completed_cases: 0,
            completed_subattempts: 0,
            path: AbortBeforeReady,
        },
        CrossFailureCase {
            label: "dedicated_abort_before_ready",
            instance: B15,
            category: 3,
            measurements: 0,
            active_item: None,
            cleanup: true,
            dispatched: false,
            acknowledged: false,
            reaped: true,
            completed_cases: 0,
            completed_subattempts: 0,
            path: AbortBeforeReady,
        },
        CrossFailureCase {
            label: "c01_abort_after_ready_zero_progress",
            instance: C01,
            category: 4,
            measurements: 0,
            active_item: None,
            cleanup: true,
            dispatched: false,
            acknowledged: false,
            reaped: true,
            completed_cases: 0,
            completed_subattempts: 0,
            path: AbortAfterReady,
        },
        CrossFailureCase {
            label: "positive_abort_after_ready_progress",
            instance: P02,
            category: 5,
            measurements: 7,
            active_item: Some((2, 2)),
            cleanup: true,
            dispatched: true,
            acknowledged: true,
            reaped: true,
            completed_cases: 1,
            completed_subattempts: 0,
            path: AbortAfterReady,
        },
        CrossFailureCase {
            label: "shared_abort_after_ready_progress",
            instance: B01,
            category: 6,
            measurements: 1,
            active_item: Some((2, 1)),
            cleanup: true,
            dispatched: false,
            acknowledged: false,
            reaped: true,
            completed_cases: 0,
            completed_subattempts: 1,
            path: AbortAfterReady,
        },
        CrossFailureCase {
            label: "dedicated_abort_after_ready_progress",
            instance: B16,
            category: 7,
            measurements: 1,
            active_item: Some((16, 1)),
            cleanup: true,
            dispatched: false,
            acknowledged: false,
            reaped: true,
            completed_cases: 0,
            completed_subattempts: 0,
            path: AbortAfterReady,
        },
        CrossFailureCase {
            label: "c01_host_abort_before_start",
            instance: C01,
            category: 1,
            measurements: 0,
            active_item: None,
            cleanup: true,
            dispatched: false,
            acknowledged: false,
            reaped: true,
            completed_cases: 0,
            completed_subattempts: 0,
            path: HostAbortBeforeStart,
        },
        CrossFailureCase {
            label: "positive_host_abort_before_start",
            instance: P01,
            category: 2,
            measurements: 0,
            active_item: None,
            cleanup: true,
            dispatched: false,
            acknowledged: false,
            reaped: true,
            completed_cases: 0,
            completed_subattempts: 0,
            path: HostAbortBeforeStart,
        },
        CrossFailureCase {
            label: "shared_host_abort_before_start",
            instance: B01,
            category: 3,
            measurements: 0,
            active_item: None,
            cleanup: true,
            dispatched: false,
            acknowledged: false,
            reaped: true,
            completed_cases: 0,
            completed_subattempts: 0,
            path: HostAbortBeforeStart,
        },
        CrossFailureCase {
            label: "dedicated_host_abort_before_start",
            instance: B12O,
            category: 4,
            measurements: 0,
            active_item: None,
            cleanup: true,
            dispatched: false,
            acknowledged: false,
            reaped: true,
            completed_cases: 0,
            completed_subattempts: 0,
            path: HostAbortBeforeStart,
        },
        CrossFailureCase {
            label: "b40_host_abort_before_start",
            instance: B40,
            category: 15,
            measurements: 0,
            active_item: None,
            cleanup: true,
            dispatched: false,
            acknowledged: false,
            reaped: true,
            completed_cases: 0,
            completed_subattempts: 0,
            path: HostAbortBeforeStart,
        },
        CrossFailureCase {
            label: "b41_host_abort_before_start",
            instance: B41,
            category: 16,
            measurements: 0,
            active_item: None,
            cleanup: true,
            dispatched: false,
            acknowledged: false,
            reaped: true,
            completed_cases: 0,
            completed_subattempts: 0,
            path: HostAbortBeforeStart,
        },
    ]
}

fn host_failure_state(case: CrossFailureCase) -> C2b2aTerminalState {
    C2b2aTerminalState {
        primary_category: C2b2aPrimaryCategory::try_from(case.category).unwrap(),
        active_item_code: case
            .active_item
            .map(|(item, subitem)| C2b2aTerminalState::active_item(item, subitem).unwrap())
            .unwrap_or(0),
        guest_cleanup_proven: case.cleanup,
        write_may_have_been_dispatched: case.dispatched,
        writer_acknowledged: case.acknowledged,
        roles_reaped: case.reaped,
        completed_case_count: case.completed_cases,
        completed_subattempt_count: case.completed_subattempts,
    }
}

fn payload_failure_state(case: CrossFailureCase) -> contract::TerminalStateV1 {
    contract::TerminalStateV1 {
        primary_category: case.category,
        active_item_code: case
            .active_item
            .map(|(item, subitem)| contract::active_item_code(item, subitem))
            .unwrap_or(0),
        guest_cleanup_proven: u32::from(case.cleanup),
        write_may_have_been_dispatched: u32::from(case.dispatched),
        writer_acknowledged: u32::from(case.acknowledged),
        roles_reaped: u32::from(case.reaped),
        completed_case_count: case.completed_cases,
        completed_subattempt_count: case.completed_subattempts,
    }
}

fn failure_campaign(index: usize) -> C2b2aCampaignBindings {
    let mut campaign = parity_campaign(0);
    campaign.nonce = nonce(0x40 + u8::try_from(index).unwrap());
    campaign
}

fn failure_challenge(index: usize) -> C2b2aChallenge {
    challenge(0xa0 + u8::try_from(index).unwrap())
}

fn assert_invalid_failure_progress_poison(
    invalid_case: CrossFailureCase,
    valid_recovery: CrossFailureCase,
) {
    assert_eq!(invalid_case.instance, valid_recovery.instance);
    assert_eq!(invalid_case.measurements, valid_recovery.measurements);
    let instance = invalid_case.instance;
    let (mut host_machine, mut payload_machine, campaign, binding) =
        crossed_protocols_after_measurements(instance, invalid_case.measurements);
    let sequence = u32::try_from(invalid_case.measurements).unwrap() + 2;
    let invalid_host = host_wire(
        &encode_terminal_ready(
            instance,
            campaign,
            sequence,
            host_failure_state(invalid_case),
        )
        .unwrap(),
    );
    // The nested builder rejects phase/shape-invalid state, so feed the host builder's sealed bytes
    // to both decoders and retain independent equality for the valid recovery below.
    let invalid_nested = invalid_host.clone();
    let invalid_header: &[u8; C2B2A_HOST_HEADER_BYTES] = invalid_nested[..C2B2A_HOST_HEADER_BYTES]
        .try_into()
        .unwrap();
    assert!(host_machine
        .accept_supervisor_frame(invalid_header, &invalid_nested[C2B2A_HOST_HEADER_BYTES..],)
        .is_err());
    assert!(payload_machine
        .accept_frame(
            payload_protocol::HostDirection::SupervisorToHost,
            &invalid_host,
        )
        .is_err());

    let valid_host = host_wire(
        &encode_terminal_ready(
            instance,
            campaign,
            sequence,
            host_failure_state(valid_recovery),
        )
        .unwrap(),
    );
    let valid_nested = payload_frame(
        binding,
        contract::MessageKind::TerminalReady,
        sequence,
        u16::try_from(valid_recovery.category).unwrap(),
        &payload_protocol::encode_terminal_state(payload_failure_state(valid_recovery)),
        None,
    );
    assert_eq!(valid_host, valid_nested);
    let valid_header: &[u8; C2B2A_HOST_HEADER_BYTES] =
        valid_nested[..C2B2A_HOST_HEADER_BYTES].try_into().unwrap();
    assert!(host_machine
        .accept_supervisor_frame(valid_header, &valid_nested[C2B2A_HOST_HEADER_BYTES..],)
        .is_err());
    assert_eq!(
        payload_machine.accept_frame(
            payload_protocol::HostDirection::SupervisorToHost,
            &valid_host,
        ),
        Err(payload_protocol::ProtocolError::Poisoned)
    );
}

fn bindings(nonce_byte: u8) -> C2b2aCampaignBindings {
    C2b2aCampaignBindings {
        nonce: nonce(nonce_byte),
        contract_sha256: C2B2A_CONTRACT_SHA256,
        characterization_schedule_sha256: digest(0x33),
        acceptance_schedule_sha256: digest(0x44),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let result = Sha256::digest(bytes);
    result.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn sha256_bytes(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn put_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}

fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
}

fn reseal_frame_payload(header: &mut [u8; C2B2A_HOST_HEADER_BYTES], payload: &[u8]) {
    header[128..160].copy_from_slice(&sha256_bytes(payload));
}

fn reseal_receipt(receipt: &mut [u8; C2B2A_HOST_TERMINAL_RECEIPT_BYTES]) {
    let digest = sha256_bytes(&receipt[..224]);
    receipt[224..256].copy_from_slice(&digest);
}

fn valid_measurement(key: C2b2aMeasurementKey) -> C2b2aMeasurement {
    let mut measurement = C2b2aMeasurement {
        scope_code: key.scope_code,
        monotonic_elapsed_ns: 1,
        ..C2b2aMeasurement::default()
    };
    if key.role_id == C2b2aRoleId::Boundary && key.case_or_probe_id == 17 {
        measurement.descriptor_type_bitmap = 1 << 7;
    }
    if key.role_id == C2b2aRoleId::Boundary && key.case_or_probe_id == 18 {
        measurement.descriptor_type_bitmap = 1 << 14;
    }
    measurement
}

fn accept_frame(
    machine: &mut C2b2aHostProtocolMachine,
    frame: &C2b2aEncodedFrame,
) -> C2b2aHostProtocolEvent {
    machine
        .accept_supervisor_frame(frame.header(), frame.payload())
        .expect("fixture frame must be accepted")
}

fn enter_collecting(
    instance: C2b2aInstanceName,
    campaign: C2b2aCampaignBindings,
) -> C2b2aHostProtocolMachine {
    let (mut machine, _) = C2b2aHostProtocolMachine::begin_for_test(instance, campaign).unwrap();
    let ready = encode_host_ready(instance, campaign).unwrap();
    assert_eq!(
        accept_frame(&mut machine, &ready),
        C2b2aHostProtocolEvent::HostReady
    );
    machine
}

fn accept_all_measurements(
    machine: &mut C2b2aHostProtocolMachine,
    instance: C2b2aInstanceName,
    campaign: C2b2aCampaignBindings,
) {
    let spec = instance_spec(instance);
    for index in 0..usize::try_from(spec.measurement_count).unwrap() {
        let key = expected_measurement_at(instance, index).unwrap();
        let sequence = u32::try_from(index).unwrap() + 2;
        let frame =
            encode_measurement(instance, campaign, sequence, key, valid_measurement(key)).unwrap();
        assert_eq!(
            accept_frame(machine, &frame),
            C2b2aHostProtocolEvent::MeasurementAccepted {
                one_based_index: u32::try_from(index).unwrap() + 1,
                key,
            }
        );
    }
}

#[test]
fn cross_golden_selectors_and_all_measurement_identities_are_exact() {
    let mut host_selectors = Vec::new();
    let mut payload_selectors = Vec::new();
    let mut host_identities = Vec::new();
    let mut payload_identities = Vec::new();
    let mut host_count = 0_usize;
    let mut payload_count = 0_usize;

    for (instance_index, instance) in CROSS_INSTANCES.into_iter().enumerate() {
        let host_spec = instance_spec(instance);
        let payload_spec = payload_entry(instance);
        assert_eq!(payload_spec.name, instance.label());
        assert_eq!(
            u32::from(contract::measurement_count(payload_spec)),
            host_spec.measurement_count
        );

        let host_start = encode_host_start(instance, parity_campaign(instance_index)).unwrap();
        let host_selector: &[u8; C2B2A_SELECTOR_BYTES] = host_start.payload().try_into().unwrap();
        let nested_selector =
            payload_protocol::encode_host_selector(payload_selector(payload_spec));
        assert_eq!(host_selector, &nested_selector);
        host_selectors.extend_from_slice(host_selector);
        payload_selectors.extend_from_slice(&nested_selector);

        for index in 0..usize::try_from(host_spec.measurement_count).unwrap() {
            let host = expected_measurement_at(instance, index).unwrap();
            let nested =
                contract::expected_measurement(payload_spec, u16::try_from(index).unwrap())
                    .unwrap();
            assert_eq!(payload_identity(host), nested);
            append_identity(
                &mut host_identities,
                host_selector,
                u16::try_from(index).unwrap(),
                host.case_or_probe_id,
                host.fixture_or_subattempt_id,
                u16::try_from(host.scope_code).unwrap(),
                host.role_id as u16,
            );
            append_identity(
                &mut payload_identities,
                &nested_selector,
                u16::try_from(index).unwrap(),
                nested.item_id,
                nested.subitem_id,
                nested.scope_code,
                nested.role_id as u16,
            );
            host_count += 1;
            payload_count += 1;
        }
        assert!(expected_measurement_at(
            instance,
            usize::try_from(host_spec.measurement_count).unwrap()
        )
        .is_none());
        assert!(contract::expected_measurement(
            payload_spec,
            contract::measurement_count(payload_spec)
        )
        .is_none());
    }

    assert_eq!(host_count, 505);
    assert_eq!(payload_count, 505);
    assert_eq!(host_selectors.len(), 88);
    assert_eq!(payload_selectors.len(), 88);
    assert_eq!(host_selectors, payload_selectors);
    assert_eq!(sha256_hex(&host_selectors), SELECTOR_CORPUS_SHA256);
    assert_eq!(sha256_hex(&payload_selectors), SELECTOR_CORPUS_SHA256);
    assert_eq!(host_identities.len(), 9_090);
    assert_eq!(payload_identities.len(), 9_090);
    assert_eq!(host_identities, payload_identities);
    assert_eq!(sha256_hex(&host_identities), IDENTITY_CORPUS_SHA256);
    assert_eq!(sha256_hex(&payload_identities), IDENTITY_CORPUS_SHA256);
}

#[test]
fn cross_golden_terminal_state_codecs_cover_success_and_every_failure_category() {
    let mut host_success = Vec::new();
    let mut payload_success = Vec::new();
    let mut challenged_count = 0_usize;

    for (instance_index, instance) in CROSS_INSTANCES.into_iter().enumerate() {
        let host_spec = instance_spec(instance);
        if host_spec.terminal_mode != C2b2aTerminalMode::Challenged {
            continue;
        }
        let host_start = encode_host_start(instance, parity_campaign(instance_index)).unwrap();
        let host_selector: &[u8; C2B2A_SELECTOR_BYTES] = host_start.payload().try_into().unwrap();
        let payload_spec = payload_entry(instance);
        let nested_selector =
            payload_protocol::encode_host_selector(payload_selector(payload_spec));
        let host_state = C2b2aTerminalState::success_for(instance).unwrap();
        let nested_state = contract::successful_terminal(payload_spec).unwrap();
        let host_bytes = host_state.encode();
        let nested_bytes = payload_protocol::encode_terminal_state(nested_state);
        assert_eq!(host_bytes, nested_bytes);
        assert_eq!(C2b2aTerminalState::decode(&host_bytes).unwrap(), host_state);
        assert_eq!(
            payload_protocol::decode_terminal_state(&nested_bytes).unwrap(),
            nested_state
        );
        host_success.extend_from_slice(host_selector);
        host_success.extend_from_slice(&host_bytes);
        payload_success.extend_from_slice(&nested_selector);
        payload_success.extend_from_slice(&nested_bytes);
        challenged_count += 1;
    }

    assert_eq!(challenged_count, 9);
    assert_eq!(host_success.len(), 360);
    assert_eq!(payload_success.len(), 360);
    assert_eq!(host_success, payload_success);
    assert_eq!(sha256_hex(&host_success), SUCCESS_TERMINAL_CORPUS_SHA256);
    assert_eq!(sha256_hex(&payload_success), SUCCESS_TERMINAL_CORPUS_SHA256);

    for category_code in 1_u32..=17 {
        let outcome_uncertain = category_code == 8;
        let after_acknowledgement = (9..=14).contains(&category_code);
        let host_state = C2b2aTerminalState {
            primary_category: C2b2aPrimaryCategory::try_from(category_code).unwrap(),
            active_item_code: 0,
            guest_cleanup_proven: true,
            write_may_have_been_dispatched: outcome_uncertain || after_acknowledgement,
            writer_acknowledged: after_acknowledgement,
            roles_reaped: true,
            completed_case_count: 0,
            completed_subattempt_count: 0,
        };
        let nested_state = contract::TerminalStateV1 {
            primary_category: category_code,
            active_item_code: 0,
            guest_cleanup_proven: 1,
            write_may_have_been_dispatched: u32::from(outcome_uncertain || after_acknowledgement),
            writer_acknowledged: u32::from(after_acknowledgement),
            roles_reaped: 1,
            completed_case_count: 0,
            completed_subattempt_count: 0,
        };
        let host_bytes = host_state.encode();
        let nested_bytes = payload_protocol::encode_terminal_state(nested_state);
        assert_eq!(host_bytes, nested_bytes);
        assert_eq!(C2b2aTerminalState::decode(&host_bytes).unwrap(), host_state);
        assert_eq!(
            payload_protocol::decode_terminal_state(&nested_bytes).unwrap(),
            nested_state
        );
    }
}

#[test]
fn cross_golden_full_transcripts_and_all_host_frame_kinds_are_exact() {
    // Corpus recipe: C2B2A_CAMPAIGN_ORDER; nonce byte instance_index + 1; challenge byte
    // 0x80 + instance_index + 1; accepted contract digest; 0x33/0x44 schedule digests. Each
    // instance contributes HostStart, HostReady, every Measurement, and, when challenged,
    // TerminalReady, Challenge, Terminal. The tail contains every category legal before dispatch
    // as a C01 supervisor Abort at sequence 1, followed by host-originated categories 1..=7 that
    // replace HostStart. All tail states have zero active item/count/dispatch/ack and cleanup/reap
    // set. Every frame is prefixed by its big-endian u32 wire length; same-direction EOF adds no
    // bytes.
    let mut host_corpus = Vec::new();
    let mut payload_corpus = Vec::new();
    let mut frame_count = 0_usize;

    for (instance_index, instance) in CROSS_INSTANCES.into_iter().enumerate() {
        let campaign = parity_campaign(instance_index);
        let binding = payload_binding(instance, campaign);
        let entry = payload_entry(instance);
        let selector = payload_selector(entry);
        let selector_bytes = payload_protocol::encode_host_selector(selector);
        let mut payload_machine =
            payload_protocol::HostProtocolV1::new_bound_for_test(entry, selector, binding).unwrap();
        let (mut host_machine, host_start_frame) =
            C2b2aHostProtocolMachine::begin_for_test(instance, campaign).unwrap();

        let host_start = host_wire(&host_start_frame);
        let nested_start = payload_frame(
            binding,
            contract::MessageKind::HostStart,
            1,
            0,
            &selector_bytes,
            None,
        );
        assert_eq!(host_start, nested_start);
        append_len_prefixed(&mut host_corpus, &host_start);
        append_len_prefixed(&mut payload_corpus, &nested_start);
        frame_count += 1;
        payload_machine
            .accept_frame(
                payload_protocol::HostDirection::HostToSupervisor,
                &host_start,
            )
            .unwrap();

        let host_ready = host_wire(&encode_host_ready(instance, campaign).unwrap());
        let nested_ready = payload_frame(
            binding,
            contract::MessageKind::HostReady,
            1,
            0,
            &selector_bytes,
            None,
        );
        assert_eq!(host_ready, nested_ready);
        append_len_prefixed(&mut host_corpus, &host_ready);
        append_len_prefixed(&mut payload_corpus, &nested_ready);
        frame_count += 1;
        payload_machine
            .accept_frame(
                payload_protocol::HostDirection::SupervisorToHost,
                &host_ready,
            )
            .unwrap();
        assert_eq!(
            accept_payload_wire_by_host(&mut host_machine, &nested_ready),
            C2b2aHostProtocolEvent::HostReady
        );

        let host_spec = instance_spec(instance);
        for index in 0..usize::try_from(host_spec.measurement_count).unwrap() {
            let host_key = expected_measurement_at(instance, index).unwrap();
            let nested_identity =
                contract::expected_measurement(entry, u16::try_from(index).unwrap()).unwrap();
            let sequence = u32::try_from(index).unwrap() + 2;
            let host_measurement = host_wire(
                &encode_measurement(
                    instance,
                    campaign,
                    sequence,
                    host_key,
                    valid_measurement(host_key),
                )
                .unwrap(),
            );
            let nested_measurement = payload_frame(
                binding,
                contract::MessageKind::Measurement,
                sequence,
                0,
                &payload_protocol::encode_measurement(payload_measurement(nested_identity)),
                Some(nested_identity),
            );
            assert_eq!(host_measurement, nested_measurement);
            append_len_prefixed(&mut host_corpus, &host_measurement);
            append_len_prefixed(&mut payload_corpus, &nested_measurement);
            frame_count += 1;
            payload_machine
                .accept_frame(
                    payload_protocol::HostDirection::SupervisorToHost,
                    &host_measurement,
                )
                .unwrap();
            assert_eq!(
                accept_payload_wire_by_host(&mut host_machine, &nested_measurement),
                C2b2aHostProtocolEvent::MeasurementAccepted {
                    one_based_index: u32::try_from(index).unwrap() + 1,
                    key: host_key,
                }
            );
        }

        match host_spec.terminal_mode {
            C2b2aTerminalMode::Challenged => {
                let host_state = C2b2aTerminalState::success_for(instance).unwrap();
                let nested_state = contract::successful_terminal(entry).unwrap();
                let ready_sequence = host_spec.terminal_ready_sequence.unwrap();
                let terminal_sequence = host_spec.terminal_sequence.unwrap();
                let host_terminal_ready = host_wire(
                    &encode_terminal_ready(instance, campaign, ready_sequence, host_state).unwrap(),
                );
                let nested_terminal_ready = payload_frame(
                    binding,
                    contract::MessageKind::TerminalReady,
                    ready_sequence,
                    0,
                    &payload_protocol::encode_terminal_state(nested_state),
                    None,
                );
                assert_eq!(host_terminal_ready, nested_terminal_ready);
                append_len_prefixed(&mut host_corpus, &host_terminal_ready);
                append_len_prefixed(&mut payload_corpus, &nested_terminal_ready);
                frame_count += 1;
                payload_machine
                    .accept_frame(
                        payload_protocol::HostDirection::SupervisorToHost,
                        &host_terminal_ready,
                    )
                    .unwrap();
                assert_eq!(
                    accept_payload_wire_by_host(&mut host_machine, &nested_terminal_ready),
                    C2b2aHostProtocolEvent::ChallengeRequired(host_state)
                );

                let host_challenge_value = parity_challenge(instance_index);
                let nested_challenge_value = contract::Digest32(*host_challenge_value.as_bytes());
                let host_challenge = host_wire(
                    &host_machine
                        .issue_challenge_for_test(campaign, host_challenge_value)
                        .unwrap(),
                );
                let nested_challenge = payload_frame(
                    binding,
                    contract::MessageKind::Challenge,
                    2,
                    0,
                    nested_challenge_value.as_bytes(),
                    None,
                );
                assert_eq!(host_challenge, nested_challenge);
                append_len_prefixed(&mut host_corpus, &host_challenge);
                append_len_prefixed(&mut payload_corpus, &nested_challenge);
                frame_count += 1;
                payload_machine
                    .accept_frame(
                        payload_protocol::HostDirection::HostToSupervisor,
                        &host_challenge,
                    )
                    .unwrap();
                payload_machine
                    .accept_eof(payload_protocol::HostDirection::HostToSupervisor)
                    .unwrap();
                host_machine.accept_host_to_supervisor_eof().unwrap();

                let host_terminal = host_wire(
                    &encode_terminal(
                        instance,
                        campaign,
                        terminal_sequence,
                        host_challenge_value,
                        host_state,
                    )
                    .unwrap(),
                );
                let nested_terminal = payload_frame(
                    binding,
                    contract::MessageKind::Terminal,
                    terminal_sequence,
                    0,
                    &payload_protocol::encode_terminal_payload(
                        nested_challenge_value,
                        nested_state,
                    ),
                    None,
                );
                assert_eq!(host_terminal, nested_terminal);
                append_len_prefixed(&mut host_corpus, &host_terminal);
                append_len_prefixed(&mut payload_corpus, &nested_terminal);
                frame_count += 1;
                payload_machine
                    .accept_frame(
                        payload_protocol::HostDirection::SupervisorToHost,
                        &host_terminal,
                    )
                    .unwrap();
                assert_eq!(
                    accept_payload_wire_by_host(&mut host_machine, &nested_terminal),
                    C2b2aHostProtocolEvent::TerminalAccepted(host_state)
                );
                payload_machine
                    .accept_eof(payload_protocol::HostDirection::SupervisorToHost)
                    .unwrap();
                assert!(payload_machine.is_complete());
                assert!(matches!(
                    host_machine.finish_on_supervisor_eof().unwrap(),
                    C2b2aHostProtocolCompletion::ChallengedTerminal { state, .. }
                        if state == host_state
                ));
            }
            C2b2aTerminalMode::HostTerminal { probe_id } => {
                payload_machine
                    .accept_eof(payload_protocol::HostDirection::SupervisorToHost)
                    .unwrap();
                assert!(payload_machine.is_complete());
                assert_eq!(
                    host_machine.finish_on_supervisor_eof().unwrap(),
                    C2b2aHostProtocolCompletion::HostTerminalEof { probe_id }
                );
            }
        }
    }

    let instance = C2b2aInstanceName::C01;
    let campaign = parity_campaign(0);
    let binding = payload_binding(instance, campaign);
    let entry = payload_entry(instance);
    let selector = payload_selector(entry);
    let selector_bytes = payload_protocol::encode_host_selector(selector);
    let host_start = host_wire(&encode_host_start(instance, campaign).unwrap());
    for category_code in [1_u32, 2, 3, 4, 5, 6, 7, 15, 16, 17] {
        let host_state = C2b2aTerminalState {
            primary_category: C2b2aPrimaryCategory::try_from(category_code).unwrap(),
            active_item_code: 0,
            guest_cleanup_proven: true,
            write_may_have_been_dispatched: false,
            writer_acknowledged: false,
            roles_reaped: true,
            completed_case_count: 0,
            completed_subattempt_count: 0,
        };
        let nested_state = contract::TerminalStateV1 {
            primary_category: category_code,
            active_item_code: 0,
            guest_cleanup_proven: 1,
            write_may_have_been_dispatched: 0,
            writer_acknowledged: 0,
            roles_reaped: 1,
            completed_case_count: 0,
            completed_subattempt_count: 0,
        };
        let host_abort =
            host_wire(&encode_supervisor_abort(instance, campaign, 1, host_state).unwrap());
        let nested_abort = payload_frame(
            binding,
            contract::MessageKind::Abort,
            1,
            u16::try_from(category_code).unwrap(),
            &payload_protocol::encode_terminal_state(nested_state),
            None,
        );
        assert_eq!(host_abort, nested_abort);
        append_len_prefixed(&mut host_corpus, &host_abort);
        append_len_prefixed(&mut payload_corpus, &nested_abort);
        frame_count += 1;

        let mut payload_machine =
            payload_protocol::HostProtocolV1::new_bound_for_test(entry, selector, binding).unwrap();
        payload_machine
            .accept_frame(
                payload_protocol::HostDirection::HostToSupervisor,
                &host_start,
            )
            .unwrap();
        payload_machine
            .accept_frame(
                payload_protocol::HostDirection::SupervisorToHost,
                &host_abort,
            )
            .unwrap();
        payload_machine
            .accept_eof(payload_protocol::HostDirection::SupervisorToHost)
            .unwrap();
        assert!(payload_machine.is_complete());

        let (mut host_machine, _) =
            C2b2aHostProtocolMachine::begin_for_test(instance, campaign).unwrap();
        assert_eq!(
            accept_payload_wire_by_host(&mut host_machine, &nested_abort),
            C2b2aHostProtocolEvent::AbortAccepted(host_state)
        );
        assert_eq!(
            host_machine.finish_on_supervisor_eof().unwrap(),
            C2b2aHostProtocolCompletion::UnchallengedFailure {
                state: host_state,
                measurements_seen: 0,
            }
        );
    }
    for category_code in 1_u32..=7 {
        let host_state = C2b2aTerminalState {
            primary_category: C2b2aPrimaryCategory::try_from(category_code).unwrap(),
            active_item_code: 0,
            guest_cleanup_proven: true,
            write_may_have_been_dispatched: false,
            writer_acknowledged: false,
            roles_reaped: true,
            completed_case_count: 0,
            completed_subattempt_count: 0,
        };
        let nested_state = contract::TerminalStateV1 {
            primary_category: category_code,
            active_item_code: 0,
            guest_cleanup_proven: 1,
            write_may_have_been_dispatched: 0,
            writer_acknowledged: 0,
            roles_reaped: 1,
            completed_case_count: 0,
            completed_subattempt_count: 0,
        };
        let (mut host_machine, _) =
            C2b2aHostProtocolMachine::begin_for_test(instance, campaign).unwrap();
        let host_abort = host_wire(
            &host_machine
                .replace_initial_host_start_with_abort(campaign, host_state)
                .unwrap(),
        );
        let nested_abort = payload_frame(
            binding,
            contract::MessageKind::Abort,
            1,
            u16::try_from(category_code).unwrap(),
            &payload_protocol::encode_terminal_state(nested_state),
            None,
        );
        assert_eq!(host_abort, nested_abort);
        append_len_prefixed(&mut host_corpus, &host_abort);
        append_len_prefixed(&mut payload_corpus, &nested_abort);
        frame_count += 1;

        let mut payload_machine =
            payload_protocol::HostProtocolV1::new_bound_for_test(entry, selector, binding).unwrap();
        payload_machine
            .accept_frame(
                payload_protocol::HostDirection::HostToSupervisor,
                &host_abort,
            )
            .unwrap();
        payload_machine
            .accept_eof(payload_protocol::HostDirection::HostToSupervisor)
            .unwrap();
        host_machine.accept_host_to_supervisor_eof().unwrap();
        assert!(payload_machine.is_complete());
        assert_eq!(
            host_machine.finish_after_host_to_supervisor_eof().unwrap(),
            C2b2aHostProtocolCompletion::UnchallengedFailure {
                state: host_state,
                measurements_seen: 0,
            }
        );
    }
    assert_eq!(selector_bytes, host_start[C2B2A_HOST_HEADER_BYTES..]);

    assert_eq!(frame_count, 571);
    assert_eq!(host_corpus.len(), 224_796);
    assert_eq!(payload_corpus.len(), 224_796);
    assert_eq!(host_corpus, payload_corpus);
    assert_eq!(sha256_hex(&host_corpus), FRAME_CORPUS_SHA256);
    assert_eq!(sha256_hex(&payload_corpus), FRAME_CORPUS_SHA256);
}

#[test]
fn cross_golden_structured_failure_and_abort_corpus_is_exact() {
    // Corpus recipe: cross_failure_cases() order; nonce byte 0x40 + zero-based case index;
    // challenge byte 0xa0 + zero-based case index; accepted contract and 0x33/0x44 schedule
    // digests. HostAbortBeforeStart contributes a host-to-supervisor Abort in place of HostStart;
    // every other case contributes HostStart, optional HostReady and measurement prefix, then
    // either TerminalReady/Challenge/Terminal or a supervisor-to-host Abort. Every frame has a
    // big-endian u32 length prefix. Required same-direction EOF transitions are consumed by both
    // machines but add no bytes to the corpus.
    let cases = cross_failure_cases();
    assert_eq!(cases.len(), 32);
    assert_eq!(
        cases.iter().map(|case| case.label).collect::<Vec<_>>(),
        [
            "c01_lower_first_no_dispatch",
            "c01_upper_first_dispatched",
            "c01_lower_second_acknowledged",
            "c01_upper_last_acknowledged",
            "positive_lower_first_no_dispatch",
            "positive_upper_first_dispatched",
            "positive_lower_second_acknowledged",
            "positive_upper_last_acknowledged",
            "positive_second_acknowledged_engine_read",
            "positive_second_acknowledged_canonical",
            "shared_lower_first",
            "shared_upper_first",
            "shared_lower_second",
            "shared_upper_last",
            "dedicated_lower",
            "dedicated_upper",
            "c01_abort_before_ready",
            "b40_abort_before_ready",
            "b41_abort_before_ready",
            "positive_abort_before_ready",
            "shared_abort_before_ready",
            "dedicated_abort_before_ready",
            "c01_abort_after_ready_zero_progress",
            "positive_abort_after_ready_progress",
            "shared_abort_after_ready_progress",
            "dedicated_abort_after_ready_progress",
            "c01_host_abort_before_start",
            "positive_host_abort_before_start",
            "shared_host_abort_before_start",
            "dedicated_host_abort_before_start",
            "b40_host_abort_before_start",
            "b41_host_abort_before_start",
        ]
    );
    let categories: BTreeSet<_> = cases.iter().map(|case| case.category).collect();
    assert_eq!(categories, (1_u32..=17).collect());
    let before_dispatch = [1_u32, 2, 3, 4, 5, 6, 7, 15, 16, 17];
    let after_acknowledgement = [1_u32, 2, 3, 5, 9, 10, 11, 12, 13, 14, 15, 16, 17];
    for case in &cases {
        if case.dispatched && !case.acknowledged {
            assert_eq!(case.category, 8, "{}", case.label);
        } else if case.acknowledged {
            assert!(
                after_acknowledgement.contains(&case.category),
                "{}",
                case.label
            );
        } else {
            assert!(before_dispatch.contains(&case.category), "{}", case.label);
        }
    }
    for family in [
        C2b2aInstanceName::C01,
        C2b2aInstanceName::P01,
        C2b2aInstanceName::B01,
        C2b2aInstanceName::B12O,
    ] {
        assert!(cases
            .iter()
            .any(|case| case.instance == family && case.path == CrossFailurePath::Challenged));
    }
    for family in [
        C2b2aInstanceName::C01,
        C2b2aInstanceName::P01,
        C2b2aInstanceName::B01,
        C2b2aInstanceName::B15,
    ] {
        assert!(cases.iter().any(|case| {
            case.instance == family && case.path == CrossFailurePath::AbortBeforeReady
        }));
    }
    for family in [
        C2b2aInstanceName::C01,
        C2b2aInstanceName::P01,
        C2b2aInstanceName::B01,
        C2b2aInstanceName::B12O,
        C2b2aInstanceName::B40,
        C2b2aInstanceName::B41,
    ] {
        assert!(cases.iter().any(|case| {
            case.instance == family && case.path == CrossFailurePath::HostAbortBeforeStart
        }));
    }

    let mut host_corpus = Vec::new();
    let mut payload_corpus = Vec::new();
    let mut frame_count = 0_usize;
    for (case_index, case) in cases.into_iter().enumerate() {
        let campaign = failure_campaign(case_index);
        let binding = payload_binding(case.instance, campaign);
        let entry = payload_entry(case.instance);
        let selector = payload_selector(entry);
        let selector_bytes = payload_protocol::encode_host_selector(selector);
        let mut payload_machine =
            payload_protocol::HostProtocolV1::new_bound_for_test(entry, selector, binding).unwrap();
        let (mut host_machine, host_start_frame) =
            C2b2aHostProtocolMachine::begin_for_test(case.instance, campaign).unwrap();
        let host_start = host_wire(&host_start_frame);
        let nested_start = payload_frame(
            binding,
            contract::MessageKind::HostStart,
            1,
            0,
            &selector_bytes,
            None,
        );
        if case.path == CrossFailurePath::HostAbortBeforeStart {
            let host_state = host_failure_state(case);
            let nested_state = payload_failure_state(case);
            let host_abort = host_wire(
                &host_machine
                    .replace_initial_host_start_with_abort(campaign, host_state)
                    .unwrap(),
            );
            let nested_abort = payload_frame(
                binding,
                contract::MessageKind::Abort,
                1,
                u16::try_from(case.category).unwrap(),
                &payload_protocol::encode_terminal_state(nested_state),
                None,
            );
            append_cross_frame(
                &mut host_corpus,
                &mut payload_corpus,
                &mut frame_count,
                &host_abort,
                &nested_abort,
            );
            payload_machine
                .accept_frame(
                    payload_protocol::HostDirection::HostToSupervisor,
                    &host_abort,
                )
                .unwrap();
            payload_machine
                .accept_eof(payload_protocol::HostDirection::HostToSupervisor)
                .unwrap();
            assert!(payload_machine.is_complete());
            host_machine.accept_host_to_supervisor_eof().unwrap();
            assert_eq!(
                host_machine.finish_after_host_to_supervisor_eof().unwrap(),
                C2b2aHostProtocolCompletion::UnchallengedFailure {
                    state: host_state,
                    measurements_seen: 0,
                }
            );
            continue;
        }
        append_cross_frame(
            &mut host_corpus,
            &mut payload_corpus,
            &mut frame_count,
            &host_start,
            &nested_start,
        );
        payload_machine
            .accept_frame(
                payload_protocol::HostDirection::HostToSupervisor,
                &host_start,
            )
            .unwrap();

        if case.path == CrossFailurePath::AbortBeforeReady {
            let host_state = host_failure_state(case);
            let nested_state = payload_failure_state(case);
            let host_abort = host_wire(
                &encode_supervisor_abort(case.instance, campaign, 1, host_state).unwrap(),
            );
            let nested_abort = payload_frame(
                binding,
                contract::MessageKind::Abort,
                1,
                u16::try_from(case.category).unwrap(),
                &payload_protocol::encode_terminal_state(nested_state),
                None,
            );
            append_cross_frame(
                &mut host_corpus,
                &mut payload_corpus,
                &mut frame_count,
                &host_abort,
                &nested_abort,
            );
            payload_machine
                .accept_frame(
                    payload_protocol::HostDirection::SupervisorToHost,
                    &host_abort,
                )
                .unwrap();
            assert_eq!(
                accept_payload_wire_by_host(&mut host_machine, &nested_abort),
                C2b2aHostProtocolEvent::AbortAccepted(host_state)
            );
            payload_machine
                .accept_eof(payload_protocol::HostDirection::SupervisorToHost)
                .unwrap();
            assert!(payload_machine.is_complete());
            assert_eq!(
                host_machine.finish_on_supervisor_eof().unwrap(),
                C2b2aHostProtocolCompletion::UnchallengedFailure {
                    state: host_state,
                    measurements_seen: 0,
                }
            );
            continue;
        }

        let host_ready = host_wire(&encode_host_ready(case.instance, campaign).unwrap());
        let nested_ready = payload_frame(
            binding,
            contract::MessageKind::HostReady,
            1,
            0,
            &selector_bytes,
            None,
        );
        append_cross_frame(
            &mut host_corpus,
            &mut payload_corpus,
            &mut frame_count,
            &host_ready,
            &nested_ready,
        );
        payload_machine
            .accept_frame(
                payload_protocol::HostDirection::SupervisorToHost,
                &host_ready,
            )
            .unwrap();
        assert_eq!(
            accept_payload_wire_by_host(&mut host_machine, &nested_ready),
            C2b2aHostProtocolEvent::HostReady
        );

        for measurement_index in 0..case.measurements {
            let host_key = expected_measurement_at(case.instance, measurement_index).unwrap();
            let nested_identity =
                contract::expected_measurement(entry, u16::try_from(measurement_index).unwrap())
                    .unwrap();
            let sequence = u32::try_from(measurement_index).unwrap() + 2;
            let host_measurement = host_wire(
                &encode_measurement(
                    case.instance,
                    campaign,
                    sequence,
                    host_key,
                    valid_measurement(host_key),
                )
                .unwrap(),
            );
            let nested_measurement = payload_frame(
                binding,
                contract::MessageKind::Measurement,
                sequence,
                0,
                &payload_protocol::encode_measurement(payload_measurement(nested_identity)),
                Some(nested_identity),
            );
            append_cross_frame(
                &mut host_corpus,
                &mut payload_corpus,
                &mut frame_count,
                &host_measurement,
                &nested_measurement,
            );
            payload_machine
                .accept_frame(
                    payload_protocol::HostDirection::SupervisorToHost,
                    &host_measurement,
                )
                .unwrap();
            assert_eq!(
                accept_payload_wire_by_host(&mut host_machine, &nested_measurement),
                C2b2aHostProtocolEvent::MeasurementAccepted {
                    one_based_index: u32::try_from(measurement_index).unwrap() + 1,
                    key: host_key,
                }
            );
        }

        let host_state = host_failure_state(case);
        let nested_state = payload_failure_state(case);
        let sequence = u32::try_from(case.measurements).unwrap() + 2;
        if case.path == CrossFailurePath::AbortAfterReady {
            let host_abort = host_wire(
                &encode_supervisor_abort(case.instance, campaign, sequence, host_state).unwrap(),
            );
            let nested_abort = payload_frame(
                binding,
                contract::MessageKind::Abort,
                sequence,
                u16::try_from(case.category).unwrap(),
                &payload_protocol::encode_terminal_state(nested_state),
                case.active_item
                    .map(|(item_id, subitem_id)| contract::MeasurementIdentity {
                        item_id,
                        subitem_id,
                        scope_code: 8,
                        role_id: contract::RoleId::Supervisor,
                    }),
            );
            append_cross_frame(
                &mut host_corpus,
                &mut payload_corpus,
                &mut frame_count,
                &host_abort,
                &nested_abort,
            );
            payload_machine
                .accept_frame(
                    payload_protocol::HostDirection::SupervisorToHost,
                    &host_abort,
                )
                .unwrap();
            assert_eq!(
                accept_payload_wire_by_host(&mut host_machine, &nested_abort),
                C2b2aHostProtocolEvent::AbortAccepted(host_state)
            );
            payload_machine
                .accept_eof(payload_protocol::HostDirection::SupervisorToHost)
                .unwrap();
            assert!(payload_machine.is_complete());
            assert_eq!(
                host_machine.finish_on_supervisor_eof().unwrap(),
                C2b2aHostProtocolCompletion::UnchallengedFailure {
                    state: host_state,
                    measurements_seen: u32::try_from(case.measurements).unwrap(),
                }
            );
            continue;
        }

        let host_terminal_ready = host_wire(
            &encode_terminal_ready(case.instance, campaign, sequence, host_state).unwrap(),
        );
        let nested_terminal_ready = payload_frame(
            binding,
            contract::MessageKind::TerminalReady,
            sequence,
            u16::try_from(case.category).unwrap(),
            &payload_protocol::encode_terminal_state(nested_state),
            None,
        );
        append_cross_frame(
            &mut host_corpus,
            &mut payload_corpus,
            &mut frame_count,
            &host_terminal_ready,
            &nested_terminal_ready,
        );
        payload_machine
            .accept_frame(
                payload_protocol::HostDirection::SupervisorToHost,
                &host_terminal_ready,
            )
            .unwrap_or_else(|error| {
                panic!(
                    "failure case {} rejected nested TerminalReady: {error:?}",
                    case.label
                )
            });
        assert_eq!(
            accept_payload_wire_by_host(&mut host_machine, &nested_terminal_ready),
            C2b2aHostProtocolEvent::ChallengeRequired(host_state)
        );

        let host_challenge_value = failure_challenge(case_index);
        let nested_challenge_value = contract::Digest32(*host_challenge_value.as_bytes());
        let host_challenge = host_wire(
            &host_machine
                .issue_challenge_for_test(campaign, host_challenge_value)
                .unwrap(),
        );
        let nested_challenge = payload_frame(
            binding,
            contract::MessageKind::Challenge,
            2,
            0,
            nested_challenge_value.as_bytes(),
            None,
        );
        append_cross_frame(
            &mut host_corpus,
            &mut payload_corpus,
            &mut frame_count,
            &host_challenge,
            &nested_challenge,
        );
        payload_machine
            .accept_frame(
                payload_protocol::HostDirection::HostToSupervisor,
                &host_challenge,
            )
            .unwrap();
        payload_machine
            .accept_eof(payload_protocol::HostDirection::HostToSupervisor)
            .unwrap();
        host_machine.accept_host_to_supervisor_eof().unwrap();

        let terminal_sequence = sequence + 1;
        let host_terminal = host_wire(
            &encode_terminal(
                case.instance,
                campaign,
                terminal_sequence,
                host_challenge_value,
                host_state,
            )
            .unwrap(),
        );
        let nested_terminal = payload_frame(
            binding,
            contract::MessageKind::Terminal,
            terminal_sequence,
            u16::try_from(case.category).unwrap(),
            &payload_protocol::encode_terminal_payload(nested_challenge_value, nested_state),
            None,
        );
        append_cross_frame(
            &mut host_corpus,
            &mut payload_corpus,
            &mut frame_count,
            &host_terminal,
            &nested_terminal,
        );
        payload_machine
            .accept_frame(
                payload_protocol::HostDirection::SupervisorToHost,
                &host_terminal,
            )
            .unwrap();
        assert_eq!(
            accept_payload_wire_by_host(&mut host_machine, &nested_terminal),
            C2b2aHostProtocolEvent::TerminalAccepted(host_state)
        );
        payload_machine
            .accept_eof(payload_protocol::HostDirection::SupervisorToHost)
            .unwrap();
        assert!(payload_machine.is_complete());
        assert_eq!(
            host_machine.finish_on_supervisor_eof().unwrap(),
            C2b2aHostProtocolCompletion::ChallengedTerminal {
                state: host_state,
                measurements_seen: u32::try_from(case.measurements).unwrap(),
            }
        );
    }

    assert_eq!(frame_count, 385);
    assert_eq!(host_corpus, payload_corpus);
    assert_eq!(host_corpus.len(), 136_468);
    assert_eq!(sha256_hex(&host_corpus), FAILURE_FRAME_CORPUS_SHA256);
    assert_eq!(sha256_hex(&payload_corpus), FAILURE_FRAME_CORPUS_SHA256);
}

#[test]
fn cross_golden_adjacent_invalid_failure_progress_matrix_permanently_poison_both() {
    let cases = cross_failure_cases();
    let find = |label| {
        *cases
            .iter()
            .find(|candidate| candidate.label == label)
            .unwrap()
    };
    let c01_first = find("c01_lower_first_no_dispatch");
    let c01_second = find("c01_lower_second_acknowledged");
    let c01_last = find("c01_upper_last_acknowledged");
    let positive_first = find("positive_lower_first_no_dispatch");
    let positive_second = find("positive_lower_second_acknowledged");
    let positive_last = find("positive_upper_last_acknowledged");
    let shared_first = find("shared_lower_first");
    let shared_second = find("shared_lower_second");
    let shared_last = find("shared_upper_last");
    let dedicated = find("dedicated_lower");

    let matrix = vec![
        (
            "c01_wrong_active_item",
            CrossFailureCase {
                active_item: Some((1, 2)),
                ..c01_first
            },
            c01_first,
        ),
        (
            "c01_completed_case_nonzero",
            CrossFailureCase {
                completed_cases: 1,
                ..c01_first
            },
            c01_first,
        ),
        (
            "c01_completed_subattempt_bound",
            CrossFailureCase {
                completed_subattempts: 3,
                ..c01_last
            },
            c01_last,
        ),
        (
            "c01_below_completed_interval",
            CrossFailureCase {
                measurements: 6,
                ..c01_second
            },
            CrossFailureCase {
                measurements: 6,
                ..c01_first
            },
        ),
        (
            "c01_above_active_interval",
            CrossFailureCase {
                measurements: 8,
                ..c01_first
            },
            CrossFailureCase {
                measurements: 8,
                ..c01_second
            },
        ),
        (
            "positive_wrong_active_item",
            CrossFailureCase {
                active_item: Some((2, 2)),
                ..positive_first
            },
            positive_first,
        ),
        (
            "positive_completed_subattempt_nonzero",
            CrossFailureCase {
                completed_subattempts: 1,
                ..positive_first
            },
            positive_first,
        ),
        (
            "positive_completed_case_bound",
            CrossFailureCase {
                completed_cases: 20,
                ..positive_last
            },
            positive_last,
        ),
        (
            "positive_below_completed_interval",
            CrossFailureCase {
                measurements: 6,
                ..positive_second
            },
            CrossFailureCase {
                instance: positive_second.instance,
                measurements: 6,
                ..positive_first
            },
        ),
        (
            "positive_above_active_interval",
            CrossFailureCase {
                measurements: 8,
                ..positive_first
            },
            CrossFailureCase {
                instance: positive_first.instance,
                measurements: 8,
                ..positive_second
            },
        ),
        (
            "shared_dispatch_forbidden",
            CrossFailureCase {
                dispatched: true,
                acknowledged: true,
                ..shared_first
            },
            shared_first,
        ),
        (
            "shared_completed_case_nonzero",
            CrossFailureCase {
                completed_cases: 1,
                ..shared_first
            },
            shared_first,
        ),
        (
            "shared_completed_subattempt_bound",
            CrossFailureCase {
                completed_subattempts: 60,
                ..shared_last
            },
            shared_last,
        ),
        (
            "shared_below_completed_interval",
            CrossFailureCase {
                measurements: 0,
                ..shared_second
            },
            shared_first,
        ),
        (
            "shared_above_active_interval",
            CrossFailureCase {
                measurements: 2,
                ..shared_first
            },
            CrossFailureCase {
                measurements: 2,
                ..shared_second
            },
        ),
        (
            "dedicated_dispatch_forbidden",
            CrossFailureCase {
                dispatched: true,
                ..dedicated
            },
            dedicated,
        ),
        (
            "dedicated_completed_case_nonzero",
            CrossFailureCase {
                completed_cases: 1,
                ..dedicated
            },
            dedicated,
        ),
        (
            "dedicated_completed_subattempt_bound",
            CrossFailureCase {
                completed_subattempts: 1,
                measurements: 1,
                ..dedicated
            },
            CrossFailureCase {
                measurements: 1,
                ..dedicated
            },
        ),
        (
            "dedicated_wrong_active_item",
            CrossFailureCase {
                active_item: Some((12, 2)),
                ..dedicated
            },
            dedicated,
        ),
    ];
    assert_eq!(
        matrix.iter().map(|(label, ..)| *label).collect::<Vec<_>>(),
        [
            "c01_wrong_active_item",
            "c01_completed_case_nonzero",
            "c01_completed_subattempt_bound",
            "c01_below_completed_interval",
            "c01_above_active_interval",
            "positive_wrong_active_item",
            "positive_completed_subattempt_nonzero",
            "positive_completed_case_bound",
            "positive_below_completed_interval",
            "positive_above_active_interval",
            "shared_dispatch_forbidden",
            "shared_completed_case_nonzero",
            "shared_completed_subattempt_bound",
            "shared_below_completed_interval",
            "shared_above_active_interval",
            "dedicated_dispatch_forbidden",
            "dedicated_completed_case_nonzero",
            "dedicated_completed_subattempt_bound",
            "dedicated_wrong_active_item",
        ]
    );
    for (_, invalid, valid) in matrix {
        assert_invalid_failure_progress_poison(invalid, valid);
    }
}

#[test]
fn cross_golden_failure_phase_milestones_are_mirrored_and_poison_permanently() {
    let valid_case = *cross_failure_cases()
        .iter()
        .find(|case| case.label == "c01_lower_first_no_dispatch")
        .unwrap();
    let sequence = 2;
    let campaign = parity_campaign(0);
    let valid = host_wire(
        &encode_terminal_ready(
            valid_case.instance,
            campaign,
            sequence,
            host_failure_state(valid_case),
        )
        .unwrap(),
    );
    let mut mutations = vec![
        (
            "dispatch_without_ack_requires_outcome_uncertain",
            mutate_wire(valid.clone(), |wire| {
                put_u32(wire, C2B2A_HOST_HEADER_BYTES + 12, 1);
                reseal_wire_payload(wire);
            }),
        ),
        (
            "outcome_uncertain_forbidden_before_dispatch",
            mutate_wire(valid.clone(), |wire| {
                put_u16(wire, 24, 8);
                put_u32(wire, C2B2A_HOST_HEADER_BYTES, 8);
                reseal_wire_payload(wire);
            }),
        ),
        (
            "outcome_uncertain_forbidden_after_acknowledgement",
            mutate_wire(valid.clone(), |wire| {
                put_u16(wire, 24, 8);
                put_u32(wire, C2B2A_HOST_HEADER_BYTES, 8);
                put_u32(wire, C2B2A_HOST_HEADER_BYTES + 12, 1);
                put_u32(wire, C2B2A_HOST_HEADER_BYTES + 16, 1);
                reseal_wire_payload(wire);
            }),
        ),
    ];
    for (label, category) in [
        ("exclusivity_forbidden_before_dispatch", 9_u16),
        ("contender_forbidden_before_dispatch", 10),
        ("release_forbidden_before_dispatch", 11),
        ("reopen_forbidden_before_dispatch", 12),
        ("engine_read_forbidden_before_dispatch", 13),
        ("canonical_forbidden_before_dispatch", 14),
    ] {
        mutations.push((
            label,
            mutate_wire(valid.clone(), |wire| {
                put_u16(wire, 24, category);
                put_u32(wire, C2B2A_HOST_HEADER_BYTES, u32::from(category));
                reseal_wire_payload(wire);
            }),
        ));
    }
    for (label, category) in [
        ("static_firewall_forbidden_after_acknowledgement", 4_u16),
        ("invalid_fixture_forbidden_after_acknowledgement", 6),
        ("store_open_forbidden_after_acknowledgement", 7),
    ] {
        mutations.push((
            label,
            mutate_wire(valid.clone(), |wire| {
                put_u16(wire, 24, category);
                put_u32(wire, C2B2A_HOST_HEADER_BYTES, u32::from(category));
                put_u32(wire, C2B2A_HOST_HEADER_BYTES + 12, 1);
                put_u32(wire, C2B2A_HOST_HEADER_BYTES + 16, 1);
                reseal_wire_payload(wire);
            }),
        ));
    }

    for (label, invalid) in mutations {
        assert!(
            host_decode_wire(C2b2aDirection::SupervisorToHost, &invalid).is_err(),
            "host decoder accepted {label}"
        );
        assert!(
            payload_decode_host_wire(&invalid).is_err(),
            "nested decoder accepted {label}"
        );
        let (mut host_machine, mut payload_machine, _, _) =
            crossed_protocols_after_measurements(valid_case.instance, 0);
        let header: &[u8; C2B2A_HOST_HEADER_BYTES] =
            invalid[..C2B2A_HOST_HEADER_BYTES].try_into().unwrap();
        assert!(host_machine
            .accept_supervisor_frame(header, &invalid[C2B2A_HOST_HEADER_BYTES..])
            .is_err());
        assert!(payload_machine
            .accept_frame(payload_protocol::HostDirection::SupervisorToHost, &invalid,)
            .is_err());

        let valid_header: &[u8; C2B2A_HOST_HEADER_BYTES] =
            valid[..C2B2A_HOST_HEADER_BYTES].try_into().unwrap();
        assert!(host_machine
            .accept_supervisor_frame(valid_header, &valid[C2B2A_HOST_HEADER_BYTES..])
            .is_err());
        assert_eq!(
            payload_machine
                .accept_frame(payload_protocol::HostDirection::SupervisorToHost, &valid,),
            Err(payload_protocol::ProtocolError::Poisoned)
        );
    }
}

#[test]
fn cross_golden_initial_host_abort_eof_direction_and_poison_are_exact() {
    let instance = C2b2aInstanceName::C01;
    let campaign = failure_campaign(40);
    let binding = payload_binding(instance, campaign);
    let entry = payload_entry(instance);
    let selector = payload_selector(entry);
    let case = CrossFailureCase {
        label: "host_abort_eof_matrix",
        instance,
        category: 16,
        measurements: 0,
        active_item: None,
        cleanup: true,
        dispatched: false,
        acknowledged: false,
        reaped: true,
        completed_cases: 0,
        completed_subattempts: 0,
        path: CrossFailurePath::HostAbortBeforeStart,
    };
    let state = host_failure_state(case);
    let nested_state = payload_failure_state(case);
    let independent_abort = payload_frame(
        binding,
        contract::MessageKind::Abort,
        1,
        u16::try_from(case.category).unwrap(),
        &payload_protocol::encode_terminal_state(nested_state),
        None,
    );

    let host_abort = || {
        let (mut machine, _) =
            C2b2aHostProtocolMachine::begin_for_test(instance, campaign).unwrap();
        let abort = host_wire(
            &machine
                .replace_initial_host_start_with_abort(campaign, state)
                .unwrap(),
        );
        assert_eq!(abort, independent_abort);
        (machine, abort)
    };

    let (mut valid_host, valid_abort) = host_abort();
    let mut valid_nested =
        payload_protocol::HostProtocolV1::new_bound_for_test(entry, selector, binding).unwrap();
    valid_nested
        .accept_frame(
            payload_protocol::HostDirection::HostToSupervisor,
            &valid_abort,
        )
        .unwrap();
    valid_nested
        .accept_eof(payload_protocol::HostDirection::HostToSupervisor)
        .unwrap();
    valid_host.accept_host_to_supervisor_eof().unwrap();
    assert!(valid_nested.is_complete());
    assert!(matches!(
        valid_host.finish_after_host_to_supervisor_eof().unwrap(),
        C2b2aHostProtocolCompletion::UnchallengedFailure { .. }
    ));

    let (mut host_early, _) = C2b2aHostProtocolMachine::begin_for_test(instance, campaign).unwrap();
    assert!(host_early.accept_host_to_supervisor_eof().is_err());
    assert!(host_early
        .replace_initial_host_start_with_abort(campaign, state)
        .is_err());
    let mut nested_early =
        payload_protocol::HostProtocolV1::new_bound_for_test(entry, selector, binding).unwrap();
    assert!(nested_early
        .accept_eof(payload_protocol::HostDirection::HostToSupervisor)
        .is_err());
    assert_eq!(
        nested_early.accept_frame(
            payload_protocol::HostDirection::HostToSupervisor,
            &independent_abort,
        ),
        Err(payload_protocol::ProtocolError::Poisoned)
    );

    let mut wrong_direction = C2b2aCampaignController::new();
    wrong_direction.begin_next(campaign).unwrap();
    wrong_direction
        .replace_active_initial_host_start_with_abort(state)
        .unwrap();
    assert!(wrong_direction.finish_active_on_supervisor_eof().is_err());
    assert!(wrong_direction
        .accept_active_host_to_supervisor_eof()
        .is_err());
    let mut nested_wrong =
        payload_protocol::HostProtocolV1::new_bound_for_test(entry, selector, binding).unwrap();
    nested_wrong
        .accept_frame(
            payload_protocol::HostDirection::HostToSupervisor,
            &independent_abort,
        )
        .unwrap();
    assert!(nested_wrong
        .accept_eof(payload_protocol::HostDirection::SupervisorToHost)
        .is_err());
    assert_eq!(
        nested_wrong.accept_eof(payload_protocol::HostDirection::HostToSupervisor),
        Err(payload_protocol::ProtocolError::Poisoned)
    );

    let (mut host_duplicate, _) = host_abort();
    host_duplicate.accept_host_to_supervisor_eof().unwrap();
    assert!(host_duplicate.accept_host_to_supervisor_eof().is_err());
    assert!(host_duplicate
        .finish_after_host_to_supervisor_eof()
        .is_err());
    let mut nested_duplicate =
        payload_protocol::HostProtocolV1::new_bound_for_test(entry, selector, binding).unwrap();
    nested_duplicate
        .accept_frame(
            payload_protocol::HostDirection::HostToSupervisor,
            &independent_abort,
        )
        .unwrap();
    nested_duplicate
        .accept_eof(payload_protocol::HostDirection::HostToSupervisor)
        .unwrap();
    assert_eq!(
        nested_duplicate.accept_eof(payload_protocol::HostDirection::HostToSupervisor),
        Err(payload_protocol::ProtocolError::LateEvent)
    );
    assert_eq!(
        nested_duplicate.accept_eof(payload_protocol::HostDirection::HostToSupervisor),
        Err(payload_protocol::ProtocolError::Poisoned)
    );

    let mut late = C2b2aCampaignController::new();
    late.begin_next(campaign).unwrap();
    late.replace_active_initial_host_start_with_abort(state)
        .unwrap();
    late.accept_active_host_to_supervisor_eof().unwrap();
    late.finish_active_after_host_to_supervisor_eof().unwrap();
    assert!(late.accept_active_host_to_supervisor_eof().is_err());
    assert!(late.attest_runtime_closure_for_test(true).is_err());
}

#[test]
fn cross_golden_host_terminal_instances_reject_post_ready_failure_and_abort() {
    for (instance_index, instance) in [C2b2aInstanceName::B40, C2b2aInstanceName::B41]
        .into_iter()
        .enumerate()
    {
        let campaign = failure_campaign(instance_index + 30);
        let binding = payload_binding(instance, campaign);
        let entry = payload_entry(instance);
        let selector = payload_selector(entry);
        let start = host_wire(&encode_host_start(instance, campaign).unwrap());
        let ready = host_wire(&encode_host_ready(instance, campaign).unwrap());
        let failure_case = CrossFailureCase {
            label: "host_terminal_post_ready_failure",
            instance,
            category: 16,
            measurements: 0,
            active_item: None,
            cleanup: true,
            dispatched: false,
            acknowledged: false,
            reaped: true,
            completed_cases: 0,
            completed_subattempts: 0,
            path: CrossFailurePath::Challenged,
        };

        for kind in [
            contract::MessageKind::TerminalReady,
            contract::MessageKind::Abort,
        ] {
            let (mut host_machine, _) =
                C2b2aHostProtocolMachine::begin_for_test(instance, campaign).unwrap();
            let mut nested_machine =
                payload_protocol::HostProtocolV1::new_bound_for_test(entry, selector, binding)
                    .unwrap();
            nested_machine
                .accept_frame(payload_protocol::HostDirection::HostToSupervisor, &start)
                .unwrap();
            nested_machine
                .accept_frame(payload_protocol::HostDirection::SupervisorToHost, &ready)
                .unwrap();
            accept_payload_wire_by_host(&mut host_machine, &ready);

            let host_state = host_failure_state(failure_case);
            let nested_state = payload_failure_state(failure_case);
            let host_frame = match kind {
                contract::MessageKind::TerminalReady => {
                    host_wire(&encode_terminal_ready(instance, campaign, 2, host_state).unwrap())
                }
                contract::MessageKind::Abort => {
                    host_wire(&encode_supervisor_abort(instance, campaign, 2, host_state).unwrap())
                }
                _ => unreachable!(),
            };
            let nested_frame = payload_frame(
                binding,
                kind,
                2,
                u16::try_from(failure_case.category).unwrap(),
                &payload_protocol::encode_terminal_state(nested_state),
                None,
            );
            assert_eq!(host_frame, nested_frame);
            let header: &[u8; C2B2A_HOST_HEADER_BYTES] =
                nested_frame[..C2B2A_HOST_HEADER_BYTES].try_into().unwrap();
            assert!(host_machine
                .accept_supervisor_frame(header, &nested_frame[C2B2A_HOST_HEADER_BYTES..],)
                .is_err());
            assert!(nested_machine
                .accept_frame(
                    payload_protocol::HostDirection::SupervisorToHost,
                    &host_frame,
                )
                .is_err());
            assert!(host_machine.finish_on_supervisor_eof().is_err());
            assert_eq!(
                nested_machine.accept_eof(payload_protocol::HostDirection::SupervisorToHost),
                Err(payload_protocol::ProtocolError::Poisoned)
            );
        }
    }
}

#[test]
fn cross_golden_closed_header_and_selector_mutation_matrix_fails_both_decoders() {
    let baseline =
        host_wire(&encode_host_start(C2b2aInstanceName::C01, parity_campaign(0)).unwrap());
    assert!(host_decode_wire(C2b2aDirection::HostToSupervisor, &baseline).is_ok());
    assert!(payload_decode_host_wire(&baseline).is_ok());

    let mutations = vec![
        ("magic", mutate_wire(baseline.clone(), |wire| wire[0] ^= 1)),
        (
            "version",
            mutate_wire(baseline.clone(), |wire| put_u16(wire, 8, 2)),
        ),
        (
            "kind",
            mutate_wire(baseline.clone(), |wire| put_u16(wire, 10, u16::MAX)),
        ),
        (
            "zero_sequence",
            mutate_wire(baseline.clone(), |wire| put_u32(wire, 12, 0)),
        ),
        (
            "item_id",
            mutate_wire(baseline.clone(), |wire| put_u16(wire, 16, 1)),
        ),
        (
            "subitem_id",
            mutate_wire(baseline.clone(), |wire| put_u16(wire, 18, 1)),
        ),
        (
            "profile",
            mutate_wire(baseline.clone(), |wire| put_u16(wire, 20, 2)),
        ),
        (
            "role",
            mutate_wire(baseline.clone(), |wire| put_u16(wire, 22, u16::MAX)),
        ),
        (
            "status",
            mutate_wire(baseline.clone(), |wire| put_u16(wire, 24, 1)),
        ),
        (
            "flags",
            mutate_wire(baseline.clone(), |wire| put_u16(wire, 26, 1)),
        ),
        (
            "payload_length",
            mutate_wire(baseline.clone(), |wire| put_u32(wire, 28, 7)),
        ),
        (
            "contract",
            mutate_wire(baseline.clone(), |wire| wire[64] ^= 1),
        ),
        (
            "payload_digest",
            mutate_wire(baseline.clone(), |wire| wire[128] ^= 1),
        ),
        (
            "missing_payload_byte",
            mutate_wire(baseline.clone(), |wire| {
                wire.pop();
            }),
        ),
        (
            "trailing_payload_byte",
            mutate_wire(baseline.clone(), |wire| wire.push(0)),
        ),
        (
            "unresealed_payload",
            mutate_wire(baseline.clone(), |wire| {
                wire[C2B2A_HOST_HEADER_BYTES] ^= 1;
            }),
        ),
        (
            "selector_phase",
            mutate_wire(baseline.clone(), |wire| {
                put_u16(wire, C2B2A_HOST_HEADER_BYTES, 0);
                reseal_wire_payload(wire);
            }),
        ),
        (
            "selector_class",
            mutate_wire(baseline.clone(), |wire| {
                put_u16(wire, C2B2A_HOST_HEADER_BYTES + 2, 0);
                reseal_wire_payload(wire);
            }),
        ),
        (
            "selector_ordinal",
            mutate_wire(baseline.clone(), |wire| {
                put_u16(wire, C2B2A_HOST_HEADER_BYTES + 4, 0);
                reseal_wire_payload(wire);
            }),
        ),
        (
            "selector_reserved",
            mutate_wire(baseline.clone(), |wire| {
                put_u16(wire, C2B2A_HOST_HEADER_BYTES + 6, 1);
                reseal_wire_payload(wire);
            }),
        ),
    ];
    let labels: Vec<_> = mutations.iter().map(|(label, _)| *label).collect();
    assert_eq!(
        labels,
        [
            "magic",
            "version",
            "kind",
            "zero_sequence",
            "item_id",
            "subitem_id",
            "profile",
            "role",
            "status",
            "flags",
            "payload_length",
            "contract",
            "payload_digest",
            "missing_payload_byte",
            "trailing_payload_byte",
            "unresealed_payload",
            "selector_phase",
            "selector_class",
            "selector_ordinal",
            "selector_reserved",
        ]
    );
    for (label, mutated) in mutations {
        assert!(
            host_decode_wire(C2b2aDirection::HostToSupervisor, &mutated).is_err(),
            "host decoder accepted {label}"
        );
        assert!(
            payload_decode_host_wire(&mutated).is_err(),
            "payload decoder accepted {label}"
        );
    }
}

#[test]
fn cross_golden_stream_binding_sequence_and_legal_selector_drift_poison_both() {
    let instance = C2b2aInstanceName::C01;
    let campaign = parity_campaign(0);
    let binding = payload_binding(instance, campaign);
    let entry = payload_entry(instance);
    let selector = payload_selector(entry);
    let host_start = host_wire(&encode_host_start(instance, campaign).unwrap());
    let valid_ready = host_wire(&encode_host_ready(instance, campaign).unwrap());
    let different_selector = payload_protocol::encode_host_selector(payload_selector(
        payload_entry(C2b2aInstanceName::P01),
    ));
    let mutations = vec![
        (
            "sequence",
            mutate_wire(valid_ready.clone(), |wire| put_u32(wire, 12, 2)),
        ),
        (
            "nonce",
            mutate_wire(valid_ready.clone(), |wire| wire[32] ^= 1),
        ),
        (
            "contract",
            mutate_wire(valid_ready.clone(), |wire| wire[64] ^= 1),
        ),
        (
            "schedule",
            mutate_wire(valid_ready.clone(), |wire| wire[96] ^= 1),
        ),
        (
            "different_legal_selector",
            mutate_wire(valid_ready.clone(), |wire| {
                wire[C2B2A_HOST_HEADER_BYTES..].copy_from_slice(&different_selector);
                reseal_wire_payload(wire);
            }),
        ),
    ];
    assert_eq!(
        mutations
            .iter()
            .map(|(label, _)| *label)
            .collect::<Vec<_>>(),
        [
            "sequence",
            "nonce",
            "contract",
            "schedule",
            "different_legal_selector",
        ]
    );

    for (label, mutated) in mutations {
        let (mut host_machine, _) =
            C2b2aHostProtocolMachine::begin_for_test(instance, campaign).unwrap();
        let mut payload_machine =
            payload_protocol::HostProtocolV1::new_bound_for_test(entry, selector, binding).unwrap();
        payload_machine
            .accept_frame(
                payload_protocol::HostDirection::HostToSupervisor,
                &host_start,
            )
            .unwrap();

        let header: &[u8; C2B2A_HOST_HEADER_BYTES] =
            mutated[..C2B2A_HOST_HEADER_BYTES].try_into().unwrap();
        assert!(
            host_machine
                .accept_supervisor_frame(header, &mutated[C2B2A_HOST_HEADER_BYTES..])
                .is_err(),
            "host stream accepted {label}"
        );
        assert!(
            payload_machine
                .accept_frame(payload_protocol::HostDirection::SupervisorToHost, &mutated,)
                .is_err(),
            "payload stream accepted {label}"
        );

        let ready_header: &[u8; C2B2A_HOST_HEADER_BYTES] =
            valid_ready[..C2B2A_HOST_HEADER_BYTES].try_into().unwrap();
        assert!(host_machine
            .accept_supervisor_frame(ready_header, &valid_ready[C2B2A_HOST_HEADER_BYTES..],)
            .is_err());
        assert_eq!(
            payload_machine.accept_frame(
                payload_protocol::HostDirection::SupervisorToHost,
                &valid_ready,
            ),
            Err(payload_protocol::ProtocolError::Poisoned)
        );
    }
}

#[test]
fn cross_golden_constructor_domain_requires_contract_but_defers_schedule_authority() {
    assert_eq!(
        C2B2A_CONTRACT_SHA256.as_bytes(),
        contract::CONTRACT_SHA256.as_bytes()
    );
    let instance = C2b2aInstanceName::P01;
    let entry = payload_entry(instance);
    let selector = payload_selector(entry);

    let mut wrong_contract = parity_campaign(0);
    wrong_contract.contract_sha256 = digest(0x99);
    assert!(C2b2aHostProtocolMachine::begin_for_test(instance, wrong_contract).is_err());
    let wrong_contract_frame = host_wire(&encode_host_start(instance, wrong_contract).unwrap());
    assert!(host_decode_wire(C2b2aDirection::HostToSupervisor, &wrong_contract_frame).is_err());
    assert!(payload_decode_host_wire(&wrong_contract_frame).is_err());
    let wrong_nested_binding = payload_protocol::StreamBinding {
        run_nonce: contract::Digest32(*wrong_contract.nonce.as_bytes()),
        contract_sha256: contract::Digest32([0x99; 32]),
        manifest_sha256: contract::Digest32(*wrong_contract.acceptance_schedule_sha256.as_bytes()),
    };
    let nested_wrong =
        payload_protocol::HostProtocolV1::new_bound_for_test(entry, selector, wrong_nested_binding);
    assert_eq!(
        nested_wrong,
        Err(payload_protocol::ProtocolError::BindingMismatch)
    );

    let mut controller = C2b2aCampaignController::new();
    assert!(controller.begin_next(wrong_contract).is_err());
    assert!(controller.begin_next(parity_campaign(1)).is_err());

    let mut caller_supplied_schedule = parity_campaign(2);
    caller_supplied_schedule.characterization_schedule_sha256 = digest(0x55);
    caller_supplied_schedule.acceptance_schedule_sha256 = digest(0x66);
    assert!(C2b2aHostProtocolMachine::begin_for_test(instance, caller_supplied_schedule).is_ok());
    let nested_binding = payload_binding(instance, caller_supplied_schedule);
    assert!(
        payload_protocol::HostProtocolV1::new_bound_for_test(entry, selector, nested_binding,)
            .is_ok()
    );
    assert_eq!(
        payload_protocol::HostProtocolV1::new(
            contract::Digest32(*caller_supplied_schedule.nonce.as_bytes()),
            selector,
        ),
        Err(payload_protocol::ProtocolError::ManifestDigestNotFrozen(
            contract::ManifestKindV1::AcceptanceSchedule,
        ))
    );
    // Constructor and standalone decoder parity are exact for the accepted contract. Schedule
    // digests remain explicitly caller-supplied in the host foundation and non-authoritative in
    // the nested test seam until a later reviewed record freezes them; neither constructor is
    // runtime/handle evidence.
}

#[test]
fn host_machine_requires_real_write_close_transition_and_poison_is_permanent() {
    let instance = C2b2aInstanceName::B12O;
    let campaign = bindings(1);
    let state = C2b2aTerminalState::success_for(instance).unwrap();
    let terminal = encode_terminal(instance, campaign, 4, challenge(7), state).unwrap();
    let prepared = || {
        let mut machine = enter_collecting(instance, campaign);
        accept_all_measurements(&mut machine, instance, campaign);
        let ready = encode_terminal_ready(instance, campaign, 3, state).unwrap();
        accept_frame(&mut machine, &ready);
        machine
            .issue_challenge_for_test(campaign, challenge(7))
            .unwrap();
        machine
    };

    let (mut early, _) = C2b2aHostProtocolMachine::begin_for_test(instance, campaign).unwrap();
    assert!(early.accept_host_to_supervisor_eof().is_err());
    let ready = encode_host_ready(instance, campaign).unwrap();
    assert!(early
        .accept_supervisor_frame(ready.header(), ready.payload())
        .is_err());

    let mut missing = prepared();
    assert!(missing
        .accept_supervisor_frame(terminal.header(), terminal.payload())
        .is_err());
    assert!(missing.accept_host_to_supervisor_eof().is_err());

    let mut duplicate = prepared();
    duplicate.accept_host_to_supervisor_eof().unwrap();
    assert!(duplicate.accept_host_to_supervisor_eof().is_err());
    assert!(duplicate
        .accept_supervisor_frame(terminal.header(), terminal.payload())
        .is_err());

    let mut late = prepared();
    late.accept_host_to_supervisor_eof().unwrap();
    accept_frame(&mut late, &terminal);
    assert!(late.accept_host_to_supervisor_eof().is_err());
    assert!(late.finish_on_supervisor_eof().is_err());

    let mut complete = prepared();
    complete.accept_host_to_supervisor_eof().unwrap();
    accept_frame(&mut complete, &terminal);
    assert!(matches!(
        complete.finish_on_supervisor_eof().unwrap(),
        C2b2aHostProtocolCompletion::ChallengedTerminal { .. }
    ));

    let mut controller = C2b2aCampaignController::new();
    controller.begin_next(campaign).unwrap();
    assert!(controller.accept_active_host_to_supervisor_eof().is_err());
    assert!(controller
        .accept_supervisor_frame(ready.header(), ready.payload())
        .is_err());
}

#[test]
fn cross_golden_measurement_semantic_mutation_matrix_poison_both() {
    let b01 = C2b2aInstanceName::B01;
    let b01_count = usize::try_from(instance_spec(b01).measurement_count).unwrap();
    let probe_17 = (0..b01_count)
        .find(|index| {
            expected_measurement_at(b01, *index)
                .unwrap()
                .case_or_probe_id
                == 17
        })
        .unwrap();
    let probe_18 = (0..b01_count)
        .find(|index| {
            expected_measurement_at(b01, *index)
                .unwrap()
                .case_or_probe_id
                == 18
        })
        .unwrap();

    let matrix = [
        ("scope_drift", C2b2aInstanceName::P01, 0, 0, 2),
        ("probe_17_missing_diagnostic_fd", b01, probe_17, 30, 0),
        ("probe_18_missing_socket_fd", b01, probe_18, 30, 0),
        ("probe_18_forbidden_high_fd", b01, probe_18, 30, 1 << 15),
        (
            "probe_15_nonzero_inapplicable_field",
            C2b2aInstanceName::B15,
            0,
            1,
            1,
        ),
        (
            "probe_16_nonzero_inapplicable_field",
            C2b2aInstanceName::B16,
            0,
            29,
            1,
        ),
    ];
    assert_eq!(
        matrix.map(|(label, ..)| label),
        [
            "scope_drift",
            "probe_17_missing_diagnostic_fd",
            "probe_18_missing_socket_fd",
            "probe_18_forbidden_high_fd",
            "probe_15_nonzero_inapplicable_field",
            "probe_16_nonzero_inapplicable_field",
        ]
    );
    for (_, instance, measurement_index, word_index, replacement) in matrix {
        assert_measurement_word_mutation_rejected(
            instance,
            measurement_index,
            word_index,
            replacement,
        );
    }
}

#[test]
fn cross_golden_terminal_and_abort_semantic_mutation_matrix_fails_both_decoders() {
    let instance = C2b2aInstanceName::C01;
    let campaign = parity_campaign(0);
    let binding = payload_binding(instance, campaign);
    let failure = C2b2aTerminalState {
        primary_category: C2b2aPrimaryCategory::ProtocolFailure,
        active_item_code: C2b2aTerminalState::active_item(1, 1).unwrap(),
        guest_cleanup_proven: true,
        write_may_have_been_dispatched: false,
        writer_acknowledged: false,
        roles_reaped: true,
        completed_case_count: 0,
        completed_subattempt_count: 0,
    };
    let baseline = host_wire(&encode_terminal_ready(instance, campaign, 2, failure).unwrap());
    let nested_baseline = payload_frame(
        binding,
        contract::MessageKind::TerminalReady,
        2,
        contract::ErrorCategory::ProtocolFailure.header_code(),
        &payload_protocol::encode_terminal_state(contract::TerminalStateV1 {
            primary_category: contract::ErrorCategory::ProtocolFailure as u32,
            active_item_code: contract::active_item_code(1, 1),
            guest_cleanup_proven: 1,
            write_may_have_been_dispatched: 0,
            writer_acknowledged: 0,
            roles_reaped: 1,
            completed_case_count: 0,
            completed_subattempt_count: 0,
        }),
        None,
    );
    assert_eq!(baseline, nested_baseline);
    assert!(host_decode_wire(C2b2aDirection::SupervisorToHost, &baseline).is_ok());
    assert!(payload_decode_host_wire(&baseline).is_ok());

    let mut mutations = Vec::new();
    mutations.push((
        "unknown_category",
        mutate_wire(baseline.clone(), |wire| {
            put_u16(wire, 24, 18);
            put_u32(wire, C2B2A_HOST_HEADER_BYTES, 18);
            reseal_wire_payload(wire);
        }),
    ));
    for (label, active_item) in [
        ("half_zero_active_item_high", 1_u32 << 16),
        ("half_zero_active_item_low", 1_u32),
    ] {
        mutations.push((
            label,
            mutate_wire(baseline.clone(), |wire| {
                put_u32(wire, C2B2A_HOST_HEADER_BYTES + 4, active_item);
                reseal_wire_payload(wire);
            }),
        ));
    }
    for (label, word_index) in [
        ("cleanup_nonboolean", 2_usize),
        ("dispatch_nonboolean", 3),
        ("ack_nonboolean", 4),
        ("reap_nonboolean", 5),
    ] {
        mutations.push((
            label,
            mutate_wire(baseline.clone(), |wire| {
                put_u32(wire, C2B2A_HOST_HEADER_BYTES + word_index * 4, 2);
                reseal_wire_payload(wire);
            }),
        ));
    }
    mutations.push((
        "cleanup_without_reap",
        mutate_wire(baseline.clone(), |wire| {
            put_u32(wire, C2B2A_HOST_HEADER_BYTES + 5 * 4, 0);
            reseal_wire_payload(wire);
        }),
    ));
    mutations.push((
        "ack_without_dispatch",
        mutate_wire(baseline.clone(), |wire| {
            put_u32(wire, C2B2A_HOST_HEADER_BYTES + 4 * 4, 1);
            reseal_wire_payload(wire);
        }),
    ));
    mutations.push((
        "terminal_status_mismatch",
        mutate_wire(baseline.clone(), |wire| put_u16(wire, 24, 15)),
    ));
    assert_eq!(
        mutations
            .iter()
            .map(|(label, _)| *label)
            .collect::<Vec<_>>(),
        [
            "unknown_category",
            "half_zero_active_item_high",
            "half_zero_active_item_low",
            "cleanup_nonboolean",
            "dispatch_nonboolean",
            "ack_nonboolean",
            "reap_nonboolean",
            "cleanup_without_reap",
            "ack_without_dispatch",
            "terminal_status_mismatch",
        ]
    );
    for (label, mutated) in mutations {
        assert!(
            host_decode_wire(C2b2aDirection::SupervisorToHost, &mutated).is_err(),
            "host decoder accepted {label}"
        );
        assert!(
            payload_decode_host_wire(&mutated).is_err(),
            "payload decoder accepted {label}"
        );
    }

    let pre_ready_failure = C2b2aTerminalState {
        active_item_code: 0,
        ..failure
    };
    let abort =
        host_wire(&encode_supervisor_abort(instance, campaign, 1, pre_ready_failure).unwrap());
    let abort_mutations = [
        (
            "abort_item_mismatch",
            mutate_wire(abort.clone(), |wire| put_u16(wire, 16, 1)),
        ),
        (
            "abort_status_mismatch",
            mutate_wire(abort.clone(), |wire| put_u16(wire, 24, 15)),
        ),
        (
            "abort_success",
            mutate_wire(abort, |wire| {
                put_u16(wire, 24, 0);
                put_u32(wire, C2B2A_HOST_HEADER_BYTES, 0);
                reseal_wire_payload(wire);
            }),
        ),
    ];
    for (label, mutated) in abort_mutations {
        assert!(
            host_decode_wire(C2b2aDirection::SupervisorToHost, &mutated).is_err(),
            "host decoder accepted {label}"
        );
        assert!(
            payload_decode_host_wire(&mutated).is_err(),
            "payload decoder accepted {label}"
        );
    }
}

#[test]
fn cross_golden_stateful_terminal_count_and_challenge_mutations_poison_both() {
    let instance = C2b2aInstanceName::P01;
    let count = usize::try_from(instance_spec(instance).measurement_count).unwrap();
    let (mut host_machine, mut payload_machine, campaign, binding) =
        crossed_protocols_after_measurements(instance, count);
    let entry = payload_entry(instance);
    let host_state = C2b2aTerminalState::success_for(instance).unwrap();
    let nested_state = contract::successful_terminal(entry).unwrap();
    let sequence = instance_spec(instance).terminal_ready_sequence.unwrap();
    let valid = payload_frame(
        binding,
        contract::MessageKind::TerminalReady,
        sequence,
        0,
        &payload_protocol::encode_terminal_state(nested_state),
        None,
    );
    let wrong_count = mutate_wire(valid.clone(), |wire| {
        put_u32(wire, C2B2A_HOST_HEADER_BYTES + 6 * 4, 19);
        reseal_wire_payload(wire);
    });
    let wrong_header: &[u8; C2B2A_HOST_HEADER_BYTES] =
        wrong_count[..C2B2A_HOST_HEADER_BYTES].try_into().unwrap();
    assert!(host_machine
        .accept_supervisor_frame(wrong_header, &wrong_count[C2B2A_HOST_HEADER_BYTES..],)
        .is_err());
    assert!(payload_machine
        .accept_frame(
            payload_protocol::HostDirection::SupervisorToHost,
            &wrong_count,
        )
        .is_err());
    let valid_host =
        host_wire(&encode_terminal_ready(instance, campaign, sequence, host_state).unwrap());
    let valid_header: &[u8; C2B2A_HOST_HEADER_BYTES] =
        valid_host[..C2B2A_HOST_HEADER_BYTES].try_into().unwrap();
    assert!(host_machine
        .accept_supervisor_frame(valid_header, &valid_host[C2B2A_HOST_HEADER_BYTES..],)
        .is_err());
    assert_eq!(
        payload_machine.accept_frame(payload_protocol::HostDirection::SupervisorToHost, &valid,),
        Err(payload_protocol::ProtocolError::Poisoned)
    );

    let instance = C2b2aInstanceName::B12O;
    let (mut host_machine, mut payload_machine, campaign, binding) =
        crossed_protocols_after_measurements(instance, 1);
    let entry = payload_entry(instance);
    let host_state = C2b2aTerminalState::success_for(instance).unwrap();
    let nested_state = contract::successful_terminal(entry).unwrap();
    let ready_sequence = instance_spec(instance).terminal_ready_sequence.unwrap();
    let host_ready =
        host_wire(&encode_terminal_ready(instance, campaign, ready_sequence, host_state).unwrap());
    let nested_ready = payload_frame(
        binding,
        contract::MessageKind::TerminalReady,
        ready_sequence,
        0,
        &payload_protocol::encode_terminal_state(nested_state),
        None,
    );
    payload_machine
        .accept_frame(
            payload_protocol::HostDirection::SupervisorToHost,
            &host_ready,
        )
        .unwrap();
    assert_eq!(
        accept_payload_wire_by_host(&mut host_machine, &nested_ready),
        C2b2aHostProtocolEvent::ChallengeRequired(host_state)
    );
    let host_challenge_value = parity_challenge(5);
    let nested_challenge_value = contract::Digest32(*host_challenge_value.as_bytes());
    let host_challenge = host_wire(
        &host_machine
            .issue_challenge_for_test(campaign, host_challenge_value)
            .unwrap(),
    );
    payload_machine
        .accept_frame(
            payload_protocol::HostDirection::HostToSupervisor,
            &host_challenge,
        )
        .unwrap();
    payload_machine
        .accept_eof(payload_protocol::HostDirection::HostToSupervisor)
        .unwrap();
    host_machine.accept_host_to_supervisor_eof().unwrap();

    let terminal_sequence = instance_spec(instance).terminal_sequence.unwrap();
    let valid_terminal = payload_frame(
        binding,
        contract::MessageKind::Terminal,
        terminal_sequence,
        0,
        &payload_protocol::encode_terminal_payload(nested_challenge_value, nested_state),
        None,
    );
    let wrong_challenge = mutate_wire(valid_terminal.clone(), |wire| {
        wire[C2B2A_HOST_HEADER_BYTES] ^= 1;
        reseal_wire_payload(wire);
    });
    let wrong_header: &[u8; C2B2A_HOST_HEADER_BYTES] = wrong_challenge[..C2B2A_HOST_HEADER_BYTES]
        .try_into()
        .unwrap();
    assert!(host_machine
        .accept_supervisor_frame(wrong_header, &wrong_challenge[C2B2A_HOST_HEADER_BYTES..],)
        .is_err());
    assert!(payload_machine
        .accept_frame(
            payload_protocol::HostDirection::SupervisorToHost,
            &wrong_challenge,
        )
        .is_err());
    let valid_header: &[u8; C2B2A_HOST_HEADER_BYTES] = valid_terminal[..C2B2A_HOST_HEADER_BYTES]
        .try_into()
        .unwrap();
    assert!(host_machine
        .accept_supervisor_frame(valid_header, &valid_terminal[C2B2A_HOST_HEADER_BYTES..],)
        .is_err());
    assert_eq!(
        payload_machine.accept_frame(
            payload_protocol::HostDirection::SupervisorToHost,
            &valid_terminal,
        ),
        Err(payload_protocol::ProtocolError::Poisoned)
    );
}

#[test]
fn cross_golden_direction_order_eof_late_and_retry_are_fail_closed() {
    let instance = C2b2aInstanceName::C01;
    let campaign = parity_campaign(0);
    let binding = payload_binding(instance, campaign);
    let entry = payload_entry(instance);
    let selector = payload_selector(entry);
    let host_start = host_wire(&encode_host_start(instance, campaign).unwrap());
    let ready = host_wire(&encode_host_ready(instance, campaign).unwrap());

    assert!(host_decode_wire(C2b2aDirection::HostToSupervisor, &ready).is_err());
    let mut wrong_direction =
        payload_protocol::HostProtocolV1::new_bound_for_test(entry, selector, binding).unwrap();
    wrong_direction
        .accept_frame(
            payload_protocol::HostDirection::HostToSupervisor,
            &host_start,
        )
        .unwrap();
    assert!(wrong_direction
        .accept_frame(payload_protocol::HostDirection::HostToSupervisor, &ready,)
        .is_err());
    assert_eq!(
        wrong_direction.accept_frame(payload_protocol::HostDirection::SupervisorToHost, &ready,),
        Err(payload_protocol::ProtocolError::Poisoned)
    );

    let first_key = expected_measurement_at(instance, 0).unwrap();
    let first_identity = contract::expected_measurement(entry, 0).unwrap();
    let host_measurement = host_wire(
        &encode_measurement(
            instance,
            campaign,
            1,
            first_key,
            valid_measurement(first_key),
        )
        .unwrap(),
    );
    let nested_measurement = payload_frame(
        binding,
        contract::MessageKind::Measurement,
        1,
        0,
        &payload_protocol::encode_measurement(payload_measurement(first_identity)),
        Some(first_identity),
    );
    assert_eq!(host_measurement, nested_measurement);
    let (mut host_wrong_order, _) =
        C2b2aHostProtocolMachine::begin_for_test(instance, campaign).unwrap();
    let mut nested_wrong_order =
        payload_protocol::HostProtocolV1::new_bound_for_test(entry, selector, binding).unwrap();
    nested_wrong_order
        .accept_frame(
            payload_protocol::HostDirection::HostToSupervisor,
            &host_start,
        )
        .unwrap();
    let measurement_header: &[u8; C2B2A_HOST_HEADER_BYTES] = nested_measurement
        [..C2B2A_HOST_HEADER_BYTES]
        .try_into()
        .unwrap();
    assert!(host_wrong_order
        .accept_supervisor_frame(
            measurement_header,
            &nested_measurement[C2B2A_HOST_HEADER_BYTES..],
        )
        .is_err());
    assert!(nested_wrong_order
        .accept_frame(
            payload_protocol::HostDirection::SupervisorToHost,
            &host_measurement,
        )
        .is_err());
    let ready_header: &[u8; C2B2A_HOST_HEADER_BYTES] =
        ready[..C2B2A_HOST_HEADER_BYTES].try_into().unwrap();
    assert!(host_wrong_order
        .accept_supervisor_frame(ready_header, &ready[C2B2A_HOST_HEADER_BYTES..])
        .is_err());
    assert_eq!(
        nested_wrong_order.accept_frame(payload_protocol::HostDirection::SupervisorToHost, &ready,),
        Err(payload_protocol::ProtocolError::Poisoned)
    );

    let (host_early_eof, _) = C2b2aHostProtocolMachine::begin_for_test(instance, campaign).unwrap();
    assert!(host_early_eof.finish_on_supervisor_eof().is_err());
    let mut nested_early_eof =
        payload_protocol::HostProtocolV1::new_bound_for_test(entry, selector, binding).unwrap();
    nested_early_eof
        .accept_frame(
            payload_protocol::HostDirection::HostToSupervisor,
            &host_start,
        )
        .unwrap();
    assert!(nested_early_eof
        .accept_eof(payload_protocol::HostDirection::SupervisorToHost)
        .is_err());
    assert_eq!(
        nested_early_eof.accept_frame(payload_protocol::HostDirection::SupervisorToHost, &ready,),
        Err(payload_protocol::ProtocolError::Poisoned)
    );

    let terminal_instance = C2b2aInstanceName::B40;
    let terminal_campaign = parity_campaign(9);
    let terminal_binding = payload_binding(terminal_instance, terminal_campaign);
    let terminal_entry = payload_entry(terminal_instance);
    let terminal_selector = payload_selector(terminal_entry);
    let terminal_start =
        host_wire(&encode_host_start(terminal_instance, terminal_campaign).unwrap());
    let terminal_ready =
        host_wire(&encode_host_ready(terminal_instance, terminal_campaign).unwrap());
    let (mut host_complete, _) =
        C2b2aHostProtocolMachine::begin_for_test(terminal_instance, terminal_campaign).unwrap();
    assert_eq!(
        accept_payload_wire_by_host(&mut host_complete, &terminal_ready),
        C2b2aHostProtocolEvent::HostReady
    );
    assert_eq!(
        host_complete.finish_on_supervisor_eof().unwrap(),
        C2b2aHostProtocolCompletion::HostTerminalEof { probe_id: 40 }
    );
    let mut nested_complete = payload_protocol::HostProtocolV1::new_bound_for_test(
        terminal_entry,
        terminal_selector,
        terminal_binding,
    )
    .unwrap();
    nested_complete
        .accept_frame(
            payload_protocol::HostDirection::HostToSupervisor,
            &terminal_start,
        )
        .unwrap();
    nested_complete
        .accept_frame(
            payload_protocol::HostDirection::SupervisorToHost,
            &terminal_ready,
        )
        .unwrap();
    nested_complete
        .accept_eof(payload_protocol::HostDirection::SupervisorToHost)
        .unwrap();
    assert!(nested_complete.is_complete());
    assert_eq!(
        nested_complete.accept_eof(payload_protocol::HostDirection::SupervisorToHost),
        Err(payload_protocol::ProtocolError::LateEvent)
    );
    assert!(!nested_complete.is_complete());
}

#[test]
fn accepted_freeze_and_protected_inputs_have_exact_identities() {
    assert_eq!(sha256_hex(FREEZE), ACCEPTED_C2B2A_FREEZE_SHA256);
    assert_eq!(std::str::from_utf8(FREEZE).unwrap().lines().count(), 1451);
    assert_eq!(
        sha256_hex(C2B2_RESEARCH),
        "00735349dd10c8c18414a4e611fc5030a82ae014c9ac95bfd9872b3f9427ece0"
    );
    assert_eq!(
        sha256_hex(C2B2_RESEARCH_REVIEW),
        "f68400713ceef924d2e0f24d642ffa77b61bfc3dd15c7a1ad010a60970ba07cd"
    );
    assert_eq!(
        sha256_hex(ACCEPTED_C2B1_REPORT),
        "75b88affe568d1ab70686f0d58050e3f05ba213c7658ab981b67aeb81d1e82ef"
    );
    assert_eq!(
        sha256_hex(C2B1_FREEZE),
        "04f1be24c758f4a7f52baafab2d9ff27b9fd59241eecc48545deb3e5365600cb"
    );
    assert_eq!(
        sha256_hex(C2B1_ACQUISITION_SOURCE),
        "bf10b94a3c2ba8f1f5a55567c0447ede83e2e412e9149a25f6d0a49c3217bde0"
    );
    assert_eq!(
        sha256_hex(NATIVE_VM_SOURCE),
        "7236583a702a85998e1be5a4c32950df239d32a384d4bf82e2b9dc961c90b69a"
    );
    assert_eq!(
        sha256_hex(NATIVE_VM_FOUNDATION_RECORD),
        "5ad40650a92cde03a35db6017aaa2cf72c1d7deb6dea14b4189e531dc6649981"
    );
    assert_eq!(
        sha256_hex(EVALUATOR_MANIFEST),
        "4176c6a4886492f7d5adaa0bdc3134d7e04bfc8d1536de4d5eb8d4b8985ce2bb"
    );
    assert_eq!(
        sha256_hex(WORKSPACE_MANIFEST),
        "ec61531f8ffd401211f649c7b91e28a4ee5f0fa1a0fe2f199c63a841c9724f5a"
    );
    assert_eq!(
        sha256_hex(WORKSPACE_LOCK),
        "0eb924bb9008bb417cb9990081df8429520131aa6b3f7367866c5b805ec474de"
    );
}

#[test]
fn compiled_schedule_is_closed_bijective_and_exact() {
    validate_compiled_foundation().unwrap();
    let expected = [
        (C2b2aInstanceName::C01, 1, 1, 1, 21, false),
        (C2b2aInstanceName::P01, 2, 101, 1, 140, true),
        (C2b2aInstanceName::P02, 2, 102, 2, 140, true),
        (C2b2aInstanceName::P03, 2, 103, 3, 140, true),
        (C2b2aInstanceName::B01, 2, 201, 4, 60, true),
        (C2b2aInstanceName::B12O, 2, 212, 5, 1, true),
        (C2b2aInstanceName::B12P, 2, 213, 6, 1, true),
        (C2b2aInstanceName::B15, 2, 215, 7, 1, true),
        (C2b2aInstanceName::B16, 2, 216, 8, 1, true),
        (C2b2aInstanceName::B40, 2, 240, 9, 0, true),
        (C2b2aInstanceName::B41, 2, 241, 10, 0, true),
    ];

    for (name, phase, class, ordinal, count, acceptance) in expected {
        let spec = instance_spec(name);
        assert_eq!(spec.name.label(), name.label());
        assert_eq!(spec.selector.phase_code, phase);
        assert_eq!(spec.selector.instance_class_code, class);
        assert_eq!(spec.selector.instance_ordinal, ordinal);
        assert_eq!(spec.measurement_count, count);
        assert_eq!(
            spec.authority == C2b2aEvidenceAuthority::Acceptance,
            acceptance
        );
        assert_eq!(instance_for_selector(spec.selector).unwrap(), spec);

        for illegal in [
            C2b2aInstanceSelector {
                phase_code: phase + 1,
                ..spec.selector
            },
            C2b2aInstanceSelector {
                instance_class_code: class + 1,
                ..spec.selector
            },
            C2b2aInstanceSelector {
                instance_ordinal: ordinal + 1,
                ..spec.selector
            },
            C2b2aInstanceSelector {
                phase_code: 0,
                ..spec.selector
            },
        ] {
            assert!(instance_for_selector(illegal).is_err());
        }
    }
}

#[test]
fn measurement_schedule_has_exact_order_and_cardinality() {
    let challenged = [
        C2b2aInstanceName::C01,
        C2b2aInstanceName::P01,
        C2b2aInstanceName::P02,
        C2b2aInstanceName::P03,
        C2b2aInstanceName::B01,
        C2b2aInstanceName::B12O,
        C2b2aInstanceName::B12P,
        C2b2aInstanceName::B15,
        C2b2aInstanceName::B16,
    ];
    for instance in challenged {
        let spec = instance_spec(instance);
        let keys: Vec<_> = (0..usize::try_from(spec.measurement_count).unwrap())
            .map(|index| expected_measurement_at(instance, index).unwrap())
            .collect();
        assert_eq!(keys.len(), usize::try_from(spec.measurement_count).unwrap());
        assert!(expected_measurement_at(instance, keys.len()).is_none());
    }

    let c01: Vec<_> = (0..21)
        .map(|index| expected_measurement_at(C2b2aInstanceName::C01, index).unwrap())
        .collect();
    assert_eq!(
        (c01[0].case_or_probe_id, c01[0].fixture_or_subattempt_id),
        (1, 1)
    );
    assert_eq!(
        (c01[7].case_or_probe_id, c01[7].fixture_or_subattempt_id),
        (1, 2)
    );
    assert_eq!(
        (c01[20].scope_code, c01[20].role_id),
        (7, C2b2aRoleId::JourneyAggregate)
    );

    let positive: Vec<_> = (0..140)
        .map(|index| expected_measurement_at(C2b2aInstanceName::P01, index).unwrap())
        .collect();
    assert_eq!(
        (positive[0].case_or_probe_id, positive[0].scope_code),
        (1, 1)
    );
    assert_eq!(
        (positive[139].case_or_probe_id, positive[139].scope_code),
        (20, 7)
    );
    assert_eq!(positive[1].role_id, C2b2aRoleId::Contender);
    assert_eq!(positive[4].role_id, C2b2aRoleId::Release);

    let b01: Vec<_> = (0..60)
        .map(|index| expected_measurement_at(C2b2aInstanceName::B01, index).unwrap())
        .collect();
    assert_eq!(b01.len(), 60);
    assert!(b01
        .iter()
        .all(|key| !matches!(key.case_or_probe_id, 12 | 15 | 16)));
    let unique: BTreeSet<_> = b01
        .iter()
        .map(|key| (key.case_or_probe_id, key.fixture_or_subattempt_id))
        .collect();
    assert_eq!(unique.len(), 60);
    for (probe, count) in [(32, 2), (33, 14), (35, 8), (36, 4)] {
        assert_eq!(
            b01.iter()
                .filter(|key| key.case_or_probe_id == probe)
                .count(),
            count
        );
    }
}

#[test]
fn freshness_ledger_rejects_every_cross_kind_reuse_and_has_fixed_capacity() {
    let mut duplicate_nonce = C2b2aCampaignFreshness::new();
    duplicate_nonce.register_nonce(nonce(1)).unwrap();
    assert!(duplicate_nonce.register_nonce(nonce(1)).is_err());
    assert!(duplicate_nonce.register_nonce(nonce(2)).is_err());

    let mut cross_kind = C2b2aCampaignFreshness::new();
    cross_kind.register_nonce(nonce(1)).unwrap();
    assert!(cross_kind.register_challenge(challenge(1)).is_err());
    assert!(cross_kind.register_challenge(challenge(2)).is_err());

    let mut ledger = C2b2aCampaignFreshness::new();
    assert_eq!(ledger.value_count(), 0);

    for byte in 1..=11 {
        ledger.register_nonce(nonce(byte)).unwrap();
    }
    for byte in 12..=20 {
        ledger.register_challenge(challenge(byte)).unwrap();
    }
    assert_eq!(ledger.nonce_count(), C2B2A_CAMPAIGN_NONCE_COUNT);
    assert_eq!(ledger.challenge_count(), C2B2A_CAMPAIGN_CHALLENGE_COUNT);
    assert_eq!(ledger.value_count(), C2B2A_CAMPAIGN_FRESH_VALUE_COUNT);
    assert!(ledger.is_complete());
    assert!(ledger.register_nonce(nonce(21)).is_err());
    assert!(ledger.register_challenge(challenge(21)).is_err());
}

#[test]
fn campaign_controller_enforces_exact_serial_order_and_campaign_wide_freshness() {
    let mut controller = C2b2aCampaignController::new();
    assert_eq!(
        controller.next_expected_instance(),
        Some(C2b2aInstanceName::C01)
    );

    for (index, expected_instance) in C2B2A_CAMPAIGN_ORDER.into_iter().enumerate() {
        let nonce_byte = u8::try_from(index + 1).unwrap();
        let campaign = bindings(nonce_byte);
        let (instance, start) = controller.begin_next(campaign).unwrap();
        assert_eq!(instance, expected_instance);
        assert_eq!(controller.active_instance(), Some(instance));
        assert_eq!(start.header()[12..16], [0, 0, 0, 1]);

        let ready = encode_host_ready(instance, campaign).unwrap();
        assert_eq!(
            controller
                .accept_supervisor_frame(ready.header(), ready.payload())
                .unwrap(),
            C2b2aHostProtocolEvent::HostReady
        );

        let spec = instance_spec(instance);
        if spec.terminal_mode == C2b2aTerminalMode::Challenged {
            for measurement_index in 0..usize::try_from(spec.measurement_count).unwrap() {
                let key = expected_measurement_at(instance, measurement_index).unwrap();
                let sequence = u32::try_from(measurement_index).unwrap() + 2;
                let frame =
                    encode_measurement(instance, campaign, sequence, key, valid_measurement(key))
                        .unwrap();
                controller
                    .accept_supervisor_frame(frame.header(), frame.payload())
                    .unwrap();
            }
            let state = C2b2aTerminalState::success_for(instance).unwrap();
            let terminal_ready = encode_terminal_ready(
                instance,
                campaign,
                spec.terminal_ready_sequence.unwrap(),
                state,
            )
            .unwrap();
            controller
                .accept_supervisor_frame(terminal_ready.header(), terminal_ready.payload())
                .unwrap();
            let fresh_challenge = challenge(u8::try_from(index).unwrap() + 101);
            controller.issue_challenge(fresh_challenge).unwrap();
            controller.accept_active_host_to_supervisor_eof().unwrap();
            let terminal = encode_terminal(
                instance,
                campaign,
                spec.terminal_sequence.unwrap(),
                fresh_challenge,
                state,
            )
            .unwrap();
            controller
                .accept_supervisor_frame(terminal.header(), terminal.payload())
                .unwrap();
        }
        controller.finish_active_on_supervisor_eof().unwrap();
        assert_eq!(controller.completed_instance_count(), index);
        assert_eq!(controller.next_expected_instance(), None);
        controller.attest_runtime_closure_for_test(true).unwrap();
        assert_eq!(controller.completed_instance_count(), index + 1);
    }

    assert!(controller.is_complete());
    assert_eq!(controller.next_expected_instance(), None);
    assert!(controller.begin_next(bindings(99)).is_err());

    let mut overlapping = C2b2aCampaignController::new();
    overlapping.begin_next(bindings(1)).unwrap();
    assert!(overlapping.begin_next(bindings(2)).is_err());
    assert!(overlapping.begin_next(bindings(3)).is_err());
}

#[test]
fn campaign_controller_cannot_advance_without_runtime_closure() {
    let campaign = bindings(1);
    let failure = C2b2aTerminalState {
        primary_category: C2b2aPrimaryCategory::ProtocolFailure,
        active_item_code: 0,
        guest_cleanup_proven: false,
        write_may_have_been_dispatched: false,
        writer_acknowledged: false,
        roles_reaped: false,
        completed_case_count: 0,
        completed_subattempt_count: 0,
    };

    let mut controller = C2b2aCampaignController::new();
    controller.begin_next(campaign).unwrap();
    let ready = encode_host_ready(C2b2aInstanceName::C01, campaign).unwrap();
    controller
        .accept_supervisor_frame(ready.header(), ready.payload())
        .unwrap();
    let abort = encode_supervisor_abort(C2b2aInstanceName::C01, campaign, 2, failure).unwrap();
    controller
        .accept_supervisor_frame(abort.header(), abort.payload())
        .unwrap();
    assert_eq!(
        controller.finish_active_on_supervisor_eof().unwrap(),
        C2b2aHostProtocolCompletion::UnchallengedFailure {
            state: failure,
            measurements_seen: 0,
        }
    );
    assert_eq!(controller.completed_instance_count(), 0);
    assert_eq!(controller.next_expected_instance(), None);
    controller.attest_runtime_closure_for_test(true).unwrap();
    assert_eq!(controller.completed_instance_count(), 0);
    assert!(!controller.is_complete());
    assert!(controller.begin_next(bindings(2)).is_err());

    let mut unproven = C2b2aCampaignController::new();
    unproven.begin_next(campaign).unwrap();
    let ready = encode_host_ready(C2b2aInstanceName::C01, campaign).unwrap();
    unproven
        .accept_supervisor_frame(ready.header(), ready.payload())
        .unwrap();
    let abort = encode_supervisor_abort(C2b2aInstanceName::C01, campaign, 2, failure).unwrap();
    unproven
        .accept_supervisor_frame(abort.header(), abort.payload())
        .unwrap();
    unproven.finish_active_on_supervisor_eof().unwrap();
    assert!(unproven.attest_runtime_closure_for_test(false).is_err());
    assert!(unproven.attest_runtime_closure_for_test(true).is_err());
}

#[test]
fn terminal_failure_item_state_is_bound_to_the_protocol_phase() {
    let campaign = bindings(1);
    let instance = C2b2aInstanceName::P01;
    let base_failure = C2b2aTerminalState {
        primary_category: C2b2aPrimaryCategory::ProtocolFailure,
        active_item_code: 0,
        guest_cleanup_proven: false,
        write_may_have_been_dispatched: false,
        writer_acknowledged: false,
        roles_reaped: false,
        completed_case_count: 0,
        completed_subattempt_count: 0,
    };

    let (mut pre_ready_with_item, _) =
        C2b2aHostProtocolMachine::begin_for_test(instance, campaign).unwrap();
    let pre_ready_item = C2b2aTerminalState {
        active_item_code: C2b2aTerminalState::active_item(1, 1).unwrap(),
        ..base_failure
    };
    let abort = encode_supervisor_abort(instance, campaign, 1, pre_ready_item).unwrap();
    assert!(pre_ready_with_item
        .accept_supervisor_frame(abort.header(), abort.payload())
        .is_err());

    let (mut pre_ready_with_milestone, _) =
        C2b2aHostProtocolMachine::begin_for_test(instance, campaign).unwrap();
    let pre_ready_milestone = C2b2aTerminalState {
        write_may_have_been_dispatched: true,
        ..base_failure
    };
    let abort = encode_supervisor_abort(instance, campaign, 1, pre_ready_milestone).unwrap();
    assert!(pre_ready_with_milestone
        .accept_supervisor_frame(abort.header(), abort.payload())
        .is_err());

    let mut no_active_after_ready = enter_collecting(instance, campaign);
    let terminal_ready = encode_terminal_ready(instance, campaign, 2, base_failure).unwrap();
    assert!(no_active_after_ready
        .accept_supervisor_frame(terminal_ready.header(), terminal_ready.payload())
        .is_err());

    let mut abort_before_first_item = enter_collecting(instance, campaign);
    let cleaned_before_first_item = C2b2aTerminalState {
        guest_cleanup_proven: true,
        roles_reaped: true,
        ..base_failure
    };
    let abort = encode_supervisor_abort(instance, campaign, 2, cleaned_before_first_item).unwrap();
    assert_eq!(
        accept_frame(&mut abort_before_first_item, &abort),
        C2b2aHostProtocolEvent::AbortAccepted(cleaned_before_first_item)
    );
    assert_eq!(
        abort_before_first_item.finish_on_supervisor_eof().unwrap(),
        C2b2aHostProtocolCompletion::UnchallengedFailure {
            state: cleaned_before_first_item,
            measurements_seen: 0,
        }
    );

    let mut active_first = enter_collecting(instance, campaign);
    let first_item_failure = C2b2aTerminalState {
        active_item_code: C2b2aTerminalState::active_item(1, 1).unwrap(),
        ..base_failure
    };
    let terminal_ready = encode_terminal_ready(instance, campaign, 2, first_item_failure).unwrap();
    assert_eq!(
        accept_frame(&mut active_first, &terminal_ready),
        C2b2aHostProtocolEvent::ChallengeRequired(first_item_failure)
    );

    let mut no_active_after_completed_item = enter_collecting(instance, campaign);
    for index in 0..7 {
        let key = expected_measurement_at(instance, index).unwrap();
        let frame = encode_measurement(
            instance,
            campaign,
            u32::try_from(index).unwrap() + 2,
            key,
            valid_measurement(key),
        )
        .unwrap();
        accept_frame(&mut no_active_after_completed_item, &frame);
    }
    let missing_second_item = C2b2aTerminalState {
        completed_case_count: 1,
        ..base_failure
    };
    let terminal_ready = encode_terminal_ready(instance, campaign, 9, missing_second_item).unwrap();
    assert!(no_active_after_completed_item
        .accept_supervisor_frame(terminal_ready.header(), terminal_ready.payload())
        .is_err());

    let mut active_first_after_measurements = enter_collecting(instance, campaign);
    for index in 0..7 {
        let key = expected_measurement_at(instance, index).unwrap();
        let frame = encode_measurement(
            instance,
            campaign,
            u32::try_from(index).unwrap() + 2,
            key,
            valid_measurement(key),
        )
        .unwrap();
        accept_frame(&mut active_first_after_measurements, &frame);
    }
    let current_first_item = C2b2aTerminalState {
        active_item_code: C2b2aTerminalState::active_item(1, 1).unwrap(),
        ..base_failure
    };
    let terminal_ready = encode_terminal_ready(instance, campaign, 9, current_first_item).unwrap();
    assert_eq!(
        accept_frame(&mut active_first_after_measurements, &terminal_ready),
        C2b2aHostProtocolEvent::ChallengeRequired(current_first_item)
    );

    let mut active_second_after_completed_item = enter_collecting(instance, campaign);
    for index in 0..7 {
        let key = expected_measurement_at(instance, index).unwrap();
        let frame = encode_measurement(
            instance,
            campaign,
            u32::try_from(index).unwrap() + 2,
            key,
            valid_measurement(key),
        )
        .unwrap();
        accept_frame(&mut active_second_after_completed_item, &frame);
    }
    let second_item_failure = C2b2aTerminalState {
        active_item_code: C2b2aTerminalState::active_item(2, 2).unwrap(),
        completed_case_count: 1,
        ..base_failure
    };
    let terminal_ready = encode_terminal_ready(instance, campaign, 9, second_item_failure).unwrap();
    assert_eq!(
        accept_frame(&mut active_second_after_completed_item, &terminal_ready),
        C2b2aHostProtocolEvent::ChallengeRequired(second_item_failure)
    );
}

#[test]
fn host_start_and_measurement_wire_are_fixed_big_endian() {
    let campaign = bindings(0x11);
    let start = encode_host_start(C2b2aInstanceName::P02, campaign).unwrap();
    assert_eq!(start.payload_len(), 8);
    assert_eq!(start.wire_len(), 168);
    assert_eq!(&start.header()[0..8], b"ENGC2A01");
    assert_eq!(&start.header()[8..10], &[0, 1]);
    assert_eq!(&start.header()[10..12], &[0, 19]);
    assert_eq!(&start.header()[12..16], &[0, 0, 0, 1]);
    assert_eq!(&start.header()[20..22], &[0, 1]);
    assert_eq!(&start.header()[22..28], &[0, 0, 0, 0, 0, 0]);
    assert_eq!(&start.header()[28..32], &[0, 0, 0, 8]);
    assert_eq!(&start.header()[32..64], &[0x11; 32]);
    assert_eq!(start.header()[64..96], *C2B2A_CONTRACT_SHA256.as_bytes());
    assert_eq!(&start.header()[96..128], &[0x44; 32]);
    assert_eq!(start.payload(), &[0, 2, 0, 102, 0, 2, 0, 0]);
    assert_eq!(&start.header()[128..160], &sha256_bytes(start.payload()));
    let decoded = decode_host_control_frame(
        C2b2aDirection::HostToSupervisor,
        start.header(),
        start.payload(),
    )
    .unwrap();
    assert_eq!(decoded.header.kind, C2b2aHostKind::HostStart);
    assert_eq!(
        decoded.payload,
        C2b2aDecodedPayload::HostStart(instance_spec(C2b2aInstanceName::P02).selector)
    );

    let fields = std::array::from_fn(|index| u64::try_from(index + 1).unwrap());
    let measurement = C2b2aMeasurement::from_fields(fields);
    let encoded = measurement.encode();
    for (index, value) in fields.iter().enumerate() {
        assert_eq!(&encoded[index * 8..index * 8 + 8], &value.to_be_bytes());
    }
    assert_eq!(C2b2aMeasurement::decode(&encoded).unwrap(), measurement);

    let mut wire = [0xff_u8; C2B2A_MAX_HOST_FRAME_BYTES];
    assert_eq!(start.copy_wire_into(&mut wire), 168);
    assert_eq!(&wire[..160], start.header());
    assert_eq!(&wire[160..168], start.payload());
    assert!(wire[168..].iter().all(|byte| *byte == 0));
}

#[test]
fn strict_decoder_rejects_header_payload_and_direction_mutations() {
    let campaign = bindings(0x11);
    let start = encode_host_start(C2b2aInstanceName::P01, campaign).unwrap();

    for (offset, value) in [(0, b'X'), (9, 2), (11, 99), (15, 0), (21, 2), (27, 1)] {
        let mut header = *start.header();
        header[offset] = value;
        assert!(decode_host_control_frame(
            C2b2aDirection::HostToSupervisor,
            &header,
            start.payload()
        )
        .is_err());
    }
    assert!(decode_host_control_frame(
        C2b2aDirection::SupervisorToHost,
        start.header(),
        start.payload()
    )
    .is_err());
    assert!(decode_host_control_frame(
        C2b2aDirection::HostToSupervisor,
        start.header(),
        &start.payload()[..7]
    )
    .is_err());
    let mut trailing = start.payload().to_vec();
    trailing.push(0);
    assert!(
        decode_host_control_frame(C2b2aDirection::HostToSupervisor, start.header(), &trailing)
            .is_err()
    );

    let mut bad_digest = *start.header();
    bad_digest[128] ^= 1;
    assert!(decode_host_control_frame(
        C2b2aDirection::HostToSupervisor,
        &bad_digest,
        start.payload()
    )
    .is_err());

    let mut reserved = start.payload().to_vec();
    reserved[7] = 1;
    let mut reserved_header = *start.header();
    reseal_frame_payload(&mut reserved_header, &reserved);
    assert!(decode_host_control_frame(
        C2b2aDirection::HostToSupervisor,
        &reserved_header,
        &reserved
    )
    .is_err());

    let mut unknown_selector = start.payload().to_vec();
    put_u16(&mut unknown_selector, 2, 104);
    let mut selector_header = *start.header();
    reseal_frame_payload(&mut selector_header, &unknown_selector);
    assert!(decode_host_control_frame(
        C2b2aDirection::HostToSupervisor,
        &selector_header,
        &unknown_selector
    )
    .is_err());
}

#[test]
fn measurement_validation_distinguishes_positive_case_ids_from_boundary_probe_ids() {
    let campaign = bindings(0x11);
    let mut positive = enter_collecting(C2b2aInstanceName::P01, campaign);
    for index in 0..126 {
        let key = expected_measurement_at(C2b2aInstanceName::P01, index).unwrap();
        let frame = encode_measurement(
            C2b2aInstanceName::P01,
            campaign,
            u32::try_from(index).unwrap() + 2,
            key,
            valid_measurement(key),
        )
        .unwrap();
        accept_frame(&mut positive, &frame);
    }
    assert_eq!(positive.measurements_seen(), 126);

    let probe_17_index = (0..60)
        .find(|index| {
            expected_measurement_at(C2b2aInstanceName::B01, *index)
                .unwrap()
                .case_or_probe_id
                == 17
        })
        .unwrap();
    let mut missing_probe_17_bit = enter_collecting(C2b2aInstanceName::B01, campaign);
    for index in 0..probe_17_index {
        let key = expected_measurement_at(C2b2aInstanceName::B01, index).unwrap();
        let frame = encode_measurement(
            C2b2aInstanceName::B01,
            campaign,
            u32::try_from(index).unwrap() + 2,
            key,
            valid_measurement(key),
        )
        .unwrap();
        accept_frame(&mut missing_probe_17_bit, &frame);
    }
    let probe_17_key = expected_measurement_at(C2b2aInstanceName::B01, probe_17_index).unwrap();
    let missing_bit = encode_measurement(
        C2b2aInstanceName::B01,
        campaign,
        u32::try_from(probe_17_index).unwrap() + 2,
        probe_17_key,
        C2b2aMeasurement {
            scope_code: probe_17_key.scope_code,
            monotonic_elapsed_ns: 1,
            ..C2b2aMeasurement::default()
        },
    )
    .unwrap();
    assert!(missing_probe_17_bit
        .accept_supervisor_frame(missing_bit.header(), missing_bit.payload())
        .is_err());

    let mut boundary = enter_collecting(C2b2aInstanceName::B01, campaign);
    let probe_18_index = (0..60)
        .find(|index| {
            expected_measurement_at(C2b2aInstanceName::B01, *index)
                .unwrap()
                .case_or_probe_id
                == 18
        })
        .unwrap();
    for index in 0..=probe_18_index {
        let key = expected_measurement_at(C2b2aInstanceName::B01, index).unwrap();
        let frame = encode_measurement(
            C2b2aInstanceName::B01,
            campaign,
            u32::try_from(index).unwrap() + 2,
            key,
            valid_measurement(key),
        )
        .unwrap();
        accept_frame(&mut boundary, &frame);
    }

    for instance in [C2b2aInstanceName::B15, C2b2aInstanceName::B16] {
        let mut machine = enter_collecting(instance, campaign);
        let key = expected_measurement_at(instance, 0).unwrap();
        let mut invalid = valid_measurement(key);
        invalid.memory_current_bytes = 1;
        let frame = encode_measurement(instance, campaign, 2, key, invalid).unwrap();
        assert!(machine
            .accept_supervisor_frame(frame.header(), frame.payload())
            .is_err());
    }
}

#[test]
fn every_challenged_success_uses_exact_cardinality_challenge_terminal_and_eof() {
    let instances = [
        C2b2aInstanceName::C01,
        C2b2aInstanceName::P01,
        C2b2aInstanceName::P02,
        C2b2aInstanceName::P03,
        C2b2aInstanceName::B01,
        C2b2aInstanceName::B12O,
        C2b2aInstanceName::B12P,
        C2b2aInstanceName::B15,
        C2b2aInstanceName::B16,
    ];
    for (instance_index, instance) in instances.into_iter().enumerate() {
        let campaign = bindings(u8::try_from(instance_index).unwrap() + 1);
        let mut machine = enter_collecting(instance, campaign);
        accept_all_measurements(&mut machine, instance, campaign);
        let state = C2b2aTerminalState::success_for(instance).unwrap();
        let spec = instance_spec(instance);
        let ready = encode_terminal_ready(
            instance,
            campaign,
            spec.terminal_ready_sequence.unwrap(),
            state,
        )
        .unwrap();
        assert_eq!(
            accept_frame(&mut machine, &ready),
            C2b2aHostProtocolEvent::ChallengeRequired(state)
        );
        let challenge = challenge(u8::try_from(instance_index).unwrap() + 32);
        let host_frame = machine
            .issue_challenge_for_test(campaign, challenge)
            .unwrap();
        assert_eq!(host_frame.header()[10..12], [0, 22]);
        assert_eq!(host_frame.payload(), challenge.as_bytes());
        machine.accept_host_to_supervisor_eof().unwrap();
        let terminal = encode_terminal(
            instance,
            campaign,
            spec.terminal_sequence.unwrap(),
            challenge,
            state,
        )
        .unwrap();
        assert_eq!(
            accept_frame(&mut machine, &terminal),
            C2b2aHostProtocolEvent::TerminalAccepted(state)
        );
        assert_eq!(
            machine.finish_on_supervisor_eof().unwrap(),
            C2b2aHostProtocolCompletion::ChallengedTerminal {
                state,
                measurements_seen: spec.measurement_count,
            }
        );
    }
}

#[test]
fn state_machine_rejects_wrong_order_sequence_binding_state_and_retry() {
    let campaign = bindings(1);

    let (mut wrong_sequence, _) =
        C2b2aHostProtocolMachine::begin_for_test(C2b2aInstanceName::P01, campaign).unwrap();
    let key = expected_measurement_at(C2b2aInstanceName::P01, 0).unwrap();
    let measurement = encode_measurement(
        C2b2aInstanceName::P01,
        campaign,
        2,
        key,
        valid_measurement(key),
    )
    .unwrap();
    assert!(wrong_sequence
        .accept_supervisor_frame(measurement.header(), measurement.payload())
        .is_err());
    let ready = encode_host_ready(C2b2aInstanceName::P01, campaign).unwrap();
    assert!(wrong_sequence
        .accept_supervisor_frame(ready.header(), ready.payload())
        .is_err());

    let (mut wrong_selector, _) =
        C2b2aHostProtocolMachine::begin_for_test(C2b2aInstanceName::P01, campaign).unwrap();
    let other_ready = encode_host_ready(C2b2aInstanceName::P02, campaign).unwrap();
    assert!(wrong_selector
        .accept_supervisor_frame(other_ready.header(), other_ready.payload())
        .is_err());

    let mut wrong_key = enter_collecting(C2b2aInstanceName::P01, campaign);
    let second_key = expected_measurement_at(C2b2aInstanceName::P01, 1).unwrap();
    let second = encode_measurement(
        C2b2aInstanceName::P01,
        campaign,
        2,
        second_key,
        valid_measurement(second_key),
    )
    .unwrap();
    assert!(wrong_key
        .accept_supervisor_frame(second.header(), second.payload())
        .is_err());

    let mut wrong_binding = enter_collecting(C2b2aInstanceName::P01, campaign);
    let altered = bindings(2);
    let first = encode_measurement(
        C2b2aInstanceName::P01,
        altered,
        2,
        key,
        valid_measurement(key),
    )
    .unwrap();
    assert!(wrong_binding
        .accept_supervisor_frame(first.header(), first.payload())
        .is_err());

    let mut early_success = enter_collecting(C2b2aInstanceName::P01, campaign);
    let success = C2b2aTerminalState::success_for(C2b2aInstanceName::P01).unwrap();
    let terminal_ready =
        encode_terminal_ready(C2b2aInstanceName::P01, campaign, 2, success).unwrap();
    assert!(early_success
        .accept_supervisor_frame(terminal_ready.header(), terminal_ready.payload())
        .is_err());

    let mut early_challenge = enter_collecting(C2b2aInstanceName::P01, campaign);
    assert!(early_challenge
        .issue_challenge_for_test(campaign, challenge(9))
        .is_err());

    let mut skipped_item = enter_collecting(C2b2aInstanceName::P01, campaign);
    for index in 0..7 {
        let current = expected_measurement_at(C2b2aInstanceName::P01, index).unwrap();
        let frame = encode_measurement(
            C2b2aInstanceName::P01,
            campaign,
            u32::try_from(index).unwrap() + 2,
            current,
            valid_measurement(current),
        )
        .unwrap();
        accept_frame(&mut skipped_item, &frame);
    }
    let skipped_state = C2b2aTerminalState {
        primary_category: C2b2aPrimaryCategory::ProtocolFailure,
        active_item_code: C2b2aTerminalState::active_item(3, 3).unwrap(),
        guest_cleanup_proven: true,
        write_may_have_been_dispatched: false,
        writer_acknowledged: false,
        roles_reaped: true,
        completed_case_count: 1,
        completed_subattempt_count: 0,
    };
    let skipped_terminal =
        encode_terminal_ready(C2b2aInstanceName::P01, campaign, 9, skipped_state).unwrap();
    assert!(skipped_item
        .accept_supervisor_frame(skipped_terminal.header(), skipped_terminal.payload())
        .is_err());
}

#[test]
fn structured_failure_supports_challenged_or_unchallenged_terminal_paths() {
    let campaign = bindings(1);
    let instance = C2b2aInstanceName::P01;
    let mut challenged = enter_collecting(instance, campaign);
    for index in 0..7 {
        let key = expected_measurement_at(instance, index).unwrap();
        let frame = encode_measurement(
            instance,
            campaign,
            u32::try_from(index).unwrap() + 2,
            key,
            valid_measurement(key),
        )
        .unwrap();
        accept_frame(&mut challenged, &frame);
    }
    let state = C2b2aTerminalState {
        primary_category: C2b2aPrimaryCategory::ProtocolFailure,
        active_item_code: C2b2aTerminalState::active_item(2, 2).unwrap(),
        guest_cleanup_proven: true,
        write_may_have_been_dispatched: false,
        writer_acknowledged: false,
        roles_reaped: true,
        completed_case_count: 1,
        completed_subattempt_count: 0,
    };
    let terminal_ready = encode_terminal_ready(instance, campaign, 9, state).unwrap();
    assert_eq!(
        accept_frame(&mut challenged, &terminal_ready),
        C2b2aHostProtocolEvent::ChallengeRequired(state)
    );
    let challenge = challenge(7);
    challenged
        .issue_challenge_for_test(campaign, challenge)
        .unwrap();
    challenged.accept_host_to_supervisor_eof().unwrap();
    let terminal = encode_terminal(instance, campaign, 10, challenge, state).unwrap();
    accept_frame(&mut challenged, &terminal);
    assert_eq!(
        challenged.finish_on_supervisor_eof().unwrap(),
        C2b2aHostProtocolCompletion::ChallengedTerminal {
            state,
            measurements_seen: 7,
        }
    );

    let before_item = C2b2aTerminalState {
        active_item_code: 0,
        completed_case_count: 0,
        ..state
    };
    let (mut unchallenged, _) =
        C2b2aHostProtocolMachine::begin_for_test(instance, campaign).unwrap();
    let abort = encode_supervisor_abort(instance, campaign, 1, before_item).unwrap();
    assert_eq!(
        accept_frame(&mut unchallenged, &abort),
        C2b2aHostProtocolEvent::AbortAccepted(before_item)
    );
    assert_eq!(
        unchallenged.finish_on_supervisor_eof().unwrap(),
        C2b2aHostProtocolCompletion::UnchallengedFailure {
            state: before_item,
            measurements_seen: 0,
        }
    );
    assert!(encode_supervisor_abort(
        instance,
        campaign,
        1,
        C2b2aTerminalState::success_for(instance).unwrap()
    )
    .is_err());
}

#[test]
fn challenge_must_follow_terminal_ready_and_cannot_equal_nonce_or_change_bindings() {
    let campaign = bindings(9);
    let instance = C2b2aInstanceName::B12O;
    let mut equal_nonce = enter_collecting(instance, campaign);
    accept_all_measurements(&mut equal_nonce, instance, campaign);
    let state = C2b2aTerminalState::success_for(instance).unwrap();
    let ready = encode_terminal_ready(instance, campaign, 3, state).unwrap();
    accept_frame(&mut equal_nonce, &ready);
    assert!(equal_nonce
        .issue_challenge_for_test(campaign, challenge(9))
        .is_err());

    let mut changed_binding = enter_collecting(instance, campaign);
    accept_all_measurements(&mut changed_binding, instance, campaign);
    accept_frame(&mut changed_binding, &ready);
    assert!(changed_binding
        .issue_challenge_for_test(bindings(10), challenge(11))
        .is_err());
}

#[test]
fn terminal_must_echo_the_exact_committed_challenge_and_state() {
    let campaign = bindings(1);
    let instance = C2b2aInstanceName::B12O;
    let mut machine = enter_collecting(instance, campaign);
    accept_all_measurements(&mut machine, instance, campaign);
    let state = C2b2aTerminalState::success_for(instance).unwrap();
    let ready = encode_terminal_ready(instance, campaign, 3, state).unwrap();
    accept_frame(&mut machine, &ready);
    machine
        .issue_challenge_for_test(campaign, challenge(7))
        .unwrap();
    machine.accept_host_to_supervisor_eof().unwrap();
    let wrong_echo = encode_terminal(instance, campaign, 4, challenge(8), state).unwrap();
    assert!(machine
        .accept_supervisor_frame(wrong_echo.header(), wrong_echo.payload())
        .is_err());
}

#[test]
fn host_terminal_instances_accept_only_ready_then_eof() {
    let campaign = bindings(1);
    for (instance, probe_id) in [(C2b2aInstanceName::B40, 40), (C2b2aInstanceName::B41, 41)] {
        let (mut machine, _) =
            C2b2aHostProtocolMachine::begin_for_test(instance, campaign).unwrap();
        let ready = encode_host_ready(instance, campaign).unwrap();
        accept_frame(&mut machine, &ready);
        assert_eq!(
            machine.finish_on_supervisor_eof().unwrap(),
            C2b2aHostProtocolCompletion::HostTerminalEof { probe_id }
        );

        let (mut rejects_byte, _) =
            C2b2aHostProtocolMachine::begin_for_test(instance, campaign).unwrap();
        accept_frame(&mut rejects_byte, &ready);
        let impossible_state = C2b2aTerminalState {
            primary_category: C2b2aPrimaryCategory::GuestConfiguration,
            active_item_code: 0,
            guest_cleanup_proven: false,
            write_may_have_been_dispatched: false,
            writer_acknowledged: false,
            roles_reaped: false,
            completed_case_count: 0,
            completed_subattempt_count: 0,
        };
        let extra = encode_terminal_ready(instance, campaign, 2, impossible_state).unwrap();
        assert!(rejects_byte
            .accept_supervisor_frame(extra.header(), extra.payload())
            .is_err());
    }
}

fn receipt(
    kind: C2b2aHostReceiptKind,
    probe_id: u16,
    host_cleanup_proven: bool,
    injected_cleanup_fault: bool,
) -> C2b2aHostTerminalReceiptV1 {
    C2b2aHostTerminalReceiptV1 {
        kind,
        probe_id,
        host_cleanup_proven,
        injected_cleanup_fault,
        monotonic_elapsed_ns: 0x0102_0304_0506_0708,
        nonce: nonce(0x11),
        contract_sha256: digest(0x22),
        acceptance_schedule_sha256: digest(0x33),
        host_executable_sha256: digest(0x44),
        instance_identity_record_sha256: digest(0x55),
        disk_identity_record_sha256: digest(0x66),
    }
}

fn receipt_expectations(value: C2b2aHostTerminalReceiptV1) -> C2b2aHostReceiptExpectations {
    C2b2aHostReceiptExpectations {
        kind: value.kind,
        probe_id: value.probe_id,
        nonce: value.nonce,
        contract_sha256: value.contract_sha256,
        acceptance_schedule_sha256: value.acceptance_schedule_sha256,
        host_executable_sha256: value.host_executable_sha256,
        instance_identity_record_sha256: value.instance_identity_record_sha256,
        disk_identity_record_sha256: value.disk_identity_record_sha256,
    }
}

#[test]
fn host_terminal_and_recovery_receipts_have_exact_fixed_layouts() {
    let b40 = receipt(C2b2aHostReceiptKind::Terminal, 40, true, false);
    let b41 = receipt(C2b2aHostReceiptKind::Terminal, 41, false, true);
    let recovery = receipt(C2b2aHostReceiptKind::Recovery, 41, true, true);
    for value in [b40, b41, recovery] {
        let bytes = value.encode().unwrap();
        let expected_magic = match value.kind {
            C2b2aHostReceiptKind::Terminal => b"ENGC2HR1",
            C2b2aHostReceiptKind::Recovery => b"ENGC2RC1",
        };
        assert_eq!(&bytes[0..8], expected_magic);
        assert_eq!(&bytes[8..10], &[0, 1]);
        assert_eq!(&bytes[10..12], &value.probe_id.to_be_bytes());
        assert_eq!(&bytes[12..16], &[0, 0, 0, 2]);
        assert_eq!(
            &bytes[16..20],
            &u32::from(value.host_cleanup_proven).to_be_bytes()
        );
        assert_eq!(
            &bytes[20..24],
            &u32::from(value.injected_cleanup_fault).to_be_bytes()
        );
        assert_eq!(&bytes[24..32], &value.monotonic_elapsed_ns.to_be_bytes());
        assert_eq!(&bytes[32..64], value.nonce.as_bytes());
        assert_eq!(&bytes[224..256], &sha256_bytes(&bytes[..224]));
        assert_eq!(C2b2aHostTerminalReceiptV1::decode(&bytes).unwrap(), value);
        value.validate_against(receipt_expectations(value)).unwrap();
    }
}

#[test]
fn host_terminal_receipt_triplet_binds_b41_recovery_to_original_identity() {
    let b40 = receipt(C2b2aHostReceiptKind::Terminal, 40, true, false);
    let mut b41 = receipt(C2b2aHostReceiptKind::Terminal, 41, false, true);
    b41.nonce = nonce(0x12);
    let mut recovery = b41;
    recovery.kind = C2b2aHostReceiptKind::Recovery;
    recovery.host_cleanup_proven = true;
    recovery.monotonic_elapsed_ns = 99;
    validate_host_terminal_receipt_triplet(b40, b41, recovery).unwrap();

    let mut wrong_identity = recovery;
    wrong_identity.disk_identity_record_sha256 = digest(0x99);
    assert!(validate_host_terminal_receipt_triplet(b40, b41, wrong_identity).is_err());

    let mut reused_nonce = b41;
    reused_nonce.nonce = b40.nonce;
    let mut reused_recovery = recovery;
    reused_recovery.nonce = b40.nonce;
    assert!(validate_host_terminal_receipt_triplet(b40, reused_nonce, reused_recovery).is_err());
}

#[test]
fn receipt_decoder_rejects_corruption_illegal_tuples_and_binding_drift() {
    let value = receipt(C2b2aHostReceiptKind::Terminal, 40, true, false);
    let bytes = value.encode().unwrap();

    for offset in [0, 9, 12, 16, 20, 100, 224] {
        let mut corrupted = bytes;
        corrupted[offset] ^= 1;
        assert!(C2b2aHostTerminalReceiptV1::decode(&corrupted).is_err());
    }

    let mut invalid_boolean = bytes;
    put_u32(&mut invalid_boolean, 16, 2);
    reseal_receipt(&mut invalid_boolean);
    assert!(C2b2aHostTerminalReceiptV1::decode(&invalid_boolean).is_err());

    let mut illegal_tuple = bytes;
    put_u16(&mut illegal_tuple, 10, 41);
    reseal_receipt(&mut illegal_tuple);
    assert!(C2b2aHostTerminalReceiptV1::decode(&illegal_tuple).is_err());

    let mut expected = receipt_expectations(value);
    expected.contract_sha256 = digest(0x99);
    assert!(value.validate_against(expected).is_err());
}

#[test]
fn terminal_state_codec_and_invariants_are_exact() {
    let state = C2b2aTerminalState {
        primary_category: C2b2aPrimaryCategory::OutcomeUncertain,
        active_item_code: C2b2aTerminalState::active_item(7, 9).unwrap(),
        guest_cleanup_proven: true,
        write_may_have_been_dispatched: true,
        writer_acknowledged: false,
        roles_reaped: true,
        completed_case_count: 6,
        completed_subattempt_count: 0,
    };
    let encoded = state.encode();
    assert_eq!(&encoded[0..4], &[0, 0, 0, 8]);
    assert_eq!(&encoded[4..8], &[0, 7, 0, 9]);
    assert_eq!(C2b2aTerminalState::decode(&encoded).unwrap(), state);
    assert_eq!(state.active_item_parts().unwrap(), Some((7, 9)));

    let mut bad_boolean = encoded;
    put_u32(&mut bad_boolean, 8, 2);
    assert!(C2b2aTerminalState::decode(&bad_boolean).is_err());
    let mut impossible_cleanup = encoded;
    put_u32(&mut impossible_cleanup, 20, 0);
    assert!(C2b2aTerminalState::decode(&impossible_cleanup).is_err());
    let mut acknowledgement_ends_uncertain_phase = encoded;
    put_u32(&mut acknowledgement_ends_uncertain_phase, 16, 1);
    assert!(C2b2aTerminalState::decode(&acknowledgement_ends_uncertain_phase).is_err());
    let mut non_outcome_during_uncertain_phase = encoded;
    put_u32(&mut non_outcome_during_uncertain_phase, 0, 1);
    assert!(C2b2aTerminalState::decode(&non_outcome_during_uncertain_phase).is_err());
    let mut outcome_before_dispatch = encoded;
    put_u32(&mut outcome_before_dispatch, 12, 0);
    assert!(C2b2aTerminalState::decode(&outcome_before_dispatch).is_err());
    assert!(C2b2aTerminalState::active_item(0, 1).is_err());
    assert!(C2b2aTerminalState::active_item(1, 0).is_err());
}

#[test]
fn foundation_constants_do_not_claim_runtime_or_invent_calibrated_limits() {
    assert_eq!(C2B2A_GUEST_CPUS, 2);
    assert_eq!(C2B2A_GUEST_MEMORY_BYTES, 4 * 1024 * 1024 * 1024);
    assert_eq!(C2B2A_PRIVATE_DATA_DISK_BYTES, 4_294_967_296);
    assert_eq!(C2B2A_INPUT_BUNDLE_MAX_BYTES, 512 * 1024 * 1024);
    assert_eq!(C2B2A_RETAINED_EVIDENCE_MAX_BYTES, 64 * 1024 * 1024);
    assert_eq!(C2B2A_TERMINAL_STRUCTURAL_RECEIPT_MAX_BYTES, 256 * 1024);
    assert_eq!(C2B2A_PER_ROLE_OUTPUT_MAX_BYTES, 64 * 1024);
    assert_eq!(C2B2A_PRE_INSTANCE_AVAILABLE_KIB, 134_217_728);
    assert_eq!(C2B2A_POST_CLEANUP_AVAILABLE_KIB, 19_427_004);
    assert_eq!(C2B2A_FILESYSTEM_NOISE_TOLERANCE_BYTES, 1_073_741_824);
    assert_eq!(C2B2A_ROLE_DEADLINE_SECONDS, 60);
    assert_eq!(C2B2A_JOURNEY_DEADLINE_SECONDS, 240);
    assert_eq!(C2B2A_SHORT_GUEST_DEADLINE_SECONDS, 600);
    assert_eq!(C2B2A_LONG_GUEST_DEADLINE_SECONDS, 2_700);
    assert_eq!(C2B2A_ACCEPTANCE_DEADLINE_SECONDS, 18_000);
    assert_eq!(C2B2A_CAMPAIGN_DEADLINE_SECONDS, 18_600);
}
