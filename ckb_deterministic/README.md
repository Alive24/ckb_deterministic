# prototype-deterministic-recipe-based-contracts

## Background

### Conventions over Configurations

- Deterministic: With inputs including intent, cell deps, and the same script, the output should be the same
- One Source of Code: for both verification and transaction assembly on all ends (frontend, backend, and contracts)
- Recipe-based Validation: Strictly validate transactions based on recipes
- Role-based Layered Cells with Authentications: Protocol, User, Public, and External

#### One Source of Code

- Parsing: Deserialize, authenticate
- Calculation: Deterministic output from input
- Verification: Data hash

##### Shared Code

- Transaction Skeleton: From recipe
- Serialization and Deserialization: `serde-molecules` + `serde-wasm-bindgen`
- Deterministic Calculation: CellInput -> CellOutput + OutputData
- (TBD) Verification on Data Hash

#### Role-based Layered Cells with Authentications

##### Interconnected Authentications

- Protocol Cells:
    - Unique Type ID
    - Global Configs
- User Cells:
    - Project Type connected to Recipe
    - Project User Lock which is Proxied and Overridable
    - User Data
- Public Cells:
    - Unique Type ID
    - Project Type connected to Recipe
    - Project Public Lock connected to Protocol
    - Public Data
- External Cells:
    - External Data
    - Authenticated by Protocol

##### Simplified Scripts Structure

- Project Type
    - Type ID Authentication on protocol cells
    - Args Flag for Different Cells
    - Recipe-based validation and calculation
    - Deterministic verification
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

