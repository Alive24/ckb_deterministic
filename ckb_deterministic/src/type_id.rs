//! Type ID implementation for CKB smart contracts
//! 
//! This module provides functionality for working with Type ID, a mechanism in CKB
//! that allows creating unique cell types with guaranteed uniqueness.
//!
//! # Type ID Overview
//! 
//! Type ID is a pattern in CKB that ensures a cell type can only be created once
//! by binding it to a specific transaction input and output position. This is useful
//! for creating singleton cells or unique tokens.
//!
//! The Type ID is calculated as:
//! ```text
//! blake2b(input_cell | output_index)
//! ```
//!
//! # Usage
//!
//! ```rust,no_run
//! # use ckb_deterministic::errors::Error;
//! # fn example() -> Result<(), Error> {
//! use ckb_deterministic::type_id::{validate_type_id, load_type_id_from_script_args};
//!
//! // In a smart contract:
//! let type_id = load_type_id_from_script_args(0)?;
//! validate_type_id(type_id)?;
//! # Ok(())
//! # }
//! ```
//!
//! # Rules
//!
//! 1. There can be at most one input and one output cell with the same Type ID
//! 2. When creating a new Type ID cell, the Type ID must match the calculated value
//! 3. When consuming a Type ID cell, validation ensures the Type ID rules are followed
//! 4. By default it uses the whole args as input, but you can specify a custom input by passing a custom input to the function

extern crate alloc;

use crate::errors::Error;
use crate::{debug_info, debug_error};
use alloc::vec::Vec;
use blake2b_ref::Blake2bBuilder;
use ckb_std::{
    ckb_constants::Source,
    error::SysError,
    high_level::{load_cell_type_hash, load_input, load_script, load_script_hash},
    syscalls::load_cell,
};
use molecule::prelude::Entity;

/// Checks if a cell exists at the given index and source
///
/// This function is used internally to verify cell presence without
/// loading the entire cell data.
fn has_type_id_cell(index: usize, source: Source) -> bool {
    let mut buf = Vec::new();
    match load_cell(&mut buf, 0, index, source) {
        Ok(_) => true,
        Err(e) => {
            // LengthNotEnough is expected when buffer is too small
            // but still indicates the cell exists
            match e {
                SysError::LengthNotEnough(_) => return true,
                _ => {
                    debug_info!("load cell err: {:?}", e);
                    false
                }
            }
        }
    }
}

/// Finds the index of the first output cell that has the current script as its type script
///
/// This is used when creating a new Type ID cell to determine the output index
/// needed for Type ID calculation.
fn locate_first_type_id_output_index() -> Result<usize, Error> {
    let current_script_hash = load_script_hash()?;

    let mut i = 0;
    loop {
        let type_hash = load_cell_type_hash(i, Source::Output)?;

        if type_hash == Some(current_script_hash) {
            break;
        }
        i += 1
    }
    Ok(i)
}

/// Calculates a Type ID from a cell input and output index
///
/// This function implements the Type ID calculation algorithm:
/// `blake2b(input_cell | output_index)` with CKB's default personalization.
///
/// # Arguments
///
/// * `input` - The serialized first input cell of the transaction
/// * `output_index` - The index of the output cell in the transaction
///
/// # Returns
///
/// A 32-byte Type ID
///
/// # Example
///
/// ```no_run
/// use ckb_deterministic::type_id::calculate_type_id;
/// 
/// let input_data = vec![/* serialized input cell */];
/// let output_index = 0;
/// let type_id = calculate_type_id(&input_data, output_index);
/// ```
pub fn calculate_type_id(input: &[u8], output_index: usize) -> [u8; 32] {
    let mut blake2b = Blake2bBuilder::new(32)
        .personal(b"ckb-default-hash")
        .build();
    blake2b.update(input);
    blake2b.update(&(output_index as u64).to_le_bytes());
    let mut ret = [0; 32];
    blake2b.finalize(&mut ret);
    ret
}

