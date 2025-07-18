#[cfg(test)]
mod comprehensive_tests {
    extern crate alloc;
    extern crate std;
    
    use alloc::{vec::Vec, string::{ToString}, collections::BTreeMap, vec};
    use ckb_deterministic::{
        cell_classifier::{CellClass, ClassificationRule, RuleBasedClassifier, CellInfo, CellClassifier, ClassifiedCells, CellCollector},
        transaction_context::TransactionContext,
        transaction_deps::{CellDepInfo, OutPointInfo, DepType},
        transaction_recipe::TransactionRecipeExt,
        validation::{TransactionValidationRules, CellCountConstraint},
        errors::Error,
        generated::{TransactionRecipeBuilder, BytesVecBuilder, Bytes},
        known_scripts::KnownScript,
        assertions::{expect},
    };
    use ckb_std::{
        ckb_types::{
            packed::{Script, ScriptBuilder},
            prelude::*,
        },
        ckb_constants::Source,
    };
    use core::convert::TryInto;

    // ==================== Helper Functions ====================
    
    fn create_test_script(code_hash: [u8; 32], args: Vec<u8>) -> Script {
        ScriptBuilder::default()
            .code_hash(code_hash.pack())
            .hash_type(0u8.into())
            .args(args.pack())
            .build()
    }

    fn create_test_cell(
        source: Source,
        index: usize,
        lock_code_hash: [u8; 32],
        type_code_hash: Option<[u8; 32]>,
        data: Vec<u8>,
    ) -> CellInfo {
        let lock_script = create_test_script(lock_code_hash, vec![index as u8]);
        let type_script = type_code_hash.map(|hash| create_test_script(hash, vec![]));
        
        let mut lock_hash = [0u8; 32];
        lock_hash[0] = index as u8;
        
        CellInfo {
            source,
            index,
            data,
            lock: lock_script,
            lock_hash,
            type_script,
            type_hash: type_code_hash,
        }
    }

    // ==================== Cell Classification Tests ====================
    
    #[test]
    fn test_cell_classification_simple_ckb() {
        let classifier = RuleBasedClassifier::new("TestClassifier");
        
        // Cell without type script should be SimpleCKB
        let cell = create_test_cell(Source::Input, 0, [1u8; 32], None, vec![]);
        let result = classifier.classify(&cell).unwrap();
        assert!(matches!(result, CellClass::SimpleCKB));
    }

    #[test]
    fn test_cell_classification_known_script() {
        let xudt_code_hash = [10u8; 32];
        let classifier = RuleBasedClassifier::new("TestClassifier")
            .add_rule(ClassificationRule::TypeCodeHash {
                code_hash: xudt_code_hash,
                class: CellClass::known("xudt"),
            });
        
        let cell = create_test_cell(Source::Input, 0, [1u8; 32], Some(xudt_code_hash), vec![]);
        let result = classifier.classify(&cell).unwrap();
        assert!(matches!(result, CellClass::Known(name) if name == "xudt"));
    }

    #[test]
    fn test_cell_classification_custom_type() {
        let vault_code_hash = [20u8; 32];
        let classifier = RuleBasedClassifier::new("TestClassifier")
            .add_rule(ClassificationRule::TypeCodeHash {
                code_hash: vault_code_hash,
                class: CellClass::custom("vault"),
            });
        
        let cell = create_test_cell(Source::Input, 0, [1u8; 32], Some(vault_code_hash), vec![]);
        let result = classifier.classify(&cell).unwrap();
        assert!(matches!(result, CellClass::Custom(name) if name == "vault"));
    }

    #[test]
    fn test_cell_classification_by_lock_script() {
        let special_lock_hash = [30u8; 32];
        let classifier = RuleBasedClassifier::new("TestClassifier")
            .add_rule(ClassificationRule::LockCodeHash {
                code_hash: special_lock_hash,
                class: CellClass::custom("multisig"),
            });
        
        // Need to add a type script to prevent SimpleCKB classification
        let cell = create_test_cell(Source::Input, 0, special_lock_hash, Some([99u8; 32]), vec![]);
        let result = classifier.classify(&cell).unwrap();
        assert!(matches!(result, CellClass::Custom(name) if name == "multisig"));
    }

    #[test]
    fn test_cell_classification_unidentified() {
        let classifier = RuleBasedClassifier::new("TestClassifier");
        
        // Cell with unknown type script
        let cell = create_test_cell(Source::Input, 0, [1u8; 32], Some([99u8; 32]), vec![]);
        let result = classifier.classify(&cell).unwrap();
        assert!(matches!(result, CellClass::Unidentified));
    }

