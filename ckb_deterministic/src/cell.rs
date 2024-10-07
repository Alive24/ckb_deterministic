pub struct RecipeCell {
  cell_definition: RecipeCellDefinition,
  cell_source: Source,
  cell_index: u8,
  cell_lock: Script,
  cell_type: Option<Script>,
  cell_data: Vec<u8>
}

pub trait MoleculeCellData {
  type MoleculeCellDataType;
}