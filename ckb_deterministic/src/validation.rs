/// Transaction validation rules and constraints
/// 
/// This module provides a framework for defining and enforcing validation rules
/// for different transaction types based on their method paths.

use crate::cell_classifier::ClassifiedCells;
use crate::errors::ValidationError;
use crate::generated::TransactionRecipe;
use crate::transaction_recipe::TransactionRecipeExt;
extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::format;

/// Cell count constraint for validation
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
    /// Create a constraint for exactly n cells
    pub fn exactly(n: usize) -> Self {
        Self {
            min: None,
            max: None,
            exact: Some(n),
        }
    }
    
    /// Create a constraint for at least n cells
    pub fn at_least(n: usize) -> Self {
        Self {
            min: Some(n),
            max: None,
            exact: None,
        }
    }
    
    /// Create a constraint for at most n cells
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
        if let Some(exact) = self.exact {
            return count == exact;
        }
        
        if let Some(min) = self.min {
            if count < min {
                return false;
            }
        }
        
        if let Some(max) = self.max {
            if count > max {
                return false;
            }
        }
        
        true
    }
    
    /// Get a description of the constraint for error messages
    pub fn description(&self) -> String {
        if let Some(exact) = self.exact {
            return format!("exactly {}", exact);
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
pub struct CellTypeRule {
    /// The cell type identifier (for known cells) or custom ID
    pub cell_type: Vec<u8>,
    /// Whether this is a known cell type (true) or custom (false)
    pub is_known: bool,
    /// Input count constraint
    pub input_constraint: CellCountConstraint,
    /// Output count constraint
    pub output_constraint: CellCountConstraint,
}

/// Complete validation rules for a transaction type
#[derive(Debug, Clone)]
pub struct TransactionValidationRules {
    /// Method path this rule applies to
    pub method_path: Vec<u8>,
    /// Expected number of arguments
    pub expected_arguments: Option<usize>,
    /// Rules for each cell type
    pub cell_rules: Vec<CellTypeRule>,
    /// Whether to allow unidentified cells
    pub allow_unidentified: bool,
    /// Custom validation function (optional)
    pub custom_validator: Option<fn(&TransactionRecipe, &ClassifiedCells, &ClassifiedCells) -> Result<(), String>>,
}

impl TransactionValidationRules {
    /// Create new validation rules for a method
    pub fn new(method_path: impl Into<Vec<u8>>) -> Self {
        Self {
            method_path: method_path.into(),
            expected_arguments: None,
            cell_rules: Vec::new(),
            allow_unidentified: false,
            custom_validator: None,
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
        cell_type: impl Into<Vec<u8>>,
        input_constraint: CellCountConstraint,
        output_constraint: CellCountConstraint,
    ) -> Self {
        self.cell_rules.push(CellTypeRule {
            cell_type: cell_type.into(),
            is_known: true,
            input_constraint,
            output_constraint,
        });
        self
    }
    
    /// Add a rule for a custom cell type
    pub fn with_custom_cell(
        mut self,
        cell_id: impl Into<Vec<u8>>,
        input_constraint: CellCountConstraint,
        output_constraint: CellCountConstraint,
    ) -> Self {
        self.cell_rules.push(CellTypeRule {
            cell_type: cell_id.into(),
            is_known: false,
            input_constraint,
            output_constraint,
        });
        self
    }
    
    /// Allow unidentified cells in the transaction
    pub fn allow_unidentified(mut self) -> Self {
        self.allow_unidentified = true;
        self
    }
    
    /// Add a custom validation function
    pub fn with_custom_validator(
        mut self,
        validator: fn(&TransactionRecipe, &ClassifiedCells, &ClassifiedCells) -> Result<(), String>,
    ) -> Self {
        self.custom_validator = Some(validator);
        self
    }
    
    /// Validate a transaction against these rules
    pub fn validate(
        &self,
        recipe: &TransactionRecipe,
        input_cells: &ClassifiedCells,
        output_cells: &ClassifiedCells,
    ) -> Result<(), ValidationError> {
        // Check method path
        let recipe_path = recipe.method_path_bytes();
        if recipe_path != self.method_path {
            return Err(ValidationError::WrongMethodPath {
                expected: self.method_path.clone(),
                actual: recipe_path,
            });
        }
        
        // Check arguments count
        if let Some(expected_args) = self.expected_arguments {
            let actual_args = recipe.arguments_vec().len();
            if actual_args != expected_args {
                return Err(ValidationError::InvalidArgumentCount {
                    expected: expected_args,
                    actual: actual_args,
                });
            }
        }
        
        // Check cell rules
        for rule in &self.cell_rules {
            let (input_count, output_count) = if rule.is_known {
                // For known cells, convert Vec<u8> to str
                let cell_type_str = core::str::from_utf8(&rule.cell_type)
                    .map_err(|_| ValidationError::CustomValidation("Invalid UTF-8 in cell type".to_string()))?;
                (
                    input_cells.get_known(cell_type_str).map_or(0, |cells| cells.len()),
                    output_cells.get_known(cell_type_str).map_or(0, |cells| cells.len()),
                )
            } else {
                (
                    input_cells.get_custom(&rule.cell_type).map_or(0, |cells| cells.len()),
                    output_cells.get_custom(&rule.cell_type).map_or(0, |cells| cells.len()),
                )
            };
            
            if !rule.input_constraint.is_satisfied_by(input_count) {
                return Err(ValidationError::CellCountViolation {
                    cell_type: rule.cell_type.clone(),
                    is_input: true,
                    expected: rule.input_constraint.description(),
                    actual: input_count,
                });
            }
            
            if !rule.output_constraint.is_satisfied_by(output_count) {
                return Err(ValidationError::CellCountViolation {
                    cell_type: rule.cell_type.clone(),
                    is_input: false,
                    expected: rule.output_constraint.description(),
                    actual: output_count,
                });
            }
        }
        
        // Check unidentified cells
        if !self.allow_unidentified {
            if !input_cells.unidentified_cells.is_empty() {
                return Err(ValidationError::UnidentifiedCells {
                    is_input: true,
                    count: input_cells.unidentified_cells.len(),
                });
            }
            if !output_cells.unidentified_cells.is_empty() {
                return Err(ValidationError::UnidentifiedCells {
                    is_input: false,
                    count: output_cells.unidentified_cells.len(),
                });
            }
        }
        
        // Run custom validator if provided
        if let Some(validator) = self.custom_validator {
            validator(recipe, input_cells, output_cells)
                .map_err(|msg| ValidationError::CustomValidation(msg))?;
        }
        
        Ok(())
    }
}


/// Registry for transaction validation rules
pub struct ValidationRegistry {
    rules: BTreeMap<Vec<u8>, TransactionValidationRules>,
}

impl ValidationRegistry {
    /// Create a new validation registry
    pub fn new() -> Self {
        Self {
            rules: BTreeMap::new(),
        }
    }
    
    /// Register validation rules for a method
    pub fn register(&mut self, rules: TransactionValidationRules) {
        self.rules.insert(rules.method_path.clone(), rules);
    }
    
    /// Get validation rules for a method path
    pub fn get(&self, method_path: &[u8]) -> Option<&TransactionValidationRules> {
        self.rules.get(method_path)
    }
    
    /// Validate a transaction
    pub fn validate(
        &self,
        recipe: &TransactionRecipe,
        input_cells: &ClassifiedCells,
        output_cells: &ClassifiedCells,
    ) -> Result<(), ValidationError> {
        let method_path = recipe.method_path_bytes();
        
        if let Some(rules) = self.get(&method_path) {
            rules.validate(recipe, input_cells, output_cells)
        } else {
            // No rules registered for this method - could be OK or error depending on policy
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cell_count_constraint() {
        let exactly_one = CellCountConstraint::exactly(1);
        assert!(exactly_one.is_satisfied_by(1));
        assert!(!exactly_one.is_satisfied_by(0));
        assert!(!exactly_one.is_satisfied_by(2));
        
        let at_least_two = CellCountConstraint::at_least(2);
        assert!(!at_least_two.is_satisfied_by(1));
        assert!(at_least_two.is_satisfied_by(2));
        assert!(at_least_two.is_satisfied_by(3));
        
        let range = CellCountConstraint::range(1, 3);
        assert!(!range.is_satisfied_by(0));
        assert!(range.is_satisfied_by(1));
        assert!(range.is_satisfied_by(2));
        assert!(range.is_satisfied_by(3));
        assert!(!range.is_satisfied_by(4));
    }
    
    #[test]
    fn test_validation_rules_builder() {
        let rules = TransactionValidationRules::new(b"test.method")
            .with_arguments(2)
            .with_known_cell(b"udt", CellCountConstraint::at_least(1), CellCountConstraint::any())
            .with_custom_cell(b"pool", CellCountConstraint::exactly(1), CellCountConstraint::exactly(1))
            .allow_unidentified();
            
        assert_eq!(rules.method_path, b"test.method");
        assert_eq!(rules.expected_arguments, Some(2));
        assert_eq!(rules.cell_rules.len(), 2);
        assert!(rules.allow_unidentified);
    }
}