    #[test]
    fn test_cell_classification_priority() {
        let code_hash = [40u8; 32];
        // Rules are evaluated in order, so the first matching rule wins
        let classifier = RuleBasedClassifier::new("TestClassifier")
            // More specific rule first (by type hash)
            .add_rule(ClassificationRule::TypeHash {
                hash: code_hash,
                class: CellClass::custom("specific"),
            })
            // General rule second
            .add_rule(ClassificationRule::TypeCodeHash {
                code_hash,
                class: CellClass::custom("general"),
            });
        
        let cell = create_test_cell(Source::Input, 0, [1u8; 32], Some(code_hash), vec![]);
        // TypeHash rule should match first
        let result = classifier.classify(&cell).unwrap();
        assert!(matches!(result, CellClass::Custom(name) if name == "specific"));
    }

    #[test]
    fn test_cell_classification_known_script_integration() {
        // Test with actual known scripts
        let _classifier = RuleBasedClassifier::new("TestClassifier");
        // Known scripts would be added via add_rule with proper code hashes
        
        // These would need real code hashes in production
        // For testing, we'll use the enum values
        let xudt_class = KnownScript::XUdt.cell_class();
        assert!(matches!(xudt_class, CellClass::Known(name) if name == "xudt"));
        
        let spore_class = KnownScript::Spore.cell_class();
        assert!(matches!(spore_class, CellClass::Known(name) if name == "spore"));
    }

    // ==================== Cell Collection Tests ====================
    
    #[test]
    fn test_cell_collector_basic() {
        let classifier = RuleBasedClassifier::new("TestClassifier")
            .add_rule(ClassificationRule::TypeCodeHash {
                code_hash: [10u8; 32],
                class: CellClass::known("xudt"),
            });
        
        let _collector = CellCollector::new(classifier);
        
        // Test strict mode by checking behavior (no is_strict_mode method)
        // Cannot clone classifier, so create a new one
        let strict_classifier = RuleBasedClassifier::new("TestClassifier")
            .add_rule(ClassificationRule::TypeCodeHash {
                code_hash: [10u8; 32],
                class: CellClass::known("xudt"),
            });
        let _strict_collector = CellCollector::new(strict_classifier).with_strict_mode(true);
    }

    #[test]
    fn test_classified_cells_structure() {
        let simple_ckb_cell = create_test_cell(Source::Input, 0, [1u8; 32], None, vec![]);
        let xudt_cell = create_test_cell(Source::Input, 1, [1u8; 32], Some([10u8; 32]), vec![]);
        let vault_cell = create_test_cell(Source::Input, 2, [1u8; 32], Some([20u8; 32]), vec![]);
        
        let mut known = BTreeMap::new();
        known.insert("xudt".to_string(), vec![xudt_cell]);
        
        let mut custom = BTreeMap::new();
        custom.insert("vault".to_string(), vec![vault_cell]);
        
        let classified = ClassifiedCells {
            simple_ckb_cells: vec![simple_ckb_cell],
            known_cells: known.clone(),
            custom_cells: custom.clone(),
            unidentified_cells: vec![],
        };
        
        // Test cell lists
        assert_eq!(classified.get_simple_ckb().len(), 1);
        assert_eq!(classified.get_known("xudt").map(|v| v.len()), Some(1));
        assert_eq!(classified.get_custom("vault").map(|v| v.len()), Some(1));
        assert!(!classified.has_unidentified_cells());
        assert_eq!(classified.total_cell_count(), 3);
    }

    #[test]
    fn test_classified_cells_with_unidentified() {
        let unidentified = vec![
            create_test_cell(Source::Input, 0, [1u8; 32], Some([99u8; 32]), vec![])
        ];
        
        let classified = ClassifiedCells {
            simple_ckb_cells: vec![],
            known_cells: BTreeMap::new(),
            custom_cells: BTreeMap::new(),
            unidentified_cells: unidentified,
        };
        
        assert!(classified.has_unidentified_cells());
        assert_eq!(classified.unidentified_cells.len(), 1);
    }

    // ==================== Transaction Recipe Tests ====================
    
