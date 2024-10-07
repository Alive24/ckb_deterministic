use ckb_std::ckb_constants::Source;

pub struct RecipeDefinition {
    pub recipe_name: String,
    pub description: String,
    pub input_definition: Vec<RecipeCellDefinition>,
    pub output_definition: Vec<RecipeCellDefinition>,
    pub dep_definition: Vec<RecipeCellDefinition>,
    pub recipe_configs: Vec<RecipeConfig>,
}

pub struct RecipeCellDefinition {
    pub cell_name: String,
    pub cell_type: RecipeCellType,
    pub cell_description: Option<String>,
    pub cell_configs: Vec<RecipeCellConfig>,
}

pub enum RecipeCellType {
    Protocol,
    User,
    Public,
    External
}

pub enum RecipeCellConfig {
    Fungible(bool),
    Overridable(bool),
    ExternalTypeHash(Vec<u8>),
    ExternalTypeCodeHash(Vec<u8>),
    ExternalLockHash(Vec<u8>),
    ExternalLockCodeHash(Vec<u8>),
}

pub enum RecipeConfig {
    Count((RecipeCellDefinition, Source, u8)),
    CountDelta(RecipeCellDefinition, i8),
}