# Native strict successor Stage C2A — frozen pure semantic-snapshot validator

Date: 2026-09-06 (Asia/Jerusalem)

## Status and reason for this slice

This design is frozen for exact independent review before implementation. It authorizes only a
private, pure validator and keyed authentication over an already-materialized bounded persisted-
state snapshot. It is a prerequisite to, not proof of, an atomic datastore collector.

The narrower slice is mandatory because the current public `engram_store::Db` is
`Surreal<Any>` and can represent a remote connection. A function accepting that value cannot prove
local, evaluator-owned, environment-independent or network-free acquisition. In SurrealDB 2.6.0,
an explicit `BEGIN` block also obtains a write transaction even when its body contains only reads.
Therefore C2A accepts no `Db`, path, configuration, connection or reader capability. A later C2B
must separately mint a local evaluator-owned read witness and use exactly one bare read-only
`RETURN { SELECT ... }` statement before it may feed C2A.

C2A does not authorize a store open, daemon, listener, socket, HTTP/MCP request, provider,
authentication, filesystem access, environment access, Git operation, process, thread, async task,
clock read, mutation, artifact writer, public CLI, public MCP method, public run API, or relay. It
does not prove atomic acquisition, transport byte bounds, correction attribution, deletion
completion outside the included tables, or flagship success.

## Exact source boundary

The only authorized implementation files are:

```text
engram-eval/src/native_successor_semantic.rs
engram-eval/src/lib.rs
```

`lib.rs` adds one private module declaration. `native_successor_semantic` is never `pub`, is not
re-exported, and has no non-test caller in C2A. No existing public facade, CLI, MCP, Store, Index,
runner, daemon, correction, isolation or VM module may refer to it. Its raw input and validated
snapshot types do not implement `Debug`, `Display`, `Clone`, `Serialize` or `Deserialize`.

The production source may use only `engram-core` domain types plus `serde`, `serde_json`, `sha2`,
`time`, `unicode-normalization`, `zeroize`, and domain-independent standard-library
collections/conversion. It must
not import or reference
`engram-store`, `engram-index`, `engram-mcp`, `native_execution`, `native_successor_core`,
`native_successor_artifact`, `native_successor_relay`, historical native runners, filesystem,
network, process, environment, async/runtime, synchronization, randomness, logging or time-now
APIs. Existing timestamps may be parsed, converted to UTC, compared and reserialized; the clock is
never read.

Serialization and normalization behavior is pinned to this repository's `Cargo.lock`: serde
1.0.228, serde_json 1.0.149, sha2 0.10.9, time 0.3.46, unicode-normalization 0.1.25 and zeroize
1.8.2. The entire lockfile SHA-256 is
`8741698619a41dcce8f58aae3609e6ee48929d6c2e91b955caf0ed586920572e`; the accepted compiler is
`rustc 1.93.0 (254b59607 2026-01-19)`. Any dependency/toolchain change requires a new frozen
design/hash and rerunning every golden.

The exact semantic source inputs are pinned as follows:

```text
engram-core/src/id.rs          b7ee04c068128f48a5df9ffea355dcdca66817ff54c10b65c7e2b197021e6953
engram-core/src/memory.rs      26f9f2e7f8920b1328220c1408a8a6f638e7aba8dc3f61d75d4d83cb547a911c
engram-core/src/repository.rs  ec2917ebb76e8dbac10a5a953d1f5b7b74aa352c03f008dc831205a1fbdf3f01
engram-core/src/work.rs        ffbb62e50d9084717680bf076f0d9473b3c6e6499cdbe024a5171ce7551466c6
```

The copied reference algorithms are pinned to store secret source SHA-256
`0db4abdabbfe2ddd57fb7f4ef94694b29c0b367a679eaa45989e50cfcbb74f92`, store memory source
`c285cb5b8359e2768d2f031f12e9ff3e87717879b59d17b194cd15ae2f3ae8ca`, index memory source
`7ece3d1c85c325a2d6e0cd5ba9dcaebd77d32d5d5240bffec2a42375018886b4`, and index repository
source `85cccf12cec5af20af3ec87ef3ffe53d34fca0a6fb5309ecc58b8d40505ab13a`.
The persisted topology projection reference
`engram-store/src/repos/repository.rs` is pinned at
`20834be588301dc669822b25f919df9c557c82dbc47363ce04cd9b3f5f1f344e`.

## Raw snapshot input and exact tables

The sole production entry consumes one private `C2RawSnapshot`, a bounded collection of opaque
prior correction bindings minted by an earlier successful C2A validation, and one private one-shot
32-byte MAC key:

```text
fn validate_semantic_snapshot(
    raw: C2RawSnapshot,
    prior_bindings: Vec<C2PriorCorrectionBinding>,
    mac_key: C2OneShotMacKey,
) -> Result<C2ValidatedSnapshot, C2SemanticError>
```

All five named types and the function are module-private and marked `#[allow(dead_code)]` only
because C2A intentionally has no production caller. The snapshot value contains a target selector
and raw persisted rows from exactly these nine tables in this fixed order:

1. `memory_item`;
2. `correction_proposal`;
3. `memory_forget_receipt`;
4. `work_project`;
5. `work_task`;
6. `git_repository`;
7. `local_checkout`;
8. `monorepo_component`; and
9. `project_repository_link`.

Each table is a vector of raw `serde_json::Value` objects containing the complete persisted row,
including `record_id`; a future collector must preserve unknown fields rather than projecting them
away. A row does not gain authority because it is already represented by a Rust domain type: C2A
rejects unknown or state-required missing keys at every object level, then strictly decodes. It
never uses repository conversion helpers that overwrite inner IDs, enum parsers that default
unknown values, or timestamp parsers that substitute the current time.

The exact private Rust input shape is:

```text
C2RawSnapshot {
  target: C2RawTarget,
  memory_item: Vec<Value>, correction_proposal: Vec<Value>,
  memory_forget_receipt: Vec<Value>, work_project: Vec<Value>, work_task: Vec<Value>,
  git_repository: Vec<Value>, local_checkout: Vec<Value>,
  monorepo_component: Vec<Value>, project_repository_link: Vec<Value>
}
C2RawTarget {
  project_id: String, project_name: String,
  repository_id: String, repository_remote: String,
  checkout_id: String, checkout_path: String,
  task_id: Option<String>, task_name: Option<String>
}
```

The target selector contains exact project ID and project name, exact repository ID and remote
URL, exact checkout ID and absolute lexical path, and optional exact task ID/name. All strings are
nonempty NFC UTF-8 without NUL or ASCII control characters. If two supplied selectors identify the
same concept, they are conjunctive; disagreement rejects rather than falling back to one field.
The optional target task ID and name are either both absent or both present.

The exact successful wrapper fields are:

```text
C2ValidatedSnapshot {
  memory_items: Vec<MemoryItem>, correction_proposals: Vec<CorrectionProposal>,
  projects: Vec<Project>, tasks: Vec<Task>, repositories: Vec<GitRepository>,
  checkouts: Vec<LocalCheckout>, components: Vec<MonorepoComponent>,
  project_links: Vec<ProjectRepositoryLink>,
  target: C2ResolvedTarget,
  applicable_memory_ids: Vec<Id>, target_affecting_correction_ids: Vec<Id>,
  pending_bindings: Vec<C2PriorCorrectionBinding>, audit: C2SanitizedAudit
}
C2ResolvedTarget {
  project_id: Id, repository_id: Id, checkout_id: Id, task_id: Option<Id>
}
C2PriorCorrectionBinding {
  proposal_id: Id,
  proposal_row: Value, obsolete_row: Value, replacement_row: Value,
  proposal: CorrectionProposal, obsolete: MemoryItem, replacement: MemoryItem,
  proposal_record_mac: [u8; 32], obsolete_record_mac: [u8; 32],
  replacement_record_mac: [u8; 32]
}
```

