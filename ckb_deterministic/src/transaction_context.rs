//! Transaction context management for validation workflows.
//!
//! This module provides a unified context for transaction validation, combining
//! transaction recipes, classified cells, and dependency information into a single
//! structure that can be passed to validators.
//!
//! # Key Components
//!
//! - `TransactionContext`: Central context containing all transaction data
//! - `TransactionContextBuilder`: Fluent builder for context configuration
//! - `TransactionSummary`: Statistics and metadata about the transaction
//! - Integration with cell classification and recipe parsing
//!
//! # Example
//!
//! ```no_run
//! use ckb_deterministic::{
//!     transaction_context::TransactionContext,
//!     cell_classifier::{RuleBasedClassifier, CellClass, CellCollector},
//! };
//!
//! // Create classifier and collector
//! let classifier = RuleBasedClassifier::new("MyContract")
//!     .add_type_hash([10u8; 32], CellClass::known("xudt"));
//! let collector = CellCollector::new(classifier);
//!
//! // Create transaction context
//! let context = TransactionContext::new(collector)?;
//!
//! // Access classified cells
//! let input_xudt = context.input_cells.get_known("xudt");
//! let output_vaults = context.output_cells.get_custom("vault");
//! ```

use crate::cell_classifier::{CellClassifier, CellCollector, ClassifiedCells};
use crate::errors::Error;
use crate::generated::TransactionRecipe;
use crate::transaction_recipe::{self as recipe, TransactionRecipeExt};
use crate::{debug_error, debug_info, debug_trace};
extern crate alloc;
use alloc::vec::Vec;
use core::{
    option::Option::*,
    result::{Result, Result::*},
};

/// Complete transaction context for validation workflows.
///
/// Contains all necessary information about a transaction including:
/// - Parsed transaction recipe from witness
/// - Classified input/output cells
/// - Cell dependencies and header dependencies
///
/// The generic parameter `C` allows different cell classification strategies.
pub struct TransactionContext<C: CellClassifier> {
    pub recipe: TransactionRecipe,
    pub input_cells: ClassifiedCells,
    pub output_cells: ClassifiedCells,
    pub cell_deps: ClassifiedCells,
    pub header_deps: Vec<[u8; 32]>,
    _phantom: core::marker::PhantomData<C>,
}

impl<C: CellClassifier> TransactionContext<C> {
    /// Create a transaction context by parsing recipe and collecting cells.
    ///
    /// This automatically:
    /// 1. Parses the transaction recipe from witness
    /// 2. Collects and classifies all cells (inputs, outputs, deps)
    /// 3. Extracts header dependencies
    ///
    /// # Errors
    ///
    /// Returns `Error::RecipeError` if no recipe is found in witness.
    /// Returns `Error::UnidentifiedCells` if strict mode is enabled and unidentified cells are found.
    pub fn new(collector: CellCollector<C>) -> Result<Self, Error> {
        debug_trace!("Creating generic transaction context");

        // Parse transaction recipe from witness
        let recipe = match recipe::parse_transaction_recipe()? {
            Some(r) => r,
            None => {
                debug_info!("No transaction recipe found in witness");
                return Err(Error::RecipeError);
            }
        };

        debug_info!("Transaction recipe found");

        // Collect and classify cells
        let (input_cells, output_cells, cell_deps) = collector.collect_cells()?;

        debug_info!(
            "Input cells collected: {} total, {} unidentified",
            input_cells.total_cell_count(),
            input_cells.unidentified_cells.len()
        );
        debug_info!(
            "Output cells collected: {} total, {} unidentified",
            output_cells.total_cell_count(),
            output_cells.unidentified_cells.len()
        );
        debug_info!(
            "CellDep cells collected: {} total, {} unidentified",
            cell_deps.total_cell_count(),
            cell_deps.unidentified_cells.len()
        );

        let header_deps = match recipe.header_deps().to_opt() {
            Some(deps) => {
                let mut hashes = Vec::new();
                for i in 0..deps.len() {
                    match deps.get(i) {
                        Some(hash) => {
                            let raw_data = hash.raw_data();
                            let mut hash_bytes = [0u8; 32];
                            hash_bytes.copy_from_slice(&raw_data);
                            hashes.push(hash_bytes);
                        }
                        None => {}
                    }
                }
                hashes
            }
            None => Vec::new(),
        };

        debug_info!("Header deps count: {}", header_deps.len());

        Ok(Self {
            recipe,
            input_cells,
            output_cells,
            cell_deps,
            header_deps,
            _phantom: core::marker::PhantomData,
        })
    }