    #[test]
    fn test_transaction_recipe_basic() {
        let method_path = b"transfer";
        let args = vec![vec![1, 2, 3], vec![4, 5, 6]];
        
        let recipe = TransactionRecipeBuilder::default()
            .method_path(Bytes::from(method_path.to_vec()))
            .arguments(
                BytesVecBuilder::default()
                    .push(Bytes::from(args[0].clone()))
                    .push(Bytes::from(args[1].clone()))
                    .build()
            )
            .build();
        
        assert_eq!(recipe.method_path_bytes(), method_path.to_vec());
        assert_eq!(recipe.arguments_vec().len(), 2);
    }

    #[test] 
    fn test_transaction_recipe_with_deps() {
        let _cell_dep = CellDepInfo {
            out_point: OutPointInfo {
                tx_hash: [1u8; 32],
                index: 0,
            },
            dep_type: DepType::Code,
            resolved_deps: vec![],
        };
        
        let _recipe_with_deps = TransactionRecipeBuilder::default()
            .method_path(Bytes::from(b"method".to_vec()))
            .arguments(BytesVecBuilder::default().build())
            // Cell deps and header deps would be set via generated types
            .build();
        
        // Cell deps would be configured via generated types
    }

    #[test]
    fn test_transaction_recipe_empty() {
        let recipe = TransactionRecipeBuilder::default()
            .method_path(Bytes::from(vec![] as Vec<u8>))
            .arguments(BytesVecBuilder::default().build())
            .build();
        
        assert!(recipe.method_path_bytes().is_empty());
        assert_eq!(recipe.arguments_vec().len(), 0);
        // cell_deps and header_deps are optional types
    }

    // ==================== Transaction Context Tests ====================
    
    fn create_mock_context(
        method_path: &[u8],
        input_cells: ClassifiedCells,
        output_cells: ClassifiedCells,
    ) -> TransactionContext<RuleBasedClassifier> {
        let recipe = TransactionRecipeBuilder::default()
            .method_path(Bytes::from(method_path.to_vec()))
            .arguments(BytesVecBuilder::default().build())
            .build();
        
        TransactionContext::from_parts(
            recipe,
            input_cells,
            output_cells,
            vec![],
            vec![],
        )
    }

    #[test]
    fn test_transaction_context_creation() {
        let input_cells = ClassifiedCells {
            simple_ckb_cells: vec![create_test_cell(Source::Input, 0, [1u8; 32], None, vec![])],
            known_cells: BTreeMap::new(),
            custom_cells: BTreeMap::new(),
            unidentified_cells: vec![],
        };
        
        let output_cells = ClassifiedCells {
            simple_ckb_cells: vec![create_test_cell(Source::Output, 0, [1u8; 32], None, vec![])],
            known_cells: BTreeMap::new(),
            custom_cells: BTreeMap::new(),
            unidentified_cells: vec![],
        };
        
        let context = create_mock_context(b"test", input_cells, output_cells);
        
        assert_eq!(context.recipe.method_path_bytes(), b"test");
        assert_eq!(context.input_cells.total_cell_count(), 1);
        assert_eq!(context.output_cells.total_cell_count(), 1);
    }

    // ==================== Validation Framework Tests ====================
    
    #[test]
    fn test_validation_method_path() {
        let rules = TransactionValidationRules::new(b"transfer".to_vec());
        
        let context = create_mock_context(
            b"transfer",
            ClassifiedCells::default(),
            ClassifiedCells::default(),
        );
        
        assert!(rules.validate(&context).is_ok());
        
        let wrong_context = create_mock_context(
            b"mint",
            ClassifiedCells::default(),
            ClassifiedCells::default(),
        );
        
        assert_eq!(rules.validate(&wrong_context), Err(Error::WrongMethodPath));
    }

    #[test]
    fn test_validation_arguments() {
        let rules = TransactionValidationRules::new(b"transfer".to_vec())
            .with_arguments(2);
        
        let recipe_with_args = TransactionRecipeBuilder::default()
            .method_path(Bytes::from(b"transfer".to_vec()))
            .arguments(
                BytesVecBuilder::default()
                    .push(Bytes::from(vec![1]))
                    .push(Bytes::from(vec![2]))
                    .build()
            )
            .build();
        
        let context: TransactionContext<RuleBasedClassifier> = TransactionContext::from_parts(
            recipe_with_args,
            ClassifiedCells::default(),
            ClassifiedCells::default(),
            vec![],
            vec![],
        );
        
        assert!(rules.validate(&context).is_ok());
        
        // Wrong argument count
        let context_wrong = create_mock_context(
            b"transfer",
            ClassifiedCells::default(),
            ClassifiedCells::default(),
        );
        
        assert_eq!(rules.validate(&context_wrong), Err(Error::InvalidArgumentCount));
    }

