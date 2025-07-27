#![no_std]

#[cfg(test)]
#[macro_use]
extern crate std;

pub mod assertions;
pub mod cell_classifier;
pub mod errors;
pub mod known_scripts;
pub mod generated;
pub mod transaction_recipe;
pub mod transaction_context;
pub mod transaction_deps;
pub mod validation;
pub mod type_id;

// Re-export commonly used helper functions from transaction_recipe
pub use transaction_recipe::{
    create_inline_argument,
    create_output_data_reference,
    create_input_data_reference,
    create_cell_dep_data_reference,
    create_header_reference,
    create_recipe_with_reference,
    create_recipe_with_args,
    create_recipe_flexible,
    serialize_transaction_recipe,
    resolve_recipe_argument,
};

