use ckb_deterministic::cell_classifier::{
    CellClass, CellClassifier, CellInfo, ClassificationRule, ClassifiedCells, RuleBasedClassifier,
};
use ckb_deterministic::generated::{Bytes, RecipeArgumentVec, TransactionRecipeBuilder};
use ckb_deterministic::transaction_context::TransactionContext;
use ckb_deterministic::transaction_recipe::create_inline_argument;
use ckb_deterministic::transaction_recipe::TransactionRecipeExt;
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{
        packed::{Script, ScriptBuilder},
        prelude::*,
    },
};
extern crate alloc;
use alloc::{collections::BTreeMap, string::ToString, vec::Vec};

fn create_test_script(code_hash: [u8; 32], args: Vec<u8>) -> Script {
    ScriptBuilder::default()
        .code_hash(code_hash.pack())
        .hash_type(0u8.into())
        .args(args.pack())
        .build()
}

fn create_test_cell(index: usize, type_hash: Option<[u8; 32]>) -> CellInfo {
    CellInfo {
        source: Source::Input,
        index,
        data: vec![],
        lock: create_test_script([1u8; 32], vec![index as u8]),
        lock_hash: [index as u8; 32],
        type_script: type_hash.map(|h| create_test_script(h, vec![])),
        type_hash,
    }
}

#[test]
fn test_context_creation_basic() {
    let recipe = TransactionRecipeBuilder::default()
        .method_path(Bytes::from(b"transfer".to_vec()))
        .arguments(RecipeArgumentVec::new_builder().build())
        .build();

    let context: TransactionContext<RuleBasedClassifier> = TransactionContext::from_parts(
        recipe,
        ClassifiedCells::default(),
        ClassifiedCells::default(),
        ClassifiedCells::default(),
        vec![],
    );

    assert_eq!(context.recipe.method_path_bytes(), b"transfer");
    assert_eq!(context.input_cells.total_cell_count(), 0);
    assert_eq!(context.output_cells.total_cell_count(), 0);
    assert!(context.cell_deps.is_empty());
    assert!(context.header_deps.is_empty());
}

#[test]
fn test_context_with_classified_cells() {
    let classifier = RuleBasedClassifier::new("TestClassifier")
        .add_rule(ClassificationRule::TypeCodeHash {
            code_hash: [10u8; 32],
            class: CellClass::known("xudt"),
        })
        .add_rule(ClassificationRule::TypeCodeHash {
            code_hash: [20u8; 32],
            class: CellClass::custom("vault"),
        });

    // Create various cells
    let simple_ckb = create_test_cell(0, None);
    let xudt_cell = create_test_cell(1, Some([10u8; 32]));
    let vault_cell = create_test_cell(2, Some([20u8; 32]));
    let unknown_cell = create_test_cell(3, Some([99u8; 32]));

    // Classify cells
    let mut input_cells = ClassifiedCells::new();
    for cell in vec![simple_ckb, xudt_cell, vault_cell, unknown_cell] {
        let classification = classifier.classify(&cell).unwrap();
        input_cells.add_cell(cell, classification);
    }

    let recipe = TransactionRecipeBuilder::default()
        .method_path(Bytes::from(b"complex".to_vec()))
        .arguments(
            RecipeArgumentVec::new_builder()
                .push(create_inline_argument(&vec![1, 2, 3]))
                .push(create_inline_argument(&vec![4, 5, 6]))
                .build(),
        )
        .build();

    let context: TransactionContext<RuleBasedClassifier> = TransactionContext::from_parts(
        recipe,
        input_cells,
        ClassifiedCells::default(),
        ClassifiedCells::default(),
        vec![],
    );

    assert_eq!(context.recipe.method_path_bytes(), b"complex");
    assert_eq!(context.recipe.arguments_vec().len(), 2);
    assert_eq!(context.input_cells.get_simple_ckb().len(), 1);
    assert_eq!(
        context.input_cells.get_known("xudt").map(|v| v.len()),
        Some(1)
    );
    assert_eq!(
        context.input_cells.get_custom("vault").map(|v| v.len()),
        Some(1)
    );
    assert!(context.input_cells.has_unidentified_cells());
}

