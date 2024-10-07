use alloc::string::ToString;
use ckb_deterministic::recipes::{RecipeCellDefinition, RecipeDefinition, RecipeCellType};
use ckb_deterministic::project::Deterministic;

pub const OPEN_VAULT: RecipeDefinition = {
  RecipeDefinition {
    recipe_name: "OpenVault".to_string(),
    description: "OpenVault".to_string(),
    input_definition: vec![POOL],
    output_definition: vec![POOL],
    dep_definition: vec![],
    recipe_configs: vec![],
  }
};

pub const POOL: RecipeCellDefinition = {
  RecipeCellDefinition {
    cell_name: "Pool".to_string(),
    cell_type: RecipeCellType::Public,
    cell_description: Some("Pool".to_string()),
    cell_configs: vec![RecipeCellConfig::Fungible(true), RecipeCellConfig::Overridable(true)],
  }
};