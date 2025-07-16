use deterministic_cdp_shared::transaction_recipe::*;
use ckb_deterministic::transaction_recipe::TransactionRecipeExt;

#[test]
fn test_create_open_vault_recipe() {
    let collateral = 1000_000_000_000_000_000u128; // 1000 tokens
    let debt = 500_000_000_000_000_000u128; // 500 stablecoin
    
    let recipe = create_open_vault_recipe(collateral, debt).unwrap();
    
    // Verify method path
    assert_eq!(recipe.method_path_bytes(), CDP_OPEN_VAULT_METHOD.as_bytes());
    
    // Verify arguments
    let args = recipe.arguments_vec();
    assert_eq!(args.len(), 2);
    assert_eq!(args[0], collateral.to_le_bytes().to_vec());
    assert_eq!(args[1], debt.to_le_bytes().to_vec());
}

#[test]
fn test_parse_collateral_amount() {
    let collateral = 1500_000_000_000_000_000u128;
    let recipe = create_open_vault_recipe(collateral, 100u128).unwrap();
    
    let parsed = parse_collateral_amount(&recipe).unwrap();
    assert_eq!(parsed, collateral);
}

#[test]
fn test_create_close_vault_recipe() {
    let vault_id = 42u64;
    let recipe = create_close_vault_recipe(vault_id).unwrap();
    
    assert_eq!(recipe.method_path_bytes(), CDP_CLOSE_VAULT_METHOD.as_bytes());
    
    let parsed_id = parse_vault_id(&recipe).unwrap();
    assert_eq!(parsed_id, vault_id);
}