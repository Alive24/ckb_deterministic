use ckb_std::ckb_constants::Source;

pub struct RecipeDefinition {
    pub recipe_name: String,
    pub description: String,
    pub input_definition: Vec<RecipeCellDefinition>,
    pub output_definition: Vec<RecipeCellDefinition>,
    pub dep_definition: Vec<RecipeCellDefinition>,
}

pub struct RecipeCellDefinition {
    pub cell_name: String,
    pub cell_type: RecipeCellType,
    pub cell_description: Option<String>,
    pub cell_configs: Vec<RecipeCellConfig>,
    pub _type: Option<Script>,
    pub lock: Option<Script>,
    pub data_type: CellDataTypes
}

pub struct DeterministicRecipe {
    recipe_definition: RecipeDefinition,
    input_cells: Vec<RecipeCell>,
    dep_cells: Vec<RecipeCell>,
}

pub struct RecipeCell {
    cell_definition: RecipeCellDefinition,
    cell_source: Source,
    cell_index: u8,
    cell_lock: Script,
    cell_type: Option<Script>,
    cell_data: Vec<u8>
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
    Count((Source, u8)),
    CountDelta(i8),
}