All vectors are UUID-byte sorted; forget receipts must be empty. `C2ValidatedSnapshot` implements
none of `Debug`,
`Display`, `Clone`, `Serialize` or `Deserialize`; every field is private. The binding owns the
three complete strict pending raw rows—proposal, obsolete and replacement—plus their already-
verified typed values and prior record MACs. It has no field or
constructor accessible outside this module and implements none of those five traits. Tests obtain a binding only by
consuming it from a successful pending-state `C2ValidatedSnapshot`, never by constructing it.

The prior-binding input has at most 32 entries, is UUID-byte sorted, move-only, and rejects
duplicate proposal IDs. Its complete retained rows count toward the next validation's total
node/byte limits before the new raw snapshot. C2A does not infer a binding from an applied proposal:
only a prior successful validation can mint one. C2B must later prove that both validations and the
operator transition belong to the same evaluator-owned execution; C2A proves the exact semantic
transition between the supplied states but does not by itself prove process attribution.

## Exact raw schemas

In the schemas below every listed key is required and every unlisted key rejects. A key in square
brackets is omitted exactly when its value is absent and is required with a non-null value when
present. A key written as `null | T` is always present. `record_id` is canonical UUID text supplied
by the collector, never a Surreal record object. C2B must follow the per-field
`NONE`-to-null-or-omission table below and normalize native datetimes to strict canonical UTC
RFC3339 strings before constructing this private value, while preserving every unknown stored
field so C2A can reject it.

Exact table-row key sets are:

```text
memory_item:
  record_id, item, kind_key, status_key, scope_key, harness_key, model_key,
  session_id, snapshot_digest, created_at, updated_at

correction_proposal pending:
  record_id, proposal, status_key, obsolete_id, pending_obsolete_id, replacement_id,
  snapshot_digest, created_at
correction_proposal applied:
  record_id, proposal, status_key, obsolete_id, replacement_id, snapshot_digest, created_at

memory_forget_receipt (always terminal-invalid in C2A):
  record_id, deleted, proposal_ids, pending_replacement_ids, unlocked_obsolete_ids,
  cleanup_complete, created_at, [completed_at]

work_project:
  record_id, name, description, status, created_at, updated_at
work_task:
  record_id, project_id, name, description, status, priority, jira_key, blocked_by,
  created_at, updated_at

git_repository:
  record_id, repository, name_key, remote_url, provider_key, created_at, updated_at
local_checkout:
  record_id, checkout, repository_id, local_path_key, current_branch, head_sha, is_dirty,
  created_at, updated_at, last_seen_at
monorepo_component:
  record_id, component, repository_id, name_key, path_key, kind, created_at, updated_at
project_repository_link:
  record_id, link, project_id, project_name_key, repository_id, component_id,
  component_path_key, role, created_at, updated_at
```

All optional scalar projections such as `session_id`, `remote_url`, `repository_id`, branch, HEAD,
dirty state, kind and component selectors are required keys whose value is JSON null or the exact
scalar. Projection timestamps are required strict RFC3339 strings and must represent the same
instant as their embedded value.

Embedded payload key sets are:

```text
MemoryItem:
  id, kind, title, content, scope, origin, writer, evidence, confidence, status,
  supersedes, tags, created_at, updated_at, last_used_at, review_after, archive, procedure,
  [correction_proposal_id], [pending_correction_proposal_id]

CorrectionProposal pending:
  id, obsolete_id, replacement_id, memory_kind, scope, canonical_digest,
  digest_schema_version, status, proposer, created_at, applied_at
CorrectionProposal applied:
  id, obsolete_id, replacement_id, memory_kind, scope, canonical_digest,
  digest_schema_version, applied_digest, status, proposer, created_at, applied_at

GitRepository:
  id, name, remote_url, provider, default_branch, description, created_at, updated_at
LocalCheckout:
  id, repository_id, local_path, current_branch, head_sha, is_dirty, created_at,
  updated_at, last_seen_at
MonorepoComponent without source:
  id, repository_id, name, path, kind, description, created_at, updated_at
MonorepoComponent with source:
  id, repository_id, name, path, kind, description, source_path, source_sha256,
  created_at, updated_at
ProjectRepositoryLink:
  id, project_id, project_name, repository_id, component_id, component_path, role,
  created_at, updated_at
```

The work-project and work-task row body is itself the strict domain payload after `record_id` is
converted to the domain ID; there is no second embedded object. Forget-receipt ID vectors contain
canonical UUID text only.

Nested exact key sets are:

```text
WriterProvenance:
  harness, harness_version, model, surface, actor, session_id, written_at
ModelIdentity:
  provider, model, version
EvidenceRef:
  kind, target, summary, excerpt, observed_at
ArchiveMetadata:
  reason, archived_by, archived_at
ProcedureCard:
  task, prerequisites, commands, failure_signatures, verification, expires_at
ProcedurePrerequisite without source:
  key, expected
ProcedurePrerequisite with source:
  key, expected, source
ProcedurePrerequisiteSource::Toml:
  format="toml", relative_path, key_path
ProcedureVerification:
  command, expected_exit_code, expected_output_contains, evidence_path, evidence_sha256,
  verified_at
```

`MemoryScope` is an internally tagged object with these exact variant key sets:

```text
global:     type="global"
user:       type="user"
project:    type="project", project_id, project_name
task:       type="task", project_id, project_name, task_id, task_name
entity:     type="entity", entity_id, entity_name
repository: type="repository", repository_id, remote_url, local_path
session:    type="session", session_id
custom:     type="custom", name
```

Unit enum variants use their exact snake-case JSON string. The data-bearing variants
`MemoryKind::Custom`, `Harness::Other`, `ClaimOrigin::Custom`, `EvidenceKind::Custom`, and
`RepositoryProvider::Other` use the single-key externally tagged objects `{"custom": string}` or
`{"other": string}` as applicable; no parser aliases are accepted. JSON null is accepted only at
the always-present optional fields listed above. `ProcedurePrerequisite.source`,
`correction_proposal_id`, `pending_correction_proposal_id`, `applied_digest`, and component source
fields use omission rather than null exactly as their current persisted Serde contract requires.

## Exact collector normalization and projection formulas

C2B must preserve unknown fields but normalize Surreal `NONE` only as follows. It emits JSON null
for these required nullable top-level keys: project `description`; task `description` and
`jira_key`; memory `session_id`; repository `remote_url`; checkout `repository_id`,
`current_branch`, `head_sha`, and `is_dirty`; component `kind`; and link `project_id`,
`component_id`, and `component_path_key`. It omits `pending_obsolete_id` exactly for an applied
proposal and requires it for pending; it omits receipt `completed_at` when `NONE`; and it preserves
the embedded JSON's Serde omission/null distinction without rewriting it. It removes Surreal's
record-object `id` only after emitting its canonical UUID payload as `record_id`. Native top-level
datetimes become the exact canonical UTC RFC3339 strings required above. No other missing known
field is repaired: C2A rejects it.

Let `lower(s)` mean pinned Rust `str::to_lowercase()` over NFC input. Denormalized projections must
equal these exact formulas, with nullable output represented by JSON null:

