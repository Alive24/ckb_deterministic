use ckb_std::ckb_constants::Source;
extern crate alloc;
use alloc::vec::Vec;

pub struct RecipeDefinition {
    pub recipe_name: &'static str,
    pub description: &'static str,
    pub intent_definition: RecipeCellDefinition,
    pub input_definition: &'static [RecipeCellDefinition],
    pub output_definition: &'static [RecipeCellDefinition],
    pub dep_definition: &'static [RecipeCellDefinition],
    pub recipe_configs: &'static [RecipeConfig],
}

#[derive(Clone)]
pub struct RecipeCellDefinition {
    pub cell_name: &'static str,
    pub cell_type: RecipeCellType,
    pub cell_description: &'static str,
    pub cell_configs: &'static [RecipeCellConfig],
}

#[derive(Clone)]
pub enum RecipeCellType {
    Intent,
    Protocol,
    User,
    Public,
    External
}

#[derive(Clone)]
pub enum RecipeCellConfig {
    Preset(&'static [RecipeCellConfig]),
    Fungible(bool),
    Overridable(bool),
    ExternalTypeHash(Vec<Vec<u8>>),
    ExternalTypeCodeHash(Vec<Vec<u8>>),
    ExternalLockHash(Vec<Vec<u8>>),
    ExternalLockCodeHash(Vec<Vec<u8>>),
}

#[derive(Clone)]
pub enum RecipeConfig {
    Count((RecipeCellDefinition, Source, u8)),
    CountDelta(RecipeCellDefinition, i8),
}
