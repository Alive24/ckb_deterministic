# prototype-deterministic-recipe-based-contracts

## Background

### Conventions over Configurations

- Deterministic: Transactions with intent
- Recipe-based Validation: Abstract on the level of recipes
- One Source of Code: for both verification and transaction assembly on all ends (frontend, backend, and contracts)
- Role-based Authentication: Protocol, User, Public, and External

#### One Source of Code

- Parsing: Deserialize, authenticate
- Calculation: Deterministic output from input
- Verification: Data hash

##### Shared Code

- Transaction Skeleton: From recipe
- Serialization and Deserialization: `serde-molecules` + `serde-wasm-bindgen`
- Deterministic Calculation: CellInput -> CellOutput + OutputData
- (TBD) Verification on Data Hash

#### Role-based Cell Contracts and Verification

- Project Type
    - Type ID Authentication for protocol cells
    - Args Flag for Different Cells
    - Recipe-based validation
- Project User Lock
    - Proxied User Authentication on Intent
    - Recipe-based overriding
- (Optional) Project Public Lock
    - Type ID Authentication for public cells

## TODO

0. CLI
1. Define recipe in Rust
    - Better hinting and validation
    - Easier code scaffolding
2. Spawn-based invocable recipe validations
