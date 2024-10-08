use ckb_std::ckb_types::packed::{CellInput, CellOutput};
use proc_macro::TokenStream;

use crate::recipes::{RecipeCell, RecipeDefinition};

pub trait Deterministic {
    const RECIPES_DEFINITIONS: &'static [RecipeDefinition];
    fn parse_transaction(&self) -> Result<(Recipe, Vec<RecipeCell>), Error> {
        todo!();
    }
    fn get_outputs(&self) -> Vec<CellOutput> {
        todo!();
    }
    fn verify(&self) -> i8 {
        todo!();
    }
    fn get_outputs_data(&self) -> Vec<Vec<u8>>;
}