#[test]
fn test_context_with_dependencies() {
    let header_deps = vec![[10u8; 32], [11u8; 32], [12u8; 32]];

    let recipe = TransactionRecipeBuilder::default()
        .method_path(Bytes::from(b"advanced".to_vec()))
        .arguments(RecipeArgumentVec::new_builder().build())
        .build();

    let context: TransactionContext<RuleBasedClassifier> = TransactionContext::from_parts(
        recipe,
        ClassifiedCells::default(),
        ClassifiedCells::default(),
        ClassifiedCells::default(), // cell_deps is now ClassifiedCells
        header_deps,
    );

    assert_eq!(context.cell_deps.total_cell_count(), 0); // ClassifiedCells is empty
    assert_eq!(context.header_deps.len(), 3);
}

#[test]
fn test_context_cell_access_patterns() {
    let mut input_known = BTreeMap::new();
    input_known.insert(
        "xudt".to_string(),
        vec![
            create_test_cell(0, Some([10u8; 32])),
            create_test_cell(1, Some([10u8; 32])),
        ],
    );

    let mut input_custom = BTreeMap::new();
    input_custom.insert(
        "vault".to_string(),
        vec![create_test_cell(2, Some([20u8; 32]))],
    );

    let mut output_known = BTreeMap::new();
    output_known.insert(
        "xudt".to_string(),
        vec![create_test_cell(0, Some([10u8; 32]))],
    );

    let input_cells = ClassifiedCells {
        simple_ckb_cells: vec![create_test_cell(3, None)],
        known_cells: input_known,
        custom_cells: input_custom,
        unidentified_cells: vec![],
    };

    let output_cells = ClassifiedCells {
        simple_ckb_cells: vec![create_test_cell(0, None)],
        known_cells: output_known,
        custom_cells: BTreeMap::new(),
        unidentified_cells: vec![],
    };

    let recipe = TransactionRecipeBuilder::default()
        .method_path(Bytes::from(b"transfer".to_vec()))
        .arguments(RecipeArgumentVec::new_builder().build())
        .build();

    let context: TransactionContext<RuleBasedClassifier> = TransactionContext::from_parts(
        recipe,
        input_cells,
        output_cells,
        ClassifiedCells::default(),
        vec![],
    );

    // Test various access patterns
    assert_eq!(context.input_cells.get_known("xudt").unwrap().len(), 2);
    assert_eq!(context.input_cells.get_custom("vault").unwrap().len(), 1);
    assert_eq!(context.input_cells.get_simple_ckb().len(), 1);

    assert_eq!(context.output_cells.get_known("xudt").unwrap().len(), 1);
    assert!(context.output_cells.get_custom("vault").is_none());
    assert_eq!(context.output_cells.get_simple_ckb().len(), 1);

    // Test summary
    let input_summary = context.input_cells.summary();
    assert_eq!(input_summary.simple_ckb_count, 1);
    assert_eq!(input_summary.known_types.len(), 1);
    assert_eq!(input_summary.custom_types.len(), 1);
    assert_eq!(input_summary.total_count, 4);
}

#[test]
fn test_context_empty_collections() {
    let recipe = TransactionRecipeBuilder::default()
        .method_path(Bytes::from(b"empty".to_vec()))
        .arguments(RecipeArgumentVec::new_builder().build())
        .build();

    let context: TransactionContext<RuleBasedClassifier> = TransactionContext::from_parts(
        recipe,
        ClassifiedCells::default(),
        ClassifiedCells::default(),
        ClassifiedCells::default(),
        vec![],
    );

    assert_eq!(context.input_cells.total_cell_count(), 0);
    assert_eq!(context.output_cells.total_cell_count(), 0);
    assert!(!context.input_cells.has_unidentified_cells());
    assert!(!context.output_cells.has_unidentified_cells());

    // Test accessing non-existent cells
    assert!(context.input_cells.get_known("xudt").is_none());
    assert!(context.input_cells.get_custom("vault").is_none());
    assert_eq!(context.input_cells.get_simple_ckb().len(), 0);
}

#[test]
fn test_context_with_unidentified_cells() {
    let unidentified = vec![
        create_test_cell(0, Some([99u8; 32])),
        create_test_cell(1, Some([100u8; 32])),
    ];

    let input_cells = ClassifiedCells {
        simple_ckb_cells: vec![],
        known_cells: BTreeMap::new(),
        custom_cells: BTreeMap::new(),
        unidentified_cells: unidentified,
    };

    let recipe = TransactionRecipeBuilder::default()
        .method_path(Bytes::from(b"test".to_vec()))
        .arguments(RecipeArgumentVec::new_builder().build())
        .build();

    let context: TransactionContext<RuleBasedClassifier> = TransactionContext::from_parts(
        recipe,
        input_cells,
        ClassifiedCells::default(),
        ClassifiedCells::default(),
        vec![],
    );

    assert!(context.input_cells.has_unidentified_cells());
    assert_eq!(context.input_cells.unidentified_cells.len(), 2);
    assert_eq!(context.input_cells.identified_cell_count(), 0);
    assert_eq!(context.input_cells.total_cell_count(), 2);
}

