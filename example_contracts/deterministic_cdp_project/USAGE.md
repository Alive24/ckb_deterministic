# CDP Contract Example - Using ckb_deterministic

This example demonstrates how to use the `ckb_deterministic` framework to build a CDP (Collateralized Debt Position) smart contract on CKB.

## Key Concepts

### 1. Transaction Context
The framework provides a unified way to load and process transaction data:
```rust
let context = create_cdp_transaction_context()?;
```

This single line:
- Loads all input and output cells from the transaction
- Classifies cells using your custom classifier (vault, stable token, xUDT, etc.)
- Parses the transaction recipe (method path + arguments)
- Prepares everything for validation

### 2. Cell Classification
Cells are automatically classified into known types (like xUDT) and custom types (like vault):
```rust
// Define custom cell types in your classifier
classifier.register_custom(b"vault", Box::new(VaultCellMatcher));
classifier.register_custom(b"stable", Box::new(StableTokenMatcher));
```

### 3. Transaction Validation
The framework provides a two-layer validation approach:

**Structural Validation** (handled by the framework):
- Correct number of arguments
- Expected cell types present
- Cell count constraints

**Business Logic Validation** (your custom validators):
```rust
.with_custom_validator(validate_open_vault_transaction)
```

### 4. Method Routing
Methods are routed using deterministic hashes for efficiency:
```rust
match context.method_path_hash {
    hash if hash == self.open_vault_hash => self.open_vault(context),
    hash if hash == self.close_vault_hash => self.close_vault(context),
    // ...
}
```

## Usage Pattern

1. **Create Transaction Context**
   ```rust
   let context = create_cdp_transaction_context()?;
   ```

2. **Create Project Handler**
   ```rust
   let cdp_project = CDPProject::new();
   ```

3. **Process Transaction**
   ```rust
   cdp_project.process_transaction(&context)?;
   ```

## Benefits

- **Deterministic**: All operations are deterministic and reproducible
- **Type-Safe**: Cell classification prevents type confusion bugs
- **Validated**: Automatic structural validation with custom business rules
- **Efficient**: Hash-based routing and minimal allocations
- **Testable**: Clear separation of concerns enables easy testing

## Testing

The framework makes testing straightforward:
```rust
// Create a recipe for testing using helper functions
let recipe = create_open_vault_recipe(collateral_amount, debt_amount)?;

// Or manually create a recipe using the generated builder
use ckb_deterministic::generated::{TransactionRecipeBuilder, BytesVec, Bytes};
let recipe = TransactionRecipeBuilder::default()
    .method_path(Bytes::from(b"CDP.openVault".to_vec()))
    .arguments(BytesVec::from(vec![
        Bytes::from(collateral_amount.to_le_bytes().to_vec()),
        Bytes::from(debt_amount.to_le_bytes().to_vec()),
    ]))
    .build();

// Test validation rules
let result = validator(&recipe, &input_cells, &output_cells);
assert!(result.is_ok());
```

## Jest-like Assertions

The framework includes Jest-style assertions for readable validators:
```rust
// Value assertions
expect(amount).to_be_greater_than(0)?;

// Transaction assertions  
expect_transaction(recipe).to_have_method_path(b"CDP.openVault")?;
expect_transaction(recipe).to_have_arguments_count(2)?;

// Cell assertions
expect_cells(input_cells).to_have_known_cells_count("xudt", 1)?;
expect_cells(output_cells).to_have_custom_cells(b"vault")?;

// Validation blocks
validation_block!("Collateral ratio check", {
    let min_collateral = debt * 3 / 2;
    expect(collateral).to_be_greater_than_or_equal(min_collateral)?;
    Ok(())
})?;
```