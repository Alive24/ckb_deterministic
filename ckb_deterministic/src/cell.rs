use ckb_std::high_level::{load_cell_data, load_cell_lock, load_cell_type};
use ckb_std::{ckb_constants::Source, ckb_types::packed::Script};
use molecule::prelude::Entity;
use core::result::Result;

use crate::recipes::RecipeCellDefinition;
use crate::errors::Error;

pub struct RecipeCell<T: MoleculeCellData> {
    pub cell_definition: RecipeCellDefinition,
    pub cell_source: Source,
    pub cell_index: usize,
    pub cell_lock: Script,
    pub cell_type: Option<Script>,
    pub cell_data: T::MoleculeCellDataType,
}

pub trait MoleculeCellData {
    type MoleculeCellDataType: Entity;
    const RECIPE_CELL_DEFINITION: &'static RecipeCellDefinition;
}

impl<T: MoleculeCellData> RecipeCell<T> {
    pub fn parse_recipe_cell(index: usize, source: Source) -> Result<Self, Error> {
        let data = load_cell_data(index, source)?;
        let cell_data = T::MoleculeCellDataType::from_slice(&data).map_err(|_| Error::DataError)?;

        let cell_lock = load_cell_lock(index, source)?;
        let cell_type = load_cell_type(index, source);

        Ok(RecipeCell {
            cell_definition: T::RECIPE_CELL_DEFINITION.clone(),
            cell_data,
            cell_source: source,
            cell_index: index,
            cell_lock,
            cell_type: cell_type?,
        })
    }
}
