//! Example: Using Jest-like assertions in CKB validators

extern crate alloc;
use alloc::{format, string::{String, ToString}};
use ckb_deterministic::{
    generated::TransactionRecipe,
    cell_classifier::ClassifiedCells,
    assertions::*,
    validate_all,
    validation_block,
};

/// Example validator showing various assertion patterns
pub fn example_complex_validator(
    recipe: &TransactionRecipe,
    input_cells: &ClassifiedCells,
    output_cells: &ClassifiedCells,
) -> Result<(), String> {
    // Example 1: Basic value assertions
    let some_value = 42u64;
    expect(some_value).to_be_greater_than(40)?;
    expect(some_value).to_be_in_range(40, 50)?;
    expect(some_value).to_equal(42)?;

    // Example 2: Transaction structure validation
    expect_transaction(recipe).to_have_method_path(b"Example.method")?;
    expect_transaction(recipe).to_have_arguments_count(3)?;
    expect_transaction(recipe).to_have_argument_with_length(0, 8)?;  // u64
    expect_transaction(recipe).to_have_argument_with_length(1, 16)?; // u128
    expect_transaction(recipe).to_have_argument_with_length(2, 32)?; // hash

    // Example 3: Parse and validate arguments with helpers
    let args = expect_arguments(recipe)?;
    let id = expect_u64_argument(&args[0], "ID")?;
    let amount = expect_u128_argument(&args[1], "Amount")?;
    
    expect(id).to_be_greater_than(0)?;
    expect(amount).to_be_greater_than(1000)?;

    // Example 4: Cell validation patterns
    expect_cells(input_cells).to_have_known_cells("xudt")?;
    expect_cells(input_cells).to_have_known_cells_count("xudt", 2)?;
    expect_cells(output_cells).to_have_custom_cells(b"vault")?;
    expect_cells(output_cells).to_have_custom_cells_count(b"vault", 1)?;

    // Example 5: Using validation blocks for complex logic
    validation_block!("Token consistency check", {
        let input_tokens = input_cells.get_known("xudt")
            .ok_or("No input tokens found".to_string())?;
        let output_tokens = output_cells.get_known("xudt")
            .ok_or("No output tokens found".to_string())?;
        
        expect(input_tokens.len()).to_equal(output_tokens.len())?;
        Ok(())
    })?;

    // Example 6: Custom matchers
    let hash = &args[2];
    expect(hash).to_satisfy(
        |h| h.iter().any(|&b| b != 0),
        "Hash cannot be all zeros"
    )?;
    
    expect(hash).to_satisfy_with_error(|h| {
        if h.len() == 32 {
            Ok(())
        } else {
            Err(format!("Hash must be 32 bytes, got {}", h.len()))
        }
    })?;

    // Example 7: Boolean assertions
    let is_valid = amount > 1000 && id > 0;
    expect(is_valid).to_be_true()?;
    
    let is_empty = args.is_empty();
    expect(is_empty).to_be_false()?;

    // Example 8: Option assertions
    let maybe_cell = input_cells.get_known("xudt").and_then(|cells| cells.first());
    expect(maybe_cell).to_be_some()?;

    // Example 9: Combining multiple validations
    validate_all! {
        expect(id).to_be_greater_than(0),
        expect(amount).to_be_greater_than(1000),
        expect(hash).to_have_length(32),
        validation_block!("Business rule check", {
            if amount > 1_000_000 {
                expect(id).to_be_less_than(100)
                    .map_err(|_| "Large amounts require ID < 100".to_string())?;
            }
            Ok(())
        }),
    }?;

    // Example 10: Error context with assert functions
    assert(amount > 0, "Amount must be positive")?;
    assert_eq(hash.len(), 32, "Hash length validation")?;

    Ok(())
}

/// Example: Simple validator using only expect
pub fn example_simple_validator(
    recipe: &TransactionRecipe,
    input_cells: &ClassifiedCells,
    output_cells: &ClassifiedCells,
) -> Result<(), String> {
    // Validate transaction has correct method and arguments
    expect_transaction(recipe).to_have_method_path(b"Simple.transfer")?;
    expect_transaction(recipe).to_have_arguments_count(2)?;
    
    // Parse arguments
    let args = expect_arguments(recipe)?;
    let from_account = expect_u64_argument(&args[0], "From account")?;
    let to_account = expect_u64_argument(&args[1], "To account")?;
    
    // Validate arguments
    expect(from_account).not_to_equal(to_account)?;
    expect(from_account).to_be_greater_than(0)?;
    expect(to_account).to_be_greater_than(0)?;
    
    // Validate cells
    expect_cells(input_cells).to_have_known_cells_count("xudt", 1)?;
    expect_cells(output_cells).to_have_known_cells_count("xudt", 1)?;
    
    Ok(())
}

/// Example: Validator with detailed error messages
pub fn example_detailed_error_validator(
    recipe: &TransactionRecipe,
    _input_cells: &ClassifiedCells,
    _output_cells: &ClassifiedCells,
) -> Result<(), String> {
    validation_block!("Method validation", {
        expect_transaction(recipe).to_have_method_path(b"Detailed.process")?;
        Ok(())
    })?;
    
    validation_block!("Argument validation", {
        expect_transaction(recipe).to_have_arguments_count(1)?;
        let args = expect_arguments(recipe)?;
        
        validation_block!("Process ID validation", {
            let process_id = expect_u64_argument(&args[0], "Process ID")?;
            
            expect(process_id)
                .to_satisfy_with_error(|&id| {
                    if id == 0 {
                        Err("Process ID cannot be zero".to_string())
                    } else if id > 1000 {
                        Err(format!("Process ID {} exceeds maximum allowed value of 1000", id))
                    } else if id % 2 == 0 {
                        Err(format!("Process ID {} must be odd", id))
                    } else {
                        Ok(())
                    }
                })?;
            
            Ok(())
        })?;
        
        Ok(())
    })?;
    
    Ok(())
}