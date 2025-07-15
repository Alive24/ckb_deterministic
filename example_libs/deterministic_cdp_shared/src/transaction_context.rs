use alloc::{format, string::{String, ToString}, vec::Vec};
use ckb_deterministic::{
    transaction_context::{TransactionContext, create_transaction_context},
    cell_classifier::{CellCollector, CellClassifier},
    validation::{TransactionValidationRules, ValidationRegistry, CellCountConstraint},
};
use crate::{Error, cell_collector::create_cdp_classifier};

// CDP-specific transaction context creation
pub fn create_cdp_transaction_context() -> Result<TransactionContext<impl CellClassifier>, Error> {
    let classifier = create_cdp_classifier();
    let collector = CellCollector::new(classifier).with_strict_mode(false);
    
    create_transaction_context(collector)
        .map_err(|_| Error::InvalidArguments)
}

// CDP Project that processes transactions
pub struct CDPProject {
    open_vault_hash: u64,
    close_vault_hash: u64,
    update_vault_hash: u64,
}

impl CDPProject {
    pub fn new() -> Self {
        use crate::transaction_recipe::*;
        
        Self {
            open_vault_hash: cdp_open_vault_hash(),
            close_vault_hash: cdp_close_vault_hash(),
            update_vault_hash: cdp_update_vault_hash(),
        }
    }

    pub fn process_transaction<C: CellClassifier>(
        &self, 
        context: &TransactionContext<C>
    ) -> Result<(), Error> {
        // First, apply universal validation using the validation registry
        let registry = create_cdp_validation_registry();
        registry.validate(&context.recipe, &context.input_cells, &context.output_cells)
            .map_err(|_| Error::InvalidArguments)?;
        
        // Then, apply method-specific business logic
        match context.method_path_hash {
            hash if hash == self.open_vault_hash => self.open_vault(context),
            hash if hash == self.close_vault_hash => self.close_vault(context),
            hash if hash == self.update_vault_hash => self.update_vault(context),
            _ => Err(Error::InvalidArguments),
        }
    }

    fn open_vault<C: CellClassifier>(
        &self, 
        context: &TransactionContext<C>
    ) -> Result<(), Error> {
        use crate::transaction_recipe::validate_cdp_create_args;
        
        // Validate arguments format (business logic validation)
        validate_cdp_create_args(&context.arguments)?;
        
        // Business logic: Parse collateral and debt amounts
        let collateral_amount = u128::from_le_bytes(
            context.arguments[0][..16].try_into()
                .map_err(|_| Error::InvalidArguments)?
        );
        let debt_amount = u128::from_le_bytes(
            context.arguments[1][..16].try_into()
                .map_err(|_| Error::InvalidArguments)?
        );
        
        // Business logic: Check minimum collateral ratio, etc.
        if collateral_amount == 0 {
            return Err(Error::InvalidArguments);
        }
        
        // Additional business logic would go here (collateral ratio checks, etc.)
        
        Ok(())
    }

    fn close_vault<C: CellClassifier>(
        &self, 
        context: &TransactionContext<C>
    ) -> Result<(), Error> {
        // Business logic: Parse vault ID from arguments
        let vault_id = u64::from_le_bytes(
            context.arguments[0][..8].try_into()
                .map_err(|_| Error::InvalidArguments)?
        );
        
        // Business logic: Verify vault has no outstanding debt
        // In real implementation, would parse vault cell data and check debt amount
        // For now, just validate the vault ID is reasonable
        if vault_id == 0 {
            return Err(Error::InvalidArguments);
        }
        
        // Additional business logic would go here (debt verification, etc.)
        
        Ok(())
    }

    fn update_vault<C: CellClassifier>(
        &self, 
        context: &TransactionContext<C>
    ) -> Result<(), Error> {
        // Business logic: Parse vault ID and new collateral amount from arguments
        let vault_id = u64::from_le_bytes(
            context.arguments[0][..8].try_into()
                .map_err(|_| Error::InvalidArguments)?
        );
        let new_collateral_amount = u128::from_le_bytes(
            context.arguments[1][..16].try_into()
                .map_err(|_| Error::InvalidArguments)?
        );
        
        // Business logic: Validate the update parameters
        if vault_id == 0 {
            return Err(Error::InvalidArguments);
        }
        
        // Additional business logic would go here (collateral ratio checks, etc.)
        
        Ok(())
    }

}

