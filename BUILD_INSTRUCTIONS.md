# Build Instructions

## Prerequisites

- Rust toolchain with RISC-V target:
  ```bash
  rustup target add riscv64imac-unknown-none-elf
  ```

## Building the Project

### 1. Build Libraries (Host Platform)

To build the main library and shared libraries:

```bash
make build
```

This builds:
- `ckb_deterministic` - The main deterministic framework library
- `deterministic-cdp-shared` - Shared library for example contracts

### 2. Build Example Contracts (RISC-V Target)

To build example contracts for CKB blockchain:

```bash
make build-contracts
```

This builds the example contracts in `example_contracts/` directory for the RISC-V target.

The compiled binaries will be placed in `build/release/`.

## Output

- Libraries: `target/release/`
- Contracts: `build/release/` (when using make) or `target/riscv64imac-unknown-none-elf/release/` (when using cargo directly)

## Notes

- The example contracts demonstrate how to use the `ckb_deterministic` framework
- Contract binaries are built for RISC-V target to run on CKB
- Libraries are built for the host platform for development and testing