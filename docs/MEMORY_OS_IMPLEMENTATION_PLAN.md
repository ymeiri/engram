# Engram Memory OS: Design and Implementation Plan

Status: Implementation in progress
Date: 2026-04-26
Last Updated: 2026-05-06
Audience: Engram maintainers, AI coding-agent users, future contributors
Scope: Whether to extend Engram or build a new system; full design for a local-first AI memory operating system.

---

## Living Implementation Checklist

- [x] Memory OS ontology, writer provenance, lifecycle statuses, and knowledge commits.
- [x] Consolidated MCP `memory(action=commit)` surface; no separate `memory_commit` tool.
- [x] CLI `engram memory log`, `engram memory diff`, `engram memory changes-since`, `engram memory writer-stats`, and archive UX.
- [x] Generated Markdown vault with generated-file marker and Obsidian backlinks.
- [x] Orientation packet with repository/project resolution and deterministic relevance ordering.
- [x] Brain Loop v1 projection inside `orient`; bounded compiled context from already selected
      memory, with graph/obligations/lint kept out of the hot path.
- [x] Prompt-aware Brain Loop top-item calibration; buckets stay balanced, but a strongly
      prompt-matched reviewed decision can lead the bounded context.
- [x] MCP `orient` surfaces already-open agent obligations as a compact summary and recommended
      action without running obligation detection in the hot path.
- [x] `orient` filters stale git-status document obligations and untracked root instruction files
      from its open-obligation summary, while leaving stored obligation lifecycle explicit.
- [x] Review-gated entity observation promotion through `memory(action=promote_observation)`,
      preserving the source observation as evidence instead of widening `orient`.
- [x] Review-gated migration and digest extraction flows; no automatic promotion from orphan, digest, or legacy data.
- [x] Agent harness layer: MCP `harness`, CLI `engram harness`, Claude Code adapter rendering, Codex skill rendering, Gemini CLI custom command/context rendering, Cursor Agent skill rendering, dry-run install by default.
- [x] Agent-native obligations layer: MCP `obligations`, CLI `engram obligations`, lifecycle detection for document disposition, source/design context reading, failed tool-call recovery, tests, handoffs, and commit preference checks.
- [x] Context-compaction lifecycle contract: harness policy and adapters tell agents to update handoffs and persist compact durable memory before expected context loss.
- [x] Lint/recalibration MVP: missing evidence, stale preference, duplicate entity candidate, orphan task/project scope, stale active session, superseded active item, vault marker/frontmatter, handoff missing next actions.
- [x] Archive metadata and archive-aware retrieval; normal orientation uses active memory only.
- [x] Derived graph traversal: MCP/CLI `graph around|path|subgraph|export`.
- [x] Multi-writer changes_since filters and relevance reasons for harness/model/surface/session-aware polling.
- [x] Rolling handoff: MCP/CLI `handoff get|update|compile`.
- [x] Session-following event vocabulary: prompts, plans, tool results, tests, preferences, rules, limitations, and handoff updates.
- [x] Session distillation dry-run candidate generation through `memory(action=distill_session)` and `engram memory distill-session`.
- [x] Full validation and live daemon smoke from installed binary.
- [ ] Migration completion run: inventory, review export, prioritize/dedupe, human review, dry-run apply, approved write apply, knowledge commit, vault compile, lint run.

Current MCP/CLI Memory OS surface:

- MCP: `orient`, `memory`, `harness`, `obligations`, `lint`, `graph`, `handoff`, `vault`, `digest`, `repo`.
- CLI: `engram orient`, `engram memory`, `engram harness`, `engram obligations`, `engram lint`, `engram graph`, `engram handoff`, `engram vault`, `engram digest`, `engram repo`.

Migration safety rule:

No orphan, digest, or legacy Engram data is automatically promoted to active memory. Promotion requires review decisions, dry-run apply, approved write apply, and a knowledge commit.

---

## 1. Executive Summary

Engram should be extended, not replaced from scratch.

The current Engram codebase already has valuable foundations: a Rust workspace, MCP server, CLI, SurrealDB-backed persistence, session history, entity knowledge, document indexing, tool intelligence, coordination, knowledge management, and work management. Rebuilding those from zero would mostly recreate solved infrastructure.

However, the current product shape is too tool-centric. It behaves like a memory database that agents may use if prompted. The desired product is a memory environment that agents naturally orient through, update while working, lint for correctness, and expose as a human-readable Obsidian-compatible Markdown library.

The recommended path is a staged extension:

```text
Keep:
  current Engram MCP, CLI, storage, sessions, entities, work management

Add:
  Memory OS layer
  Markdown vault
  knowledge commits
  ontology and graph traversal
  lint and recalibration
  automatic orientation
  live session distillation

Avoid:
  a ground-up rewrite before the desired behavior is proven
```

The strategic shift is:

```text
Old framing:
  Persistent memory for AI coding agents.

New framing:
  Git-like, source-grounded, Obsidian-readable memory for AI coding agents.
```

In the new model:

- Markdown is the durable, human-editable memory surface.
- SurrealDB is the index, graph, and query engine.
- MCP is the agent interface.
- The CLI is the human/operator interface.
- Lint keeps memory current, sourced, non-contradictory, and scoped.
- Knowledge commits record how project understanding changes over time.

---

## 2. Why This Needs To Change

### 2.1 Current Daily Workflow Pain

Observed user pain:

1. The user has to remind the agent every few prompts to use Engram.
2. Session start/end commands are too manual.
3. Project and subproject identity gets confused, especially when working in notes folders.
4. The entity graph exists, but it does not drive behavior enough.
5. RAG/embeddings retrieve snippets, but daily coding needs a durable narrative.
6. Preferences and working agreements are discovered but not reliably retained.
7. Old knowledge is not archived, superseded, or recalibrated in a clear way.

The result is that Engram can store memory, but it does not yet behave like a persistent brain.

### 2.2 The Product Gap

Current shape:

```text
Agent works
  -> user reminds agent to log something
  -> agent may call MCP
  -> fact goes into database
  -> future retrieval depends on search/tool use
```

Desired shape:

```text
User prompts agent
  -> agent automatically orients through Engram
  -> Engram identifies project/task/scope
  -> Engram returns current context pack
  -> work happens
  -> Engram follows session events
  -> session is distilled into knowledge commits
  -> Markdown vault and graph are updated
  -> lint checks contradictions, staleness, scope, and provenance
```

### 2.3 Why RAG Is Not Enough

Modern models have very large context windows. This changes the value of embeddings.

Embeddings are still useful for discovery:

- Find candidate memories.
- Search old sessions.
- Search long documents.
- Retrieve similar events.

But embeddings should not be the main user experience.

Daily coding requires compiled context:

- What is this project?
- What subproject am I in?
- What decisions are active?
- What rules apply?
- What does the user prefer?
- What changed since the last related work?
- Which old facts are stale or superseded?

Compiled Markdown pages and knowledge commits answer these better than raw RAG snippets.

---

## 3. Extend Engram Or Build New?

### 3.1 Decision

Recommendation: extend Engram.

Build the new Memory OS as a new subsystem inside Engram, with new crates/modules and clear interfaces. Do not rewrite the existing system before proving the new behavior.

### 3.2 Decision Matrix

| Option | Benefits | Costs | Verdict |
|---|---|---|---|
| Extend Engram in-place | Reuses MCP, CLI, SurrealDB, sessions, entities, tests | Must manage legacy concepts and migration | Recommended |
| Build new system from scratch | Clean design, no legacy constraints | Rebuilds storage, MCP, CLI, tests, daemon, project model | Not worth it now |
| Fork Engram into a new product | Clean branding and architecture | Splits attention and compatibility | Consider later only if Memory OS outgrows current product |
| Add only an Obsidian export | Fastest visible improvement | Does not solve graph, lint, orientation, or lifecycle | Useful but insufficient |
| Add only more MCP tools | Easy incremental work | Keeps user reminding agent to use tools | Insufficient |

### 3.3 Why Extend

Engram already has:

- Rust workspace structure.
- Domain types.
- SurrealDB repository layer.
- MCP server.
- CLI.
- Session history.
- Work management.
- Knowledge registry.
- Entity relationships.
- Document indexing.
- Tests for most layers.

