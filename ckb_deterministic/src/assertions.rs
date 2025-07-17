//! Jest-like assertion framework for CKB transaction validation
//! 
//! This module provides a fluent API for writing expressive assertions
//! in custom validators, similar to Jest's expect/assert patterns.

extern crate alloc;
use alloc::{vec::Vec, ffi};
use crate::{
    errors::Error,
    generated::TransactionRecipe,
    cell_classifier::ClassifiedCells,
    transaction_recipe::TransactionRecipeExt,
    transaction_deps::{CellDepInfo, DepType},
    known_scripts::{KnownScript, Network, get_script_info},
};

/// Main entry point for assertions - creates an expectation on a value
pub fn expect<T>(actual: T) -> Expectation<T> {
    Expectation { actual }
}

/// Direct assertion function for simple boolean checks
pub fn assert(condition: bool, _message: &str) -> Result<(), Error> {
    if condition {
        Ok(())
    } else {
        Err(Error::ExpectationViolation)
    }
}

/// Assertion function that includes the actual value in the error message
pub fn assert_eq<T: core::fmt::Debug + PartialEq>(actual: T, expected: T, _context: &str) -> Result<(), Error> {
    if actual == expected {
        Ok(())
    } else {
        Err(Error::ExpectationViolation)
    }
}

/// Wrapper for values being tested
pub struct Expectation<T> {
    actual: T,
}

impl<T> Expectation<T> {
    /// Create a custom matcher
    pub fn to_satisfy<F>(self, matcher: F, _message: &str) -> Result<(), Error>
    where
        F: FnOnce(&T) -> bool,
    {
        if matcher(&self.actual) {
            Ok(())
        } else {
            Err(Error::ExpectationViolation)
        }
    }

    /// Create a custom matcher with detailed error
    pub fn to_satisfy_with_error<F>(self, matcher: F) -> Result<(), Error>
    where
        F: FnOnce(&T) -> Result<(), Error>,
    {
        matcher(&self.actual)
    }
}

// Numeric matchers
impl<T> Expectation<T>
where
    T: PartialOrd + core::fmt::Display,
{
    pub fn to_be_greater_than(self, _expected: T) -> Result<(), Error> {
        if self.actual > _expected {
            Ok(())
        } else {
            Err(Error::ExpectationViolation)
        }
    }

    pub fn to_be_greater_than_or_equal(self, expected: T) -> Result<(), Error> {
        if self.actual >= expected {
            Ok(())
        } else {
            Err(Error::ExpectationViolation)
        }
    }

    pub fn to_be_less_than(self, expected: T) -> Result<(), Error> {
        if self.actual < expected {
            Ok(())
        } else {
            Err(Error::ExpectationViolation)
        }
    }

    pub fn to_be_less_than_or_equal(self, expected: T) -> Result<(), Error> {
        if self.actual <= expected {
            Ok(())
        } else {
            Err(Error::ExpectationViolation)
        }
    }

    pub fn to_be_in_range(self, min: T, max: T) -> Result<(), Error> {
        if self.actual >= min && self.actual <= max {
            Ok(())
        } else {
            Err(Error::ExpectationViolation)
        }
    }
}

// Equality matchers
impl<T> Expectation<T>
where
    T: PartialEq + core::fmt::Debug,
{
    pub fn to_equal(self, expected: T) -> Result<(), Error> {
        if self.actual == expected {
            Ok(())
        } else {
            Err(Error::ExpectationViolation)
        }
    }

    pub fn not_to_equal(self, expected: T) -> Result<(), Error> {
        if self.actual != expected {
            Ok(())
        } else {
            Err(Error::ExpectationViolation)
        }
    }
}

// Boolean matchers
impl Expectation<bool> {
    pub fn to_be_true(self) -> Result<(), Error> {
        if self.actual {
            Ok(())
        } else {
            Err(Error::ExpectationViolation)
        }
    }

    pub fn to_be_false(self) -> Result<(), Error> {
        if !self.actual {
            Ok(())
        } else {
            Err(Error::ExpectationViolation)
        }
    }
}