```text
memory_item:
  kind_key = Display(item.kind)
  status_key = Display(item.status)
  scope_key = match item.scope {
    global => "global"; user => "user";
    project => "project:" + project_name;
    task => "task:" + task_name;
    entity => "entity:" + entity_name;
    repository => "repository:" + first_non_null(remote_url, local_path, "");
    session => "session:" + session_id; custom => "custom:" + name
  }
  harness_key = Display(item.writer.harness)
  model_key = item.writer.model.model
  session_id = item.writer.session_id
  snapshot_digest = frozen embedded-item snapshot digest
  created_at = item.created_at; updated_at = item.updated_at

correction_proposal:
  status_key = Display(proposal.status)
  obsolete_id = proposal.obsolete_id
  pending_obsolete_id = proposal.obsolete_id only when pending, otherwise omitted
  replacement_id = proposal.replacement_id
  snapshot_digest = frozen embedded-proposal snapshot digest
  created_at = proposal.created_at

git_repository:
  name_key = lower(repository.name); remote_url = repository.remote_url
  provider_key = Display(repository.provider)
  created_at = repository.created_at; updated_at = repository.updated_at

local_checkout:
  repository_id = checkout.repository_id; local_path_key = checkout.local_path
  current_branch = checkout.current_branch; head_sha = checkout.head_sha
  is_dirty = checkout.is_dirty; created_at = checkout.created_at
  updated_at = checkout.updated_at; last_seen_at = checkout.last_seen_at

monorepo_component:
  repository_id = component.repository_id; name_key = lower(component.name)
  path_key = component.path; kind = component.kind
  created_at = component.created_at; updated_at = component.updated_at

project_repository_link:
  project_id = link.project_id; project_name_key = lower(link.project_name)
  repository_id = link.repository_id; component_id = link.component_id
  component_path_key = link.component_path; role = Display(link.role)
  created_at = link.created_at; updated_at = link.updated_at
```

The top-level project/task fields are their strict payload directly, with `record_id` injected as
the domain ID; there is no denormalized duplicate beyond the exact fields listed. Every projected
timestamp string must byte-equal its canonical embedded string, not merely denote the same instant.

## Frozen limits and validation order

The input owner must have bounded acquisition before constructing `C2RawSnapshot`; C2A does not
claim to undo an allocation already performed by its caller. C2A nevertheless streaming-walks and
charges the complete virtual input before secret scanning, semantic projection or MAC computation.
The virtual input is the exact JSON-shaped object below; it is never allocated or serialized as a
whole:

```text
{
  "target": {the eight C2RawTarget fields},
  "prior_correction_bindings": [
    {"proposal_id": id, "proposal_row": complete_row,
     "obsolete_row": complete_row, "replacement_row": complete_row}, ...
  ],
  "memory_item": [...], "correction_proposal": [...],
  "memory_forget_receipt": [...], "work_project": [...], "work_task": [...],
  "git_repository": [...], "local_checkout": [...], "monorepo_component": [...],
  "project_repository_link": [...]
}
```

It includes every table container, known and unknown row key/value, and the complete retained raw
rows in every prior binding. The MAC key is not part of the virtual input. C2A checks all of these
limits:

| Boundary | Maximum |
| --- | ---: |
| all raw nodes | 131,072 |
| all raw UTF-8 and traversal-framing bytes | 2,097,152 |
| one row's raw UTF-8 and traversal-framing bytes | 65,536 |
| `memory_item` rows | 64 |
| `correction_proposal` rows | 32 |
| `memory_forget_receipt` rows | 32 |
| `work_project` rows | 8 |
| `work_task` rows | 64 |
| `git_repository` rows | 16 |
| `local_checkout` rows | 32 |
| `monorepo_component` rows | 64 |
| `project_repository_link` rows | 64 |
| JSON/object nesting depth | 16 |
| keys in one object | 64 |
| elements in one vector | 64 |
| ordinary scalar UTF-8 bytes | 4,096 |
| exact `item.content`, procedure command, or evidence excerpt UTF-8 bytes | 32,768 |
| name, title, tag, enum spelling or branch UTF-8 bytes | 256 |
| evidence, tags or supersedes entries | 32 |
| procedure commands, prerequisites or failure signatures | 16 |
| prerequisite key-path entries | 16 |

The exact accounting pseudocode is:

```text
walk(value, depth, path):
  require depth <= 16
  nodes += 1
  null/bool: bytes += 1
  number:    bytes += 1 + 8 + len(Number::to_string())
  string:    enforce path scalar cap; bytes += 1 + 8 + utf8_len
  array:     enforce path vector cap; bytes += 1 + 8
             walk(each element, depth + 1, indexed path, in stored order)
  object:    require member_count <= 64; bytes += 1 + 8
             for keys in UTF-8-byte order:
               nodes += 1; require depth + 1 <= 16
               bytes += 8 + key_utf8_len; enforce ordinary scalar cap on unknown keys
               walk(value, depth + 1, keyed path)
```

`walk(virtual_input, 0, root)` means the root object itself consumes one node. Thus each object key
is a separate node at parent depth plus one, while its value is another node at that depth. The
byte total is exactly the length produced by the canonical JSON-value encoder frozen below; there
are no additional implicit wrapper, selector, table-tag or row-framing charges. Each complete raw
row must also independently satisfy `len(encode(row)) <= 65,536`; table-array/key overhead is only
global. Each retained binding row was independently row-bounded when minted and is charged again
globally on consumption.

Record IDs and selector/binding strings are ordinary scalars. The three named 32,768-byte paths
are the only exceptions to the ordinary 4,096-byte string cap: any exact embedded
`item.content`, each `item.procedure.commands[]`, and each `item.evidence[].excerpt`. Names,
titles, tags, enum spellings and branches use the stricter 256-byte cap. Unknown strings and keys
receive no exception. Binding count is capped at 32; its three retained row objects are not subject
to a second arbitrary-vector cap beyond their original table/schema vector caps.

The complete empty-table count golden uses target project/repository/checkout IDs ending
`000000000010`, `000000000011`, and `000000000012`, project `engram`, remote
`https://github.com/ymeiri/engram.git`, path `/workspace/engram`, null task pair, no bindings, and
all nine empty tables. It has exactly 39 nodes, maximum depth 2 and 747 accounting bytes.

All arithmetic uses checked `u64` addition and conversion. Limits are inclusive. Per-row counters
reset only at a table row boundary, while the global counters never reset. The pre-walk inspects
all table lengths first and reports every overflowing table as one fixed nine-bit mask in the
table order above. Any other first excess in deterministic selector, binding, table and row order
yields categorical `LimitExceeded` with no MAC or partial result. The nine-bit overflow mask
contains no caller text.

Validation precedence is exact:

1. table-count mask, then raw node/byte/scalar/vector/depth failure;
2. likely-secret failure;
3. exact key-set and strict scalar/domain decoding;
4. record-ID and denormalized-projection consistency;
5. whole-store referential and correction consistency;
6. target identity and scope resolution;
7. canonical framing and keyed authentication.

The exact private error enum is:

```text
enum C2SemanticError {
  LimitExceeded { table_overflow_mask: Option<u16> },
  SecretMaterial,
  InvalidRecord,
  InconsistentProjection,
  IncompleteDeletion,
  AmbiguousIdentity,
  ScopeMismatch,
  AppliedProvenanceUnproven,
}
```

Only the table-count precheck produces `LimitExceeded { Some(mask) }`; every other limit error uses
`None`. Bits 0 through 8 correspond to the fixed table order. All other variants are units, so no
row ordering or location can become an output side channel. `Display` is exactly the lowercase
snake-case variant name and never includes the mask; derived `Debug` may additionally show the
numeric mask. The enum is not serializable. It never contains a raw database error, key, value,
name, title, content, evidence, writer, path, remote URL, digest, secret kind, row ordinal or other
caller-controlled string.

