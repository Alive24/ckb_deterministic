use alloc::vec::Vec;
use alloc::vec;
use ckb_deterministic::{
    transaction_recipe::{method_path, create_transaction_recipe, TransactionRecipeExt},
    generated::TransactionRecipe,
};
use crate::Error;

// CDP-specific transaction recipe constants
pub const CDP_OPEN_VAULT_METHOD: &str = "CDP.openVault";
pub const CDP_CLOSE_VAULT_METHOD: &str = "CDP.closeVault";
pub const CDP_UPDATE_VAULT_METHOD: &str = "CDP.updateVault";

// Pre-computed method path hashes for efficiency
pub fn cdp_open_vault_hash() -> u64 {
    method_path(CDP_OPEN_VAULT_METHOD)
}

pub fn cdp_close_vault_hash() -> u64 {
    method_path(CDP_CLOSE_VAULT_METHOD)
}

pub fn cdp_update_vault_hash() -> u64 {
    method_path(CDP_UPDATE_VAULT_METHOD)
}


// Helper to validate CDP-specific arguments
pub fn validate_cdp_create_args(args: &[Vec<u8>]) -> Result<(), Error> {
    // Expected args: [collateral_amount, initial_debt]
    if args.len() < 2 {
        return Err(Error::InvalidArguments);
    }
    
    // Validate collateral amount (16 bytes for u128)
    if args[0].len() != 16 {
        return Err(Error::InvalidArguments);
    }
    
    // Validate initial debt (16 bytes for u128)
    if args[1].len() != 16 {
        return Err(Error::InvalidArguments);
    }
    
    Ok(())
}

// ============================================================================
// CDP Transaction Recipe Builders
// ============================================================================

/// Create a recipe for opening a new CDP vault
/// Arguments:
/// - collateral_amount: Amount of collateral to deposit (u128 as bytes)
/// - initial_debt: Amount of stablecoin to mint (u128 as bytes)
pub fn create_open_vault_recipe(
    collateral_amount: u128,
    initial_debt: u128,
) -> Result<TransactionRecipe, Error> {
    let args = vec![
        collateral_amount.to_le_bytes().to_vec(),
        initial_debt.to_le_bytes().to_vec(),
    ];
    
    create_transaction_recipe(CDP_OPEN_VAULT_METHOD, &args)
        .map_err(|_| Error::InvalidArguments)
}

/// Create a recipe for closing a CDP vault
/// Arguments:
/// - vault_id: ID of the vault to close (u64 as bytes)
pub fn create_close_vault_recipe(vault_id: u64) -> Result<TransactionRecipe, Error> {
    let args = vec![vault_id.to_le_bytes().to_vec()];
    
    create_transaction_recipe(CDP_CLOSE_VAULT_METHOD, &args)
        .map_err(|_| Error::InvalidArguments)
}

/// Create a recipe for updating a CDP vault
/// Arguments:
/// - vault_id: ID of the vault to update (u64 as bytes)
/// - new_collateral_amount: New collateral amount (u128 as bytes)
pub fn create_update_vault_recipe(
    vault_id: u64,
    new_collateral_amount: u128,
) -> Result<TransactionRecipe, Error> {
    let args = vec![
        vault_id.to_le_bytes().to_vec(),
        new_collateral_amount.to_le_bytes().to_vec(),
    ];
    
    create_transaction_recipe(CDP_UPDATE_VAULT_METHOD, &args)
        .map_err(|_| Error::InvalidArguments)
}


// ============================================================================
// Recipe Parsing and Validation Helpers
// ============================================================================

/// Parse collateral amount from recipe arguments
pub fn parse_collateral_amount(recipe: &TransactionRecipe) -> Result<u128, Error> {
    let args = recipe.arguments_vec();
    if args.is_empty() {
        return Err(Error::InvalidArguments);
    }
    
    if args[0].len() != 16 {
        return Err(Error::InvalidArguments);
    }
    
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&args[0]);
    Ok(u128::from_le_bytes(bytes))
}

/// Parse vault ID from recipe arguments
pub fn parse_vault_id(recipe: &TransactionRecipe) -> Result<u64, Error> {
    let args = recipe.arguments_vec();
    if args.is_empty() {
        return Err(Error::InvalidArguments);
    }
    
    if args[0].len() != 8 {
        return Err(Error::InvalidArguments);
    }
    
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&args[0]);
    Ok(u64::from_le_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_open_vault_recipe() {
        let collateral = 1000_000_000_000_000_000u128; // 1000 tokens
        let debt = 500_000_000_000_000_000u128; // 500 stablecoin
        
        let recipe = create_open_vault_recipe(collateral, debt).unwrap();
        
        // Verify method path
        assert_eq!(recipe.method_path_bytes(), CDP_OPEN_VAULT_METHOD.as_bytes());
        
        // Verify arguments
        let args = recipe.arguments_vec();
        assert_eq!(args.len(), 2);
        assert_eq!(args[0], collateral.to_le_bytes().to_vec());
        assert_eq!(args[1], debt.to_le_bytes().to_vec());
    }
    
    #[test]
    fn test_parse_collateral_amount() {
        let collateral = 1500_000_000_000_000_000u128;
        let recipe = create_open_vault_recipe(collateral, 100u128).unwrap();
        
        let parsed = parse_collateral_amount(&recipe).unwrap();
        assert_eq!(parsed, collateral);
    }
    
    #[test]
    fn test_create_close_vault_recipe() {
        let vault_id = 42u64;
        let recipe = create_close_vault_recipe(vault_id).unwrap();
        
        assert_eq!(recipe.method_path_bytes(), CDP_CLOSE_VAULT_METHOD.as_bytes());
        
        let parsed_id = parse_vault_id(&recipe).unwrap();
        assert_eq!(parsed_id, vault_id);
    }
}