The desired system needs all of these. The missing parts are higher-level memory lifecycle, not the base infrastructure.

### 3.4 How To Avoid Legacy Drag

Add a new boundary:

```text
engram-memory
  memory ontology
  knowledge commits
  vault compiler
  orientation service
  lint service
  distillation service
```

This can live initially inside `engram-index` and `engram-core`, but the design should keep it separable.

Suggested final crate shape:

```text
engram-core
  existing domain types
  new memory ontology types

engram-store
  existing repos
  new memory/vault/lint repos

engram-index
  existing services
  new MemoryService, OrientationService, VaultService, LintService

engram-mcp
  existing tools
  new orient, memory, lint, vault, graph tools

engram-cli
  existing commands
  new orient, memory, vault, lint, graph commands
```

---

## 4. Target Product

### 4.1 One-Sentence Product Definition

Engram Memory OS is a local-first, source-grounded, Obsidian-readable memory system that orients AI coding agents, tracks how project knowledge evolves, and lints memory for freshness, scope, contradictions, and provenance.

### 4.2 Core Principles

1. The agent should orient automatically.
2. Markdown is the durable human-readable memory.
3. Raw evidence is immutable.
4. Compiled memory is editable and linted.
5. Every important fact needs provenance.
6. Memory is typed and scoped.
7. Old knowledge is archived, not lost.
8. Preferences need lifecycle and recalibration.
9. Graph relationships must drive behavior.
10. Search should be hybrid: keyword, vector, graph, time, and scope.

### 4.3 What This Is Not

This is not just:

- A vector database.
- A chat transcript store.
- A generic note-taking app.
- A hidden agent memory blob.
- A replacement for Git.
- A replacement for Obsidian.

It is a bridge between:

- raw evidence,
- compiled project memory,
- graph structure,
- human-readable notes,
- and agent runtime context.

---

## 5. System Architecture

### 5.1 High-Level Architecture

```text
User prompt
    |
    v
Agent calls orient
    |
    v
Orientation Service
    |        |          |          |
    |        |          |          |
    v        v          v          v
Project   Graph      Vault      Knowledge
Identity  Index      Pages      Commits
    |        |          |          |
    +--------+----------+----------+
             |
             v
       Context Pack
             |
             v
      Agent does work
             |
             v
      Live Event Stream
             |
             v
      Session Distiller
             |
             v
      Knowledge Commit
             |
      +------+-------+
      |              |
      v              v
   Vault Update    Graph Update
      |              |
      +------+-------+
             |
             v
          Lint
```

### 5.2 Storage Layers

```text
Raw evidence layer
  append-only events, session logs, command results, file refs, external refs

Compiled Markdown layer
  Obsidian-compatible vault pages

Graph/index layer
  entities, relationships, scopes, timestamps, embeddings, provenance

Knowledge commit layer
  Git-like memory diffs between project knowledge states

Runtime context layer
  generated context packs for agents
```

### 5.3 Relationship To Existing Engram

Existing Engram layers map naturally:

| Existing Layer | Memory OS Use |
|---|---|
| Entity Knowledge | Becomes graph backbone |
| Session History | Raw evidence and episodic memory |
| Document Knowledge | Source ingestion and retrieval |
| Tool Intelligence | Procedural/tool preference memory |
| Session Coordination | Live agent awareness |
| Knowledge Management | Vault/source registry foundation |
| Work Management | Project/task/subproject scope |

---

## 6. Tech Stack

### 6.1 Recommended Stack

| Concern | Technology |
|---|---|
| Core language | Rust |
| Persistence | SurrealDB embedded RocksDB initially |
| Graph/index | SurrealDB tables now; optional native graph traversal later |
| Markdown vault | Plain Markdown files, Obsidian-compatible links |
| MCP interface | Existing `rmcp` stack |
| CLI | Existing `clap` CLI |
| Embeddings | Existing `fastembed` path |
| Reranking | Start with deterministic scoring; add local/LLM reranking later |
| Lint rules | Rust rule engine with optional LLM-assisted checks |
| Session event capture | MCP calls plus optional shell/git/file watcher hooks |
| Frontend | Obsidian first; local web UI later |

### 6.2 Why Obsidian First

Building a polished graph browser is expensive. Obsidian gives immediate value:

- readable Markdown,
- backlinks,
- graph view,
- manual edits,
- local-first workflow,
- no new UI stack,
- easy user trust.

Engram should compile and index the vault; Obsidian should browse it.

### 6.3 Future UI

After the vault model is stable, add a local dashboard:

```text
engram dashboard
  project overview
  active tasks
  graph explorer
  lint findings
  stale memory
  recent knowledge commits
```

Do not build this before the memory model is proven.

---

## 7. Memory Ontology

### 7.1 Why Ontology Matters

Without ontology, memory becomes a bag of notes.

The system needs to distinguish:

- a user preference,
- a project rule,
- a decision,
- a limitation,
- an open question,
- a task,
- a session event,
- and a source document.

These need different fields, lifecycles, retrieval rules, and lint checks.

### 7.2 Core Node Types

```text
User
Agent
Workspace
Project
Subproject
Task
Session
Entity
Decision
Rule
Procedure
Preference
Limitation
OpenQuestion
Evidence
SourceDocument
KnowledgeCommit
Handoff
Tool
Command
TestRun
CodeFile
ExternalReference
```

### 7.3 Core Edge Types

```text
parent_of
subproject_of
contains
touches
modifies
creates
depends_on
uses
owned_by
documented_by
implements
decided_by
derived_from
evidenced_by
supersedes
superseded_by
contradicts
resolves
blocks
applies_to
triggered_by
mentioned_in
archived_from
loaded_by
```

### 7.4 Memory Types

Use three memory classes:

```text
Semantic memory
  facts, preferences, rules, decisions, limitations

Episodic memory
  session events, tool calls, failures, user corrections, commands

Procedural memory
  how the agent should act, project workflows, preferred tool usage
```

Examples:

```text
User dislikes AI-branded commit messages
  -> Semantic Preference

The Google Workspace MCP returned 401 invalid_token
  -> Episodic Event / Limitation Evidence

When OAuth is needed, use browser login with user assistance
  -> Procedural Rule
```

### 7.5 Required Fields By Type

#### Preference

```text
id
statement
scope
applies_to
evidence_refs
confidence
sensitivity
created_at
last_used_at
usage_count
review_after
status
```

#### Decision

```text
id
title
decision
rationale
alternatives_rejected
scope
evidence_refs
supersedes
status
created_at
updated_at
```

#### Rule

```text
id
statement
scope
trigger
strength
evidence_refs
status
review_after
```

#### Limitation

```text
id
statement
cause
workaround
scope
evidence_refs
status
recheck_after
```

#### OpenQuestion

```text
id
question
scope
owner
evidence_refs
created_at
status
resolved_by
```

#### KnowledgeCommit

```text
id
parent_id
project_id
scope_id
session_id
title
summary
added_memory_ids
changed_memory_ids
superseded_memory_ids
archived_memory_ids
vault_files_changed
graph_changes
evidence_refs
created_at
author_agent
lint_status
```

---

## 8. Project And Subproject Model

### 8.1 Problem

The current system can confuse a notes folder, a repo, a high-level project, and a sub-feature.

This is common:

```text
~/projects/engram
~/notes/engram
~/notes/ai-memory
~/notes/engram/memory-os
```

These should not automatically become unrelated projects.

### 8.2 Target Hierarchy

```text
Workspace
  Project
    Initiative
      Subproject
        Task
          Session
```

Example:

```text
Workspace: local-yuval
Project: Engram
Initiative: Memory OS
Subproject: Obsidian Vault Compiler
Task: Design knowledge commit schema
Session: 2026-04-26 planning
```

### 8.3 Project Identity Signals

Engram should infer project identity from:

```text
git root
git remote
current branch
current working directory
AGENTS.md
CLAUDE.md
package/Cargo metadata
existing Engram project records
vault path
recent sessions
user prompt
active task
```

### 8.4 Orientation Should Return Ambiguity

If identity is ambiguous, Engram should not silently create a new project.

Example response:

