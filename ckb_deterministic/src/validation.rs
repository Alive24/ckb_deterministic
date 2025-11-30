//! Declarative transaction validation framework.
//!
//! This module provides a flexible framework for defining and enforcing validation rules
//! for CKB transactions based on their method paths and cell types.
//!
//! # Key Components
//!
//! - `TransactionValidationRules`: Main rule container and validator
//! - `CellCountConstraint`: Flexible constraints for cell counts
//! - `CellCountRule`: Rules for specific cell types
//! - Business rule functions for custom validation logic
//! - Dependency validation for cell deps and header deps
//!
//! # Example
//!
//! ```no_run
//! use ckb_deterministic::validation::{
//!     TransactionValidationRules,
//!     CellCountConstraint,
//! };
//!
//! let rules = TransactionValidationRules::new(b"Protocol.update".to_vec())
//!     .with_arguments(2)  // Require exactly 2 arguments
//!     .with_known_cell(
//!         "xudt",
//!         CellCountConstraint::at_least(1),  // At least 1 xUDT input
//!         CellCountConstraint::any(),  // Any number of outputs
//!     )
//!     .with_custom_cell(
//!         "vault",
//!         CellCountConstraint::exactly(1),  // Exactly 1 vault input
//!         CellCountConstraint::exactly(1),  // Exactly 1 vault output
//!     );
//! ```
use crate::cell_classifier::CellClassifier;
use crate::debug_trace;
use crate::errors::Error;
use crate::transaction_context::TransactionContext;
use crate::transaction_deps::DepType;
use crate::transaction_recipe::TransactionRecipeExt;
extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Cell count constraint for validation.
///
/// Provides flexible ways to specify how many cells of a certain type
/// are allowed in inputs or outputs.
#[derive(Debug, Clone, Copy)]
pub struct CellCountConstraint {
    /// Minimum number of cells (inclusive)
    pub min: Option<usize>,
    /// Maximum number of cells (inclusive)
    pub max: Option<usize>,
    /// Exact count (overrides min/max if set)
    pub exact: Option<usize>,
}

impl CellCountConstraint {
    /// Create a constraint requiring exactly n cells.
    pub fn exactly(n: usize) -> Self {
        Self {
            min: None,
            max: None,
            exact: Some(n),
        }
    }

    /// Create a constraint requiring at least n cells.
    pub fn at_least(n: usize) -> Self {
        Self {
            min: Some(n),
            max: None,
            exact: None,
        }
    }

    /// Create a constraint requiring at most n cells.
    pub fn at_most(n: usize) -> Self {
        Self {
            min: None,
            max: Some(n),
            exact: None,
        }
    }

    /// Create a constraint for a range of cells
    pub fn range(min: usize, max: usize) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
            exact: None,
        }
    }

    /// Create a constraint that allows any number of cells
    pub fn any() -> Self {
        Self {
            min: None,
            max: None,
            exact: None,
        }
    }

    /// Check if a count satisfies this constraint
    pub fn is_satisfied_by(&self, count: usize) -> bool {
        match self.exact {
            Some(exact) => return count == exact,
            None => {}
        }

        match self.min {
            Some(min) => {
                if count < min {
                    return false;
                }
            }
            None => {}
        }

        match self.max {
            Some(max) => {
                if count > max {
                    return false;
                }
            }
            None => {}
        }

        true
    }

    /// Get a description of the constraint for error messages
    pub fn description(&self) -> String {
        match self.exact {
            Some(exact) => return format!("exactly {}", exact),
            None => {}
        }

        match (self.min, self.max) {
            (Some(min), Some(max)) => format!("between {} and {}", min, max),
            (Some(min), None) => format!("at least {}", min),
            (None, Some(max)) => format!("at most {}", max),
            (None, None) => "any".to_string(),
        }
    }
}

