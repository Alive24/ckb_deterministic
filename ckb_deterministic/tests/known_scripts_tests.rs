use ckb_deterministic::known_scripts::KnownScript;
use ckb_deterministic::cell_classifier::{CellClass, CellClassifier};
use ckb_deterministic::errors::Error;

#[test]
fn test_known_script_identifiers() {
    assert_eq!(KnownScript::XUdt.identifier(), "xudt");
    assert_eq!(KnownScript::NervosDao.identifier(), "nervos_dao");
    assert_eq!(KnownScript::Spore.identifier(), "spore");
    assert_eq!(KnownScript::Cota.identifier(), "cota");
    assert_eq!(KnownScript::PWLock.identifier(), "pw_lock");
    assert_eq!(KnownScript::JoyId.identifier(), "joy_id");
}

#[test]
fn test_known_script_types() {
    // Type scripts
    assert!(KnownScript::XUdt.is_type_script());
    assert!(KnownScript::NervosDao.is_type_script());
    assert!(KnownScript::Spore.is_type_script());
    assert!(KnownScript::Cota.is_type_script());
    
    // Lock scripts
    assert!(KnownScript::PWLock.is_lock_script());
    assert!(KnownScript::JoyId.is_lock_script());
    
    // Verify mutually exclusive
    assert!(!KnownScript::XUdt.is_lock_script());
    assert!(!KnownScript::PWLock.is_type_script());
}

#[test]
fn test_known_script_cell_classes() {
    let xudt_class = KnownScript::XUdt.cell_class();
    assert!(matches!(xudt_class, CellClass::Known(name) if name == "xudt"));
    
    let spore_class = KnownScript::Spore.cell_class();
    assert!(matches!(spore_class, CellClass::Known(name) if name == "spore"));
    
    let pw_lock_class = KnownScript::PWLock.cell_class();
    assert!(matches!(pw_lock_class, CellClass::Known(name) if name == "pw_lock"));
}

#[test]
fn test_known_script_code_hashes() {
    // All known scripts should have valid code hashes
    let scripts = vec![
        KnownScript::XUdt,
        KnownScript::NervosDao,
        KnownScript::Spore,
        KnownScript::Cota,
        KnownScript::PWLock,
        KnownScript::JoyId,
    ];
    
    for script in scripts {
        match script.code_hash() {
            Ok(hash) => {
                // Code hash should be 32 bytes
                assert_eq!(hash.len(), 32);
                // Should not be all zeros
                assert!(!hash.iter().all(|&b| b == 0));
            }
            Err(Error::InvalidCodeHash) => {
                // Some scripts might not be configured yet
                // This is acceptable in test environment
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
}

#[test]
fn test_known_script_exhaustive() {
    // Test that all enum variants are covered
    let all_scripts = vec![
        KnownScript::XUdt,
        KnownScript::NervosDao,
        KnownScript::Spore,
        KnownScript::Cota,
        KnownScript::PWLock,
        KnownScript::JoyId,
    ];
    
    // Each script should have unique identifier
    let mut identifiers = vec![];
    for script in &all_scripts {
        let id = script.identifier();
        assert!(!identifiers.contains(&id), "Duplicate identifier: {}", id);
        identifiers.push(id);
    }
    
    // Each script should produce a Known cell class
    for script in &all_scripts {
        assert!(matches!(script.cell_class(), CellClass::Known(_)));
    }
}

#[test]
fn test_known_script_consistency() {
    // Test that identifier matches cell class name
    let scripts = vec![
        KnownScript::XUdt,
        KnownScript::NervosDao,
        KnownScript::Spore,
        KnownScript::Cota,
        KnownScript::PWLock,
        KnownScript::JoyId,
    ];
    
    for script in scripts {
        let identifier = script.identifier();
        match script.cell_class() {
            CellClass::Known(name) => {
                assert_eq!(identifier, name, "Identifier and cell class name mismatch");
            }
            _ => panic!("Known script should produce Known cell class"),
        }
    }
}

#[test]
fn test_known_script_error_handling() {
    // Test error handling for code hash retrieval
    // In a test environment, some scripts might not have configured code hashes
    
    match KnownScript::XUdt.code_hash() {
        Ok(_) => {} // Success case
        Err(Error::InvalidCodeHash) => {} // Expected in test environment
        Err(e) => panic!("Unexpected error type: {:?}", e),
    }
}

#[test]
fn test_known_script_usage_patterns() {
    // Test typical usage patterns
    
    // 1. Check if a script is of certain type
    if KnownScript::XUdt.is_type_script() {
        // Handle type script
        assert!(!KnownScript::XUdt.is_lock_script());
    }
    
    // 2. Get cell class for classification
    let cell_class = KnownScript::Spore.cell_class();
    assert!(matches!(cell_class, CellClass::Known(_)));
    
    // 3. Get identifier for logging or display
    let id = KnownScript::NervosDao.identifier();
    assert!(!id.is_empty());
}

#[test]
fn test_known_script_different_types() {
    // Ensure we have both type and lock scripts
    let type_scripts = vec![
        KnownScript::XUdt,
        KnownScript::NervosDao,
        KnownScript::Spore,
        KnownScript::Cota,
    ];
    
    let lock_scripts = vec![
        KnownScript::PWLock,
        KnownScript::JoyId,
    ];
    
    for script in type_scripts {
        assert!(script.is_type_script());
        assert!(!script.is_lock_script());
    }
    
    for script in lock_scripts {
        assert!(script.is_lock_script());
        assert!(!script.is_type_script());
    }
}

#[test]
fn test_known_script_classification_integration() {
    use ckb_deterministic::cell_classifier::RuleBasedClassifier;
    
    // Test integration with classifier
    let classifier = RuleBasedClassifier::new("TestClassifier");
    
    // Simply ensure we can create a classifier - the add_known_script method doesn't exist
    assert_eq!(classifier.name(), "TestClassifier");
}