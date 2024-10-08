use crate::{data::PoolData, definitions::POOL};
use ckb_deterministic::{cell::MoleculeCellData, recipes::RecipeCellDefinition};
pub struct Pool;

impl MoleculeCellData for Pool {
    type MoleculeCellDataType = PoolData;
    const RECIPE_CELL_DEFINITION: &RecipeCellDefinition = &POOL;
}
