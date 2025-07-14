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
        .with_custom_cell(b"stable", CellCountConstraint::exactly(0), CellCountConstraint::at_least(0)); // May mint stablecoin
    
    registry.register(open_vault_rules);
    
    // Add validation rules for close vault transaction
    let close_vault_rules = TransactionValidationRules::new(b"CDP.closeVault")
        .with_arguments(1) // vault_id
        .with_custom_cell(b"vault", CellCountConstraint::exactly(1), CellCountConstraint::exactly(0)) // Consume vault
        .with_custom_cell(b"stable", CellCountConstraint::at_least(1), CellCountConstraint::exactly(0)) // Burn stablecoin
        .with_known_cell(b"xudt", CellCountConstraint::exactly(0), CellCountConstraint::at_least(1)); // Return collateral
    
    registry.register(close_vault_rules);
    
    // Add validation rules for update vault transaction
    let update_vault_rules = TransactionValidationRules::new(b"CDP.updateVault")
        .with_arguments(2) // vault_id, new_collateral_amount  
        .with_custom_cell(b"vault", CellCountConstraint::exactly(1), CellCountConstraint::exactly(1)) // Update vault
        .with_known_cell(b"xudt", CellCountConstraint::any(), CellCountConstraint::any()); // May add/remove collateral
    
    registry.register(update_vault_rules);
    
    
    registry
}