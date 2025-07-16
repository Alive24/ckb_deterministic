# CKB Deterministic

A comprehensive framework for building deterministic CKB smart contracts with unified transaction recipe handling, automatic cell classification, Jest-like assertions, and project-agnostic transaction contexts.

## Overview

CKB Deterministic provides a standardized approach to:

- **Unified Transaction Recipes**: Single TransactionRecipe type generated from Molecule schema with extension traits for additional functionality
- **Automatic Cell Classification**: Rule-based cell classification system for organizing transaction inputs/outputs by type hash
- **Transaction Context**: Complete transaction contexts that include parsed recipes and classified cells
- **Jest-like Assertions**: Familiar assertion API for writing readable validation logic
- **Validation Framework**: Two-phase validation with structural rules and custom validators
- **SSRI Support**: Script-to-Script Remote Invocation pattern implementation
- **Project Flexibility**: Generic, reusable components that can be specialized for any CKB project

## Features

- ✅ **Unified TransactionRecipe**: Single source of truth for transaction recipes using generated Molecule types
- ✅ **Cell Classification**: Automatic classification of cells into known, custom, and unidentified categories
- ✅ **Transaction Context**: Complete transaction state with recipe parsing and cell collection
- ✅ **Jest-like Assertions**: expect() API with comprehensive matchers for validation
- ✅ **Validation Registry**: Declarative validation rules with automatic enforcement
- ✅ **Method Path Hashing**: Blake2b-256 hash-based method identification (first 8 bytes as u64)
- ✅ **Detailed Error Codes**: Specific i8 error codes for different validation failures
- ✅ **Extension Traits**: Add functionality to generated types without modification
- ✅ **Debugging Support**: Built-in debug logging for transaction analysis

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ckb_deterministic = { path = "../path/to/ckb_deterministic" }
ckb-std = "0.16"
```

## Quick Start

### 1. Basic Transaction Recipe Usage

```rust
use ckb_deterministic::transaction_recipe::{
    parse_transaction_recipe, 
    create_transaction_recipe,
    TransactionRecipeExt
};

// Parse recipe from witness
let recipe = parse_transaction_recipe()?.unwrap();

// Access recipe data using extension trait
let method_path_hash = recipe.method_path_hash()?;
let method_name = recipe.method_path_name()?;
let arguments = recipe.arguments_vec();

// Create new recipe
let new_recipe = create_transaction_recipe("UDT.transfer", &[
    b"recipient_address".to_vec(),
    1000u64.to_le_bytes().to_vec()
])?;
```

### 2. Cell Classification

```rust
use ckb_deterministic::cell_classifier::{
    RuleBasedClassifier, 
    CellCollector, 
    CellClass
};

// Create classifier with type hash rules
let classifier = RuleBasedClassifier::new("MyProject")
    .add_type_hash([1u8; 32], CellClass::known("udt"))
    .add_type_hash([2u8; 32], CellClass::custom("vault"))
    .add_type_hash([3u8; 32], CellClass::custom("amm_pool"));

// Collect and classify cells
let collector = CellCollector::new(classifier);
let (input_cells, output_cells) = collector.collect_inputs_and_outputs()?;

// Access classified cells
let udt_inputs = &input_cells.known_cells[b"udt".as_slice()];
let vault_outputs = &output_cells.custom_cells[b"vault".as_slice()];
let unidentified = &input_cells.unidentified_cells;
```

### 3. Complete Transaction Context

```rust
use ckb_deterministic::transaction_context::{
    TransactionContext,
    create_transaction_context
};

// Create complete transaction context
let context = create_transaction_context(collector)?;

// Access all transaction data
println!("Method: {}", context.method_path_name);
println!("Hash: {}", context.method_path_hash);
println!("Arguments: {:?}", context.arguments);
println!("Input cells: {}", context.input_cells.total_cell_count());
println!("Output cells: {}", context.output_cells.total_cell_count());

// Validate transaction
context.validate()?;
```

### 4. Jest-like Assertions

```rust
use ckb_deterministic::assertions::*;
use ckb_deterministic::{validate_all, validation_block};

// Basic assertions
expect(value).to_equal(expected)?;
expect(value).to_be_greater_than(10)?;
expect(collection).not_to_be_empty()?;

