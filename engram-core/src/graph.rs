//! Memory graph traversal types.

use serde::{Deserialize, Serialize};

/// Node in the Memory OS graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MemoryNode {
    /// Stable graph node ID, e.g. `memory:<uuid>`.
    pub id: String,
    /// Node kind.
    pub kind: String,
    /// Human-readable label.
    pub label: String,
}

impl MemoryNode {
    /// Create a graph node.
    #[must_use]
    pub fn new(id: impl Into<String>, kind: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind: kind.into(),
            label: label.into(),
        }
    }
}

/// Edge in the Memory OS graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MemoryEdge {
    /// Source node ID.
    pub from: String,
    /// Target node ID.
    pub to: String,
    /// Relationship label.
    pub relation: String,
}

impl MemoryEdge {
    /// Create a graph edge.
    #[must_use]
    pub fn new(
        from: impl Into<String>,
        to: impl Into<String>,
        relation: impl Into<String>,
    ) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            relation: relation.into(),
        }
    }
}

/// Graph subgraph response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemorySubgraph {
    /// Included nodes.
    pub nodes: Vec<MemoryNode>,
    /// Included edges.
    pub edges: Vec<MemoryEdge>,
}

impl MemorySubgraph {
    /// Create a graph.
    #[must_use]
    pub fn new(nodes: Vec<MemoryNode>, edges: Vec<MemoryEdge>) -> Self {
        Self { nodes, edges }
    }
}

/// Path response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryGraphPath {
    /// Node IDs from start to end.
    pub nodes: Vec<String>,
    /// Edges along the path.
    pub edges: Vec<MemoryEdge>,
}