/// Validation rules for a specific cell type
#[derive(Debug, Clone)]
pub struct CellTypeCountRule {
    /// The cell type identifier (for known cells) or custom ID
    pub cell_type: String,
    /// Input cell type count constraint
    pub input_constraint: CellCountConstraint,
    /// Output cell type count constraint
    pub output_constraint: CellCountConstraint,
    /// CellDep cell type count constraint
    pub cell_dep_constraint: CellCountConstraint,
}

// Enhanced validation predicate with full transaction context access
pub type ValidationPredicate<C> = fn(&TransactionContext<C>) -> Result<(), Error>;

#[derive(Debug, Clone)]
pub struct ValidationRule<C: CellClassifier> {
    pub rule_name: String,
    pub rule_description: String,
    pub involved_cell_types: Vec<String>,
    pub predicate: ValidationPredicate<C>,
}

/// Required dependency specification
#[derive(Debug, Clone)]
pub struct RequiredDep {
    pub tx_hash: [u8; 32],
    pub index: u32,
    pub dep_type: DepType,
}

/// Complete validation rules for a transaction type
#[derive(Debug, Clone)]
pub struct TransactionValidationRules<C: CellClassifier> {
    /// Method path this rule applies to
    pub method_path: Vec<u8>,
    /// Expected number of arguments
    pub expected_arguments: Option<usize>,
    /// Whether to allow unidentified cells
    pub allow_unidentified: bool,
    /// Rules for each cell type count
    pub cell_type_count_rules: Vec<CellTypeCountRule>,
    /// Rules for cell relationships
    pub cell_relationship_rules: Vec<ValidationRule<C>>,
    /// Business rules
    pub business_rules: Vec<ValidationRule<C>>,
    /// Additional rules
    pub additional_rules: Vec<ValidationRule<C>>,
}

impl<C: CellClassifier> TransactionValidationRules<C> {
    /// Create new validation rules for a method
    pub fn new(method_path: impl Into<Vec<u8>>) -> Self {
        Self {
            method_path: method_path.into(),
            expected_arguments: None,
            cell_type_count_rules: Vec::new(),
            cell_relationship_rules: Vec::new(),
            business_rules: Vec::new(),
            additional_rules: Vec::new(),
            allow_unidentified: false,
        }
    }

    /// Set expected number of arguments
    pub fn with_arguments(mut self, count: usize) -> Self {
        self.expected_arguments = Some(count);
        self
    }

    /// Add a rule for a known cell type
    pub fn with_known_cell(
        mut self,
        identifier: String,
        input_constraint: CellCountConstraint,
        output_constraint: CellCountConstraint,
    ) -> Self {
        self.cell_type_count_rules.push(CellTypeCountRule {
            cell_type: identifier,
            input_constraint,
            output_constraint,
            cell_dep_constraint: CellCountConstraint::any(),
        });
        self
    }

    /// Add a rule for a custom cell type
    pub fn with_custom_cell(
        mut self,
        cell_id: impl Into<String>,
        input_constraint: CellCountConstraint,
        output_constraint: CellCountConstraint,
    ) -> Self {
        self.cell_type_count_rules.push(CellTypeCountRule {
            cell_type: cell_id.into(),
            input_constraint,
            output_constraint,
            cell_dep_constraint: CellCountConstraint::any(),
        });
        self
    }

    /// Allow unidentified cells in the transaction
    pub fn allow_unidentified(mut self) -> Self {
        self.allow_unidentified = true;
        self
    }

    /// Add a cell relationship rule
    pub fn with_cell_relationship(
        mut self,
        rule_name: String,
        rule_description: String,
        involved_cell_types: Vec<String>,
        predicate: ValidationPredicate<C>,
    ) -> Self {
        self.cell_relationship_rules.push(ValidationRule {
            rule_name,
            rule_description,
            involved_cell_types,
            predicate,
        });
        self
    }

    /// Add a business rule
    pub fn with_business_rule(
        mut self,
        rule_name: String,
        rule_description: String,
        involved_cell_types: Vec<String>,
        predicate: ValidationPredicate<C>,
    ) -> Self {
        self.business_rules.push(ValidationRule {
            rule_name,
            rule_description,
            involved_cell_types,
            predicate,
        });
        self
    }

