# CKB Deterministic

A comprehensive framework for building deterministic CKB smart contracts with unified transaction recipe handling, automatic cell classification, and project-agnostic transaction contexts.

## Overview

CKB Deterministic provides a standardized approach to:

- **Unified Transaction Recipes**: Single TransactionRecipe type generated from Molecule schema with extension traits for additional functionality
- **Automatic Cell Classification**: Rule-based cell classification system for organizing transaction inputs/outputs by type hash
- **Transaction Context**: Complete transaction contexts that include parsed recipes and classified cells
- **SSRI Support**: Script-to-Script Remote Invocation pattern implementation
- **Project Flexibility**: Generic, reusable components that can be specialized for any CKB project

## Features

- ✅ **Unified TransactionRecipe**: Single source of truth for transaction recipes using generated Molecule types
- ✅ **Cell Classification**: Automatic classification of cells into known, custom, and unidentified categories
- ✅ **Transaction Context**: Complete transaction state with recipe parsing and cell collection
- ✅ **Method Path Hashing**: Blake2b-256 hash-based method identification (first 8 bytes as u64)
- ✅ **Error Handling**: Comprehensive error types with CKB system error integration
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

## Architecture

### Core Components

1. **TransactionRecipe** (generated): Core Molecule-generated type for SSRI recipes
2. **TransactionRecipeExt** (trait): Extension methods for TransactionRecipe
3. **CellClassifier** (trait): Interface for cell classification logic
4. **CellCollector**: Collects and classifies transaction cells
5. **TransactionContext**: Complete transaction state container

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

See the complete example in `example_contracts/deterministic_cdp_project_type/`:

```rust
pub fn run() -> Result<(), Error> {
    // Create project-specific components
    let cdp = DeterministicCDP::new();
    let classifier = cdp.create_classifier();
    let collector = CellCollector::new(classifier);
    
    // Create unified transaction context
    let context = create_transaction_context(collector)?;
    
    // Validate and process
    context.validate()?;
    cdp.process_transaction(&context)?;
    
    Ok(())
}
```

## Method Path Conventions

Please use dot notation as in SSRI protocol: `UDT.transfer`, `AMM.swapExactTokensForTokens`

Each method path is hashed using Blake2b-256, with the first 8 bytes used as a u64 identifier for efficient matching.

## Error Handling

```rust
use ckb_deterministic::errors::Error;

match result {
    Err(Error::DataError) => return 1,           // Cell data parsing error
    Err(Error::UnidentifiedCells) => return 2,   // Unknown cells in strict mode
    Err(Error::RecipeError) => return 3,         // Recipe parsing error
    Err(Error::SystemError(code)) => return code, // CKB system error
    Ok(()) => return 0,
}
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

## Development Workflow

1. **Define Method Paths**: Create constants for your project's SSRI methods
2. **Create Classifier**: Define type hash rules for your cell types
3. **Implement Processing**: Handle different method paths in your contract
4. **Add Tests**: Verify behavior with different transaction patterns
5. **Optimize**: Use method path hashing for efficient dispatch

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