Within each precedence stage, target fields precede prior bindings, tables use fixed table order,
and raw rows use supplied order until all strict decoding succeeds; subsequent semantic vectors
use UUID-byte order. Because each stage maps all of its failures to one fixed category, ties within
a stage cannot alter the returned value. Exact error mapping is:

- `LimitExceeded`: every count, node, byte, scalar, vector, nesting or arithmetic limit;
- `SecretMaterial`: any frozen detector match after the complete limit walk;
- `InvalidRecord`: unknown/missing/state-forbidden key, wrong JSON type, malformed/canonical ID,
  enum, timestamp, confidence, digest, path or remote, forged embedded ID, or invalid domain field;
- `InconsistentProjection`: stale/missing denormalized value or persisted snapshot digest,
  duplicate record or logical identity (including ASCII-case-fold project twins), orphan/dangling
  foreign key, blocked-by edge, supersedes edge, proposal pair or marker, correction cycle,
  component/link disagreement, and any current-snapshot correction state, canonical-digest or
  applied-digest rule that does not depend on a prior binding;
- `IncompleteDeletion`: after limit, secret and strict schema validation succeeds, any
  forget-receipt row, regardless of its otherwise valid fields or claimed completion;
- `AmbiguousIdentity`: missing or multiple target project/repository/checkout/task resolutions,
  zero or multiple primary links, or competing target identity other than a duplicate logical
  identity already owned by `InconsistentProjection`;
- `ScopeMismatch`: a memory scope that touches a target identifier but whose other present
  selectors disagree, including an empty or split repository scope; and
- `AppliedProvenanceUnproven`: the target-affecting applied-proposal set and opaque binding set
  differ, or any binding-dependent complete-row, inverse-transition, timestamp or inverse-pending-
  digest comparison fails. Current-snapshot persisted, canonical and applied digest failures are
  instead `InconsistentProjection`.

When several semantic failures coexist, C2A finishes strict per-record validation first, then
checks forget receipts, correction graph, whole-store foreign keys, target topology, scope
projection and applied bindings in that exact order. It returns the earliest category under that
ordering.

## Secret and raw-data boundary

Before any record or state MAC is computed, C2A recursively scans the entire bounded raw snapshot,
including selector fields, binding fields, table/row projections, unknown object keys and every
string value. Matching is byte-for-byte the following frozen algorithm:

1. An ASCII-uppercase copy containing `-----BEGIN PRIVATE KEY-----`,
   `-----BEGIN RSA PRIVATE KEY-----`, `-----BEGIN OPENSSH PRIVATE KEY-----`, or
   `AUTHORIZATION: BEARER ` matches.
2. Each whitespace-delimited token containing `://` is split at the first scheme delimiter, then
   at the first `/`; its authority matches when text before `@` contains `:`.
3. For each line and whitespace token, trim `'`, `"`, backtick, comma, semicolon, parentheses and
   brackets; strip a leading literal `export`; split once at `=`; ASCII-uppercase the trimmed name.
   Trim whitespace and then leading/trailing `'`, `"` or backtick from the assigned value. A
   resulting value of at least eight bytes matches when the name is `API_KEY`,
   `API_TOKEN`, `ACCESS_TOKEN`, `AUTH_TOKEN`, `PASSWORD`, `PASSWD`, `CLIENT_SECRET`, or
   `PRIVATE_KEY`.
4. Split tokens on whitespace or the same quote/comma/semicolon/parenthesis/bracket delimiters,
   then trim leading/trailing `:` and `=`. Match `AKIA` plus exactly 16 ASCII alphanumerics;
   `github_pat_` of at least 30 bytes; `ghp_`, `gho_`, `ghu_`, `ghs_`, or `ghr_` of at least 20
   bytes; `xoxb-`, `xoxp-`, `xoxa-`, `xoxr-`, or `xoxs-` of at least 20 bytes; or `sk-` of at least
   24 bytes.
5. A JWT matches exactly three dot-separated segments when the first begins `eyJ`, every segment
   is at least eight bytes, and every byte is ASCII alphanumeric, `-`, `_`, or `=`.
6. An object key, after trim, replacement of `-` and space by `_`, and ASCII uppercasing, is a
   credential field for the same eight names above plus `APIKEY` and `AUTHORIZATION`. Its nested
   value matches if it contains any string of at least eight trimmed bytes except the ASCII-
   lowercase placeholders `[redacted]`, `changeme`, `example`, `not-set`, `placeholder`,
   `redacted`, and `your_token_here`. Object keys are also scanned as ordinary strings.

Raw limit and secret traversal order is target fields in declared order, bindings in supplied
order, then fixed table order and supplied row order; object keys precede their values and are
UTF-8-byte sorted. UUID sorting begins only after strict decoding succeeds. Any match returns
`SecretMaterial`; it is neither redacted-and-accepted nor included in an unkeyed digest. Tests use
only synthetic canaries and include one positive and one near-miss vector for every rule.

No serializable or format-capable success value or error exposes raw rows. The only serializable result is the sanitized audit
defined below; `C2SemanticError` is not serializable and its `Debug`/`Display` implementations emit
only the fixed category, with the numeric overflow mask visible through `Debug` alone. Source tests
serialize the success shape and format every error shape, proving that synthetic
names, content, evidence, writer strings, paths, remotes, and secret canaries are absent. C2A does
not claim heap zeroization of raw `serde_json::Value`; that remains a C2B acquisition and ownership
requirement.

## Strict persisted-record validation

Every UUID is canonical lowercase UUID-v7 text and the outer record ID must equal the embedded ID
where the domain payload has one. A timestamp is accepted only when parsing it as RFC3339,
converting it to UTC, and formatting it with pinned
`time::format_description::well_known::Rfc3339` reproduces the exact original UTF-8 string;
equivalent non-UTC offset spellings therefore reject. The accepted string is what
the canonical JSON encoder authenticates. Confidence is finite and in `[0, 1]`. SHA-256 values are
exactly 64 lowercase hexadecimal bytes. All stored enum spellings must be exact; unknown values
reject.

Every identity- or projection-bearing string—project/task/repository/component/link names, Jira
keys, branches, remotes, paths, scope selector names/URLs/paths, tags, custom enum payloads, writer
harness/model/provider/actor/surface/version fields and evidence targets—must already be NFC; C2A
never silently normalizes stored text. Required names and custom payloads are nonempty after
Unicode trim. NUL and ASCII control characters reject in these fields. Memory/evidence prose may
contain newline and tab but not NUL or other ASCII control characters and is authenticated exactly.

For every row, all persisted denormalized selector fields must equal values recomputed from the
strict payload, including memory kind/status/scope/writer/session and snapshot digest; proposal
status/obsolete/replacement/digest; repository name/remote/provider; checkout repository/path/Git
state; component repository/name/path; and project-link project/repository/component/role fields.
Missing legacy projections reject for this strict successor rather than being repaired. Duplicate
record IDs or duplicate logical identities reject.

Repository remotes must contain no credentials and must normalize to one unambiguous stable
identity using this exact algorithm: reject the raw value if it contains any `?` or `#`; trim
Unicode whitespace and trailing `/`; remember whether the remaining value ends in `.git` and whether it begins
`git@` or `ssh://git@`; strip one recognized prefix from `https://`, `http://`, `ssh://git@`,
`git@`, or a bare `github.com/`, `gitlab.com/`, or `bitbucket.org/`; strip trailing `.git` and `/`;
split host/path at the first `:` when present, otherwise the first `/`; require a dotted host and
either a recognized hosting host (ASCII-lowercase exactly `github.com`, `gitlab.com`, or
`bitbucket.org`, or containing `.git.`), the remembered `.git`, or Git-SSH form; take exactly the first
two nonempty slash-separated path segments as owner/repository; strip their trailing `.git`; and
emit lowercase host plus original owner/repository as `host/owner/repository`. Extra path segments,
empty owner/repository, user-info other than the exact stripped `git@`, and failure to parse reject.
Two repositories with the same normalized bytes are a duplicate logical identity.

