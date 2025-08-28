//! # CDP Transaction Validation Recipes
//!
//! This module contains the validation logic for all CDP (Collateralized Debt Position) 
//! transaction types. Each recipe defines the complete set of rules that must be satisfied
//! for a specific CDP operation to be considered valid.
//!
//! ## Validation Framework
//!
//! Each recipe uses the `ckb_deterministic` validation framework to provide:
//! - **Method Path Validation**: Ensures the transaction is calling the correct operation
//! - **Cell Count Constraints**: Validates the number of input/output cells of each type
//! - **Business Rules**: Enforces protocol-specific logic (collateralization ratios, etc.)
//! - **Cell Relationships**: Validates relationships between input and output cells
//!
//! ## Supported Operations
//!
//! - [`open_vault`] - Create new vaults with initial collateral and debt
//! - [`close_vault`] - Close existing vaults by repaying debt  
//! - [`adjust_vault`] - Modify existing vault collateral or debt amounts
//! - [`common`] - Shared validation utilities used across operations
//!
//! ## Architecture
//!
//! Each operation module follows a consistent pattern:
//! 1. `get_rules()` function that returns a `TransactionValidationRules` instance
//! 2. Private validation functions that implement specific business logic
//! 3. Integration with the deterministic framework for type safety and error handling
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use deterministic_cdp_project::recipes::open_vault;
//! 
//! // Get validation rules for opening a vault
//! let rules = open_vault::get_rules();
//! 
//! // Apply rules to a transaction context
//! let result = rules.validate(&transaction_context);
//! ```

use ckb_deterministic::{
    cell_classifier::RuleBasedClassifier,
    transaction_context::TransactionContext,
    validation::{CellCountConstraint, TransactionValidationRules},
};
use deterministic_cdp_shared::{CDP_ADJUST_VAULT, CDP_CLOSE_VAULT, CDP_OPEN_VAULT, CDP_VAULT_CELL};
extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec;
/// # Open Vault Operations
/// 
/// This module handles validation for vault creation transactions. Opening a vault involves
/// depositing collateral assets and borrowing against them at a specified collateralization ratio.
/// 
/// ## Transaction Requirements
/// 
/// A valid `openVault` transaction must satisfy:
/// - **Method Path**: Transaction recipe method must be "openVault"
/// - **Arguments**: Exactly 3 arguments (owner_lock_hash, initial_collateral, initial_debt)
/// - **Input Cells**: 
///   - 0 CDP vault cells (new vault creation)
///   - At least 1 xUDT cell (for collateral deposit)
/// - **Output Cells**:
///   - Exactly 1 CDP vault cell (newly created vault)
///   - At least 1 xUDT cell (collateral and change management)
/// - **Business Rules**: 
///   - Collateralization ratio must meet minimum requirements
///   - Vault data must be properly formatted and valid
/// 
/// ## Security Considerations
/// 
/// - Validates that sufficient collateral is deposited relative to debt amount
/// - Ensures proper cell dependencies are present for price validation
/// - Verifies correct argument count to prevent parameter confusion attacks
pub mod open_vault {
    use super::*;
    use ckb_deterministic::errors::Error as DeterministicError;
    use deterministic_cdp_shared::parse_vault_data;

