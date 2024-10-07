use ckb_std::ckb_types::packed::{CellInput, CellOutput};
use proc_macro::TokenStream;

use crate::recipes::{RecipeCell, RecipeDefinition};

pub trait Deterministic {
    const RECIPES_DEFINITIONS: Vec<RecipeDefinition>;
    pub fn parse_transaction(&self) -> Result<(Recipe, Vec<RecipeCell>), Error> {
        implement!();
    }
    pub fn get_outputs(&self) -> Vec<CellOutput> {
        implement!();
    }
    pub fn verify(&self) -> i8 {
        implement!();
    }
    pub fn get_outputs_data(&self) -> Vec<Vec<u8>>;
}

