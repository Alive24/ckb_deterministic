//! CDP-specific types and constants.
//!
//! This module defines the core data structures and constants used throughout
//! the CDP protocol, including vault data encoding, method paths, and protocol parameters.
//!
//! # Vault Data Structure
//!
//! Vaults store their state in cell data with the following layout:
//! - Bytes 0-31: Owner lock hash (identifies the vault owner)
//! - Bytes 32-47: Collateral amount (u128, little-endian)
//! - Bytes 48-63: Debt amount (u128, little-endian)
//!
//! # Protocol Parameters
//!
//! - Minimum Collateralization Ratio: 150%
//! - Liquidation Threshold: 130%

/// CDP method paths as byte strings for transaction recipes.
pub const CDP_OPEN_VAULT: &[u8] = b"openVault";
pub const CDP_CLOSE_VAULT: &[u8] = b"closeVault";
pub const CDP_ADJUST_VAULT: &[u8] = b"adjustVault";

/// CDP protocol parameters for risk management.
pub const MIN_COLLATERALIZATION_RATIO: u64 = 150; // 150%
pub const LIQUIDATION_THRESHOLD: u64 = 130; // 130%

/// Size of vault data in bytes.
/// Format: [owner_lock_hash: 32 bytes][collateral_amount: 16 bytes][debt_amount: 16 bytes]
pub const VAULT_DATA_SIZE: usize = 32 + 16 + 16;

/// Parse vault data from raw bytes.
///
/// # Parameters
///
/// - `data`: Raw cell data containing vault information
///
/// # Returns
///
/// Returns `Some((owner_lock_hash, collateral_amount, debt_amount))` if parsing succeeds,
/// or `None` if the data is invalid.
pub fn parse_vault_data(data: &[u8]) -> Option<([u8; 32], u128, u128)> {
    if data.len() != VAULT_DATA_SIZE {
        return None;
    }

    let mut owner_lock_hash = [0u8; 32];
    owner_lock_hash.copy_from_slice(&data[0..32]);

    let collateral_amount = u128::from_le_bytes(data[32..48].try_into().ok()?);

    let debt_amount = u128::from_le_bytes(data[48..64].try_into().ok()?);

    Some((owner_lock_hash, collateral_amount, debt_amount))
}

/// Encode vault data into raw bytes.
///
/// # Parameters
///
/// - `owner_lock_hash`: 32-byte hash identifying the vault owner
/// - `collateral_amount`: Amount of collateral tokens in the vault
/// - `debt_amount`: Amount of debt tokens owed by the vault
///
/// # Returns
///
/// A 64-byte array containing the encoded vault data.
pub fn encode_vault_data(
    owner_lock_hash: [u8; 32],
    collateral_amount: u128,
    debt_amount: u128,
) -> [u8; VAULT_DATA_SIZE] {
    let mut data = [0u8; VAULT_DATA_SIZE];
    data[0..32].copy_from_slice(&owner_lock_hash);
    data[32..48].copy_from_slice(&collateral_amount.to_le_bytes());
    data[48..64].copy_from_slice(&debt_amount.to_le_bytes());
    data
}