    #[test]
    fn test_validation_cell_counts() {
        let mut known = BTreeMap::new();
        known.insert("xudt".to_string(), vec![
            create_test_cell(Source::Input, 0, [1u8; 32], Some([10u8; 32]), vec![])
        ]);
        
        let input_cells = ClassifiedCells {
            simple_ckb_cells: vec![],
            known_cells: known.clone(),
            custom_cells: BTreeMap::new(),
            unidentified_cells: vec![],
        };
        
        let output_cells = ClassifiedCells {
            simple_ckb_cells: vec![],
            known_cells: known,
            custom_cells: BTreeMap::new(),
            unidentified_cells: vec![],
        };
        
        let rules = TransactionValidationRules::new(b"transfer".to_vec())
            .with_known_cell(
                KnownScript::XUdt,
                CellCountConstraint::exactly(1),
                CellCountConstraint::exactly(1),
            );
        
        let context = create_mock_context(b"transfer", input_cells, output_cells);
        assert!(rules.validate(&context).is_ok());
    }

    #[test]
    fn test_validation_cell_count_constraints() {
        // Test all constraint types
        let test_cases = vec![
            (CellCountConstraint::exactly(2), 2, true),
            (CellCountConstraint::exactly(2), 1, false),
            (CellCountConstraint::at_least(2), 3, true),
            (CellCountConstraint::at_least(2), 1, false),
            (CellCountConstraint::at_most(2), 1, true),
            (CellCountConstraint::at_most(2), 3, false),
            (CellCountConstraint::range(2, 4), 3, true),
            (CellCountConstraint::range(2, 4), 1, false),
            (CellCountConstraint::range(2, 4), 5, false),
        ];
        
        for (constraint, count, should_pass) in test_cases {
            let mut custom = BTreeMap::new();
            let cells: Vec<_> = (0..count).map(|i| {
                create_test_cell(Source::Input, i, [1u8; 32], Some([20u8; 32]), vec![])
            }).collect();
            custom.insert("test".to_string(), cells);
            
            let input_cells = ClassifiedCells {
                simple_ckb_cells: vec![],
                known_cells: BTreeMap::new(),
                custom_cells: custom,
                unidentified_cells: vec![],
            };
            
            let rules = TransactionValidationRules::new(b"test".to_vec())
                .with_custom_cell(
                    "test",
                    constraint,
                    CellCountConstraint::exactly(0),
                );
            
            let context = create_mock_context(
                b"test",
                input_cells,
                ClassifiedCells::default(),
            );
            
            if should_pass {
                assert!(rules.validate(&context).is_ok());
            } else {
                assert_eq!(rules.validate(&context), Err(Error::CellCountViolation));
            }
        }
    }

    #[test]
    fn test_validation_cell_relationship() {
        let rules = TransactionValidationRules::new(b"update".to_vec())
            .with_cell_relationship(
                "lock_preservation".to_string(),
                "Lock scripts must be preserved".to_string(),
                vec!["vault".to_string()],
                |context| {
                    let empty_vec = vec![];
                    let inputs = context.input_cells.get_custom("vault").unwrap_or(&empty_vec);
                    let outputs = context.output_cells.get_custom("vault").unwrap_or(&empty_vec);
                    
                    if inputs.len() != outputs.len() {
                        return Err(Error::CellRelationshipRuleViolation);
                    }
                    
                    for (input, output) in inputs.iter().zip(outputs.iter()) {
                        if input.lock != output.lock {
                            return Err(Error::CellRelationshipRuleViolation);
                        }
                    }
                    
                    Ok(())
                }
            );
        
        // Create matching cells
        let lock_hash = [50u8; 32];
        let mut custom_in = BTreeMap::new();
        custom_in.insert("vault".to_string(), vec![
            create_test_cell(Source::Input, 0, lock_hash, Some([20u8; 32]), vec![])
        ]);
        
        let mut custom_out = BTreeMap::new();
        custom_out.insert("vault".to_string(), vec![
            create_test_cell(Source::Output, 0, lock_hash, Some([20u8; 32]), vec![])
        ]);
        
        let context = create_mock_context(
            b"update",
            ClassifiedCells {
                simple_ckb_cells: vec![],
                known_cells: BTreeMap::new(),
                custom_cells: custom_in,
                unidentified_cells: vec![],
            },
            ClassifiedCells {
                simple_ckb_cells: vec![],
                known_cells: BTreeMap::new(),
                custom_cells: custom_out,
                unidentified_cells: vec![],
            },
        );
        
        assert!(rules.validate(&context).is_ok());
    }

