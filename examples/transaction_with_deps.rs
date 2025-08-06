/// Example: Working with CellDep and HeaderDep in CKB Deterministic
/// 
/// This example demonstrates how to create and validate transactions that include
/// cell dependencies and header dependencies using the ckb_deterministic framework.

use ckb_deterministic::{
    transaction_recipe::{
        create_transaction_recipe_with_deps, RecipeParams, CellDepParams,
        TransactionRecipeExt,
    },
    transaction_context::{TransactionContext, create_transaction_context},
    cell_classifier::{RuleBasedClassifier, CellCollector, CellClass},
    transaction_deps::{validate_known_script_deps, DepType},
    known_scripts::{KnownScript, get_script_info, Network},
};
use ckb_std::ckb_types::packed::Script;

/// Example: Creating a transaction that uses xUDT with proper cell deps
fn example_xudt_transfer_with_deps() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Create arguments for the transfer
    let recipient = b"ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqvglkprurm00l7hrs3rfqmmzyy3ll7djdsujdm6";
    let amount = 1000u128;
    
    let arguments = vec![
        recipient.to_vec(),
        amount.to_le_bytes().to_vec(),
    ];
    
    // Step 2: Get xUDT script info for mainnet
    let xudt_info = get_script_info(KnownScript::XUdt, Network::Mainnet)
        .ok_or("xUDT script info not found")?;
    
    // Step 3: Create cell deps from script info
    let mut cell_deps = Vec::new();
    for (tx_hash_str, index, dep_type) in &xudt_info.cell_deps {
        let tx_hash = hex_to_bytes(tx_hash_str)?;
        cell_deps.push(CellDepParams {
            tx_hash,
            index: *index,
            dep_type: *dep_type,
        });
    }
    
    // Step 4: Create example header deps (block hashes)
    let header_deps = vec![
        [0x12u8; 32], // Example block hash 1
        [0x34u8; 32], // Example block hash 2
    ];
    
    // Step 5: Create transaction recipe with dependencies
    let params = RecipeParams {
        method_name: "xUDT.transfer",
        arguments: &arguments,
        cell_deps: Some(&cell_deps),
        header_deps: Some(&header_deps),
    };
    
    let recipe = create_transaction_recipe_with_deps(params)?;
    
    // Step 6: Verify the recipe was created correctly
    println!("Created recipe for: {}", recipe.method_path_name()?);
    println!("Arguments count: {}", recipe.arguments_vec().len());
    
    match recipe.cell_deps().to_opt() {
        Some(deps) => println!("Cell deps count: {}", deps.len()),
        None => {}
    }
    
    match recipe.header_deps().to_opt() {
        Some(deps) => println!("Header deps count: {}", deps.len()),
        None => {}
    }
    
    Ok(())
}

/// Example: Validating cell dependencies in a transaction context
fn example_validate_cell_deps() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Create a classifier that recognizes xUDT cells
    let xudt_type_hash = hex_to_bytes("0x50bd8d6680b8b9cf98b73f3c08faf8b2a21914311954118ad6609be6e78a1b95")?;
    
    let classifier = RuleBasedClassifier::new("xUDT Example")
        .add_type_hash(xudt_type_hash, CellClass::known("xudt"));
    
    // Step 2: Create cell collector
    let collector = CellCollector::new(classifier);
    
    // Step 3: Create transaction context (would load from actual transaction)
    // In a real contract, this would parse the transaction and collect cells
    let context = create_transaction_context(collector)?;
    
    // Step 4: Validate that required cell deps are present
    // Get the xUDT script that we expect to find
    let xudt_info = get_script_info(KnownScript::XUdt, Network::Mainnet)
        .ok_or("xUDT script info not found")?;
    
    // Convert script info to expected format
    let required_deps: Vec<(String, u32, DepType)> = xudt_info.cell_deps.iter()
        .map(|(hash, index, dep_type)| {
            (hash.to_string(), *index, DepType::from(*dep_type))
        })
        .collect();
    
    // Create a dummy script for validation
    let xudt_script = Script::default(); // In real usage, this would be the actual script
    
    // Validate deps
    let required_scripts = vec![(xudt_script, required_deps)];
    validate_known_script_deps(&context.cell_deps, &required_scripts)?;
    
    println!("Cell dependencies validated successfully!");
    
    Ok(())
}

