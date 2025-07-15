/// Transaction recipe implementation based on generated TransactionRecipe
/// 
/// This module provides functionality for working with transaction recipes
/// using the generated TransactionRecipe as the base type.
/// 
/// Method paths are calculated using Blake2b-256 hash (first 8 bytes as u64).

use crate::errors::Error;
use crate::generated::{TransactionRecipe, Bytes, BytesVec};
use ckb_std::{high_level, ckb_constants::Source};
use ckb_hash::blake2b_256;
extern crate alloc;
use alloc::{vec::Vec, string::String};
use molecule::prelude::*;
use core::{
    option::{Option, Option::*},
    result::{Result, Result::*},
};

/// Calculate SSRI method path hash from method name string
/// Returns the first 8 bytes of Blake2b-256 hash as u64
pub fn method_path(name: impl AsRef<[u8]>) -> u64 {
    u64::from_le_bytes(blake2b_256(name)[0..8].try_into().unwrap())
}

/// Extension trait that adds functionality to the generated TransactionRecipe
pub trait TransactionRecipeExt {
    /// Get the method path hash for this recipe
    fn method_path_hash(&self) -> Result<u64, Error>;
    
    /// Get the method path name as string for this recipe (for debugging/logging)
    fn method_path_name(&self) -> Result<String, Error>;
    
    /// Get the arguments as vector of byte vectors
    fn arguments_vec(&self) -> Vec<Vec<u8>>;
    
    /// Check if the method path matches expected hash
    fn matches_method_path_hash(&self, expected_hash: u64) -> Result<bool, Error> {
        Ok(self.method_path_hash()? == expected_hash)
    }
    
    /// Check if the method path matches expected name
    fn matches_method_name(&self, expected_name: &str) -> Result<bool, Error> {
        Ok(self.method_path_hash()? == method_path(expected_name))
    }
    
    /// Get method path as raw bytes
    fn method_path_bytes(&self) -> Vec<u8>;
}

impl TransactionRecipeExt for TransactionRecipe {
    fn method_path_hash(&self) -> Result<u64, Error> {
        let method_name = self.method_path_name()?;
        Ok(method_path(&method_name))
    }
    
    fn method_path_name(&self) -> Result<String, Error> {
        let method_path_bytes = self.method_path().raw_data();
        String::from_utf8(method_path_bytes.to_vec())
            .map_err(|_| Error::DataError)
    }
    
    fn arguments_vec(&self) -> Vec<Vec<u8>> {
        let args = self.arguments();
        let mut result = Vec::new();
        
        for i in 0..args.len() {
            if let Some(arg) = args.get(i) {
                result.push(arg.raw_data().to_vec());
            }
        }
        
        result
    }
    
    fn method_path_bytes(&self) -> Vec<u8> {
        self.method_path().raw_data().to_vec()
    }
}

/// Create a TransactionRecipe from method name and arguments
pub fn create_transaction_recipe(method_name: &str, arguments: &[Vec<u8>]) -> Result<TransactionRecipe, Error> {
    // Convert method path name to bytes
    let method_path_bytes = Bytes::new_builder()
        .set(method_name.as_bytes().iter().copied().map(Into::into).collect())
        .build();
    
    // Convert arguments to BytesVec
    let mut args_builder = BytesVec::new_builder();
    for arg in arguments {
        let arg_bytes = Bytes::new_builder()
            .set(arg.iter().copied().map(Into::into).collect())
            .build();
        args_builder = args_builder.push(arg_bytes);
    }
    let arguments = args_builder.build();
    
    Ok(TransactionRecipe::new_builder()
        .method_path(method_path_bytes)
        .arguments(arguments)
        .build())
}

/// Parse transaction recipe from the last witness item
/// Returns None if no witnesses exist or parsing fails
pub fn parse_transaction_recipe() -> Result<Option<TransactionRecipe>, Error> {
    // Try to find the last witness by attempting to load witnesses
    let mut last_witness_data = None;
    let mut index = 0;
    
    loop {
        match high_level::load_witness(index, Source::Input) {
            Ok(data) => {
                last_witness_data = Some(data);
                index += 1;
            }
            Err(_) => break,
        }
    }
    
    let witness_data = match last_witness_data {
        Some(data) => data,
        None => return Ok(None),
    };
    
    // Try to parse as TransactionRecipe using molecule
    match TransactionRecipe::from_slice(&witness_data) {
        Ok(recipe) => Ok(Some(recipe)),
        Err(_) => Ok(None),
    }
}

/// Parse transaction recipe from a specific witness index
pub fn parse_transaction_recipe_at(index: usize) -> Result<Option<TransactionRecipe>, Error> {
    let witness_data = high_level::load_witness(index, Source::Input)
        .map_err(|_| Error::DataError)?;
    
    match TransactionRecipe::from_slice(&witness_data) {
        Ok(recipe) => Ok(Some(recipe)),
        Err(_) => Ok(None),
    }
}

