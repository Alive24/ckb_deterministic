use ckb_deterministic::cell_classifier::{CellInfo, ClassifiedCells, RuleBasedClassifier};
use ckb_deterministic::errors::Error;
use ckb_deterministic::generated::{Bytes, RecipeArgumentVec, TransactionRecipeBuilder};
use ckb_deterministic::transaction_context::TransactionContext;
use ckb_deterministic::transaction_recipe::create_inline_argument;
use ckb_deterministic::validation::{CellCountConstraint, TransactionValidationRules};
use ckb_std::ckb_types::packed::{Script, ScriptBuilder};
use ckb_std::ckb_types::prelude::*;
extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::string::ToString;
use alloc::vec::Vec;

fn create_test_script(code_hash: [u8; 32]) -> Script {
    ScriptBuilder::default()
        .code_hash(code_hash.pack())
        .hash_type(0u8.into())
        .args(vec![0u8].pack())
        .build()
}

fn create_test_cell_info(_cell_class: &str, type_code_hash: Option<[u8; 32]>) -> CellInfo {
    let lock_script = create_test_script([1u8; 32]);
    let type_script = type_code_hash.map(|hash| create_test_script(hash));
    let lock_hash = [0u8; 32];
    let type_hash = type_code_hash;

    CellInfo {
        source: ckb_std::ckb_constants::Source::Input,
        index: 0,
        data: vec![],
        lock: lock_script,
        lock_hash,
        type_script,
        type_hash,
    }
}

fn create_test_context(
    method_path: &[u8],
    input_cells: Vec<(&str, usize)>,
    output_cells: Vec<(&str, usize)>,
) -> TransactionContext<RuleBasedClassifier> {
    // Create recipe
    let recipe = TransactionRecipeBuilder::default()
        .method_path(Bytes::from(method_path.to_vec()))
        .arguments(RecipeArgumentVec::new_builder().build())
        .build();

    // Create classified cells
    let mut input_map = BTreeMap::new();
    let mut output_map = BTreeMap::new();

    for (cell_type, count) in input_cells {
        let cells: Vec<CellInfo> = (0..count)
            .map(|_| {
                if cell_type == "simple_ckb" {
                    create_test_cell_info(cell_type, None)
                } else {
                    create_test_cell_info(cell_type, Some([2u8; 32]))
                }
            })
            .collect();
        input_map.insert(cell_type.to_string(), cells);
    }

    for (cell_type, count) in output_cells {
        let cells: Vec<CellInfo> = (0..count)
            .map(|_| {
                if cell_type == "simple_ckb" {
                    create_test_cell_info(cell_type, None)
                } else {
                    create_test_cell_info(cell_type, Some([2u8; 32]))
                }
            })
            .collect();
        output_map.insert(cell_type.to_string(), cells);
    }

    let input_cells = ClassifiedCells {
        simple_ckb_cells: input_map.get("simple_ckb").cloned().unwrap_or_default(),
        known_cells: input_map
            .iter()
            .filter(|(k, _)| k.as_str() != "simple_ckb" && !k.starts_with("custom_"))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        custom_cells: input_map
            .iter()
            .filter(|(k, _)| k.starts_with("custom_"))
            .map(|(k, v)| (k.strip_prefix("custom_").unwrap().to_string(), v.clone()))
            .collect(),
        unidentified_cells: vec![],
    };

    let output_cells = ClassifiedCells {
        simple_ckb_cells: output_map.get("simple_ckb").cloned().unwrap_or_default(),
        known_cells: output_map
            .iter()
            .filter(|(k, _)| k.as_str() != "simple_ckb" && !k.starts_with("custom_"))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        custom_cells: output_map
            .iter()
            .filter(|(k, _)| k.starts_with("custom_"))
            .map(|(k, v)| (k.strip_prefix("custom_").unwrap().to_string(), v.clone()))
            .collect(),
        unidentified_cells: vec![],
    };

    TransactionContext::from_parts(
        recipe,
        input_cells,
        output_cells,
        ClassifiedCells::default(),
        vec![],
    )
}

#[test]
fn test_method_path_validation() {
    let rules = TransactionValidationRules::new(b"transfer".to_vec());

    // Matching method path should pass
    let context = create_test_context(b"transfer", vec![], vec![]);
    assert!(rules.validate(&context).is_ok());

    // Different method path should fail
    let context = create_test_context(b"mint", vec![], vec![]);
    assert_eq!(rules.validate(&context), Err(Error::WrongMethodPath));
}

#[test]
fn test_argument_count_validation() {
    let rules = TransactionValidationRules::new(b"transfer".to_vec()).with_arguments(3);

    // Create recipe with 3 arguments
    let recipe_with_args = TransactionRecipeBuilder::default()
        .method_path(Bytes::from(b"transfer".to_vec()))
        .arguments(
            RecipeArgumentVec::new_builder()
                .push(create_inline_argument(&vec![1]))
                .push(create_inline_argument(&vec![2]))
                .push(create_inline_argument(&vec![3]))
                .build(),
        )
        .build();

    let context = TransactionContext::from_parts(
        recipe_with_args,
        ClassifiedCells::default(),
        ClassifiedCells::default(),
        ClassifiedCells::default(),
        vec![],
    );

    assert!(rules.validate(&context).is_ok());

    // Wrong argument count should fail
    let context_wrong_args = create_test_context(b"transfer", vec![], vec![]);
    assert_eq!(
        rules.validate(&context_wrong_args),
        Err(Error::InvalidArgumentCount)
    );
}

