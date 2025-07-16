#![no_std]
#![cfg_attr(not(test), no_main)]

#[cfg(test)]
extern crate alloc;

#[cfg(not(test))]
use ckb_std::default_alloc;
#[cfg(not(test))]
ckb_std::entry!(program_entry);
#[cfg(not(test))]
default_alloc!();

use ckb_std::debug;
use ckb_deterministic::{
    transaction_context::TransactionContext,
    cell_classifier::CellClassifier,
};
use deterministic_cdp_shared::{
    transaction_context::{create_cdp_transaction_context, create_cdp_validation_registry},
    transaction_recipe::*,
    Error,
};

/// Main entry point for the CDP smart contract
/// 
/// This demonstrates the typical usage pattern of ckb_deterministic:
/// 1. Create a transaction context with project-specific cell classifier
/// 2. Create a project instance with transaction handlers
/// 3. Process the transaction using the deterministic framework
pub fn program_entry() -> i8 {
    match main() {
        Ok(()) => 0,
        Err(err) => err as i8,
    }
}

fn main() -> Result<(), Error> {
    debug!("CDP Contract: Starting execution");
    
    // Step 1: Create a transaction context
    // This automatically:
    // - Loads and classifies all input/output cells
    // - Parses the transaction recipe (method path + arguments)
    // - Prepares the context for validation
    let context = create_cdp_transaction_context()?;
    
    debug!("CDP Contract: Transaction method = {}", context.method_path_name);
    debug!("CDP Contract: Input cells = {}", context.input_cells.total_cell_count());
    debug!("CDP Contract: Output cells = {}", context.output_cells.total_cell_count());
    
    // Step 2: Create the project handler
    // The CDPProject knows how to handle different transaction types:
    // - openVault: Create a new CDP vault
    // - closeVault: Close an existing vault
    // - updateVault: Update collateral in a vault
    let cdp_project = CDPProject::new();
    
    // Step 3: Process the transaction
    // This will:
    // - Apply universal validation rules from the registry
    // - Run method-specific custom validators
    // - Execute business logic for the specific method
    cdp_project.process_transaction(&context)?;
    
    debug!("CDP Contract: Execution completed successfully");
    Ok(())
}

// CDP Project that processes transactions
pub struct CDPProject {
    open_vault_hash: u64,
    close_vault_hash: u64,
    update_vault_hash: u64,
}

impl CDPProject {
    pub fn new() -> Self {
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
        
        // This validation can return different error codes to the VM:
        // - Error code 41: Wrong method path (not CDP.openVault, etc.)
        // - Error code 42: Invalid argument count (e.g., expected 2 args, got 3)
        // - Error code 43: Missing required cells (e.g., no xUDT input for collateral)
        // - Error code 44: Too many cells (e.g., multiple vault outputs when expecting 1)
        // - Error code 45: Unexpected cell type (unidentified cells in strict mode)
        // - Error code 46: Custom validation failed (e.g., collateral ratio too low)
        registry.validate(&context.recipe, &context.input_cells, &context.output_cells)
            .map_err(|e| Error::from(e))?;
        
        // Then, apply method-specific business logic
        match context.method_path_hash {
            hash if hash == self.open_vault_hash => self.open_vault(context),
            hash if hash == self.close_vault_hash => self.close_vault(context),
            hash if hash == self.update_vault_hash => self.update_vault(context),
            _ => Err(Error::WrongMethodPath), // Error code 41: Unknown method
        }
    }

    fn open_vault<C: CellClassifier>(
        &self, 
        context: &TransactionContext<C>
    ) -> Result<(), Error> {
        // Validate arguments format (business logic validation)
        validate_cdp_create_args(&context.arguments)?;
        
        // Business logic: Parse collateral and debt amounts
        let collateral_amount = u128::from_le_bytes(
            context.arguments[0][..16].try_into()
                .map_err(|_| Error::InvalidArguments)?
        );
        let _debt_amount = u128::from_le_bytes(
            context.arguments[1][..16].try_into()
                .map_err(|_| Error::InvalidArguments)?
        );
        
        // Business logic: Check minimum collateral ratio, etc.
        if collateral_amount == 0 {
            return Err(Error::InvalidArguments);
        }
        
        // Additional business logic would go here (collateral ratio checks, etc.)
        debug!("CDP Contract: Opened vault with {} collateral", collateral_amount);
        
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
        debug!("CDP Contract: Closed vault {}", vault_id);
        
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
        let _new_collateral_amount = u128::from_le_bytes(
            context.arguments[1][..16].try_into()
                .map_err(|_| Error::InvalidArguments)?
        );
        
        // Business logic: Validate the update parameters
        if vault_id == 0 {
            return Err(Error::InvalidArguments);
        }
        
        // Additional business logic would go here (collateral ratio checks, etc.)
        debug!("CDP Contract: Updated vault {}", vault_id);
        
        Ok(())
    }
}