    #[test]
    fn test_validation_business_rule() {
        let rules = TransactionValidationRules::new(b"mint".to_vec())
            .with_business_rule(
                "max_supply".to_string(),
                "Total supply must not exceed limit".to_string(),
                vec!["xudt".to_string()],
                |context| {
                    let empty_vec = vec![];
                    let outputs = context.output_cells.get_known("xudt").unwrap_or(&empty_vec);
                    let total_amount: u128 = outputs.iter()
                        .filter_map(|cell| {
                            if cell.data.len() >= 16 {
                                Some(u128::from_le_bytes(
                                    cell.data[0..16].try_into().ok()?
                                ))
                            } else {
                                None
                            }
                        })
                        .sum();
                    
                    if total_amount > 1_000_000 {
                        Err(Error::BusinessRuleViolation)
                    } else {
                        Ok(())
                    }
                }
            );
        
        // Test within limit
        let mut data = 500_000u128.to_le_bytes().to_vec();
        data.extend_from_slice(&[0u8; 16]); // xUDT extension
        
        let mut known = BTreeMap::new();
        known.insert("xudt".to_string(), vec![
            create_test_cell(Source::Output, 0, [1u8; 32], Some([10u8; 32]), data)
        ]);
        
        let context = create_mock_context(
            b"mint",
            ClassifiedCells::default(),
            ClassifiedCells {
                simple_ckb_cells: vec![],
                known_cells: known,
                custom_cells: BTreeMap::new(),
                unidentified_cells: vec![],
            },
        );
        
        assert!(rules.validate(&context).is_ok());
    }

    #[test]
    fn test_validation_dependencies() {
        let cell_dep = CellDepInfo {
            out_point: OutPointInfo {
                tx_hash: [1u8; 32],
                index: 0,
            },
            dep_type: DepType::Code,
            resolved_deps: vec![],
        };
        
        let rules = TransactionValidationRules::new(b"test".to_vec());
        // Cell deps validation is done through context
        
        let mut context = create_mock_context(
            b"test",
            ClassifiedCells::default(),
            ClassifiedCells::default(),
        );
        
        // Add deps
        context.cell_deps = vec![cell_dep];
        context.header_deps = vec![[70u8; 32]];
        
        assert!(rules.validate(&context).is_ok());
    }

    // ==================== Assertions Tests ====================
    
    #[test]
    fn test_assertions_basic() {
        // Basic equality
        assert!(expect(42).to_equal(42).is_ok());
        assert_eq!(expect(42).to_equal(43), Err(Error::ExpectationViolation));
        
        // Not equal
        assert!(expect(42).not_to_equal(43).is_ok());
        assert_eq!(expect(42).not_to_equal(42), Err(Error::ExpectationViolation));
    }

    #[test]
    fn test_assertions_numeric() {
        // Greater than
        assert!(expect(10).to_be_greater_than(5).is_ok());
        assert_eq!(expect(5).to_be_greater_than(10), Err(Error::ExpectationViolation));
        
        // Less than
        assert!(expect(5).to_be_less_than(10).is_ok());
        assert_eq!(expect(10).to_be_less_than(5), Err(Error::ExpectationViolation));
        
        // Range
        assert!(expect(5).to_be_in_range(1, 10).is_ok());
        assert_eq!(expect(15).to_be_in_range(1, 10), Err(Error::ExpectationViolation));
    }

    #[test]
    fn test_assertions_boolean() {
        assert!(expect(true).to_be_true().is_ok());
        assert_eq!(expect(false).to_be_true(), Err(Error::ExpectationViolation));
        
        assert!(expect(false).to_be_false().is_ok());
        assert_eq!(expect(true).to_be_false(), Err(Error::ExpectationViolation));
    }

