# ckb_deterministic Testing Guide

## Overview

The ckb_deterministic project provides a deterministic framework for CKB smart contract development. This library focuses on unit testing its core components. Integration testing with actual smart contracts should be performed in downstream projects (like CKBoost) that build upon this framework.

## Test Structure

### Core Library Tests (`ckb_deterministic/tests/`)

The library has comprehensive unit tests covering all major components:

- **assertions_tests.rs** (14 tests) - Tests for the assertion framework
- **cell_classifier_tests.rs** (8 tests) - Tests for cell classification logic
- **comprehensive_tests.rs** (37 tests) - Comprehensive unit tests for all modules
- **known_scripts_tests.rs** (10 tests) - Tests for known script registry
- **transaction_context_tests.rs** (9 tests) - Tests for transaction context
- **validation_tests.rs** (7 tests) - Tests for validation rules
- **lib.rs unit tests** (24 tests) - Core module unit tests

Total: 109 unit tests

## Running Tests

### Run All Tests

```bash
cargo test --package ckb_deterministic
```

## Test Coverage

### Cell Classification

- Simple CKB cells (no type script)
- Known script types (xUDT, NervosDAO, Spore, etc.)
- Custom cells with predicates
- Lock script classification
- Unidentified cells

### Transaction Validation

- Method path validation
- Argument count validation
- Cell count constraints
- Cell relationship rules
- Business rule validation
- Dependency validation

### Transaction Context

- Recipe creation and parsing
- Cell collection and classification
- Dependency management (cell deps & header deps)
- Context construction

### Error Handling

- All error types are tested
- Error propagation through the framework
- Proper error messages

## Writing New Tests

Add new unit tests in the appropriate test file under `ckb_deterministic/tests/`:

```rust
#[test]
fn test_my_feature() {
    // Your test code
}
```

## Integration Testing

Integration testing with actual smart contracts should be done in downstream projects that use ckb_deterministic. For example:

- **CKBoost**: Will contain integration tests for boost functionality
- **Other projects**: Should test their specific contract logic

This separation ensures that:

1. The framework remains focused on core functionality
2. Contract-specific tests live with their implementations
3. Tests are meaningful and maintainable
