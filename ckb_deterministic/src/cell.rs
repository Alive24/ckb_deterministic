use ckb_std::high_level::{load_cell_data, load_cell_lock, load_cell_type};
use ckb_std::{ckb_constants::Source, ckb_types::packed::Script};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_molecule::{from_slice, to_vec};

use crate::recipes::RecipeCellDefinition;

pub struct RecipeCell<T: MoleculeCellData> {
    pub cell_definition: RecipeCellDefinition,
    pub cell_source: Source,
    pub cell_index: usize,
    pub cell_lock: Script,
    pub cell_type: Option<Script>,
    pub cell_data: T::MoleculeCellDataType,
}

pub trait MoleculeCellData {
    type MoleculeCellDataType: Serialize + DeserializeOwned;
    const RECIPE_CELL_DEFINITION: &RecipeCellDefinition;
}

impl<T: MoleculeCellData> RecipeCell<T> {
    pub fn parse_recipe_cell(index: usize, source: Source) -> Result<Self, Error> {
        let data = load_cell_data(index, source)?;
        let cell_data =
            from_slice::<T::MoleculeCellDataType>(&data, false).map_err(|_| Error::DataError)?;

        let cell_lock = load_cell_lock(index, source)?;
        let cell_type = load_cell_type(index, source);

        Ok(RecipeCell {
            cell_definition: T::RECIPE_CELL_DEFINITION,
            cell_data,
            cell_source: source,
            cell_index: index,
            cell_lock,
            cell_type,
        })
    }
}
