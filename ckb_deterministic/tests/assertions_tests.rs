use ckb_deterministic::assertions::*;
use ckb_deterministic::errors::Error;
extern crate alloc;
use alloc::{vec, vec::Vec};

#[test]
fn test_basic_equality() {
    assert!(expect(42).to_equal(42).is_ok());
    assert_eq!(expect(42).to_equal(43), Err(Error::ExpectationViolation));
    
    assert!(expect("hello").to_equal("hello").is_ok());
    assert_eq!(expect("hello").to_equal("world"), Err(Error::ExpectationViolation));
}

#[test]
fn test_not_equal() {
    assert!(expect(42).not_to_equal(43).is_ok());
    assert_eq!(expect(42).not_to_equal(42), Err(Error::ExpectationViolation));
}

#[test]
fn test_numeric_comparisons() {
    // Greater than
    assert!(expect(10).to_be_greater_than(5).is_ok());
    assert_eq!(expect(5).to_be_greater_than(10), Err(Error::ExpectationViolation));
    assert_eq!(expect(5).to_be_greater_than(5), Err(Error::ExpectationViolation));
    
    // Greater than or equal
    assert!(expect(10).to_be_greater_than_or_equal(5).is_ok());
    assert!(expect(5).to_be_greater_than_or_equal(5).is_ok());
    assert_eq!(expect(4).to_be_greater_than_or_equal(5), Err(Error::ExpectationViolation));
    
    // Less than
    assert!(expect(5).to_be_less_than(10).is_ok());
    assert_eq!(expect(10).to_be_less_than(5), Err(Error::ExpectationViolation));
    assert_eq!(expect(5).to_be_less_than(5), Err(Error::ExpectationViolation));
    
    // Less than or equal
    assert!(expect(5).to_be_less_than_or_equal(10).is_ok());
    assert!(expect(5).to_be_less_than_or_equal(5).is_ok());
    assert_eq!(expect(6).to_be_less_than_or_equal(5), Err(Error::ExpectationViolation));
}

#[test]
fn test_range_checks() {
    assert!(expect(5).to_be_in_range(1, 10).is_ok());
    assert!(expect(1).to_be_in_range(1, 10).is_ok());
    assert!(expect(10).to_be_in_range(1, 10).is_ok());
    assert_eq!(expect(0).to_be_in_range(1, 10), Err(Error::ExpectationViolation));
    assert_eq!(expect(11).to_be_in_range(1, 10), Err(Error::ExpectationViolation));
}

#[test]
fn test_boolean_assertions() {
    assert!(expect(true).to_be_true().is_ok());
    assert_eq!(expect(false).to_be_true(), Err(Error::ExpectationViolation));
    
    assert!(expect(false).to_be_false().is_ok());
    assert_eq!(expect(true).to_be_false(), Err(Error::ExpectationViolation));
}

#[test]
fn test_option_assertions() {
    let some_value = Some(42);
    let none_value: Option<i32> = None;
    
    assert!(expect(some_value).to_be_some().is_ok());
    assert_eq!(expect(none_value).to_be_some(), Err(Error::ExpectationViolation));
    
    assert!(expect(none_value).to_be_none().is_ok());
    assert_eq!(expect(some_value).to_be_none(), Err(Error::ExpectationViolation));
    
    assert!(expect(some_value).to_be_some_and_equal(42).is_ok());
    assert_eq!(expect(some_value).to_be_some_and_equal(43), Err(Error::ExpectationViolation));
    assert_eq!(expect(none_value).to_be_some_and_equal(42), Err(Error::ExpectationViolation));
}

#[test]
fn test_result_assertions() {
    let ok_result: Result<i32, &str> = Ok(42);
    let err_result: Result<i32, &str> = Err("error");
    
    assert!(expect(ok_result).to_be_ok().is_ok());
    assert_eq!(expect(err_result).to_be_ok(), Err(Error::ExpectationViolation));
    
    assert!(expect(err_result).to_be_err().is_ok());
    assert_eq!(expect(ok_result).to_be_err(), Err(Error::ExpectationViolation));
    
    // to_be_ok_and_equal is not implemented in the current version
}

#[test]
fn test_collection_length() {
    let vec = vec![1, 2, 3];
    let empty_vec: Vec<i32> = vec![];
    
    assert!(expect(&vec).to_have_length(3).is_ok());
    assert_eq!(expect(&vec).to_have_length(2), Err(Error::CellCountViolation));
    
    assert!(expect(&empty_vec).to_have_length(0).is_ok());
    assert_eq!(expect(&empty_vec).to_have_length(1), Err(Error::CellCountViolation));
}

#[test]
fn test_collection_emptiness() {
    let vec = vec![1, 2, 3];
    let empty_vec: Vec<i32> = vec![];
    
    assert!(expect(&empty_vec).to_be_empty().is_ok());
    assert_eq!(expect(&vec).to_be_empty(), Err(Error::CellCountViolation));
    
    assert!(expect(&vec).not_to_be_empty().is_ok());
    assert_eq!(expect(&empty_vec).not_to_be_empty(), Err(Error::CellCountViolation));
}

#[test]
fn test_custom_predicate() {
    let is_even = |n: &i32| *n % 2 == 0;
    
    assert!(expect(4).to_satisfy(is_even, "should be even").is_ok());
    assert_eq!(expect(3).to_satisfy(is_even, "should be even"), Err(Error::ExpectationViolation));
}

#[test]
fn test_chained_assertions() {
    // Test that multiple assertions can be chained
    let vec = vec![2, 4, 6, 8];
    
    // All elements should be even
    for &num in &vec {
        assert!(expect(num).to_be_greater_than(0).is_ok());
        assert!(expect(num).to_satisfy(|n| n % 2 == 0, "should be even").is_ok());
    }
}

#[test]
fn test_error_types() {
    // Test that different assertion types return appropriate errors
    let vec = vec![1, 2, 3];
    
    // Length assertion should return CellCountViolation
    assert_eq!(expect(&vec).to_have_length(5), Err(Error::CellCountViolation));
    
    // Empty assertion should return CellCountViolation
    assert_eq!(expect(&vec).to_be_empty(), Err(Error::CellCountViolation));
    
    // Other assertions should return ExpectationViolation
    assert_eq!(expect(42).to_equal(43), Err(Error::ExpectationViolation));
    assert_eq!(expect(true).to_be_false(), Err(Error::ExpectationViolation));
}

#[test]
fn test_edge_cases() {
    // Test with zero
    assert!(expect(0).to_equal(0).is_ok());
    assert!(expect(0).to_be_less_than(1).is_ok());
    assert!(expect(0).to_be_greater_than_or_equal(0).is_ok());
    
    // Test with negative numbers
    assert!(expect(-5).to_be_less_than(0).is_ok());
    assert!(expect(-5).to_be_greater_than(-10).is_ok());
    
    // Test with empty string  
    assert!(expect("").to_equal("").is_ok());
    // String length and empty methods are not implemented for &str
}

#[test]
fn test_type_inference() {
    // Test that type inference works correctly
    let _result = expect(42).to_equal(42);
    let _result = expect("hello").to_equal("hello");
    let _result = expect(vec![1, 2, 3]).to_have_length(3);
    let _result = expect(Some(42)).to_be_some();
    let _result = expect(Ok::<i32, &str>(42)).to_be_ok();
}