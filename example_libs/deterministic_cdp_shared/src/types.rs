use alloc::vec::Vec;

// CDP-specific types similar to CKBoost's types module

#[derive(Debug, Clone)]
pub struct VaultData {
    pub vault_id: [u8; 32],
    pub owner: [u8; 32],
    pub collateral_amount: u128,
    pub debt_amount: u128,
    pub last_updated: u64,
    pub status: VaultStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultStatus {
    Active = 0,
    UnderCollateralized = 1,
    Liquidated = 2,
    Closed = 3,
}

#[derive(Debug, Clone)]
pub struct PoolData {
    pub total_collateral: u128,
    pub total_debt: u128,
    pub collateral_price: u128,
    pub liquidation_ratio: u64,
}

#[derive(Debug, Clone)]
pub struct CDPConfig {
    pub min_collateral_ratio: u64,  // e.g., 150 for 150%
    pub liquidation_ratio: u64,      // e.g., 130 for 130%
    pub stability_fee: u64,          // Annual fee in basis points
    pub liquidation_penalty: u64,    // Penalty in basis points
}

impl Default for CDPConfig {
    fn default() -> Self {
        Self {
            min_collateral_ratio: 150,
            liquidation_ratio: 130,
            stability_fee: 200,  // 2%
            liquidation_penalty: 1300,  // 13%
        }
    }
}

// Helper functions
pub fn encode_vault_data(data: &VaultData) -> Vec<u8> {
    let mut result = Vec::with_capacity(113); // 32 + 32 + 16 + 16 + 8 + 1
    result.extend_from_slice(&data.vault_id);
    result.extend_from_slice(&data.owner);
    result.extend_from_slice(&data.collateral_amount.to_le_bytes());
    result.extend_from_slice(&data.debt_amount.to_le_bytes());
    result.extend_from_slice(&data.last_updated.to_le_bytes());
    result.push(data.status as u8);
    result
}

pub fn decode_vault_data(data: &[u8]) -> Result<VaultData, crate::Error> {
    if data.len() < 113 {
        return Err(crate::Error::InvalidArguments);
    }
    
    let mut vault_id = [0u8; 32];
    vault_id.copy_from_slice(&data[0..32]);
    
    let mut owner = [0u8; 32];
    owner.copy_from_slice(&data[32..64]);
    
    let collateral_amount = u128::from_le_bytes(data[64..80].try_into().unwrap());
    let debt_amount = u128::from_le_bytes(data[80..96].try_into().unwrap());
    let last_updated = u64::from_le_bytes(data[96..104].try_into().unwrap());
    
    let status = match data[104] {
        0 => VaultStatus::Active,
        1 => VaultStatus::UnderCollateralized,
        2 => VaultStatus::Liquidated,
        3 => VaultStatus::Closed,
        _ => return Err(crate::Error::InvalidArguments),
    };
    
    Ok(VaultData {
        vault_id,
        owner,
        collateral_amount,
        debt_amount,
        last_updated,
        status,
    })
}