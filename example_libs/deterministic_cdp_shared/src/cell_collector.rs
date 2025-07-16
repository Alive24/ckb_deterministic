use ckb_deterministic::cell_classifier::{RuleBasedClassifier, CellClass, create_universal_classifier};

// CDP-specific cell type hashes (placeholders - would be actual hashes in production)
pub const VAULT_TYPE_HASH: [u8; 32] = [1u8; 32];
pub const STABLE_TYPE_HASH: [u8; 32] = [3u8; 32];
pub const POOL_TYPE_HASH: [u8; 32] = [4u8; 32];

// Known script type hashes (placeholders - would be actual network hashes in production)
pub const XUDT_CODE_HASH: [u8; 32] = [10u8; 32];  // UDT for collateral tokens
pub const SPORE_CODE_HASH: [u8; 32] = [20u8; 32]; // NFT for CDP positions

// Create a CDP-specific cell classifier using universal known scripts
pub fn create_cdp_classifier() -> RuleBasedClassifier {
    // Start with universal classifier that includes common CKB scripts
    let mut classifier = create_universal_classifier("CDPClassifier");
    
    // Add CDP-specific custom cell types
    classifier = classifier
        .add_type_hash(VAULT_TYPE_HASH, CellClass::custom(b"vault".to_vec()))
        .add_type_hash(STABLE_TYPE_HASH, CellClass::custom(b"stable".to_vec()))
        .add_type_hash(POOL_TYPE_HASH, CellClass::custom(b"pool".to_vec()));
    
    // The universal classifier already includes xUDT and Spore support,
    // so we don't need to manually add them
    
    classifier
}