```json
{
  "status": "ambiguous",
  "candidates": [
    {
      "kind": "project",
      "name": "Engram",
      "confidence": 0.84,
      "reason": "cwd is inside repo with matching git remote"
    },
    {
      "kind": "subproject",
      "name": "Memory OS",
      "parent": "Engram",
      "confidence": 0.68,
      "reason": "prompt mentions memory architecture and recent notes path"
    }
  ],
  "recommended_action": "attach_to_existing_subproject"
}
```

The agent can then ask:

```text
This looks related to Engram / Memory OS. Should I attach this work there?
```

### 8.5 Rules For New Project Creation

Only create a new project when:

1. No existing project matches with confidence above threshold.
2. The user explicitly indicates a new standalone project.
3. Git remote and workspace path are new.
4. No parent project is semantically close.

Otherwise create or attach a subproject/task.

---

## 9. Repository Topology And Monorepo Awareness

### 9.1 Problem

Real projects are not always one Git repository.

Example:

```text
Project: Debug with AI
  Repo: co-gen backend
  Repo: WebUI
  Repo: dd-source
    Type: monorepo
    Relevant internal areas:
      path/to/debug-ai-service
      path/to/shared-platform-code
      path/to/frontend-or-infra-surface
```

Engram needs to remember:

- original Git remote,
- local checkout path,
- worktree paths,
- monorepo internal components,
- which subproject touches which repo/component,
- which agent/session/task modified or investigated which area,
- and whether two local folders are checkouts of the same remote repo.

Without this, a notes folder, repo folder, monorepo subdirectory, and project can be mistaken for separate projects.

### 9.2 Repository Ontology

Add repository-specific node types:

```text
GitProvider
GitRepository
LocalCheckout
GitWorktree
MonorepoComponent
Branch
Commit
PullRequest
CodeOwner
BuildTarget
Service
Package
```

Add repository-specific edge types:

```text
hosted_on
checkout_of
worktree_of
contains_component
component_of
implements
owned_by
builds
deploys
touches
depends_on
uses
modified_by
investigated_by
source_of_truth_for
```

### 9.3 GitRepository Schema

```rust
pub struct GitRepository {
    pub id: Id,
    pub name: String,
    pub provider: GitProvider,
    pub remote_url: String,
    pub normalized_remote: String,
    pub default_branch: Option<String>,
    pub repo_kind: RepoKind,
    pub owner: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

pub enum RepoKind {
    SingleProject,
    Monorepo,
    Unknown,
}
```

### 9.4 LocalCheckout Schema

```rust
pub struct LocalCheckout {
    pub id: Id,
    pub repo_id: Id,
    pub path: PathBuf,
    pub git_root: PathBuf,
    pub current_branch: Option<String>,
    pub current_head: Option<String>,
    pub is_worktree: bool,
    pub last_seen_at: OffsetDateTime,
}
```

### 9.5 MonorepoComponent Schema

```rust
pub struct MonorepoComponent {
    pub id: Id,
    pub repo_id: Id,
    pub name: String,
    pub root_path: PathBuf,
    pub component_type: ComponentType,
    pub owners: Vec<String>,
    pub build_targets: Vec<String>,
    pub tags: Vec<String>,
}
```

### 9.6 Example Graph

```text
Project: Debug with AI
  --uses_repo--> GitRepository: co-gen-backend
  --uses_repo--> GitRepository: webui
  --uses_repo--> GitRepository: dd-source

GitRepository: dd-source
  --contains_component--> MonorepoComponent: debug-ai-service
  --contains_component--> MonorepoComponent: shared-llm-platform

LocalCheckout: /Users/yuval/src/dd-source
  --checkout_of--> GitRepository: dd-source

Subproject: Debug with AI / Investigation UX
  --touches--> GitRepository: webui
  --touches--> MonorepoComponent: debug-ai-service
```

### 9.7 Repo Detection

When orienting, Engram should collect:

```bash
git rev-parse --show-toplevel
git remote -v
git branch --show-current
git rev-parse HEAD
git worktree list
```

Then normalize remotes:

```text
git@github.com:DataDog/dd-source.git
https://github.com/DataDog/dd-source
ssh://git@github.com/DataDog/dd-source.git

all become:
github.com/DataDog/dd-source
```

### 9.8 Orientation Behavior

If cwd is inside a monorepo, Engram should not only identify the repo. It should identify the relevant component.

Example:

```json
{
  "project": "Debug with AI",
  "repo": "dd-source",
  "repo_kind": "monorepo",
  "component": "debug-ai-service",
  "local_checkout": "/Users/yuval/src/dd-source",
  "confidence": 0.86
}
```

### 9.9 Migration Of Existing Repo Knowledge

Migration should cluster existing memory by:

```text
git remote
local path
repo name mentions
project name mentions
file path prefixes
branch names
PR URLs
```

Any migrated memory with unclear repo identity should be quarantined:

```text
80_Archive/Needs-Review/Repo-Identity.md
```

---

## 10. Markdown Vault Design

### 10.1 Vault Location

Default:

```text
~/.engram/vault/
```

Configurable:

```toml
[vault]
path = "/Users/yuval.meiri/notes/engram"
format = "obsidian"
```

### 10.2 Folder Structure

```text
engram-vault/
  00_Inbox/
    Untriaged.md

  10_Projects/
    Engram/
      Overview.md
      Context-Pack.md
      Decisions.md
      Rules.md
      Limitations.md
      Open-Questions.md
      Handoffs.md
      Entities.md
      Features/
        Memory-OS.md
        Multi-Session.md
        Google-Workspace-MCP.md

  20_Areas/
    AI-Coding/
      Overview.md
      Agent-Memory.md

  30_Procedures/
    Git.md
    Code-Review.md
    Testing.md
    Browser-Login.md

  40_Preferences/
    User-Profile.md
    Communication.md
    Coding-Style.md
    Tool-Use.md

  50_Knowledge/
    Concepts/
    Systems/
    Patterns/

  80_Archive/
    Projects/
    Sessions/
    Superseded-Decisions/

  90_Evidence/
    Sessions/
    Commands/
    External-Docs/
    Commits/

  95_Knowledge_Commits/
    2026/
      2026-04-26-engram-memory-os.md

  99_System/
    Ontology.md
    Lint-Rules.md
    Vault-Index.md
```

### 10.3 Markdown Page Frontmatter

Every generated page should include YAML frontmatter.

Example:

```yaml
---
engram_id: mem_01
type: preference
scope: global
status: active
confidence: high
created_at: 2026-04-26T10:00:00Z
updated_at: 2026-04-26T10:00:00Z
review_after: 2026-07-26
evidence:
  - session:2026-04-26-memory-os
tags:
  - engram/preference
  - user/communication
---
```

### 10.4 User Preference Page Example

```markdown
# Communication Preferences

## Active Preferences

### Avoid AI-branded commit messages

The user does not want commit messages or summaries that reference the AI
agent's name as the author or brand.

- Scope: global coding work
- Confidence: high
- Evidence: [[2026-04-26-memory-os-session]]
- Last used: 2026-04-26
- Review after: 2026-07-26

### Be explicit about blockers

When a requested action cannot be completed, state the blocker directly and
do not imply success.

- Scope: global
- Confidence: high
- Evidence: Google Docs publishing blocker during Engram analysis
```

### 10.5 Project Context Pack Example

```markdown
# Context Pack: Engram / Memory OS

Generated: 2026-04-26T12:00:00Z

## Current Focus

Design Engram Memory OS: an Obsidian-compatible, source-grounded memory layer
with knowledge commits, graph traversal, linting, and automatic orientation.

## Active Decisions

- Extend Engram rather than rebuild from scratch.
- Markdown vault is durable memory.
- SurrealDB is the graph/index layer.
- MCP is the agent interface.
- Lint is mandatory for memory quality.

## Relevant Preferences

- Do not require the user to remind the agent to use Engram.
- Prefer direct, implementation-oriented plans.
- Record blockers explicitly.

## Known Limitations

- Current Engram graph supports single-hop entity relationships but not
  graph browsing or multi-hop traversal.
- Current multi-session tests have known failures.

## Open Questions

- Should the vault live under `~/.engram/vault` or user-selected notes folder?
- Should generated pages be overwritten directly or updated through patch blocks?
```

### 10.6 Generated Vs User-Edited Sections