// Vec/Slice matchers
impl<T> Expectation<Vec<T>> {
    pub fn to_have_length(self, expected: usize) -> Result<(), Error> {
        if self.actual.len() == expected {
            Ok(())
        } else {
            Err(Error::CellCountViolation)
        }
    }

    pub fn to_be_empty(self) -> Result<(), Error> {
        if self.actual.is_empty() {
            Ok(())
        } else {
            Err(Error::CellCountViolation)
        }
    }

    pub fn not_to_be_empty(self) -> Result<(), Error> {
        if !self.actual.is_empty() {
            Ok(())
        } else {
            Err(Error::CellCountViolation)
        }
    }
}

// Slice matchers
impl<T> Expectation<&[T]> {
    pub fn to_have_length(self, expected: usize) -> Result<(), Error> {
        if self.actual.len() == expected {
            Ok(())
        } else {
            Err(Error::CellCountViolation)
        }
    }

    pub fn to_be_empty(self) -> Result<(), Error> {
        if self.actual.is_empty() {
            Ok(())
        } else {
            Err(Error::CellCountViolation)
        }
    }

    pub fn not_to_be_empty(self) -> Result<(), Error> {
        if !self.actual.is_empty() {
            Ok(())
        } else {
            Err(Error::CellCountViolation)
        }
    }
}

// Reference to Vec matchers
impl<T> Expectation<&Vec<T>> {
    pub fn to_have_length(self, expected: usize) -> Result<(), Error> {
        if self.actual.len() == expected {
            Ok(())
        } else {
            Err(Error::CellCountViolation)
        }
    }

    pub fn to_be_empty(self) -> Result<(), Error> {
        if self.actual.is_empty() {
            Ok(())
        } else {
            Err(Error::CellCountViolation)
        }
    }

    pub fn not_to_be_empty(self) -> Result<(), Error> {
        if !self.actual.is_empty() {
            Ok(())
        } else {
            Err(Error::CellCountViolation)
        }
    }
}

// Option matchers
impl<T> Expectation<Option<T>> {
    pub fn to_be_some(self) -> Result<(), Error> {
        if self.actual.is_some() {
            Ok(())
        } else {
            Err(Error::ExpectationViolation)
        }
    }

    pub fn to_be_none(self) -> Result<(), Error> {
        if self.actual.is_none() {
            Ok(())
        } else {
            Err(Error::ExpectationViolation)
        }
    }
}

impl<T: core::fmt::Debug + PartialEq> Expectation<Option<T>> {
    pub fn to_be_some_and_equal(self, expected: T) -> Result<(), Error> {
        match self.actual {
            Some(val) if val == expected => Ok(()),
            Some(_val) => Err(Error::ExpectationViolation),
            None => Err(Error::ExpectationViolation),
        }
    }
}

// Result matchers
impl<T, E> Expectation<Result<T, E>> {
    pub fn to_be_ok(self) -> Result<(), Error> {
        if self.actual.is_ok() {
            Ok(())
        } else {
            Err(Error::ExpectationViolation)
        }
    }

    pub fn to_be_err(self) -> Result<(), Error> {
        if self.actual.is_err() {
            Ok(())
        } else {
            Err(Error::ExpectationViolation)
        }
    }
}

// CKB-specific matchers for TransactionRecipe
pub struct TransactionExpectation<'a> {
    recipe: &'a TransactionRecipe,
}

impl<'a> TransactionExpectation<'a> {
    pub fn new(recipe: &'a TransactionRecipe) -> Self {
        Self { recipe }
    }

    pub fn to_have_method_path(self, expected: &[u8]) -> Result<(), Error> {
        let actual = self.recipe.method_path().raw_data();
        if actual.as_ref() == expected {
            Ok(())
        } else {
            Err(Error::WrongMethodPath)
        }
    }

    pub fn to_have_arguments_count(self, expected: usize) -> Result<(), Error> {
        let args = self.recipe.arguments_vec();
        if args.len() == expected {
            Ok(())
        } else {
            Err(Error::InvalidArgumentCount)
        }
    }

