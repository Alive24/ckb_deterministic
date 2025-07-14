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

use ckb_std::debug;
use ckb_deterministic::transaction_recipe::TransactionRecipeExt;
use deterministic_cdp_shared::{
    transaction_context::{create_cdp_transaction_context, CDPProject},
    transaction_recipe::{
        create_open_vault_recipe, create_close_vault_recipe,
        parse_collateral_amount, parse_vault_id,
        CDP_OPEN_VAULT_METHOD, CDP_CLOSE_VAULT_METHOD
    },
    Error
};

pub fn run() -> Result<(), Error> {
    debug!("Starting CDP contract execution");
    
    // Create CDP-specific transaction context
    let context = create_cdp_transaction_context()?;
    
    // Log transaction details
    debug!("Method: {}", context.method_path_name);
    debug!("Hash: {}", context.method_path_hash);
    debug!("Arguments: {}", context.arguments.len());
    debug!("Input cells: {}", context.input_cells.total_cell_count());
    debug!("Output cells: {}", context.output_cells.total_cell_count());
    
    // Demonstrate transaction recipe creation and validation
    demonstrate_recipe_creation()?;
    
    // Validate transaction context using ckb_deterministic validation framework
    context.validate()
        .map_err(|_| Error::InvalidArguments)?;
    
    // Create CDP project instance and process transaction
    let cdp_project = CDPProject::new();
    cdp_project.process_transaction(&context)?;
    
    debug!("CDP contract execution completed successfully");
    Ok(())
}

/// Demonstrate transaction recipe creation and parsing
pub fn demonstrate_recipe_creation() -> Result<(), Error> {
    debug!("=== Demonstrating CDP Transaction Recipe Creation ===");
    
    // Example 1: Create an open vault recipe
    let collateral_amount = 1000_000_000_000_000_000u128; // 1000 tokens (18 decimals)
    let initial_debt = 500_000_000_000_000_000u128; // 500 stablecoin
    
    let open_vault_recipe = create_open_vault_recipe(collateral_amount, initial_debt)?;
    debug!("Created open vault recipe");
    
    // Parse and verify the recipe
    let parsed_collateral = parse_collateral_amount(&open_vault_recipe)?;
    debug!("Parsed collateral amount: {}", parsed_collateral);
    assert_eq!(parsed_collateral, collateral_amount);
    
    // Example 2: Create a close vault recipe
    let vault_id = 42u64;
    let close_vault_recipe = create_close_vault_recipe(vault_id)?;
    debug!("Created close vault recipe");
    
    let parsed_vault_id = parse_vault_id(&close_vault_recipe)?;
    debug!("Parsed vault ID: {}", parsed_vault_id);
    assert_eq!(parsed_vault_id, vault_id);
    
    // Verify method paths
    assert_eq!(open_vault_recipe.method_path_bytes(), CDP_OPEN_VAULT_METHOD.as_bytes());
    assert_eq!(close_vault_recipe.method_path_bytes(), CDP_CLOSE_VAULT_METHOD.as_bytes());
    
    debug!("Recipe creation and parsing demonstration completed successfully");
    Ok(())
}

pub fn program_entry() -> i8 {
    match run() {
        Ok(()) => 0,
        Err(err) => {
            debug!("Contract execution failed with error: {:?}", err);
            err as i8
        }
    }
}