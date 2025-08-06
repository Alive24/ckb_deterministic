# CKB Deterministic

A comprehensive framework for building deterministic CKB smart contracts with unified transaction recipe handling, automatic cell classification, Jest-like assertions, dependency validation, and project-agnostic transaction contexts.

## Overview

CKB Deterministic provides a standardized approach to:

- **Unified Transaction Recipes**: Single TransactionRecipe type generated from Molecule schema with extension traits for additional functionality
- **Universal Cell Classification**: Automatic classification system with SimpleCKB as default for cells without type scripts
- **Transaction Context**: Complete transaction contexts that include parsed recipes, classified cells, and dependency validation
- **Jest-like Assertions**: Familiar assertion API that returns proper Error types instead of String messages
- **Enhanced Validation Framework**: ValidationPredicate functions receive full TransactionContext for comprehensive validation
- **Known Scripts Registry**: Built-in support for 22+ known CKB ecosystem scripts
- **Dependency Management**: Automatic validation of required cell and header dependencies
- **SSRI Support**: Script-to-Script Remote Invocation pattern implementation
- **Direct Recipe Validation**: Validation rules check method paths internally without external matching
- **Project Flexibility**: Generic, reusable components that can be specialized for any CKB project
- **Type ID Support**: Complete Type ID implementation for creating unique cell types

## Features

- ✅ **Unified TransactionRecipe**: Single source of truth for transaction recipes using generated Molecule types
- ✅ **Universal Cell Classification**: Automatic SimpleCKB classification for basic cells, with rule-based system for custom types
- ✅ **Transaction Context**: Complete transaction state with recipe parsing, cell collection, and dependency tracking
- ✅ **Jest-like Assertions**: expect() API with comprehensive matchers for validation
- ✅ **Validation Framework**: Declarative validation rules with automatic enforcement
- ✅ **Dependency Validation**: Automatic validation of cell deps and header deps, including dep groups
- ✅ **Known Scripts Support**: Pre-configured support for xUDT, Spore, DAO, and 19+ other ecosystem scripts
- ✅ **Direct Recipe Validation**: Recipe-based validation without external method path matching
- ✅ **Error Propagation**: Proper error handling throughout the classification and validation pipeline
- ✅ **Detailed Error Codes**: Specific i8 error codes with #[repr(i8)] for direct casting
- ✅ **Extension Traits**: Add functionality to generated types without modification
- ✅ **Network Awareness**: Mainnet/testnet configuration for known scripts
- ✅ **Debugging Support**: Built-in debug logging for transaction analysis
- ✅ **Type ID Implementation**: Complete Type ID support for unique cell type creation

## Architecture

The framework follows a modular architecture with clear separation of concerns:

- **Core Library (`ckb_deterministic`)**: Generic, reusable components for any CKB project
  - Cell classification framework
  - Transaction context and recipe handling
  - Validation framework with dependency support
  - Jest-like assertions
  - Known scripts registry
  - Type ID implementation
  
- **Project-Specific Libraries**: Implement domain-specific logic
  - Custom cell types and classification rules
  - Business-specific validation logic
  - Domain types and constants
  - Error mapping

- **Smart Contracts**: Use the framework for transaction validation
  - Import core library and project-specific library
  - Create transaction context with classification
  - Apply validation rules
  - Execute business logic

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ckb_deterministic = { path = "../path/to/ckb_deterministic" }
```

## Quick Start

### 1. Cell Classification

The framework provides a universal cell classification system that automatically identifies SimpleCKB cells (cells without type scripts) and allows custom classification rules:

```rust
use ckb_deterministic::cell_classifier::{
    CellClass, ClassificationRule, RuleBasedClassifier, CellCollector
};
use ckb_deterministic::known_scripts::KnownScript;

// Create a classifier - SimpleCKB cells are automatically recognized
let classifier = RuleBasedClassifier::new("MyClassifier")
    // Add known scripts like xUDT
    .add_known_script(KnownScript::XUdt, KnownScript::XUdt.cell_class())
    // Add custom cells by type code hash
    .add_rule(ClassificationRule::TypeCodeHash {
        code_hash: my_custom_type_hash,
        class: CellClass::custom("my_custom_cell"),
    });

// Create collector with the classifier
let collector = CellCollector::new(classifier)
    .with_strict_mode(true); // Reject transactions with unidentified cells

// Collect and classify cells - errors are properly propagated
let (inputs, outputs) = collector.collect_inputs_and_outputs()?;
```

### 2. Transaction Context with Automatic Classification

```rust
use ckb_deterministic::transaction_context::TransactionContext;

// Create transaction context with automatic cell classification
let context = TransactionContext::new(collector)?;

