//! CDP-specific error types.
//! 
//! This module defines error types specific to the CDP protocol,
//! mapping errors from the deterministic framework to CDP-specific error codes.
//! Each error has a unique i8 code for efficient on-chain error reporting.
use ckb_deterministic::errors::Error as DeterministicError;
extern crate alloc;

/// Error codes for CDP contract.
/// 
/// Error codes are grouped by category:
/// - 1-9: CKB system errors
/// - 10-19: Rust/encoding errors
/// - 20-29: CDP contract-specific errors
/// - 30-39: Validation errors from deterministic framework
/// - -1: Unknown/catch-all error
#[repr(i8)]
#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    // * CKB Error
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,
    Encoding = 4,
    SpawnExceededMaxContentLength = 5,
    SpawnWrongMemoryLimit = 6,
    SpawnExceededMaxPeakMemory = 7,

    // * Rust Error
    Utf8Error = 10,
    
    // * CDP Contract Errors
    InvalidCodeHash = 20,
    RecipeError = 21,
    
    // * Validation errors from deterministic library
    UnidentifiedCells = 30,
    WrongMethodPath = 31,
    InvalidArgumentCount = 32,
    CellCountViolation = 33,
    CellRelationshipRuleViolation = 34,
    BusinessRuleViolation = 35,
    AdditionalRuleValidation = 36,
    MissingCellDep = 37,
    MissingHeaderDep = 38,
    InvalidDepGroup = 39,
    
    // Unknown error
    Unknown = -1,
}

impl From<DeterministicError> for Error {
    fn from(err: DeterministicError) -> Self {
        match err {
            // Map CKB errors
            DeterministicError::IndexOutOfBound => Error::IndexOutOfBound,
            DeterministicError::ItemMissing => Error::ItemMissing,
            DeterministicError::LengthNotEnough => Error::LengthNotEnough,
            DeterministicError::Encoding => Error::Encoding,
            
            // Map Rust errors
            DeterministicError::Utf8Error => Error::Utf8Error,
            
            // Map deterministic library errors
            DeterministicError::InvalidCodeHash => Error::InvalidCodeHash,
            DeterministicError::RecipeError => Error::RecipeError,
            DeterministicError::UnidentifiedCells => Error::UnidentifiedCells,
            
            // Map validation errors
            DeterministicError::WrongMethodPath => Error::WrongMethodPath,
            DeterministicError::InvalidArgumentCount => Error::InvalidArgumentCount,
            DeterministicError::CellCountViolation => Error::CellCountViolation,
            DeterministicError::CellRelationshipRuleViolation => Error::CellRelationshipRuleViolation,
            DeterministicError::BusinessRuleViolation => Error::BusinessRuleViolation,
            DeterministicError::AdditionalRuleValidation => Error::AdditionalRuleValidation,
            DeterministicError::MissingCellDep => Error::MissingCellDep,
            DeterministicError::MissingHeaderDep => Error::MissingHeaderDep,
            DeterministicError::InvalidDepGroup => Error::InvalidDepGroup,
            
            // All other errors
            _ => Error::Unknown,
        }
    }
}