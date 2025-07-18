#[cfg(test)]
mod tests {
    use ckb_deterministic::{
        cell_classifier::{CellClass, ClassificationRule, RuleBasedClassifier, CellInfo, CellClassifier, ClassifiedCells},
        transaction_context::TransactionContext,
        validation::{TransactionValidationRules, CellCountConstraint},
        errors::Error,
        generated::{TransactionRecipeBuilder, BytesVecBuilder, Bytes},
    };
    use ckb_std::{
        ckb_types::{
            packed::{Script, ScriptBuilder},
            prelude::*,
        },
        ckb_constants::Source,
    };
    use deterministic_cdp_shared::{
        cdp_types::{CDP_OPEN_VAULT, CDP_CLOSE_VAULT, CDP_ADJUST_VAULT},
    };

    // Helper functions
    fn create_test_script(code_hash: [u8; 32], args: Vec<u8>) -> Script {
        ScriptBuilder::default()
            .code_hash(code_hash.pack())
            .hash_type(0u8.into())
            .args(args.pack())
            .build()
    }

    fn create_simple_ckb_cell(index: usize) -> CellInfo {
        let lock_script = create_test_script([1u8; 32], vec![index as u8]);
        CellInfo {
            source: Source::Input,
            index,
            data: vec![],
            lock: lock_script.clone(),
            lock_hash: [index as u8; 32],
            type_script: None,
            type_hash: None,
        }
    }

    fn create_xudt_cell(index: usize, amount: u128) -> CellInfo {
        let lock_script = create_test_script([1u8; 32], vec![index as u8]);
        let type_script = create_test_script([10u8; 32], vec![1, 2, 3, 4]); // xUDT owner lock hash
        
        // xUDT amount as little-endian bytes
        let mut data = amount.to_le_bytes().to_vec();
        data.extend_from_slice(&[0u8; 16]); // xUDT extension data
        
        CellInfo {
            source: Source::Input,
            index,
            data,
            lock: lock_script.clone(),
            lock_hash: [index as u8; 32],
            type_script: Some(type_script),
            type_hash: Some([10u8; 32]),
        }
    }

    fn create_vault_cell(index: usize, collateral: u128, debt: u128) -> CellInfo {
        let lock_script = create_test_script([1u8; 32], vec![index as u8]);
        let type_script = create_test_script([20u8; 32], vec![5, 6, 7, 8]); // Vault type
        
        // Vault data: collateral + debt
        let mut data = collateral.to_le_bytes().to_vec();
        data.extend_from_slice(&debt.to_le_bytes());
        
        CellInfo {
            source: Source::Input,
            index,
            data,
            lock: lock_script.clone(),
            lock_hash: [index as u8; 32],
            type_script: Some(type_script),
            type_hash: Some([20u8; 32]),
        }
    }