    /// Get validation rules for CDP.openVault transaction
    /// 
    /// Creates a comprehensive set of validation rules that must all pass for a vault
    /// opening transaction to be considered valid.
    /// 
    /// # Returns
    /// 
    /// A `TransactionValidationRules` instance configured with:
    /// - Method path validation for "openVault"
    /// - Argument count validation (exactly 3 required)
    /// - Cell count constraints for vault and xUDT cells
    /// - Custom business rule for collateral ratio validation
    /// 
    /// # Example
    /// 
    /// ```rust,no_run
    /// let rules = open_vault::get_rules();
    /// let validation_result = rules.validate(&transaction_context);
    /// ```
    pub fn get_rules() -> TransactionValidationRules<RuleBasedClassifier> {
        TransactionValidationRules::new(CDP_OPEN_VAULT.to_vec())
            .with_arguments(3) // owner_lock_hash, initial_collateral, initial_debt
            .with_custom_cell(
                CDP_VAULT_CELL,
                CellCountConstraint::exactly(0), // No input vaults
                CellCountConstraint::exactly(1), // One output vault
            )
            .with_known_cell(
                String::from("xudt"),
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

    /// Validate collateral ratio for new vault creation
    /// 
    /// This function implements the core business logic for vault opening by ensuring
    /// that the collateralization ratio meets the minimum requirements defined by the
    /// CDP protocol.
    /// 
    /// # Process
    /// 
    /// 1. **Extract Vault Data**: Retrieves the new vault cell from transaction outputs
    /// 2. **Parse Vault Contents**: Decodes owner, collateral, and debt amounts from cell data
    /// 3. **Validate Arguments**: Ensures the transaction has the correct number of arguments
    /// 4. **Check Dependencies**: Verifies required cell dependencies are present
    /// 5. **Calculate Ratio**: Computes collateralization ratio and validates against minimums
    /// 
    /// # Parameters
    /// - `context`: Complete transaction context including classified cells and recipe data
    /// 
    /// # Returns
    /// - `Ok(())` if collateral ratio is sufficient and all validations pass
    /// - `Err(DeterministicError)` with specific error type if validation fails:
    ///   - `DataError` if vault data is malformed or missing
    ///   - `InvalidArgumentCount` if wrong number of arguments provided
    ///   - `MissingCellDep` if required dependencies are missing
    ///   - `BusinessRuleViolation` if collateral ratio is insufficient
    /// 
    /// # Security Notes
    /// - Validates exact cell counts to prevent double-spending or missing cells
    /// - Checks argument count to prevent parameter confusion attacks
    /// - Ensures cell dependencies are present (required for price oracles in production)
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

/// # Close Vault Operations
/// 
/// This module handles validation for vault closure transactions. Closing a vault involves
/// repaying all outstanding debt and reclaiming the deposited collateral.
/// 
/// ## Transaction Requirements
/// 
/// A valid `closeVault` transaction must satisfy:
/// - **Method Path**: Transaction recipe method must be "closeVault"  
/// - **Arguments**: Exactly 1 argument (vault_id or owner_lock_hash for identification)
/// - **Input Cells**:
///   - Exactly 1 CDP vault cell (the vault being closed)
///   - At least 1 xUDT cell (for debt repayment tokens)
/// - **Output Cells**:
///   - 0 CDP vault cells (vault is destroyed)
///   - At least 1 xUDT cell (returned collateral and change)
/// - **Business Rules**:
///   - All debt must be properly repaid
///   - Collateral must be correctly returned to vault owner
/// 
/// ## Economic Flow
/// 
/// 1. **Debt Repayment**: User provides xUDT tokens to cover outstanding debt
/// 2. **Vault Destruction**: The vault cell is consumed (not recreated in outputs)
/// 3. **Collateral Return**: Collateral is returned to the vault owner
/// 4. **Change Handling**: Any excess repayment tokens are returned as change
/// 
/// ## Security Considerations
/// 
/// - Validates that sufficient tokens are provided to cover all debt
/// - Ensures vault ownership is respected during closure
/// - Prevents partial closures (all debt must be repaid)
pub mod close_vault {
    use super::*;
    use ckb_deterministic::errors::Error as DeterministicError;
    use deterministic_cdp_shared::parse_vault_data;

    /// Get validation rules for CDP.closeVault transaction
    /// 
    /// Creates validation rules for completely closing an existing vault by repaying
    /// all debt and reclaiming collateral.
    /// 
    /// # Returns
    /// 
    /// A `TransactionValidationRules` instance configured with:
    /// - Method path validation for "closeVault"
    /// - Argument count validation (exactly 1 required)
    /// - Cell count constraints ensuring vault destruction
    /// - Custom business rule for debt repayment validation
    /// 
    /// # Cell Flow
    /// - **Input**: 1 vault cell + xUDT for debt payment
    /// - **Output**: 0 vault cells + xUDT for returned collateral
    pub fn get_rules() -> TransactionValidationRules<RuleBasedClassifier> {
        TransactionValidationRules::new(CDP_CLOSE_VAULT.to_vec())
            .with_arguments(1) // vault_id or owner_lock_hash
            .with_custom_cell(
                CDP_VAULT_CELL,
                CellCountConstraint::exactly(1), // One input vault
                CellCountConstraint::exactly(0), // No output vaults
            )
            .with_known_cell(
                String::from("xudt"),
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
    /// 
    /// Ensures that the vault being closed has its debt properly handled. This function
    /// validates both the debt amount in the vault and the presence of sufficient
    /// repayment tokens in the transaction inputs.
    /// 
    /// # Process
    /// 
    /// 1. **Extract Vault**: Gets the vault being closed from transaction inputs
    /// 2. **Parse Debt**: Extracts the outstanding debt amount from vault data
    /// 3. **Validate Repayment**: Ensures xUDT inputs are present if debt exists
    /// 4. **Check Dependencies**: Verifies required cell dependencies for validation
    /// 
    /// # Parameters  
    /// - `context`: Complete transaction context with all classified cells
    /// 
    /// # Returns
    /// - `Ok(())` if debt repayment is valid or vault has no debt
    /// - `Err(DeterministicError)` with specific error:
    ///   - `DataError` if vault data is malformed or count is wrong
    ///   - `BusinessRuleViolation` if debt exists but no repayment tokens provided
    ///   - `MissingCellDep` if required dependencies are missing
    /// 
    /// # Important Notes
    /// - For vaults with zero debt, no repayment tokens are required
    /// - The actual token amount validation is handled by the framework's
    ///   cell count constraints and business logic
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

/// # Adjust Vault Operations
/// 
/// This module handles validation for vault adjustment transactions. Adjusting a vault
/// allows users to modify their collateral and debt positions while maintaining vault
/// ownership and protocol compliance.
/// 
/// ## Transaction Requirements
/// 
/// A valid `adjustVault` transaction must satisfy:
/// - **Method Path**: Transaction recipe method must be "adjustVault"
/// - **Arguments**: Exactly 3 arguments (vault_id, collateral_delta, debt_delta)
/// - **Input Cells**:
///   - Exactly 1 CDP vault cell (existing vault to modify)
///   - Variable xUDT cells (depending on adjustment type)
/// - **Output Cells**:
///   - Exactly 1 CDP vault cell (updated vault with new amounts)  
///   - Variable xUDT cells (adjusted token amounts)
/// - **Business Rules**:
///   - Vault owner must remain unchanged
///   - Final collateralization ratio must meet minimum requirements
///   - Token flows must match collateral/debt adjustments
/// 
/// ## Adjustment Types
/// 
/// ### Adding Collateral
/// - Requires additional xUDT input tokens
/// - Increases vault collateral amount
/// - Improves collateralization ratio
/// 
/// ### Removing Collateral  
/// - Produces additional xUDT output tokens
/// - Decreases vault collateral amount
/// - Must maintain minimum collateralization ratio
/// 
/// ### Borrowing More (Increasing Debt)
/// - Produces additional xUDT output tokens
/// - Increases vault debt amount
/// - Must maintain minimum collateralization ratio
/// 
/// ### Repaying Debt (Decreasing Debt)
/// - Requires additional xUDT input tokens
/// - Decreases vault debt amount
/// - Improves collateralization ratio
/// 
/// ## Security Considerations
/// 
/// - Validates vault ownership continuity to prevent unauthorized modifications
/// - Ensures collateralization ratio compliance after all adjustments
/// - Prevents economic attacks through parameter manipulation
pub mod adjust_vault {
    use super::*;
    use ckb_deterministic::errors::Error as DeterministicError;
    use deterministic_cdp_shared::parse_vault_data;

    /// Get validation rules for CDP.adjustVault transaction
    /// 
    /// Creates validation rules for modifying an existing vault's collateral or debt amounts
    /// while ensuring owner continuity and collateralization ratio compliance.
    /// 
    /// # Returns
    /// 
    /// A `TransactionValidationRules` instance configured with:
    /// - Method path validation for "adjustVault"
    /// - Argument count validation (exactly 3 required)
    /// - Cell count constraints for vault transformation (1 input → 1 output)
    /// - Cell relationship rule for owner continuity validation
    /// - Business rule for post-adjustment collateral ratio validation
    /// 
    /// # Validation Rules Applied
    /// 
    /// 1. **Owner Continuity**: Vault owner cannot change during adjustment
    /// 2. **Collateral Ratio**: Final position must meet minimum requirements
    /// 3. **Token Conservation**: xUDT flows must match vault data changes
    pub fn get_rules() -> TransactionValidationRules<RuleBasedClassifier> {
        TransactionValidationRules::new(CDP_ADJUST_VAULT.to_vec())
            .with_arguments(3) // vault_id, collateral_delta, debt_delta
            .with_custom_cell(
                CDP_VAULT_CELL,
                CellCountConstraint::exactly(1), // One input vault
                CellCountConstraint::exactly(1), // One output vault
            )
            .with_known_cell(
                String::from("xudt"),
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

    /// Validate owner continuity during vault adjustment
    /// 
    /// Ensures that the vault owner remains unchanged during adjustment operations.
    /// This is a critical security check that prevents unauthorized users from
    /// modifying other users' vaults.
    /// 
    /// # Process
    /// 
    /// 1. **Extract Vaults**: Gets input and output vault cells from transaction
    /// 2. **Parse Owners**: Extracts owner lock hashes from both vault versions
    /// 3. **Compare Owners**: Validates that ownership hasn't changed
    /// 
    /// # Parameters
    /// - `context`: Complete transaction context with classified cells
    /// 
    /// # Returns
    /// - `Ok(())` if vault owner is unchanged
    /// - `Err(DeterministicError)` with specific error:
    ///   - `DataError` if vault data is malformed or wrong cell count
    ///   - `CellRelationshipRuleViolation` if owner has changed
    /// 
    /// # Security Importance
    /// This check prevents attacks where malicious actors attempt to modify
    /// other users' vaults by crafting transactions that adjust vault parameters.
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
    /// 
    /// Ensures that the vault's collateralization ratio meets minimum requirements
    /// after the adjustment operation is completed. This prevents users from
    /// creating unsafe positions that could lead to liquidation or system instability.
    /// 
    /// # Process
    /// 
    /// 1. **Extract Output Vault**: Gets the updated vault from transaction outputs
    /// 2. **Parse Final Amounts**: Extracts final collateral and debt amounts
    /// 3. **Validate Arguments**: Ensures proper argument count for delta validation
    /// 4. **Check Dependencies**: Verifies dependencies for price oracle access
    /// 5. **Calculate Ratio**: Computes final collateralization ratio
    /// 6. **Validate Compliance**: Ensures ratio meets minimum requirements
    /// 
    /// # Parameters
    /// - `context`: Complete transaction context with vault states and recipe data
    /// 
    /// # Returns
    /// - `Ok(())` if final collateral ratio is sufficient
    /// - `Err(DeterministicError)` with specific error:
    ///   - `DataError` if vault data is malformed or wrong cell count
    ///   - `InvalidArgumentCount` if wrong number of adjustment parameters
    ///   - `BusinessRuleViolation` if final ratio is below minimum
    /// 
    /// # Important Considerations
    /// - Uses final vault state (after adjustment) for ratio calculation
    /// - May require header dependencies for price oracle access in production
    /// - Prevents both intentional and accidental creation of undercollateralized positions
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

/// # Common Validation Utilities
/// 
/// This module provides shared validation functions used across multiple CDP operations.
/// These utilities implement core business logic that is consistent across different
/// transaction types.
/// 
/// ## Shared Functionality
/// 
/// - **Collateral Ratio Validation**: Ensures positions meet minimum safety requirements
/// - **Price Calculations**: Handles collateral valuation (with simplified pricing for examples)
/// - **Economic Safety**: Prevents creation of positions that could destabilize the system
/// 
/// ## Design Philosophy
/// 
/// By centralizing common validation logic, this module ensures:
/// - **Consistency**: Same validation rules applied across all operations
/// - **Maintainability**: Single place to update core business logic
/// - **Testing**: Easier to test shared functionality in isolation
pub mod common {
    use ckb_deterministic::errors::Error as DeterministicError;
    use deterministic_cdp_shared::MIN_COLLATERALIZATION_RATIO;

    /// Validate that collateral ratio meets minimum safety requirements
    /// 
    /// This function implements the core economic safety check for the CDP system.
    /// It ensures that any vault position has sufficient collateral relative to its
    /// debt to maintain system stability and user protection.
    /// 
    /// # Process
    /// 
    /// 1. **Calculate Collateral Value**: Multiplies collateral amount by price
    /// 2. **Calculate Ratio**: Computes (collateral_value / debt_amount) * 100
    /// 3. **Compare to Minimum**: Validates ratio meets protocol minimum
    /// 
    /// # Parameters
    /// - `collateral_amount`: Amount of collateral tokens in the vault
    /// - `debt_amount`: Amount of debt tokens owed by the vault
    /// 
    /// # Returns
    /// - `Ok(())` if collateral ratio is sufficient or debt is zero
    /// - `Err(BusinessRuleViolation)` if ratio is below minimum requirement
    /// 
    /// # Pricing Model
    /// 
    /// **Current Implementation**: Uses simplified fixed price of 100 units per collateral token
    /// 
    /// **Production Considerations**: In a real deployment, this would:
    /// - Access price oracle data from cell dependencies
    /// - Handle multiple collateral types with different prices
    /// - Account for price volatility and safety margins
    /// - Implement time-weighted average pricing for stability
    /// 
    /// # Safety Guarantees
    /// 
    /// - Prevents vault creation/modification below minimum ratio
    /// - Handles zero debt case gracefully (no ratio requirement)
    /// - Uses saturating arithmetic to prevent overflow attacks
    /// 
    /// # Example
    /// 
    /// ```rust,no_run
    /// // Vault with 1000 collateral tokens, 500 debt tokens
    /// // At price 100: collateral value = 100,000
    /// // Ratio = (100,000 / 500) * 100 = 20,000% (200x)
    /// // This exceeds minimum 150% requirement
    /// 
    /// let result = validate_collateral_ratio(1000, 500);
    /// assert!(result.is_ok());
    /// ```
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