/// Parse transaction recipe from specific witness data
pub fn parse_transaction_recipe_from_data(data: &[u8]) -> Result<Option<TransactionRecipe>, Error> {
    match TransactionRecipe::from_slice(data) {
        Ok(recipe) => Ok(Some(recipe)),
        Err(_) => Ok(None),
    }
}

/// Serialize a transaction recipe to bytes for witness data
pub fn serialize_transaction_recipe(recipe: &TransactionRecipe) -> Vec<u8> {
    recipe.as_slice().to_vec()
}

/// Check if witness data at index contains a transaction recipe
pub fn has_transaction_recipe_at(index: usize) -> bool {
    match parse_transaction_recipe_at(index) {
        Ok(Some(_)) => true,
        _ => false,
    }
}

/// Find the first witness index that contains a transaction recipe
pub fn find_transaction_recipe_witness() -> Option<usize> {
    let mut index = 0;
    loop {
        match high_level::load_witness(index, Source::Input) {
            Ok(_) => {
                if has_transaction_recipe_at(index) {
                    return Some(index);
                }
                index += 1;
            }
            Err(_) => break,
        }
    }
    None
}

/// Helper functions for working with TransactionRecipe directly

/// Get method path as bytes from recipe
pub fn get_method_path(recipe: &TransactionRecipe) -> Vec<u8> {
    recipe.method_path_bytes()
}

/// Get method path as string from recipe
pub fn get_method_path_string(recipe: &TransactionRecipe) -> Result<String, Error> {
    recipe.method_path_name()
}

/// Get method path hash from recipe
pub fn get_method_path_hash(recipe: &TransactionRecipe) -> Result<u64, Error> {
    recipe.method_path_hash()
}

/// Get arguments as vector of byte vectors from recipe
pub fn get_arguments(recipe: &TransactionRecipe) -> Vec<Vec<u8>> {
    recipe.arguments_vec()
}

/// Check if the method path matches expected value
pub fn matches_method_path(recipe: &TransactionRecipe, expected_path: &[u8]) -> bool {
    recipe.method_path_bytes() == expected_path
}

/// Check if the method path matches expected string
pub fn matches_method_path_string(recipe: &TransactionRecipe, expected_path: &str) -> bool {
    matches_method_path(recipe, expected_path.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_transaction_recipe() {
        let args = vec![b"from_address".to_vec(), b"to_address".to_vec(), b"amount".to_vec()];
        let recipe = create_transaction_recipe("UDT.transfer", &args).unwrap();
        
        // Test method path
        assert_eq!(recipe.method_path_name().unwrap(), "UDT.transfer");
        assert_eq!(recipe.method_path_hash().unwrap(), method_path("UDT.transfer"));
        assert!(recipe.matches_method_name("UDT.transfer").unwrap());
        assert!(!recipe.matches_method_name("UDT.mint").unwrap());
        
        // Test arguments
        let result_args = recipe.arguments_vec();
        assert_eq!(result_args.len(), 3);
        assert_eq!(result_args[0], b"from_address");
        assert_eq!(result_args[1], b"to_address");
        assert_eq!(result_args[2], b"amount");
    }
    
    #[test]
    fn test_helper_functions() {
        let args = vec![b"token_a".to_vec(), b"token_b".to_vec()];
        let recipe = create_transaction_recipe("AMM.swap", &args).unwrap();
        
        // Test helper functions
        assert_eq!(get_method_path_string(&recipe).unwrap(), "AMM.swap");
        assert!(matches_method_path_string(&recipe, "AMM.swap"));
        assert!(!matches_method_path_string(&recipe, "AMM.mint"));
        
        let result_args = get_arguments(&recipe);
        assert_eq!(result_args.len(), 2);
        assert_eq!(result_args[0], b"token_a");
        assert_eq!(result_args[1], b"token_b");
    }
    
    #[test]
    fn test_serialization() {
        let args = vec![b"vault_id".to_vec(), b"amount".to_vec()];
        let recipe = create_transaction_recipe("Vault.withdraw", &args).unwrap();
        
        // Serialize
        let serialized = serialize_transaction_recipe(&recipe);
        
        // Deserialize
        let deserialized = parse_transaction_recipe_from_data(&serialized).unwrap().unwrap();
        
        assert_eq!(deserialized.method_path_name().unwrap(), "Vault.withdraw");
        let result_args = deserialized.arguments_vec();
        assert_eq!(result_args.len(), 2);
        assert_eq!(result_args[0], b"vault_id");
        assert_eq!(result_args[1], b"amount");
    }
    
    #[test]
    fn test_multiple_arguments() {
        let args_data = vec![b"arg1".to_vec(), b"arg2".to_vec(), b"arg3".to_vec()];
        let recipe = create_transaction_recipe("Test.method", &args_data).unwrap();
        
        let result_args = recipe.arguments_vec();
        assert_eq!(result_args.len(), 3);
        assert_eq!(result_args, args_data);
    }
}