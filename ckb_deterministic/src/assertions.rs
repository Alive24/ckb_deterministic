//! Jest-like assertion framework for CKB transaction validation
//! 
//! This module provides a fluent API for writing expressive assertions
//! in custom validators, similar to Jest's expect/assert patterns.

extern crate alloc;
use alloc::{format, string::{String, ToString}, vec::Vec, ffi};
use crate::{
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
pub fn assert(condition: bool, message: &str) -> Result<(), String> {
    if condition {
        Ok(())
    } else {
        Err(message.to_string())
    }
}

/// Assertion function that includes the actual value in the error message
pub fn assert_eq<T: core::fmt::Debug + PartialEq>(actual: T, expected: T, context: &str) -> Result<(), String> {
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{}: expected {:?}, got {:?}", context, expected, actual))
    }
}

/// Wrapper for values being tested
pub struct Expectation<T> {
    actual: T,
}

impl<T> Expectation<T> {
    /// Create a custom matcher
    pub fn to_satisfy<F>(self, matcher: F, message: &str) -> Result<(), String>
    where
        F: FnOnce(&T) -> bool,
    {
        if matcher(&self.actual) {
            Ok(())
        } else {
            Err(message.to_string())
        }
    }

    /// Create a custom matcher with detailed error
    pub fn to_satisfy_with_error<F>(self, matcher: F) -> Result<(), String>
    where
        F: FnOnce(&T) -> Result<(), String>,
    {
        matcher(&self.actual)
    }
}

// Numeric matchers
impl<T> Expectation<T>
where
    T: PartialOrd + core::fmt::Display,
{
    pub fn to_be_greater_than(self, expected: T) -> Result<(), String> {
        if self.actual > expected {
            Ok(())
        } else {
            Err(format!("Expected {} to be greater than {}", self.actual, expected))
        }
    }

    pub fn to_be_greater_than_or_equal(self, expected: T) -> Result<(), String> {
        if self.actual >= expected {
            Ok(())
        } else {
            Err(format!("Expected {} to be greater than or equal to {}", self.actual, expected))
        }
    }

    pub fn to_be_less_than(self, expected: T) -> Result<(), String> {
        if self.actual < expected {
            Ok(())
        } else {
            Err(format!("Expected {} to be less than {}", self.actual, expected))
        }
    }

    pub fn to_be_less_than_or_equal(self, expected: T) -> Result<(), String> {
        if self.actual <= expected {
            Ok(())
        } else {
            Err(format!("Expected {} to be less than or equal to {}", self.actual, expected))
        }
    }

    pub fn to_be_in_range(self, min: T, max: T) -> Result<(), String> {
        if self.actual >= min && self.actual <= max {
            Ok(())
        } else {
            Err(format!("Expected {} to be between {} and {}", self.actual, min, max))
        }
    }
}

// Equality matchers
impl<T> Expectation<T>
where
    T: PartialEq + core::fmt::Debug,
{
    pub fn to_equal(self, expected: T) -> Result<(), String> {
        if self.actual == expected {
            Ok(())
        } else {
            Err(format!("Expected {:?} to equal {:?}", self.actual, expected))
        }
    }

    pub fn not_to_equal(self, expected: T) -> Result<(), String> {
        if self.actual != expected {
            Ok(())
        } else {
            Err(format!("Expected {:?} not to equal {:?}", self.actual, expected))
        }
    }
}

// Boolean matchers
impl Expectation<bool> {
    pub fn to_be_true(self) -> Result<(), String> {
        if self.actual {
            Ok(())
        } else {
            Err("Expected true but got false".to_string())
        }
    }

    pub fn to_be_false(self) -> Result<(), String> {
        if !self.actual {
            Ok(())
        } else {
            Err("Expected false but got true".to_string())
        }
    }
}

// Vec/Slice matchers
impl<T> Expectation<Vec<T>> {
    pub fn to_have_length(self, expected: usize) -> Result<(), String> {
        if self.actual.len() == expected {
            Ok(())
        } else {
            Err(format!("Expected length {} but got {}", expected, self.actual.len()))
        }
    }