Every target repository has at least one checkout. A checkout path is NFC, begins with exactly one
`/`, is not `/`, has no trailing `/`, `//`, NUL, ASCII control, `.` or `..` segment, and otherwise
preserves UTF-8 bytes; equality is exact bytes after NFC. No filesystem case-folding or symlink
resolution occurs. Optional HEAD values are exact lowercase 40- or 64-byte hexadecimal IDs.
Component and component-source paths are NFC and either exactly `.` or a slash-separated relative
path with no leading/trailing slash, empty, `.` or `..` segment. They preserve all other bytes.
A project link is either repository-wide, with both component ID/path absent, or binds both to one
existing component in the same repository. Every foreign key resolves exactly once.

Duplicate identity rejection uses only these exact tuples/comparison domains:

```text
every table row:                    (table_tag, record_uuid)
work_project:                       ASCII-lowercase(project.name)
work_task selector namespace:       (project_id, exact NFC task.name or non-null jira_key)
git_repository with remote:         normalized_remote
local_checkout:                     exact normalized absolute local_path (global)
monorepo_component:                 (repository_id, exact normalized component.path)
project_repository_link:            (resolved_project_id, repository_id, component_id-or-none)
correction proposal pair:           (obsolete_id, replacement_id)
```

A task name and Jira key token may coincide only when both resolve to the same task; otherwise the
selector namespace is ambiguous. ASCII-case-fold twins for task names or Jira keys in one project
also reject even though strict lookup uses exact bytes. Repository rows without a remote have no
remote identity tuple but still have unique record IDs. A replacement ID belongs to at most one
proposal, an obsolete ID belongs to at most one proposal, and the complete replacement-ID and
obsolete-ID sets are disjoint. Proposal correction chains are unsupported in this strict slice:
no item may occupy either proposal role more than once or become the obsolete member of a later
proposal. Link role is deliberately absent from the identity tuple, so two otherwise equal links
with different roles are conflicting duplicates.
Names of repositories/components and byte-identical content/titles are not identity keys.

After limit, secret and strict schema validation, any `memory_forget_receipt` rejects as
`IncompleteDeletion`; secret-bearing or malformed receipts retain the earlier `SecretMaterial` or
`InvalidRecord` result. A completed receipt is invalid residue because completed receipts must
have been removed. Missing correction pairs, dangling proposal markers, dangling supersedes IDs,
correction cycles, and orphan topology reject without partial output. Ordinary direct-correction
`supersedes` edges need not have a proposal, but every item's `supersedes` vector is duplicate-free,
every named target has status `superseded`, and each target ID has at most one incoming
`supersedes` edge across all direct and applied-correction items.

Whole-store referential validation is exact: every task project exists; every `blocked_by` ID is
unique, is not self, resolves to a task in the same project, and the directed task graph is
acyclic; every present checkout repository ID exists; every component repository exists; every
link repository exists; every link project name resolves to exactly one exact-name project and
every present link project ID identifies that same project; and a component-scoped link resolves
both its component ID and exact component path to one component in the link repository. Every
memory `supersedes` ID exists, is not self, is unique within that item's vector, names a
`superseded` item, and has no second incoming superseder anywhere in the snapshot. Every proposal
ID, proposal pair, item marker and prior binding is unique and cross-resolves exactly as specified
below. These checks apply to all rows, not only rows selected for the target view. Entity- and
session-scope IDs are opaque because their owning tables are deliberately outside this snapshot;
they grant no target applicability.

Every project memory scope resolves its exact name to one project and any present project ID must
identify that project. Every task memory scope resolves its name as exact task name or exact Jira
key, constrained by every present project/task ID/name field, to exactly one task and its project;
zero or multiple candidates reject. Every repository memory scope has at least one non-null
repository ID, normalized remote, or lexical checkout path, and all present fields conjunctively
resolve to exactly one repository plus, when a path is present, exactly one checkout. These
whole-store resolutions establish identity only; target applicability is the separate projection
rule below.

## Frozen persisted and correction digests

Persisted `snapshot_digest` is not part of the serializable audit and confers no secrecy. It is
validated only for the two tables that store it: `memory_item` and `correction_proposal`. The
validator strictly decodes the embedded payload, reserializes that typed payload with
`serde_json::to_vec`, computes SHA-256, formats all 32 digest bytes as 64 lowercase hexadecimal
characters, and compares in constant time. Serialization has no whitespace; struct fields use the
declaration order shown under **Exact raw schemas**; object enum variants use their exact Serde
shape; timestamps must already equal their `time::serde::rfc3339` reserialization; and the two
item marker fields, proposal `applied_digest`, and component source fields follow the exact
state-dependent omission rules above. A present JSON number must decode into its declared integer
type or, for confidence, finite `f32`; negative zero rejects. Because `serde_json::Value` has
already discarded some source-text numeric spelling, C2A makes no claim about the original JSON
lexeme and authenticates `serde_json::Number::to_string()` exactly. The row's outer `record_id` and
all denormalized projections are excluded from this persisted snapshot digest.

The persisted-snapshot goldens are these single compact lines without trailing newlines:

```json
{"id":"01890f5e-7b00-7000-8000-000000000002","kind":"decision","title":"old decision","content":"old content","scope":{"type":"project","project_id":null,"project_name":"engram"},"origin":"user_stated","writer":{"harness":"codex","harness_version":null,"model":{"provider":"openai","model":"fixture","version":null},"surface":null,"actor":"agent","session_id":null,"written_at":"2026-09-06T00:00:00Z"},"evidence":[],"confidence":0.8,"status":"active","supersedes":[],"tags":[],"created_at":"2026-09-06T00:00:00Z","updated_at":"2026-09-06T00:00:00Z","last_used_at":null,"review_after":null,"archive":null,"procedure":null,"pending_correction_proposal_id":"01890f5e-7b00-7000-8000-000000000001"}
{"id":"01890f5e-7b00-7000-8000-000000000001","obsolete_id":"01890f5e-7b00-7000-8000-000000000002","replacement_id":"01890f5e-7b00-7000-8000-000000000003","memory_kind":"decision","scope":{"type":"project","project_id":null,"project_name":"engram"},"canonical_digest":"ebd471b14169c74083751c4b8ce6fbf44e27f1613c6c34c9b7fb6cf926232a01","digest_schema_version":1,"status":"pending","proposer":{"harness":"codex","harness_version":null,"model":{"provider":"openai","model":"fixture","version":null},"surface":null,"actor":"agent","session_id":null,"written_at":"2026-09-06T00:00:00Z"},"created_at":"2026-09-06T00:00:00Z","applied_at":null}
```

The memory line is 693 bytes with SHA-256
`c5ad92ecdd3a719b61cfca6ef91430a354132c8761c793e5c34e01b6578e3e51`; the pending-proposal line
is 635 bytes with SHA-256
`3b0a8f366d4c7478edcbe6c778f3f4d4c006916ad107706d24c47ca328865563`. Tests construct the typed
values and assert exact bytes, lengths and digests.

Correction digest schema 1 is the existing integrity preimage, not an audit identifier. It is the
SHA-256 lowercase hex digest of `serde_json::to_vec(CorrectionDigestPayload)` with these fields in
this exact order:

```text
schema_version, proposal_id, obsolete_id, replacement_id, memory_kind, scope,
obsolete, replacement
```

`schema_version` is JSON integer `1`. The four ID/kind/scope fields use their exact strict typed
Serde representations. `obsolete` and `replacement` each use this exact field order:

```text
id, kind, title, content, scope, origin, writer, evidence, confidence, status,
supersedes, tags, review_after, archive, procedure, correction_proposal_id,
pending_correction_proposal_id
```

Unlike the embedded `MemoryItem`, both final marker keys are always present here and serialize as
an ID string or JSON `null`. Item `created_at`, `updated_at`, and `last_used_at` are absent. All
other nested values use the exact Serde shapes and declaration orders already frozen above. The
schema-1 golden fixture is this single compact UTF-8 line with no trailing newline:

```json
{"schema_version":1,"proposal_id":"01890f5e-7b00-7000-8000-000000000001","obsolete_id":"01890f5e-7b00-7000-8000-000000000002","replacement_id":"01890f5e-7b00-7000-8000-000000000003","memory_kind":"decision","scope":{"type":"project","project_id":null,"project_name":"engram"},"obsolete":{"id":"01890f5e-7b00-7000-8000-000000000002","kind":"decision","title":"old decision","content":"old content","scope":{"type":"project","project_id":null,"project_name":"engram"},"origin":"user_stated","writer":{"harness":"codex","harness_version":null,"model":{"provider":"openai","model":"fixture","version":null},"surface":null,"actor":"agent","session_id":null,"written_at":"2026-09-06T00:00:00Z"},"evidence":[],"confidence":0.8,"status":"active","supersedes":[],"tags":[],"review_after":null,"archive":null,"procedure":null,"correction_proposal_id":null,"pending_correction_proposal_id":"01890f5e-7b00-7000-8000-000000000001"},"replacement":{"id":"01890f5e-7b00-7000-8000-000000000003","kind":"decision","title":"new decision","content":"new content","scope":{"type":"project","project_id":null,"project_name":"engram"},"origin":"agent_inferred","writer":{"harness":"codex","harness_version":null,"model":{"provider":"openai","model":"fixture","version":null},"surface":null,"actor":"agent","session_id":null,"written_at":"2026-09-06T00:00:00Z"},"evidence":[{"kind":"file","target":"fixture.md","summary":"fixture","excerpt":null,"observed_at":"2026-09-06T00:00:00Z"}],"confidence":0.8,"status":"needs_review","supersedes":[],"tags":[],"review_after":null,"archive":null,"procedure":null,"correction_proposal_id":"01890f5e-7b00-7000-8000-000000000001","pending_correction_proposal_id":null}}
```

It is exactly 1,683 bytes and its SHA-256 is
`ebd471b14169c74083751c4b8ce6fbf44e27f1613c6c34c9b7fb6cf926232a01`. The test constructs the
strict typed values, asserts exact equality with these bytes, asserts the byte count, and then
asserts the digest, so a superficially matching digest string cannot hide a different preimage.

## Exact correction projection

For a pending proposal:

- status is `pending`, with no applied time or digest;
- obsolete and replacement rows both exist and exactly match proposal IDs, kind and scope;
- proposal `proposer` exactly equals the replacement writer;
- obsolete is `active` with exactly the matching pending-proposal lock;
- replacement is `needs_review`, `agent_inferred`, non-procedure, hidden from relay, has exactly the
  matching proposal marker, has no pending-proposal marker, has writer actor exactly `agent`, has
  confidence, tags and review-after exactly equal to the obsolete item, has null archive and
  last-used time, has an empty `supersedes` vector, and retains nonempty non-manual evidence;
- the obsolete item has no replacement-side correction-proposal marker;
- neither item has an evidence target equal to `memory.apply_correction:<proposal-id>`;
- no second pending proposal names that obsolete item; and
- the schema-1 canonical pair digest recomputes exactly.

The obsolete item remains the relay-eligible member until application. A successful validation
mints one move-only binding containing the exact complete pending proposal/obsolete/replacement
rows, typed values and record MACs.

For an applied proposal:

- status is `applied`, with both applied time and applied digest;
- neither pair member participates in any other proposal; proposal chains are invalid;
- proposal `proposer` equals the replacement writer;
- replacement is active, `agent_inferred`, marker-free, has `supersedes` exactly
  `[obsolete_id]`, and contains no manual-review evidence;
- obsolete is superseded and marker-free; and
- the applied pair digest recomputes exactly.

Both item evidence vectors must end in the same exact application `EvidenceRef`: kind `tool_call`,
target `memory.apply_correction:<proposal-id>`, summary
`Operator-selected correction proposal <proposal-id> applied replacement <replacement-id> over
<obsolete-id>`, null excerpt, and identical observed timestamp. No earlier evidence entry may have
that target. C2A accepts an applied proposal only against the unique move-only binding with that ID
and compares the complete strict rows under this sole permitted transition:

```text
proposal embedded/top row:
  status pending -> applied; applied_at null -> current non-null value;
  applied_digest omitted -> current non-null value;
  status_key and snapshot_digest change accordingly; pending_obsolete_id is removed;
  every other field and projection is byte/typed-equal to the bound pending row.

replacement embedded/top row:
  status needs_review -> active; supersedes [] -> [obsolete_id];
  correction_proposal_id present -> omitted;
  append exactly the shared application EvidenceRef; updated_at may advance;
  status_key, snapshot_digest and projected updated_at change accordingly;
  every other field/projection—including created_at, last_used_at, review_after, writer,
  origin, confidence, tags, prior evidence and pending marker—is byte/typed-equal.

obsolete embedded/top row:
  status active -> superseded; pending_correction_proposal_id present -> omitted;
  append exactly the shared application EvidenceRef; updated_at may advance;
  status_key, snapshot_digest and projected updated_at change accordingly;
  every other field/projection—including created_at, last_used_at, existing supersedes,
  prior evidence, correction marker and archive/procedure—is byte/typed-equal.
```

Let `t_e` be the shared application evidence time, `u_r0/u_o0` the bound pending replacement and
obsolete update times, `u_r1/u_o1` the applied update times, and `t_a` proposal `applied_at`.
The exact permitted ordering is `u_r0 <= t_e`, `u_o0 <= t_e`, `t_e <= u_r1`, `t_e <= u_o1`,
`u_r1 <= t_a`, and `u_o1 <= t_a`; every comparison is over parsed instants. C2A then reconstructs
the complete pending rows by reversing only the transitions above and restoring the two bound
pending update strings, and requires canonical-encoded equality with all three bound rows. It
recomputes both persisted snapshot digests, the inverse-pending schema-1 canonical digest, and the
current applied digest. The inverse digest must equal the unchanged proposal canonical digest and
the bound pending value; the current applied digest must equal the current proposal field.

After target projection, the sorted set of target-affecting applied proposal IDs must exactly equal
the sorted binding-ID set: no missing, extra or duplicate binding is accepted. Absence or digest
mismatch, row mismatch, disallowed timestamp relation or unavailable binding is
`AppliedProvenanceUnproven`. Historical applied state without an in-process prior binding fails
closed. C2A proves semantic continuity from its own prior validated value but still does not prove
that the operator action occurred between collections; only the later C1/C2B process authority can
bind that chronology.

## Target identity and scope projection

Exactly one work project must match both target ID and exact name. ASCII-case-fold project-name
twins reject. Every link touching the target by either ID or name must agree on both, and exactly
one target link has role `primary`. That primary must be repository-wide: either component field
being present is `AmbiguousIdentity`, because C2A has no evaluation-CWD/component selector. The
selector's repository, remote, checkout and path must all resolve to that primary linked topology.
Optional task ID/name must resolve to exactly one task under the same project. C2A performs no
filesystem prefix, canonicalization, Git or live discovery. A later component-aware design may add
exact CWD containment and precedence; this slice cannot authorize a component-scoped target.

