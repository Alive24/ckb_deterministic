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
use deterministic_cdp::project::DeterministicCDP;


pub fn run() -> Result<i8, Error> {
    let cdp = DeterministicCDP;
    let (recipe, inputs) = cdp.parse_transaction()?;
    let (outputs, outputs_data) = cdp.get_outputs_data(recipe, inputs)?;
    Ok(cdp.verify())
}

pub fn program_entry() -> i8 {
    match run() {
        Ok(()) => 0,
        Err(error_code) => error_code,
    }
}