/// Create a validation registry for CDP transactions
pub fn create_cdp_validation_registry() -> ValidationRegistry {
    let mut registry = ValidationRegistry::new();
    
    // Add validation rules for open vault transaction
    let open_vault_rules = TransactionValidationRules::new(b"CDP.openVault")
        .with_arguments(2) // collateral_amount, debt_amount
        // Input requirements
        .with_known_cell(b"xudt", CellCountConstraint::at_least(1), CellCountConstraint::any()) // Collateral tokens
        .with_known_cell(b"simple_ckb", CellCountConstraint::at_least(1), CellCountConstraint::any()) // For fees
        // Output requirements  
        .with_custom_cell(b"vault", CellCountConstraint::exactly(0), CellCountConstraint::exactly(1)) // Create vault
        .with_custom_cell(b"stable", CellCountConstraint::exactly(0), CellCountConstraint::at_least(0)) // May mint stablecoin
        // Add custom validator for argument content and business logic validation
        .with_custom_validator(validate_open_vault_transaction);
    
    registry.register(open_vault_rules);
    
    // Add validation rules for close vault transaction
    let close_vault_rules = TransactionValidationRules::new(b"CDP.closeVault")
        .with_arguments(1) // vault_id
        .with_custom_cell(b"vault", CellCountConstraint::exactly(1), CellCountConstraint::exactly(0)) // Consume vault
        .with_custom_cell(b"stable", CellCountConstraint::at_least(1), CellCountConstraint::exactly(0)) // Burn stablecoin
        .with_known_cell(b"xudt", CellCountConstraint::exactly(0), CellCountConstraint::at_least(1)) // Return collateral
        // Add custom validator for argument content and business logic validation
        .with_custom_validator(validate_close_vault_transaction);
    
    registry.register(close_vault_rules);
    
    // Add validation rules for update vault transaction
    let update_vault_rules = TransactionValidationRules::new(b"CDP.updateVault")
        .with_arguments(2) // vault_id, new_collateral_amount  
        .with_custom_cell(b"vault", CellCountConstraint::exactly(1), CellCountConstraint::exactly(1)) // Update vault
        .with_known_cell(b"xudt", CellCountConstraint::any(), CellCountConstraint::any()) // May add/remove collateral
        // Add custom validator for argument content and business logic validation
        .with_custom_validator(validate_update_vault_transaction);
    
    registry.register(update_vault_rules);
    
    registry
}

// ============================================================================
// Custom Validation Functions
// ============================================================================

/// Custom validator for open vault transactions
/// Validates argument content and business logic beyond basic structural checks
fn validate_open_vault_transaction(
    recipe: &ckb_deterministic::generated::TransactionRecipe,
    _input_cells: &ckb_deterministic::cell_classifier::ClassifiedCells,
    _output_cells: &ckb_deterministic::cell_classifier::ClassifiedCells,
) -> Result<(), String> {
    use ckb_deterministic::transaction_recipe::TransactionRecipeExt;
    
    // Validate argument format and content
    let args = recipe.arguments_vec();
    if args.len() != 2 {
        return Err(format!("Expected 2 arguments, got {}", args.len()));
    }
    
    // Validate collateral amount format (must be 16-byte u128)
    if args[0].len() != 16 {
        return Err("Collateral amount must be 16-byte u128".to_string());
    }
    
    // Validate debt amount format (must be 16-byte u128)
    if args[1].len() != 16 {
        return Err("Debt amount must be 16-byte u128".to_string());
    }
    
    // Parse and validate argument values
    let collateral_amount = u128::from_le_bytes(
        args[0][..16].try_into()
            .map_err(|_| "Invalid collateral amount format".to_string())?
    );
    
    let debt_amount = u128::from_le_bytes(
        args[1][..16].try_into()
            .map_err(|_| "Invalid debt amount format".to_string())?
    );
    
    // Business logic validation
    if collateral_amount == 0 {
        return Err("Collateral amount must be greater than zero".to_string());
    }
    
    // Minimum collateral ratio check (150% = debt * 3 / 2 <= collateral)
    // This prevents creating under-collateralized vaults
    if debt_amount > 0 && (debt_amount * 3 / 2) > collateral_amount {
        return Err("Insufficient collateral ratio (minimum 150%)".to_string());
    }
    
    // Validate lock script arguments for xUDT inputs
    if let Some(xudt_cells) = _input_cells.get_known("xudt") {
        for cell in xudt_cells {
            let lock_args_data = cell.lock.args().raw_data();
            let lock_args = lock_args_data.as_ref();
            
            // Validate lock args length (should be 20 bytes for secp256k1 pubkey hash)
            if lock_args.len() != 20 {
                return Err(format!(
                    "Invalid lock args length for xUDT input: expected 20 bytes, got {}", 
                    lock_args.len()
                ));
            }
            
            // Validate lock args are not all zeros (invalid pubkey hash)
            if lock_args.iter().all(|&b| b == 0) {
                return Err("xUDT input lock args cannot be all zeros".to_string());
            }
        }
    }
    
    // Validate type script arguments for xUDT consistency
    if let Some(xudt_cells) = _input_cells.get_known("xudt") {
        let mut expected_udt_type_id: Option<Vec<u8>> = None;
        
        for cell in xudt_cells {
            if let Some(type_script) = &cell.type_script {
                let type_args_data = type_script.args().raw_data();
                let type_args = type_args_data.as_ref().to_vec();
                
                // xUDT type args format: [owner_lock_hash:32][unique_id:variable]
                if type_args.len() < 32 {
                    return Err("xUDT type args must be at least 32 bytes".to_string());
                }
                
                // Validate owner lock hash is not zero
                if type_args[0..32].iter().all(|&b| b == 0) {
                    return Err("xUDT owner lock hash cannot be zero".to_string());
                }
                
                // Ensure all xUDT cells have the same type ID
                if let Some(ref expected) = expected_udt_type_id {
                    if &type_args != expected {
                        return Err("All xUDT inputs must have the same type ID".to_string());
                    }
                } else {
                    expected_udt_type_id = Some(type_args);
                }
            } else {
                return Err("xUDT cell must have type script".to_string());
            }
        }
    }
    
    Ok(())
}