    /// Create a transaction context from pre-collected parts.
    ///
    /// Useful for testing or when cells have already been collected and classified.
    pub fn from_parts(
        recipe: TransactionRecipe,
        input_cells: ClassifiedCells,
        output_cells: ClassifiedCells,
        cell_deps: ClassifiedCells,
        header_deps: Vec<[u8; 32]>,
    ) -> Self {
        Self {
            recipe,
            input_cells,
            output_cells,
            cell_deps,
            header_deps,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Validate that the transaction context is internally consistent.
    ///
    /// Checks for:
    /// - Presence of unidentified cells (considered an error in strict mode)
    /// - Basic structural validity
    ///
    /// Note: This performs structural validation only. Business logic validation
    /// should be done using `TransactionValidationRules`.
    pub fn validate(&self) -> Result<(), Error> {
        // Check for unidentified cells (if collector was in strict mode, this should already be caught)
        if self.input_cells.has_unidentified_cells() || self.output_cells.has_unidentified_cells() {
            debug_error!("Transaction context validation failed: unidentified cells detected");
            return Err(Error::UnidentifiedCells);
        }

        debug_info!("Transaction context validation passed");
        Ok(())
    }

    /// Get summary statistics about the transaction.
    ///
    /// Useful for debugging and logging transaction details.
    pub fn summary(&self) -> TransactionSummary {
        TransactionSummary {
            argument_count: self.recipe.arguments_vec().len(),
            input_known_cells: self.input_cells.known_cells.len(),
            input_custom_cells: self.input_cells.custom_cells.len(),
            input_unidentified_cells: self.input_cells.unidentified_cells.len(),
            output_known_cells: self.output_cells.known_cells.len(),
            output_custom_cells: self.output_cells.custom_cells.len(),
            output_unidentified_cells: self.output_cells.unidentified_cells.len(),
            cell_deps_known_cells: self.cell_deps.known_cells.len(),
            cell_deps_custom_cells: self.cell_deps.custom_cells.len(),
            cell_deps_unidentified_cells: self.cell_deps.unidentified_cells.len(),
            header_deps_count: self.header_deps.len(),
        }
    }
}

/// Summary statistics for a transaction context.
///
/// Provides counts of different cell types and dependencies,
/// useful for debugging and monitoring transaction patterns.
#[derive(Debug)]
pub struct TransactionSummary {
    pub argument_count: usize,
    pub input_known_cells: usize,
    pub input_custom_cells: usize,
    pub input_unidentified_cells: usize,
    pub output_known_cells: usize,
    pub output_custom_cells: usize,
    pub output_unidentified_cells: usize,
    pub cell_deps_known_cells: usize,
    pub cell_deps_custom_cells: usize,
    pub cell_deps_unidentified_cells: usize,
    pub header_deps_count: usize,
}

/// Builder for creating transaction contexts with custom configuration.
///
/// Allows fluent configuration of context creation parameters.
pub struct TransactionContextBuilder<C: CellClassifier> {
    collector: CellCollector<C>,
}

impl<C: CellClassifier> TransactionContextBuilder<C> {
    /// Create a new builder with a specific collector
    pub fn new(collector: CellCollector<C>) -> Self {
        Self { collector }
    }

    /// Enable or disable strict mode for unidentified cells
    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.collector = self.collector.with_strict_mode(strict);
        self
    }

    /// Build the transaction context
    pub fn build(self) -> Result<TransactionContext<C>, Error> {
        TransactionContext::new(self.collector)
    }
}

/// Convenience function to create a transaction context.
///
/// Equivalent to `TransactionContext::new(collector)`.
pub fn create_transaction_context<C: CellClassifier>(
    collector: CellCollector<C>,
) -> Result<TransactionContext<C>, Error> {
    TransactionContext::new(collector)
}

/// Convenience function to create a transaction context builder.
///
/// Equivalent to `TransactionContextBuilder::new(collector)`.
pub fn transaction_context_builder<C: CellClassifier>(
    collector: CellCollector<C>,
) -> TransactionContextBuilder<C> {
    TransactionContextBuilder::new(collector)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell_classifier::{CellClass, RuleBasedClassifier};

    #[test]
    fn test_transaction_summary() {
        let summary = TransactionSummary {
            argument_count: 3,
            input_known_cells: 2,
            input_custom_cells: 1,
            input_unidentified_cells: 0,
            output_known_cells: 2,
            output_custom_cells: 1,
            output_unidentified_cells: 0,
            header_deps_count: 1,
            cell_deps_known_cells: 1,
            cell_deps_custom_cells: 2,
            cell_deps_unidentified_cells: 3,
        };

        // Test that summary contains expected values
        assert_eq!(summary.argument_count, 3);
        assert_eq!(summary.input_known_cells, 2);
        assert_eq!(summary.header_deps_count, 1);
    }

    #[test]
    fn test_transaction_context_builder() {
        let udt_type_hash = [10u8; 32];
        let amm_type_hash = [11u8; 32];

        let classifier = RuleBasedClassifier::new("TestClassifier")
            .add_type_hash(udt_type_hash, CellClass::known("udt"))
            .add_type_hash(amm_type_hash, CellClass::known("amm"));

        let collector = CellCollector::new(classifier);
        let _builder = TransactionContextBuilder::new(collector).with_strict_mode(true);

        // Builder should be configured correctly
        // Note: Actual build() would fail in test environment due to missing CKB syscalls
    }

    #[test]
    fn test_method_path_matching() {
        use crate::transaction_recipe::method_path;

        // Test method path patterns for different protocols
        let udt_transfer_name = "UDT.transfer";
        let amm_swap_name = "AMM.swapExactTokensForTokens";
        let dao_proposal_name = "DAO::submitProposal";

        // Test hash calculation
        let udt_transfer_hash = method_path(udt_transfer_name);
        let amm_swap_hash = method_path(amm_swap_name);
        let dao_proposal_hash = method_path(dao_proposal_name);

        // Verify method path names are reasonable
        assert!(udt_transfer_name.len() > 0);
        assert!(amm_swap_name.len() > udt_transfer_name.len());
        assert!(dao_proposal_name.len() > 0);

        // Test hash uniqueness
        assert_ne!(udt_transfer_hash, amm_swap_hash);
        assert_ne!(udt_transfer_hash, dao_proposal_hash);
        assert_ne!(amm_swap_hash, dao_proposal_hash);

        // Test matching logic
        assert_eq!(udt_transfer_name, "UDT.transfer");
        assert_ne!(udt_transfer_name, "UDT.mint");
    }

    #[test]
    fn test_transaction_context_type() {
        let test_type_hash = [20u8; 32];

        let classifier = RuleBasedClassifier::new("TestClassifier")
            .add_type_hash(test_type_hash, CellClass::known("test"));

        let collector = CellCollector::new(classifier);

        // Test that type alias works correctly
        let _builder: TransactionContextBuilder<RuleBasedClassifier> =
            transaction_context_builder(collector);
    }

    #[test]
    fn test_realistic_method_paths() {
        use crate::transaction_recipe::method_path;

        // Test various realistic method path patterns
        let method_paths = [
            "UDT.transfer",
            "UDT.mint",
            "UDT.burn",
            "UDT.is_paused",
            "UDT.enumerate_paused",
            "AMM.addLiquidity",
            "AMM.removeLiquidity",
            "AMM.swapExactTokensForTokens",
            "Vault.deposit",
            "Vault.withdraw",
            "Bridge::lockAndMint",
            "Bridge::burnAndUnlock",
            "DAO::submitProposal",
            "DAO::vote",
            "NFTMarketplace.createListing",
            "NFTMarketplace.buyNow",
            "LendingProtocol.deposit",
            "LendingProtocol.borrow",
            "LendingProtocol.repay",
            "CKBoostCampaign::createQuest",
            "CKBoostCampaign::completeQuest",
            "SSRI.version",
            "SSRI.get_methods",
        ];

        for path in &method_paths {
            assert!(path.len() > 0);
            assert!(path.len() < 100); // Reasonable upper bound

            // Test that hash calculation works
            let hash = method_path(path);
            assert_ne!(hash, 0); // Hash should not be zero for valid method names
        }

        // Test that different method names produce different hashes
        let hash1 = method_path("UDT.transfer");
        let hash2 = method_path("UDT.mint");
        assert_ne!(hash1, hash2);
    }
}