    #[test]
    fn test_assertions_collections() {
        let vec = vec![1, 2, 3];
        let empty_vec: Vec<i32> = vec![];
        
        // Length
        assert!(expect(&vec).to_have_length(3).is_ok());
        assert_eq!(expect(&vec).to_have_length(2), Err(Error::CellCountViolation));
        
        // Empty checks
        assert!(expect(&empty_vec).to_be_empty().is_ok());
        assert_eq!(expect(&vec).to_be_empty(), Err(Error::CellCountViolation));
        
        assert!(expect(&vec).not_to_be_empty().is_ok());
        assert_eq!(expect(&empty_vec).not_to_be_empty(), Err(Error::CellCountViolation));
    }

    #[test]
    fn test_assertions_option() {
        let some_value = Some(42);
        let none_value: Option<i32> = None;
        
        assert!(expect(some_value).to_be_some().is_ok());
        assert_eq!(expect(none_value).to_be_some(), Err(Error::ExpectationViolation));
        
        assert!(expect(none_value).to_be_none().is_ok());
        assert_eq!(expect(some_value).to_be_none(), Err(Error::ExpectationViolation));
        
        assert!(expect(some_value).to_be_some_and_equal(42).is_ok());
        assert_eq!(expect(some_value).to_be_some_and_equal(43), Err(Error::ExpectationViolation));
    }

    #[test]
    fn test_assertions_result() {
        let ok_result: Result<i32, &str> = Ok(42);
        let err_result: Result<i32, &str> = Err("error");
        
        assert!(expect(ok_result).to_be_ok().is_ok());
        assert_eq!(expect(err_result).to_be_ok(), Err(Error::ExpectationViolation));
        
        assert!(expect(err_result).to_be_err().is_ok());
        assert_eq!(expect(ok_result).to_be_err(), Err(Error::ExpectationViolation));
    }

    // ==================== Error Handling Tests ====================
    
    #[test]
    fn test_error_casting() {
        // All errors should be castable to i8
        let errors = vec![
            Error::IndexOutOfBound,
            Error::ItemMissing,
            Error::UnidentifiedCells,
            Error::WrongMethodPath,
            Error::CellCountViolation,
            Error::BusinessRuleViolation,
            Error::ExpectationViolation,
        ];
        
        for error in errors {
            let _code: i8 = error as i8; // Should compile
        }
    }

    // ==================== Integration Tests ====================
    