/// Custom validator for close vault transactions  
/// Validates argument content and ensures proper vault closure
fn validate_close_vault_transaction(
    recipe: &ckb_deterministic::generated::TransactionRecipe,
    _input_cells: &ckb_deterministic::cell_classifier::ClassifiedCells,
    _output_cells: &ckb_deterministic::cell_classifier::ClassifiedCells,
) -> Result<(), String> {
    use ckb_deterministic::transaction_recipe::TransactionRecipeExt;
    
    // Validate argument format
    let args = recipe.arguments_vec();
    if args.len() != 1 {
        return Err(format!("Expected 1 argument, got {}", args.len()));
    }
    
    // Validate vault ID format (must be 8-byte u64)
    if args[0].len() != 8 {
        return Err("Vault ID must be 8-byte u64".to_string());
    }
    
    let vault_id = u64::from_le_bytes(
        args[0][..8].try_into()
            .map_err(|_| "Invalid vault ID format".to_string())?
    );
    
    // Business logic validation
    if vault_id == 0 {
        return Err("Vault ID must be greater than zero".to_string());
    }
    
    // Validate vault cell type script arguments
    if let Some(vault_cells) = _input_cells.get_custom(b"vault") {
        for cell in vault_cells {
            if let Some(type_script) = &cell.type_script {
                let type_args_data = type_script.args().raw_data();
                let type_args = type_args_data.as_ref();
                
                // Validate vault type args are present and reasonable length
                if type_args.is_empty() {
                    return Err("Vault type script must have arguments".to_string());
                }
                
                // Could validate specific vault type arg format here
                // For example, if vault args contain vault ID or protocol parameters
            } else {
                return Err("Vault cell must have type script".to_string());
            }
        }
    }
    
    // Validate stable token type script arguments
    if let Some(stable_cells) = _input_cells.get_custom(b"stable") {
        for cell in stable_cells {
            if let Some(type_script) = &cell.type_script {
                let type_args_data = type_script.args().raw_data();
                let type_args = type_args_data.as_ref();
                
                // Validate stable token args format (similar to xUDT)
                if type_args.len() < 32 {
                    return Err("Stable token type args must be at least 32 bytes".to_string());
                }
                
                // Validate stable token owner/issuer lock hash
                if type_args[0..32].iter().all(|&b| b == 0) {
                    return Err("Stable token issuer lock hash cannot be zero".to_string());
                }
            } else {
                return Err("Stable token cell must have type script".to_string());
            }
        }
    }
    
    // Validate output xUDT lock script arguments (returned collateral)
    if let Some(xudt_outputs) = _output_cells.get_known("xudt") {
        for cell in xudt_outputs {
            let lock_args_data = cell.lock.args().raw_data();
            let lock_args = lock_args_data.as_ref();
            
            // Validate lock args length for returned collateral
            if lock_args.len() != 20 {
                return Err(format!(
                    "Invalid lock args length for xUDT output: expected 20 bytes, got {}",
                    lock_args.len()
                ));
            }
            
            // Validate lock args are not all zeros
            if lock_args.iter().all(|&b| b == 0) {
                return Err("xUDT output lock args cannot be all zeros".to_string());
            }
        }
    }
    
    Ok(())
}