    /// Add an additional validation rule
    pub fn with_additional_rule(
        mut self,
        rule_name: String,
        rule_description: String,
        involved_cell_types: Vec<String>,
        predicate: ValidationPredicate<C>,
    ) -> Self {
        self.additional_rules.push(ValidationRule {
            rule_name,
            rule_description,
            involved_cell_types,
            predicate,
        });
        self
    }

    /// Validate a transaction with dependencies against these rules
    pub fn validate(&self, context: &TransactionContext<C>) -> Result<(), Error> {
        // Check method path
        let recipe_path = context.recipe.method_path_bytes();
        if recipe_path != self.method_path {
            debug_trace!(
                "WrongMethodPath: Expected method path {:?}, got {:?}",
                self.method_path,
                recipe_path
            );
            return Err(Error::WrongMethodPath);
        }

        // Check arguments count
        match self.expected_arguments {
            Some(expected_args) => {
                let actual_args = context.recipe.arguments_vec().len();
                if actual_args != expected_args {
                    debug_trace!(
                        "InvalidArgumentCount: Expected {} arguments, got {}",
                        expected_args,
                        actual_args
                    );
                    return Err(Error::InvalidArgumentCount);
                }
            }
            None => {}
        }

        // Check cell type count rules
        for rule in &self.cell_type_count_rules {
            // Try to get counts from both known and custom cells
            let cell_type_str = &rule.cell_type;
            let (input_count, output_count) = 
                // Check if it's a known cell type
                match context.input_cells.get_known(cell_type_str) {
                    Some(cells) => {
                        (
                            cells.len(),
                            context
                                .output_cells
                                .get_known(cell_type_str)
                                .map_or(0, |cells| cells.len()),
                        )
                    }
                    None => {
                        // Not a known cell, check custom cells
                        (
                            context
                                .input_cells
                                .get_custom(&rule.cell_type)
                                .map_or(0, |cells| cells.len()),
                            context
                                .output_cells
                                .get_custom(&rule.cell_type)
                                .map_or(0, |cells| cells.len()),
                        )
                    }
                };

            if !rule.input_constraint.is_satisfied_by(input_count) {
                debug_trace!(
                    "CellCountViolation: Input count for {} is {}",
                    cell_type_str,
                    input_count
                );
                return Err(Error::CellCountViolation);
            }

            if !rule.output_constraint.is_satisfied_by(output_count) {
                debug_trace!(
                    "CellCountViolation: Output count for {} is {}",
                    cell_type_str,
                    output_count
                );
                return Err(Error::CellCountViolation);
            }
        }

        // Check unidentified cells
        if !self.allow_unidentified {
            if !context.input_cells.unidentified_cells.is_empty() {
                debug_trace!("UnidentifiedCells: Input cells contain unidentified cells");
                return Err(Error::UnidentifiedCells);
            }
            if !context.output_cells.unidentified_cells.is_empty() {
                debug_trace!("UnidentifiedCells: Output cells contain unidentified cells");
                return Err(Error::UnidentifiedCells);
            }
        }

        // Run cell relationship rules
        for cell_relationship_rule in &self.cell_relationship_rules {
            debug_trace!(
                "Running cell relationship rule: {}",
                cell_relationship_rule.rule_name
            );
            (cell_relationship_rule.predicate)(context)?;
        }

        // Run business rules
        for business_rule in &self.business_rules {
            debug_trace!("Running business rule: {}", business_rule.rule_name);
            (business_rule.predicate)(context)?;
        }

        // Run additional validation rules
        for additional_rule in &self.additional_rules {
            debug_trace!("Running additional rule: {}", additional_rule.rule_name);
            (additional_rule.predicate)(context)?;
        }

        // All validation rules have been applied

        Ok(())
    }
}