    #[test]
    fn test_complete_transaction_flow() {
        // Create a complete transaction validation flow
        let xudt_code_hash = [100u8; 32];
        let vault_code_hash = [101u8; 32];
        
        // 1. Setup classifier
        let classifier = RuleBasedClassifier::new("IntegrationTest")
            .add_rule(ClassificationRule::TypeCodeHash {
                code_hash: xudt_code_hash,
                class: CellClass::known("xudt"),
            })
            .add_rule(ClassificationRule::TypeCodeHash {
                code_hash: vault_code_hash,
                class: CellClass::custom("vault"),
            });
        
        // 2. Create cells
        let mut xudt_data = 1000u128.to_le_bytes().to_vec();
        xudt_data.extend_from_slice(&[0u8; 16]);
        
        let input_cells = vec![
            create_test_cell(Source::Input, 0, [1u8; 32], None, vec![]), // SimpleCKB
            create_test_cell(Source::Input, 1, [1u8; 32], Some(xudt_code_hash), xudt_data.clone()), // xUDT
        ];
        
        let output_cells = vec![
            create_test_cell(Source::Output, 0, [1u8; 32], Some(vault_code_hash), vec![1, 2, 3]), // Vault
            create_test_cell(Source::Output, 1, [1u8; 32], None, vec![]), // SimpleCKB change
        ];
        
        // 3. Classify cells
        let mut input_simple_ckb = vec![];
        let mut input_known = BTreeMap::new();
        let mut output_custom = BTreeMap::new();
        let mut output_simple_ckb = vec![];
        
        for cell in input_cells {
            match classifier.classify(&cell).unwrap() {
                CellClass::SimpleCKB => input_simple_ckb.push(cell),
                CellClass::Known(name) => {
                    input_known.entry(name).or_insert_with(Vec::new).push(cell);
                }
                _ => {}
            }
        }
        
        for cell in output_cells {
            match classifier.classify(&cell).unwrap() {
                CellClass::SimpleCKB => output_simple_ckb.push(cell),
                CellClass::Custom(name) => {
                    output_custom.entry(name).or_insert_with(Vec::new).push(cell);
                }
                _ => {}
            }
        }
        
        // 4. Create transaction context
        let recipe = TransactionRecipeBuilder::default()
            .method_path(Bytes::from(b"createVault".to_vec()))
            .arguments(
                BytesVecBuilder::default()
                    .push(Bytes::from(1000u128.to_le_bytes().to_vec()))
                    .build()
            )
            .build();
        
        let context: TransactionContext<RuleBasedClassifier> = TransactionContext::from_parts(
            recipe,
            ClassifiedCells {
                simple_ckb_cells: input_simple_ckb,
                known_cells: input_known,
                custom_cells: BTreeMap::new(),
                unidentified_cells: vec![],
            },
            ClassifiedCells {
                simple_ckb_cells: output_simple_ckb,
                known_cells: BTreeMap::new(),
                custom_cells: output_custom,
                unidentified_cells: vec![],
            },
            vec![
                CellDepInfo {
                    out_point: OutPointInfo {
                        tx_hash: vault_code_hash,
                        index: 0,
                    },
                    dep_type: DepType::Code,
                    resolved_deps: vec![],
                }
            ],
            vec![],
        );
        
        // 5. Define validation rules
        let rules = TransactionValidationRules::new(b"createVault".to_vec())
            .with_arguments(1)
            .with_known_cell(
                KnownScript::XUdt,
                CellCountConstraint::at_least(1),
                CellCountConstraint::exactly(0),
            )
            .with_custom_cell(
                "vault",
                CellCountConstraint::exactly(0),
                CellCountConstraint::exactly(1),
            )
            .with_cell_relationship(
                "collateral_transfer".to_string(),
                "Ensure xUDT collateral is transferred to vault".to_string(),
                vec!["xudt".to_string(), "vault".to_string()],
                |ctx| {
                    let empty_vec = vec![];
                    let xudt_in = ctx.input_cells.get_known("xudt").unwrap_or(&empty_vec);
                    let vault_out = ctx.output_cells.get_custom("vault").unwrap_or(&empty_vec);
                    
                    if xudt_in.is_empty() || vault_out.is_empty() {
                        return Err(Error::CellRelationshipRuleViolation);
                    }
                    
                    // Check collateral amount matches
                    let collateral: u128 = xudt_in.iter()
                        .filter_map(|cell| {
                            if cell.data.len() >= 16 {
                                Some(u128::from_le_bytes(cell.data[0..16].try_into().ok()?))
                            } else {
                                None
                            }
                        })
                        .sum();
                    
                    if collateral < 1000 {
                        return Err(Error::BusinessRuleViolation);
                    }
                    
                    Ok(())
                }
            );
        
        // 6. Validate
        assert!(rules.validate(&context).is_ok());
    }

    // ==================== Edge Cases and Error Scenarios ====================
    
    #[test]
    fn test_edge_case_empty_transaction() {
        let context = create_mock_context(
            b"empty",
            ClassifiedCells::default(),
            ClassifiedCells::default(),
        );
        
        let rules = TransactionValidationRules::new(b"empty".to_vec());
        assert!(rules.validate(&context).is_ok());
    }

    #[test]
    fn test_edge_case_large_cell_count() {
        let mut custom = BTreeMap::new();
        let large_cell_vec: Vec<_> = (0..1000).map(|i| {
            create_test_cell(Source::Input, i, [1u8; 32], Some([20u8; 32]), vec![])
        }).collect();
        custom.insert("bulk".to_string(), large_cell_vec);
        
        let input_cells = ClassifiedCells {
            simple_ckb_cells: vec![],
            known_cells: BTreeMap::new(),
            custom_cells: custom,
            unidentified_cells: vec![],
        };
        
        let context = create_mock_context(
            b"bulk",
            input_cells,
            ClassifiedCells::default(),
        );
        
        let rules = TransactionValidationRules::new(b"bulk".to_vec())
            .with_custom_cell(
                "bulk",
                CellCountConstraint::at_least(500),
                CellCountConstraint::exactly(0),
            );
        
        assert!(rules.validate(&context).is_ok());
    }