Every strict row participates in the whole-store MAC. Only these active memories can enter the
target relay-candidate set:

- global and user scope;
- project scope whose every present ID/name selector agrees with the target;
- task scope only when the target supplies an exact task pair and every project/task selector
  resolves to that exact target task under the rule below; and
- repository scope whose every present repository/remote/path selector agrees with the resolved
  target repository and checkout.

Every project scope has a nonempty exact project name and its optional ID, when present, must
resolve to that same target project. A task-scoped memory is never applicable when the target task
pair is absent. When the exact target task pair is present, a task scope has a nonempty exact task
name that resolves as the unique exact task name or exact Jira key for that target task; an
optional task ID must identify the same task, and the scope must additionally contain either that
exact task ID or a matching target project ID/name. A task name alone with null task/project IDs
and null project name is never adopted. Every present selector is conjunctive. An ambiguous
name/Jira-key collision rejects rather than selecting one.

Every repository scope must contain at least one non-null selector; an all-null repository scope
is `ScopeMismatch`. Every present repository ID, normalized remote and lexical checkout path is
conjunctive and must identify the target repository/checkout; no ID-first, remote-second or
path-third fallback is permitted. A selector that touches the target on one field but disagrees on
another is `ScopeMismatch`. The later relay policy must call this exact strict projection and may
not reuse the current ranking fallback semantics.

Entity, session and custom scopes, all task scopes when no target task was supplied, other
projects/tasks/repositories, all non-active items, and
pending replacement rows remain in the store-state MAC but never enter the project-view or
relay-candidate MAC. A partially matching multi-field scope is `ScopeMismatch`, not a match.

## Canonical keyed MACs and sanitized audit

All serializable state authenticators are HMAC-SHA-256 per RFC 2104, implemented directly with
`sha2::Sha256` and one consumed `C2OneShotMacKey`; no extra dependency is authorized. The key is
padded with zeros to the SHA-256 64-byte block size, XORed with `0x36`/`0x5c`, and the result is
`SHA256(opad || SHA256(ipad || message))`. The 32 output bytes are encoded as 64 lowercase hex.
The key and a private wrapper around each 64-byte pad implement `Drop` by calling `Zeroize`; the
validator consumes the key, zeroizes all pads after each HMAC, and has no API that returns or
reuses key material. The key implements
none of `Debug`, `Display`, `Clone`, `Copy`, `Serialize` or `Deserialize`.

Its exact C2A definition is `struct C2OneShotMacKey([u8; 32]);` with a `Drop` implementation that
zeroizes the array. The only C2A constructor is
`#[cfg(test)] fn from_test_bytes(bytes: [u8; 32]) -> Self`; it is private and absent from production
compilation.

C2A contains no production constructor for `C2OneShotMacKey`; its fixed-byte constructor exists
only under `#[cfg(test)]`. C2B is required to add the sole production mint under a separately
frozen source boundary: exactly 32 bytes from an evaluator-owned operating-system CSPRNG, once per
validation, never caller/snapshot/config-derived, persisted, emitted, logged, cloned, retried or
reused across snapshots. Pre/post continuity uses the move-only complete prior binding above, not
MAC equality and not key reuse. A compile/source gate must prove that a production key cannot be
constructed from fixed or caller bytes and that a consumed key cannot validate a second snapshot.
Until C2B proves that mint, C2A's offline-oracle resistance is a conditional design property, not
a live acquisition claim. Under that premise, `_mac` fields are not stable public content
identifiers or offline guessed-value verifiers.

Canonical JSON-value encoding is exact and recursive:

```text
null    = 00
false   = 01
true    = 02
number  = 03 || u64be(byte_len(Number::to_string())) || Number::to_string() UTF-8
string  = 04 || u64be(byte_len) || UTF-8
array   = 05 || u64be(element_count) || encode(each element in stored order)
object  = 06 || u64be(member_count) || for each UTF-8-key-byte-sorted member:
          u64be(key_byte_len) || key UTF-8 || encode(value)
```

No concatenand is an unframed caller string. The golden encoding of JSON value
`{"a":1,"b":false}` is
`0600000000000000020000000000000001610300000000000000013100000000000000016201`;
with the `#[cfg(test)]`-only all-zero 32-byte key its HMAC-SHA-256 is
`a2751529cd3929ac5ff03c72909a40ad401f9cc9cbd1f0ea6f3428a0f0c5841c`.

Rows are sorted by the 16 raw UUID bytes within the fixed table order. Every UUID operand below is
its 16 raw bytes, every MAC operand is its 32 raw bytes, and every tag/status operand is ASCII.
`frame(x)` is `u64be(x.len) || x`; counts are unsigned 64-bit big-endian. `row_json` is the complete strict raw
row, including `record_id`, the embedded payload and every persisted projection, encoded by the
canonical encoder above. The exact MAC preimages are:

```text
record = "engram-native-semantic-record-v1\0" || frame(table_tag) ||
         frame(uuid_16_bytes) || frame(encode(row_json))
table  = "engram-native-semantic-table-v1\0" || frame(table_tag) || u64be(row_count) ||
         repeat(frame(uuid_16_bytes) || frame(record_mac_32_bytes))
store  = "engram-native-semantic-store-v1\0" ||
         repeat(frame(table_tag) || frame(table_mac_32_bytes)) for all nine tables
```

Table tags are the exact nine ASCII table names and order frozen above. The target project-view
preimage is

```text
"engram-native-semantic-project-view-v1\0" ||
frame(target_project_uuid) || frame(target_project_record_mac) ||
frame(target_repository_uuid) || frame(target_repository_record_mac) ||
frame(target_checkout_uuid) || frame(target_checkout_record_mac) ||
option(target_task_uuid, target_task_record_mac) ||
u64be(competing_link_count) || repeat(frame(link_uuid) || frame(link_record_mac)) ||
u64be(linked_component_count) || repeat(frame(component_uuid) || frame(component_record_mac)) ||
u64be(correction_edge_count) || repeat(correction_edge_frame) ||
u64be(applicable_memory_count) || repeat(frame(memory_uuid) || frame(memory_record_mac))
```

`option(None)=00`; `option(Some(id,mac))=01 || frame(id_16_bytes) || frame(mac_32_bytes)`.
Competing links means every link touching the target project ID or exact name, including the
selected primary. Linked components means every unique component referenced by those competing
links, UUID-byte sorted; their record MACs therefore bind component semantics into the project
view even though the selected primary itself must be repository-wide. A correction edge is
target-affecting exactly when its shared scope would be
target-applicable under the rules above without considering either item's lifecycle status; both
pending and applied edges are included. Each `correction_edge_frame` is
`frame(proposal_uuid) || frame(obsolete_uuid) || frame(replacement_uuid) || frame(status_ascii) ||
frame(proposal_record_mac) || frame(obsolete_record_mac) || frame(replacement_record_mac)`.
Applicable memories include global/user plus all target-applicable project/task/repository active
memories. All variable collections in this preimage are UUID-byte sorted.

The relay-candidate preimage is

```text
"engram-native-semantic-relay-candidates-v1\0" || u64be(candidate_count) ||
repeat(frame(memory_uuid) || frame(memory_record_mac))
```

Candidates are the same applicable active memories, UUID-byte sorted. Every strict row contributes
to a record MAC and one table MAC even if it is out of scope. Capture time is absent. Any exact
semantic field or persisted projection change changes the owning record, table and store MACs;
insertion order does not. A project-view-relevant change also changes the project-view MAC, and a
candidate change changes the relay-candidate MAC.

