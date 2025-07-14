use ckb_deterministic::cell_classifier::{RuleBasedClassifier, CellClass};

// CDP-specific cell type hashes (placeholders - would be actual hashes in production)
pub const VAULT_TYPE_HASH: [u8; 32] = [1u8; 32];
pub const COLLATERAL_TYPE_HASH: [u8; 32] = [2u8; 32];
pub const STABLE_TYPE_HASH: [u8; 32] = [3u8; 32];
pub const POOL_TYPE_HASH: [u8; 32] = [4u8; 32];

// Create a CDP-specific cell classifier
pub fn create_cdp_classifier() -> RuleBasedClassifier {
    RuleBasedClassifier::new("CDPClassifier")
        .add_type_hash(VAULT_TYPE_HASH, CellClass::custom("vault"))
        .add_type_hash(COLLATERAL_TYPE_HASH, CellClass::known("collateral"))
        .add_type_hash(STABLE_TYPE_HASH, CellClass::known("stable"))
        .add_type_hash(POOL_TYPE_HASH, CellClass::custom("pool"))
}