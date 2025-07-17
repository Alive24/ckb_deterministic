/// CDP recipe-specific validation logic
use ckb_deterministic::{
    cell_classifier::RuleBasedClassifier,
    known_scripts::KnownScript,
    transaction_context::TransactionContext,
    validation::{CellCountConstraint, TransactionValidationRules},
};
use deterministic_cdp_shared::{CDP_ADJUST_VAULT, CDP_CLOSE_VAULT, CDP_OPEN_VAULT, CDP_VAULT_CELL};
extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec;
/// OpenVault validation functions
pub mod open_vault {
    use super::*;
    use ckb_deterministic::errors::Error as DeterministicError;
    use deterministic_cdp_shared::parse_vault_data;

    /// Get validation rules for CDP.openVault transaction
    pub fn get_rules() -> TransactionValidationRules<RuleBasedClassifier> {
        TransactionValidationRules::new(CDP_OPEN_VAULT.to_vec())
            .with_arguments(3) // owner_lock_hash, initial_collateral, initial_debt
            .with_custom_cell(
                CDP_VAULT_CELL,
                CellCountConstraint::exactly(0), // No input vaults
                CellCountConstraint::exactly(1), // One output vault
            )
            .with_known_cell(
                KnownScript::XUdt,
                CellCountConstraint::at_least(1), // At least one xUDT input
                CellCountConstraint::at_least(1), // At least one xUDT output
            )
            .with_business_rule(
                "validate_collateral_ratio".to_string(),
                "Validate collateral ratio for new vault creation".to_string(),
                vec![String::from(CDP_VAULT_CELL), String::from("xudt")],
                validate_collateral_ratio,
            )
    }

    /// Validate collateral ratio for new vault creation with full context access
    fn validate_collateral_ratio(
        context: &TransactionContext<RuleBasedClassifier>,
    ) -> core::result::Result<(), DeterministicError> {
        // Access vault cells from transaction context
        let vault_cells = context
            .output_cells
            .get_custom(CDP_VAULT_CELL)
            .ok_or(DeterministicError::DataError)?;

        if vault_cells.len() != 1 {
            return Err(DeterministicError::DataError);
        }

        let vault = &vault_cells[0];
        let (_owner_lock_hash, collateral_amount, debt_amount) =
            parse_vault_data(&vault.data).ok_or(DeterministicError::DataError)?;

        // Example: Access recipe arguments for additional validation
        let args = &context.recipe.arguments();
        if args.len() != 3 {
            return Err(DeterministicError::InvalidArgumentCount);
        }

        // Example: Check required dependencies are present
        if context.cell_deps.is_empty() {
            return Err(DeterministicError::MissingCellDep);
        }

        // Validate initial collateral ratio
        crate::recipes::common::validate_collateral_ratio(collateral_amount, debt_amount)?;

        Ok(())
    }
}

/// CloseVault validation functions
pub mod close_vault {
    use super::*;
    use ckb_deterministic::errors::Error as DeterministicError;
    use deterministic_cdp_shared::parse_vault_data;

    /// Get validation rules for CDP.closeVault transaction
    pub fn get_rules() -> TransactionValidationRules<RuleBasedClassifier> {
        TransactionValidationRules::new(CDP_CLOSE_VAULT.to_vec())
            .with_arguments(1) // vault_id or owner_lock_hash
            .with_custom_cell(
                CDP_VAULT_CELL,
                CellCountConstraint::exactly(1), // One input vault
                CellCountConstraint::exactly(0), // No output vaults
            )
            .with_known_cell(
                KnownScript::XUdt,
                CellCountConstraint::at_least(1), // At least one xUDT input (for repaying debt)
                CellCountConstraint::at_least(1), // At least one xUDT output (returning collateral)
            )
            .with_business_rule(
                "validate_debt_repayment".to_string(),
                "Validate debt repayment for vault closure".to_string(),
                vec![String::from(CDP_VAULT_CELL), String::from("xudt")],
                validate_debt_repayment,
            )
    }

    /// Validate debt repayment for vault closure
    fn validate_debt_repayment(
        context: &TransactionContext<RuleBasedClassifier>,
    ) -> core::result::Result<(), DeterministicError> {
        // Get the vault being closed from input cells
        let vault_cells = context
            .input_cells
            .get_custom(CDP_VAULT_CELL)
            .ok_or(DeterministicError::DataError)?;

        if vault_cells.len() != 1 {
            return Err(DeterministicError::DataError);
        }

        let vault = &vault_cells[0];
        let (_owner_lock_hash, _collateral_amount, debt_amount) =
            parse_vault_data(&vault.data).ok_or(DeterministicError::DataError)?;

        // Access xUDT cells for debt repayment validation
        let empty_vec = vec![];
        let xudt_inputs = context.input_cells.get_known("xudt").unwrap_or(&empty_vec);
        
        // Example: Check that we have xUDT inputs to repay the debt
        if debt_amount > 0 && xudt_inputs.is_empty() {
            return Err(DeterministicError::BusinessRuleViolation);
        }

        // Example: Validate dependencies are present
        if context.cell_deps.is_empty() {
            return Err(DeterministicError::MissingCellDep);
        }

        Ok(())
    }
}

