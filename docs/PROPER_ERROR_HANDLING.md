# Proper Error Handling in CKB Smart Contracts

## Key Principle

Different errors should map to different i8 error codes so the VM and users can understand what went wrong.

## Implementation

### 1. Organized Error Codes

```rust
#[repr(i8)]
pub enum Error {
    // Business logic errors (1-20)
    InvalidArguments = 1,
    UnderCollateralized = 4,
    
    // Validation errors (41-60)
    WrongMethodPath = 41,          // Called unknown method
    InvalidArgumentCount = 42,      // Wrong number of arguments
    MissingRequiredCells = 43,     // Required input cells not found
    TooManyCells = 44,             // Too many output cells
    CustomValidationFailed = 46,   // Business rule failed
    
    // System errors (61-80)
    IndexOutOfBound = 62,
    ItemMissing = 63,
}
```

### 2. Automatic Error Conversion

```rust
// The From trait automatically converts validation errors to specific codes
impl From<ValidationError> for Error {
    fn from(err: ValidationError) -> Self {
        match err {
            WrongMethodPath { .. } => Error::WrongMethodPath,        // 41
            InvalidArgumentCount { .. } => Error::InvalidArgumentCount, // 42
            CellCountViolation { is_input: true, .. } => Error::MissingRequiredCells,  // 43
            CellCountViolation { is_input: false, .. } => Error::TooManyCells,         // 44
            CustomValidation(_) => Error::CustomValidationFailed,     // 46
        }
    }
}
```

### 3. Usage in Contract

```rust
// Don't hide the actual error:
❌ .map_err(|_| Error::InvalidArguments)?;  // Always returns 1

// Properly propagate specific errors:
✅ .map_err(|e| Error::from(e))?;  // Returns 41-46 based on actual error
```

## Benefits

1. **Debugging**: Know exactly what validation failed from the error code
2. **Testing**: Can assert specific error scenarios
3. **User Experience**: Wallets can show meaningful messages
4. **Monitoring**: Track which errors occur most frequently

## Example Error Scenarios

- **Error 41**: Transaction called "CDP.withdraw" but contract doesn't support it
- **Error 42**: Method expects 2 arguments but got 3
- **Error 43**: No xUDT cells found for collateral
- **Error 44**: Created 2 vault outputs instead of 1
- **Error 46**: Collateral ratio 125% is below minimum 150%

This approach ensures errors are informative and actionable, not generic "InvalidArguments".