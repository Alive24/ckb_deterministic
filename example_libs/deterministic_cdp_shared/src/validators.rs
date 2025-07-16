//! CDP transaction validators using Jest-like assertions

extern crate alloc;
use alloc::{format, string::{String, ToString}, vec::Vec};
use ckb_deterministic::{
    generated::TransactionRecipe,
    cell_classifier::ClassifiedCells,
    transaction_recipe::TransactionRecipeExt,
    assertions::*,
    validate_all,
    validation_block,
};

/// Validates open vault transaction using Jest-like assertions
pub fn validate_open_vault_transaction(
    recipe: &TransactionRecipe,
    input_cells: &ClassifiedCells,
    output_cells: &ClassifiedCells,
) -> Result<(), String> {
    // Validate transaction structure
    validate_all! {
        expect_transaction(recipe).to_have_method_path(b"CDP.openVault"),
        expect_transaction(recipe).to_have_arguments_count(2),
        expect_transaction(recipe).to_have_argument_with_length(0, 16), // collateral_amount
        expect_transaction(recipe).to_have_argument_with_length(1, 16), // debt_amount
    }?;

    // Parse and validate arguments
    let args = recipe.arguments_vec();
    let collateral_amount = expect_u128_argument(&args[0], "Collateral amount")?;
    let debt_amount = expect_u128_argument(&args[1], "Debt amount")?;

    // Business logic validations
    // These will map to Error code 46 (CustomValidationFailed) if they fail
    validate_all! {
        expect(collateral_amount)
            .to_be_greater_than(0u128)
            .map_err(|_| "Collateral amount must be greater than zero".to_string()),
        validation_block!("Collateral ratio check", {
            if debt_amount > 0 {
                let minimum_collateral = debt_amount.saturating_mul(3).saturating_div(2);
                expect(collateral_amount)
                    .to_be_greater_than_or_equal(minimum_collateral)
                    .map_err(|_| format!(
                        "Insufficient collateral ratio: need {} but got {}", 
                        minimum_collateral, collateral_amount
                    ))?;
            }
            Ok(())
        }),
    }?;

    // Validate xUDT input cells
    validation_block!("xUDT input validation", {
        let xudt_cells = input_cells.get_known("xudt")
            .ok_or("No xUDT input cells found")?;
        
        expect(xudt_cells).not_to_be_empty()?;
        
        // Validate each xUDT cell
        let mut expected_udt_type_id: Option<Vec<u8>> = None;
        
        for cell in xudt_cells {
            // Validate lock script args
            let lock_args = cell.lock.args().raw_data();
            expect(lock_args.as_ref())
                .to_have_length(20)
                .map_err(|_| "xUDT lock args must be 20 bytes (secp256k1 pubkey hash)")?;
            
            expect(lock_args.as_ref().iter().all(|&b| b == 0))
                .to_be_false()
                .map_err(|_| "xUDT lock args cannot be all zeros")?;
            
            // Validate type script
            let type_script = cell.type_script.as_ref()
                .ok_or("xUDT cell must have type script")?;
            
            let type_args = type_script.args().raw_data();
            expect(type_args.as_ref().len())
                .to_be_greater_than_or_equal(32)
                .map_err(|_| "xUDT type args must be at least 32 bytes")?;
            
            // Validate owner lock hash
            expect(type_args.as_ref()[0..32].iter().all(|&b| b == 0))
                .to_be_false()
                .map_err(|_| "xUDT owner lock hash cannot be zero")?;
            
            // Ensure consistent UDT type ID
            let current_type_args = type_args.as_ref().to_vec();
            match &expected_udt_type_id {
                Some(expected) => {
                    expect(&current_type_args).to_equal(expected)?;
                },
                None => {
                    expected_udt_type_id = Some(current_type_args);
                }
            }
        }
        
        Ok(())
    })?;

    // Validate output cells
    validate_all! {
        expect_cells(output_cells).to_have_custom_cells_count(b"vault", 1),
        validation_block!("Vault output validation", {
            let vault_cells = output_cells.get_custom(b"vault")
                .ok_or("Vault output not found")?;
            
            let vault_cell = vault_cells.first()
                .ok_or("Vault output cell missing")?;
            
            // Ensure vault has type script
            expect(vault_cell.type_script.is_some())
                .to_be_true()
                .map_err(|_| "Vault cell must have type script")?;
            
            Ok(())
        }),
    }?;

    Ok(())
}

