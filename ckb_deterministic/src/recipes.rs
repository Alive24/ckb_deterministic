use ckb_std::ckb_constants::Source;

pub struct RecipeDefinition {
    pub recipe_name: &'static str,
    pub description: &'static str,
    pub intent_definition: RecipeCellDefinition,
    pub input_definition: &'static [RecipeCellDefinition],
    pub output_definition: &'static [RecipeCellDefinition],
    pub dep_definition: &'static [RecipeCellDefinition],
    pub recipe_configs: &'static [RecipeConfig],
}

pub struct RecipeCellDefinition {
    pub cell_name: &'static str,
    pub cell_type: RecipeCellType,
    pub cell_description: &'static str,
    pub cell_configs: &'static [RecipeCellConfig],
}

pub enum RecipeCellType {
    Intent,
    Protocol,
    User,
    Public,
    External
}

pub enum RecipeCellConfig {
    Preset([RecipeCellConfig]),
    Fungible(bool),
    Overridable(bool),
    ExternalTypeHash(Vec<Vec<u8>>),
    ExternalTypeCodeHash(Vec<Vec<u8>>),
    ExternalLockHash(Vec<Vec<u8>>),
    ExternalLockCodeHash(Vec<Vec<u8>>),
}

pub enum RecipeConfig {
    Count((RecipeCellDefinition, Source, u8)),
    CountDelta(RecipeCellDefinition, i8),
}
