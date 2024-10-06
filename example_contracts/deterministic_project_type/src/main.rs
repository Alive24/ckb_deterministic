#![no_std]
#![cfg_attr(not(test), no_main)]

#[cfg(test)]
extern crate alloc;

#[cfg(not(test))]
use ckb_std::default_alloc;
#[cfg(not(test))]
ckb_std::entry!(program_entry);
#[cfg(not(test))]
default_alloc!();

pub struct DeterministicCDP;

impl Deterministic for DeterministicCDP {
    const RECIPES_DEFINITIONS: Vec<RecipeDefinition> = vec![OPEN_VAULT];
    fn get_output_data(&self) -> Vec<Vec<u8>> {
        implement!();
    }
}

pub fn program_entry() -> i8 {
    let cdp = DeterministicCDP;
    let (recipe, inputs) = cdp.parse_transaction().unwrap();
    let (outputs, outputs_data) = calculate(recipe, inputs).unwrap();
    verify(recipe, outputs, outputs_data).unwrap()
}
