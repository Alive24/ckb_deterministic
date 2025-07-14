use ckb_deterministic::{
    transaction_context::{TransactionContext, create_transaction_context},
    cell_classifier::{CellCollector, CellClassifier},
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
    liquidate_vault_hash: u64,
}

impl CDPProject {
    pub fn new() -> Self {
        use crate::transaction_recipe::*;
        
        Self {
            open_vault_hash: cdp_open_vault_hash(),
            close_vault_hash: cdp_close_vault_hash(),
            update_vault_hash: cdp_update_vault_hash(),
            liquidate_vault_hash: cdp_liquidate_vault_hash(),
        }
    }

    pub fn process_transaction<C: CellClassifier>(
        &self, 
        context: &TransactionContext<C>
    ) -> Result<(), Error> {
        match context.method_path_hash {
            hash if hash == self.open_vault_hash => self.open_vault(context),
            hash if hash == self.close_vault_hash => self.close_vault(context),
            hash if hash == self.update_vault_hash => self.update_vault(context),
            hash if hash == self.liquidate_vault_hash => self.liquidate_vault(context),
            _ => Err(Error::InvalidArguments),
        }
    }

    fn open_vault<C: CellClassifier>(
        &self, 
        context: &TransactionContext<C>
    ) -> Result<(), Error> {
        use crate::transaction_recipe::validate_cdp_create_args;
        
        // Validate arguments
        validate_cdp_create_args(&context.arguments)?;
        
        // Check that we have collateral inputs
        let collateral_key = b"collateral".to_vec();
        if !context.input_cells.custom_cells.contains_key(&collateral_key) {
            return Err(Error::InsufficientCapacity);
        }
        
        // Check that we have vault output
        let vault_key = b"vault".to_vec();
        if !context.output_cells.custom_cells.contains_key(&vault_key) {
            return Err(Error::InvalidArguments);
        }
        
        Ok(())
    }

    fn close_vault<C: CellClassifier>(
        &self, 
        context: &TransactionContext<C>
    ) -> Result<(), Error> {
        // Check that we have vault cells in inputs
        let vault_key = b"vault".to_vec();
        if !context.input_cells.custom_cells.contains_key(&vault_key) {
            return Err(Error::Unauthorized);
        }
        
        // Vault should have no debt to close
        // In real implementation, would check vault data
        
        Ok(())
    }

    fn update_vault<C: CellClassifier>(
        &self, 
        context: &TransactionContext<C>
    ) -> Result<(), Error> {
        // Check that we have vault cells in inputs
        let vault_key = b"vault".to_vec();
        if !context.input_cells.custom_cells.contains_key(&vault_key) {
            return Err(Error::Unauthorized);
        }
        
        Ok(())
    }

    fn liquidate_vault<C: CellClassifier>(
        &self, 
        context: &TransactionContext<C>
    ) -> Result<(), Error> {
        // Check liquidation conditions
        if context.arguments.len() < 2 {
            return Err(Error::InvalidArguments);
        }
        
        // Check that we have vault to liquidate
        let vault_key = b"vault".to_vec();
        if !context.input_cells.custom_cells.contains_key(&vault_key) {
            return Err(Error::InvalidArguments);
        }
        
        Ok(())
    }
}