/// Universal known scripts for CKB
/// Provides standardized classification for common script types in the CKB ecosystem

use crate::cell_classifier::{CellClass, ClassificationRule, RuleBasedClassifier};
extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;

/// Known script types in the CKB ecosystem
/// Simplified for contract-side classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnownScript {
    /// Nervos DAO type script
    NervosDao,
    /// Secp256k1 Blake160 lock script (most common lock)
    Secp256k1Blake160,
    /// Secp256k1 Multisig lock script
    Secp256k1Multisig,
    /// Anyone Can Pay lock script
    AnyoneCanPay,
    /// JoyID lock script
    JoyId,
    /// OmniLock script
    OmniLock,
    /// Nostr lock script
    NostrLock,
    /// Type ID type script
    TypeId,
    /// xUDT (User Defined Token) type script
    XUdt,
    /// Unique type script
    UniqueType,
    /// Spore (NFT) type script
    Spore,
}

impl KnownScript {
    /// Get the identifier string for this known script
    pub fn identifier(&self) -> &'static str {
        match self {
            KnownScript::NervosDao => "nervos_dao",
            KnownScript::Secp256k1Blake160 => "secp256k1_blake160",
            KnownScript::Secp256k1Multisig => "secp256k1_multisig",
            KnownScript::AnyoneCanPay => "anyone_can_pay",
            KnownScript::JoyId => "joy_id",
            KnownScript::OmniLock => "omni_lock",
            KnownScript::NostrLock => "nostr_lock",
            KnownScript::TypeId => "type_id",
            KnownScript::XUdt => "xudt",
            KnownScript::UniqueType => "unique_type",
            KnownScript::Spore => "spore",
        }
    }
    
    /// Get the cell class for this known script
    pub fn cell_class(&self) -> CellClass {
        CellClass::known(self.identifier())
    }
    
    /// Check if this is a lock script type
    pub fn is_lock_script(&self) -> bool {
        matches!(self, 
            KnownScript::Secp256k1Blake160 |
            KnownScript::Secp256k1Multisig |
            KnownScript::AnyoneCanPay |
            KnownScript::JoyId |
            KnownScript::OmniLock |
            KnownScript::NostrLock
        )
    }
    
    /// Check if this is a type script type
    pub fn is_type_script(&self) -> bool {
        matches!(self,
            KnownScript::NervosDao |
            KnownScript::TypeId |
            KnownScript::XUdt |
            KnownScript::UniqueType |
            KnownScript::Spore
        )
    }
    
    /// Get all known script types
    pub fn all() -> Vec<KnownScript> {
        vec![
            KnownScript::NervosDao,
            KnownScript::Secp256k1Blake160,
            KnownScript::Secp256k1Multisig,
            KnownScript::AnyoneCanPay,
            KnownScript::JoyId,
            KnownScript::OmniLock,
            KnownScript::NostrLock,
            KnownScript::TypeId,
            KnownScript::XUdt,
            KnownScript::UniqueType,
            KnownScript::Spore,
        ]
    }
}

/// Helper function to create a simple CKB cell class
/// A simple CKB cell has no type script (type_script is None)
pub fn simple_ckb_cell_class() -> CellClass {
    CellClass::known("simple_ckb")
}

/// Helper function to add simple CKB cell rule to a classifier
/// Simple CKB cells are known cells identified by having no type script
/// Uses a custom predicate rule but classifies as a known cell type
pub fn add_simple_ckb_rule(classifier: RuleBasedClassifier) -> RuleBasedClassifier {
    classifier.add_custom(
        "simple_ckb",
        |cell| cell.type_script.is_none(),
        simple_ckb_cell_class()
    )
}

/// Create a classifier with all universal known scripts
/// This can be extended by projects with their own custom cell types
pub fn create_universal_classifier(name: impl Into<String>) -> RuleBasedClassifier {
    let mut classifier = RuleBasedClassifier::new(name);
    
    // Add simple CKB cell rule first (highest priority)
    classifier = add_simple_ckb_rule(classifier);
    
    // TODO: Add code hash rules for known scripts when we have access to network configuration
    // For now, this provides the structure that projects can extend
    
    classifier
}

/// Project configuration for known scripts
/// Allows projects to specify which code hashes correspond to which known scripts
pub struct KnownScriptConfig {
    pub script_type: KnownScript,
    pub code_hash: [u8; 32],
}

impl KnownScriptConfig {
    pub fn new(script_type: KnownScript, code_hash: [u8; 32]) -> Self {
        Self { script_type, code_hash }
    }
}

/// Builder for creating classifiers with known script configurations
pub struct KnownScriptClassifierBuilder {
    classifier: RuleBasedClassifier,
    configs: Vec<KnownScriptConfig>,
}

