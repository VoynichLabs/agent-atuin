use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

pub mod database;

/// A memory entry that links natural language descriptions to commands
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Memory {
    /// UUIDv7 identifier
    pub id: String,
    /// Natural language description of what was done
    pub description: String,
    /// Current working directory when memory was created
    pub cwd: String,
    /// Git repository root (if in a repo)
    pub repo_root: Option<String>,
    /// Git branch at creation time
    pub git_branch: Option<String>,
    /// Git commit hash at creation time
    pub git_commit: Option<String>,
    /// Which agent created this memory
    pub agent_id: Option<String>,
    /// Parent memory ID for hierarchical relationships
    pub parent_memory_id: Option<String>,
    /// When the memory was created
    pub created_at: OffsetDateTime,
}

/// Link between a memory and a history command
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryCommand {
    pub memory_id: String,
    pub history_id: String,
}

impl Memory {
    pub fn new(
        description: String,
        cwd: String,
        repo_root: Option<String>,
        git_branch: Option<String>,
        git_commit: Option<String>,
        agent_id: Option<String>,
        parent_memory_id: Option<String>,
    ) -> Self {
        Self {
            id: atuin_common::utils::uuid_v7().as_simple().to_string(),
            description,
            cwd,
            repo_root,
            git_branch,
            git_commit,
            agent_id,
            parent_memory_id,
            created_at: OffsetDateTime::now_utc(),
        }
    }
}

/// JSON output format for memory list
#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryJson {
    pub id: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_memory_id: Option<String>,
    pub created_at: String,
    pub commands_count: usize,
}

impl From<&Memory> for MemoryJson {
    fn from(m: &Memory) -> Self {
        Self {
            id: m.id.clone(),
            description: m.description.clone(),
            repo: m.repo_root.as_ref().and_then(|r| {
                std::path::Path::new(r)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(String::from)
            }),
            parent_memory_id: m.parent_memory_id.clone(),
            created_at: m
                .created_at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default(),
            commands_count: 0, // Will be filled in when querying
        }
    }
}

/// JSON output for memory creation result
#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryCreateJson {
    pub id: String,
    pub description: String,
    pub commands_linked: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_memory_id: Option<String>,
}

