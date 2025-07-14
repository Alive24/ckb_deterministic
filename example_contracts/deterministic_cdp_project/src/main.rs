#![no_std]
#![cfg_attr(not(test), no_main)]

#[cfg(any(feature = "library", test))]
extern crate alloc;

#[cfg(not(test))]
use ckb_std::default_alloc;
#[cfg(not(test))]
ckb_std::entry!(program_entry);
#[cfg(not(test))]
default_alloc!();

use ckb_std::debug;
use deterministic_cdp_shared::{
    transaction_context::{create_cdp_transaction_context, CDPProject},
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
    
    // Validate transaction context
    context.validate()
        .map_err(|_| Error::InvalidArguments)?;
    
    // Create CDP project instance and process transaction
    let cdp_project = CDPProject::new();
    cdp_project.process_transaction(&context)?;
    
    debug!("CDP contract execution completed successfully");
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