/// AdjustVault validation functions
pub mod adjust_vault {
    use super::*;
    use ckb_deterministic::errors::Error as DeterministicError;
    use deterministic_cdp_shared::parse_vault_data;

    /// Get validation rules for CDP.adjustVault transaction
    pub fn get_rules() -> TransactionValidationRules<RuleBasedClassifier> {
        TransactionValidationRules::new(CDP_ADJUST_VAULT.to_vec())
            .with_arguments(3) // vault_id, collateral_delta, debt_delta
            .with_custom_cell(
                CDP_VAULT_CELL,
                CellCountConstraint::exactly(1), // One input vault
                CellCountConstraint::exactly(1), // One output vault
            )
            .with_known_cell(
                KnownScript::XUdt,
                CellCountConstraint::any(), // Variable xUDT cells
                CellCountConstraint::any(), // Variable xUDT cells
            )
            .with_cell_relationship(
                "validate_owner_continuity".to_string(),
                "Validate owner continuity when adjusting vault".to_string(),
                vec![String::from(CDP_VAULT_CELL)],
                validate_owner_continuity,
            )
            .with_business_rule(
                "validate_collateral_ratio".to_string(),
                "Validate collateral ratio after vault adjustment".to_string(),
                vec![String::from(CDP_VAULT_CELL), String::from("xudt")],
                validate_collateral_ratio,
            )
    }

    /// Validate owner continuity when adjusting vault
    fn validate_owner_continuity(
        context: &TransactionContext<RuleBasedClassifier>,
    ) -> core::result::Result<(), DeterministicError> {
        let input_vaults = context
            .input_cells
            .get_custom(CDP_VAULT_CELL)
            .ok_or(DeterministicError::DataError)?;
        let output_vaults = context
            .output_cells
            .get_custom(CDP_VAULT_CELL)
            .ok_or(DeterministicError::DataError)?;

        if input_vaults.len() != 1 || output_vaults.len() != 1 {
            return Err(DeterministicError::DataError);
        }

        let input_vault = &input_vaults[0];
        let output_vault = &output_vaults[0];

        let (input_owner, _, _) =
            parse_vault_data(&input_vault.data).ok_or(DeterministicError::DataError)?;
        let (output_owner, _, _) =
            parse_vault_data(&output_vault.data).ok_or(DeterministicError::DataError)?;

        // Ensure owner doesn't change
        if input_owner != output_owner {
            return Err(DeterministicError::CellRelationshipRuleViolation);
        }

        Ok(())
    }

    /// Validate collateral ratio after vault adjustment
    fn validate_collateral_ratio(
        context: &TransactionContext<RuleBasedClassifier>,
    ) -> core::result::Result<(), DeterministicError> {
        let output_vaults = context
            .output_cells
            .get_custom(CDP_VAULT_CELL)
            .ok_or(DeterministicError::DataError)?;

        if output_vaults.len() != 1 {
            return Err(DeterministicError::DataError);
        }

        let output_vault = &output_vaults[0];

        let (_owner, output_collateral, output_debt) =
            parse_vault_data(&output_vault.data).ok_or(DeterministicError::DataError)?;

        // Example: Access recipe arguments for delta validation
        let args = &context.recipe.arguments();
        if args.len() != 3 {
            return Err(DeterministicError::InvalidArgumentCount);
        }

        // Example: Check dependencies for price oracle
        if context.header_deps.is_empty() {
            // In a real CDP, we might require price oracle header deps
            // For now, just continue
        }

        // Validate final collateral ratio
        crate::recipes::common::validate_collateral_ratio(output_collateral, output_debt)?;

        Ok(())
    }
}

/// Common validation helpers
pub mod common {
    use ckb_deterministic::errors::Error as DeterministicError;
    use deterministic_cdp_shared::MIN_COLLATERALIZATION_RATIO;

    /// Validate collateral ratio meets minimum requirement
    pub fn validate_collateral_ratio(
        collateral_amount: u128,
        debt_amount: u128,
    ) -> core::result::Result<(), DeterministicError> {
        // For simplicity, assume collateral price is 100 units
        let collateral_price = 100u64;

        if debt_amount > 0 {
            let collateral_value = (collateral_amount as u64).saturating_mul(collateral_price);
            let ratio = collateral_value.saturating_mul(100) / (debt_amount as u64);

            if ratio < MIN_COLLATERALIZATION_RATIO {
                return Err(DeterministicError::BusinessRuleViolation);
            }
        }

        Ok(())
    }
}
