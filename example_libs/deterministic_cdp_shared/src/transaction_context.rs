use ckb_deterministic::{
    transaction_context::{TransactionContext, create_transaction_context},
    cell_classifier::{CellCollector, CellClassifier},
    validation::{TransactionValidationRules, ValidationRegistry, CellCountConstraint},
    known_scripts::Network,
    transaction_deps::DepType,
};
use crate::{Error, cell_collector::create_cdp_classifier};

// CDP-specific transaction context creation
pub fn create_cdp_transaction_context() -> Result<TransactionContext<impl CellClassifier>, Error> {
    let classifier = create_cdp_classifier();
    let collector = CellCollector::new(classifier).with_strict_mode(false);
    
    create_transaction_context(collector)
        .map_err(|e| Error::from(e))
}


/// Create a validation registry for CDP transactions
pub fn create_cdp_validation_registry() -> ValidationRegistry {
    create_cdp_validation_registry_with_network(Network::Mainnet)
}

/// Create a validation registry for CDP transactions with specific network
pub fn create_cdp_validation_registry_with_network(network: Network) -> ValidationRegistry {
    let mut registry = ValidationRegistry::new();
    
    // Example CDP-specific dependency (hypothetical)
    let cdp_oracle_dep: [u8; 32] = [100u8; 32]; // CDP price oracle cell dep
    let recent_block_hash: [u8; 32] = [200u8; 32]; // Recent block for time validation
    
    // Add validation rules for open vault transaction
    let open_vault_rules = TransactionValidationRules::new(b"CDP.openVault")
        .with_arguments(2) // collateral_amount, debt_amount
        // Input requirements
        .with_known_cell(b"xudt", CellCountConstraint::at_least(1), CellCountConstraint::any()) // Collateral tokens
        .with_known_cell(b"simple_ckb", CellCountConstraint::at_least(1), CellCountConstraint::any()) // For fees
        // Output requirements  
        .with_custom_cell(b"vault", CellCountConstraint::exactly(0), CellCountConstraint::exactly(1)) // Create vault
        .with_custom_cell(b"stable", CellCountConstraint::exactly(0), CellCountConstraint::at_least(0)) // May mint stablecoin
        // Network-specific configuration
        .with_network(network)
        // Enable automatic validation of known script dependencies (e.g., xUDT deps)
        .with_auto_known_script_validation()
        // Add custom CDP dependencies
        .with_required_cell_dep(cdp_oracle_dep, 0, DepType::Code) // Price oracle
        .with_required_header_dep(recent_block_hash) // Recent block for time checks
        // Add custom validator for argument content and business logic validation
        .with_custom_validator(crate::validators::validate_open_vault_transaction)
        // Optional: Add specific dependency validator
        .with_dep_validator(crate::validators::validate_transaction_dependencies);
    
    registry.register(open_vault_rules);
    
    // Add validation rules for close vault transaction
    let close_vault_rules = TransactionValidationRules::new(b"CDP.closeVault")
        .with_arguments(1) // vault_id
        .with_custom_cell(b"vault", CellCountConstraint::exactly(1), CellCountConstraint::exactly(0)) // Consume vault
        .with_custom_cell(b"stable", CellCountConstraint::at_least(1), CellCountConstraint::exactly(0)) // Burn stablecoin
        .with_known_cell(b"xudt", CellCountConstraint::exactly(0), CellCountConstraint::at_least(1)) // Return collateral
        // Add custom validator for argument content and business logic validation
        .with_custom_validator(crate::validators::validate_close_vault_transaction);
    
    registry.register(close_vault_rules);
    
    // Add validation rules for update vault transaction
    let update_vault_rules = TransactionValidationRules::new(b"CDP.updateVault")
        .with_arguments(2) // vault_id, new_collateral_amount  
        .with_custom_cell(b"vault", CellCountConstraint::exactly(1), CellCountConstraint::exactly(1)) // Update vault
        .with_known_cell(b"xudt", CellCountConstraint::any(), CellCountConstraint::any()) // May add/remove collateral
        // Add custom validator for argument content and business logic validation
        .with_custom_validator(crate::validators::validate_update_vault_transaction);
    
    registry.register(update_vault_rules);
    
    registry
}