//! # CKB Deterministic Framework
//!
//! A comprehensive framework for building deterministic smart contracts on Nervos CKB.
//! This library provides essential building blocks for transaction validation, cell classification,
//! and recipe-based transaction processing.
//!
//! ## Key Features
//!
//! - **Transaction Recipes**: Structured transaction metadata using the SSRI standard
//! - **Cell Classification**: Flexible system for categorizing and managing different cell types
//! - **Validation Rules**: Declarative transaction validation with customizable constraints
//! - **Type ID Management**: Tools for working with CKB's Type ID pattern
//! - **Debug Utilities**: Enhanced debugging with context-aware logging
//!
//! ## Usage Example
//!
//! ```no_run
//! use ckb_deterministic::{
//!     cell_classifier::{RuleBasedClassifier, CellClass, CellCollector},
//!     transaction_context::TransactionContext,
//!     validation::TransactionValidationRules,
//! };
//!
//! // Create a classifier for your cell types
//! let classifier = RuleBasedClassifier::new("MyContract")
//!     .add_type_hash([10u8; 32], CellClass::known("xudt"));
//!
//! // Collect and classify cells
//! let collector = CellCollector::new(classifier);
//! let context = TransactionContext::new(collector)?;
//!
//! // Define validation rules
//! let rules = TransactionValidationRules::new(b"transfer".to_vec())
//!     .with_arguments(2);
//!
//! // Validate the transaction
//! rules.validate(&context)?;
//! ```

#![no_std]

#[cfg(test)]
#[macro_use]
extern crate std;

/// Jest-like assertion utilities for testing CKB contracts
pub mod assertions;
/// Cell classification system for identifying and organizing different cell types
pub mod cell_classifier;
/// Enhanced debugging utilities with context-aware logging
pub mod debug_utils;
/// Error types used throughout the framework
pub mod errors;
/// Auto-generated types from molecule schemas
pub mod generated;
/// Transaction context management for validation workflows
pub mod transaction_context;
/// Cell dependency management and resolution
pub mod transaction_deps;
/// Transaction recipe parsing and manipulation utilities
pub mod transaction_recipe;
/// Type ID calculation and validation utilities
pub mod type_id;
/// Common utility functions including hex decoding
pub mod utils;
/// Declarative transaction validation framework
pub mod validation;

// Re-export commonly used helper functions from transaction_recipe
pub use transaction_recipe::{
    create_cell_dep_data_reference, create_header_reference, create_inline_argument,
    create_input_data_reference, create_output_data_reference, create_recipe_flexible,
    create_recipe_with_args, create_recipe_with_reference, resolve_recipe_argument,
    serialize_transaction_recipe,
};

// Re-export commonly used utility functions
pub use utils::decode_hex;