/// Custom validator for update vault transactions
/// Validates argument content and ensures valid vault updates  
fn validate_update_vault_transaction(
    recipe: &ckb_deterministic::generated::TransactionRecipe,
    _input_cells: &ckb_deterministic::cell_classifier::ClassifiedCells,
    _output_cells: &ckb_deterministic::cell_classifier::ClassifiedCells,
) -> Result<(), String> {
    use ckb_deterministic::transaction_recipe::TransactionRecipeExt;
    
    // Validate argument format
    let args = recipe.arguments_vec();
    if args.len() != 2 {
        return Err(format!("Expected 2 arguments, got {}", args.len()));
    }
    
    // Validate vault ID format (must be 8-byte u64)
    if args[0].len() != 8 {
        return Err("Vault ID must be 8-byte u64".to_string());
    }
    
    // Validate new collateral amount format (must be 16-byte u128)
    if args[1].len() != 16 {
        return Err("New collateral amount must be 16-byte u128".to_string());
    }
    
    let vault_id = u64::from_le_bytes(
        args[0][..8].try_into()
            .map_err(|_| "Invalid vault ID format".to_string())?
    );
    
    let new_collateral_amount = u128::from_le_bytes(
        args[1][..16].try_into()
            .map_err(|_| "Invalid new collateral amount format".to_string())?
    );
    
    // Business logic validation
    if vault_id == 0 {
        return Err("Vault ID must be greater than zero".to_string());
    }
    
    if new_collateral_amount == 0 {
        return Err("New collateral amount must be greater than zero".to_string());
    }
    
    // Validate vault cell type script arguments consistency between input and output
    let input_vault_type_args = if let Some(vault_inputs) = _input_cells.get_custom(b"vault") {
        let vault_cell = vault_inputs.first()
            .ok_or("At least one vault input required")?;
        
        if let Some(type_script) = &vault_cell.type_script {
            let type_args = type_script.args().raw_data().as_ref().to_vec();
            
            if type_args.is_empty() {
                return Err("Input vault type script must have arguments".to_string());
            }
            
            type_args
        } else {
            return Err("Input vault cell must have type script".to_string());
        }
    } else {
        return Err("Vault input cells not found".to_string());
    };
    
    // Validate output vault has same type args (vault ID consistency)
    if let Some(vault_outputs) = _output_cells.get_custom(b"vault") {
        let vault_cell = vault_outputs.first()
            .ok_or("At least one vault output required")?;
        
        if let Some(type_script) = &vault_cell.type_script {
            let type_args_data = type_script.args().raw_data();
            let type_args = type_args_data.as_ref();
            
            if type_args != input_vault_type_args.as_slice() {
                return Err("Vault type args must remain consistent between input and output".to_string());
            }
        } else {
            return Err("Output vault cell must have type script".to_string());
        }
    }
    
    // Validate xUDT lock script arguments if collateral is being added/removed
    if let Some(xudt_inputs) = _input_cells.get_known("xudt") {
        for cell in xudt_inputs {
            let lock_args_data = cell.lock.args().raw_data();
            let lock_args = lock_args_data.as_ref();
            
            if lock_args.len() != 20 {
                return Err(format!(
                    "Invalid xUDT input lock args length: expected 20 bytes, got {}",
                    lock_args.len()
                ));
            }
            
            if lock_args.iter().all(|&b| b == 0) {
                return Err("xUDT input lock args cannot be all zeros".to_string());
            }
        }
    }
    
    if let Some(xudt_outputs) = _output_cells.get_known("xudt") {
        for cell in xudt_outputs {
            let lock_args_data = cell.lock.args().raw_data();
            let lock_args = lock_args_data.as_ref();
            
            if lock_args.len() != 20 {
                return Err(format!(
                    "Invalid xUDT output lock args length: expected 20 bytes, got {}",
                    lock_args.len()
                ));
            }
            
            if lock_args.iter().all(|&b| b == 0) {
                return Err("xUDT output lock args cannot be all zeros".to_string());
            }
        }
    }
    
    Ok(())
}