    fn create_mock_transaction_context(
        method_path: &[u8],
        input_cells: Vec<CellInfo>,
        output_cells: Vec<CellInfo>,
    ) -> Result<TransactionContext<RuleBasedClassifier>, Error> {
        // Create a classifier
        let classifier = RuleBasedClassifier::new("TestClassifier")
            .add_rule(ClassificationRule::TypeCodeHash {
                code_hash: [10u8; 32],
                class: CellClass::known("xudt"),
            })
            .add_rule(ClassificationRule::TypeCodeHash {
                code_hash: [20u8; 32],
                class: CellClass::custom("vault"),
            });

        // Mock the transaction recipe
        let recipe = TransactionRecipeBuilder::default()
            .method_path(Bytes::from(method_path.to_vec()))
            .arguments(BytesVecBuilder::default().build())
            .build();

        // Manually create classified cells
        let mut input_simple_ckb = vec![];
        let mut input_known = std::collections::BTreeMap::new();
        let mut input_custom = std::collections::BTreeMap::new();
        
        for cell in input_cells {
            match classifier.classify(&cell).unwrap() {
                CellClass::SimpleCKB => input_simple_ckb.push(cell),
                CellClass::Known(name) => {
                    input_known.entry(name).or_insert_with(Vec::new).push(cell);
                }
                CellClass::Custom(name) => {
                    input_custom.entry(name).or_insert_with(Vec::new).push(cell);
                }
                CellClass::Unidentified => {}
            }
        }

        let mut output_simple_ckb = vec![];
        let mut output_known = std::collections::BTreeMap::new();
        let mut output_custom = std::collections::BTreeMap::new();
        
        for cell in output_cells {
            match classifier.classify(&cell).unwrap() {
                CellClass::SimpleCKB => output_simple_ckb.push(cell),
                CellClass::Known(name) => {
                    output_known.entry(name).or_insert_with(Vec::new).push(cell);
                }
                CellClass::Custom(name) => {
                    output_custom.entry(name).or_insert_with(Vec::new).push(cell);
                }
                CellClass::Unidentified => {}
            }
        }

        let input_cells = ClassifiedCells {
            simple_ckb_cells: input_simple_ckb,
            known_cells: input_known,
            custom_cells: input_custom,
            unidentified_cells: vec![],
        };
        
        let output_cells = ClassifiedCells {
            simple_ckb_cells: output_simple_ckb,
            known_cells: output_known,
            custom_cells: output_custom,
            unidentified_cells: vec![],
        };
        
        Ok(TransactionContext::from_parts(
            recipe,
            input_cells,
            output_cells,
            vec![],
            vec![],
        ))
    }

    #[test]
    fn test_open_vault_validation() {
        // Import the open_vault module - need to check if recipes is exposed
        // Let's just create the rules inline for now
        
        // Create a valid openVault transaction context
        let input_cells = vec![
            create_simple_ckb_cell(0), // CKB for fees
            create_xudt_cell(1, 1000 * 10u128.pow(8)), // 1000 xUDT as collateral
        ];
        
        let output_cells = vec![
            create_vault_cell(0, 1000 * 10u128.pow(8), 500 * 10u128.pow(8)), // New vault
            create_simple_ckb_cell(1), // Change
        ];
        
        let context = create_mock_transaction_context(
            CDP_OPEN_VAULT,
            input_cells,
            output_cells,
        ).unwrap();
        
        // Validation should pass
        let rules = TransactionValidationRules::new(CDP_OPEN_VAULT.to_vec())
            .with_custom_cell(
                "vault",
                CellCountConstraint::exactly(0),  // No vault in inputs
                CellCountConstraint::exactly(1),  // One vault in outputs
            );
        assert!(rules.validate(&context).is_ok());
    }

    #[test]
    fn test_open_vault_wrong_method_path() {
        // Import the open_vault module - need to check if recipes is exposed
        // Let's just create the rules inline for now
        
        let input_cells = vec![create_simple_ckb_cell(0)];
        let output_cells = vec![create_vault_cell(0, 1000, 500)];
        
        // Use wrong method path
        let context = create_mock_transaction_context(
            b"wrongMethod",
            input_cells,
            output_cells,
        ).unwrap();
        
        let rules = TransactionValidationRules::new(CDP_OPEN_VAULT.to_vec())
            .with_custom_cell(
                "vault",
                CellCountConstraint::exactly(0),
                CellCountConstraint::exactly(1),
            );
        assert_eq!(rules.validate(&context), Err(Error::WrongMethodPath));
    }

    #[test]
    fn test_close_vault_validation() {
        // Import the open_vault module - need to check if recipes is exposed
        // Let's just create the rules inline for now
        
        // Create a valid closeVault transaction
        let input_cells = vec![
            create_vault_cell(0, 1000 * 10u128.pow(8), 0), // Vault with no debt
            create_simple_ckb_cell(1), // CKB for fees
        ];
        
        let output_cells = vec![
            create_xudt_cell(0, 1000 * 10u128.pow(8)), // Collateral returned
            create_simple_ckb_cell(1), // Change
        ];
        
        let context = create_mock_transaction_context(
            CDP_CLOSE_VAULT,
            input_cells,
            output_cells,
        ).unwrap();
        
        let rules = TransactionValidationRules::new(CDP_CLOSE_VAULT.to_vec())
            .with_custom_cell(
                "vault",
                CellCountConstraint::exactly(1),  // One vault in inputs
                CellCountConstraint::exactly(0),  // No vault in outputs
            );
        assert!(rules.validate(&context).is_ok());
    }

