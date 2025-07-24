//! Example demonstrating Type ID usage in CKB smart contracts
//!
//! This example shows how to use the type_id module from ckb_deterministic
//! to implement and validate Type ID cells.

use ckb_deterministic::type_id::{
    calculate_type_id, check_type_id_from_script_args, load_type_id_from_script_args,
    validate_type_id,
};
use ckb_deterministic::errors::Error;

/// Example of a smart contract that validates Type ID
fn validate_type_id_contract() -> Result<(), Error> {
    // Method 1: Simple one-line validation
    // This assumes Type ID is at the beginning of script args
    check_type_id_from_script_args()?;
    
    Ok(())
}

/// Example of manually loading and validating Type ID
fn manual_type_id_validation() -> Result<(), Error> {
    // Load Type ID from script args at offset 0
    let type_id = load_type_id_from_script_args(0)?;
    
    // Validate the Type ID against transaction rules
    validate_type_id(type_id)?;
    
    // Continue with contract logic...
    println!("Type ID validated successfully: {:?}", type_id);
    
    Ok(())
}

/// Example of calculating Type ID for testing
fn calculate_expected_type_id() {
    // In tests, you can calculate the expected Type ID
    let first_input = b"serialized_first_input_cell";
    let output_index = 0;
    
    let type_id = calculate_type_id(first_input, output_index);
    println!("Calculated Type ID: {:?}", type_id);
    
    // Different inputs produce different Type IDs
    let type_id_2 = calculate_type_id(b"different_input", output_index);
    assert_ne!(type_id, type_id_2);
    
    // Different output indices produce different Type IDs
    let type_id_3 = calculate_type_id(first_input, 1);
    assert_ne!(type_id, type_id_3);
}

/// Example of a Type ID cell creation scenario
fn type_id_cell_creation_example() {
    // When creating a new Type ID cell:
    // 1. The Type ID is calculated from the first input and output index
    // 2. This Type ID must be included in the type script args
    // 3. The validation will ensure uniqueness
    
    let first_input_data = b"tx_first_input";
    let type_id_output_index = 2; // The output index where Type ID cell will be
    
    // Calculate what the Type ID should be
    let expected_type_id = calculate_type_id(first_input_data, type_id_output_index);
    
    println!("Type ID for new cell: {:?}", expected_type_id);
    println!("This Type ID must be included in the type script args of output[{}]", type_id_output_index);
}

fn main() {
    println!("=== Type ID Usage Examples ===\n");
    
    println!("1. Calculating Type ID:");
    calculate_expected_type_id();
    
    println!("\n2. Type ID Cell Creation:");
    type_id_cell_creation_example();
    
    println!("\n3. Contract validation would use:");
    println!("   - check_type_id_from_script_args() for simple validation");
    println!("   - Or manual load_type_id_from_script_args() + validate_type_id()");
    
    println!("\n=== Type ID Rules ===");
    println!("- At most one input and one output cell with the same Type ID");
    println!("- New Type ID = blake2b(first_input | output_index)");
    println!("- Ensures global uniqueness for cell types");
}