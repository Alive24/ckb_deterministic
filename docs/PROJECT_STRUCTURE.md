# CKB Deterministic Project Structure

## Overview

This project demonstrates a clean separation between framework code, shared libraries, and contract implementations.

## Directory Structure

### `/ckb_deterministic` - Core Framework
The main deterministic framework providing:
- **Cell Classification**: Automatic categorization of cells by type
- **Transaction Context**: Unified transaction data loading and parsing
- **Validation Framework**: Structural and custom validation support
- **Jest-like Assertions**: Expressive validation API (`expect()`, `assert()`)

### `/example_libs/deterministic_cdp_shared` - Shared Library
Reusable components for CDP contracts:
- **Cell Collectors**: CDP-specific cell classifiers (vault, stable token)
- **Transaction Recipes**: Method definitions and argument parsing helpers
- **Validation Registry**: Transaction validation rules
- **Custom Validators**: Business logic validators using Jest-like assertions

### `/example_contracts/deterministic_cdp_project` - Contract Implementation
The actual smart contract containing:
- **CDPProject**: Main business logic handler
- **Method Handlers**: `open_vault()`, `close_vault()`, `update_vault()`
- **Entry Point**: Contract main function

## Key Design Decisions

1. **Framework vs Implementation**: The framework (ckb_deterministic) provides generic functionality, while specific business logic lives in contracts.

2. **Shared Libraries**: Common functionality that might be used by multiple contracts goes in the libs folder.

3. **Contract Focus**: The contract project contains only the essential business logic and entry point.

## Usage Flow

1. Transaction arrives at contract entry point
2. Framework creates transaction context (loads cells, parses recipe)
3. Contract creates project handler (CDPProject)
4. Validation registry checks structural constraints
5. Custom validators check business rules
6. Method-specific handler executes business logic

## Benefits

- **Clean Separation**: Framework, libraries, and contracts are clearly separated
- **Reusability**: Shared components can be used by multiple contracts
- **Testability**: Each layer can be tested independently
- **Type Safety**: Strong typing throughout with CellClassifier generics
- **Readable Validation**: Jest-like assertions make validators self-documenting