Pages should support safe update markers:

```markdown
# Rules

<!-- engram:generated:start rules-active -->
Generated content here.
<!-- engram:generated:end rules-active -->

## User Notes

Human edits here. Engram must not overwrite this section.
```

This allows the user to edit Obsidian pages without Engram destroying manual notes.

---

## 11. Knowledge Commits

### 11.1 Concept

Knowledge commits are Git-like commits for memory.

They answer:

```text
What did we learn during this session?
What changed in the project understanding?
What was superseded?
What evidence supports the change?
Which vault pages changed?
Which graph edges changed?
```

### 11.2 Difference From Session Logs

Session logs are raw evidence:

```text
At 10:31, command ran.
At 10:35, user corrected preference.
At 10:42, agent found internal docs.
```

Knowledge commits are distilled diffs:

```text
Added preference:
  User wants Engram orientation to be automatic, not manual.

Changed decision:
  "resume session" should become internal orientation behavior.

Added limitation:
  Google Workspace MCP requires OAuth reset when stale token appears.
```

### 11.3 Knowledge Commit Markdown

```markdown
# Knowledge Commit: Engram Memory OS Direction

Date: 2026-04-26
Project: [[Engram]]
Scope: [[Memory OS]]
Parent: [[2026-04-25-engram-project-analysis]]

## Summary

Reframed Engram from an MCP memory database into a local-first memory OS
with an Obsidian-readable vault, knowledge commits, graph ontology, and lint.

## Added

- Preference: Agent should orient automatically through Engram.
- Decision: Extend Engram rather than rebuild from scratch.
- Architecture: Markdown vault is durable memory; DB is index/graph.

## Changed

- Session resume is no longer a user-facing primitive; it becomes orient.
- RAG is discovery support, not the primary memory surface.

## Superseded

- "Manual start/resume/end is enough" is superseded by live orientation
  and session-following brain.

## Evidence

- [[session-2026-04-26-memory-os-discussion]]
- [[engram-project-book-2026-04-25]]

## Vault Files Changed

- [[10_Projects/Engram/Features/Memory-OS]]
- [[40_Preferences/Communication]]
- [[99_System/Ontology]]
```

### 11.4 Storage Model

Store in DB and vault.

DB:

```text
knowledge_commit
knowledge_commit_change
knowledge_commit_evidence
```

Vault:

```text
95_Knowledge_Commits/YYYY/YYYY-MM-DD-title.md
```

### 11.5 Diff Types

```text
added_fact
changed_fact
superseded_fact
archived_fact
added_decision
changed_rule
added_preference
updated_preference
resolved_question
added_question
graph_edge_added
graph_edge_removed
vault_page_changed
```

---

## 12. Graph Structure

### 12.1 Current Engram Graph

Current Engram has:

- entities as nodes,
- relationships as directed edges,
- relationship types such as `depends_on`, `uses`, `owned_by`, `documents`, and `related_to`,
- incoming/outgoing single-hop retrieval.

Current limitation:

- no graph browser,
- no multi-hop traversal MCP tool,
- no graph lint,
- project/task entity connections are separate from the entity graph,
- relationships do not yet drive orientation enough.

### 12.2 Target Graph

Graph should become the memory backbone.

```text
               +----------------+
               | User           |
               +-------+--------+
                       |
                       | has_preference
                       v
               +-------+--------+
               | Preference     |
               +-------+--------+
                       |
                       | applies_to
                       v
+------------+  parent_of   +------------+  contains  +------------+
| Workspace  +-------------> | Project    +----------> | Subproject |
+------------+              +-----+------+            +-----+------+
                                  |                         |
                                  | has_decision            | has_task
                                  v                         v
                            +-----+------+            +-----+------+
                            | Decision   |            | Task       |
                            +-----+------+            +-----+------+
                                  |                         |
                                  | evidenced_by            | touches
                                  v                         v
                            +-----+------+            +-----+------+
                            | Evidence   |            | Entity     |
                            +------------+            +------------+
```

### 12.3 Graph Node Tables

Initial tables:

```text
memory_node
memory_edge
memory_fact
memory_preference
memory_decision
memory_rule
memory_limitation
memory_question
memory_evidence
knowledge_commit
vault_page
```

Alternative:

Use typed tables for each node. This gives stronger validation but more migrations.

Recommendation:

Start with typed domain tables for high-value concepts and a general `memory_edge` table for relationships.

### 12.4 Memory Edge Schema

```rust
pub struct MemoryEdge {
    pub id: Id,
    pub source_id: Id,
    pub source_type: MemoryNodeType,
    pub target_id: Id,
    pub target_type: MemoryNodeType,
    pub relation: MemoryRelation,
    pub scope_id: Option<Id>,
    pub evidence_refs: Vec<Id>,
    pub confidence: Confidence,
    pub status: MemoryStatus,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}
```

### 12.5 Graph Compilation

Graph compilation is the process of turning raw evidence and Markdown into nodes and edges.

Inputs:

```text
session events
user corrections
tool results
files touched
docs indexed
vault pages
git commits
manual notes
```

Compiler stages:

```text
1. Parse evidence
2. Extract candidate memories
3. Classify by ontology type
4. Resolve entities
5. Assign scope
6. Create or update nodes
7. Create or update edges
8. Write knowledge commit
9. Update vault pages
10. Run lint
```

### 12.6 Entity Resolution

Entity resolution should prevent duplicates.

Signals:

```text
exact name
aliases
git remote
file path
URL
semantic similarity
parent project
entity type
recent usage
```

Example:

```text
"Google Workspace MCP"
"datadog-google-workspace"
"GWS MCP"

Same canonical entity:
  Entity: Google Workspace MCP Server
```

### 12.7 Project/Subproject Edges

Example:

```text
Engram --contains--> Memory OS
Memory OS --contains--> Obsidian Vault Compiler
Memory OS --has_decision--> Extend Engram Rather Than Rebuild
Obsidian Vault Compiler --modifies--> Vault Pages
User --has_preference--> Avoid Manual Resume Ritual
Preference --applies_to--> AI Coding Agents
```

### 12.8 Supersession Edges

Example:

```text
Decision: Use SurrealDB server mode
  --superseded_by-->
Decision: Use Engram HTTP daemon + stdio proxy
```

This lets retrieval avoid stale active context while preserving history.

### 12.9 Graph Traversal Examples

Find current context for task:

```text
Task
  -> parent subproject
  -> parent project
  -> active decisions
  -> active rules
  -> limitations
  -> relevant preferences
  -> recent knowledge commits
```

Find what a file is connected to:

```text
CodeFile
  <- touches - Task
  <- modifies - KnowledgeCommit
  <- applies_to - Decision
```

Find stale docs:

```text
SourceDocument
  -> documents Entity
  <- modified_by CodeCommit
  -> older_than active implementation evidence
```

---

## 13. Retrieval Architecture

### 13.1 Retrieval Should Be Hybrid

Use multiple retrieval paths:

```text
scope retrieval
keyword search
vector search
graph traversal
recency search
knowledge commit diff search
vault page lookup
```

### 13.2 Orientation Retrieval

`orient` should not simply search the prompt.

It should build context in layers:

```text
1. Identify workspace/project/task
2. Load project context pack
3. Load active subproject page
4. Load active decisions/rules/limitations
5. Load relevant preferences
6. Load recent knowledge commits
7. Search for prompt-specific facts
8. Rerank and trim
9. Return context pack and citations
```

### 13.3 Candidate Generation

Candidate sources:

```text
exact project match
active task
recent sessions
vault page frontmatter
graph neighbors
keyword matches
embedding matches
open questions
lint warnings
known limitations
```

### 13.4 Reranking

Initial deterministic score:

```text
score =
  semantic_similarity * 0.25
  + keyword_score * 0.15
  + scope_match * 0.25
  + graph_distance_score * 0.15
  + recency_score * 0.10
  + confidence_score * 0.05
  + active_status_score * 0.05
```

Hard filters:

```text
exclude superseded unless specifically asking history
exclude archived unless needed
exclude sensitive memories unless allowed by task type
exclude low-confidence preferences from hard rules
```

### 13.5 Reranking With LLM

Optional second-stage rerank:

```text
Input:
  user prompt
  project identity
  candidate memories with source and scope

Output:
  include
  exclude
  reason
  priority
```

This should be optional because it adds latency and model dependency.

### 13.6 Context Pack Format

MCP `orient` response:

```json
{
  "project": "Engram",
  "scope": "Engram / Memory OS",
  "confidence": 0.91,
  "context_pack_path": "/Users/.../Context-Pack.md",
  "summary": "...",
  "active_decisions": [],
  "active_rules": [],
  "preferences": [],
  "limitations": [],
  "open_questions": [],
  "recent_knowledge_commits": [],
  "recommended_actions": [],
  "ambiguities": []
}
```

---

## 14. Session-Following Brain

### 14.1 Remove Manual Resume As A User Ritual

Users should not need to say "resume session."

Every meaningful prompt should trigger:

```text
orient(cwd, prompt, agent, recent_files)
```

The agent should ask Engram:

```text
Where am I?
What do we know?
What changed recently?
What rules apply?
What should I avoid?
```

### 14.2 Session Lifecycle

Keep sessions internally, but change user-facing behavior.

```text
Implicit start:
  first orient call creates or attaches active session

During work:
  event stream accumulates evidence

Periodic distill:
  every N tool events, major user correction, or task boundary

Explicit end optional:
  writes final handoff and knowledge commit

Idle close:
  stale sessions are closed automatically with partial handoff
```

### 14.3 Event Capture

Event types:

```text
user_prompt
assistant_plan
tool_call
tool_result
file_read
file_write
command_run
test_result
error
decision_candidate
preference_candidate
rule_candidate
limitation_candidate
handoff_update
```

### 14.4 What To Record

Do not save everything as durable memory.

Record raw events cheaply. Promote only useful items.

Promotion triggers:

```text
user says "remember", "do not", "I prefer", "we decided"
agent changes approach due to discovered limitation
test failure reveals project constraint
external doc resolves a blocker
manual correction to style/workflow
new project/subproject identity inferred
decision affects future work
tool behavior is reusable
```

### 14.5 Rolling Handoff

Instead of handoff only at the end, maintain a rolling handoff.

```markdown
# Active Handoff: Engram / Memory OS

## Current State

Designing Memory OS architecture and implementation plan.

## Decisions So Far

- Extend Engram rather than rebuild.
- Add Markdown vault as durable memory.
- Add knowledge commits for memory diffs.

## Next Actions

- Implement orientation service.
- Implement vault compiler.
- Implement lint MVP.

## Open Questions

- Vault default path.
- Whether generated sections should be patch-only or full page rewrites.
```

---

## 15. Multi-Writer Provenance And Session-Time Memory Awareness

### 15.1 Problem

Different models and harnesses may write to the same memory system.

Examples:

```text
Agent A: Claude Code explores the backend failure.
Agent B: Codex analyzes the frontend state.
Agent C: ChatGPT reviews the design plan.
```

They may be:

- using different harnesses,
- using different models,
- working from different directories,
- or writing to the same memory surface.

Memory OS must know who wrote what and when. It does not need to become a collaboration platform in v1.

### 15.2 First Principle

Separate memory provenance from agent collaboration.

The required v1 behavior is:

```text
Claude Code wrote/explored these facts.
Codex wrote/explored these facts.
ChatGPT contributed these analysis notes.
This memory was user-stated.
This memory was tool-observed.
This memory was agent-inferred.
```

The system should not try to coordinate agents like a team chat or task delegation platform yet.

### 15.3 Writer Provenance

Every memory item, event, evidence record, and knowledge commit should record writer provenance.

```rust
pub struct WriterProvenance {
    pub actor_type: ActorType,
    pub harness: Option<String>,
    pub model: Option<String>,
    pub surface: Option<String>,
    pub session_id: Option<Id>,
    pub workspace_id: Option<Id>,
    pub origin: ClaimOrigin,
    pub created_at: OffsetDateTime,
}
```

Actor types:

```text
user
agent
tool
system
migration
```

Harness examples:

```text
claude-code
codex
chatgpt
cursor
unknown
```

Origin examples:

```text
user_stated
user_corrected_agent
agent_inferred
tool_observed
code_observed
document_observed
migrated
lint_generated
```

This is enough to distinguish:

```text
Claude Code found this in dd-source.
Codex generated this frontend implementation note.
ChatGPT summarized this design tradeoff.
The user explicitly corrected this preference.
```

### 15.4 Trust And Retrieval Implications

Writer provenance should influence retrieval and lint.

Trust order:

```text
highest:
  user_stated
  user_corrected_agent
  tool_observed
  code_observed

medium:
  document_observed
  agent_distilled_with_evidence

lower:
  agent_inferred_without_evidence
  migrated_unsourced
```

Retrieval should show source attribution:

```text
Preference: Avoid AI-branded commit messages
Origin: user_corrected_agent
Written by: Claude Code / unknown model
Scope: global coding work
Confidence: high
```

### 15.5 Session-Time Memory Change Awareness

The important multi-writer problem is not collaboration. It is awareness.

After an agent receives its initial context pack, another agent may write a relevant memory item during the same session.

Engram should support:

```text
memory changes since orientation
memory changes relevant to current project/scope
memory changes written by other harnesses
memory changes that supersede loaded context
memory changes that contradict loaded context
```

### 15.6 Memory Cursor

Every `orient` response should include a memory cursor.

```json
{
  "context_pack_path": "...",
  "memory_cursor": "kc_2026_04_26_00042"
}
```

During a session, the agent can ask:

```json
{
  "action": "changes_since",
  "cursor": "kc_2026_04_26_00042",
  "project": "Engram",
  "scope": "Memory OS"
}
```

Response:

```json
{
  "changes": [
    {
      "type": "new_limitation",
      "title": "Claude Code Google Workspace MCP has stale OAuth token",
      "writer": {
        "harness": "claude-code",
        "model": "claude-opus-4.7"
      },
      "relevance": 0.81,
      "reason": "Current session discusses Google Workspace publishing"
    }
  ],
  "new_cursor": "kc_2026_04_26_00044"
}
```

### 15.7 Notification Model

Start with polling, not push.

Agent behavior:

```text
1. orient at task start
2. receive memory_cursor
3. before major decisions, call memory changes_since cursor
4. before final answer/handoff, call changes_since cursor
5. if relevant changes exist, incorporate or mention them
```

Do not require real-time streaming in v1.

### 15.8 Relevance Scoring For New Memory

A new memory item is relevant to an active session if it matches:

```text
same project
same subproject
same repo/component
same task
same file path
same entity
same decision/rule/limitation
semantic similarity to current prompt
supersedes an item already loaded in context pack
contradicts an item already loaded in context pack
```

### 15.9 Why Not Full A2A First?

Google's Agent2Agent protocol is an open protocol for communication and interoperability between opaque agentic applications. It is valuable when agents are independent applications, often remote, potentially built with different frameworks, and need formal contracts, discovery, authentication, streaming, and long-running task collaboration.

References:

```text
A2A project: https://github.com/a2aproject/A2A
A2A specification: https://a2a-protocol.org/latest/specification/
Google ADK A2A guidance: https://adk.dev/a2a/intro/
```

That is not the first problem for Engram.

For Engram v1, the first problem is:

```text
How do agents know which memory was written by which harness/model,
and whether memory changed after they oriented?
```

### 15.10 When A2A Becomes Worth It

Add A2A or A2A-inspired support when:

```text
agents run as independent long-lived services
agents are maintained by different teams
agents run across machines
agents use different frameworks/languages
formal capability discovery is needed
delegation crosses trust boundaries
network authentication/authorization matters
```

Do not use A2A for:

```text
two local Claude/Codex sessions sharing one Engram daemon
simple helper agents inside one harness
basic memory provenance
checking memory changes since orientation
```

### 15.11 Future Full A2A Bridge

If needed later, Engram can expose an A2A-compatible memory agent:

```text
Engram Memory Agent
  capabilities:
    orient
    retrieve_context
    get_memory_changes_since
    query_graph
    create_knowledge_commit
```

This lets external agents use Engram through A2A while local agents continue using MCP.

Recommended order:

```text
1. Add writer provenance.
2. Add memory_cursor and changes_since.
3. Add relevance scoring for newly written memory.
4. Add A2A-compatible facade only if independent remote agents need it.
```