The complete framing goldens use only the `#[cfg(test)]` all-zero key. The record vector is a
`memory_item` row whose `item` is the 693-byte persisted-snapshot golden above and whose remaining
fields are: matching `record_id`, `kind_key="decision"`, `status_key="active"`,
`scope_key="project:engram"`, `harness_key="codex"`, `model_key="fixture"`, null `session_id`,
`snapshot_digest=c5ad92ecdd3a719b61cfca6ef91430a354132c8761c793e5c34e01b6578e3e51`, and both projected
timestamps `2026-09-06T00:00:00Z`. Exact expected values are:

```text
encoded row bytes:       1,432
record preimage bytes:   1,516
record MAC:              a9e0669d85e989b1abb3d77ba797f940792fa58ffffaeda18cd91c2c72104e24
one-row table preimage:    123
memory_item table MAC:   ef7c82bb7927a73db9152a0f935654c54c372428ca4c78e8a8940bc40cae0632
store preimage bytes:      605
store MAC:               a99a636011b9fdca4b709edbe60e5b53b45c5c7a19f441e28cf19a40749a084f
```

That store contains the one memory row and zero rows in every other fixed table; each empty table
MAC is computed by the exact table formula, not replaced by zeros. The project-view primitive
vector uses target IDs ending `000000000010`, `000000000011`, and `000000000012`, respective
synthetic record-MAC bytes repeated `0x11`, `0x22`, and `0x33`, null task, and zero links, linked
components, correction edges and memories. Its preimage is 264 bytes and its MAC is
`18b2228f9af7cc94b603117bb220f1d4148ad34225e1ed11e8a8880aed5e44a8`. The zero-candidate relay
preimage is 51 bytes and its MAC is
`f81ca811ea6d1bcbc9ae94fb1315d3f3841ab07c55083f9e3baad480089abfec`.

The only serializable success value has this exact key set and order; all counts are JSON `u64`,
IDs are canonical UUID text, and every MAC is 64 lowercase hex:

```text
C2SanitizedAudit:
  schema_version=1, target, row_counts, memory_status_counts, proposal_status_counts,
  table_macs, store_state_mac, project_view_mac, relay_candidate_mac,
  correction_edges, relay_candidates
target:
  project_id, repository_id, checkout_id, task_id=null|string
row_counts:
  memory_item, correction_proposal, memory_forget_receipt, work_project, work_task,
  git_repository, local_checkout, monorepo_component, project_repository_link
memory_status_counts:
  active, needs_review, superseded, archived, rejected
proposal_status_counts:
  pending, applied
table_macs:
  memory_item, correction_proposal, memory_forget_receipt, work_project, work_task,
  git_repository, local_checkout, monorepo_component, project_repository_link
correction_edges[]:
  proposal_id, obsolete_id, replacement_id, status,
  proposal_record_mac, obsolete_record_mac, replacement_record_mac
relay_candidates[]:
  memory_id, status, record_mac
```

`correction_edges` contains exactly the target-affecting edges and `relay_candidates` exactly the
applicable active candidate set, both UUID-byte sorted as used by their MAC preimages.

It contains no names, titles, content, evidence text, writer strings, project/partition names,
URLs, local/source/home paths, raw values, capture time, environment, token, capability, or
executable data. It has no field whose name ends `_sha256`; persisted digests remain private. A
later relay may consume content only from the still-private validated snapshot and only under a
separately frozen phase policy; MACs and IDs alone grant no relay authority.

## Required provider-free tests

Tests must cover, at minimum:

1. a valid project/repository/checkout/task snapshot and deterministic sanitized audit under one
   fixed test key, including every byte/count/MAC golden above;
2. row insertion permutations and tied timestamps producing identical MACs under the same key;
3. one-field and nested-vector-order mutations changing the correct MACs;
4. exact-cap acceptance and plus-one rejection for every table, plus row/total/depth/key/vector and
   checked-arithmetic boundaries, and the exact 39-node/747-byte/depth-2 count golden;
5. unknown/missing fields, every `NONE` null/omission permutation, forged inner IDs, duplicate IDs,
   invalid UUID/enums/timestamps/numeric types or ranges, equivalent non-UTC timestamp spellings,
   every exact projection formula, and stale snapshot digests for memory and proposal rows;
6. project ID/name split-brain, case-fold twins, missing project, competing/duplicate primary links,
   repository/remote/checkout/path disagreement and optional task disagreement;
7. every missing/orphan/cross-repository topology edge, path traversal and every exact duplicate
   identity tuple, plus component-scoped primary rejection and linked-component MAC mutation;
8. every pending/applied proposal status, marker, pair, kind, scope, origin, evidence, procedure,
   digest, uniqueness and cycle permutation, including pending proposer/writer mismatch, writer
   actor other than exact `agent`, changed confidence/tags/review-after, non-null replacement
   archive/last-used/pending marker, non-null obsolete replacement marker, nonempty replacement
   supersedes, preexisting same-proposal application evidence, repeated proposal roles, and an
   attempted proposal correction chain;
9. applied proposal without the exact move-only prior binding; acceptance with one consumed binding;
   every forbidden current/prior field mutation; every allowed transition; and every timestamp
   ordering failure;
10. incomplete and completed forget receipts, secret-bearing and malformed receipt precedence,
    dangling supersedes/proposal references, duplicate supersedes, a non-superseded edge target,
    and competing incoming superseders across direct and applied-correction edges;
11. out-of-scope rows remaining in the store MAC but never the project/relay MAC; task memory
    excluded without the target task pair; and task-name-only memory excluded even with a target;
12. synthetic secret canaries in every raw string/key family returning content-free failure before
    any MAC, including remote query and fragment credentials plus colonless user-info;
13. success serialization and error formatting containing none of the raw canaries and no
    serializable content-derived `_sha256` field, with different test keys producing different
    MACs;
14. key/pad drop-zeroization probes, compile/source gates proving the all-zero constructor is
    test-only and no production fixed/caller-byte constructor exists, and a move test proving one
    key cannot validate twice; and
15. bidirectional source firewalls proving the private, pure, non-runnable capability boundary and
    absence of a production caller.

Tests may use deterministic in-memory fixtures only. They may not open a real Engram store, daemon,
socket, provider, authentication cache, filesystem path or live settings. Acceptance requires the
focused suite repeatedly, the full provider-free library and integration suites, strict Clippy,
format, whitespace, external-target, repository-`target/debug`, and three fresh independent exact-
source audits with P0=0 and P1=0.

## Deferred C2B gate

C2B must separately freeze and prove a bounded acquisition owner. It must distinguish local Memory
or evaluator-owned RocksDB from `Surreal<Any>` remote state without trusting a caller boolean,
perform exactly one bare `RETURN` statement that SurrealDB 2.6.0 classifies as non-writeable, check
one response statement before extraction, probe every table at cap plus one, and bound complete raw
row bytes and nodes—including unknown keys and values—before returning raw values. Its acquisition
shape must preserve unknown fields so C2A can reject them. It must prove concurrent readers observe
a complete pre-state or post-state rather than a hybrid. It must own the sole production
`C2OneShotMacKey` mint from exactly 32 operating-system CSPRNG bytes per validation and prove that
the key is never caller-controlled, persisted, emitted, cloned, retried or reused. For a correction
journey it must move the selected prior binding from the successful pre-state value into exactly
one post-state validation and bind both collections plus the operator action to one process
receipt. Because every validation uses a fresh one-shot key, C2B and later phases must never compare
C2A MAC equality across snapshots. Any required unchanged-state comparison must use moved private
canonical state (or a separately frozen, private scoped equality witness derived inside the same
process) before either state is discarded; MACs remain per-validation audit authenticators only.
C2A acceptance alone does not satisfy any of those acquisition, equality or chronology claims.
