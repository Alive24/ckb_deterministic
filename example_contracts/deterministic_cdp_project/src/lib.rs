//! # Deterministic CDP Project Library
//!
//! This library provides the public interface for the Deterministic CDP (Collateralized Debt Position)
//! smart contract system built on the Nervos CKB blockchain.
//!
//! ## Overview
//!
//! The CDP system allows users to:
//! - **Open Vaults**: Create new collateralized debt positions by depositing collateral
//! - **Close Vaults**: Close existing vaults by repaying debt and reclaiming collateral  
//! - **Adjust Vaults**: Modify existing vaults by adding/removing collateral or debt
//!
//! ## Architecture
//!
//! The system uses a deterministic validation framework that ensures transaction correctness
//! through structured validation rules and business logic enforcement.
//!
//! ### Key Components:
//! - [`recipes`] - Contains validation logic for each CDP operation
//! - Shared types and utilities from `deterministic_cdp_shared`
//! - Integration with the `ckb_deterministic` validation framework
//!
//! ## Usage
//!
//! This library is primarily used for:
//! 1. **Testing** - Provides access to internal validation logic without compiling the full binary
//! 2. **Integration** - Allows other contracts or tools to reuse CDP validation logic
//! 3. **Development** - Enables easier development and debugging of CDP functionality
//!
//! ## Example
//!
//! ```rust,no_run
//! use deterministic_cdp_project::recipes::open_vault;
//!
//! // Get validation rules for opening a vault
//! let rules = open_vault::get_rules();
//! // rules can then be used to validate transactions
//! ```

pub mod recipes;

// Re-export necessary items for tests and external usage
pub use deterministic_cdp_shared::*;
