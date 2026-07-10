// ============================================================================
// chain.rs — Backup Chain tracking for Restore Point resolution
// ============================================================================
//
// A Backup Chain represents a sequence of Restore Points (Full → Inc → Inc).
// chain_id groups all restore points in the same chain.
// chain_position identifies each point's position (0=Full).

use crate::repository::error::RepositoryError;

/// Reference to a Restore Point within a chain
#[derive(Debug, Clone)]
pub struct ChainLink {
    pub point_id: String,
    pub chain_position: u32,             // 0 = Full, 1+ = Incremental
    pub parent_point_id: Option<String>, // NULL for Full
    pub block_count: u64,
}

/// A Backup Chain groups related Restore Points.
/// All points share the same chain_id.
#[derive(Debug, Clone)]
pub struct BackupChain {
    pub chain_id: String,
    pub links: Vec<ChainLink>,
}

impl BackupChain {
    pub fn new(chain_id: String) -> Self {
        BackupChain {
            chain_id,
            links: Vec::new(),
        }
    }

    /// Add a link to the chain. Links should be added in position order.
    pub fn add_link(&mut self, link: ChainLink) {
        self.links.push(link);
    }

    /// Returns the last (most recent) chain position.
    /// Returns 0 if the chain is empty (no Full has been created yet).
    pub fn latest_position(&self) -> u32 {
        self.links
            .iter()
            .map(|l| l.chain_position)
            .max()
            .unwrap_or(0)
    }

    /// Returns the first (Full) link in the chain.
    pub fn full_link(&self) -> Option<&ChainLink> {
        self.links.iter().find(|l| l.chain_position == 0)
    }

    /// Resolve which chain links are needed to restore the given position.
    /// Returns all positions from 0 (Full) up to and including the given position.
    pub fn resolve_to_position(&self, position: u32) -> Result<Vec<&ChainLink>, RepositoryError> {
        let mut result: Vec<&ChainLink> = self
            .links
            .iter()
            .filter(|l| l.chain_position <= position)
            .collect();

        result.sort_by_key(|l| l.chain_position);

        if result.is_empty() {
            return Err(RepositoryError::General {
                detail: format!(
                    "No chain links found for position {} in chain {}",
                    position, self.chain_id
                ),
            });
        }

        // Verify that position 0 (Full) exists
        if result[0].chain_position != 0 {
            return Err(RepositoryError::General {
                detail: format!(
                    "Chain {} is missing a Full backup (position 0)",
                    self.chain_id
                ),
            });
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_chain() -> BackupChain {
        let mut chain = BackupChain::new("chain-001".to_string());
        chain.add_link(ChainLink {
            point_id: "full-001".to_string(),
            chain_position: 0,
            parent_point_id: None,
            block_count: 1000,
        });
        chain.add_link(ChainLink {
            point_id: "inc-001".to_string(),
            chain_position: 1,
            parent_point_id: Some("full-001".to_string()),
            block_count: 50,
        });
        chain.add_link(ChainLink {
            point_id: "inc-002".to_string(),
            chain_position: 2,
            parent_point_id: Some("inc-001".to_string()),
            block_count: 30,
        });
        chain
    }

    #[test]
    fn test_chain_resolve_full() {
        let chain = make_chain();
        let resolved = chain.resolve_to_position(0).unwrap();
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].point_id, "full-001");
    }

    #[test]
    fn test_chain_resolve_incremental() {
        let chain = make_chain();
        let resolved = chain.resolve_to_position(2).unwrap();
        assert_eq!(resolved.len(), 3);
        assert_eq!(resolved[0].point_id, "full-001");
        assert_eq!(resolved[1].point_id, "inc-001");
        assert_eq!(resolved[2].point_id, "inc-002");
    }

    #[test]
    fn test_chain_latest_position() {
        let chain = make_chain();
        assert_eq!(chain.latest_position(), 2);

        let empty_chain = BackupChain::new("empty".to_string());
        assert_eq!(empty_chain.latest_position(), 0);
    }

    #[test]
    fn test_chain_missing_full_is_error() {
        let mut chain = BackupChain::new("no-full".to_string());
        chain.add_link(ChainLink {
            point_id: "inc-bad".to_string(),
            chain_position: 1,
            parent_point_id: None,
            block_count: 10,
        });
        let result = chain.resolve_to_position(1);
        assert!(result.is_err());
    }

    #[test]
    fn test_chain_empty_is_error() {
        let chain = BackupChain::new("empty".to_string());
        let result = chain.resolve_to_position(0);
        assert!(result.is_err());
    }
}