---

## 16. Lint And Recalibration

### 16.1 Purpose

Lint prevents memory rot.

It should find:

- contradictions,
- stale facts,
- missing evidence,
- orphan nodes,
- duplicate entities,
- over-broad preferences,
- unused old preferences,
- unresolved questions,
- project hierarchy mistakes,
- docs that drift from code,
- old active sessions.

### 16.2 Lint Finding Schema

```rust
pub struct LintFinding {
    pub id: Id,
    pub severity: Severity,
    pub finding_type: LintFindingType,
    pub scope_id: Option<Id>,
    pub subject_id: Option<Id>,
    pub title: String,
    pub detail: String,
    pub evidence_refs: Vec<Id>,
    pub suggested_fix: Option<String>,
    pub auto_fixable: bool,
    pub status: LintStatus,
    pub created_at: OffsetDateTime,
}
```

### 16.3 Lint Rules MVP

```text
missing_evidence
stale_preference
contradictory_fact
superseded_loaded_as_active
orphan_entity
duplicate_entity_candidate
weak_related_to_overuse
subproject_without_parent
project_without_context_pack
task_without_entity_links
active_session_stale
handoff_missing_next_actions
vault_page_missing_frontmatter
generated_section_modified
```

### 16.4 Recalibration

Some memories should be periodically reviewed.

Example:

```text
Preference:
  "User allows browser login when OAuth is needed"

Review:
  after 90 days
  or after 5 relevant tasks where it was not used
```

Agent prompt:

```text
You previously allowed browser login for OAuth flows. I have not used that
preference recently. Should I keep it as a global preference, narrow it to
Google Workspace auth, or retire it?
```

### 16.5 Auto-Fix Policy

Safe auto-fixes:

- close stale active sessions,
- regenerate context packs,
- update `last_used_at`,
- add missing backlinks when evidence is unambiguous,
- move old closed projects to archive,
- merge exact duplicate aliases.

Human-reviewed fixes:

- factual contradictions,
- preference changes,
- decision supersession,
- project hierarchy changes,
- deletion of memories.

---

## 17. Archival Memory

### 17.1 Archiving Is Not Deleting

Archived memory remains searchable but not loaded by default.

Archive candidates:

```text
closed projects
superseded decisions
old sessions
inactive subprojects
outdated context packs
resolved limitations
stale preferences
```

### 17.2 Archive Metadata

```text
archived_at
archive_reason
archive_scope
retrieval_policy
summary
retained_facts
evidence_refs
```

### 17.3 Archive Page Example

```markdown
# Archived Project: Google Workspace MCP Import Experiment

Status: archived
Archived: 2026-04-26

## Retained Facts

- Remote Google Workspace MCP supports Docs writes.
- Claude Code stale OAuth cache can cause `401 invalid_token`.
- Recommended reset: clear `~/.mcp-auth`, remove old config, re-add server.

## Why Archived

The immediate publishing task is blocked pending user re-authentication.

## Reopen If

- User authenticates the remote MCP.
- Codex exposes write-capable Google Docs tools.
```

---

## 18. MCP API Design

### 18.1 New Tools

Add tools:

```text
orient
repo
memory
memory_commit
vault
graph
lint
handoff
```

### 18.2 `orient`

Purpose:

Return context for the current user request.

Request:

```json
{
  "cwd": "/Users/yuval.meiri/projects/engram",
  "prompt": "write implementation plan",
  "agent": "codex",
  "include_recent_commits": true
}
```

Response:

```json
{
  "project": "Engram",
  "scope": "Engram / Memory OS",
  "context_pack": "...",
  "brain_loop": {
    "compiled_context": "...",
    "top_items": [],
    "degraded": false
  },
  "active_decisions": [],
  "active_rules": [],
  "preferences": [],
  "limitations": [],
  "review_needed": [],
  "memory_cursor": {
    "timestamp": "...",
    "commit_id": "..."
  },
  "ambiguities": []
}
```

### 18.3 `repo`

Actions:

```text
detect
register
list
get
connect_project
connect_component
local_checkouts
monorepo_components
```

Purpose:

Track Git remotes, local checkouts, worktrees, and monorepo components so project orientation is not confused by folder layout.

Request:

```json
{
  "action": "detect",
  "cwd": "/Users/yuval/src/dd-source"
}
```

Response:

```json
{
  "repo": "github.com/DataDog/dd-source",
  "repo_kind": "monorepo",
  "local_checkout": "/Users/yuval/src/dd-source",
  "branch": "feature/debug-ai",
  "head": "abc123",
  "components": [
    {
      "name": "debug-ai-service",
      "root_path": "path/to/debug-ai-service"
    }
  ]
}
```

### 18.4 `memory`

Actions:

```text
add
get
list
search
update
archive
supersede
link
unlink
recalibrate
changes_since
```

### 18.5 `memory_commit`

Actions:

```text
create
get
list
diff
log
rollback_compiled_state
```

Rollback should not erase raw evidence. It should create a new commit that reverts compiled memory.

### 18.6 `vault`

Actions:

```text
init
compile
sync
status
page
context_pack
validate
```

### 18.7 `graph`

Actions:

```text
neighbors
traverse
path
subgraph
connect
disconnect
stats
lint
export
```

Example:

```json
{
  "action": "traverse",
  "start": "Engram",
  "depth": 2,
  "relations": ["contains", "has_decision", "has_rule", "has_limitation"]
}
```

### 18.8 `lint`

Actions:

```text
run
list
get
resolve
ignore
stats
```

### 18.9 `handoff`

Actions:

```text
get
update
write
finalize
```

---

## 19. CLI Design

### 19.1 Commands

```bash
engram orient
engram orient --prompt "fix multi-session tests"

engram repo detect
engram repo register --remote git@github.com:DataDog/dd-source.git --kind monorepo
engram repo component add dd-source debug-ai-service --path path/to/debug-ai-service
engram repo connect-project "Debug with AI" dd-source

engram vault init
engram vault compile
engram vault status
engram vault page "Engram/Memory OS"

engram memory add preference ...
engram memory search "commit message style"
engram memory archive --project old-project

engram graph around "Google Workspace MCP" --depth 2
engram graph path "User Preference" "Git Procedure"
engram graph lint

engram memory changes-since <cursor> --project Engram
engram memory writer-stats --project Engram

engram lint run --project Engram
engram lint list --severity high
engram lint resolve <finding-id>

engram commit-memory --session current
engram memory-log --project Engram
engram memory-diff <commit-a> <commit-b>
```

### 19.2 Human Workflow

```bash
cd ~/projects/engram
engram orient
engram graph around Engram --depth 2
engram lint run --project Engram
engram vault compile
```

### 19.3 Agent Workflow

At start of meaningful task:

```text
call orient
read context pack
continue work
```

During work:

```text
record raw events
record candidate durable memories
update rolling handoff
```

At boundary:

```text
distill
create knowledge commit
compile vault
run lint
```

---

## 20. Implementation Plan

### Phase 0: Stabilize Existing Base

Goal:

Make current Engram reliable enough to build on.

Tasks:

1. Fix `cargo clippy --all-targets -- -D warnings`.
2. Fix multi-session daemon/proxy tests.
3. Update stale multi-session docs.
4. Add current MCP tool reference.
5. Add smoke test for active Engram MCP capability surface.

Exit criteria:

```text
cargo fmt --all --check passes
cargo clippy --all-targets -- -D warnings passes
cargo test --all-targets --no-fail-fast passes except intentional ignored embedding tests
```

### Phase 1: Memory Ontology MVP

Goal:

Add typed memory objects without changing the whole product.

Tasks:

1. Add core types:
   - `MemoryItem`
   - `MemoryType`
   - `MemoryScope`
   - `MemoryStatus`
   - `EvidenceRef`
   - `Preference`
   - `Decision`
   - `Rule`
   - `Limitation`
   - `OpenQuestion`
2. Add `WriterProvenance` and `ClaimOrigin`.
3. Add store repos.
4. Add service methods.
5. Add MCP `memory` tool.
6. Add CLI `memory` commands.
7. Add tests.

Exit criteria:

The system can store and retrieve typed preferences, decisions, rules, limitations, and questions with scope, evidence, and writer provenance.

