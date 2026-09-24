//! Private C1 synthetic JSON-RPC grammar and sequence automaton.
//!
//! This module deliberately has no production constructor or external-I/O boundary. It accepts
//! injected bytes only and returns fixed local bytes or inert syntax evidence.

use crate::native_document::{strict_json_value_from_slice_categorized, StrictJsonReadError};
use crate::native_successor_policy::{C1RequestStringId, C1Sha256};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::marker::PhantomData;
use std::rc::Rc;

const MAX_MESSAGE_BYTES: usize = 65_536;
const MAX_INTEGER_ID: u64 = 9_007_199_254_740_991;
const JSON_RPC_VERSION: &str = "2.0";
const INITIALIZE_METHOD: &str = "initialize";
const INITIALIZED_METHOD: &str = "notifications/initialized";
const TOOLS_LIST_METHOD: &str = "tools/list";
const TOOLS_CALL_METHOD: &str = "tools/call";
const READ_TOOL_NAME: &str = "c1_synthetic_read";
const MUTATION_TOOL_NAME: &str = "c1_synthetic_mutation";
const ARGUMENTS_CANONICAL_JSON: &[u8] = br#"{"nonce":"fixture"}"#;
const ARGUMENTS_DIGEST_DOMAIN: &[u8] = b"engram-c1-synthetic-arguments-v1\0";
const LEDGER_DIGEST_DOMAIN: &[u8] = b"engram-c1-synthetic-relay-v1\0";

const INITIALIZE_RESPONSE_SUFFIX: &str = concat!(
    ",\"result\":{\"protocolVersion\":\"2024-11-05\",",
    "\"capabilities\":{\"tools\":{}},",
    "\"serverInfo\":{\"name\":\"engram-c1-synthetic\",\"version\":\"1\"}}}"
);

#[cfg(test)]
const READ_TOOL_DEFINITION: &str = concat!(
    "{\"name\":\"c1_synthetic_read\",",
    "\"description\":\"Synthetic read fixture; performs no operation.\",",
    "\"inputSchema\":{\"type\":\"object\",",
    "\"properties\":{\"nonce\":{\"type\":\"string\",\"const\":\"fixture\"}},",
    "\"required\":[\"nonce\"],\"additionalProperties\":false}}"
);

#[cfg(test)]
const MUTATION_TOOL_DEFINITION: &str = concat!(
    "{\"name\":\"c1_synthetic_mutation\",",
    "\"description\":\"Synthetic mutation fixture; performs no operation.\",",
    "\"inputSchema\":{\"type\":\"object\",",
    "\"properties\":{\"nonce\":{\"type\":\"string\",\"const\":\"fixture\"}},",
    "\"required\":[\"nonce\"],\"additionalProperties\":false}}"
);

const TOOLS_LIST_RESPONSE_SUFFIX: &str = concat!(
    ",\"result\":{\"tools\":[",
    "{\"name\":\"c1_synthetic_read\",",
    "\"description\":\"Synthetic read fixture; performs no operation.\",",
    "\"inputSchema\":{\"type\":\"object\",",
    "\"properties\":{\"nonce\":{\"type\":\"string\",\"const\":\"fixture\"}},",
    "\"required\":[\"nonce\"],\"additionalProperties\":false}},",
    "{\"name\":\"c1_synthetic_mutation\",",
    "\"description\":\"Synthetic mutation fixture; performs no operation.\",",
    "\"inputSchema\":{\"type\":\"object\",",
    "\"properties\":{\"nonce\":{\"type\":\"string\",\"const\":\"fixture\"}},",
    "\"required\":[\"nonce\"],\"additionalProperties\":false}}]}}"
);

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum C1SyntheticProgram {
    None,
    SingleRead,
    SingleMutation,
    MutationThenRead,
}

