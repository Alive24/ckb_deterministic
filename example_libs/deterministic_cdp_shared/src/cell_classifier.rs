use crate::Error;
/// CDP-specific cell classification
use ckb_deterministic::cell_classifier::{
    CellClass, ClassificationRule, RuleBasedClassifier,
};
use ckb_deterministic::known_scripts::{KnownScript, Network};
extern crate alloc;

// Custom cell type identifiers
pub const CDP_VAULT_CELL: &str = "cdp_vault";
pub const CDP_PROTOCOL_CELL: &str = "cdp_protocol";
pub const CDP_POOL_CELL: &str = "cdp_pool";

/// Create a CDP-specific cell classifier
pub fn create_cdp_classifier(
    vault_code_hash: [u8; 32],
    protocol_code_hash: [u8; 32],
    pool_code_hash: [u8; 32],
    _network: Network,
) -> Result<RuleBasedClassifier, Error> {
    let classifier = RuleBasedClassifier::new("CDPClassifier")
        // Custom CDP cells - using code hashes from protocol
        .add_rule(ClassificationRule::TypeCodeHash {
            code_hash: vault_code_hash,
            class: CellClass::custom(CDP_VAULT_CELL),
        })
        .add_rule(ClassificationRule::TypeCodeHash {
            code_hash: protocol_code_hash,
            class: CellClass::custom(CDP_PROTOCOL_CELL),
        })
        .add_rule(ClassificationRule::TypeCodeHash {
            code_hash: pool_code_hash,
            class: CellClass::custom(CDP_POOL_CELL),
        })
        // Known scripts - using KnownScript rule for xUDT
        .add_known_script(KnownScript::XUdt, KnownScript::XUdt.cell_class());

    Ok(classifier)
}
