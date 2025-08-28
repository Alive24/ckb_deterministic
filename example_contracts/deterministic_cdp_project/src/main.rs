//! # Deterministic CDP Smart Contract
//!
//! This is the main entry point for the Deterministic CDP (Collateralized Debt Position) 
//! smart contract that runs on the Nervos CKB blockchain.
//!
//! ## Contract Overview
//!
//! The CDP contract implements a decentralized lending protocol that allows users to:
//! - Create collateralized debt positions (vaults) by depositing collateral assets
//! - Borrow against their collateral at a specified collateralization ratio
//! - Manage their positions by adjusting collateral and debt levels
//! - Close positions by repaying debt and reclaiming collateral
//!
//! ## Transaction Types
//!
//! The contract validates three primary transaction types:
//! - **`openVault`** - Creates a new vault with initial collateral and debt
//! - **`closeVault`** - Closes an existing vault by repaying all debt
//! - **`adjustVault`** - Modifies collateral or debt amounts in an existing vault
//!
//! ## Validation Framework
//!
//! The contract uses the `ckb_deterministic` framework to provide:
//! - Structured validation rules for each transaction type
//! - Cell classification and type safety
//! - Business rule enforcement (collateralization ratios, etc.)
//! - Deterministic transaction context analysis
//!
//! ## Build Configurations
//!
//! - **Default build**: Produces a no_std binary for on-chain execution
//! - **Library feature**: Enables std library for testing and development
//! - **Native simulator**: Allows running with CKB simulator for debugging

#![cfg_attr(not(any(feature = "library", test)), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(any(feature = "library", test))]
extern crate alloc;

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

/// Contract identifier used in debug output and logging
/// 
/// This constant helps identify log messages and debug output as originating
/// from the CDP vault contract, making debugging and monitoring easier.
const CONTRACT_NAME: &str = "CDP_VAULT";

/// Main function for library and test builds
/// 
/// When compiled with the "library" or "test" features, this function serves as the
/// standard entry point. It wraps the core contract logic in error handling and
/// returns appropriate exit codes.
/// 
/// # Returns
/// - `0` if the transaction validation succeeds
/// - Error code (as i8) if validation fails
#[cfg(any(feature = "library", test))]
fn main() -> i8 {
    match program_entry_wrap() {
        Ok(_) => 0,
        Err(e) => e as i8,
    }
}

/// Core contract validation logic
/// 
/// This function implements the main validation workflow for CDP transactions:
/// 1. Sets up the cell classification system to identify CDP-specific cells
/// 2. Creates a transaction context for deterministic validation
/// 3. Routes to appropriate validation rules based on the method path
/// 4. Executes validation and returns results
/// 
/// # Process Flow
/// 
/// 1. **Cell Classifier Setup**: Creates a classifier that can identify:
///    - CDP vault cells (containing collateral and debt data)
///    - CDP protocol cells (containing system parameters)  
///    - CDP pool cells (for liquidity management)
///    - Standard xUDT token cells
/// 
/// 2. **Transaction Context**: Builds a complete view of the transaction including:
///    - Input and output cells, classified by type
///    - Cell dependencies and header dependencies
///    - Recipe method path and arguments
/// 
/// 3. **Method Routing**: Determines which CDP operation is being performed:
///    - `openVault` - Create new vault with collateral and debt
///    - `closeVault` - Close existing vault by repaying debt
///    - `adjustVault` - Modify existing vault collateral/debt levels
/// 
/// 4. **Validation**: Applies operation-specific validation rules including:
///    - Cell count constraints (right number of inputs/outputs)
///    - Business rules (collateralization ratios, debt limits)
///    - Cell relationship rules (owner continuity, etc.)
/// 
/// # Returns
/// - `Ok(())` if all validation rules pass
/// - `Err(Error)` with specific error type if validation fails
/// 
/// # Notes
/// - Code hashes are currently hardcoded for testing; in production these would
///   be loaded from protocol configuration cells
/// - Uses testnet configuration; production would use mainnet parameters
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

/// Main entry point for on-chain execution
/// 
/// This is the primary entry point when the contract runs on the CKB blockchain.
/// It wraps the core validation logic with proper error handling and logging.
/// 
/// # Returns
/// - `0` if transaction validation succeeds
/// - Non-zero error code if validation fails (specific to the error type)
/// 
/// # Error Handling
/// All errors are logged for debugging purposes before returning error codes.
/// The error codes correspond to the variants defined in the Error enum.
pub fn program_entry() -> i8 {
    match program_entry_wrap() {
        Ok(_) => 0,
        Err(err) => {
            debug_error!("Contract execution failed: {:?}", err);
            err as i8
        }
    }
}

/// Panic handler for no_std on-chain environment
/// 
/// In the no_std environment when running on-chain, this handler is required
/// to handle any panic situations. It immediately exits the contract with
/// error code -1 to signal a critical failure.
/// 
/// # Parameters
/// - `_info`: Panic information (unused in minimal handler)
/// 
/// # Behavior
/// - Calls `ckb_std::syscalls::exit(-1)` to terminate contract execution
/// - Never returns (marked with `!` return type)
#[cfg(not(any(feature = "library", test)))]
#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    ckb_std::syscalls::exit(-1)
}