use ckb_deterministic::recipes::{RecipeCellDefinition, RecipeDefinition};
use ckb_deterministic::project::Deterministic;

pub const OPEN_VAULT: RecipeDefinition = {
  RecipeDefinition {
    recipe_name: "OpenVault".to_string(),
    description: "OpenVault".to_string(),
    input_definition: vec![POOL],
    output_definition: vec![POOL],
    dep_definition: vec![]
  }
};

pub const POOL: RecipeCellDefinition = {
  RecipeCellDefinition {
    cell_name: "Pool".to_string(),
    cell_type: RecipeCellType::User,
    cell_description: Some("Pool".to_string()),
    cell_configs: vec![RecipeCellConfig::Fungible(true), RecipeCellConfig::Overridable(true)],
    _type: None,
    lock: None,
    data_type: CellDataTypes::Bytes
  }
};