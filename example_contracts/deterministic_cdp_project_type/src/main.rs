#![no_std]
#![cfg_attr(not(test), no_main)]

#[cfg(test)]
extern crate alloc;
use alloc::vec::Vec;
#[cfg(not(test))]
use ckb_std::default_alloc;
#[cfg(not(test))]
ckb_std::entry!(program_entry);
#[cfg(not(test))]
default_alloc!();

use ckb_deterministic::project::Deterministic;
use ckb_deterministic::recipes::{RecipeCellDefinition, RecipeDefinition};
use deterministic_cdp::definitions::{OPEN_VAULT, POOL};


pub struct DeterministicCDP;

impl Deterministic for DeterministicCDP {
    const RECIPES_DEFINITIONS: Vec<RecipeDefinition> = vec![OPEN_VAULT];

    fn get_outputs_data(&self) -> Vec<Vec<u8>> {
        todo!()
    }
}

pub fn program_entry() -> i8 {
    let cdp = DeterministicCDP;
    let (recipe, inputs) = cdp.parse_transaction().unwrap();
    let (outputs, outputs_data) = cdp.get_output_data(recipe, inputs).unwrap();
    verify(recipe, outputs, outputs_data).unwrap()
}