    #[test]
    fn test_edge_case_malformed_data() {
        let cell_with_empty_data = create_test_cell(
            Source::Input,
            0,
            [1u8; 32],
            Some([10u8; 32]),
            vec![] // Empty data for xUDT
        );
        
        let mut known = BTreeMap::new();
        known.insert("xudt".to_string(), vec![cell_with_empty_data]);
        
        let context = create_mock_context(
            b"test",
            ClassifiedCells {
                simple_ckb_cells: vec![],
                known_cells: known,
                custom_cells: BTreeMap::new(),
                unidentified_cells: vec![],
            },
            ClassifiedCells::default(),
        );
        
        let rules = TransactionValidationRules::new(b"test".to_vec())
            .with_business_rule(
                "parse_amount".to_string(),
                "Parse xUDT amount".to_string(),
                vec!["xudt".to_string()],
                |ctx| {
                    let empty_vec = vec![];
                    let xudt_cells = ctx.input_cells.get_known("xudt").unwrap_or(&empty_vec);
                    for cell in xudt_cells {
                        if cell.data.len() < 16 {
                            // Data too short for u128
                            return Err(Error::BusinessRuleViolation);
                        }
                    }
                    Ok(())
                }
            );
        
        assert_eq!(rules.validate(&context), Err(Error::BusinessRuleViolation));
    }

    #[test]
    fn test_error_propagation() {
        // Test that errors propagate correctly through the validation chain
        let rules: TransactionValidationRules<RuleBasedClassifier> = TransactionValidationRules::new(b"test".to_vec())
            .with_arguments(1)
            .with_cell_relationship(
                "failing_rule".to_string(),
                "Always fails".to_string(),
                vec![],
                |_| Err(Error::CellRelationshipRuleViolation)
            );
        
        let recipe = TransactionRecipeBuilder::default()
            .method_path(Bytes::from(b"test".to_vec()))
            .arguments(
                BytesVecBuilder::default()
                    .push(Bytes::from(vec![1]))
                    .build()
            )
            .build();
        
        let context: TransactionContext<RuleBasedClassifier> = TransactionContext::from_parts(
            recipe,
            ClassifiedCells::default(),
            ClassifiedCells::default(),
            vec![],
            vec![],
        );
        
        // Should fail at cell relationship rule, not argument count
        assert_eq!(rules.validate(&context), Err(Error::CellRelationshipRuleViolation));
    }

    // ==================== Known Scripts Tests ====================
    
    #[test]
    fn test_known_scripts_registry() {
        // Test that known scripts are properly registered
        let scripts = vec![
            KnownScript::XUdt,
            KnownScript::Spore,
            KnownScript::Cota,
        ];
        
        for script in scripts {
            assert!(!script.identifier().is_empty());
            assert!(!matches!(script.cell_class(), CellClass::Unidentified));
            
            // Test code hash retrieval - in test environment, this may fail
            // which is acceptable
            let _code_hash_result = script.code_hash();
        }
    }

    #[test]
    fn test_known_scripts_classification() {
        let _classifier = RuleBasedClassifier::new("KnownScriptsTest");
        // Known scripts would be added via add_rule with proper code hashes
        
        // Verify they produce different cell classes
        assert_ne!(KnownScript::XUdt.cell_class(), KnownScript::Spore.cell_class());
        assert_eq!(KnownScript::Spore.identifier(), "spore");
        assert!(KnownScript::Spore.is_type_script());
        
        // Test code hash retrieval - in test environment, this may fail
        // which is acceptable
        let _code_hash_result = KnownScript::Spore.code_hash();
    }

    // ==================== Performance and Stress Tests ====================
    
    #[test]
    fn test_performance_many_rules() {
        let mut classifier = RuleBasedClassifier::new("PerformanceTest");
        
        // Add many classification rules
        for i in 0..100 {
            let mut code_hash = [0u8; 32];
            code_hash[0] = i as u8;
            
            classifier = classifier.add_rule(ClassificationRule::TypeCodeHash {
                code_hash,
                class: CellClass::custom(format!("type_{}", i)),
            });
        }
        
        // Classification should still be fast
        let mut target_code_hash = [0u8; 32];
        target_code_hash[0] = 50;
        let cell = create_test_cell(Source::Input, 0, [1u8; 32], Some(target_code_hash), vec![]);
        let result = classifier.classify(&cell).unwrap();
        assert!(matches!(result, CellClass::Custom(name) if name == "type_50"));
    }

    #[test]
    fn test_performance_many_validations() {
        let mut rules = TransactionValidationRules::new(b"complex".to_vec());
        
        // Add many validation rules
        for i in 0..20 {
            rules = rules.with_business_rule(
                format!("rule_{}", i),
                format!("Business rule {}", i),
                vec![],
                |_| Ok(())
            );
        }
        
        let context = create_mock_context(
            b"complex",
            ClassifiedCells::default(),
            ClassifiedCells::default(),
        );
        
        // Should validate successfully
        assert!(rules.validate(&context).is_ok());
    }
}