#[test]
fn test_cell_count_constraints() {
    let rules = TransactionValidationRules::new(b"transfer".to_vec()).with_custom_cell(
        "vault",
        CellCountConstraint::at_least(1),
        CellCountConstraint::exactly(1),
    );

    // Valid counts - note that custom cells are stored with "custom_" prefix removed
    let context = create_test_context(
        b"transfer",
        vec![("custom_vault", 2)],
        vec![("custom_vault", 1)],
    );
    assert!(rules.validate(&context).is_ok());

    // Too few vault inputs
    let context_fail = create_test_context(
        b"transfer",
        vec![("custom_vault", 0)], // Less than 1 required
        vec![("custom_vault", 1)],
    );
    assert_eq!(
        rules.validate(&context_fail),
        Err(Error::CellCountViolation)
    );
}

#[test]
fn test_cell_relationship_validation() {
    let rules = TransactionValidationRules::new(b"update".to_vec()).with_cell_relationship(
        "lock_consistency".to_string(),
        "Ensure lock scripts match".to_string(),
        vec!["vault".to_string()],
        |context| {
            let empty_vec = vec![];
            let inputs = context
                .input_cells
                .get_custom("vault")
                .unwrap_or(&empty_vec);
            let outputs = context
                .output_cells
                .get_custom("vault")
                .unwrap_or(&empty_vec);

            if inputs.is_empty() || outputs.is_empty() {
                return Err(Error::CellRelationshipRuleViolation);
            }

            // Check that lock scripts match
            if inputs[0].lock != outputs[0].lock {
                return Err(Error::CellRelationshipRuleViolation);
            }

            Ok(())
        },
    );

    // Valid - same lock scripts
    let context = create_test_context(
        b"update",
        vec![("custom_vault", 1)],
        vec![("custom_vault", 1)],
    );
    assert!(rules.validate(&context).is_ok());
}

#[test]
fn test_business_rule_validation() {
    let rules = TransactionValidationRules::new(b"mint".to_vec()).with_business_rule(
        "supply_limit".to_string(),
        "Check total supply doesn't exceed limit".to_string(),
        vec!["xudt".to_string()],
        |context| {
            let empty_vec = vec![];
            let outputs = context.output_cells.get_known("xudt").unwrap_or(&empty_vec);

            // Simulate checking total supply
            if outputs.len() > 5 {
                return Err(Error::BusinessRuleViolation);
            }

            Ok(())
        },
    );

    // Valid - within limit
    let context = create_test_context(b"mint", vec![], vec![("xudt", 3)]);
    assert!(rules.validate(&context).is_ok());

    // Invalid - exceeds limit
    let context_fail = create_test_context(b"mint", vec![], vec![("xudt", 6)]);
    assert_eq!(
        rules.validate(&context_fail),
        Err(Error::BusinessRuleViolation)
    );
}

#[test]
fn test_multiple_validation_rules() {
    let rules: TransactionValidationRules<RuleBasedClassifier> =
        TransactionValidationRules::new(b"complex".to_vec())
            .with_arguments(2)
            .with_cell_relationship(
                "capacity_check".to_string(),
                "Ensure capacity is preserved".to_string(),
                vec!["simple_ckb".to_string()],
                |_context| Ok(()),
            )
            .with_business_rule(
                "fee_check".to_string(),
                "Ensure sufficient fee".to_string(),
                vec!["simple_ckb".to_string()],
                |_context| Ok(()),
            );

    // Create context with all requirements met
    let recipe = TransactionRecipeBuilder::default()
        .method_path(Bytes::from(b"complex".to_vec()))
        .arguments(
            RecipeArgumentVec::new_builder()
                .push(create_inline_argument(&vec![1]))
                .push(create_inline_argument(&vec![2]))
                .build(),
        )
        .build();

    let input_cells = ClassifiedCells {
        simple_ckb_cells: vec![create_test_cell_info("simple_ckb", None)],
        known_cells: BTreeMap::new(),
        custom_cells: BTreeMap::new(),
        unidentified_cells: vec![],
    };

    let output_cells = ClassifiedCells {
        simple_ckb_cells: vec![create_test_cell_info("simple_ckb", None)],
        known_cells: BTreeMap::new(),
        custom_cells: BTreeMap::new(),
        unidentified_cells: vec![],
    };

    let context = TransactionContext::from_parts(
        recipe,
        input_cells,
        output_cells,
        ClassifiedCells::default(),
        vec![],
    );

    assert!(rules.validate(&context).is_ok());
}

#[test]
fn test_range_constraints() {
    let rules = TransactionValidationRules::new(b"test".to_vec()).with_custom_cell(
        "token",
        CellCountConstraint::range(2, 5),
        CellCountConstraint::range(1, 3),
    );

    // Valid range
    let context = create_test_context(
        b"test",
        vec![("custom_token", 3)],
        vec![("custom_token", 2)],
    );
    assert!(rules.validate(&context).is_ok());

    // Too few inputs
    let context_few = create_test_context(
        b"test",
        vec![("custom_token", 1)],
        vec![("custom_token", 2)],
    );
    assert_eq!(rules.validate(&context_few), Err(Error::CellCountViolation));

    // Too many outputs
    let context_many = create_test_context(
        b"test",
        vec![("custom_token", 3)],
        vec![("custom_token", 4)],
    );
    assert_eq!(
        rules.validate(&context_many),
        Err(Error::CellCountViolation)
    );
}
