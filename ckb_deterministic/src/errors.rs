extern crate alloc;
use alloc::{string::String, vec::Vec};

/// Errors for the ckb_deterministic library
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// Cell data parsing or validation error
    DataError,
    /// Cell classification error - unidentified cells found in strict mode
    UnidentifiedCells,
    /// Recipe parsing error
    RecipeError,
    /// General CKB system error
    SystemError(i8),
    /// Validation error from the validation framework
    ValidationError(ValidationError),
}

/// Validation error types
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Wrong method path
    WrongMethodPath {
        expected: Vec<u8>,
        actual: Vec<u8>,
    },
    /// Invalid argument count
    InvalidArgumentCount {
        expected: usize,
        actual: usize,
    },
    /// Cell count constraint violation
    CellCountViolation {
        cell_type: Vec<u8>,
        is_input: bool,
        expected: String,
        actual: usize,
    },
    /// Unidentified cells found when not allowed
    UnidentifiedCells {
        is_input: bool,
        count: usize,
    },
    /// Custom validation error with message
    CustomValidation(String),
    /// Missing required cell dependency
    MissingCellDep {
        tx_hash: [u8; 32],
        index: u32,
        dep_type: String,
    },
    /// Missing required header dependency
    MissingHeaderDep {
        header_hash: [u8; 32],
    },
    /// Invalid dep group
    InvalidDepGroup {
        reason: String,
    },
}

impl From<ValidationError> for Error {
    fn from(err: ValidationError) -> Self {
        Error::ValidationError(err)
    }
}

impl From<ckb_std::error::SysError> for Error {
    fn from(err: ckb_std::error::SysError) -> Self {
        // Convert SysError to its i8 representation
        Error::SystemError(match err {
            ckb_std::error::SysError::IndexOutOfBound => -1,
            ckb_std::error::SysError::ItemMissing => -2,
            ckb_std::error::SysError::LengthNotEnough(_) => -3,
            ckb_std::error::SysError::Encoding => -4,
            ckb_std::error::SysError::WaitFailure => -5,
            ckb_std::error::SysError::InvalidFd => -6,
            ckb_std::error::SysError::OtherEndClosed => -7,
            ckb_std::error::SysError::MaxVmsSpawned => -8,
            ckb_std::error::SysError::MaxFdsCreated => -9,
            ckb_std::error::SysError::Unknown(_) => -99,
        })
    }
}