### Phase 2: Vault MVP

Goal:

Create Obsidian-readable Markdown pages from memory.

Tasks:

1. Add vault config.
2. Add vault initializer.
3. Add page templates.
4. Add generated section markers.
5. Add project overview page compiler.
6. Add preference/rule/decision pages.
7. Add context pack compiler.
8. Add vault status command.

Exit criteria:

Running:

```bash
engram vault init
engram vault compile --project Engram
```

creates a readable Markdown vault with project pages and context pack.

### Phase 3: Orientation

Goal:

Remove manual resume as the main user ritual.

Tasks:

1. Add `OrientationService`.
2. Implement project identity detection.
3. Implement repository detection from Git root, remote, branch, worktree, and HEAD.
4. Add repository topology records for Git repositories, local checkouts, and monorepo components.
5. Connect projects/subprojects/tasks to repositories and monorepo components.
6. Implement ambiguity response.
7. Implement context pack retrieval.
8. Add MCP `orient` and `repo`.
9. Add CLI `orient` and `repo`.
10. Add agent instructions for automatic orientation.

Exit criteria:

Given cwd and prompt, Engram returns project, scope, relevant memory, relevant Git repo/local checkout/component, and context pack path.

### Phase 4: Knowledge Commits

Goal:

Track memory evolution like Git.

Tasks:

1. Add `KnowledgeCommit` domain type.
2. Add commit changes table.
3. Add memory diff builder.
4. Add vault commit page writer.
5. Add memory cursor generation.
6. Add `changes_since` query for session-time memory awareness.
7. Add CLI `memory-log`, `memory-diff`, and `memory changes-since`.
8. Add MCP `memory_commit` and `memory(action=changes_since)`.

Exit criteria:

After a session distillation, Engram can show what knowledge changed since the previous commit. After orientation, an agent can query what relevant memory changed since its initial context pack.

### Phase 5: Live Session Distillation

Goal:

Follow the session and promote useful events.

Tasks:

1. Add raw event stream.
2. Add event capture MCP endpoint.
3. Add rolling handoff file.
4. Add candidate memory extractor.
5. Add human confirmation mode for high-impact memories.
6. Add periodic distillation.

Exit criteria:

Engram can produce a handoff and candidate knowledge commit from session events without relying on manual logging every few prompts.

### Phase 6: Graph Upgrade

Goal:

Make the graph behaviorally useful.

Tasks:

1. Add general `memory_edge`.
2. Link projects, subprojects, tasks, entities, decisions, rules, and evidence.
3. Add graph traversal service.
4. Add MCP `graph`.
5. Add CLI `graph`.
6. Add Obsidian graph-friendly backlinks.
7. Add graph export.

Exit criteria:

Engram can answer:

```text
What decisions apply to this subproject?
What preferences apply to this task?
What limitations are connected to this tool?
What changed because of this knowledge commit?
```

### Phase 7: Multi-Writer Awareness

Goal:

Let sessions notice relevant memory written by other harnesses/models after their initial orientation.

Tasks:

1. Add writer/harness/model filters to memory queries.
2. Add `changes_since` relevance scoring.
3. Add checks for memory that supersedes or contradicts context loaded at orientation.
4. Add agent guidance to poll changes before major decisions and final responses.
5. Add CLI `memory writer-stats`.
6. Add tests for cross-harness memory writes and changes-since behavior.

Exit criteria:

If Claude Code writes a relevant memory item while Codex is in-session, Codex can discover it with `changes_since` and decide whether to incorporate it.

### Phase 8: Lint And Recalibration

Goal:

Keep memory healthy.

Tasks:

1. Add lint finding types.
2. Add lint runner.
3. Add core lint rules.
4. Add recalibration scheduling.
5. Add safe auto-fixes.
6. Add MCP/CLI lint APIs.
7. Add vault lint report page.

Exit criteria:

Engram can report stale preferences, missing evidence, duplicate entities, orphan subprojects, contradictions, and stale sessions.

### Phase 9: Archival Memory

Goal:

Keep old memory accessible but out of the default context.

Tasks:

1. Add archive status and archive reason.
2. Add archive policies.
3. Add archive pages.
4. Add retrieval rules for archived memory.
5. Add archive search.

Exit criteria:

Closed projects and superseded decisions move out of default context while remaining discoverable.

### Phase 10: Retrieval Reranking

Goal:

Improve context quality.

Tasks:

1. Implement deterministic scoring.
2. Add scope-aware ranking.
3. Add graph distance scoring.
4. Add freshness/supersession filtering.
5. Add optional LLM rerank hook.
6. Add retrieval evaluation tests.

Exit criteria:

Orientation returns fewer irrelevant memories and avoids stale/superseded items by default.

---

## 21. Example End-To-End Flow

### 21.1 User Prompt

```text
Let's design the Engram graph model for the memory OS.
```

### 21.2 Agent Calls Orient

```json
{
  "cwd": "/Users/yuval.meiri/projects/engram",
  "prompt": "Let's design the Engram graph model for the memory OS.",
  "agent": "codex"
}
```

### 21.3 Engram Returns Context

```json
{
  "project": "Engram",
  "scope": "Memory OS",
  "active_decisions": [
    "Extend Engram rather than rebuild",
    "Markdown vault is durable memory",
    "Graph should drive orientation and lint"
  ],
  "preferences": [
    "User wants comprehensive implementation plans",
    "User dislikes having to remind agent to use Engram"
  ],
  "limitations": [
    "Current graph is single-hop and not browsable"
  ],
  "recent_knowledge_commits": [
    "2026-04-26-memory-os-direction"
  ],
  "context_pack_path": "~/.engram/vault/10_Projects/Engram/Context-Pack.md"
}
```

### 21.4 Work Happens

Raw events:

```text
user_prompt
file_read docs/MEMORY_OS_IMPLEMENTATION_PLAN.md
decision_candidate "Graph should use typed nodes plus memory_edge"
preference_candidate "User wants ambiguity called out explicitly"
```

### 21.5 Distillation

Generated memory:

```text
Decision:
  Use typed domain tables plus general memory_edge table.

Preference:
  User wants uncertainty/ambiguity surfaced before implementation.

OpenQuestion:
  Should vault default to ~/.engram/vault or user's existing notes folder?
```

### 21.6 Knowledge Commit

```text
Knowledge commit:
  Added graph schema decision.
  Added vault path open question.
  Updated Memory OS implementation page.
```

### 21.7 Vault Update

Files updated:

```text
10_Projects/Engram/Features/Memory-OS.md
10_Projects/Engram/Decisions.md
40_Preferences/Communication.md
95_Knowledge_Commits/2026/2026-04-26-graph-schema.md
```

---

## 22. Migration Strategy

### 22.1 Existing Data

Existing Engram data should be migrated into the new model.

Mapping:

```text
Entity -> Entity node
Relationship -> memory_edge or entity relationship
Session -> Session + Evidence
Event -> Evidence event
Observation -> MemoryItem candidate
Project -> Project node
Task -> Task node
PR -> ExternalReference / WorkArtifact
KnowledgeDoc -> SourceDocument
ToolUsage -> Episodic event + Tool memory
Git remote/local path references -> GitRepository + LocalCheckout
Monorepo path references -> MonorepoComponent candidates
```

### 22.2 Observation Migration

Existing observations are mixed-type. Migrate conservatively.

Example:

```text
key starts with decisions.*
  -> Decision candidate

key starts with patterns.*
  -> Rule or Procedure candidate

key starts with gotchas.*
  -> Limitation candidate

key starts with status.*
  -> Project status memory
```

Mark migrated items:

```text
status: active
confidence: medium
source: migrated_observation
needs_review: true
```

### 22.3 Repository Migration

Existing Engram memory may mention projects by repo name, local folder, product name, branch, or monorepo path. Migration must normalize these before promoting memories.

Migration steps:

```text
1. Scan all project paths, session cwd values, file refs, PR URLs, and observations.
2. Extract candidate Git remotes and local checkout paths.
3. Normalize remote URLs.
4. Cluster checkouts that point to the same remote.
5. Detect monorepos by size, known names, repeated path prefixes, or user-provided config.
6. Create GitRepository and LocalCheckout records.
7. Create MonorepoComponent candidates from recurring path prefixes.
8. Ask user to review ambiguous repo/project mappings.
9. Create a repository migration knowledge commit.
```