impl C1SyntheticProgram {
    fn spelling(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::SingleRead => "single_read",
            Self::SingleMutation => "single_mutation",
            Self::MutationThenRead => "mutation_then_read",
        }
    }

    fn calls(self) -> &'static [C1SyntheticTool] {
        const NONE: &[C1SyntheticTool] = &[];
        const SINGLE_READ: &[C1SyntheticTool] = &[C1SyntheticTool::Read];
        const SINGLE_MUTATION: &[C1SyntheticTool] = &[C1SyntheticTool::Mutation];
        const MUTATION_THEN_READ: &[C1SyntheticTool] =
            &[C1SyntheticTool::Mutation, C1SyntheticTool::Read];

        match self {
            Self::None => NONE,
            Self::SingleRead => SINGLE_READ,
            Self::SingleMutation => SINGLE_MUTATION,
            Self::MutationThenRead => MUTATION_THEN_READ,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum C1SyntheticTool {
    Read,
    Mutation,
}

impl C1SyntheticTool {
    fn name(self) -> &'static str {
        match self {
            Self::Read => READ_TOOL_NAME,
            Self::Mutation => MUTATION_TOOL_NAME,
        }
    }

    fn ledger_tag(self) -> u8 {
        match self {
            Self::Read => 3,
            Self::Mutation => 4,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
enum C1RequestId {
    Integer(u64),
    String(C1RequestStringId),
}

impl C1RequestId {
    fn parse(value: Option<&Value>) -> Option<Self> {
        match value? {
            Value::Number(number) => {
                let value = number.as_u64()?;
                (1..=MAX_INTEGER_ID)
                    .contains(&value)
                    .then_some(Self::Integer(value))
            }
            Value::String(value) => C1RequestStringId::parse(value).map(Self::String),
            _ => None,
        }
    }

    fn append_canonical_json(&self, output: &mut Vec<u8>) {
        match self {
            Self::Integer(value) => output.extend_from_slice(value.to_string().as_bytes()),
            Self::String(value) => {
                output.push(b'"');
                output.extend_from_slice(value.as_str().as_bytes());
                output.push(b'"');
            }
        }
    }

    fn append_ledger_encoding(&self, output: &mut Vec<u8>) {
        match self {
            Self::Integer(value) => {
                output.push(1);
                output.extend_from_slice(&value.to_be_bytes());
            }
            Self::String(value) => {
                output.push(2);
                append_len_prefixed(output, value.as_str().as_bytes());
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum C1RelayRejectionCode {
    AlreadyPoisoned,
    Oversized,
    MalformedJson,
    DuplicateKey,
    TrailingData,
    BatchForbidden,
    MetaForbidden,
    InvalidJsonrpc,
    InvalidId,
    ReusedId,
    UnexpectedMessage,
    ExtraField,
    ContractMismatch,
    IncompleteProgram,
}

impl C1RelayRejectionCode {
    #[cfg(test)]
    fn as_str(self) -> &'static str {
        match self {
            Self::AlreadyPoisoned => "already_poisoned",
            Self::Oversized => "oversized",
            Self::MalformedJson => "malformed_json",
            Self::DuplicateKey => "duplicate_key",
            Self::TrailingData => "trailing_data",
            Self::BatchForbidden => "batch_forbidden",
            Self::MetaForbidden => "meta_forbidden",
            Self::InvalidJsonrpc => "invalid_jsonrpc",
            Self::InvalidId => "invalid_id",
            Self::ReusedId => "reused_id",
            Self::UnexpectedMessage => "unexpected_message",
            Self::ExtraField => "extra_field",
            Self::ContractMismatch => "contract_mismatch",
            Self::IncompleteProgram => "incomplete_program",
        }
    }
}

#[allow(dead_code)]
enum C1RelayDecision {
    LocalResponse {
        bytes: Vec<u8>,
    },
    NotificationAccepted,
    SyntheticCallAccepted {
        request_id: C1RequestId,
        ordinal: u64,
        tool: C1SyntheticTool,
        arguments_sha256: C1Sha256,
    },
    SequenceAccepted {
        ledger_sha256: C1Sha256,
    },
    Rejected {
        code: C1RelayRejectionCode,
    },
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum C1RelayState {
    AwaitInitialize,
    AwaitInitialized,
    AwaitToolsList,
    AwaitSyntheticCalls,
    Poisoned,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum C1MessageKind {
    Initialize,
    Initialized,
    ToolsList,
    ToolsCall,
}

struct C1LedgerRecord {
    tag: u8,
    request_id: Option<C1RequestId>,
    method: &'static str,
    tool: Option<C1SyntheticTool>,
    arguments_sha256: Option<C1Sha256>,
}

/// The marker makes the automaton single-owner even if all other fields later become thread-safe.
#[allow(dead_code)]
struct C1SyntheticRelay {
    program: C1SyntheticProgram,
    state: C1RelayState,
    accepted_call_count: usize,
    seen_ids: Vec<C1RequestId>,
    ledger: Vec<C1LedgerRecord>,
    _single_owner: PhantomData<Rc<()>>,
}

#[allow(dead_code)]
impl C1SyntheticRelay {
    #[cfg(test)]
    fn fixture(program: C1SyntheticProgram) -> Self {
        Self {
            program,
            state: C1RelayState::AwaitInitialize,
            accepted_call_count: 0,
            seen_ids: Vec::new(),
            ledger: Vec::new(),
            _single_owner: PhantomData,
        }
    }

    fn accept(&mut self, bytes: &[u8]) -> C1RelayDecision {
        if self.state == C1RelayState::Poisoned {
            return C1RelayDecision::Rejected {
                code: C1RelayRejectionCode::AlreadyPoisoned,
            };
        }
        if bytes.len() > MAX_MESSAGE_BYTES {
            return self.reject(C1RelayRejectionCode::Oversized);
        }

        let value = match strict_json_value_from_slice_categorized(bytes) {
            Ok(value) => value,
            Err(StrictJsonReadError::MalformedJson) => {
                return self.reject(C1RelayRejectionCode::MalformedJson);
            }
            Err(StrictJsonReadError::DuplicateKey) => {
                return self.reject(C1RelayRejectionCode::DuplicateKey);
            }
            Err(StrictJsonReadError::TrailingData) => {
                return self.reject(C1RelayRejectionCode::TrailingData);
            }
        };

        if value.is_array() {
            return self.reject(C1RelayRejectionCode::BatchForbidden);
        }
        let Some(object) = value.as_object() else {
            return self.reject(C1RelayRejectionCode::UnexpectedMessage);
        };
        if contains_meta(&value) {
            return self.reject(C1RelayRejectionCode::MetaForbidden);
        }
        if object.get("jsonrpc").and_then(Value::as_str) != Some(JSON_RPC_VERSION) {
            return self.reject(C1RelayRejectionCode::InvalidJsonrpc);
        }

        let Some(method) = object.get("method").and_then(Value::as_str) else {
            return self.reject(C1RelayRejectionCode::UnexpectedMessage);
        };
        let Some(kind) = C1MessageKind::from_method(method) else {
            return self.reject(C1RelayRejectionCode::UnexpectedMessage);
        };

        if has_extra_field(kind, object) {
            return self.reject(C1RelayRejectionCode::ExtraField);
        }

        let request_id = if kind == C1MessageKind::Initialized {
            None
        } else {
            let Some(request_id) = C1RequestId::parse(object.get("id")) else {
                return self.reject(C1RelayRejectionCode::InvalidId);
            };
            if self.seen_ids.iter().any(|seen| seen == &request_id) {
                return self.reject(C1RelayRejectionCode::ReusedId);
            }
            Some(request_id)
        };

        if !self.state_accepts(kind) {
            return self.reject(C1RelayRejectionCode::UnexpectedMessage);
        }

        match kind {
            C1MessageKind::Initialize if !initialize_contract_matches(object) => {
                self.reject(C1RelayRejectionCode::ContractMismatch)
            }
            C1MessageKind::ToolsCall if !self.tool_call_contract_matches(object) => {
                self.reject(C1RelayRejectionCode::ContractMismatch)
            }
            C1MessageKind::Initialize => {
                let request_id = request_id.expect("request ID was validated");
                self.seen_ids.push(request_id.clone());
                self.ledger.push(C1LedgerRecord {
                    tag: 0,
                    request_id: Some(request_id.clone()),
                    method: INITIALIZE_METHOD,
                    tool: None,
                    arguments_sha256: None,
                });
                self.state = C1RelayState::AwaitInitialized;
                C1RelayDecision::LocalResponse {
                    bytes: local_response(&request_id, INITIALIZE_RESPONSE_SUFFIX),
                }
            }
            C1MessageKind::Initialized => {
                self.ledger.push(C1LedgerRecord {
                    tag: 1,
                    request_id: None,
                    method: INITIALIZED_METHOD,
                    tool: None,
                    arguments_sha256: None,
                });
                self.state = C1RelayState::AwaitToolsList;
                C1RelayDecision::NotificationAccepted
            }
            C1MessageKind::ToolsList => {
                let request_id = request_id.expect("request ID was validated");
                self.seen_ids.push(request_id.clone());
                self.ledger.push(C1LedgerRecord {
                    tag: 2,
                    request_id: Some(request_id.clone()),
                    method: TOOLS_LIST_METHOD,
                    tool: None,
                    arguments_sha256: None,
                });
                self.state = C1RelayState::AwaitSyntheticCalls;
                C1RelayDecision::LocalResponse {
                    bytes: local_response(&request_id, TOOLS_LIST_RESPONSE_SUFFIX),
                }
            }
            C1MessageKind::ToolsCall => {
                let request_id = request_id.expect("request ID was validated");
                let tool = self.program.calls()[self.accepted_call_count];
                let arguments_sha256 = canonical_arguments_sha256();
                self.accepted_call_count += 1;
                self.seen_ids.push(request_id.clone());
                self.ledger.push(C1LedgerRecord {
                    tag: tool.ledger_tag(),
                    request_id: Some(request_id.clone()),
                    method: TOOLS_CALL_METHOD,
                    tool: Some(tool),
                    arguments_sha256: Some(arguments_sha256.clone()),
                });
                C1RelayDecision::SyntheticCallAccepted {
                    request_id,
                    ordinal: u64::try_from(self.accepted_call_count)
                        .expect("synthetic program has at most two calls"),
                    tool,
                    arguments_sha256,
                }
            }
        }
    }

    fn finish(mut self) -> C1RelayDecision {
        if self.state == C1RelayState::Poisoned {
            return C1RelayDecision::Rejected {
                code: C1RelayRejectionCode::AlreadyPoisoned,
            };
        }
        if self.state != C1RelayState::AwaitSyntheticCalls
            || self.accepted_call_count != self.program.calls().len()
        {
            self.state = C1RelayState::Poisoned;
            return C1RelayDecision::Rejected {
                code: C1RelayRejectionCode::IncompleteProgram,
            };
        }
        C1RelayDecision::SequenceAccepted {
            ledger_sha256: self.ledger_sha256(),
        }
    }

    fn state_accepts(&self, kind: C1MessageKind) -> bool {
        match self.state {
            C1RelayState::AwaitInitialize => kind == C1MessageKind::Initialize,
            C1RelayState::AwaitInitialized => kind == C1MessageKind::Initialized,
            C1RelayState::AwaitToolsList => kind == C1MessageKind::ToolsList,
            C1RelayState::AwaitSyntheticCalls => {
                kind == C1MessageKind::ToolsCall
                    && self.accepted_call_count < self.program.calls().len()
            }
            C1RelayState::Poisoned => false,
        }
    }

    fn tool_call_contract_matches(&self, object: &Map<String, Value>) -> bool {
        let Some(expected_tool) = self.program.calls().get(self.accepted_call_count) else {
            return false;
        };
        let Some(params) = object.get("params").and_then(Value::as_object) else {
            return false;
        };
        params.get("name").and_then(Value::as_str) == Some(expected_tool.name())
            && params.get("arguments") == Some(&canonical_arguments_value())
    }

    fn reject(&mut self, code: C1RelayRejectionCode) -> C1RelayDecision {
        self.state = C1RelayState::Poisoned;
        C1RelayDecision::Rejected { code }
    }

    fn ledger_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(LEDGER_DIGEST_DOMAIN);
        append_len_prefixed(&mut bytes, self.program.spelling().as_bytes());
        append_u64(&mut bytes, self.ledger.len());
        for record in &self.ledger {
            bytes.push(record.tag);
            match &record.request_id {
                Some(request_id) => request_id.append_ledger_encoding(&mut bytes),
                None => bytes.push(0),
            }
            append_len_prefixed(&mut bytes, record.method.as_bytes());
            append_len_prefixed(
                &mut bytes,
                record
                    .tool
                    .map(C1SyntheticTool::name)
                    .unwrap_or("")
                    .as_bytes(),
            );
            match &record.arguments_sha256 {
                Some(digest) => {
                    bytes.push(1);
                    bytes.extend_from_slice(digest.as_digest_bytes());
                }
                None => bytes.push(0),
            }
        }
        bytes
    }

    fn ledger_sha256(&self) -> C1Sha256 {
        C1Sha256::from_digest_bytes(Sha256::digest(self.ledger_bytes()).into())
    }
}

impl C1MessageKind {
    fn from_method(method: &str) -> Option<Self> {
        match method {
            INITIALIZE_METHOD => Some(Self::Initialize),
            INITIALIZED_METHOD => Some(Self::Initialized),
            TOOLS_LIST_METHOD => Some(Self::ToolsList),
            TOOLS_CALL_METHOD => Some(Self::ToolsCall),
            _ => None,
        }
    }
}

fn initialize_contract_matches(object: &Map<String, Value>) -> bool {
    object.get("params")
        == Some(&serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "engram-c1-synthetic",
                "version": "1"
            }
        }))
}

fn canonical_arguments_value() -> Value {
    serde_json::json!({"nonce": "fixture"})
}

fn canonical_arguments_sha256() -> C1Sha256 {
    let mut hasher = Sha256::new();
    hasher.update(ARGUMENTS_DIGEST_DOMAIN);
    hasher.update(ARGUMENTS_CANONICAL_JSON);
    C1Sha256::from_digest_bytes(hasher.finalize().into())
}

fn local_response(request_id: &C1RequestId, suffix: &str) -> Vec<u8> {
    let mut bytes = br#"{"jsonrpc":"2.0","id":"#.to_vec();
    request_id.append_canonical_json(&mut bytes);
    bytes.extend_from_slice(suffix.as_bytes());
    bytes
}

fn contains_meta(value: &Value) -> bool {
    match value {
        Value::Object(object) => object.contains_key("_meta") || object.values().any(contains_meta),
        Value::Array(values) => values.iter().any(contains_meta),
        _ => false,
    }
}

fn has_extra_field(kind: C1MessageKind, object: &Map<String, Value>) -> bool {
    let allowed_top_level = match kind {
        C1MessageKind::Initialize | C1MessageKind::ToolsCall => {
            &["id", "jsonrpc", "method", "params"][..]
        }
        C1MessageKind::Initialized => &["jsonrpc", "method"][..],
        C1MessageKind::ToolsList => &["id", "jsonrpc", "method"][..],
    };
    if !has_only_fields(object, allowed_top_level) {
        return true;
    }

    match kind {
        C1MessageKind::Initialize => initialize_has_extra_field(object),
        C1MessageKind::ToolsCall => tool_call_has_extra_field(object),
        C1MessageKind::Initialized | C1MessageKind::ToolsList => false,
    }
}

fn initialize_has_extra_field(object: &Map<String, Value>) -> bool {
    let Some(params) = object.get("params").and_then(Value::as_object) else {
        return false;
    };
    if !has_only_fields(params, &["capabilities", "clientInfo", "protocolVersion"]) {
        return true;
    }
    if let Some(capabilities) = params.get("capabilities").and_then(Value::as_object) {
        if !capabilities.is_empty() {
            return true;
        }
    }
    if let Some(client_info) = params.get("clientInfo").and_then(Value::as_object) {
        if !has_only_fields(client_info, &["name", "version"]) {
            return true;
        }
    }
    false
}

fn tool_call_has_extra_field(object: &Map<String, Value>) -> bool {
    let Some(params) = object.get("params").and_then(Value::as_object) else {
        return false;
    };
    if !has_only_fields(params, &["arguments", "name"]) {
        return true;
    }
    if let Some(arguments) = params.get("arguments").and_then(Value::as_object) {
        if !has_only_fields(arguments, &["nonce"]) {
            return true;
        }
    }
    false
}

fn has_only_fields(object: &Map<String, Value>, allowed: &[&str]) -> bool {
    object.keys().all(|key| allowed.contains(&key.as_str()))
}

fn append_u64(output: &mut Vec<u8>, value: usize) {
    let value = u64::try_from(value).expect("C1 bounded collection length fits u64");
    output.extend_from_slice(&value.to_be_bytes());
}

fn append_len_prefixed(output: &mut Vec<u8>, value: &[u8]) {
    append_u64(output, value.len());
    output.extend_from_slice(value);
}

#[cfg(test)]
mod tests {
    use super::*;

    const _: fn() = || {
        trait AmbiguousIfSend<A: ?Sized> {
            fn prove() {}
        }
        impl<T: ?Sized> AmbiguousIfSend<()> for T {}
        #[allow(coherence_leak_check)]
        impl<T: ?Sized + Send> AmbiguousIfSend<dyn Send> for T {}
        let _ = <C1SyntheticRelay as AmbiguousIfSend<_>>::prove;
    };

    const _: fn() = || {
        trait AmbiguousIfSync<A: ?Sized> {
            fn prove() {}
        }
        impl<T: ?Sized> AmbiguousIfSync<()> for T {}
        #[allow(coherence_leak_check)]
        impl<T: ?Sized + Sync> AmbiguousIfSync<dyn Sync> for T {}
        let _ = <C1SyntheticRelay as AmbiguousIfSync<_>>::prove;
    };

    const INITIALIZE_1: &str = concat!(
        "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{",
        "\"protocolVersion\":\"2024-11-05\",\"capabilities\":{},",
        "\"clientInfo\":{\"name\":\"engram-c1-synthetic\",\"version\":\"1\"}}}"
    );
    const INITIALIZED: &str = "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}";

    fn initialize(id: &str) -> String {
        format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":{id},\"method\":\"initialize\",\"params\":{{\"protocolVersion\":\"2024-11-05\",\"capabilities\":{{}},\"clientInfo\":{{\"name\":\"engram-c1-synthetic\",\"version\":\"1\"}}}}}}"
        )
    }

    fn tools_list(id: &str) -> String {
        format!("{{\"jsonrpc\":\"2.0\",\"id\":{id},\"method\":\"tools/list\"}}")
    }

    fn tool_call(id: &str, tool: &str) -> String {
        format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":{id},\"method\":\"tools/call\",\"params\":{{\"name\":\"{tool}\",\"arguments\":{{\"nonce\":\"fixture\"}}}}}}"
        )
    }

    fn expect_local_response(decision: C1RelayDecision) -> Vec<u8> {
        match decision {
            C1RelayDecision::LocalResponse { bytes } => bytes,
            _ => panic!("expected local response"),
        }
    }

    fn expect_rejection(decision: C1RelayDecision, expected: C1RelayRejectionCode) {
        match decision {
            C1RelayDecision::Rejected { code } => assert_eq!(code.as_str(), expected.as_str()),
            _ => panic!("expected rejection"),
        }
    }

    fn accept_handshake(relay: &mut C1SyntheticRelay, initialize_id: &str, list_id: &str) {
        let _ = expect_local_response(relay.accept(initialize(initialize_id).as_bytes()));
        assert!(matches!(
            relay.accept(INITIALIZED.as_bytes()),
            C1RelayDecision::NotificationAccepted
        ));
        let _ = expect_local_response(relay.accept(tools_list(list_id).as_bytes()));
    }

    #[test]
    fn exact_local_responses_and_fixed_tool_order_match_the_frozen_fixtures() {
        let mut relay = C1SyntheticRelay::fixture(C1SyntheticProgram::None);
        let initialize_response = expect_local_response(relay.accept(INITIALIZE_1.as_bytes()));
        assert_eq!(
            initialize_response,
            concat!(
                "{\"jsonrpc\":\"2.0\",\"id\":1,",
                "\"result\":{\"protocolVersion\":\"2024-11-05\",",
                "\"capabilities\":{\"tools\":{}},",
                "\"serverInfo\":{\"name\":\"engram-c1-synthetic\",\"version\":\"1\"}}}"
            )
            .as_bytes()
        );
        assert!(matches!(
            relay.accept(INITIALIZED.as_bytes()),
            C1RelayDecision::NotificationAccepted
        ));
        let tools_response =
            expect_local_response(relay.accept(tools_list("\"list-1\"").as_bytes()));
        let expected =
            format!("{{\"jsonrpc\":\"2.0\",\"id\":\"list-1\"{TOOLS_LIST_RESPONSE_SUFFIX}");
        assert_eq!(tools_response, expected.as_bytes());
        let response = String::from_utf8(tools_response).unwrap();
        assert!(
            response.find(READ_TOOL_DEFINITION).unwrap()
                < response.find(MUTATION_TOOL_DEFINITION).unwrap()
        );
        assert!(!response.ends_with('\n'));
    }

    #[test]
    fn every_synthetic_program_has_exact_order_and_cardinality() {
        for (program, calls) in [
            (C1SyntheticProgram::None, &[][..]),
            (C1SyntheticProgram::SingleRead, &[READ_TOOL_NAME][..]),
            (
                C1SyntheticProgram::SingleMutation,
                &[MUTATION_TOOL_NAME][..],
            ),
            (
                C1SyntheticProgram::MutationThenRead,
                &[MUTATION_TOOL_NAME, READ_TOOL_NAME][..],
            ),
        ] {
            let mut relay = C1SyntheticRelay::fixture(program);
            accept_handshake(&mut relay, "1", "2");
            for (index, tool) in calls.iter().enumerate() {
                let id = (index + 3).to_string();
                match relay.accept(tool_call(&id, tool).as_bytes()) {
                    C1RelayDecision::SyntheticCallAccepted {
                        ordinal,
                        tool: accepted_tool,
                        arguments_sha256,
                        ..
                    } => {
                        assert_eq!(ordinal, u64::try_from(index + 1).unwrap());
                        assert_eq!(accepted_tool.name(), *tool);
                        assert_eq!(
                            arguments_sha256.as_str(),
                            "75d3234c0ad7a748c1fc9cb107bd1714a9e66a239c77ec82e50ec2de044e1e20"
                        );
                    }
                    _ => panic!("expected inert synthetic-call acceptance"),
                }
            }
            assert!(matches!(
                relay.finish(),
                C1RelayDecision::SequenceAccepted { .. }
            ));
        }
    }

    #[test]
    fn accepted_sequence_matches_normative_ledger_vector() {
        let mut relay = C1SyntheticRelay::fixture(C1SyntheticProgram::MutationThenRead);
        accept_handshake(&mut relay, "1", "\"list-1\"");
        assert!(matches!(
            relay.accept(tool_call("2", MUTATION_TOOL_NAME).as_bytes()),
            C1RelayDecision::SyntheticCallAccepted { .. }
        ));
        assert!(matches!(
            relay.accept(tool_call("\"read-1\"", READ_TOOL_NAME).as_bytes()),
            C1RelayDecision::SyntheticCallAccepted { .. }
        ));
        assert_eq!(relay.ledger_bytes().len(), 369);
        match relay.finish() {
            C1RelayDecision::SequenceAccepted { ledger_sha256 } => assert_eq!(
                ledger_sha256.as_str(),
                "cbbfc4a6780d0d0866b815d4626cff06b20f7e15cfd820f05b710db8d7f689a0"
            ),
            _ => panic!("expected accepted sequence"),
        }
    }

    #[test]
    fn numeric_and_string_id_boundaries_and_reuse_are_exact() {
        let valid_ids = vec![
            "1".to_string(),
            "9007199254740991".to_string(),
            "\"a\"".to_string(),
            "\"A.Z_9-x\"".to_string(),
            format!("\"{}\"", "z".repeat(64)),
        ];
        for valid in valid_ids {
            let mut relay = C1SyntheticRelay::fixture(C1SyntheticProgram::None);
            assert!(matches!(
                relay.accept(initialize(&valid).as_bytes()),
                C1RelayDecision::LocalResponse { .. }
            ));
        }
        for invalid in [
            "0",
            "9007199254740992",
            "-1",
            "1.0",
            "true",
            "null",
            "\"\"",
            "\"bad@id\"",
            "\"bad\\ncontrol\"",
            "\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"",
        ] {
            let mut relay = C1SyntheticRelay::fixture(C1SyntheticProgram::None);
            expect_rejection(
                relay.accept(initialize(invalid).as_bytes()),
                C1RelayRejectionCode::InvalidId,
            );
        }

        let mut relay = C1SyntheticRelay::fixture(C1SyntheticProgram::None);
        let _ = relay.accept(INITIALIZE_1.as_bytes());
        let _ = relay.accept(INITIALIZED.as_bytes());
        expect_rejection(
            relay.accept(tools_list("1").as_bytes()),
            C1RelayRejectionCode::ReusedId,
        );

        let mut relay = C1SyntheticRelay::fixture(C1SyntheticProgram::None);
        let escaped = initialize("\"\\u0061\"");
        let response = expect_local_response(relay.accept(escaped.as_bytes()));
        assert!(String::from_utf8(response)
            .unwrap()
            .contains("\"id\":\"a\""));
        let _ = relay.accept(INITIALIZED.as_bytes());
        expect_rejection(
            relay.accept(tools_list("\"a\"").as_bytes()),
            C1RelayRejectionCode::ReusedId,
        );
    }

    #[test]
    fn parser_boundaries_and_recursive_structural_rejections_are_exact() {
        let mut at_cap = INITIALIZE_1.as_bytes().to_vec();
        at_cap.resize(MAX_MESSAGE_BYTES, b' ');
        let mut relay = C1SyntheticRelay::fixture(C1SyntheticProgram::None);
        assert!(matches!(
            relay.accept(&at_cap),
            C1RelayDecision::LocalResponse { .. }
        ));

        let mut over_cap = INITIALIZE_1.as_bytes().to_vec();
        over_cap.resize(MAX_MESSAGE_BYTES + 1, b' ');
        let mut relay = C1SyntheticRelay::fixture(C1SyntheticProgram::None);
        expect_rejection(relay.accept(&over_cap), C1RelayRejectionCode::Oversized);

        for (input, code) in [
            ("{", C1RelayRejectionCode::MalformedJson),
            (
                "{\"jsonrpc\":\"2.0\",\"jsonrpc\":\"2.0\"}",
                C1RelayRejectionCode::DuplicateKey,
            ),
            (
                "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2024-11-05\",\"capabilities\":{},\"clientInfo\":{\"name\":\"engram-c1-synthetic\",\"name\":\"duplicate\",\"version\":\"1\"}}}",
                C1RelayRejectionCode::DuplicateKey,
            ),
            (
                "{\"jsonrpc\":\"2.0\",\"method\":\"initialize\"}x",
                C1RelayRejectionCode::TrailingData,
            ),
            ("[]", C1RelayRejectionCode::BatchForbidden),
            (
                "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2024-11-05\",\"capabilities\":{},\"clientInfo\":{\"name\":\"engram-c1-synthetic\",\"version\":\"1\",\"nested\":{\"_meta\":{}}}}}",
                C1RelayRejectionCode::MetaForbidden,
            ),
        ] {
            let mut relay = C1SyntheticRelay::fixture(C1SyntheticProgram::None);
            expect_rejection(relay.accept(input.as_bytes()), code);
        }
    }

    #[test]
    fn all_semantic_rejection_categories_are_stable_and_poison_permanently() {
        assert_eq!(
            [
                C1RelayRejectionCode::AlreadyPoisoned,
                C1RelayRejectionCode::Oversized,
                C1RelayRejectionCode::MalformedJson,
                C1RelayRejectionCode::DuplicateKey,
                C1RelayRejectionCode::TrailingData,
                C1RelayRejectionCode::BatchForbidden,
                C1RelayRejectionCode::MetaForbidden,
                C1RelayRejectionCode::InvalidJsonrpc,
                C1RelayRejectionCode::InvalidId,
                C1RelayRejectionCode::ReusedId,
                C1RelayRejectionCode::UnexpectedMessage,
                C1RelayRejectionCode::ExtraField,
                C1RelayRejectionCode::ContractMismatch,
                C1RelayRejectionCode::IncompleteProgram,
            ]
            .map(C1RelayRejectionCode::as_str),
            [
                "already_poisoned",
                "oversized",
                "malformed_json",
                "duplicate_key",
                "trailing_data",
                "batch_forbidden",
                "meta_forbidden",
                "invalid_jsonrpc",
                "invalid_id",
                "reused_id",
                "unexpected_message",
                "extra_field",
                "contract_mismatch",
                "incomplete_program",
            ]
        );

        let cases = [
            (
                "{\"jsonrpc\":\"1.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}",
                C1RelayRejectionCode::InvalidJsonrpc,
            ),
            (
                "{\"jsonrpc\":\"2.0\",\"method\":\"initialize\",\"params\":{}}",
                C1RelayRejectionCode::InvalidId,
            ),
            (
                "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"extra\":true,\"params\":{}}",
                C1RelayRejectionCode::ExtraField,
            ),
            (
                "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\"}",
                C1RelayRejectionCode::UnexpectedMessage,
            ),
            (
                "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}",
                C1RelayRejectionCode::ContractMismatch,
            ),
        ];
        for (input, expected) in cases {
            let mut relay = C1SyntheticRelay::fixture(C1SyntheticProgram::None);
            expect_rejection(relay.accept(input.as_bytes()), expected);
            expect_rejection(
                relay.accept(INITIALIZE_1.as_bytes()),
                C1RelayRejectionCode::AlreadyPoisoned,
            );
        }

        let relay = C1SyntheticRelay::fixture(C1SyntheticProgram::SingleRead);
        expect_rejection(relay.finish(), C1RelayRejectionCode::IncompleteProgram);
    }

    #[test]
    fn ordering_alias_case_hyphen_and_cardinality_mismatches_fail_closed() {
        for wrong_tool in [
            MUTATION_TOOL_NAME,
            "C1_SYNTHETIC_READ",
            "c1-synthetic-read",
            "c1_synthetic_read_extra",
        ] {
            let mut relay = C1SyntheticRelay::fixture(C1SyntheticProgram::SingleRead);
            accept_handshake(&mut relay, "1", "2");
            expect_rejection(
                relay.accept(tool_call("3", wrong_tool).as_bytes()),
                C1RelayRejectionCode::ContractMismatch,
            );
        }

        let mut relay = C1SyntheticRelay::fixture(C1SyntheticProgram::SingleRead);
        accept_handshake(&mut relay, "1", "2");
        let wrong_arguments = "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"c1_synthetic_read\",\"arguments\":{\"nonce\":\"changed\"}}}";
        expect_rejection(
            relay.accept(wrong_arguments.as_bytes()),
            C1RelayRejectionCode::ContractMismatch,
        );

        let mut relay = C1SyntheticRelay::fixture(C1SyntheticProgram::SingleRead);
        accept_handshake(&mut relay, "1", "2");
        let extra_argument = "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"c1_synthetic_read\",\"arguments\":{\"nonce\":\"fixture\",\"extra\":true}}}";
        expect_rejection(
            relay.accept(extra_argument.as_bytes()),
            C1RelayRejectionCode::ExtraField,
        );

        let mut relay = C1SyntheticRelay::fixture(C1SyntheticProgram::MutationThenRead);
        accept_handshake(&mut relay, "1", "2");
        expect_rejection(
            relay.accept(tool_call("3", READ_TOOL_NAME).as_bytes()),
            C1RelayRejectionCode::ContractMismatch,
        );

        let mut relay = C1SyntheticRelay::fixture(C1SyntheticProgram::SingleRead);
        accept_handshake(&mut relay, "1", "2");
        let _ = relay.accept(tool_call("3", READ_TOOL_NAME).as_bytes());
        expect_rejection(
            relay.accept(tool_call("4", READ_TOOL_NAME).as_bytes()),
            C1RelayRejectionCode::UnexpectedMessage,
        );
    }

    #[test]
    fn multi_error_precedence_matches_the_frozen_order() {
        let mut relay = C1SyntheticRelay::fixture(C1SyntheticProgram::None);
        expect_rejection(
            relay.accept(br#"{"jsonrpc":"1.0","id":null,"method":"initialize","extra":true}"#),
            C1RelayRejectionCode::InvalidJsonrpc,
        );

        let mut relay = C1SyntheticRelay::fixture(C1SyntheticProgram::None);
        expect_rejection(
            relay.accept(br#"{"jsonrpc":"2.0","id":null,"method":"initialize","extra":true}"#),
            C1RelayRejectionCode::ExtraField,
        );

        let mut relay = C1SyntheticRelay::fixture(C1SyntheticProgram::None);
        expect_rejection(
            relay.accept(br#"[{"_meta":{}}]"#),
            C1RelayRejectionCode::BatchForbidden,
        );

        let mut relay = C1SyntheticRelay::fixture(C1SyntheticProgram::None);
        let _ = relay.accept(INITIALIZE_1.as_bytes());
        expect_rejection(
            relay.accept(br#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#),
            C1RelayRejectionCode::ReusedId,
        );
    }

    #[test]
    fn source_and_output_remain_inert_private_and_secret_free() {
        let source = include_str!("native_successor_relay.rs");
        assert!(source.contains("PhantomData<Rc<()>>"));
        assert!(source.contains("#[cfg(test)]\n    fn fixture("));
        for forbidden in [
            concat!("std::", "net"),
            concat!("std::", "process"),
            concat!("std::", "thread"),
            concat!("std::", "sync"),
            concat!("tokio", "::"),
            concat!("req", "west"),
            concat!("Command", "::new"),
        ] {
            assert!(!source.contains(forbidden), "forbidden API: {forbidden}");
        }

        let marker = "PRIVATE-CANARY-MUST-NOT-ESCAPE";
        let mut relay = C1SyntheticRelay::fixture(C1SyntheticProgram::None);
        let first = expect_local_response(relay.accept(INITIALIZE_1.as_bytes()));
        let _ = relay.accept(INITIALIZED.as_bytes());
        let second = expect_local_response(relay.accept(tools_list("2").as_bytes()));
        assert!(!first
            .windows(marker.len())
            .any(|window| window == marker.as_bytes()));
        assert!(!second
            .windows(marker.len())
            .any(|window| window == marker.as_bytes()));
    }
}
