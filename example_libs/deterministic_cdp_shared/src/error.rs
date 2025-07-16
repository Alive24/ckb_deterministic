#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    // Business logic errors (1-20)
    InvalidArguments = 1,
    InsufficientCapacity = 2,
    Unauthorized = 3,
    UnderCollateralized = 4,
    
    // Framework errors (21-40)
    DataError = 21,
    RecipeError = 22,
    UnidentifiedCells = 23,
    
    // Validation errors (41-60)
    WrongMethodPath = 41,
    InvalidArgumentCount = 42,
    MissingRequiredCells = 43,
    TooManyCells = 44,
    UnexpectedCellType = 45,
    CustomValidationFailed = 46,
    MissingCellDep = 47,
    MissingHeaderDep = 48,
    InvalidDepGroup = 49,
    
    // System errors (61-80)
    SystemError = 61,
    IndexOutOfBound = 62,
    ItemMissing = 63,
    Encoding = 64,
    
    // Generic fallback
    Unknown = 99,
}

// Convert from ckb_deterministic errors with detailed mapping
impl From<ckb_deterministic::errors::Error> for Error {
    fn from(err: ckb_deterministic::errors::Error) -> Self {
        use ckb_deterministic::errors::Error::*;
        match err {
            DataError => Error::DataError,
            UnidentifiedCells => Error::UnidentifiedCells,
            RecipeError => Error::RecipeError,
            SystemError(code) => match code {
                -1 => Error::IndexOutOfBound,
                -2 => Error::ItemMissing,
                -4 => Error::Encoding,
                _ => Error::SystemError,
            },
            ValidationError(validation_err) => Error::from(validation_err),
        }
    }
}

// Convert from ckb_deterministic validation errors with specific codes
impl From<ckb_deterministic::errors::ValidationError> for Error {
    fn from(err: ckb_deterministic::errors::ValidationError) -> Self {
        use ckb_deterministic::errors::ValidationError::*;
        match err {
            WrongMethodPath { .. } => Error::WrongMethodPath,
            InvalidArgumentCount { .. } => Error::InvalidArgumentCount,
            CellCountViolation { is_input: true, .. } => Error::MissingRequiredCells,
            CellCountViolation { is_input: false, .. } => Error::TooManyCells,
            UnidentifiedCells { .. } => Error::UnexpectedCellType,
            CustomValidation(_) => Error::CustomValidationFailed,
            MissingCellDep { .. } => Error::MissingCellDep,
            MissingHeaderDep { .. } => Error::MissingHeaderDep,
            InvalidDepGroup { .. } => Error::InvalidDepGroup,
        }
    }
}