    pub fn to_have_argument_with_length(self, index: usize, expected_length: usize) -> Result<(), Error> {
        let args = self.recipe.arguments_vec();
        match args.get(index) {
            Some(arg) if arg.len() == expected_length => Ok(()),
            Some(_arg) => Err(Error::InvalidArgumentCount),
            None => Err(Error::InvalidArgumentCount),
        }
    }
}

// CKB-specific matchers for ClassifiedCells
pub struct CellsExpectation<'a> {
    cells: &'a ClassifiedCells,
}

impl<'a> CellsExpectation<'a> {
    pub fn new(cells: &'a ClassifiedCells) -> Self {
        Self { cells }
    }

    pub fn to_have_known_cells(self, cell_type: &str) -> Result<(), Error> {
        if self.cells.get_known(cell_type).is_some() {
            Ok(())
        } else {
            Err(Error::CellCountViolation)
        }
    }

    pub fn to_have_custom_cells(self, cell_type: &str) -> Result<(), Error> {
        if self.cells.get_custom(cell_type).is_some() {
            Ok(())
        } else {
            Err(Error::CellCountViolation)
        }
    }

    pub fn to_have_known_cells_count(self, cell_type: &str, expected: usize) -> Result<(), Error> {
        let count = self.cells
            .get_known(cell_type)
            .map(|cells| cells.len())
            .unwrap_or(0);
        
        if count == expected {
            Ok(())
        } else {
            Err(Error::CellCountViolation)
        }
    }

    pub fn to_have_custom_cells_count(self, cell_type: &str, expected: usize) -> Result<(), Error> {
        let count = self.cells
            .get_custom(cell_type)
            .map(|cells| cells.len())
            .unwrap_or(0);
        
        if count == expected {
            Ok(())
        } else {
            Err(Error::CellCountViolation)
        }
    }

    pub fn to_have_total_cells_count(self, expected: usize) -> Result<(), Error> {
        let count = self.cells.total_cell_count();
        if count == expected {
            Ok(())
        } else {
            Err(Error::CellCountViolation)
        }
    }
}

// Helper functions for CKB-specific expectations
pub fn expect_transaction(recipe: &TransactionRecipe) -> TransactionExpectation {
    TransactionExpectation::new(recipe)
}

pub fn expect_cells(cells: &ClassifiedCells) -> CellsExpectation {
    CellsExpectation::new(cells)
}

// Common validation helpers
pub fn expect_arguments(recipe: &TransactionRecipe) -> Result<Vec<Vec<u8>>, Error> {
    let args = recipe.arguments_vec();
    if args.is_empty() {
        Err(Error::InvalidArgumentCount)
    } else {
        Ok(args)
    }
}

pub fn expect_u64_argument(arg: &[u8], _name: &str) -> Result<u64, Error> {
    if arg.len() != 8 {
        return Err(Error::InvalidArgumentCount);
    }
    
    u64::from_le_bytes(
        arg[..8].try_into()
            .map_err(|_| Error::InvalidArgumentCount)?
    )
    .to_result()
}

pub fn expect_u128_argument(arg: &[u8], _name: &str) -> Result<u128, Error> {
    if arg.len() != 16 {
        return Err(Error::InvalidArgumentCount);
    }
    
    u128::from_le_bytes(
        arg[..16].try_into()
            .map_err(|_| Error::InvalidArgumentCount)?
    )
    .to_result()
}

// Extension trait to convert values to Result for chaining
trait ToResult {
    fn to_result(self) -> Result<Self, Error>
    where
        Self: Sized;
}

impl<T> ToResult for T {
    fn to_result(self) -> Result<Self, Error> {
        Ok(self)
    }
}

// Macro for creating custom validation blocks with descriptive names
#[macro_export]
macro_rules! validation_block {
    ($name:expr, $block:block) => {{
        (|| -> Result<(), Error> {
            $block
        })()
        .map_err(|_e| Error::ExpectationViolation)
    }};
}

// Macro for chaining multiple validations
#[macro_export]
macro_rules! validate_all {
    ($($validation:expr),+ $(,)?) => {{
        $(
            $validation?;
        )+
        Ok::<(), Error>(())
    }};
}

