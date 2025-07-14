use alloc::vec::Vec;
use ckb_deterministic::transaction_recipe::method_path;
use crate::Error;

// CDP-specific transaction recipe constants
pub const CDP_OPEN_VAULT_METHOD: &str = "CDP.openVault";
pub const CDP_CLOSE_VAULT_METHOD: &str = "CDP.closeVault";
pub const CDP_UPDATE_VAULT_METHOD: &str = "CDP.updateVault";
pub const CDP_LIQUIDATE_VAULT_METHOD: &str = "CDP.liquidateVault";

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

pub fn cdp_liquidate_vault_hash() -> u64 {
    method_path(CDP_LIQUIDATE_VAULT_METHOD)
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