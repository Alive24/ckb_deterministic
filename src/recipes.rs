pub struct Recipe {
    recipe_name: String,
    description: String,
    input_definition: Vec<RecipeCellDefinition>,
    output_definition: Vec<RecipeCellDefinition>,
    dep_definition: Vec<RecipeCellDefinition>,
    input_cells: Option<Vec<CellInput>>,
}

trait DeterministicRecipe {
    pub fn get_output_data(&self) -> Vec<Vec<u8>>;
    pub fn parse_input(&self) -> i8 {
        implement!();
    }
    pub fn verify_data_hash(&self) -> i8 {
        implement!();
    }
}

pub struct RecipeCellDefinition {
    cell_name: String,
    cell_type: RecipeCellType,
    cell_description: Option<String>,
    cell_configs: Vec<RecipeCellConfig>,
    _type: Option<Script>,
    lock: Option<Script>,
    data_type: CellDataTypes
}

pub struct RecipeCell {
    cell_definition: RecipeCellDefinition,
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
    Count(u8),
    CountDelta(i8),
    AllowedInSource(Source)
}