// Transaction-specific assertions
expect_transaction(recipe).to_have_method_path(b"CDP.openVault")?;
expect_transaction(recipe).to_have_arguments_count(2)?;
expect_cells(input_cells).to_have_custom_cells_count(b"vault", 1)?;

// Combine multiple validations
validate_all! {
    expect(collateral_amount).to_be_greater_than(0u128),
    expect_transaction(recipe).to_have_method_path(b"CDP.openVault"),
    validation_block!("Collateral ratio check", {
        let ratio = collateral_amount * 100 / debt_amount;
        expect(ratio).to_be_greater_than_or_equal(150)
    }),
}?;
```

### 5. Validation Framework

```rust
use ckb_deterministic::validation::{
    ValidationRegistry,
    TransactionValidationRules,
    CellCountConstraint,
};

// Create validation registry
let mut registry = ValidationRegistry::new();

// Register validation rules for a method
registry.register(
    TransactionValidationRules::new(b"CDP.openVault")
        .with_argument_count(2)
        .with_cell_constraint(b"xudt", 
            CellCountConstraint::at_least(1),  // input
            CellCountConstraint::any()         // output
        )
        .with_cell_constraint(b"vault",
            CellCountConstraint::exactly(0),   // input
            CellCountConstraint::exactly(1)    // output
        )
        // Enable automatic known script dependency validation
        .with_auto_known_script_validation()
        // Add required dependencies
        .with_required_cell_dep(oracle_dep, 0, DepType::Code)
        .with_required_header_dep(recent_block_hash)
        // Custom validator now receives deps
        .with_custom_validator(validate_open_vault_transaction)
);

// Validate transaction with dependencies
registry.validate_with_deps(
    &recipe, 
    &input_cells, 
    &output_cells,
    &cell_deps,
    &header_deps
)?;
```

## Architecture

### Core Components

1. **TransactionRecipe** (generated): Core Molecule-generated type for SSRI recipes
2. **TransactionRecipeExt** (trait): Extension methods for TransactionRecipe
3. **CellClassifier** (trait): Interface for cell classification logic
4. **CellCollector**: Collects and classifies transaction cells
5. **TransactionContext**: Complete transaction state container
6. **Assertions Module**: Jest-like assertion API for readable validation logic
7. **ValidationRegistry**: Declarative validation rules with automatic enforcement

### Design Patterns

#### Extension Trait Pattern
```rust
// Generated type stays untouched
pub struct TransactionRecipe(molecule::bytes::Bytes);

// Extension trait adds functionality
pub trait TransactionRecipeExt {
    fn method_path_hash(&self) -> Result<u64, Error>;
    fn method_path_name(&self) -> Result<String, Error>;
    fn arguments_vec(&self) -> Vec<Vec<u8>>;
}

impl TransactionRecipeExt for TransactionRecipe {
    // Implementation details...
}
```

#### Rule-Based Classification
```rust
let classifier = RuleBasedClassifier::new("ProjectName")
    .add_type_hash(KNOWN_TYPE_HASH, CellClass::known("label"))
    .add_type_hash(CUSTOM_TYPE_HASH, CellClass::custom("label"));
```

#### Method Path Hashing
```rust
use ckb_deterministic::transaction_recipe::method_path;

// Convert method name to hash for efficient comparison
let transfer_hash = method_path("UDT.transfer");
let mint_hash = method_path("UDT.mint");

match context.method_path_hash {
    hash if hash == transfer_hash => process_transfer(),
    hash if hash == mint_hash => process_mint(),
    _ => return Err(Error::RecipeError),
}
```

## Examples

### Project-Specific Implementation

See the complete example in `example_libs/deterministic_cdp/`:

```rust
/// CDP-specific wrapper
pub struct DeterministicCDP {
    vault_type_hash: [u8; 32],
    collateral_type_hash: [u8; 32],
    debt_type_hash: [u8; 32],
}

impl DeterministicCDP {
    pub fn create_classifier(&self) -> RuleBasedClassifier {
        RuleBasedClassifier::new("CDP")
            .add_type_hash(self.vault_type_hash, CellClass::custom("vault"))
            .add_type_hash(self.collateral_type_hash, CellClass::custom("collateral"))
            .add_type_hash(self.debt_type_hash, CellClass::custom("debt"))
    }
    