/// Validates that the current transaction conforms to Type ID rules
///
/// This function checks:
/// 1. There is at most one input and one output cell with this Type ID
/// 2. If creating a new Type ID cell, the Type ID matches the calculated value
///
/// # Arguments
///
/// * `type_id` - The 32-byte Type ID to validate
///
/// # Errors
///
/// Returns an error if:
/// - Multiple Type ID cells are found in inputs or outputs
/// - The Type ID doesn't match the calculated value when creating a new cell
///
/// # Example
///
/// ```rust,no_run
/// # use ckb_deterministic::errors::Error;
/// # fn example() -> Result<(), Error> {
/// use ckb_deterministic::type_id::validate_type_id;
/// 
/// let type_id = [0u8; 32]; // Your Type ID
/// validate_type_id(type_id)?;
/// # Ok(())
/// # }
/// ```
pub fn validate_type_id(type_id: [u8; 32]) -> Result<(), Error> {
    // Check for multiple Type ID cells
    if has_type_id_cell(1, Source::GroupInput) || has_type_id_cell(1, Source::GroupOutput) {
        debug_error!("There can only be at most one input and at most one output type ID cell!");
        return Err(Error::TypeIDMultipleCells);
    }

    // If no input Type ID cell exists, we're creating a new one
    if !has_type_id_cell(0, Source::GroupInput) {
        // Additional checks needed to ensure the Type ID is legitimate
        let index = locate_first_type_id_output_index()?;

        // The Type ID is calculated as the blake2b (with CKB's personalization) of
        // the first CellInput in current transaction, and the created output cell
        // index (in 64-bit little endian unsigned integer).
        let input = load_input(0, Source::Input)?;
        let calculated_type_id = calculate_type_id(input.as_slice(), index);

        if calculated_type_id != type_id {
            debug_error!("Invalid type ID!");
            debug_error!("Calculated type ID: {:x?}", calculated_type_id);
            debug_error!("Expected type ID: {:x?}", type_id);
            return Err(Error::TypeIDMismatch);
        }
    }
    Ok(())
}

/// Loads a Type ID from the current script's arguments
///
/// Type ID is expected to be stored in the script args at the specified offset.
/// The Type ID must be at least 32 bytes long from the offset position.
///
/// # Arguments
///
/// * `offset` - The byte offset in script args where the Type ID starts
///
/// # Errors
///
/// Returns an error if the script args don't contain enough bytes for a Type ID
///
/// # Example
///
/// ```rust,no_run
/// # use ckb_deterministic::errors::Error;
/// # fn example() -> Result<(), Error> {
/// use ckb_deterministic::type_id::load_type_id_from_script_args;
/// 
/// // Load Type ID from the beginning of script args
/// let type_id = load_type_id_from_script_args(0)?;
/// # Ok(())
/// # }
/// ```
pub fn load_type_id_from_script_args(offset: usize) -> Result<[u8; 32], Error> {
    let script = load_script()?;
    let args = script.as_reader().args();
    if offset + 32 > args.raw_data().len() {
        debug_error!("Length of type id is incorrect!");
        return Err(Error::LengthNotEnough);
    }
    let mut ret = [0; 32];
    ret.copy_from_slice(&args.raw_data()[offset..offset + 32]);
    Ok(ret)
}

/// Convenience function that loads and validates a Type ID from script args
///
/// This combines `load_type_id_from_script_args` and `validate_type_id`
/// for the common case where the Type ID is at the beginning of script args.
///
/// # Errors
///
/// Returns an error if:
/// - Loading the Type ID fails
/// - Type ID validation fails
///
/// # Example
///
/// ```rust,no_run
/// # use ckb_deterministic::errors::Error;
/// # fn example() -> Result<(), Error> {
/// use ckb_deterministic::type_id::check_type_id_from_script_args;
/// 
/// // In a smart contract's main function:
/// check_type_id_from_script_args()?;
/// # Ok(())
/// # }
/// ```
pub fn check_type_id_from_script_args() -> Result<(), Error> {
    let type_id = load_type_id_from_script_args(0)?;
    validate_type_id(type_id)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_type_id() {
        // Test with known values
        let input = b"test_input";
        let output_index = 0;
        let type_id = calculate_type_id(input, output_index);
        
        // Type ID should be 32 bytes
        assert_eq!(type_id.len(), 32);
        
        // Different inputs should produce different Type IDs
        let type_id2 = calculate_type_id(b"different_input", output_index);
        assert_ne!(type_id, type_id2);
        
        // Different output indices should produce different Type IDs
        let type_id3 = calculate_type_id(input, 1);
        assert_ne!(type_id, type_id3);
    }
}