    pub fn to_be_empty(self) -> Result<(), String> {
        if self.actual.is_empty() {
            Ok(())
        } else {
            Err(format!("Expected empty collection but got {} items", self.actual.len()))
        }
    }

    pub fn not_to_be_empty(self) -> Result<(), String> {
        if !self.actual.is_empty() {
            Ok(())
        } else {
            Err("Expected non-empty collection but got empty".to_string())
        }
    }
}

// Slice matchers
impl<T> Expectation<&[T]> {
    pub fn to_have_length(self, expected: usize) -> Result<(), String> {
        if self.actual.len() == expected {
            Ok(())
        } else {
            Err(format!("Expected length {} but got {}", expected, self.actual.len()))
        }
    }

    pub fn to_be_empty(self) -> Result<(), String> {
        if self.actual.is_empty() {
            Ok(())
        } else {
            Err(format!("Expected empty slice but got {} items", self.actual.len()))
        }
    }

    pub fn not_to_be_empty(self) -> Result<(), String> {
        if !self.actual.is_empty() {
            Ok(())
        } else {
            Err("Expected non-empty slice but got empty".to_string())
        }
    }
}

// Reference to Vec matchers
impl<T> Expectation<&Vec<T>> {
    pub fn to_have_length(self, expected: usize) -> Result<(), String> {
        if self.actual.len() == expected {
            Ok(())
        } else {
            Err(format!("Expected length {} but got {}", expected, self.actual.len()))
        }
    }

    pub fn to_be_empty(self) -> Result<(), String> {
        if self.actual.is_empty() {
            Ok(())
        } else {
            Err(format!("Expected empty collection but got {} items", self.actual.len()))
        }
    }

    pub fn not_to_be_empty(self) -> Result<(), String> {
        if !self.actual.is_empty() {
            Ok(())
        } else {
            Err("Expected non-empty collection but got empty".to_string())
        }
    }
}

// Option matchers
impl<T> Expectation<Option<T>> {
    pub fn to_be_some(self) -> Result<(), String> {
        if self.actual.is_some() {
            Ok(())
        } else {
            Err("Expected Some but got None".to_string())
        }
    }

    pub fn to_be_none(self) -> Result<(), String> {
        if self.actual.is_none() {
            Ok(())
        } else {
            Err("Expected None but got Some".to_string())
        }
    }
}

impl<T: core::fmt::Debug + PartialEq> Expectation<Option<T>> {
    pub fn to_be_some_and_equal(self, expected: T) -> Result<(), String> {
        match self.actual {
            Some(val) if val == expected => Ok(()),
            Some(val) => Err(format!("Expected Some({:?}) but got Some({:?})", expected, val)),
            None => Err(format!("Expected Some({:?}) but got None", expected)),
        }
    }
}

// Result matchers
impl<T, E> Expectation<Result<T, E>> {
    pub fn to_be_ok(self) -> Result<(), String> {
        if self.actual.is_ok() {
            Ok(())
        } else {
            Err("Expected Ok but got Err".to_string())
        }
    }