    #[test]
    fn test_adjust_vault_validation() {
        // Import the open_vault module - need to check if recipes is exposed
        // Let's just create the rules inline for now
        
        // Create a valid adjustVault transaction (adding collateral)
        let input_cells = vec![
            create_vault_cell(0, 1000 * 10u128.pow(8), 500 * 10u128.pow(8)), // Existing vault
            create_xudt_cell(1, 500 * 10u128.pow(8)), // Additional collateral
        ];
        
        let output_cells = vec![
            create_vault_cell(0, 1500 * 10u128.pow(8), 500 * 10u128.pow(8)), // Updated vault
        ];
        
        let context = create_mock_transaction_context(
            CDP_ADJUST_VAULT,
            input_cells,
            output_cells,
        ).unwrap();
        
        let rules = TransactionValidationRules::new(CDP_ADJUST_VAULT.to_vec())
            .with_custom_cell(
                "vault",
                CellCountConstraint::exactly(1),  // One vault in inputs
                CellCountConstraint::exactly(1),  // One vault in outputs
            );
        assert!(rules.validate(&context).is_ok());
    }

    #[test]
    fn test_cell_count_validation() {
        // Import the open_vault module - need to check if recipes is exposed
        // Let's just create the rules inline for now
        
        // Create transaction with too many vault outputs (should fail)
        let input_cells = vec![
            create_simple_ckb_cell(0),
            create_xudt_cell(1, 1000 * 10u128.pow(8)),
        ];
        
        let output_cells = vec![
            create_vault_cell(0, 500 * 10u128.pow(8), 250 * 10u128.pow(8)),
            create_vault_cell(1, 500 * 10u128.pow(8), 250 * 10u128.pow(8)), // Extra vault
        ];
        
        let context = create_mock_transaction_context(
            CDP_OPEN_VAULT,
            input_cells,
            output_cells,
        ).unwrap();
        
        let rules = TransactionValidationRules::new(CDP_OPEN_VAULT.to_vec())
            .with_custom_cell(
                "vault",
                CellCountConstraint::exactly(0),
                CellCountConstraint::exactly(1),
            );
        assert_eq!(rules.validate(&context), Err(Error::CellCountViolation));
    }

    #[test]
    fn test_business_rule_validation() {
        // Test a business rule like minimum collateral ratio
        // This would be implemented in the actual validation predicates
        // For now, we just test the framework
        
        let custom_rules = TransactionValidationRules::new(b"customTest".to_vec())
            .with_business_rule(
                "min_collateral".to_string(),
                "Ensure minimum collateral".to_string(),
                vec!["vault".to_string()],
                |context| {
                    // Check if vault has minimum collateral
                    let vaults = context.output_cells.get_custom("vault");
                    if let Some(vault_cells) = vaults {
                        for vault in vault_cells {
                            // Parse vault data (collateral is first 16 bytes)
                            if vault.data.len() >= 16 {
                                let collateral = u128::from_le_bytes(
                                    vault.data[0..16].try_into().unwrap()
                                );
                                if collateral < 100 * 10u128.pow(8) { // Minimum 100 units
                                    return Err(Error::BusinessRuleViolation);
                                }
                            }
                        }
                    }
                    Ok(())
                }
            );
        
        // Test with insufficient collateral
        let input_cells = vec![create_xudt_cell(0, 50 * 10u128.pow(8))];
        let output_cells = vec![create_vault_cell(0, 50 * 10u128.pow(8), 25 * 10u128.pow(8))];
        
        let context = create_mock_transaction_context(
            b"customTest",
            input_cells,
            output_cells,
        ).unwrap();
        
        assert_eq!(custom_rules.validate(&context), Err(Error::BusinessRuleViolation));
    }
}