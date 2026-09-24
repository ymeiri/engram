//! Handwritten fixed-width C2B2a wire codecs and pure protocol automata.

use crate::contract::{
    active_item_code, expected_measurement, frozen_manifest_sha256, host_schedule_manifest_kind,
    probe_subattempt_count, select_schedule_entry, sha256, Digest32, ErrorCategory,
    InstanceAutomaton, ManifestKindV1, MeasurementIdentity, MeasurementV1, MessageKind, RoleId,
    ScheduleEntry, TerminalStateV1, CONTRACT_SHA256, MAX_PROTOCOL_PAYLOAD_LEN, PROFILE_ID,
    PROTOCOL_HEADER_LEN, PROTOCOL_MAGIC, PROTOCOL_VERSION, RESOURCE_PROFILE_V1,
};

pub const MAX_FRAME_LEN: usize = PROTOCOL_HEADER_LEN + MAX_PROTOCOL_PAYLOAD_LEN;
pub const EMPTY_SHA256: Digest32 = crate::contract::digest_from_hex(
    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamBinding {
    pub run_nonce: Digest32,
    pub contract_sha256: Digest32,
    pub manifest_sha256: Digest32,
}

impl StreamBinding {
    /// Constructs an authoritative binding only from a reviewed, compiled manifest identity.
    pub fn for_manifest(
        run_nonce: Digest32,
        manifest_kind: ManifestKindV1,
    ) -> Result<Self, ProtocolError> {
        let manifest_sha256 = frozen_manifest_sha256(manifest_kind)
            .map_err(|missing| ProtocolError::ManifestDigestNotFrozen(missing.kind))?;
        Ok(Self {
            run_nonce,
            contract_sha256: CONTRACT_SHA256,
            manifest_sha256,
        })
    }

    /// Constructs bytes for isolated codec tests and non-authoritative parsing only.
    ///
    /// This constructor does not prove that `manifest_sha256` names a reviewed canonical manifest.
    /// Acceptance paths must use [`Self::for_manifest`].
    pub const fn unverified_for_codec(run_nonce: Digest32, manifest_sha256: Digest32) -> Self {
        Self {
            run_nonce,
            contract_sha256: CONTRACT_SHA256,
            manifest_sha256,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameHeader {
    pub kind: MessageKind,
    pub sequence: u32,
    pub case_or_probe_id: u16,
    pub fixture_or_subattempt_id: u16,
    pub profile_id: u16,
    pub role_id: RoleId,
    pub status: u16,
    pub payload_len: u32,
    pub binding: StreamBinding,
    pub payload_sha256: Digest32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub header: FrameHeader,
    payload: [u8; MAX_PROTOCOL_PAYLOAD_LEN],
}

impl Frame {
    pub fn new(
        kind: MessageKind,
        sequence: u32,
        case_or_probe_id: u16,
        fixture_or_subattempt_id: u16,
        role_id: RoleId,
        status: u16,
        binding: StreamBinding,
        payload: &[u8],
    ) -> Result<Self, ProtocolError> {
        if payload.len() > MAX_PROTOCOL_PAYLOAD_LEN {
            return Err(ProtocolError::PayloadTooLarge);
        }
        let mut bounded_payload = [0_u8; MAX_PROTOCOL_PAYLOAD_LEN];
        bounded_payload[..payload.len()].copy_from_slice(payload);
        let frame = Self {
            header: FrameHeader {
                kind,
                sequence,
                case_or_probe_id,
                fixture_or_subattempt_id,
                profile_id: PROFILE_ID,
                role_id,
                status,
                payload_len: payload.len() as u32,
                binding,
                payload_sha256: sha256(payload),
            },
            payload: bounded_payload,
        };
        validate_frame_shape(&frame)?;
        Ok(frame)
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload[..self.header.payload_len as usize]
    }

    pub fn encode(&self) -> EncodedFrame {
        let mut bytes = [0_u8; MAX_FRAME_LEN];
        encode_header(&self.header, &mut bytes[..PROTOCOL_HEADER_LEN]);
        let payload_len = self.header.payload_len as usize;
        bytes[PROTOCOL_HEADER_LEN..PROTOCOL_HEADER_LEN + payload_len]
            .copy_from_slice(self.payload());
        EncodedFrame {
            bytes,
            len: PROTOCOL_HEADER_LEN + payload_len,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedFrame {
    bytes: [u8; MAX_FRAME_LEN],
    len: usize,
}

impl EncodedFrame {
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolError {
    WrongLength,
    WrongMagic,
    WrongVersion,
    UnknownKind,
    ZeroSequence,
    UnknownRole,
    WrongProfile,
    NonzeroFlags,
    PayloadTooLarge,
    PayloadLengthMismatch,
    PayloadDigestMismatch,
    BindingMismatch,
    SequenceMismatch,
    WrongPayloadLength,
    WrongStatus,
    UnknownCategory,
    CategoryMismatch,
    WrongItemBinding,
    ReservedFieldNonzero,
    InvalidTerminalState,
    InvalidMeasurement,
    ChallengeMismatch,
    CommittedStateMismatch,
    RandomnessReuse,
    WrongDirection,
    OutOfState,
    EarlyEof,
    LateEvent,
    ManifestDigestNotFrozen(ManifestKindV1),
    NoRoleChannel,
    InvalidRoleSelection,
    UnexpectedOutput,
    OutputLimitExceeded,
    DuplicateEof,
    MissingTerminalEvidence,
    WrongTerminalEvidence,
    Poisoned,
}

pub fn decode_frame(bytes: &[u8]) -> Result<Frame, ProtocolError> {
    if bytes.len() < PROTOCOL_HEADER_LEN {
        return Err(ProtocolError::WrongLength);
    }
    let header = decode_header(&bytes[..PROTOCOL_HEADER_LEN])?;
    let payload_len =
        usize::try_from(header.payload_len).map_err(|_| ProtocolError::PayloadTooLarge)?;
    if payload_len > MAX_PROTOCOL_PAYLOAD_LEN {
        return Err(ProtocolError::PayloadTooLarge);
    }
    if bytes.len() != PROTOCOL_HEADER_LEN + payload_len {
        return Err(ProtocolError::PayloadLengthMismatch);
    }
    let payload = &bytes[PROTOCOL_HEADER_LEN..];
    if sha256(payload) != header.payload_sha256 {
        return Err(ProtocolError::PayloadDigestMismatch);
    }
    let mut bounded_payload = [0_u8; MAX_PROTOCOL_PAYLOAD_LEN];
    bounded_payload[..payload_len].copy_from_slice(payload);
    let frame = Frame {
        header,
        payload: bounded_payload,
    };
    validate_frame_shape(&frame)?;
    Ok(frame)
}

fn encode_header(header: &FrameHeader, output: &mut [u8]) {
    debug_assert_eq!(output.len(), PROTOCOL_HEADER_LEN);
    output[0..8].copy_from_slice(&PROTOCOL_MAGIC);
    put_u16(output, 8, PROTOCOL_VERSION);
    put_u16(output, 10, header.kind as u16);
    put_u32(output, 12, header.sequence);
    put_u16(output, 16, header.case_or_probe_id);
    put_u16(output, 18, header.fixture_or_subattempt_id);
    put_u16(output, 20, header.profile_id);
    put_u16(output, 22, header.role_id as u16);
    put_u16(output, 24, header.status);
    put_u16(output, 26, 0);
    put_u32(output, 28, header.payload_len);
    output[32..64].copy_from_slice(header.binding.run_nonce.as_bytes());
    output[64..96].copy_from_slice(header.binding.contract_sha256.as_bytes());
    output[96..128].copy_from_slice(header.binding.manifest_sha256.as_bytes());
    output[128..160].copy_from_slice(header.payload_sha256.as_bytes());
}

fn decode_header(bytes: &[u8]) -> Result<FrameHeader, ProtocolError> {
    if bytes.len() != PROTOCOL_HEADER_LEN {
        return Err(ProtocolError::WrongLength);
    }
    if bytes[0..8] != PROTOCOL_MAGIC {
        return Err(ProtocolError::WrongMagic);
    }
    if get_u16(bytes, 8) != PROTOCOL_VERSION {
        return Err(ProtocolError::WrongVersion);
    }
    let kind = MessageKind::try_from(get_u16(bytes, 10)).map_err(|_| ProtocolError::UnknownKind)?;
    let sequence = get_u32(bytes, 12);
    if sequence == 0 {
        return Err(ProtocolError::ZeroSequence);
    }
    let role_id = RoleId::try_from(get_u16(bytes, 22)).map_err(|_| ProtocolError::UnknownRole)?;
    if get_u16(bytes, 20) != PROFILE_ID {
        return Err(ProtocolError::WrongProfile);
    }
    if get_u16(bytes, 26) != 0 {
        return Err(ProtocolError::NonzeroFlags);
    }
    Ok(FrameHeader {
        kind,
        sequence,
        case_or_probe_id: get_u16(bytes, 16),
        fixture_or_subattempt_id: get_u16(bytes, 18),
        profile_id: PROFILE_ID,
        role_id,
        status: get_u16(bytes, 24),
        payload_len: get_u32(bytes, 28),
        binding: StreamBinding {
            run_nonce: copy_digest(bytes, 32),
            contract_sha256: copy_digest(bytes, 64),
            manifest_sha256: copy_digest(bytes, 96),
        },
        payload_sha256: copy_digest(bytes, 128),
    })
}

fn validate_frame_shape(frame: &Frame) -> Result<(), ProtocolError> {
    let header = &frame.header;
    if header.sequence == 0 {
        return Err(ProtocolError::ZeroSequence);
    }
    if header.profile_id != PROFILE_ID {
        return Err(ProtocolError::WrongProfile);
    }
    if header.binding.contract_sha256 != CONTRACT_SHA256 {
        return Err(ProtocolError::BindingMismatch);
    }
    let expected_len = expected_payload_len(header.kind, header.role_id)?;
    if header.payload_len as usize != expected_len {
        return Err(ProtocolError::WrongPayloadLength);
    }

    let host_fixed = matches!(
        header.kind,
        MessageKind::HostStart
            | MessageKind::HostReady
            | MessageKind::Challenge
            | MessageKind::Terminal
            | MessageKind::TerminalReady
    );
    if host_fixed
        && (header.role_id != RoleId::Supervisor
            || header.case_or_probe_id != 0
            || header.fixture_or_subattempt_id != 0)
    {
        return Err(ProtocolError::WrongItemBinding);
    }
    if header.kind == MessageKind::Measurement
        && (header.role_id == RoleId::Supervisor
            || header.case_or_probe_id == 0
            || header.fixture_or_subattempt_id == 0)
    {
        return Err(ProtocolError::WrongItemBinding);
    }
    if is_role_kind(header.kind)
        && (header.role_id == RoleId::Supervisor
            || header.case_or_probe_id == 0
            || header.fixture_or_subattempt_id == 0)
    {
        return Err(ProtocolError::WrongItemBinding);
    }

    match header.kind {
        MessageKind::Abort => validate_abort(frame),
        MessageKind::TerminalReady => {
            let state = decode_terminal_state(frame.payload())?;
            validate_terminal_status(header.status, state)
        }
        MessageKind::Terminal => {
            let (_, state) = decode_terminal_payload(frame.payload())?;
            validate_terminal_status(header.status, state)
        }
        MessageKind::HostStart | MessageKind::HostReady => {
            if header.status != 0 {
                return Err(ProtocolError::WrongStatus);
            }
            let selector = decode_host_selector(frame.payload())?;
            if select_schedule_entry(
                selector.phase_code,
                selector.instance_class_code,
                selector.instance_ordinal,
            )
            .is_none()
            {
                return Err(ProtocolError::WrongItemBinding);
            }
            Ok(())
        }
        MessageKind::Measurement => {
            if header.status != 0 {
                return Err(ProtocolError::WrongStatus);
            }
            let measurement = decode_measurement(frame.payload())?;
            let probe_id = if header.role_id == RoleId::Boundary {
                Some(header.case_or_probe_id)
            } else {
                None
            };
            measurement
                .validate_for(probe_id, header.role_id)
                .map_err(|_| ProtocolError::InvalidMeasurement)
        }
        _ => {
            if header.status != 0 {
                return Err(ProtocolError::WrongStatus);
            }
            Ok(())
        }
    }
}

const fn expected_payload_len(kind: MessageKind, role_id: RoleId) -> Result<usize, ProtocolError> {
    match kind {
        MessageKind::Start
        | MessageKind::Ready
        | MessageKind::Continue
        | MessageKind::WriterCommitted
        | MessageKind::LockObserved
        | MessageKind::ExpectedLockRejected
        | MessageKind::HandlesDropped
        | MessageKind::RoleTerminal => Ok(0),
        MessageKind::Abort => {
            if matches!(role_id, RoleId::Supervisor) {
                Ok(TerminalStateV1::ENCODED_LEN)
            } else {
                Ok(4)
            }
        }
        MessageKind::HostStart | MessageKind::HostReady => Ok(HostSelectorV1::ENCODED_LEN),
        MessageKind::Measurement => Ok(MeasurementV1::ENCODED_LEN),
        MessageKind::Challenge | MessageKind::TerminalReady => Ok(32),
        MessageKind::Terminal => Ok(64),
    }
}

const fn is_role_kind(kind: MessageKind) -> bool {
    matches!(
        kind,
        MessageKind::Start
            | MessageKind::Ready
            | MessageKind::Continue
            | MessageKind::WriterCommitted
            | MessageKind::LockObserved
            | MessageKind::ExpectedLockRejected
            | MessageKind::HandlesDropped
            | MessageKind::RoleTerminal
    )
}

fn validate_abort(frame: &Frame) -> Result<(), ProtocolError> {
    let category =
        ErrorCategory::try_from(frame.header.status).map_err(|_| ProtocolError::UnknownCategory)?;
    if category == ErrorCategory::Success {
        return Err(ProtocolError::WrongStatus);
    }
    if frame.header.role_id == RoleId::Supervisor {
        let no_item =
            frame.header.case_or_probe_id == 0 && frame.header.fixture_or_subattempt_id == 0;
        let active_item =
            frame.header.case_or_probe_id != 0 && frame.header.fixture_or_subattempt_id != 0;
        if !no_item && !active_item {
            return Err(ProtocolError::WrongItemBinding);
        }
        let state = decode_terminal_state(frame.payload())?;
        let expected_active = if active_item {
            active_item_code(
                frame.header.case_or_probe_id,
                frame.header.fixture_or_subattempt_id,
            )
        } else {
            0
        };
        if state.active_item_code != expected_active {
            return Err(ProtocolError::WrongItemBinding);
        }
        validate_terminal_status(frame.header.status, state)
    } else {
        if frame.header.case_or_probe_id == 0 || frame.header.fixture_or_subattempt_id == 0 {
            return Err(ProtocolError::WrongItemBinding);
        }
        let payload_category = ErrorCategory::try_from(get_u32(frame.payload(), 0))
            .map_err(|_| ProtocolError::UnknownCategory)?;
        if payload_category != category {
            return Err(ProtocolError::CategoryMismatch);
        }
        Ok(())
    }
}

fn validate_terminal_status(status: u16, state: TerminalStateV1) -> Result<(), ProtocolError> {
    let category = ErrorCategory::try_from(status).map_err(|_| ProtocolError::UnknownCategory)?;
    if state.primary_category != category as u32 {
        return Err(ProtocolError::CategoryMismatch);
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
pub struct StreamVerifier {
    binding: StreamBinding,
    next_sequence: u32,
    poisoned: bool,
}

impl StreamVerifier {
    pub const fn new(binding: StreamBinding) -> Self {
        Self {
            binding,
            next_sequence: 1,
            poisoned: false,
        }
    }

    pub const fn next_sequence(&self) -> u32 {
        self.next_sequence
    }

    pub fn accept(&mut self, bytes: &[u8]) -> Result<Frame, ProtocolError> {
        if self.poisoned {
            return Err(ProtocolError::Poisoned);
        }
        let result = (|| {
            let frame = decode_frame(bytes)?;
            if frame.header.binding != self.binding {
                return Err(ProtocolError::BindingMismatch);
            }
            if frame.header.sequence != self.next_sequence {
                return Err(ProtocolError::SequenceMismatch);
            }
            self.next_sequence = self
                .next_sequence
                .checked_add(1)
                .ok_or(ProtocolError::SequenceMismatch)?;
            Ok(frame)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RoleStreamVerifier {
    stream: StreamVerifier,
    case_or_probe_id: u16,
    fixture_or_subattempt_id: u16,
    role_id: RoleId,
}

impl RoleStreamVerifier {
    pub fn new(
        binding: StreamBinding,
        case_or_probe_id: u16,
        fixture_or_subattempt_id: u16,
        role_id: RoleId,
    ) -> Result<Self, ProtocolError> {
        if case_or_probe_id == 0 || fixture_or_subattempt_id == 0 || role_id == RoleId::Supervisor {
            return Err(ProtocolError::WrongItemBinding);
        }
        Ok(Self {
            stream: StreamVerifier::new(binding),
            case_or_probe_id,
            fixture_or_subattempt_id,
            role_id,
        })
    }

    pub fn accept(&mut self, bytes: &[u8]) -> Result<Frame, ProtocolError> {
        if self.stream.poisoned {
            return Err(ProtocolError::Poisoned);
        }
        let result = (|| {
            let frame = decode_frame(bytes)?;
            if frame.header.binding != self.stream.binding {
                return Err(ProtocolError::BindingMismatch);
            }
            if frame.header.sequence != self.stream.next_sequence {
                return Err(ProtocolError::SequenceMismatch);
            }
            if frame.header.case_or_probe_id != self.case_or_probe_id
                || frame.header.fixture_or_subattempt_id != self.fixture_or_subattempt_id
                || frame.header.role_id != self.role_id
            {
                return Err(ProtocolError::WrongItemBinding);
            }
            self.stream.next_sequence = self
                .stream
                .next_sequence
                .checked_add(1)
                .ok_or(ProtocolError::SequenceMismatch)?;
            Ok(frame)
        })();
        if result.is_err() {
            self.stream.poisoned = true;
        }
        result
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostSelectorV1 {
    pub phase_code: u16,
    pub instance_class_code: u16,
    pub instance_ordinal: u16,
}

impl HostSelectorV1 {
    pub const ENCODED_LEN: usize = 8;

    pub const fn from_entry(entry: &ScheduleEntry) -> Self {
        Self {
            phase_code: entry.phase_code,
            instance_class_code: entry.class_code,
            instance_ordinal: entry.ordinal,
        }
    }
}

pub fn encode_host_selector(selector: HostSelectorV1) -> [u8; HostSelectorV1::ENCODED_LEN] {
    let mut output = [0_u8; HostSelectorV1::ENCODED_LEN];
    put_u16(&mut output, 0, selector.phase_code);
    put_u16(&mut output, 2, selector.instance_class_code);
    put_u16(&mut output, 4, selector.instance_ordinal);
    put_u16(&mut output, 6, 0);
    output
}

pub fn decode_host_selector(bytes: &[u8]) -> Result<HostSelectorV1, ProtocolError> {
    if bytes.len() != HostSelectorV1::ENCODED_LEN {
        return Err(ProtocolError::WrongPayloadLength);
    }
    if get_u16(bytes, 6) != 0 {
        return Err(ProtocolError::ReservedFieldNonzero);
    }
    Ok(HostSelectorV1 {
        phase_code: get_u16(bytes, 0),
        instance_class_code: get_u16(bytes, 2),
        instance_ordinal: get_u16(bytes, 4),
    })
}

pub fn encode_measurement(measurement: MeasurementV1) -> [u8; MeasurementV1::ENCODED_LEN] {
    let mut output = [0_u8; MeasurementV1::ENCODED_LEN];
    for (index, value) in measurement.words().into_iter().enumerate() {
        put_u64(&mut output, index * 8, value);
    }
    output
}

pub fn decode_measurement(bytes: &[u8]) -> Result<MeasurementV1, ProtocolError> {
    if bytes.len() != MeasurementV1::ENCODED_LEN {
        return Err(ProtocolError::WrongPayloadLength);
    }
    let mut words = [0_u64; MeasurementV1::FIELD_COUNT];
    for (index, value) in words.iter_mut().enumerate() {
        *value = get_u64(bytes, index * 8);
    }
    Ok(MeasurementV1::from_words(words))
}

pub fn encode_terminal_state(state: TerminalStateV1) -> [u8; TerminalStateV1::ENCODED_LEN] {
    let mut output = [0_u8; TerminalStateV1::ENCODED_LEN];
    for (index, value) in state.words().into_iter().enumerate() {
        put_u32(&mut output, index * 4, value);
    }
    output
}

pub fn decode_terminal_state(bytes: &[u8]) -> Result<TerminalStateV1, ProtocolError> {
    if bytes.len() != TerminalStateV1::ENCODED_LEN {
        return Err(ProtocolError::WrongPayloadLength);
    }
    let mut words = [0_u32; TerminalStateV1::FIELD_COUNT];
    for (index, value) in words.iter_mut().enumerate() {
        *value = get_u32(bytes, index * 4);
    }
    let state = TerminalStateV1::from_words(words);
    state
        .validate()
        .map_err(|_| ProtocolError::InvalidTerminalState)?;
    let outcome_is_uncertain = state.primary_category == ErrorCategory::OutcomeUncertain as u32;
    let dispatched_without_acknowledgement =
        state.write_may_have_been_dispatched == 1 && state.writer_acknowledged == 0;
    if outcome_is_uncertain != dispatched_without_acknowledgement {
        return Err(ProtocolError::InvalidTerminalState);
    }
    let category = ErrorCategory::try_from(state.primary_category)
        .map_err(|_| ProtocolError::InvalidTerminalState)?;
    let category_is_legal_for_phase = if state.write_may_have_been_dispatched == 0 {
        matches!(
            category,
            ErrorCategory::Success
                | ErrorCategory::HostContract
                | ErrorCategory::GuestConfiguration
                | ErrorCategory::Containment
                | ErrorCategory::StaticFirewall
                | ErrorCategory::LimitExceeded
                | ErrorCategory::InvalidFixture
                | ErrorCategory::StoreOpenFailure
                | ErrorCategory::MeasurementFailure
                | ErrorCategory::ProtocolFailure
                | ErrorCategory::ChallengeFailure
        )
    } else if state.writer_acknowledged == 0 {
        outcome_is_uncertain
    } else {
        matches!(
            category,
            ErrorCategory::Success
                | ErrorCategory::HostContract
                | ErrorCategory::GuestConfiguration
                | ErrorCategory::Containment
                | ErrorCategory::LimitExceeded
                | ErrorCategory::ExclusivityBroken
                | ErrorCategory::ContenderFailure
                | ErrorCategory::ReleaseUnproven
                | ErrorCategory::ReopenFailure
                | ErrorCategory::EngineReadFailure
                | ErrorCategory::CanonicalMismatch
                | ErrorCategory::MeasurementFailure
                | ErrorCategory::ProtocolFailure
                | ErrorCategory::ChallengeFailure
        )
    };
    if !category_is_legal_for_phase {
        return Err(ProtocolError::InvalidTerminalState);
    }
    Ok(state)
}

pub fn encode_terminal_payload(challenge: Digest32, state: TerminalStateV1) -> [u8; 64] {
    let mut output = [0_u8; 64];
    output[..32].copy_from_slice(challenge.as_bytes());
    output[32..].copy_from_slice(&encode_terminal_state(state));
    output
}

pub fn decode_terminal_payload(bytes: &[u8]) -> Result<(Digest32, TerminalStateV1), ProtocolError> {
    if bytes.len() != 64 {
        return Err(ProtocolError::WrongPayloadLength);
    }
    Ok((copy_digest(bytes, 0), decode_terminal_state(&bytes[32..])?))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalCommitmentV1 {
    pub challenge: Digest32,
    pub state: TerminalStateV1,
}

impl TerminalCommitmentV1 {
    pub fn verify_echo(self, payload: &[u8]) -> Result<(), ProtocolError> {
        let (challenge, state) = decode_terminal_payload(payload)?;
        if challenge != self.challenge {
            return Err(ProtocolError::ChallengeMismatch);
        }
        if state != self.state {
            return Err(ProtocolError::CommittedStateMismatch);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    SupervisorToCollector,
    CollectorToSupervisor,
}

impl Direction {
    const fn opposite(self) -> Self {
        match self {
            Self::SupervisorToCollector => Self::CollectorToSupervisor,
            Self::CollectorToSupervisor => Self::SupervisorToCollector,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlEvent {
    Frame(MessageKind),
    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirectedControlEvent {
    pub direction: Direction,
    pub event: ControlEvent,
}

const fn frame(direction: Direction, kind: MessageKind) -> DirectedControlEvent {
    DirectedControlEvent {
        direction,
        event: ControlEvent::Frame(kind),
    }
}

const fn eof(direction: Direction) -> DirectedControlEvent {
    DirectedControlEvent {
        direction,
        event: ControlEvent::Eof,
    }
}

const WRITER_EVENTS: [DirectedControlEvent; 11] = [
    frame(Direction::SupervisorToCollector, MessageKind::Start),
    frame(Direction::CollectorToSupervisor, MessageKind::Ready),
    frame(Direction::SupervisorToCollector, MessageKind::Continue),
    frame(
        Direction::CollectorToSupervisor,
        MessageKind::WriterCommitted,
    ),
    frame(Direction::CollectorToSupervisor, MessageKind::LockObserved),
    frame(Direction::SupervisorToCollector, MessageKind::Continue),
    frame(
        Direction::CollectorToSupervisor,
        MessageKind::HandlesDropped,
    ),
    frame(Direction::SupervisorToCollector, MessageKind::Continue),
    frame(Direction::CollectorToSupervisor, MessageKind::RoleTerminal),
    eof(Direction::CollectorToSupervisor),
    eof(Direction::SupervisorToCollector),
];
const CONTENDER_EVENTS: [DirectedControlEvent; 7] = [
    frame(Direction::SupervisorToCollector, MessageKind::Start),
    frame(Direction::CollectorToSupervisor, MessageKind::Ready),
    frame(Direction::SupervisorToCollector, MessageKind::Continue),
    frame(
        Direction::CollectorToSupervisor,
        MessageKind::ExpectedLockRejected,
    ),
    frame(Direction::CollectorToSupervisor, MessageKind::RoleTerminal),
    eof(Direction::CollectorToSupervisor),
    eof(Direction::SupervisorToCollector),
];
const RELEASE_EVENTS: [DirectedControlEvent; 10] = [
    frame(Direction::SupervisorToCollector, MessageKind::Start),
    frame(Direction::CollectorToSupervisor, MessageKind::Ready),
    frame(Direction::SupervisorToCollector, MessageKind::Continue),
    frame(Direction::CollectorToSupervisor, MessageKind::LockObserved),
    frame(Direction::SupervisorToCollector, MessageKind::Continue),
    frame(
        Direction::CollectorToSupervisor,
        MessageKind::HandlesDropped,
    ),
    frame(Direction::SupervisorToCollector, MessageKind::Continue),
    frame(Direction::CollectorToSupervisor, MessageKind::RoleTerminal),
    eof(Direction::CollectorToSupervisor),
    eof(Direction::SupervisorToCollector),
];
const READER_EVENTS: [DirectedControlEvent; 6] = [
    frame(Direction::SupervisorToCollector, MessageKind::Start),
    frame(Direction::CollectorToSupervisor, MessageKind::Ready),
    frame(Direction::SupervisorToCollector, MessageKind::Continue),
    frame(Direction::CollectorToSupervisor, MessageKind::RoleTerminal),
    eof(Direction::CollectorToSupervisor),
    eof(Direction::SupervisorToCollector),
];
const BOUNDARY_KERNEL_EVENTS: [DirectedControlEvent; 5] = [
    frame(Direction::SupervisorToCollector, MessageKind::Start),
    frame(Direction::CollectorToSupervisor, MessageKind::Ready),
    frame(Direction::SupervisorToCollector, MessageKind::Continue),
    eof(Direction::CollectorToSupervisor),
    eof(Direction::SupervisorToCollector),
];
const BOUNDARY_SUPERVISOR_AFTER_CONTINUE_EVENTS: [DirectedControlEvent; 5] = [
    frame(Direction::SupervisorToCollector, MessageKind::Start),
    frame(Direction::CollectorToSupervisor, MessageKind::Ready),
    frame(Direction::SupervisorToCollector, MessageKind::Continue),
    eof(Direction::SupervisorToCollector),
    eof(Direction::CollectorToSupervisor),
];
const BOUNDARY_SUPERVISOR_AFTER_ROLE_TERMINAL_EVENTS: [DirectedControlEvent; 6] = [
    frame(Direction::SupervisorToCollector, MessageKind::Start),
    frame(Direction::CollectorToSupervisor, MessageKind::Ready),
    frame(Direction::SupervisorToCollector, MessageKind::Continue),
    frame(Direction::CollectorToSupervisor, MessageKind::RoleTerminal),
    eof(Direction::SupervisorToCollector),
    eof(Direction::CollectorToSupervisor),
];
const BOUNDARY_PRE_START_EVENTS: [DirectedControlEvent; 2] = [
    eof(Direction::SupervisorToCollector),
    eof(Direction::CollectorToSupervisor),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RolePlan {
    Writer,
    Contender,
    ReleaseProbe,
    Reader,
    BoundaryRoleTerminal,
    BoundaryKernelTerminal,
    BoundarySupervisorAfterContinue,
    BoundarySupervisorAfterRoleTerminal,
    BoundaryPreStart,
}

impl RolePlan {
    const fn events(self) -> &'static [DirectedControlEvent] {
        match self {
            Self::Writer => &WRITER_EVENTS,
            Self::Contender => &CONTENDER_EVENTS,
            Self::ReleaseProbe => &RELEASE_EVENTS,
            Self::Reader | Self::BoundaryRoleTerminal => &READER_EVENTS,
            Self::BoundaryKernelTerminal => &BOUNDARY_KERNEL_EVENTS,
            Self::BoundarySupervisorAfterContinue => &BOUNDARY_SUPERVISOR_AFTER_CONTINUE_EVENTS,
            Self::BoundarySupervisorAfterRoleTerminal => {
                &BOUNDARY_SUPERVISOR_AFTER_ROLE_TERMINAL_EVENTS
            }
            Self::BoundaryPreStart => &BOUNDARY_PRE_START_EVENTS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoleSelectionV1 {
    Characterization { subattempt_id: u16, role_id: RoleId },
    Positive { case_id: u16, role_id: RoleId },
    Boundary { probe_id: u16, subattempt_id: u16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupervisorTerminalV1 {
    StdoutOverflow,
    TrailingControl,
    LateStdout,
    PostTerminalLinkCount,
    WrongDevicePreStart,
    WritableMountPreStart,
    InheritedFdPostExec,
    InheritedSocketPostExec,
}

// These names mirror the frozen protocol's distinct terminal categories.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoleCompletionV1 {
    RoleTerminal,
    KernelTerminal,
    SupervisorTerminal(SupervisorTerminalV1),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputPolicyV1 {
    Empty,
    StdoutOverflow,
    TrailingControlByte,
    LateStdoutByte,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoleProtocolSpecV1 {
    pub manifest_kind: ManifestKindV1,
    pub case_or_probe_id: u16,
    pub fixture_or_subattempt_id: u16,
    pub role_id: RoleId,
    pub plan: RolePlan,
    pub completion: RoleCompletionV1,
    pub output_policy: OutputPolicyV1,
}

pub fn derive_role_protocol_spec(
    selection: RoleSelectionV1,
) -> Result<RoleProtocolSpecV1, ProtocolError> {
    match selection {
        RoleSelectionV1::Characterization {
            subattempt_id,
            role_id,
        } => {
            if !(1..=3).contains(&subattempt_id) {
                return Err(ProtocolError::InvalidRoleSelection);
            }
            Ok(RoleProtocolSpecV1 {
                manifest_kind: ManifestKindV1::Characterization,
                case_or_probe_id: 1,
                fixture_or_subattempt_id: subattempt_id,
                role_id,
                plan: ordinary_role_plan(role_id)?,
                completion: RoleCompletionV1::RoleTerminal,
                output_policy: OutputPolicyV1::Empty,
            })
        }
        RoleSelectionV1::Positive { case_id, role_id } => {
            if !(1..=20).contains(&case_id) {
                return Err(ProtocolError::InvalidRoleSelection);
            }
            Ok(RoleProtocolSpecV1 {
                manifest_kind: ManifestKindV1::PositiveCases,
                case_or_probe_id: case_id,
                fixture_or_subattempt_id: case_id,
                role_id,
                plan: ordinary_role_plan(role_id)?,
                completion: RoleCompletionV1::RoleTerminal,
                output_policy: OutputPolicyV1::Empty,
            })
        }
        RoleSelectionV1::Boundary {
            probe_id,
            subattempt_id,
        } => derive_boundary_role_protocol_spec(probe_id, subattempt_id),
    }
}

fn ordinary_role_plan(role_id: RoleId) -> Result<RolePlan, ProtocolError> {
    match role_id {
        RoleId::Writer => Ok(RolePlan::Writer),
        RoleId::Contender => Ok(RolePlan::Contender),
        RoleId::ReleaseProbe => Ok(RolePlan::ReleaseProbe),
        RoleId::Reader => Ok(RolePlan::Reader),
        RoleId::Supervisor | RoleId::Boundary | RoleId::JourneyAggregate => {
            Err(ProtocolError::InvalidRoleSelection)
        }
    }
}

fn derive_boundary_role_protocol_spec(
    probe_id: u16,
    subattempt_id: u16,
) -> Result<RoleProtocolSpecV1, ProtocolError> {
    let subattempt_count =
        probe_subattempt_count(probe_id).ok_or(ProtocolError::InvalidRoleSelection)?;
    if subattempt_id == 0 || subattempt_id > subattempt_count {
        return Err(ProtocolError::InvalidRoleSelection);
    }
    if matches!(probe_id, 40 | 41) {
        return Err(ProtocolError::NoRoleChannel);
    }

    let (plan, completion, output_policy) = match probe_id {
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
            RoleCompletionV1::SupervisorTerminal(SupervisorTerminalV1::PostTerminalLinkCount),
            OutputPolicyV1::Empty,
        ),
        15 => (
            RolePlan::BoundaryPreStart,
            RoleCompletionV1::SupervisorTerminal(SupervisorTerminalV1::WrongDevicePreStart),
            OutputPolicyV1::Empty,
        ),
        16 => (
            RolePlan::BoundaryPreStart,
            RoleCompletionV1::SupervisorTerminal(SupervisorTerminalV1::WritableMountPreStart),
            OutputPolicyV1::Empty,
        ),
        17 => (
            RolePlan::BoundaryPreStart,
            RoleCompletionV1::SupervisorTerminal(SupervisorTerminalV1::InheritedFdPostExec),
            OutputPolicyV1::Empty,
        ),
        18 => (
            RolePlan::BoundaryPreStart,
            RoleCompletionV1::SupervisorTerminal(SupervisorTerminalV1::InheritedSocketPostExec),
            OutputPolicyV1::Empty,
        ),
        _ => (
            RolePlan::BoundaryRoleTerminal,
            RoleCompletionV1::RoleTerminal,
            OutputPolicyV1::Empty,
        ),
    };

    Ok(RoleProtocolSpecV1 {
        manifest_kind: ManifestKindV1::BoundaryProbes,
        case_or_probe_id: probe_id,
        fixture_or_subattempt_id: subattempt_id,
        role_id: RoleId::Boundary,
        plan,
        completion,
        output_policy,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AbortState {
    None,
    SenderEof(Direction),
    ReceiverEof(Direction),
    Complete,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RoleAutomaton {
    plan: RolePlan,
    index: usize,
    abort: AbortState,
    poisoned: bool,
}

impl RoleAutomaton {
    pub const fn new(plan: RolePlan) -> Self {
        Self {
            plan,
            index: 0,
            abort: AbortState::None,
            poisoned: false,
        }
    }

    pub const fn is_complete(&self) -> bool {
        !self.poisoned
            && match self.abort {
                AbortState::Complete => true,
                AbortState::None => self.index == self.plan.events().len(),
                _ => false,
            }
    }

    pub fn accept(&mut self, event: DirectedControlEvent) -> Result<(), ProtocolError> {
        if self.poisoned {
            return Err(ProtocolError::Poisoned);
        }
        let result = self.accept_live(event);
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }

    fn accept_live(&mut self, event: DirectedControlEvent) -> Result<(), ProtocolError> {
        match self.abort {
            AbortState::SenderEof(sender) => {
                if event != eof(sender) {
                    return Err(ProtocolError::OutOfState);
                }
                self.abort = AbortState::ReceiverEof(sender.opposite());
                return Ok(());
            }
            AbortState::ReceiverEof(receiver) => {
                if event != eof(receiver) {
                    return Err(ProtocolError::OutOfState);
                }
                self.abort = AbortState::Complete;
                return Ok(());
            }
            AbortState::Complete => return Err(ProtocolError::LateEvent),
            AbortState::None => {}
        }

        let expected = self
            .plan
            .events()
            .get(self.index)
            .copied()
            .ok_or(ProtocolError::LateEvent)?;
        if let ControlEvent::Frame(MessageKind::Abort) = event.event {
            if event.direction != expected.direction
                || !matches!(expected.event, ControlEvent::Frame(_))
            {
                return Err(ProtocolError::WrongDirection);
            }
            self.abort = AbortState::SenderEof(event.direction);
            return Ok(());
        }
        if event != expected {
            return Err(match event.event {
                ControlEvent::Eof => ProtocolError::EarlyEof,
                ControlEvent::Frame(_) => ProtocolError::OutOfState,
            });
        }
        self.index += 1;
        Ok(())
    }
}

/// Pure, manifest-derived role-channel verifier.
///
/// Kernel signal, pidfd, cgroup, mount, and descriptor evidence is deliberately supplied only as
/// an already-validated semantic observation.  This type closes the wire/output schedule; it does
/// not claim to collect or authenticate that Linux evidence by itself.
#[derive(Debug, PartialEq, Eq)]
pub struct RoleProtocolV1 {
    spec: RoleProtocolSpecV1,
    binding: StreamBinding,
    supervisor_to_collector: RoleStreamVerifier,
    collector_to_supervisor: RoleStreamVerifier,
    automaton: RoleAutomaton,
    role_terminal_seen: bool,
    abort_seen: bool,
    kernel_terminal_seen: bool,
    supervisor_terminal_seen: bool,
    stdout_bytes: u64,
    stderr_bytes: u64,
    trailing_control_bytes: u64,
    supervisor_to_collector_eof: bool,
    collector_to_supervisor_eof: bool,
    stdout_eof: bool,
    stderr_eof: bool,
    poisoned: bool,
}

impl RoleProtocolV1 {
    pub fn new(run_nonce: Digest32, selection: RoleSelectionV1) -> Result<Self, ProtocolError> {
        let spec = derive_role_protocol_spec(selection)?;
        let binding = StreamBinding::for_manifest(run_nonce, spec.manifest_kind)?;
        Self::new_bound(spec, binding)
    }

    fn new_bound(spec: RoleProtocolSpecV1, binding: StreamBinding) -> Result<Self, ProtocolError> {
        let expected_manifest = frozen_manifest_sha256(spec.manifest_kind);
        if binding.contract_sha256 != CONTRACT_SHA256
            || expected_manifest.is_ok_and(|digest| digest != binding.manifest_sha256)
        {
            return Err(ProtocolError::BindingMismatch);
        }
        Ok(Self {
            spec,
            binding,
            supervisor_to_collector: RoleStreamVerifier::new(
                binding,
                spec.case_or_probe_id,
                spec.fixture_or_subattempt_id,
                spec.role_id,
            )?,
            collector_to_supervisor: RoleStreamVerifier::new(
                binding,
                spec.case_or_probe_id,
                spec.fixture_or_subattempt_id,
                spec.role_id,
            )?,
            automaton: RoleAutomaton::new(spec.plan),
            role_terminal_seen: false,
            abort_seen: false,
            kernel_terminal_seen: false,
            supervisor_terminal_seen: false,
            stdout_bytes: 0,
            stderr_bytes: 0,
            trailing_control_bytes: 0,
            supervisor_to_collector_eof: false,
            collector_to_supervisor_eof: false,
            stdout_eof: false,
            stderr_eof: false,
            poisoned: false,
        })
    }

    pub const fn spec(&self) -> RoleProtocolSpecV1 {
        self.spec
    }

    pub const fn binding(&self) -> StreamBinding {
        self.binding
    }

    pub const fn stdout_observed_bytes(&self) -> u64 {
        self.stdout_bytes
    }

    pub const fn stdout_retained_bytes(&self) -> u64 {
        if self.stdout_bytes > RESOURCE_PROFILE_V1.per_role_output_bytes {
            RESOURCE_PROFILE_V1.per_role_output_bytes
        } else {
            self.stdout_bytes
        }
    }

    pub fn accept_control_frame(
        &mut self,
        direction: Direction,
        bytes: &[u8],
    ) -> Result<(), ProtocolError> {
        self.fail_closed(|state| state.accept_control_frame_live(direction, bytes))
    }

    pub fn accept_unframed_control_bytes(
        &mut self,
        direction: Direction,
        bytes: &[u8],
    ) -> Result<(), ProtocolError> {
        self.fail_closed(|state| state.accept_unframed_control_bytes_live(direction, bytes))
    }

    pub fn accept_stdout(&mut self, bytes: &[u8]) -> Result<(), ProtocolError> {
        self.fail_closed(|state| state.accept_stdout_live(bytes))
    }

    pub fn accept_stderr(&mut self, bytes: &[u8]) -> Result<(), ProtocolError> {
        self.fail_closed(|state| state.accept_stderr_live(bytes))
    }

    pub fn observe_kernel_terminal(&mut self) -> Result<(), ProtocolError> {
        self.fail_closed(Self::observe_kernel_terminal_live)
    }

    pub fn observe_supervisor_terminal(
        &mut self,
        observed: SupervisorTerminalV1,
    ) -> Result<(), ProtocolError> {
        self.fail_closed(|state| state.observe_supervisor_terminal_live(observed))
    }

    pub fn accept_control_eof(&mut self, direction: Direction) -> Result<(), ProtocolError> {
        self.fail_closed(|state| state.accept_control_eof_live(direction))
    }

    pub fn accept_stdout_eof(&mut self) -> Result<(), ProtocolError> {
        self.fail_closed(Self::accept_stdout_eof_live)
    }

    pub fn accept_stderr_eof(&mut self) -> Result<(), ProtocolError> {
        self.fail_closed(Self::accept_stderr_eof_live)
    }

    fn fail_closed<T>(
        &mut self,
        operation: impl FnOnce(&mut Self) -> Result<T, ProtocolError>,
    ) -> Result<T, ProtocolError> {
        if self.poisoned {
            return Err(ProtocolError::Poisoned);
        }
        let result = operation(self);
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }

    fn accept_control_frame_live(
        &mut self,
        direction: Direction,
        bytes: &[u8],
    ) -> Result<(), ProtocolError> {
        if self.control_eof_seen(direction) {
            return Err(ProtocolError::LateEvent);
        }
        let frame = match direction {
            Direction::SupervisorToCollector => self.supervisor_to_collector.accept(bytes)?,
            Direction::CollectorToSupervisor => self.collector_to_supervisor.accept(bytes)?,
        };
        self.automaton.accept(DirectedControlEvent {
            direction,
            event: ControlEvent::Frame(frame.header.kind),
        })?;
        match frame.header.kind {
            MessageKind::RoleTerminal => self.role_terminal_seen = true,
            MessageKind::Abort => self.abort_seen = true,
            _ => {}
        }
        Ok(())
    }

    fn accept_unframed_control_bytes_live(
        &mut self,
        direction: Direction,
        bytes: &[u8],
    ) -> Result<(), ProtocolError> {
        if bytes.is_empty() {
            return Err(ProtocolError::WrongLength);
        }
        if direction != Direction::CollectorToSupervisor
            || self.collector_to_supervisor_eof
            || self.spec.output_policy != OutputPolicyV1::TrailingControlByte
            || !self.role_terminal_seen
        {
            return Err(ProtocolError::UnexpectedOutput);
        }
        let next = self
            .trailing_control_bytes
            .checked_add(bytes.len() as u64)
            .ok_or(ProtocolError::OutputLimitExceeded)?;
        if next != 1 {
            return Err(ProtocolError::UnexpectedOutput);
        }
        self.trailing_control_bytes = next;
        Ok(())
    }

    fn accept_stdout_live(&mut self, bytes: &[u8]) -> Result<(), ProtocolError> {
        if bytes.is_empty() {
            return Err(ProtocolError::WrongLength);
        }
        if self.stdout_eof {
            return Err(ProtocolError::LateEvent);
        }
        let next = self
            .stdout_bytes
            .checked_add(bytes.len() as u64)
            .ok_or(ProtocolError::OutputLimitExceeded)?;
        match self.spec.output_policy {
            OutputPolicyV1::StdoutOverflow => {
                if self.automaton.index != 3 || self.supervisor_terminal_seen {
                    return Err(ProtocolError::UnexpectedOutput);
                }
                if next > RESOURCE_PROFILE_V1.per_role_output_bytes + 1 {
                    return Err(ProtocolError::OutputLimitExceeded);
                }
            }
            OutputPolicyV1::LateStdoutByte => {
                if self.supervisor_terminal_seen
                    || !self.role_terminal_seen
                    || !self.collector_to_supervisor_eof
                    || next != 1
                {
                    return Err(ProtocolError::UnexpectedOutput);
                }
            }
            OutputPolicyV1::Empty | OutputPolicyV1::TrailingControlByte => {
                return Err(ProtocolError::UnexpectedOutput);
            }
        }
        self.stdout_bytes = next;
        Ok(())
    }

    fn accept_stderr_live(&mut self, bytes: &[u8]) -> Result<(), ProtocolError> {
        if bytes.is_empty() {
            return Err(ProtocolError::WrongLength);
        }
        if self.stderr_eof {
            return Err(ProtocolError::LateEvent);
        }
        let _ = self
            .stderr_bytes
            .checked_add(bytes.len() as u64)
            .ok_or(ProtocolError::OutputLimitExceeded)?;
        Err(ProtocolError::UnexpectedOutput)
    }

    fn observe_kernel_terminal_live(&mut self) -> Result<(), ProtocolError> {
        if self.abort_seen
            || self.kernel_terminal_seen
            || self.spec.completion != RoleCompletionV1::KernelTerminal
            || self.role_terminal_seen
            || self.automaton.index != 3
        {
            return Err(ProtocolError::WrongTerminalEvidence);
        }
        self.kernel_terminal_seen = true;
        Ok(())
    }

    fn observe_supervisor_terminal_live(
        &mut self,
        observed: SupervisorTerminalV1,
    ) -> Result<(), ProtocolError> {
        let RoleCompletionV1::SupervisorTerminal(expected) = self.spec.completion else {
            return Err(ProtocolError::WrongTerminalEvidence);
        };
        if self.abort_seen || self.supervisor_terminal_seen || observed != expected {
            return Err(ProtocolError::WrongTerminalEvidence);
        }
        let trigger_present = match expected {
            SupervisorTerminalV1::StdoutOverflow => {
                self.automaton.index == 3
                    && self.stdout_bytes == RESOURCE_PROFILE_V1.per_role_output_bytes + 1
            }
            SupervisorTerminalV1::TrailingControl => {
                self.automaton.index == 4 && self.trailing_control_bytes == 1
            }
            SupervisorTerminalV1::LateStdout => {
                self.automaton.index == 5
                    && self.role_terminal_seen
                    && self.collector_to_supervisor_eof
                    && self.stdout_bytes == 1
            }
            SupervisorTerminalV1::PostTerminalLinkCount => {
                self.automaton.index == 4 && self.role_terminal_seen
            }
            SupervisorTerminalV1::WrongDevicePreStart
            | SupervisorTerminalV1::WritableMountPreStart
            | SupervisorTerminalV1::InheritedFdPostExec
            | SupervisorTerminalV1::InheritedSocketPostExec => {
                self.automaton.index == 0 && !self.role_terminal_seen
            }
        };
        if !trigger_present {
            return Err(ProtocolError::MissingTerminalEvidence);
        }
        self.supervisor_terminal_seen = true;
        Ok(())
    }

    fn accept_control_eof_live(&mut self, direction: Direction) -> Result<(), ProtocolError> {
        if self.control_eof_seen(direction) {
            return Err(ProtocolError::DuplicateEof);
        }
        if !self.control_eof_allowed(direction) {
            return Err(ProtocolError::MissingTerminalEvidence);
        }
        self.automaton.accept(DirectedControlEvent {
            direction,
            event: ControlEvent::Eof,
        })?;
        match direction {
            Direction::SupervisorToCollector => self.supervisor_to_collector_eof = true,
            Direction::CollectorToSupervisor => self.collector_to_supervisor_eof = true,
        }
        Ok(())
    }

    fn accept_stdout_eof_live(&mut self) -> Result<(), ProtocolError> {
        if self.stdout_eof {
            return Err(ProtocolError::DuplicateEof);
        }
        if !self.terminal_evidence_complete() {
            return Err(ProtocolError::MissingTerminalEvidence);
        }
        self.stdout_eof = true;
        Ok(())
    }

    fn accept_stderr_eof_live(&mut self) -> Result<(), ProtocolError> {
        if self.stderr_eof {
            return Err(ProtocolError::DuplicateEof);
        }
        if !self.terminal_evidence_complete() {
            return Err(ProtocolError::MissingTerminalEvidence);
        }
        self.stderr_eof = true;
        Ok(())
    }

    pub fn is_complete(&self) -> bool {
        !self.poisoned
            && self.automaton.is_complete()
            && self.stdout_eof
            && self.stderr_eof
            && self.output_observation_complete()
            && self.terminal_evidence_complete()
    }

    fn control_eof_seen(&self, direction: Direction) -> bool {
        match direction {
            Direction::SupervisorToCollector => self.supervisor_to_collector_eof,
            Direction::CollectorToSupervisor => self.collector_to_supervisor_eof,
        }
    }

    fn control_eof_allowed(&self, direction: Direction) -> bool {
        if self.abort_seen {
            return true;
        }
        if self.spec.output_policy == OutputPolicyV1::LateStdoutByte
            && direction == Direction::CollectorToSupervisor
        {
            return self.role_terminal_seen;
        }
        self.terminal_evidence_complete()
    }

    fn terminal_evidence_complete(&self) -> bool {
        if self.abort_seen {
            return true;
        }
        match self.spec.completion {
            RoleCompletionV1::RoleTerminal => self.role_terminal_seen,
            RoleCompletionV1::KernelTerminal => self.kernel_terminal_seen,
            RoleCompletionV1::SupervisorTerminal(_) => self.supervisor_terminal_seen,
        }
    }

    fn output_observation_complete(&self) -> bool {
        if self.abort_seen {
            return self.stderr_bytes == 0
                && self.trailing_control_bytes == 0
                && match self.spec.output_policy {
                    OutputPolicyV1::StdoutOverflow => {
                        self.stdout_bytes <= RESOURCE_PROFILE_V1.per_role_output_bytes
                    }
                    OutputPolicyV1::Empty
                    | OutputPolicyV1::TrailingControlByte
                    | OutputPolicyV1::LateStdoutByte => self.stdout_bytes == 0,
                };
        }
        match self.spec.output_policy {
            OutputPolicyV1::Empty => {
                self.stdout_bytes == 0 && self.stderr_bytes == 0 && self.trailing_control_bytes == 0
            }
            OutputPolicyV1::StdoutOverflow => {
                self.stdout_bytes == RESOURCE_PROFILE_V1.per_role_output_bytes + 1
                    && self.stderr_bytes == 0
                    && self.trailing_control_bytes == 0
            }
            OutputPolicyV1::TrailingControlByte => {
                self.stdout_bytes == 0 && self.stderr_bytes == 0 && self.trailing_control_bytes == 1
            }
            OutputPolicyV1::LateStdoutByte => {
                self.stdout_bytes == 1 && self.stderr_bytes == 0 && self.trailing_control_bytes == 0
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostDirection {
    HostToSupervisor,
    SupervisorToHost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostEvent {
    Frame {
        direction: HostDirection,
        kind: MessageKind,
        status: u16,
        measurement: Option<MeasurementIdentity>,
    },
    Eof(HostDirection),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HostState {
    Start,
    Ready,
    Measurements(u16),
    TerminalReady,
    Challenge,
    HostEof,
    Terminal,
    SupervisorEof,
    HostTerminalEof,
    AbortEof(HostDirection),
    Complete,
}

#[derive(Debug, PartialEq, Eq)]
pub struct HostAutomaton {
    entry: &'static ScheduleEntry,
    state: HostState,
    committed_status: u16,
    poisoned: bool,
}

impl HostAutomaton {
    pub const fn new(entry: &'static ScheduleEntry) -> Self {
        Self {
            entry,
            state: HostState::Start,
            committed_status: 0,
            poisoned: false,
        }
    }

    pub const fn is_complete(&self) -> bool {
        !self.poisoned && matches!(self.state, HostState::Complete)
    }

    pub fn accept(&mut self, event: HostEvent) -> Result<(), ProtocolError> {
        if self.poisoned {
            return Err(ProtocolError::Poisoned);
        }
        let result = self.accept_live(event);
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }

    fn accept_live(&mut self, event: HostEvent) -> Result<(), ProtocolError> {
        use HostDirection::{HostToSupervisor as Hs, SupervisorToHost as Sh};
        use MessageKind::{Challenge, HostReady, HostStart, Measurement, Terminal, TerminalReady};

        if let HostEvent::Frame {
            direction,
            kind: MessageKind::Abort,
            status,
            measurement: None,
        } = event
        {
            let expected_direction =
                host_abort_direction(self.state).ok_or(ProtocolError::OutOfState)?;
            let category =
                ErrorCategory::try_from(status).map_err(|_| ProtocolError::UnknownCategory)?;
            if direction != expected_direction || category == ErrorCategory::Success {
                return Err(ProtocolError::OutOfState);
            }
            self.state = HostState::AbortEof(direction);
            return Ok(());
        }

        let next = match self.state {
            HostState::Start => {
                require_host_frame(event, Hs, HostStart, 0, None)?;
                HostState::Ready
            }
            HostState::Ready => {
                require_host_frame(event, Sh, HostReady, 0, None)?;
                if crate::contract::measurement_count(self.entry) == 0 {
                    HostState::HostTerminalEof
                } else {
                    HostState::Measurements(0)
                }
            }
            HostState::Measurements(index) => {
                if let HostEvent::Frame {
                    direction: Sh,
                    kind: TerminalReady,
                    status,
                    measurement: None,
                } = event
                {
                    let category = ErrorCategory::try_from(status)
                        .map_err(|_| ProtocolError::UnknownCategory)?;
                    if category == ErrorCategory::Success {
                        return Err(ProtocolError::OutOfState);
                    }
                    self.committed_status = status;
                    self.state = HostState::Challenge;
                    return Ok(());
                }
                let expected =
                    expected_measurement(self.entry, index).ok_or(ProtocolError::OutOfState)?;
                require_host_frame(event, Sh, Measurement, 0, Some(expected))?;
                let next_index = index + 1;
                if expected_measurement(self.entry, next_index).is_some() {
                    HostState::Measurements(next_index)
                } else {
                    HostState::TerminalReady
                }
            }
            HostState::TerminalReady => {
                let HostEvent::Frame {
                    direction: Sh,
                    kind: TerminalReady,
                    status,
                    measurement: None,
                } = event
                else {
                    return Err(ProtocolError::OutOfState);
                };
                ErrorCategory::try_from(status).map_err(|_| ProtocolError::UnknownCategory)?;
                self.committed_status = status;
                HostState::Challenge
            }
            HostState::Challenge => {
                require_host_frame(event, Hs, Challenge, 0, None)?;
                HostState::HostEof
            }
            HostState::HostEof => {
                require_host_eof(event, Hs)?;
                HostState::Terminal
            }
            HostState::Terminal => {
                require_host_frame(event, Sh, Terminal, self.committed_status, None)?;
                HostState::SupervisorEof
            }
            HostState::SupervisorEof | HostState::HostTerminalEof => {
                require_host_eof(event, Sh)?;
                HostState::Complete
            }
            HostState::AbortEof(direction) => {
                require_host_eof(event, direction)?;
                HostState::Complete
            }
            HostState::Complete => return Err(ProtocolError::LateEvent),
        };
        self.state = next;
        Ok(())
    }
}

/// Pure verifier for one closed, manifest-bound host/supervisor instance channel.
#[derive(Debug, PartialEq, Eq)]
pub struct HostProtocolV1 {
    entry: &'static ScheduleEntry,
    selector: HostSelectorV1,
    selector_bytes: [u8; HostSelectorV1::ENCODED_LEN],
    binding: StreamBinding,
    host_to_supervisor: StreamVerifier,
    supervisor_to_host: StreamVerifier,
    automaton: HostAutomaton,
    host_start_seen: bool,
    host_ready_seen: bool,
    measurement_count: u16,
    committed_state: Option<TerminalStateV1>,
    challenge: Option<Digest32>,
    poisoned: bool,
}

impl HostProtocolV1 {
    pub fn new(run_nonce: Digest32, selector: HostSelectorV1) -> Result<Self, ProtocolError> {
        let entry = select_schedule_entry(
            selector.phase_code,
            selector.instance_class_code,
            selector.instance_ordinal,
        )
        .ok_or(ProtocolError::WrongItemBinding)?;
        let binding = StreamBinding::for_manifest(run_nonce, host_schedule_manifest_kind(entry))?;
        Self::new_bound(entry, selector, binding)
    }

    fn new_bound(
        entry: &'static ScheduleEntry,
        selector: HostSelectorV1,
        binding: StreamBinding,
    ) -> Result<Self, ProtocolError> {
        if HostSelectorV1::from_entry(entry) != selector
            || binding.contract_sha256 != CONTRACT_SHA256
        {
            return Err(ProtocolError::BindingMismatch);
        }
        if let Ok(expected_manifest) = frozen_manifest_sha256(host_schedule_manifest_kind(entry)) {
            if binding.manifest_sha256 != expected_manifest {
                return Err(ProtocolError::BindingMismatch);
            }
        }
        Ok(Self {
            entry,
            selector,
            selector_bytes: encode_host_selector(selector),
            binding,
            host_to_supervisor: StreamVerifier::new(binding),
            supervisor_to_host: StreamVerifier::new(binding),
            automaton: HostAutomaton::new(entry),
            host_start_seen: false,
            host_ready_seen: false,
            measurement_count: 0,
            committed_state: None,
            challenge: None,
            poisoned: false,
        })
    }

    /// Constructs the pure verifier from an explicit binding for cross-implementation tests.
    ///
    /// Production callers must use [`Self::new`], which requires a reviewed frozen manifest.
    #[cfg(test)]
    pub(crate) fn new_bound_for_test(
        entry: &'static ScheduleEntry,
        selector: HostSelectorV1,
        binding: StreamBinding,
    ) -> Result<Self, ProtocolError> {
        Self::new_bound(entry, selector, binding)
    }

    pub const fn entry(&self) -> &'static ScheduleEntry {
        self.entry
    }

    pub const fn selector(&self) -> HostSelectorV1 {
        self.selector
    }

    pub const fn binding(&self) -> StreamBinding {
        self.binding
    }

    pub const fn committed_state(&self) -> Option<TerminalStateV1> {
        self.committed_state
    }

    pub fn accept_frame(
        &mut self,
        direction: HostDirection,
        bytes: &[u8],
    ) -> Result<(), ProtocolError> {
        self.fail_closed(|state| state.accept_frame_live(direction, bytes))
    }

    fn accept_frame_live(
        &mut self,
        direction: HostDirection,
        bytes: &[u8],
    ) -> Result<(), ProtocolError> {
        let frame = match direction {
            HostDirection::HostToSupervisor => self.host_to_supervisor.accept(bytes)?,
            HostDirection::SupervisorToHost => self.supervisor_to_host.accept(bytes)?,
        };

        let measurement = match frame.header.kind {
            MessageKind::HostStart => {
                if direction != HostDirection::HostToSupervisor
                    || frame.payload() != self.selector_bytes
                {
                    return Err(ProtocolError::WrongItemBinding);
                }
                None
            }
            MessageKind::HostReady => {
                if direction != HostDirection::SupervisorToHost
                    || !self.host_start_seen
                    || frame.payload() != self.selector_bytes
                {
                    return Err(ProtocolError::WrongItemBinding);
                }
                None
            }
            MessageKind::Measurement => {
                if direction != HostDirection::SupervisorToHost || !self.host_ready_seen {
                    return Err(ProtocolError::WrongDirection);
                }
                let payload = decode_measurement(frame.payload())?;
                let scope_code = u16::try_from(payload.scope_code)
                    .map_err(|_| ProtocolError::InvalidMeasurement)?;
                let identity = MeasurementIdentity {
                    item_id: frame.header.case_or_probe_id,
                    subitem_id: frame.header.fixture_or_subattempt_id,
                    scope_code,
                    role_id: frame.header.role_id,
                };
                Some(identity)
            }
            MessageKind::TerminalReady => {
                if direction != HostDirection::SupervisorToHost || !self.host_ready_seen {
                    return Err(ProtocolError::WrongDirection);
                }
                let state = decode_terminal_state(frame.payload())?;
                self.validate_terminal_ready(frame.header.status, state)?;
                self.committed_state = Some(state);
                None
            }
            MessageKind::Challenge => {
                if direction != HostDirection::HostToSupervisor || self.committed_state.is_none() {
                    return Err(ProtocolError::WrongDirection);
                }
                let challenge = copy_digest(frame.payload(), 0);
                if challenge == self.binding.run_nonce {
                    return Err(ProtocolError::RandomnessReuse);
                }
                self.challenge = Some(challenge);
                None
            }
            MessageKind::Terminal => {
                if direction != HostDirection::SupervisorToHost {
                    return Err(ProtocolError::WrongDirection);
                }
                let commitment = TerminalCommitmentV1 {
                    challenge: self.challenge.ok_or(ProtocolError::ChallengeMismatch)?,
                    state: self
                        .committed_state
                        .ok_or(ProtocolError::CommittedStateMismatch)?,
                };
                commitment.verify_echo(frame.payload())?;
                None
            }
            MessageKind::Abort => {
                if self.committed_state.is_some() {
                    return Err(ProtocolError::OutOfState);
                }
                let state = decode_terminal_state(frame.payload())?;
                self.validate_abort_state(&frame, state)?;
                None
            }
            MessageKind::Start
            | MessageKind::Ready
            | MessageKind::Continue
            | MessageKind::WriterCommitted
            | MessageKind::LockObserved
            | MessageKind::ExpectedLockRejected
            | MessageKind::HandlesDropped
            | MessageKind::RoleTerminal => return Err(ProtocolError::OutOfState),
        };

        self.automaton.accept(HostEvent::Frame {
            direction,
            kind: frame.header.kind,
            status: frame.header.status,
            measurement,
        })?;
        match frame.header.kind {
            MessageKind::HostStart => self.host_start_seen = true,
            MessageKind::HostReady => self.host_ready_seen = true,
            MessageKind::Measurement => {
                self.measurement_count = self
                    .measurement_count
                    .checked_add(1)
                    .ok_or(ProtocolError::OutOfState)?;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn accept_eof(&mut self, direction: HostDirection) -> Result<(), ProtocolError> {
        self.fail_closed(|state| state.automaton.accept(HostEvent::Eof(direction)))
    }

    pub const fn is_complete(&self) -> bool {
        !self.poisoned && self.automaton.is_complete()
    }

    fn fail_closed<T>(
        &mut self,
        operation: impl FnOnce(&mut Self) -> Result<T, ProtocolError>,
    ) -> Result<T, ProtocolError> {
        if self.poisoned {
            return Err(ProtocolError::Poisoned);
        }
        let result = operation(self);
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }

    fn validate_terminal_ready(
        &self,
        status: u16,
        state: TerminalStateV1,
    ) -> Result<(), ProtocolError> {
        let category =
            ErrorCategory::try_from(status).map_err(|_| ProtocolError::UnknownCategory)?;
        if category == ErrorCategory::Success {
            state
                .validate_success_for(self.entry)
                .map_err(|_| ProtocolError::InvalidTerminalState)
        } else {
            self.validate_failure_progress(state, false)
        }
    }

    fn validate_abort_state(
        &self,
        frame: &Frame,
        state: TerminalStateV1,
    ) -> Result<(), ProtocolError> {
        if !self.host_ready_seen {
            if frame.header.case_or_probe_id != 0
                || frame.header.fixture_or_subattempt_id != 0
                || state.active_item_code != 0
                || state.write_may_have_been_dispatched != 0
                || state.writer_acknowledged != 0
                || state.completed_case_count != 0
                || state.completed_subattempt_count != 0
            {
                return Err(ProtocolError::WrongItemBinding);
            }
            return Ok(());
        }
        self.validate_failure_progress(state, true)
    }

    fn validate_failure_progress(
        &self,
        state: TerminalStateV1,
        allow_zero_before_item: bool,
    ) -> Result<(), ProtocolError> {
        if allow_zero_before_item
            && self.measurement_count == 0
            && state.active_item_code == 0
            && state.write_may_have_been_dispatched == 0
            && state.writer_acknowledged == 0
            && state.completed_case_count == 0
            && state.completed_subattempt_count == 0
        {
            return Ok(());
        }

        let (completed_items, item_count, measurements_per_item) = match self.entry.automaton {
            InstanceAutomaton::Characterization => {
                if state.completed_case_count != 0 {
                    return Err(ProtocolError::InvalidTerminalState);
                }
                (state.completed_subattempt_count, 3_u32, 7_u32)
            }
            InstanceAutomaton::Positive => {
                if state.completed_subattempt_count != 0 {
                    return Err(ProtocolError::InvalidTerminalState);
                }
                (state.completed_case_count, 20_u32, 7_u32)
            }
            InstanceAutomaton::SharedBoundary => {
                if state.completed_case_count != 0
                    || state.write_may_have_been_dispatched != 0
                    || state.writer_acknowledged != 0
                {
                    return Err(ProtocolError::InvalidTerminalState);
                }
                (state.completed_subattempt_count, 60_u32, 1_u32)
            }
            InstanceAutomaton::DedicatedBoundary => {
                if state.completed_case_count != 0
                    || state.write_may_have_been_dispatched != 0
                    || state.writer_acknowledged != 0
                {
                    return Err(ProtocolError::InvalidTerminalState);
                }
                (state.completed_subattempt_count, 1_u32, 1_u32)
            }
            InstanceAutomaton::HostTerminal => return Err(ProtocolError::InvalidTerminalState),
        };
        if completed_items >= item_count {
            return Err(ProtocolError::InvalidTerminalState);
        }
        let lower_bound = completed_items
            .checked_mul(measurements_per_item)
            .ok_or(ProtocolError::InvalidTerminalState)?;
        let upper_bound = completed_items
            .checked_add(1)
            .and_then(|value| value.checked_mul(measurements_per_item))
            .ok_or(ProtocolError::InvalidTerminalState)?;
        if !(lower_bound..=upper_bound).contains(&u32::from(self.measurement_count)) {
            return Err(ProtocolError::InvalidTerminalState);
        }
        let measurement_index =
            u16::try_from(lower_bound).map_err(|_| ProtocolError::InvalidTerminalState)?;
        let active = expected_measurement(self.entry, measurement_index)
            .ok_or(ProtocolError::InvalidTerminalState)?;
        if state.active_item_code != active_item_code(active.item_id, active.subitem_id) {
            return Err(ProtocolError::InvalidTerminalState);
        }
        Ok(())
    }
}

const fn host_abort_direction(state: HostState) -> Option<HostDirection> {
    match state {
        HostState::Start => Some(HostDirection::HostToSupervisor),
        HostState::Ready | HostState::Measurements(_) | HostState::TerminalReady => {
            Some(HostDirection::SupervisorToHost)
        }
        HostState::Challenge
        | HostState::HostEof
        | HostState::Terminal
        | HostState::SupervisorEof
        | HostState::HostTerminalEof
        | HostState::AbortEof(_)
        | HostState::Complete => None,
    }
}

fn require_host_frame(
    event: HostEvent,
    direction: HostDirection,
    kind: MessageKind,
    status: u16,
    measurement: Option<MeasurementIdentity>,
) -> Result<(), ProtocolError> {
    if event
        == (HostEvent::Frame {
            direction,
            kind,
            status,
            measurement,
        })
    {
        Ok(())
    } else {
        Err(ProtocolError::OutOfState)
    }
}

fn require_host_eof(event: HostEvent, direction: HostDirection) -> Result<(), ProtocolError> {
    if event == HostEvent::Eof(direction) {
        Ok(())
    } else {
        Err(ProtocolError::OutOfState)
    }
}

pub const LAUNCHER_CHILD_MAGIC: [u8; 8] = *b"ENGC2LC1";
pub const ROOT_TRIGGER_MAGIC: [u8; 8] = *b"ENGC2RT1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LauncherChildV1;

impl LauncherChildV1 {
    pub const ENCODED_LEN: usize = 24;
    pub const PROBE_ID: u16 = 39;
    pub const LAUNCHER_INNER_PID: u32 = 2;
    pub const COLLECTOR_INNER_PID: u32 = 3;
}

pub fn encode_launcher_child() -> [u8; LauncherChildV1::ENCODED_LEN] {
    let mut output = [0_u8; LauncherChildV1::ENCODED_LEN];
    output[..8].copy_from_slice(&LAUNCHER_CHILD_MAGIC);
    put_u16(&mut output, 8, PROTOCOL_VERSION);
    put_u16(&mut output, 10, LauncherChildV1::PROBE_ID);
    put_u32(&mut output, 12, LauncherChildV1::LAUNCHER_INNER_PID);
    put_u32(&mut output, 16, LauncherChildV1::COLLECTOR_INNER_PID);
    put_u32(&mut output, 20, 0);
    output
}

pub fn decode_launcher_child(bytes: &[u8]) -> Result<LauncherChildV1, ProtocolError> {
    if bytes.len() != LauncherChildV1::ENCODED_LEN {
        return Err(ProtocolError::WrongLength);
    }
    if bytes[..8] != LAUNCHER_CHILD_MAGIC
        || get_u16(bytes, 8) != PROTOCOL_VERSION
        || get_u16(bytes, 10) != LauncherChildV1::PROBE_ID
        || get_u32(bytes, 12) != LauncherChildV1::LAUNCHER_INNER_PID
        || get_u32(bytes, 16) != LauncherChildV1::COLLECTOR_INNER_PID
    {
        return Err(ProtocolError::BindingMismatch);
    }
    if get_u32(bytes, 20) != 0 {
        return Err(ProtocolError::ReservedFieldNonzero);
    }
    Ok(LauncherChildV1)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RootTriggerV1;

impl RootTriggerV1 {
    pub const ENCODED_LEN: usize = 16;
    pub const PROBE_ID: u16 = 39;
    pub const EVENT_CONTINUE_DELIVERED: u16 = 1;
}

pub fn encode_root_trigger() -> [u8; RootTriggerV1::ENCODED_LEN] {
    let mut output = [0_u8; RootTriggerV1::ENCODED_LEN];
    output[..8].copy_from_slice(&ROOT_TRIGGER_MAGIC);
    put_u16(&mut output, 8, PROTOCOL_VERSION);
    put_u16(&mut output, 10, RootTriggerV1::PROBE_ID);
    put_u16(&mut output, 12, RootTriggerV1::EVENT_CONTINUE_DELIVERED);
    put_u16(&mut output, 14, 0);
    output
}

pub fn decode_root_trigger(bytes: &[u8]) -> Result<RootTriggerV1, ProtocolError> {
    if bytes.len() != RootTriggerV1::ENCODED_LEN {
        return Err(ProtocolError::WrongLength);
    }
    if bytes[..8] != ROOT_TRIGGER_MAGIC
        || get_u16(bytes, 8) != PROTOCOL_VERSION
        || get_u16(bytes, 10) != RootTriggerV1::PROBE_ID
        || get_u16(bytes, 12) != RootTriggerV1::EVENT_CONTINUE_DELIVERED
    {
        return Err(ProtocolError::BindingMismatch);
    }
    if get_u16(bytes, 14) != 0 {
        return Err(ProtocolError::ReservedFieldNonzero);
    }
    Ok(RootTriggerV1)
}

fn copy_digest(bytes: &[u8], offset: usize) -> Digest32 {
    let mut digest = [0_u8; 32];
    digest.copy_from_slice(&bytes[offset..offset + 32]);
    Digest32(digest)
}

fn put_u16(output: &mut [u8], offset: usize, value: u16) {
    output[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}

fn put_u32(output: &mut [u8], offset: usize, value: u32) {
    output[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
}

fn put_u64(output: &mut [u8], offset: usize, value: u64) {
    output[offset..offset + 8].copy_from_slice(&value.to_be_bytes());
}

fn get_u16(input: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([input[offset], input[offset + 1]])
}

fn get_u32(input: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([
        input[offset],
        input[offset + 1],
        input[offset + 2],
        input[offset + 3],
    ])
}

fn get_u64(input: &[u8], offset: usize) -> u64 {
    u64::from_be_bytes([
        input[offset],
        input[offset + 1],
        input[offset + 2],
        input[offset + 3],
        input[offset + 4],
        input[offset + 5],
        input[offset + 6],
        input[offset + 7],
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{successful_terminal, ACCEPTANCE_SCHEDULE};

    const RUN_NONCE: Digest32 = Digest32([0x11; 32]);
    const TEST_MANIFEST: Digest32 = Digest32([0x22; 32]);

    const fn test_binding() -> StreamBinding {
        StreamBinding::unverified_for_codec(RUN_NONCE, TEST_MANIFEST)
    }

    fn role_frame(spec: RoleProtocolSpecV1, kind: MessageKind, sequence: u32) -> EncodedFrame {
        Frame::new(
            kind,
            sequence,
            spec.case_or_probe_id,
            spec.fixture_or_subattempt_id,
            spec.role_id,
            0,
            test_binding(),
            &[],
        )
        .unwrap()
        .encode()
    }

    fn accept_reader_prefix(protocol: &mut RoleProtocolV1) {
        let spec = protocol.spec();
        protocol
            .accept_control_frame(
                Direction::SupervisorToCollector,
                role_frame(spec, MessageKind::Start, 1).as_bytes(),
            )
            .unwrap();
        protocol
            .accept_control_frame(
                Direction::CollectorToSupervisor,
                role_frame(spec, MessageKind::Ready, 1).as_bytes(),
            )
            .unwrap();
        protocol
            .accept_control_frame(
                Direction::SupervisorToCollector,
                role_frame(spec, MessageKind::Continue, 2).as_bytes(),
            )
            .unwrap();
    }

    #[test]
    fn unified_role_protocol_requires_terminal_then_all_three_output_eofs() {
        let spec = derive_role_protocol_spec(RoleSelectionV1::Positive {
            case_id: 1,
            role_id: RoleId::Reader,
        })
        .unwrap();

        let mut wrong_order = RoleProtocolV1::new_bound(spec, test_binding()).unwrap();
        assert_eq!(
            wrong_order.accept_control_frame(
                Direction::CollectorToSupervisor,
                role_frame(spec, MessageKind::Ready, 1).as_bytes(),
            ),
            Err(ProtocolError::OutOfState)
        );
        assert_eq!(
            wrong_order.accept_control_frame(
                Direction::SupervisorToCollector,
                role_frame(spec, MessageKind::Start, 1).as_bytes(),
            ),
            Err(ProtocolError::Poisoned)
        );
        assert!(!wrong_order.is_complete());

        let mut early_eof = RoleProtocolV1::new_bound(spec, test_binding()).unwrap();
        accept_reader_prefix(&mut early_eof);
        assert_eq!(
            early_eof.accept_stdout_eof(),
            Err(ProtocolError::MissingTerminalEvidence)
        );
        assert_eq!(
            early_eof.accept_control_frame(
                Direction::CollectorToSupervisor,
                role_frame(spec, MessageKind::RoleTerminal, 2).as_bytes(),
            ),
            Err(ProtocolError::Poisoned)
        );
        assert!(!early_eof.is_complete());

        let mut unexpected_output = RoleProtocolV1::new_bound(spec, test_binding()).unwrap();
        accept_reader_prefix(&mut unexpected_output);
        unexpected_output
            .accept_control_frame(
                Direction::CollectorToSupervisor,
                role_frame(spec, MessageKind::RoleTerminal, 2).as_bytes(),
            )
            .unwrap();
        assert_eq!(
            unexpected_output.accept_stdout(&[1]),
            Err(ProtocolError::UnexpectedOutput)
        );
        assert_eq!(
            unexpected_output.accept_stderr_eof(),
            Err(ProtocolError::Poisoned)
        );
        assert!(!unexpected_output.is_complete());

        let mut protocol = RoleProtocolV1::new_bound(spec, test_binding()).unwrap();
        accept_reader_prefix(&mut protocol);
        protocol
            .accept_control_frame(
                Direction::CollectorToSupervisor,
                role_frame(spec, MessageKind::RoleTerminal, 2).as_bytes(),
            )
            .unwrap();
        protocol.accept_stderr_eof().unwrap();
        protocol.accept_stdout_eof().unwrap();
        protocol
            .accept_control_eof(Direction::CollectorToSupervisor)
            .unwrap();
        protocol
            .accept_control_eof(Direction::SupervisorToCollector)
            .unwrap();
        assert!(protocol.is_complete());
        assert_eq!(
            protocol.accept_stdout_eof(),
            Err(ProtocolError::DuplicateEof)
        );
        assert!(!protocol.is_complete());
        assert_eq!(protocol.accept_stderr_eof(), Err(ProtocolError::Poisoned));
    }

    #[test]
    fn unified_role_protocol_observes_exact_overflow_and_late_byte_boundaries() {
        let overflow_spec = derive_role_protocol_spec(RoleSelectionV1::Boundary {
            probe_id: 9,
            subattempt_id: 1,
        })
        .unwrap();
        for prefix_len in 0..3 {
            let mut too_early = RoleProtocolV1::new_bound(overflow_spec, test_binding()).unwrap();
            if prefix_len >= 1 {
                too_early
                    .accept_control_frame(
                        Direction::SupervisorToCollector,
                        role_frame(overflow_spec, MessageKind::Start, 1).as_bytes(),
                    )
                    .unwrap();
            }
            if prefix_len >= 2 {
                too_early
                    .accept_control_frame(
                        Direction::CollectorToSupervisor,
                        role_frame(overflow_spec, MessageKind::Ready, 1).as_bytes(),
                    )
                    .unwrap();
            }
            assert_eq!(
                too_early.accept_stdout(&[0]),
                Err(ProtocolError::UnexpectedOutput)
            );
            assert_eq!(
                too_early.accept_control_frame(
                    Direction::SupervisorToCollector,
                    role_frame(
                        overflow_spec,
                        if prefix_len == 0 {
                            MessageKind::Start
                        } else {
                            MessageKind::Continue
                        },
                        if prefix_len == 0 { 1 } else { 2 },
                    )
                    .as_bytes(),
                ),
                Err(ProtocolError::Poisoned)
            );
            assert!(!too_early.is_complete());
        }

        let mut premature = RoleProtocolV1::new_bound(overflow_spec, test_binding()).unwrap();
        accept_reader_prefix(&mut premature);
        premature
            .accept_stdout(&vec![0; RESOURCE_PROFILE_V1.per_role_output_bytes as usize])
            .unwrap();
        assert_eq!(
            premature.observe_supervisor_terminal(SupervisorTerminalV1::StdoutOverflow),
            Err(ProtocolError::MissingTerminalEvidence)
        );
        assert_eq!(premature.accept_stdout(&[0]), Err(ProtocolError::Poisoned));
        assert!(!premature.is_complete());

        let mut overflow = RoleProtocolV1::new_bound(overflow_spec, test_binding()).unwrap();
        accept_reader_prefix(&mut overflow);
        overflow
            .accept_stdout(&vec![0; RESOURCE_PROFILE_V1.per_role_output_bytes as usize])
            .unwrap();
        overflow.accept_stdout(&[0]).unwrap();
        overflow
            .observe_supervisor_terminal(SupervisorTerminalV1::StdoutOverflow)
            .unwrap();
        overflow
            .accept_control_eof(Direction::SupervisorToCollector)
            .unwrap();
        overflow
            .accept_control_eof(Direction::CollectorToSupervisor)
            .unwrap();
        overflow.accept_stdout_eof().unwrap();
        overflow.accept_stderr_eof().unwrap();
        assert_eq!(overflow.stdout_observed_bytes(), 65_537);
        assert_eq!(overflow.stdout_retained_bytes(), 65_536);
        assert!(overflow.is_complete());

        let mut excess = RoleProtocolV1::new_bound(overflow_spec, test_binding()).unwrap();
        accept_reader_prefix(&mut excess);
        excess
            .accept_stdout(&vec![
                0;
                RESOURCE_PROFILE_V1.per_role_output_bytes as usize + 1
            ])
            .unwrap();
        assert_eq!(
            excess.accept_stdout(&[0]),
            Err(ProtocolError::OutputLimitExceeded)
        );
        assert_eq!(
            excess.observe_supervisor_terminal(SupervisorTerminalV1::StdoutOverflow),
            Err(ProtocolError::Poisoned)
        );
        assert!(!excess.is_complete());

        let late_spec = derive_role_protocol_spec(RoleSelectionV1::Boundary {
            probe_id: 11,
            subattempt_id: 1,
        })
        .unwrap();
        let mut premature_late = RoleProtocolV1::new_bound(late_spec, test_binding()).unwrap();
        accept_reader_prefix(&mut premature_late);
        premature_late
            .accept_control_frame(
                Direction::CollectorToSupervisor,
                role_frame(late_spec, MessageKind::RoleTerminal, 2).as_bytes(),
            )
            .unwrap();
        assert_eq!(
            premature_late.accept_stdout(&[0]),
            Err(ProtocolError::UnexpectedOutput)
        );
        assert_eq!(
            premature_late.accept_control_eof(Direction::CollectorToSupervisor),
            Err(ProtocolError::Poisoned)
        );
        assert!(!premature_late.is_complete());

        let mut late = RoleProtocolV1::new_bound(late_spec, test_binding()).unwrap();
        accept_reader_prefix(&mut late);
        late.accept_control_frame(
            Direction::CollectorToSupervisor,
            role_frame(late_spec, MessageKind::RoleTerminal, 2).as_bytes(),
        )
        .unwrap();
        late.accept_control_eof(Direction::CollectorToSupervisor)
            .unwrap();
        late.accept_stdout(&[0]).unwrap();
        late.observe_supervisor_terminal(SupervisorTerminalV1::LateStdout)
            .unwrap();
        late.accept_control_eof(Direction::SupervisorToCollector)
            .unwrap();
        late.accept_stdout_eof().unwrap();
        late.accept_stderr_eof().unwrap();
        assert!(late.is_complete());
    }

    fn host_frame(
        binding: StreamBinding,
        kind: MessageKind,
        sequence: u32,
        status: u16,
        payload: &[u8],
        identity: Option<MeasurementIdentity>,
    ) -> EncodedFrame {
        let (item, subitem, role) = identity
            .map(|value| (value.item_id, value.subitem_id, value.role_id))
            .unwrap_or((0, 0, RoleId::Supervisor));
        Frame::new(
            kind, sequence, item, subitem, role, status, binding, payload,
        )
        .unwrap()
        .encode()
    }

    fn host_protocol_after_ready(entry: &'static ScheduleEntry) -> HostProtocolV1 {
        let selector = HostSelectorV1::from_entry(entry);
        let binding = test_binding();
        let mut protocol = HostProtocolV1::new_bound_for_test(entry, selector, binding).unwrap();
        protocol
            .accept_frame(
                HostDirection::HostToSupervisor,
                host_frame(
                    binding,
                    MessageKind::HostStart,
                    1,
                    0,
                    &encode_host_selector(selector),
                    None,
                )
                .as_bytes(),
            )
            .unwrap();
        protocol
            .accept_frame(
                HostDirection::SupervisorToHost,
                host_frame(
                    binding,
                    MessageKind::HostReady,
                    1,
                    0,
                    &encode_host_selector(selector),
                    None,
                )
                .as_bytes(),
            )
            .unwrap();
        protocol
    }

    fn accept_measurement_prefix(
        protocol: &mut HostProtocolV1,
        entry: &'static ScheduleEntry,
        count: u16,
    ) {
        for index in 0..count {
            let identity = expected_measurement(entry, index).unwrap();
            let measurement = MeasurementV1 {
                scope_code: u64::from(identity.scope_code),
                ..MeasurementV1::default()
            };
            protocol
                .accept_frame(
                    HostDirection::SupervisorToHost,
                    host_frame(
                        test_binding(),
                        MessageKind::Measurement,
                        u32::from(index) + 2,
                        0,
                        &encode_measurement(measurement),
                        Some(identity),
                    )
                    .as_bytes(),
                )
                .unwrap();
        }
    }

    #[test]
    fn unified_host_protocol_binds_selector_measurement_and_terminal_echo() {
        let entry = &ACCEPTANCE_SCHEDULE[4];
        let selector = HostSelectorV1::from_entry(entry);
        let selector_bytes = encode_host_selector(selector);
        let binding = test_binding();
        let mut protocol = HostProtocolV1::new_bound(entry, selector, binding).unwrap();
        protocol
            .accept_frame(
                HostDirection::HostToSupervisor,
                host_frame(binding, MessageKind::HostStart, 1, 0, &selector_bytes, None).as_bytes(),
            )
            .unwrap();
        protocol
            .accept_frame(
                HostDirection::SupervisorToHost,
                host_frame(binding, MessageKind::HostReady, 1, 0, &selector_bytes, None).as_bytes(),
            )
            .unwrap();

        let identity = expected_measurement(entry, 0).unwrap();
        let measurement = MeasurementV1 {
            scope_code: u64::from(identity.scope_code),
            ..MeasurementV1::default()
        };
        protocol
            .accept_frame(
                HostDirection::SupervisorToHost,
                host_frame(
                    binding,
                    MessageKind::Measurement,
                    2,
                    0,
                    &encode_measurement(measurement),
                    Some(identity),
                )
                .as_bytes(),
            )
            .unwrap();
        let state = successful_terminal(entry).unwrap();
        protocol
            .accept_frame(
                HostDirection::SupervisorToHost,
                host_frame(
                    binding,
                    MessageKind::TerminalReady,
                    3,
                    0,
                    &encode_terminal_state(state),
                    None,
                )
                .as_bytes(),
            )
            .unwrap();
        let challenge = Digest32([0x33; 32]);
        protocol
            .accept_frame(
                HostDirection::HostToSupervisor,
                host_frame(
                    binding,
                    MessageKind::Challenge,
                    2,
                    0,
                    challenge.as_bytes(),
                    None,
                )
                .as_bytes(),
            )
            .unwrap();
        protocol
            .accept_eof(HostDirection::HostToSupervisor)
            .unwrap();
        protocol
            .accept_frame(
                HostDirection::SupervisorToHost,
                host_frame(
                    binding,
                    MessageKind::Terminal,
                    4,
                    0,
                    &encode_terminal_payload(challenge, state),
                    None,
                )
                .as_bytes(),
            )
            .unwrap();
        protocol
            .accept_eof(HostDirection::SupervisorToHost)
            .unwrap();
        assert_eq!(protocol.committed_state(), Some(state));
        assert!(protocol.is_complete());
    }

    #[test]
    fn unified_host_protocol_distinguishes_abort_and_failure_progress_boundaries() {
        let entry = &ACCEPTANCE_SCHEDULE[0];
        let zero_item_failure = TerminalStateV1 {
            primary_category: ErrorCategory::ProtocolFailure as u32,
            guest_cleanup_proven: 1,
            roles_reaped: 1,
            ..TerminalStateV1::default()
        };

        let mut terminal_ready_without_item = host_protocol_after_ready(entry);
        assert_eq!(
            terminal_ready_without_item.accept_frame(
                HostDirection::SupervisorToHost,
                host_frame(
                    test_binding(),
                    MessageKind::TerminalReady,
                    2,
                    ErrorCategory::ProtocolFailure.header_code(),
                    &encode_terminal_state(zero_item_failure),
                    None,
                )
                .as_bytes(),
            ),
            Err(ProtocolError::InvalidTerminalState)
        );

        let mut abort_before_first_item = host_protocol_after_ready(entry);
        abort_before_first_item
            .accept_frame(
                HostDirection::SupervisorToHost,
                host_frame(
                    test_binding(),
                    MessageKind::Abort,
                    2,
                    ErrorCategory::ProtocolFailure.header_code(),
                    &encode_terminal_state(zero_item_failure),
                    None,
                )
                .as_bytes(),
            )
            .unwrap();
        abort_before_first_item
            .accept_eof(HostDirection::SupervisorToHost)
            .unwrap();
        assert!(abort_before_first_item.is_complete());

        for state in [
            TerminalStateV1 {
                primary_category: ErrorCategory::ProtocolFailure as u32,
                active_item_code: active_item_code(1, 1),
                guest_cleanup_proven: 1,
                roles_reaped: 1,
                completed_case_count: 0,
                ..TerminalStateV1::default()
            },
            TerminalStateV1 {
                primary_category: ErrorCategory::ProtocolFailure as u32,
                active_item_code: active_item_code(2, 2),
                guest_cleanup_proven: 1,
                roles_reaped: 1,
                completed_case_count: 1,
                ..TerminalStateV1::default()
            },
        ] {
            let mut protocol = host_protocol_after_ready(entry);
            accept_measurement_prefix(&mut protocol, entry, 7);
            protocol
                .accept_frame(
                    HostDirection::SupervisorToHost,
                    host_frame(
                        test_binding(),
                        MessageKind::TerminalReady,
                        9,
                        ErrorCategory::ProtocolFailure.header_code(),
                        &encode_terminal_state(state),
                        None,
                    )
                    .as_bytes(),
                )
                .unwrap();
            assert_eq!(protocol.committed_state(), Some(state));
        }
    }

    #[test]
    fn unified_host_protocol_rejects_a_nonidentical_ready_selector() {
        let entry = &ACCEPTANCE_SCHEDULE[4];
        let selector = HostSelectorV1::from_entry(entry);
        let binding = test_binding();
        let mut protocol = HostProtocolV1::new_bound(entry, selector, binding).unwrap();
        protocol
            .accept_frame(
                HostDirection::HostToSupervisor,
                host_frame(
                    binding,
                    MessageKind::HostStart,
                    1,
                    0,
                    &encode_host_selector(selector),
                    None,
                )
                .as_bytes(),
            )
            .unwrap();
        let wrong_selector =
            encode_host_selector(HostSelectorV1::from_entry(&ACCEPTANCE_SCHEDULE[5]));
        assert_eq!(
            protocol.accept_frame(
                HostDirection::SupervisorToHost,
                host_frame(binding, MessageKind::HostReady, 1, 0, &wrong_selector, None,)
                    .as_bytes(),
            ),
            Err(ProtocolError::WrongItemBinding)
        );
        for sequence in [1, 2] {
            assert_eq!(
                protocol.accept_frame(
                    HostDirection::SupervisorToHost,
                    host_frame(
                        binding,
                        MessageKind::HostReady,
                        sequence,
                        0,
                        &encode_host_selector(selector),
                        None,
                    )
                    .as_bytes(),
                ),
                Err(ProtocolError::Poisoned)
            );
        }
        assert!(!protocol.is_complete());
    }
}