#[test]
fn test_context_recipe_data() {
    let args = vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8], vec![9, 10, 11, 12]];

    let recipe = TransactionRecipeBuilder::default()
        .method_path(Bytes::from(b"complexMethod".to_vec()))
        .arguments(
            RecipeArgumentVec::new_builder()
                .push(create_inline_argument(&args[0]))
                .push(create_inline_argument(&args[1]))
                .push(create_inline_argument(&args[2]))
                .build(),
        )
        .build();

    let context: TransactionContext<RuleBasedClassifier> = TransactionContext::from_parts(
        recipe,
        ClassifiedCells::default(),
        ClassifiedCells::default(),
        ClassifiedCells::default(),
        vec![],
    );

    assert_eq!(context.recipe.method_path_bytes(), b"complexMethod");
    assert_eq!(context.recipe.arguments_vec().len(), 3);

    let args_vec = context.recipe.arguments_vec();
    assert_eq!(args_vec[0], args[0]);
    assert_eq!(args_vec[1], args[1]);
    assert_eq!(args_vec[2], args[2]);
}

#[test]
fn test_context_large_cell_collections() {
    let mut known = BTreeMap::new();
    let mut custom = BTreeMap::new();

    // Create many cells
    let xudt_cells: Vec<_> = (0..100)
        .map(|i| create_test_cell(i, Some([10u8; 32])))
        .collect();
    let vault_cells: Vec<_> = (100..200)
        .map(|i| create_test_cell(i, Some([20u8; 32])))
        .collect();
    let simple_cells: Vec<_> = (200..250).map(|i| create_test_cell(i, None)).collect();

    known.insert("xudt".to_string(), xudt_cells);
    custom.insert("vault".to_string(), vault_cells);

    let input_cells = ClassifiedCells {
        simple_ckb_cells: simple_cells,
        known_cells: known,
        custom_cells: custom,
        unidentified_cells: vec![],
    };

    let recipe = TransactionRecipeBuilder::default()
        .method_path(Bytes::from(b"bulk".to_vec()))
        .arguments(RecipeArgumentVec::new_builder().build())
        .build();

    let context: TransactionContext<RuleBasedClassifier> = TransactionContext::from_parts(
        recipe,
        input_cells,
        ClassifiedCells::default(),
        ClassifiedCells::default(),
        vec![],
    );

    assert_eq!(context.input_cells.get_known("xudt").unwrap().len(), 100);
    assert_eq!(context.input_cells.get_custom("vault").unwrap().len(), 100);
    assert_eq!(context.input_cells.get_simple_ckb().len(), 50);
    assert_eq!(context.input_cells.total_cell_count(), 250);
}

#[test]
fn test_context_mixed_sources() {
    // In a real transaction, inputs come from Source::Input and outputs from Source::Output
    let input_cell = CellInfo {
        source: Source::Input,
        index: 0,
        data: vec![],
        lock: create_test_script([1u8; 32], vec![]),
        lock_hash: [0u8; 32],
        type_script: None,
        type_hash: None,
    };

    let output_cell = CellInfo {
        source: Source::Output,
        index: 0,
        data: vec![],
        lock: create_test_script([1u8; 32], vec![]),
        lock_hash: [0u8; 32],
        type_script: None,
        type_hash: None,
    };

    let input_cells = ClassifiedCells {
        simple_ckb_cells: vec![input_cell],
        known_cells: BTreeMap::new(),
        custom_cells: BTreeMap::new(),
        unidentified_cells: vec![],
    };

    let output_cells = ClassifiedCells {
        simple_ckb_cells: vec![output_cell],
        known_cells: BTreeMap::new(),
        custom_cells: BTreeMap::new(),
        unidentified_cells: vec![],
    };

    let recipe = TransactionRecipeBuilder::default()
        .method_path(Bytes::from(b"transfer".to_vec()))
        .arguments(RecipeArgumentVec::new_builder().build())
        .build();

    let context: TransactionContext<RuleBasedClassifier> = TransactionContext::from_parts(
        recipe,
        input_cells,
        output_cells,
        ClassifiedCells::default(),
        vec![],
    );

    assert_eq!(
        context.input_cells.get_simple_ckb()[0].source,
        Source::Input
    );
    assert_eq!(
        context.output_cells.get_simple_ckb()[0].source,
        Source::Output
    );
}
