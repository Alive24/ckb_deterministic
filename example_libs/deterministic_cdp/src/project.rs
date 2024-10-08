
use crate::definitions::{OPEN_VAULT, RecipeDefinition};
use alloc::vec;

pub struct DeterministicCDP;

impl Deterministic for DeterministicCDP {
    const RECIPES_DEFINITIONS: Vec<RecipeDefinition> = vec![OPEN_VAULT];
    fn get_outputs_data(&self) -> Vec<Vec<u8>> {
        todo!()
    }
}