/// Example: Working with dep groups
fn example_dep_group_expansion() -> Result<(), Box<dyn std::error::Error>> {
    // Secp256k1 uses a dep group on mainnet
    let secp_info = get_script_info(KnownScript::Secp256k1Blake160, Network::Mainnet)
        .ok_or("Secp256k1 script info not found")?;
    
    println!("Secp256k1 Blake160 Script Info:");
    println!("  Code Hash: {}", secp_info.code_hash);
    println!("  Hash Type: {}", secp_info.hash_type);
    
    for (tx_hash, index, dep_type) in &secp_info.cell_deps {
        let dep_type_str = match dep_type {
            0 => "code",
            1 => "dep_group",
            _ => "unknown",
        };
        println!("  Cell Dep: {} @ {} ({})", tx_hash, index, dep_type_str);
    }
    
    // Note: In a real transaction, when dep_type is 1 (dep_group),
    // the CKB VM would expand this to include all cells referenced
    // by the dep group cell's data.
    
    Ok(())
}

/// Helper function to convert hex string to bytes
fn hex_to_bytes(hex: &str) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let hex = hex.trim_start_matches("0x");
    let bytes = hex::decode(hex)?;
    let array: [u8; 32] = bytes.try_into()
        .map_err(|_| "Invalid hex length")?;
    Ok(array)
}

/// Example: Complete transaction with dependencies
fn example_complete_transaction() -> Result<(), Box<dyn std::error::Error>> {
    // This example shows how to structure a complete transaction recipe
    // for a UDT transfer that requires specific cell deps
    
    // Step 1: Define the transaction parameters
    let from_lock_hash = [0x11u8; 32];
    let to_lock_hash = [0x22u8; 32];
    let amount = 5000u128;
    
    // Step 2: Create cell deps for UDT type script
    let udt_cell_deps = vec![
        CellDepParams {
            tx_hash: hex_to_bytes("0xc07844ce21b38e4b071dd0e1ee3b0e27afd8d7532491327f39b786343f558ab7")?,
            index: 0,
            dep_type: 0, // code
        },
    ];
    
    // Step 3: Create cell deps for lock scripts (e.g., secp256k1)
    let lock_cell_deps = vec![
        CellDepParams {
            tx_hash: hex_to_bytes("0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c")?,
            index: 0,
            dep_type: 1, // dep_group
        },
    ];
    
    // Combine all cell deps
    let mut all_cell_deps = udt_cell_deps;
    all_cell_deps.extend(lock_cell_deps);
    
    // Step 4: Create the recipe
    let params = RecipeParams {
        method_name: "UDT.transfer",
        arguments: &vec![
            from_lock_hash.to_vec(),
            to_lock_hash.to_vec(),
            amount.to_le_bytes().to_vec(),
        ],
        cell_deps: Some(&all_cell_deps),
        header_deps: None, // No header deps needed for simple transfer
    };
    
    let recipe = create_transaction_recipe_with_deps(params)?;
    
    // Step 5: Display the complete recipe information
    println!("Transaction Recipe Summary:");
    println!("  Method: {}", recipe.method_path_name()?);
    println!("  Method Hash: 0x{:016x}", recipe.method_path_hash()?);
    
    let args = recipe.arguments_vec();
    println!("  Arguments: {} total", args.len());
    for (i, arg) in args.iter().enumerate() {
        println!("    [{}]: {} bytes", i, arg.len());
    }
    
    match recipe.cell_deps().to_opt() {
        Some(deps) => {
            println!("  Cell Dependencies: {}", deps.len());
            use ckb_deterministic::transaction_deps::CellDepExt;
            for i in 0..deps.len() {
                match deps.get(i) {
                    Some(dep) => {
                        let dep_type = if dep.is_code() { "code" } else { "dep_group" };
                        println!("    [{}]: type={}", i, dep_type);
                    }
                    None => {}
                }
            }
        }
        None => {}
    }
    
    Ok(())
}

fn main() {
    println!("=== Example 1: xUDT Transfer with Dependencies ===");
    match example_xudt_transfer_with_deps() {
        Err(e) => eprintln!("Error: {}", e),
        Ok(_) => {}
    }
    
    println!("\n=== Example 2: Dep Group Information ===");
    match example_dep_group_expansion() {
        Err(e) => eprintln!("Error: {}", e),
        Ok(_) => {}
    }
    
    println!("\n=== Example 3: Complete Transaction Recipe ===");
    match example_complete_transaction() {
        Err(e) => eprintln!("Error: {}", e),
        Ok(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hex_conversion() {
        let hex = "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let result = hex_to_bytes(hex);
        assert!(result.is_ok());
        
        let bytes = result.unwrap();
        assert_eq!(bytes[0], 0x01);
        assert_eq!(bytes[1], 0x23);
        assert_eq!(bytes[31], 0xef);
    }
}