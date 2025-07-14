#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidArguments = 1,
    InsufficientCapacity = 2,
    Unauthorized = 3,
    UnderCollateralized = 4,
    SystemError = 5,
}