impl KnownScriptClassifierBuilder {
    /// Create a new builder with the universal classifier base
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            classifier: create_universal_classifier(name),
            configs: Vec::new(),
        }
    }
    
    /// Add a known script configuration
    pub fn add_known_script(mut self, config: KnownScriptConfig) -> Self {
        self.configs.push(config);
        self
    }
    
    /// Add a known script by type and code hash
    pub fn add_script(self, script_type: KnownScript, code_hash: [u8; 32]) -> Self {
        self.add_known_script(KnownScriptConfig::new(script_type, code_hash))
    }
    
    /// Build the classifier with all configured known scripts
    pub fn build(mut self) -> RuleBasedClassifier {
        // Add rules for all configured known scripts
        for config in self.configs {
            let rule = if config.script_type.is_type_script() {
                ClassificationRule::TypeCodeHash {
                    code_hash: config.code_hash,
                    class: config.script_type.cell_class(),
                }
            } else {
                ClassificationRule::LockCodeHash {
                    code_hash: config.code_hash,
                    class: config.script_type.cell_class(),
                }
            };
            
            self.classifier = self.classifier.add_rule(rule);
        }
        
        self.classifier
    }
}

/// Convenience function to create a classifier with xUDT support
pub fn create_xudt_classifier(name: impl Into<String>, xudt_code_hash: [u8; 32]) -> RuleBasedClassifier {
    KnownScriptClassifierBuilder::new(name)
        .add_script(KnownScript::XUdt, xudt_code_hash)
        .build()
}

/// Convenience function to create a classifier with Spore support
pub fn create_spore_classifier(name: impl Into<String>, spore_code_hash: [u8; 32]) -> RuleBasedClassifier {
    KnownScriptClassifierBuilder::new(name)
        .add_script(KnownScript::Spore, spore_code_hash)
        .build()
}

/// Convenience function to create a classifier with both xUDT and Spore support
pub fn create_xudt_spore_classifier(
    name: impl Into<String>, 
    xudt_code_hash: [u8; 32], 
    spore_code_hash: [u8; 32]
) -> RuleBasedClassifier {
    KnownScriptClassifierBuilder::new(name)
        .add_script(KnownScript::XUdt, xudt_code_hash)
        .add_script(KnownScript::Spore, spore_code_hash)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell_classifier::{CellInfo, CellClassifier};
    use ckb_std::ckb_constants::Source;
    use ckb_std::ckb_types::packed::Script;

    #[test]
    fn test_known_script_identifiers() {
        assert_eq!(KnownScript::XUdt.identifier(), "xudt");
        assert_eq!(KnownScript::Spore.identifier(), "spore");
        assert_eq!(KnownScript::Secp256k1Blake160.identifier(), "secp256k1_blake160");
        assert_eq!(KnownScript::NervosDao.identifier(), "nervos_dao");
    }
    
    #[test]
    fn test_script_type_classification() {
        assert!(KnownScript::XUdt.is_type_script());
        assert!(!KnownScript::XUdt.is_lock_script());
        
        assert!(KnownScript::Secp256k1Blake160.is_lock_script());
        assert!(!KnownScript::Secp256k1Blake160.is_type_script());
    }
    
    #[test]
    fn test_simple_ckb_cell_class() {
        let class = simple_ckb_cell_class();
        assert!(class.is_known("simple_ckb"));
    }
    
    #[test]
    fn test_universal_classifier() {
        let classifier = create_universal_classifier("test");
        
        // Test simple CKB cell (no type script)
        let simple_cell = CellInfo {
            source: Source::Input,
            index: 0,
            data: Vec::new(),
            lock: Script::default(),
            lock_hash: [0u8; 32],
            type_script: None,
            type_hash: None,
        };
        
        let result = classifier.classify(&simple_cell);
        assert!(result.is_known("simple_ckb"));
        
        // Test cell with type script (should be unidentified without additional config)
        let typed_cell = CellInfo {
            source: Source::Input,
            index: 0,
            data: Vec::new(),
            lock: Script::default(),
            lock_hash: [0u8; 32],
            type_script: Some(Script::default()),
            type_hash: Some([1u8; 32]),
        };
        
        let result2 = classifier.classify(&typed_cell);
        assert!(result2.is_unidentified());
    }
    
    #[test]
    fn test_known_script_classifier_builder() {
        let xudt_code_hash = [1u8; 32];
        let spore_code_hash = [2u8; 32];
        
        let classifier = KnownScriptClassifierBuilder::new("test")
            .add_script(KnownScript::XUdt, xudt_code_hash)
            .add_script(KnownScript::Spore, spore_code_hash)
            .build();
        
        // Test that the classifier was built successfully
        assert_eq!(classifier.name(), "test");
    }
    
    #[test]
    fn test_convenience_functions() {
        let xudt_hash = [1u8; 32];
        let spore_hash = [2u8; 32];
        
        let _xudt_classifier = create_xudt_classifier("xudt_test", xudt_hash);
        let _spore_classifier = create_spore_classifier("spore_test", spore_hash);
        let _combined_classifier = create_xudt_spore_classifier("combined_test", xudt_hash, spore_hash);
    }
    
    #[test]
    fn test_all_known_scripts() {
        let all_scripts = KnownScript::all();
        assert_eq!(all_scripts.len(), 11);
        assert!(all_scripts.contains(&KnownScript::XUdt));
        assert!(all_scripts.contains(&KnownScript::Spore));
        assert!(all_scripts.contains(&KnownScript::Secp256k1Blake160));
    }
}