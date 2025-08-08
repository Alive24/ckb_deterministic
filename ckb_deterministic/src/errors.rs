//! Error types for the CKB Deterministic framework.
//! 
//! This module defines all error types that can occur during contract execution,
//! including CKB system errors, data parsing errors, and validation errors.

extern crate alloc;

/// Comprehensive error enum for all framework operations.
/// 
/// Error codes are designed to be returned as i8 exit codes from CKB contracts.
/// Codes are grouped by category:
/// - 1-9: CKB system errors
/// - 10-19: Rust/encoding errors  
/// - 20-29: Data and parsing errors
/// - 30-39: Validation errors
/// - 40-49: Type ID errors
/// - 50-59: Assertion errors
#[repr(i8)]
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    // * CKB Error
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,
    Encoding = 4,
    WaitFailure = 5,
    InvalidFd = 6,
    OtherEndClosed = 7,
    MaxVmsSpawned = 8,
    MaxFdsCreated = 9,

    // * Rust Error
    Utf8Error = 10,
    HexError = 11,

    // * Deterministic Library Errors
    /// Cell data parsing or validation error
    DataError = 20,
    /// Cell classification error - unidentified cells found in strict mode
    UnidentifiedCells = 21,
    /// Recipe parsing error
    RecipeError = 22,
    /// Invalid code hash
    InvalidCodeHash = 23,
    /// Unknown script type
    UnknownScript = 24,

    /* Validation errors */
    /// Wrong method path
    WrongMethodPath = 30,
    /// Invalid argument count
    InvalidArgumentCount = 31,
    /// Cell count constraint violation
    CellCountViolation = 32,
    /// Cell relationship validation failed
    CellRelationshipRuleViolation = 33,
    /// Business rule validation failed
    BusinessRuleViolation = 34,
    /// Additional validation error
    AdditionalRuleValidation = 35,
    /// Missing required cell dependency
    MissingCellDep = 36,
    /// Missing required header dependency
    MissingHeaderDep = 37,
    /// Invalid dep group
    InvalidDepGroup = 38,
    /// Expectation violation (Usually need to be mapped to a specific error)
    ExpectationViolation = 39,
    
    /* Type ID errors */
    /// Multiple Type ID cells found (max 1 input and 1 output allowed)
    TypeIDMultipleCells = 40,
    /// Type ID mismatch when creating new Type ID cell
    TypeIDMismatch = 41,
    
    // Unknown error (catch-all)
    Unknown = -1,
}

impl From<ckb_std::error::SysError> for Error {
    fn from(err: ckb_std::error::SysError) -> Self {
        // Convert SysError to its i8 representation
        match err {
            ckb_std::error::SysError::IndexOutOfBound => Error::IndexOutOfBound,
            ckb_std::error::SysError::ItemMissing => Error::ItemMissing,
            ckb_std::error::SysError::LengthNotEnough(_) => Error::LengthNotEnough,
            ckb_std::error::SysError::Encoding => Error::Encoding,
            ckb_std::error::SysError::WaitFailure => Error::WaitFailure,
            ckb_std::error::SysError::InvalidFd => Error::InvalidFd,
            ckb_std::error::SysError::OtherEndClosed => Error::OtherEndClosed,
            ckb_std::error::SysError::MaxVmsSpawned => Error::MaxVmsSpawned,
            ckb_std::error::SysError::MaxFdsCreated => Error::MaxFdsCreated,
            ckb_std::error::SysError::Unknown(_) => Error::Unknown,
        }
    }
}