    pub fn process_transaction<C>(&self, context: &TransactionContext<C>) -> Result<(), Error>
    where
        C: CellClassifier,
    {
        let open_vault_hash = method_path("CDP.openVault");
        let close_vault_hash = method_path("CDP.closeVault");
        
        match context.method_path_hash {
            hash if hash == open_vault_hash => self.process_open_vault(context),
            hash if hash == close_vault_hash => self.process_close_vault(context),
            _ => Err(Error::RecipeError),
        }
    }
}
```

### Contract Implementation

See the complete example in `example_contracts/deterministic_cdp_project/`:

```rust
pub fn main() -> Result<(), Error> {
    // Step 1: Create transaction context with CDP-specific classifier
    let context = create_cdp_transaction_context()?;
    
    // Step 2: Create validation registry with rules
    let registry = create_cdp_validation_registry();
    
    // Step 3: Validate transaction structure and constraints
    // Returns specific error codes: 41-46 for different validation failures
    registry.validate(&context.recipe, &context.input_cells, &context.output_cells)
        .map_err(|e| Error::from(e))?;
    
    // Step 4: Process transaction with business logic
    let cdp_project = CDPProject::new();
    cdp_project.process_transaction(&context)?;
    
    Ok(())
}
```

### Using Jest-like Assertions in Validators

```rust
use ckb_deterministic::{validate_all, validation_block, assertions::*};

pub fn validate_open_vault_transaction(
    recipe: &TransactionRecipe,
    input_cells: &ClassifiedCells,
    output_cells: &ClassifiedCells,
    cell_deps: &[CellDepInfo],
    header_deps: &[[u8; 32]],
) -> Result<(), String> {
    // Validate transaction structure
    validate_all! {
        expect_transaction(recipe).to_have_method_path(b"CDP.openVault"),
        expect_transaction(recipe).to_have_arguments_count(2),
    }?;
    
    // Parse and validate arguments
    let args = recipe.arguments_vec();
    let collateral_amount = expect_u128_argument(&args[0], "Collateral amount")?;
    let debt_amount = expect_u128_argument(&args[1], "Debt amount")?;
    
    // Business logic validations
    validate_all! {
        expect(collateral_amount)
            .to_be_greater_than(0u128)
            .map_err(|_| "Collateral amount must be greater than zero".to_string()),
        validation_block!("Collateral ratio check", {
            if debt_amount > 0 {
                let minimum_collateral = debt_amount.saturating_mul(3).saturating_div(2);
                expect(collateral_amount)
                    .to_be_greater_than_or_equal(minimum_collateral)
                    .map_err(|_| format!(
                        "Insufficient collateral ratio: need {} but got {}", 
                        minimum_collateral, collateral_amount
                    ))?;
            }
            Ok(())
        }),
        // Validate dependencies
        validation_block!("Dependency validation", {
            // Check xUDT dependencies are present
            expect_deps(cell_deps)
                .to_have_deps_for_script(KnownScript::XUdt, Network::Mainnet)?;
            // Check time validation header
            expect_headers(header_deps)
                .to_have_count(1)
                .map_err(|_| "Header dependency required for time validation".to_string())?;
            Ok(())
        }),
    }?;
    
    Ok(())
}
```

## Method Path Conventions

Please use dot notation as in SSRI protocol: `UDT.transfer`, `AMM.swapExactTokensForTokens`

Each method path is hashed using Blake2b-256, with the first 8 bytes used as a u64 identifier for efficient matching.

## Error Handling

CKB Deterministic provides detailed error codes for different validation failures:

```rust
use ckb_deterministic::errors::{Error, ValidationError};

// Framework returns specific ValidationError variants
match validation_result {
    Err(ValidationError::WrongMethodPath { .. }) => {}, // Wrong method called
    Err(ValidationError::InvalidArgumentCount { .. }) => {}, // Wrong arg count
    Err(ValidationError::CellCountViolation { .. }) => {}, // Cell count mismatch
    Err(ValidationError::UnidentifiedCells { .. }) => {}, // Unknown cells
    Err(ValidationError::CustomValidation(_)) => {}, // Business logic failed
    Ok(()) => {},
}

// Automatic conversion to specific i8 error codes
#[repr(i8)]
pub enum Error {
    // Business logic errors (1-20)
    InvalidArguments = 1,
    UnderCollateralized = 4,
    
