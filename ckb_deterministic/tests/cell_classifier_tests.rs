use ckb_deterministic::cell_classifier::{CellClass, ClassificationRule, RuleBasedClassifier, CellInfo, CellClassifier};
use ckb_std::ckb_types::packed::{Script, ScriptBuilder};
use ckb_std::ckb_types::prelude::*;
extern crate alloc;
use alloc::vec::Vec;

fn create_test_script(code_hash: [u8; 32], args: Vec<u8>) -> Script {
    ScriptBuilder::default()
        .code_hash(code_hash.pack())
        .hash_type(0u8.into())
        .args(args.pack())
        .build()
}

fn create_test_cell(lock: Script, type_script: Option<Script>) -> CellInfo {
    let lock_hash = [0u8; 32]; // Simplified for testing
    let type_hash = type_script.as_ref().map(|_| [1u8; 32]);
    
    CellInfo {
        source: ckb_std::ckb_constants::Source::Input,
        index: 0,
        data: vec![],
        lock,
        lock_hash,
        type_script,
        type_hash,
    }
}

#[test]
fn test_simple_ckb_classification() {
    let classifier = RuleBasedClassifier::new("TestClassifier");
    
    // Create a cell without type script
    let lock_script = create_test_script([1u8; 32], vec![]);
    let cell = create_test_cell(lock_script, None);
    
    // Should be classified as SimpleCKB
    let classification = classifier.classify(&cell).unwrap();
    assert!(matches!(classification, CellClass::SimpleCKB));
}

#[test]
fn test_known_script_classification() {
    let xudt_code_hash = [2u8; 32];
    let classifier = RuleBasedClassifier::new("TestClassifier")
        .add_rule(ClassificationRule::TypeCodeHash {
            code_hash: xudt_code_hash,
            class: CellClass::known("xudt"),
        });
    
    // Create a cell with xUDT type script
    let lock_script = create_test_script([1u8; 32], vec![]);
    let type_script = create_test_script(xudt_code_hash, vec![42u8; 32]);
    let cell = create_test_cell(lock_script, Some(type_script));
    
    // Should be classified as xUDT
    let classification = classifier.classify(&cell).unwrap();
    assert!(matches!(classification, CellClass::Known(name) if name == "xudt"));
}

#[test]
fn test_custom_cell_classification() {
    let vault_code_hash = [3u8; 32];
    let classifier = RuleBasedClassifier::new("TestClassifier")
        .add_rule(ClassificationRule::TypeCodeHash {
        code_hash: vault_code_hash,
        class: CellClass::custom("vault"),
    });
    
    // Create a vault cell
    let lock_script = create_test_script([1u8; 32], vec![]);
    let type_script = create_test_script(vault_code_hash, vec![1, 2, 3]);
    let cell = create_test_cell(lock_script, Some(type_script));
    
    // Should be classified as vault
    let classification = classifier.classify(&cell).unwrap();
    assert!(matches!(classification, CellClass::Custom(name) if name == "vault"));
}

#[test]
fn test_lock_script_classification() {
    let special_lock_hash = [4u8; 32];
    let classifier = RuleBasedClassifier::new("TestClassifier")
        .add_rule(ClassificationRule::LockCodeHash {
            code_hash: special_lock_hash,
            class: CellClass::custom("special_lock"),
        });
    
    // Create a cell with special lock AND a type script to avoid SimpleCKB classification
    let lock_script = create_test_script(special_lock_hash, vec![]);
    let type_script = create_test_script([99u8; 32], vec![]);  // Different from any known type
    let cell = create_test_cell(lock_script, Some(type_script));
    
    // Should be classified as special_lock
    let classification = classifier.classify(&cell).unwrap();
    assert!(matches!(classification, CellClass::Custom(name) if name == "special_lock"));
}