// Access classified cells
let simple_ckb_cells = context.inputs.get_simple_ckb();
let xudt_cells = context.inputs.get_known("xudt");
let custom_cells = context.inputs.get_custom("my_custom_cell");

// Check for unidentified cells
if context.inputs.has_unidentified_cells() {
    return Err(Error::UnidentifiedCells);
}
```

### 3. Error Handling

All errors use `#[repr(i8)]` for elegant error code handling:

```rust
#[repr(i8)]
#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    // CKB System Errors
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,
    Encoding = 4,
    
    // Classification Errors
    UnidentifiedCells = 21,
    InvalidCodeHash = 23,
    
    // Validation Errors
    WrongMethodPath = 30,
    InvalidArgumentCount = 31,
    CellCountViolation = 32,
    CellRelationshipRuleViolation = 33,
    BusinessRuleViolation = 34,
    ExpectationViolation = 39,
    
    // Unknown error
    Unknown = -1,
}

// Errors can be directly cast to i8
fn main() -> i8 {
    match run() {
        Ok(_) => 0,
        Err(e) => e as i8,  // Direct cast, no helper method needed
    }
}
```

### 4. Known Scripts Support

The framework includes pre-configured support for 22+ known CKB ecosystem scripts:

```rust
use ckb_deterministic::known_scripts::KnownScript;

// Available known scripts include:
// - SimpleCKB (default for cells without type scripts)
// - xUDT, Spore, SporeCluster, SporeExtension
// - CKB DAO, Nervos DAO
// - CoTA, CoTARegistry, CoTAExtension
// - mNFT, TypeID, Unique Type
// - JoyID related scripts
// - RGBpp scripts
// - And more...

// Add known scripts to your classifier
let classifier = RuleBasedClassifier::new("MyClassifier")
    .add_known_script(KnownScript::XUdt, KnownScript::XUdt.cell_class())
    .add_known_script(KnownScript::Dao, KnownScript::Dao.cell_class());

// Known scripts include network-specific configurations
let xudt_info = KnownScript::XUdt.info(Network::Mainnet);
println!("xUDT code hash: {:?}", xudt_info.code_hash);
```

### 5. Comprehensive Validation Framework

The framework provides flexible validation rules with enhanced ValidationPredicate that receives full TransactionContext:

```rust
use ckb_deterministic::validation::{TransactionValidationRules, CellCountConstraint};
use ckb_deterministic::transaction_deps::{CellDepInfo, OutPointInfo};

// Define comprehensive validation rules
let rules = TransactionValidationRules::new(b"openVault".to_vec())
    // Argument validation
    .with_arguments(2)
    
    // Cell count constraints
    .with_simple_ckb(
        CellCountConstraint::exactly(1),  // Input constraint
        CellCountConstraint::at_most(1),  // Output constraint
    )
    .with_custom_cell(
        "vault",
        CellCountConstraint::exactly(0),  // No vault inputs
        CellCountConstraint::exactly(1),  // One vault output
    )
    
    // Cell relationship rules with full context access
    .with_cell_relationship(
        "validate_lock_consistency".to_string(),
        "Ensure lock scripts remain consistent".to_string(),
        vec!["vault".to_string()],
        |context| {
            // Access full transaction context
            let inputs = context.input_cells.get_custom("vault");
            let outputs = context.output_cells.get_custom("vault");
            // Validate using context.cell_deps, context.header_deps, etc.
            Ok(())
        }
    )
    
    // Business logic validation
    .with_business_rule(
        "validate_collateral_ratio".to_string(),
        "Ensure sufficient collateral".to_string(),
        vec!["vault".to_string(), "xudt".to_string()],
        validate_collateral_ratio,
    )
    
    // Dependency validation
    .with_required_cell_deps(vec![vault_script_dep])
    .with_required_header_deps(vec![price_oracle_block_hash]);

// Apply all validation rules - method path is checked internally
let result = rules.validate(&context);
```

### 6. Transaction Recipe with Dependencies

Transaction recipes now support dependency specification:

```rust
use ckb_deterministic::transaction_recipe::TransactionRecipeExt;

// Create a recipe with dependencies
let recipe = TransactionRecipe::new_builder()
    .method_path(method_path_bytes)
    .arguments(arguments)
    .cell_deps(cell_deps)      // Optional cell dependencies
    .header_deps(header_deps)  // Optional header dependencies
    .build();

// Parse recipe from witness (includes deps)
let recipe = TransactionRecipe::from_witness(0)?;

// Access dependency information
let has_deps = recipe.has_cell_deps() || recipe.has_header_deps();
```

### 7. Type ID Support

The framework includes a complete Type ID implementation for creating unique cell types:

```rust
use ckb_deterministic::type_id::{
    validate_type_id, 
    load_type_id_from_script_args,
    check_type_id_from_script_args,
    calculate_type_id
};

// In a smart contract - validate Type ID from script args
fn main() -> Result<(), Error> {
    // Simple one-line validation if Type ID is at the beginning of script args
    check_type_id_from_script_args()?;
    
    // Or manually load and validate
    let type_id = load_type_id_from_script_args(0)?;
    validate_type_id(type_id)?;
    
    // Your contract logic here...
    Ok(())
}

// For testing - calculate expected Type ID
#[test]
fn test_type_id_calculation() {
    let input_cell = vec![/* serialized first input cell */];
    let output_index = 0;
    let expected_type_id = calculate_type_id(&input_cell, output_index);
    
    // Use this to verify your Type ID cells
}
```

Type ID Rules:
- At most one input and one output cell can have the same Type ID
- When creating a new Type ID cell, the Type ID must equal `blake2b(first_input | output_index)`
- Type ID ensures global uniqueness for cell types

### 8. Jest-like Assertions

The framework provides a familiar assertion API for validation:

```rust
use ckb_deterministic::assertions::{expect, assert};

// Basic assertions - all return Error instead of String
expect(actual_value).to_equal(expected_value)?;
expect(vec.len()).to_be_greater_than(0)?;

// Transaction recipe assertions
expect_transaction(&context.recipe)
    .to_have_method_path(b"transfer")?
    .to_have_arguments_count(3)?;

// Cell assertions
expect_cells(&context.input_cells)
    .to_have_known_cells_count("xudt", 2)?
    .to_have_custom_cells("vault")?;

// Dependency assertions
expect_deps(&context.cell_deps)
    .to_have_cell_dep(&tx_hash, index)?
    .to_have_deps_for_script(KnownScript::XUdt, Network::Mainnet)?;

expect_headers(&context.header_deps)
    .to_have_header(&oracle_block_hash)?
    .to_have_count(1)?;
```

## Complete Example

Here's a complete example showing how to use the framework in a smart contract:

```rust
use ckb_deterministic::{
    cell_classifier::{RuleBasedClassifier, CellCollector},
    transaction_context::TransactionContext,
    validation::TransactionValidationRules,
    known_scripts::KnownScript,
    assertions::expect,
    errors::Error,
};

fn main() -> Result<(), Error> {
    // 1. Set up cell classification
    let classifier = RuleBasedClassifier::new("CDPValidator")
        .add_known_script(KnownScript::XUdt, KnownScript::XUdt.cell_class())
        .add_rule(ClassificationRule::TypeCodeHash {
            code_hash: VAULT_TYPE_HASH,
            class: CellClass::custom("vault"),
        });

    // 2. Create transaction context
    let collector = CellCollector::new(classifier).with_strict_mode(true);
    let context = TransactionContext::new(collector)?;

    // 3. Define validation rules for different recipes
    let open_vault_rules = TransactionValidationRules::new(b"openVault".to_vec())
        .with_arguments(2)
        .with_simple_ckb(
            CellCountConstraint::exactly(1),
            CellCountConstraint::at_most(1),
        )
        .with_custom_cell(
            "vault",
            CellCountConstraint::exactly(0),
            CellCountConstraint::exactly(1),
        )
        .with_business_rule(
            "validate_collateral".to_string(),
            "Ensure sufficient collateral for vault".to_string(),
            vec!["xudt".to_string()],
            |context| {
                // ValidationPredicate receives full TransactionContext
                let xudt_cells = context.input_cells.get_known("xudt");
                // Validate collateral using full context
                Ok(())
            }
        );

    // 4. Try to match and validate - rules check method path internally
    if open_vault_rules.validate(&context).is_ok() {
        // Execute business logic for openVault
        process_open_vault(&context)?;
    } else {
        return Err(Error::WrongMethodPath);
    }

    Ok(())
}
```

## Project Structure Example

Here's how a typical project using CKB Deterministic might be structured:

```bash
my-ckb-project/
├── contracts/
│   ├── my-contract/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs          # Uses ckb_deterministic for validation
│   │       └── recipes.rs       # Transaction recipes with validation rules
│   └── ...
├── libs/
│   └── shared-lib/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── cell_classifier.rs   # Project-specific cell types
│           ├── types.rs             # Domain types and constants
│           ├── error.rs             # Error mapping
│           └── validators.rs        # Business validation logic
│   └── ...
├── schemas/
│   └── my-protocol.mol          # Molecule schemas
└── tests/
    └── integration_tests.rs     # Test transaction flows
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License.

## Extra Notes

Limitations on recipes when unidentified cells are not allowed might conflict with the SSRI intention to chain transactions. Will address this in the future.
