/// CDP-specific types and constants

// CDP method paths as byte strings
pub const CDP_OPEN_VAULT: &[u8] = b"openVault";
pub const CDP_CLOSE_VAULT: &[u8] = b"closeVault";
pub const CDP_ADJUST_VAULT: &[u8] = b"adjustVault";


// CDP parameters
pub const MIN_COLLATERALIZATION_RATIO: u64 = 150; // 150%
pub const LIQUIDATION_THRESHOLD: u64 = 130; // 130%

// Vault data structure (packed in cell data)
// Format: [owner_lock_hash: 32 bytes][collateral_amount: 16 bytes][debt_amount: 16 bytes]
pub const VAULT_DATA_SIZE: usize = 32 + 16 + 16;

pub fn parse_vault_data(data: &[u8]) -> Option<([u8; 32], u128, u128)> {
    if data.len() != VAULT_DATA_SIZE {
        return None;
    }
    
    let mut owner_lock_hash = [0u8; 32];
    owner_lock_hash.copy_from_slice(&data[0..32]);
    
    let collateral_amount = u128::from_le_bytes(
        data[32..48].try_into().ok()?
    );
    
    let debt_amount = u128::from_le_bytes(
        data[48..64].try_into().ok()?
    );
    
    Some((owner_lock_hash, collateral_amount, debt_amount))
}

pub fn encode_vault_data(owner_lock_hash: [u8; 32], collateral_amount: u128, debt_amount: u128) -> [u8; VAULT_DATA_SIZE] {
    let mut data = [0u8; VAULT_DATA_SIZE];
    data[0..32].copy_from_slice(&owner_lock_hash);
    data[32..48].copy_from_slice(&collateral_amount.to_le_bytes());
    data[48..64].copy_from_slice(&debt_amount.to_le_bytes());
    data
}