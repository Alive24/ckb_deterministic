# Jest-like Assertion Framework for CKB Validators

This module provides a Jest-inspired assertion API for writing expressive and readable validators in CKB smart contracts.

## Overview

The assertion framework provides:
- Fluent `expect()` API similar to Jest
- Descriptive error messages
- Validation blocks for organized error handling
- CKB-specific matchers for transactions and cells
- Helper functions for common validation patterns

## Basic Usage

```rust
use ckb_deterministic::assertions::*;

// In your validator function:
pub fn validate_transaction(
    recipe: &TransactionRecipe,
    input_cells: &ClassifiedCells,
    output_cells: &ClassifiedCells,
) -> Result<(), String> {
    // Basic value assertions
    let amount = 100u128;
    expect(amount).to_be_greater_than(0)?;
    expect(amount).to_be_in_range(50, 200)?;
    
    // Transaction validation
    expect_transaction(recipe).to_have_method_path(b"Transfer.send")?;
    expect_transaction(recipe).to_have_arguments_count(2)?;
    
    // Cell validation
    expect_cells(input_cells).to_have_known_cells_count("xudt", 1)?;
    expect_cells(output_cells).to_have_custom_cells(b"vault")?;
    
    Ok(())
}
```

## API Reference

### Core Functions

- `expect(value)` - Create an expectation on any value
- `assert(condition, message)` - Simple boolean assertion
- `assert_eq(actual, expected, context)` - Equality assertion with context

### Numeric Matchers

- `to_be_greater_than(expected)`
- `to_be_greater_than_or_equal(expected)`
- `to_be_less_than(expected)`
- `to_be_less_than_or_equal(expected)`
- `to_be_in_range(min, max)`

### Equality Matchers

- `to_equal(expected)`
- `not_to_equal(expected)`

### Boolean Matchers

- `to_be_true()`
- `to_be_false()`

### Collection Matchers

- `to_have_length(expected)`
- `to_be_empty()`
- `not_to_be_empty()`

### Option Matchers

- `to_be_some()`
- `to_be_none()`
- `to_be_some_and_equal(expected)`

### Result Matchers

- `to_be_ok()`
- `to_be_err()`

### Custom Matchers

- `to_satisfy(matcher_fn, error_message)` - Custom validation logic
- `to_satisfy_with_error(matcher_fn)` - Custom validation with detailed errors

### CKB-Specific Functions

#### Transaction Expectations
- `expect_transaction(recipe).to_have_method_path(path)`
- `expect_transaction(recipe).to_have_arguments_count(count)`
- `expect_transaction(recipe).to_have_argument_with_length(index, length)`

#### Cell Expectations
- `expect_cells(cells).to_have_known_cells(type)`
- `expect_cells(cells).to_have_custom_cells(type)`
- `expect_cells(cells).to_have_known_cells_count(type, count)`
- `expect_cells(cells).to_have_custom_cells_count(type, count)`
- `expect_cells(cells).to_have_total_cells_count(count)`

#### Helper Functions
- `expect_arguments(recipe)` - Get arguments with validation
- `expect_u64_argument(bytes, name)` - Parse u64 with validation
- `expect_u128_argument(bytes, name)` - Parse u128 with validation

### Macros

#### `validation_block!`
Groups related validations with a descriptive name:
```rust
validation_block!("Token validation", {
    let tokens = input_cells.get_known("xudt").ok_or("No tokens")?;
    expect(tokens).not_to_be_empty()?;
    // More validations...
    Ok(())
})?;
```

#### `validate_all!`
Runs multiple validations, stopping at the first error:
```rust
validate_all! {
    expect(amount).to_be_greater_than(0),
    expect(recipient).not_to_equal(sender),
    expect_cells(output_cells).to_have_custom_cells_count(b"receipt", 1),
}?;
```

## Real-World Example

```rust
pub fn validate_open_vault_transaction(
    recipe: &TransactionRecipe,
    input_cells: &ClassifiedCells,
    output_cells: &ClassifiedCells,
) -> Result<(), String> {
    // Validate transaction structure
    validate_all! {
        expect_transaction(recipe).to_have_method_path(b"CDP.openVault"),
        expect_transaction(recipe).to_have_arguments_count(2),
    }?;

    // Parse and validate arguments
    let args = expect_arguments(recipe)?;
    let collateral = expect_u128_argument(&args[0], "Collateral amount")?;
    let debt = expect_u128_argument(&args[1], "Debt amount")?;

    // Business logic validations
    expect(collateral).to_be_greater_than(0)?;
    
    validation_block!("Collateral ratio check", {
        if debt > 0 {
            let min_collateral = debt.saturating_mul(3).saturating_div(2);
            expect(collateral).to_be_greater_than_or_equal(min_collateral)?;
        }
        Ok(())
    })?;

    // Validate cells
    expect_cells(input_cells).to_have_known_cells("xudt")?;
    expect_cells(output_cells).to_have_custom_cells_count(b"vault", 1)?;

    Ok(())
}
```

## Benefits

1. **Readability**: Code reads like natural language
2. **Consistency**: Uniform API across different validation types
3. **Error Quality**: Descriptive error messages for debugging
4. **Composability**: Mix and match matchers as needed
5. **Type Safety**: Leverages Rust's type system
6. **No-std Compatible**: Works in CKB's no-std environment