/// Create expectation for cell dependencies
pub fn expect_deps(deps: &[CellDepInfo]) -> DepsExpectation {
    DepsExpectation { deps }
}

/// Create expectation for header dependencies
pub fn expect_headers(headers: &[[u8; 32]]) -> HeadersExpectation {
    HeadersExpectation { headers }
}

/// Expectation for cell dependencies
pub struct DepsExpectation<'a> {
    deps: &'a [CellDepInfo],
}

impl<'a> DepsExpectation<'a> {
    /// Assert that a specific cell dep exists
    pub fn to_have_cell_dep(self, tx_hash: &[u8; 32], index: u32) -> Result<(), Error> {
        let found = self.deps.iter().any(|dep| {
            &dep.out_point.tx_hash == tx_hash && dep.out_point.index == index
        });
        
        if found {
            Ok(())
        } else {
            Err(Error::MissingCellDep)
        }
    }
    
    /// Assert that a specific dep group exists
    pub fn to_have_dep_group(self, tx_hash: &[u8; 32], index: u32) -> Result<(), Error> {
        let found = self.deps.iter().any(|dep| {
            &dep.out_point.tx_hash == tx_hash && 
            dep.out_point.index == index &&
            dep.dep_type == DepType::DepGroup
        });
        
        if found {
            Ok(())
        } else {
            Err(Error::InvalidDepGroup)
        }
    }
    
    /// Assert that deps for a known script are present
    pub fn to_have_deps_for_script(self, script: KnownScript, network: Network) -> Result<(), Error> {
        if let Some(script_info) = get_script_info(script, network) {
            for (tx_hash_str, index, dep_type_u8) in &script_info.cell_deps {
                // Convert hex string to bytes
                let tx_hash = hex_to_bytes(tx_hash_str)
                    .map_err(|_| Error::MissingCellDep)?;
                let dep_type = match *dep_type_u8 {
                    0 => DepType::Code,
                    1 => DepType::DepGroup,
                    _ => DepType::Code,
                };
                
                let found = self.deps.iter().any(|dep| {
                    dep.out_point.tx_hash == tx_hash && 
                    dep.out_point.index == *index &&
                    dep.dep_type == dep_type
                });
                
                if !found {
                    return Err(Error::MissingCellDep);
                }
            }
        }
        Ok(())
    }
    
    /// Assert that the number of deps matches
    pub fn to_have_count(self, expected: usize) -> Result<(), Error> {
        if self.deps.len() == expected {
            Ok(())
        } else {
            Err(Error::MissingCellDep)
        }
    }
}

/// Expectation for header dependencies
pub struct HeadersExpectation<'a> {
    headers: &'a [[u8; 32]],
}

impl<'a> HeadersExpectation<'a> {
    /// Assert that a specific header is included
    pub fn to_have_header(self, header_hash: &[u8; 32]) -> Result<(), Error> {
        if self.headers.contains(header_hash) {
            Ok(())
        } else {
            Err(Error::MissingHeaderDep)
        }
    }
    
    /// Assert that the number of headers matches
    pub fn to_have_count(self, expected: usize) -> Result<(), Error> {
        if self.headers.len() == expected {
            Ok(())
        } else {
            Err(Error::MissingHeaderDep)
        }
    }
}

/// Helper to convert hex string to bytes using ckb-std
fn hex_to_bytes(hex: &str) -> Result<[u8; 32], Error> {
    use ckb_std::high_level::decode_hex;
    
    let hex = hex.trim_start_matches("0x");
    
    // Convert to CString for ckb-std decode_hex
    let hex_cstr = ffi::CString::new(hex)
        .map_err(|_| Error::DataError)?;
    
    let decoded = decode_hex(&hex_cstr)
        .map_err(|_| Error::DataError)?;
    
    if decoded.len() != 32 {
        return Err(Error::DataError);
    }
    
    let mut result = [0u8; 32];
    result.copy_from_slice(&decoded);
    Ok(result)
}