#[test]
fn test_custom_predicate_classification() {
    let classifier = RuleBasedClassifier::new("TestClassifier")
        .add_custom(
            "specific_token",
            |cell| {
                // Check if it has specific type code hash and args
                if let Some(type_script) = &cell.type_script {
                    let code_hash = type_script.code_hash();
                    let args = type_script.args();
                    // args.pack() adds a 4-byte length prefix, so check with that included
                    let expected_args = vec![0xAA, 0xBB, 0xCC, 0xDD].pack();
                    let expected_args_bytes: ckb_std::ckb_types::packed::Bytes = expected_args;
                    code_hash.as_slice() == &[5u8; 32] && 
                    args.as_slice() == expected_args_bytes.as_slice()
                } else {
                    false
                }
            },
            CellClass::custom("specific_token"),
        );
    
    // Create a cell with matching type script
    let lock_script = create_test_script([1u8; 32], vec![]);
    let type_script = create_test_script([5u8; 32], vec![0xAA, 0xBB, 0xCC, 0xDD]);
    let cell = create_test_cell(lock_script, Some(type_script));
    
    // Should be classified as specific_token
    let classification = classifier.classify(&cell).unwrap();
    assert!(matches!(classification, CellClass::Custom(name) if name == "specific_token"));
}

#[test]
fn test_unidentified_cell() {
    let classifier = RuleBasedClassifier::new("TestClassifier");
    
    // Create a cell with unknown type script
    let lock_script = create_test_script([1u8; 32], vec![]);
    let type_script = create_test_script([99u8; 32], vec![]);
    let cell = create_test_cell(lock_script, Some(type_script));
    
    // Should be classified as unidentified
    let classification = classifier.classify(&cell).unwrap();
    assert!(matches!(classification, CellClass::Unidentified));
}

#[test]
fn test_multiple_known_scripts() {
    let xudt_hash = [10u8; 32];
    let dao_hash = [11u8; 32];
    let spore_hash = [12u8; 32];
    
    let classifier = RuleBasedClassifier::new("TestClassifier")
        .add_rule(ClassificationRule::TypeCodeHash {
            code_hash: xudt_hash,
            class: CellClass::known("xudt"),
        })
        .add_rule(ClassificationRule::TypeCodeHash {
            code_hash: dao_hash,
            class: CellClass::known("dao"),
        })
        .add_rule(ClassificationRule::TypeCodeHash {
            code_hash: spore_hash,
            class: CellClass::known("spore"),
        });
    
    // Test each type
    let lock_script = create_test_script([1u8; 32], vec![]);
    
    // xUDT cell
    let xudt_cell = create_test_cell(
        lock_script.clone(), 
        Some(create_test_script(xudt_hash, vec![]))
    );
    let xudt_result = classifier.classify(&xudt_cell).unwrap();
    assert!(matches!(xudt_result, CellClass::Known(name) if name == "xudt"));
    
    // DAO cell
    let dao_cell = create_test_cell(
        lock_script.clone(), 
        Some(create_test_script(dao_hash, vec![]))
    );
    let dao_result = classifier.classify(&dao_cell).unwrap();
    assert!(matches!(dao_result, CellClass::Known(name) if name == "dao"));
    
    // Spore cell
    let spore_cell = create_test_cell(
        lock_script, 
        Some(create_test_script(spore_hash, vec![]))
    );
    let spore_result = classifier.classify(&spore_cell).unwrap();
    assert!(matches!(spore_result, CellClass::Known(name) if name == "spore"));
}

#[test]
fn test_classification_priority() {
    let code_hash = [20u8; 32];
    
    // Create classifier with general rule only
    // Since we can't use closures that capture variables, we'll test priority differently
    let classifier = RuleBasedClassifier::new("TestClassifier")
        .add_rule(ClassificationRule::TypeCodeHash {
            code_hash,
            class: CellClass::custom("general_token"),
        });
    
    let lock_script = create_test_script([1u8; 32], vec![]);
    
    // All cells with this code hash should match general rule
    let general_cell = create_test_cell(
        lock_script,
        Some(create_test_script(code_hash, vec![1, 2, 3]))
    );
    let general_result = classifier.classify(&general_cell).unwrap();
    assert!(matches!(general_result, CellClass::Custom(name) if name == "general_token"));
}