Example review batch:

```text
Project: Debug with AI

Candidate repos:
  - co-gen-backend: remote unknown, local path /Users/yuval/...
  - webui: remote unknown, local path /Users/yuval/...
  - dd-source: github.com/DataDog/dd-source, monorepo

Candidate dd-source components:
  - debug-ai-service
  - shared-llm-platform

Questions:
  1. Which local paths are canonical?
  2. Which dd-source components belong to this project?
  3. Should old mentions of "Debug with AI backend" map to co-gen-backend?
```

### 22.4 Digest Source Migration

Scheduled daily digests under `~/notes` are high-value but sensitive migration
sources. They should not be bulk-imported directly into active memory.

Digest migration steps:

```text
1. Inventory digest-like files by path and metadata only.
2. Export a generated digest source review batch.
3. Review each source as accept, source_only, quarantine, or reject.
4. Read only accepted sources into generated candidate-memory review pages.
5. Review each candidate memory as accept, quarantine, or reject.
6. Apply accepted candidates into Memory OS with writer provenance and source evidence.
7. Create a knowledge commit for written digest-derived memories.
8. Skip duplicates using stable digest extraction candidate tags.
```

Current command shape:

```bash
engram digest review-export ~/notes /tmp/engram-digest-review
engram digest source-index /tmp/engram-digest-review
engram digest source-index /tmp/engram-digest-review --write
engram digest extraction-plan /tmp/engram-digest-review /tmp/engram-digest-extraction
engram memory digest-extraction-apply /tmp/engram-digest-extraction
engram memory digest-extraction-apply /tmp/engram-digest-extraction --write
```

Safety rules:

```text
- Inventory and source-review export do not read digest contents.
- Source indexing reads only `source_only` decisions and defaults to dry-run.
- Extraction reads only `accept` decisions.
- quarantine, reject, and pending source decisions are not read.
- Extraction creates review pages only; it does not write active memory.
- Source indexing writes document evidence only; it does not write active memory.
- Candidate memory apply defaults to dry-run.
- Accepted candidate pages must include memory_kind, scope_type, scope_name when required, and source evidence.
- Applied memories are marked active because the candidate itself passed human review.
```

### 22.5 Vault Bootstrap

Bootstrap steps:

```bash
engram vault init
engram memory migrate-observations --project Engram
engram vault compile --project Engram
engram lint run --project Engram
```

---

## 23. Risks And Tradeoffs

### 23.1 Risk: Memory Becomes Too Intrusive

If the agent asks too many recalibration questions, users will disable it.

Mitigation:

- batch recalibration questions,
- ask only for high-impact memories,
- silently downgrade low-value memories,
- expose weekly memory review.

### 23.2 Risk: Generated Markdown Conflicts With Human Edits

Mitigation:

- generated section markers,
- patch-based updates,
- human-only sections,
- vault lint,
- backups before write.

### 23.3 Risk: Sensitive User Preferences Are Overused

Mitigation:

- sensitivity field,
- task-type filters,
- never quote sensitive emotional context unless needed,
- distill into operational guidance.

### 23.4 Risk: Graph Complexity Bloats Implementation

Mitigation:

- start with typed nodes and generic edges,
- implement single useful traversal first,
- use Obsidian for visualization before building UI.

### 23.5 Risk: LLM Distillation Hallucinates Memory

Mitigation:

- require evidence refs,
- mark confidence,
- lint missing provenance,
- human confirmation for high-impact changes.

### 23.6 Risk: Extending Engram Is Slower Than A New Prototype

Mitigation:

- prototype Memory OS behind feature flags,
- write vault compiler first,
- do not refactor old tools until new flow proves value.

### 23.7 Risk: Repository Topology Is Wrong

If Engram maps a local checkout or monorepo component to the wrong project, agents will load incorrect context and write memory to the wrong place.

Mitigation:

- normalize Git remotes,
- track local checkouts separately from canonical repositories,
- quarantine ambiguous repo mappings,
- require user review for monorepo component assignment,
- include repo/component identity in every orientation response.

### 23.8 Risk: Multi-Writer Awareness Becomes Accidental Collaboration

There is a temptation to turn writer provenance into a full coordination or delegation system. That would add complexity before the memory system needs it.

Mitigation:

- keep v1 focused on writer provenance,
- add memory cursor and changes-since polling,
- avoid work claims, agent chat, and A2A until explicitly needed,
- make source attribution visible in retrieval results.

---

## 24. Ambiguities And Questions

These need user/product decisions.

1. Vault location:
   - default `~/.engram/vault`,
   - existing `~/notes`,
   - per-project vaults,
   - or configurable all of the above?

2. Generated page ownership:
   - Can Engram rewrite whole pages?
   - Or only generated marked sections?

3. Agent autonomy:
   - Should agents always call `orient` automatically?
   - Should this be enforced by AGENTS.md/CLAUDE.md instructions?

4. Privacy:
   - Which user preferences are allowed in global memory?
   - Should sensitive memories require explicit confirmation?

5. Distillation:
   - Should distillation require an LLM call?
   - Should Engram support local-only deterministic extraction as fallback?

6. Project hierarchy:
   - Should users manually define parent/subproject relationships?
   - Or should Engram infer and ask only when ambiguous?

7. Archival:
   - When should a project be archived?
   - Who decides: user, inactivity rule, or agent proposal?

8. Git integration:
   - Should knowledge commits be real Git commits in the vault repo?
   - Or separate Engram DB records plus Markdown files?

9. Multi-writer behavior:
   - How often should agents poll for memory changes since orientation?
   - Should changes written by other harnesses be shown by default or only if highly relevant?
   - What level of writer/model detail should be visible in normal context packs?

10. UI:
    - Is Obsidian enough for v1?
    - When is a local graph dashboard worth building?

11. Repository topology:
    - Should users manually register multi-repo projects?
    - How should Engram discover monorepo components?
    - What is the canonical local checkout when multiple worktrees exist?

12. Future coordination:
    - Is coordination beyond memory provenance needed at all?
    - At what point would work claims, agent messages, or an A2A-compatible facade become worth building?

---

## 25. Recommended MVP

The highest-value MVP is not the entire system.

Build this first:

```text
1. Vault init/compile
2. Typed preferences, decisions, rules, limitations
3. Context pack generation
4. orient tool
5. repo detection and project/repo/component mapping
6. knowledge commit page
7. writer provenance and memory changes-since cursor
8. lint for missing evidence, stale preferences, duplicate entities
```

MVP user experience:

```text
User prompts Codex/Claude in a repo.
Agent calls orient.
Engram returns project context pack.
Agent works.
Engram distills key memories at end or checkpoint.
Vault updates readable Markdown.
Lint flags memory issues.
```

This directly addresses the user's lived pain:

- no manual reminder to use Engram,
- project/subproject context is clearer,
- preferences are retained,
- memory is readable in Obsidian,
- old knowledge can be archived,
- graph starts to affect retrieval.

---

## 26. Final Recommendation

Do not build a new memory system from scratch yet.

Extend Engram into Memory OS through a new subsystem:

```text
Engram Core remains:
  storage, MCP, CLI, sessions, entities, work, documents

Memory OS adds:
  orientation, ontology, vault, graph traversal, knowledge commits, lint,
  archival memory, repository topology, writer provenance, session-time memory
  change awareness, and recalibration
```

The core product should become:

```text
Engram is Git for AI-agent knowledge:
it orients agents automatically, records how understanding changes,
maintains an Obsidian-readable memory library, and lints memory for drift.
```

This direction uses the best ideas from:

- LLM Wiki: immutable sources, compiled Markdown, lint,
- Letta/MemGPT: core versus archival memory,
- Mem0: scoped memory and reranking,
- Graphiti: temporal facts and supersession,
- Cognee: ontology and entity hygiene,
- LangMem: semantic, episodic, and procedural memory classes.

The decisive improvement is not better search. It is better memory lifecycle.

The second decisive improvement is better identity: Engram must know which project, subproject, repository, local checkout, monorepo component, agent, harness, and model produced or needs a piece of knowledge.