/// JSON output for memory tree visualization
#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryTreeNode {
    #[serde(flatten)]
    pub memory: MemoryJson,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<MemoryTreeNode>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Memory::new() Tests ====================

    #[test]
    fn test_memory_new_basic() {
        let m = Memory::new(
            "test description".into(),
            "/tmp".into(),
            None,
            None,
            None,
            None,
            None,
        );

        assert_eq!(m.description, "test description");
        assert_eq!(m.cwd, "/tmp");
        assert!(m.repo_root.is_none());
        assert!(m.git_branch.is_none());
        assert!(m.git_commit.is_none());
        assert!(m.agent_id.is_none());
        assert!(m.parent_memory_id.is_none());

        // UUID v7 as simple string should be 32 hex chars
        assert_eq!(m.id.len(), 32);
        assert!(m.id.chars().all(|c| c.is_ascii_hexdigit()));

        // created_at should be within 1 second of now
        let now = OffsetDateTime::now_utc();
        let diff = (now - m.created_at).whole_seconds().abs();
        assert!(
            diff <= 1,
            "created_at should be within 1s of now, diff={diff}s"
        );
    }

    #[test]
    fn test_memory_new_all_fields() {
        let m = Memory::new(
            "full memory".into(),
            "/home/user/project".into(),
            Some("/home/user/project".into()),
            Some("feature/test".into()),
            Some("abc123".into()),
            Some("claude-agent".into()),
            Some("parent-id-123".into()),
        );

        assert_eq!(m.repo_root.as_deref(), Some("/home/user/project"));
        assert_eq!(m.git_branch.as_deref(), Some("feature/test"));
        assert_eq!(m.git_commit.as_deref(), Some("abc123"));
        assert_eq!(m.agent_id.as_deref(), Some("claude-agent"));
        assert_eq!(m.parent_memory_id.as_deref(), Some("parent-id-123"));
    }

    #[test]
    fn test_memory_new_unique_ids() {
        let m1 = Memory::new("a".into(), "/tmp".into(), None, None, None, None, None);
        let m2 = Memory::new("b".into(), "/tmp".into(), None, None, None, None, None);
        assert_ne!(m1.id, m2.id);
    }

    // ==================== MemoryJson Conversion Tests ====================

    #[test]
    fn test_memory_json_from_basic() {
        let m = Memory::new(
            "basic memory".into(),
            "/tmp".into(),
            None,
            None,
            None,
            None,
            None,
        );
        let json = MemoryJson::from(&m);

        assert_eq!(json.id, m.id);
        assert_eq!(json.description, "basic memory");
        assert!(json.repo.is_none());
        assert!(json.parent_memory_id.is_none());
        assert_eq!(json.commands_count, 0);

        // created_at should be a valid RFC3339 string
        assert!(
            time::OffsetDateTime::parse(
                &json.created_at,
                &time::format_description::well_known::Rfc3339
            )
            .is_ok(),
            "created_at should be valid RFC3339: {}",
            json.created_at
        );
    }

    #[test]
    fn test_memory_json_repo_name_extraction() {
        let m = Memory::new(
            "repo test".into(),
            "/tmp".into(),
            Some("/home/user/projects/my-repo".into()),
            None,
            None,
            None,
            None,
        );
        let json = MemoryJson::from(&m);
        assert_eq!(json.repo.as_deref(), Some("my-repo"));
    }

    #[test]
    fn test_memory_json_repo_root_only() {
        let m = Memory::new(
            "root repo".into(),
            "/".into(),
            Some("/".into()),
            None,
            None,
            None,
            None,
        );
        let json = MemoryJson::from(&m);
        // Path::file_name returns None for "/"
        assert!(json.repo.is_none());
    }

    #[test]
    fn test_memory_json_parent_propagation() {
        let m = Memory::new(
            "child".into(),
            "/tmp".into(),
            None,
            None,
            None,
            None,
            Some("parent-abc".into()),
        );
        let json = MemoryJson::from(&m);
        assert_eq!(json.parent_memory_id.as_deref(), Some("parent-abc"));
    }

    // ==================== Serialization Tests ====================

    #[test]
    fn test_memory_json_skip_none_fields() {
        let json = MemoryJson {
            id: "test-id".into(),
            description: "test".into(),
            repo: None,
            parent_memory_id: None,
            created_at: "2024-01-01T00:00:00Z".into(),
            commands_count: 0,
        };
        let serialized = serde_json::to_string(&json).unwrap();
        assert!(
            !serialized.contains("\"repo\""),
            "repo:None should be omitted"
        );
        assert!(
            !serialized.contains("\"parent_memory_id\""),
            "parent_memory_id:None should be omitted"
        );
    }

    #[test]
    fn test_memory_json_includes_present_fields() {
        let json = MemoryJson {
            id: "test-id".into(),
            description: "test".into(),
            repo: Some("my-repo".into()),
            parent_memory_id: Some("parent-123".into()),
            created_at: "2024-01-01T00:00:00Z".into(),
            commands_count: 5,
        };
        let serialized = serde_json::to_string(&json).unwrap();
        assert!(serialized.contains("\"repo\""));
        assert!(serialized.contains("\"my-repo\""));
        assert!(serialized.contains("\"parent_memory_id\""));
        assert!(serialized.contains("\"parent-123\""));
    }

    #[test]
    fn test_memory_json_roundtrip() {
        let original = MemoryJson {
            id: "abc123".into(),
            description: "roundtrip test".into(),
            repo: Some("my-repo".into()),
            parent_memory_id: Some("parent-id".into()),
            created_at: "2024-06-15T12:30:00Z".into(),
            commands_count: 3,
        };
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: MemoryJson = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.id, original.id);
        assert_eq!(deserialized.description, original.description);
        assert_eq!(deserialized.repo, original.repo);
        assert_eq!(deserialized.parent_memory_id, original.parent_memory_id);
        assert_eq!(deserialized.created_at, original.created_at);
        assert_eq!(deserialized.commands_count, original.commands_count);
    }

    #[test]
    fn test_memory_create_json_serialization() {
        let json = MemoryCreateJson {
            id: "test-id".into(),
            description: "test".into(),
            commands_linked: 2,
            repo: None,
            branch: None,
            commit: None,
            parent_memory_id: None,
        };
        let serialized = serde_json::to_string(&json).unwrap();

        // Required fields present
        assert!(serialized.contains("\"id\""));
        assert!(serialized.contains("\"commands_linked\""));

        // Optional None fields should be skipped
        assert!(!serialized.contains("\"repo\""));
        assert!(!serialized.contains("\"branch\""));
        assert!(!serialized.contains("\"commit\""));
        assert!(!serialized.contains("\"parent_memory_id\""));
    }

    #[test]
    fn test_memory_create_json_roundtrip() {
        let original = MemoryCreateJson {
            id: "create-id".into(),
            description: "create test".into(),
            commands_linked: 5,
            repo: Some("repo-name".into()),
            branch: Some("main".into()),
            commit: Some("abc123".into()),
            parent_memory_id: Some("parent-id".into()),
        };
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: MemoryCreateJson = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.id, original.id);
        assert_eq!(deserialized.description, original.description);
        assert_eq!(deserialized.commands_linked, original.commands_linked);
        assert_eq!(deserialized.repo, original.repo);
        assert_eq!(deserialized.branch, original.branch);
        assert_eq!(deserialized.commit, original.commit);
        assert_eq!(deserialized.parent_memory_id, original.parent_memory_id);
    }

    // ==================== MemoryTreeNode Tests ====================

    #[test]
    fn test_tree_node_no_children() {
        let node = MemoryTreeNode {
            memory: MemoryJson {
                id: "node-1".into(),
                description: "leaf node".into(),
                repo: None,
                parent_memory_id: None,
                created_at: "2024-01-01T00:00:00Z".into(),
                commands_count: 1,
            },
            children: vec![],
        };
        let serialized = serde_json::to_string(&node).unwrap();

        // Empty children vec should be omitted
        assert!(!serialized.contains("\"children\""));

        // Flattened fields should be at top level (no "memory" key)
        assert!(!serialized.contains("\"memory\""));
        assert!(serialized.contains("\"id\""));
        assert!(serialized.contains("\"description\""));
    }

    #[test]
    fn test_tree_node_with_children() {
        let child = MemoryTreeNode {
            memory: MemoryJson {
                id: "child-1".into(),
                description: "child node".into(),
                repo: None,
                parent_memory_id: Some("root-1".into()),
                created_at: "2024-01-01T01:00:00Z".into(),
                commands_count: 0,
            },
            children: vec![],
        };
        let root = MemoryTreeNode {
            memory: MemoryJson {
                id: "root-1".into(),
                description: "root node".into(),
                repo: Some("my-repo".into()),
                parent_memory_id: None,
                created_at: "2024-01-01T00:00:00Z".into(),
                commands_count: 2,
            },
            children: vec![child],
        };

        let serialized = serde_json::to_string(&root).unwrap();
        let deserialized: MemoryTreeNode = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.memory.id, "root-1");
        assert_eq!(deserialized.children.len(), 1);
        assert_eq!(deserialized.children[0].memory.id, "child-1");
        assert!(deserialized.children[0].children.is_empty());
    }

    #[test]
    fn test_tree_node_flatten_behavior() {
        let node = MemoryTreeNode {
            memory: MemoryJson {
                id: "flat-test".into(),
                description: "flatten test".into(),
                repo: Some("repo".into()),
                parent_memory_id: None,
                created_at: "2024-01-01T00:00:00Z".into(),
                commands_count: 0,
            },
            children: vec![],
        };
        let value: serde_json::Value = serde_json::to_value(&node).unwrap();
        let obj = value.as_object().unwrap();

        // Fields from MemoryJson should be at top level due to #[serde(flatten)]
        assert!(obj.contains_key("id"));
        assert!(obj.contains_key("description"));
        assert!(obj.contains_key("repo"));
        assert!(obj.contains_key("created_at"));
        assert!(obj.contains_key("commands_count"));

        // There should be no nested "memory" key
        assert!(!obj.contains_key("memory"));
    }
}