    pub fn to_be_err(self) -> Result<(), String> {
        if self.actual.is_err() {
            Ok(())
        } else {
            Err("Expected Err but got Ok".to_string())
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

    pub fn to_have_method_path(self, expected: &[u8]) -> Result<(), String> {
        let actual = self.recipe.method_path().raw_data();
        if actual.as_ref() == expected {
            Ok(())
        } else {
            Err(format!(
                "Expected method path {:?} but got {:?}",
                expected,
                actual.as_ref()
            ))
        }
    }

    pub fn to_have_arguments_count(self, expected: usize) -> Result<(), String> {
        let args = self.recipe.arguments_vec();
        if args.len() == expected {
            Ok(())
        } else {
            Err(format!("Expected {} arguments but got {}", expected, args.len()))
        }
    }

    pub fn to_have_argument_with_length(self, index: usize, expected_length: usize) -> Result<(), String> {
        let args = self.recipe.arguments_vec();
        match args.get(index) {
            Some(arg) if arg.len() == expected_length => Ok(()),
            Some(arg) => Err(format!(
                "Argument {} has length {} but expected {}",
                index,
                arg.len(),
                expected_length
            )),
            None => Err(format!("Argument {} does not exist", index)),
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

    pub fn to_have_known_cells(self, cell_type: &str) -> Result<(), String> {
        if self.cells.get_known(cell_type).is_some() {
            Ok(())
        } else {
            Err(format!("Expected to have known cells of type {}", cell_type))
        }
    }

    pub fn to_have_custom_cells(self, cell_type: &[u8]) -> Result<(), String> {
        if self.cells.get_custom(cell_type).is_some() {
            Ok(())
        } else {
            Err(format!("Expected to have custom cells of type {:?}", cell_type))
        }
    }

    pub fn to_have_known_cells_count(self, cell_type: &str, expected: usize) -> Result<(), String> {
        let count = self.cells
            .get_known(cell_type)
            .map(|cells| cells.len())
            .unwrap_or(0);
        
        if count == expected {
            Ok(())
        } else {
            Err(format!(
                "Expected {} known cells of type {} but got {}",
                expected, cell_type, count
            ))
        }
    }

    pub fn to_have_custom_cells_count(self, cell_type: &[u8], expected: usize) -> Result<(), String> {
        let count = self.cells
            .get_custom(cell_type)
            .map(|cells| cells.len())
            .unwrap_or(0);
        
        if count == expected {
            Ok(())
        } else {
            Err(format!(
                "Expected {} custom cells of type {:?} but got {}",
                expected, cell_type, count
            ))
        }
    }

    pub fn to_have_total_cells_count(self, expected: usize) -> Result<(), String> {
        let count = self.cells.total_cell_count();
        if count == expected {
            Ok(())
        } else {
            Err(format!("Expected {} total cells but got {}", expected, count))
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
pub fn expect_arguments(recipe: &TransactionRecipe) -> Result<Vec<Vec<u8>>, String> {
    let args = recipe.arguments_vec();
    if args.is_empty() {
        Err("No arguments provided".to_string())
    } else {
        Ok(args)
    }
}

pub fn expect_u64_argument(arg: &[u8], name: &str) -> Result<u64, String> {
    if arg.len() != 8 {
        return Err(format!("{} must be 8-byte u64, got {} bytes", name, arg.len()));
    }
    
    u64::from_le_bytes(
        arg[..8].try_into()
            .map_err(|_| format!("Invalid {} format", name))?
    )
    .to_result()
}

pub fn expect_u128_argument(arg: &[u8], name: &str) -> Result<u128, String> {
    if arg.len() != 16 {
        return Err(format!("{} must be 16-byte u128, got {} bytes", name, arg.len()));
    }
    
    u128::from_le_bytes(
        arg[..16].try_into()
            .map_err(|_| format!("Invalid {} format", name))?
    )
    .to_result()
}

// Extension trait to convert values to Result for chaining
trait ToResult {
    fn to_result(self) -> Result<Self, String>
    where
        Self: Sized;
}

impl<T> ToResult for T {
    fn to_result(self) -> Result<Self, String> {
        Ok(self)
    }
}

// Macro for creating custom validation blocks with descriptive names
#[macro_export]
macro_rules! validation_block {
    ($name:expr, $block:block) => {{
        (|| -> Result<(), String> {
            $block
        })()
        .map_err(|e| format!("{}: {}", $name, e))
    }};
}

// Macro for chaining multiple validations
#[macro_export]
macro_rules! validate_all {
    ($($validation:expr),+ $(,)?) => {{
        $(
            $validation?;
        )+
        Ok::<(), String>(())
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
    pub fn to_have_cell_dep(self, tx_hash: &[u8; 32], index: u32) -> Result<(), String> {
        let found = self.deps.iter().any(|dep| {
            &dep.out_point.tx_hash == tx_hash && dep.out_point.index == index
        });
        
        if found {
            Ok(())
        } else {
            Err(format!(
                "Expected cell dep with tx_hash: 0x{}, index: {} not found",
                ckb_std::high_level::encode_hex(tx_hash).into_string().unwrap_or_else(|_| "<invalid_hex>".to_string()),
                index
            ))
        }
    }
    
    /// Assert that a specific dep group exists
    pub fn to_have_dep_group(self, tx_hash: &[u8; 32], index: u32) -> Result<(), String> {
        let found = self.deps.iter().any(|dep| {
            &dep.out_point.tx_hash == tx_hash && 
            dep.out_point.index == index &&
            dep.dep_type == DepType::DepGroup
        });
        
        if found {
            Ok(())
        } else {
            Err(format!(
                "Expected dep group with tx_hash: 0x{}, index: {} not found",
                ckb_std::high_level::encode_hex(tx_hash).into_string().unwrap_or_else(|_| "<invalid_hex>".to_string()),
                index
            ))
        }
    }
    
    /// Assert that deps for a known script are present
    pub fn to_have_deps_for_script(self, script: KnownScript, network: Network) -> Result<(), String> {
        if let Some(script_info) = get_script_info(script, network) {
            for (tx_hash_str, index, dep_type_u8) in &script_info.cell_deps {
                // Convert hex string to bytes
                let tx_hash = hex_to_bytes(tx_hash_str)
                    .map_err(|_| format!("Invalid hex in script info: {}", tx_hash_str))?;
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
                    return Err(format!(
                        "Missing dependency for {}: tx_hash: 0x{}, index: {}, type: {:?}",
                        script.identifier(),
                        ckb_std::high_level::encode_hex(&tx_hash).into_string().unwrap_or_else(|_| "<invalid_hex>".to_string()),
                        index,
                        dep_type
                    ));
                }
            }
        }
        Ok(())
    }
    
    /// Assert that the number of deps matches
    pub fn to_have_count(self, expected: usize) -> Result<(), String> {
        if self.deps.len() == expected {
            Ok(())
        } else {
            Err(format!(
                "Expected {} cell deps, but found {}",
                expected,
                self.deps.len()
            ))
        }
    }
}

/// Expectation for header dependencies
pub struct HeadersExpectation<'a> {
    headers: &'a [[u8; 32]],
}

impl<'a> HeadersExpectation<'a> {
    /// Assert that a specific header is included
    pub fn to_have_header(self, header_hash: &[u8; 32]) -> Result<(), String> {
        if self.headers.contains(header_hash) {
            Ok(())
        } else {
            Err(format!(
                "Expected header dep with hash: 0x{} not found",
                ckb_std::high_level::encode_hex(header_hash).into_string().unwrap_or_else(|_| "<invalid_hex>".to_string())
            ))
        }
    }
    
    /// Assert that the number of headers matches
    pub fn to_have_count(self, expected: usize) -> Result<(), String> {
        if self.headers.len() == expected {
            Ok(())
        } else {
            Err(format!(
                "Expected {} header deps, but found {}",
                expected,
                self.headers.len()
            ))
        }
    }
}

/// Helper to convert hex string to bytes using ckb-std
fn hex_to_bytes(hex: &str) -> Result<[u8; 32], String> {
    use ckb_std::high_level::decode_hex;
    
    let hex = hex.trim_start_matches("0x");
    
    // Convert to CString for ckb-std decode_hex
    let hex_cstr = ffi::CString::new(hex)
        .map_err(|_| "Invalid hex string: contains null bytes".to_string())?;
    
    let decoded = decode_hex(&hex_cstr)
        .map_err(|_| "Failed to decode hex string".to_string())?;
    
    if decoded.len() != 32 {
        return Err(format!("Invalid hash length: expected 32 bytes, got {}", decoded.len()));
    }
    
    let mut result = [0u8; 32];
    result.copy_from_slice(&decoded);
    Ok(result)
}