/// Validates close vault transaction using Jest-like assertions
pub fn validate_close_vault_transaction(
    recipe: &TransactionRecipe,
    input_cells: &ClassifiedCells,
    output_cells: &ClassifiedCells,
) -> Result<(), String> {
    // Validate transaction structure
    validate_all! {
        expect_transaction(recipe).to_have_method_path(b"CDP.closeVault"),
        expect_transaction(recipe).to_have_arguments_count(1),
        expect_transaction(recipe).to_have_argument_with_length(0, 8), // vault_id
    }?;

    // Parse and validate vault ID
    let args = recipe.arguments_vec();
    let vault_id = expect_u64_argument(&args[0], "Vault ID")?;
    
    expect(vault_id).to_be_greater_than(0u64)?;

    // Validate vault input
    validation_block!("Vault input validation", {
        expect_cells(input_cells).to_have_custom_cells_count(b"vault", 1)?;
        
        let vault_cells = input_cells.get_custom(b"vault").unwrap();
        let vault_cell = vault_cells.first().unwrap();
        
        // Validate vault has type script
        let type_script = vault_cell.type_script.as_ref()
            .ok_or("Vault cell must have type script")?;
        
        let type_args = type_script.args().raw_data();
        expect(type_args.as_ref()).not_to_be_empty()?;
        
        Ok(())
    })?;

    // Validate stable token input (debt repayment)
    validation_block!("Stable token validation", {
        expect_cells(input_cells).to_have_custom_cells(b"stable")?;
        
        let stable_cells = input_cells.get_custom(b"stable")
            .ok_or("Stable token cells not found")?;
        
        expect(stable_cells).not_to_be_empty()?;
        
        for cell in stable_cells {
            let type_script = cell.type_script.as_ref()
                .ok_or("Stable token must have type script")?;
            
            let type_args = type_script.args().raw_data();
            expect(type_args.as_ref().len())
                .to_be_greater_than_or_equal(32)
                .map_err(|_| "Stable token type args must be at least 32 bytes")?;
            
            expect(type_args.as_ref()[0..32].iter().all(|&b| b == 0))
                .to_be_false()
                .map_err(|_| "Stable token issuer lock hash cannot be zero")?;
        }
        
        Ok(())
    })?;

    // Validate outputs
    validate_all! {
        expect_cells(output_cells).to_have_custom_cells_count(b"vault", 0),
        expect_cells(output_cells).to_have_custom_cells_count(b"stable", 0),
        validation_block!("Collateral return validation", {
            let xudt_outputs = output_cells.get_known("xudt")
                .ok_or("No xUDT outputs for collateral return")?;
            
            expect(xudt_outputs).not_to_be_empty()?;
            
            for cell in xudt_outputs {
                let lock_args = cell.lock.args().raw_data();
                expect(lock_args.as_ref())
                    .to_have_length(20)
                    .map_err(|_| "Returned collateral lock args must be 20 bytes")?;
                
                expect(lock_args.as_ref().iter().all(|&b| b == 0))
                    .to_be_false()
                    .map_err(|_| "Returned collateral lock args cannot be all zeros")?;
            }
            
            Ok(())
        }),
    }?;

    Ok(())
}

/// Validates update vault transaction using Jest-like assertions
pub fn validate_update_vault_transaction(
    recipe: &TransactionRecipe,
    input_cells: &ClassifiedCells,
    output_cells: &ClassifiedCells,
) -> Result<(), String> {
    // Validate transaction structure
    validate_all! {
        expect_transaction(recipe).to_have_method_path(b"CDP.updateVault"),
        expect_transaction(recipe).to_have_arguments_count(2),
        expect_transaction(recipe).to_have_argument_with_length(0, 8),  // vault_id
        expect_transaction(recipe).to_have_argument_with_length(1, 16), // new_collateral_amount
    }?;

    // Parse and validate arguments
    let args = recipe.arguments_vec();
    let vault_id = expect_u64_argument(&args[0], "Vault ID")?;
    let new_collateral_amount = expect_u128_argument(&args[1], "New collateral amount")?;

    validate_all! {
        expect(vault_id).to_be_greater_than(0u64),
        expect(new_collateral_amount).to_be_greater_than(0u128),
    }?;

    // Validate vault consistency
    validation_block!("Vault consistency validation", {
        expect_cells(input_cells).to_have_custom_cells_count(b"vault", 1)?;
        expect_cells(output_cells).to_have_custom_cells_count(b"vault", 1)?;
        
        let input_vault = input_cells.get_custom(b"vault").unwrap().first().unwrap();
        let output_vault = output_cells.get_custom(b"vault").unwrap().first().unwrap();
        
        // Both must have type scripts
        let input_type_script = input_vault.type_script.as_ref()
            .ok_or("Input vault must have type script")?;
        let output_type_script = output_vault.type_script.as_ref()
            .ok_or("Output vault must have type script")?;
        
        // Type args must match (vault ID consistency)
        let input_type_args = input_type_script.args().raw_data();
        let output_type_args = output_type_script.args().raw_data();
        
        expect(output_type_args.as_ref()).to_equal(input_type_args.as_ref())?;
        
        Ok(())
    })?;

    // Validate xUDT cells if present
    validation_block!("xUDT validation", {
        // Helper to validate xUDT cells
        let validate_xudt_cells = |cells: &[ckb_deterministic::cell_classifier::CellInfo], cell_type: &str| -> Result<(), String> {
            for cell in cells {
                let lock_args = cell.lock.args().raw_data();
                expect(lock_args.as_ref())
                    .to_have_length(20)
                    .map_err(|e| format!("{} xUDT: {}", cell_type, e))?;
                
                expect(lock_args.as_ref().iter().all(|&b| b == 0))
                    .to_be_false()
                    .map_err(|_| format!("{} xUDT lock args cannot be all zeros", cell_type))?;
            }
            Ok(())
        };
        
        if let Some(xudt_inputs) = input_cells.get_known("xudt") {
            validate_xudt_cells(xudt_inputs, "Input")?;
        }
        
        if let Some(xudt_outputs) = output_cells.get_known("xudt") {
            validate_xudt_cells(xudt_outputs, "Output")?;
        }
        
        Ok(())
    })?;

    Ok(())
}

