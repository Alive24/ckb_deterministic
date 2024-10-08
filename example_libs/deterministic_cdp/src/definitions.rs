use alloc::string::ToString;
use alloc::vec;
use ckb_deterministic::project::Deterministic;
use ckb_deterministic::recipes::{
    RecipeCellConfig, RecipeCellDefinition, RecipeCellType, RecipeDefinition,
};

pub const OPEN_VAULT: RecipeDefinition = {
    RecipeDefinition {
        recipe_name: "OpenVault",
        description: "OpenVault",
        intent_definition: INTENT,
        input_definition: &[POOL],
        output_definition: &[POOL],
        dep_definition: &[],
        recipe_configs: &[],
    }
};

pub const INTENT: RecipeCellDefinition = {
    RecipeCellDefinition {
        cell_name: "Intent",
        cell_type: RecipeCellType::Intent,
        cell_description: "Intent",
        cell_configs: &vec![],
    }
};

pub const POOL: RecipeCellDefinition = {
    RecipeCellDefinition {
        cell_name: "Pool",
        cell_type: RecipeCellType::Public,
        cell_description: "Pool",
        cell_configs: &[
            RecipeCellConfig::Fungible(true),
            RecipeCellConfig::Overridable(true),
        ],
    }
};

pub const SUPPORTED_XUDT_ASSETS: RecipeCellConfig::Preset<Vec<RecipeCellConfig>> =
    RecipeCellConfig::Preset([
        RecipeCellConfig::Fungible(true),
        RecipeCellConfig::ExternalTypeHash([]),
    ]);
