#![cfg_attr(not(any(feature = "library", test)), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(any(feature = "library", test))]
extern crate alloc;

use ckb_deterministic::known_scripts::Network;
use ckb_deterministic::cell_classifier::CellCollector;
use ckb_deterministic::transaction_context::TransactionContext;
use ckb_deterministic::transaction_recipe::TransactionRecipeExt;
use ckb_deterministic::{debug_info, debug_error, debug_trace};

#[cfg(not(any(feature = "library", test)))]
ckb_std::entry!(program_entry);
#[cfg(not(any(feature = "library", test)))]
// By default, the following heap configuration is used:
// * 16KB fixed heap
// * 1.2MB(rounded up to be 16-byte aligned) dynamic heap
// * Minimal memory block in dynamic heap is 64 bytes
// For more details, please refer to ckb-std's default_alloc macro
// and the buddy-alloc alloc implementation.
ckb_std::default_alloc!(16384, 1258306, 64);

mod recipes;

use deterministic_cdp_shared::create_cdp_classifier;
use deterministic_cdp_shared::Error;

// Contract identifier for enhanced debug output
const CONTRACT_NAME: &str = "CDP_VAULT";

#[cfg(any(feature = "library", test))]
fn main() -> i8 {
    match program_entry_wrap() {
        Ok(_) => 0,
        Err(e) => e as i8,
    }
}

fn program_entry_wrap() -> Result<(), Error> {
    debug_trace!("ENTER program_entry_wrap");
    
    // Get code hashes from environment or use defaults for testing
    // In production, these would be loaded from protocol cells
    let vault_code_hash = [1u8; 32]; // Example vault code hash
    let protocol_code_hash = [2u8; 32]; // Example protocol code hash
    let pool_code_hash = [3u8; 32]; // Example pool code hash
    
    // Create CDP-specific cell classifier
    let classifier = create_cdp_classifier(
        vault_code_hash,
        protocol_code_hash,
        pool_code_hash,
        Network::Testnet,
    )?;
    
    // Create cell collector and transaction context
    let collector = CellCollector::new(classifier).with_strict_mode(true);
    let context = TransactionContext::new(collector)?;
    
    // Get method path from recipe to determine which validation rules to apply
    let method_path = context.recipe.method_path_bytes();
    
    // Log the method path for debugging
    debug_info!("method_path" => &method_path);
    
    // Apply the appropriate validation rules based on method path
    match method_path.as_slice() {
        b"openVault" => {
            debug_info!(CONTRACT_NAME, "Validating openVault transaction");
            recipes::open_vault::get_rules().validate(&context)?;
        }
        b"closeVault" => {
            debug_info!("Validating closeVault transaction");
            recipes::close_vault::get_rules().validate(&context)?;
        }
        b"adjustVault" => {
            debug_info!("Validating adjustVault transaction");
            recipes::adjust_vault::get_rules().validate(&context)?;
        }
        _ => {
            debug_error!("Unknown method path: {:?}", method_path);
            return Err(Error::WrongMethodPath);
        }
    }
    
    debug_info!("Transaction validation passed");
    
    Ok(())
}

pub fn program_entry() -> i8 {
    match program_entry_wrap() {
        Ok(_) => 0,
        Err(err) => {
            debug_error!("Contract execution failed: {:?}", err);
            err as i8
        }
    }
}

// Panic handler for no_std environment
#[cfg(not(any(feature = "library", test)))]
#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    ckb_std::syscalls::exit(-1)
}