//! # Deterministic CDP Shared Library
//! 
//! Shared types and utilities for the Deterministic CDP (Collateralized Debt Position) system.
//! This library provides common definitions used by both the CDP contract and its tests.
//! 
//! ## Key Components
//! 
//! - [`cell_classifier`] - Cell classification rules specific to CDP contracts
//! - [`cdp_types`] - CDP-specific type definitions and constants
//! - [`error`] - CDP-specific error types
//! 
//! ## Usage
//! 
//! This library is designed to be used both in on-chain contracts and off-chain tools.
//! It provides a consistent interface for working with CDP data structures.
//! 
//! ```no_run
//! use deterministic_cdp_shared::{CDP_OPEN_VAULT, get_cdp_classifier};
//! 
//! // Use CDP-specific constants and classifiers
//! let method_path = CDP_OPEN_VAULT;
//! let classifier = get_cdp_classifier();
//! ```

#![no_std]

pub mod cell_classifier;
pub mod cdp_types;
pub mod error;

pub use cell_classifier::*;
pub use cdp_types::*;
pub use error::*;