    // Validation errors (41-60)
    WrongMethodPath = 41,          // Called unknown method
    InvalidArgumentCount = 42,      // Wrong number of arguments
    MissingRequiredCells = 43,     // Required input cells not found
    TooManyCells = 44,             // Too many output cells
    CustomValidationFailed = 46,   // Business rule failed
    
    // System errors (61-80)
    IndexOutOfBound = 62,
    ItemMissing = 63,
}

// Proper error propagation in contracts
registry.validate(&context.recipe, &context.input_cells, &context.output_cells)
    .map_err(|e| Error::from(e))?;  // Converts to specific error codes
```

## Cell Classification

### Cell Classes

- **Known**: Standard CKB cell types (like UDT, DAO, etc.)
- **Custom**: Project-specific cell types
- **Unidentified**: Cells that don't match any classification rules

### Strict Mode

```rust
let collector = CellCollector::new(classifier)
    .with_strict_mode(true); // Reject transactions with unidentified cells

let context = create_transaction_context(collector)?; // Fails if unidentified cells found
```

## Example: Integration with CKBoost

CKB Deterministic is designed to integrate seamlessly with CKBoost and other CKB projects:

```rust
// CKBoost-specific wrapper
pub struct CKBoostTransactionContext {
    inner: TransactionContext<RuleBasedClassifier>,
    recipe: CKBoostTransactionRecipe,
    // ... CKBoost-specific fields
}

// Uses CKB Deterministic internally
impl CKBoostTransactionContext {
    pub fn new(collector: CKBoostCellCollector) -> Result<Self, Error> {
        let generic_context = TransactionContext::new(collector.into_generic())?;
        // ... wrap with CKBoost-specific functionality
    }
}
```

## Assertion API Reference

### Basic Matchers
- `expect(value).to_equal(expected)` - Exact equality
- `expect(value).to_be_greater_than(threshold)` - Greater than comparison
- `expect(value).to_be_greater_than_or_equal(threshold)` - Greater or equal
- `expect(value).to_be_less_than(threshold)` - Less than comparison
- `expect(value).to_be_less_than_or_equal(threshold)` - Less or equal
- `expect(value).to_be_true()` / `to_be_false()` - Boolean assertions
- `expect(collection).to_be_empty()` / `not_to_be_empty()` - Collection checks
- `expect(collection).to_have_length(n)` - Length assertions
- `expect(collection).to_contain(&item)` - Membership check

### Transaction-Specific Matchers
- `expect_transaction(recipe).to_have_method_path(path)` - Method path validation
- `expect_transaction(recipe).to_have_arguments_count(n)` - Argument count check
- `expect_transaction(recipe).to_have_argument_with_length(index, length)` - Arg length
- `expect_cells(cells).to_have_known_cells(label)` - Known cell presence
- `expect_cells(cells).to_have_custom_cells(label)` - Custom cell presence
- `expect_cells(cells).to_have_custom_cells_count(label, n)` - Exact cell count

### Macros
- `validate_all!` - Run multiple validations, short-circuiting on first error
- `validation_block!` - Group related validations with a descriptive name

## Development Workflow

1. **Define Method Paths**: Create constants for your project's SSRI methods
2. **Create Classifier**: Define type hash rules for your cell types
3. **Setup Validation Rules**: Register structural constraints with ValidationRegistry
4. **Write Custom Validators**: Use Jest-like assertions for business logic
5. **Implement Processing**: Handle different method paths in your contract
6. **Add Tests**: Verify behavior with different transaction patterns
7. **Optimize**: Use method path hashing for efficient dispatch

## Testing

Run the test suite:

```bash
# Test the core library
cargo test --lib

# Test with example projects
cargo test --workspace
```

## Contributing

CKB Deterministic follows these principles:

- **Project Agnostic**: Core functionality should work for any CKB project
- **Extension Pattern**: Add functionality through traits, not modification
- **Generated Types**: Never modify generated Molecule code
- **Comprehensive Testing**: Cover edge cases and error conditions

## License

[Add your license information here]

## Support

For issues and questions:
- [GitHub Issues](https://github.com/your-org/ckb_deterministic/issues)
- [Documentation](https://docs.your-domain.com/ckb_deterministic)