use crate::cell_classifier::CellClassifier;
/// Transaction recipe implementation based on generated TransactionRecipe
/// 
/// This module provides functionality for working with transaction recipes
/// using the generated TransactionRecipe as the base type.
/// 
/// Method paths are calculated using Blake2b-256 hash (first 8 bytes as u64).
/// 
/// # Examples
/// 
/// ## Using helper functions for flexible argument construction
/// 
/// ```rust,no_run
/// # use ckb_deterministic::errors::Error;
/// # fn example() -> Result<(), Error> {
/// use ckb_deterministic::transaction_recipe::{
///     create_inline_argument, create_output_data_reference, 
///     create_input_data_reference, create_cell_dep_data_reference,
///     create_recipe_with_args
/// };
/// 
/// // Simple recipe with one output data reference
/// let recipe = create_recipe_with_args(
///     "Protocol.update",
///     vec![create_output_data_reference(0)]
/// )?;
/// 
/// // Complex recipe with multiple argument types
/// let recipe = create_recipe_with_args(
///     "AMM.swap",
///     vec![
///         create_inline_argument(b"token_a"),
///         create_inline_argument(b"token_b"),
///         create_output_data_reference(1),
///         create_cell_dep_data_reference(0),
///     ]
/// )?;
/// 
/// // Recipe with dependencies would use create_transaction_recipe_with_deps
/// # Ok(())
/// # }
/// ```

use crate::errors::Error;
use crate::generated::{
    TransactionRecipe, Bytes, CellDep, CellDepVec, CellDepVecOpt,
    Byte32, Byte32Vec, Byte32VecOpt, OutPoint, Uint32, RecipeArgument, RecipeArgumentVec
};
use crate::transaction_context::TransactionContext;
use ckb_std::{high_level, ckb_constants::Source};
use ckb_hash::blake2b_256;
extern crate alloc;
use alloc::{vec::Vec, string::String};
use molecule::prelude::*;
use molecule::prelude::Byte;
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
    
    /// Get a specific argument by index
    fn get_argument(&self, index: usize) -> Option<RecipeArgument>;
    
    /// Get the type of a specific argument
    fn get_argument_type(&self, index: usize) -> Option<u8>;
    
    /// Extract inline data from an argument (returns None if not inline_data type)
    fn get_inline_data(&self, index: usize) -> Option<Vec<u8>>;
    
    /// Extract reference index from an argument (returns None if inline_data type)
    fn get_reference_index(&self, index: usize) -> Option<u32>;
    
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

pub trait Validate {
    fn validate(&self, context: &TransactionContext<impl CellClassifier>) -> Result<(), Error>;
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
                result.push(arg.data().raw_data().to_vec());
            }
        }
        
        result
    }
    
    fn method_path_bytes(&self) -> Vec<u8> {
        self.method_path().raw_data().to_vec()
    }
    
    fn get_argument(&self, index: usize) -> Option<RecipeArgument> {
        self.arguments().get(index)
    }
    
    fn get_argument_type(&self, index: usize) -> Option<u8> {
        self.get_argument(index).map(|arg| arg.arg_type().as_slice()[0])
    }
    
    fn get_inline_data(&self, index: usize) -> Option<Vec<u8>> {
        self.get_argument(index).and_then(|arg| {
            if arg.arg_type().as_slice()[0] == 0 {
                Some(arg.data().raw_data().to_vec())
            } else {
                None
            }
        })
    }
    
    fn get_reference_index(&self, index: usize) -> Option<u32> {
        self.get_argument(index).and_then(|arg| {
            let arg_type = arg.arg_type().as_slice()[0];
            // Check if it's a reference type (1-4 matching Source enum values)
            if arg_type >= 1 && arg_type <= 4 {
                let data = arg.data().raw_data();
                if data.len() >= 4 {
                    let mut bytes = [0u8; 4];
                    bytes.copy_from_slice(&data[0..4]);
                    Some(u32::from_le_bytes(bytes))
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
}

/// Create a TransactionRecipe from method name and arguments
pub fn create_transaction_recipe(method_name: &str, arguments: &[Vec<u8>]) -> Result<TransactionRecipe, Error> {
    // Convert method path name to bytes
    let method_path_bytes = Bytes::new_builder()
        .set(method_name.as_bytes().iter().copied().map(Into::into).collect())
        .build();
    
    // Convert arguments to RecipeArgumentVec (default to inline_data type)
    let mut args_builder = RecipeArgumentVec::new_builder();
    for arg in arguments {
        let arg_bytes = Bytes::new_builder()
            .set(arg.iter().copied().map(Into::into).collect())
            .build();
        let recipe_arg = RecipeArgument::new_builder()
            .arg_type(Byte::new(0u8)) // 0 = inline_data
            .data(arg_bytes)
            .build();
        args_builder = args_builder.push(recipe_arg);
    }
    let arguments = args_builder.build();
    
    Ok(TransactionRecipe::new_builder()
        .method_path(method_path_bytes)
        .arguments(arguments)
        .build())
}

/// Parse transaction recipe from any witness position
/// Searches through all witnesses and returns the first valid TransactionRecipe found
pub fn parse_transaction_recipe() -> Result<Option<TransactionRecipe>, Error> {
    let mut index = 0;
    
    // Search through all witnesses
    loop {
        match high_level::load_witness(index, Source::Input) {
            Ok(data) => {
                // Try to parse this witness as a TransactionRecipe
                match TransactionRecipe::from_slice(&data) {
                    Ok(recipe) => {
                        // Found a valid recipe, return it
                        return Ok(Some(recipe));
                    }
                    Err(_) => {
                        // Not a valid recipe, continue searching
                        index += 1;
                        continue;
                    }
                }
            }
            Err(_) => {
                // No more witnesses to check
                break;
            }
        }
    }
    
    // No valid recipe found in any witness
    Ok(None)
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

/// Create a recipe argument with inline data
pub fn create_inline_argument(data: &[u8]) -> RecipeArgument {
    let data_bytes = Bytes::new_builder()
        .set(data.iter().copied().map(Into::into).collect())
        .build();
    RecipeArgument::new_builder()
        .arg_type(Byte::new(0u8)) // 0 = inline_data
        .data(data_bytes)
        .build()
}

/// Create a recipe argument with output data reference
pub fn create_output_data_reference(index: u32) -> RecipeArgument {
    let index_bytes = Bytes::new_builder()
        .set(index.to_le_bytes().iter().copied().map(Into::into).collect())
        .build();
    RecipeArgument::new_builder()
        .arg_type(Byte::new(2u8)) // 2 = output_data_reference (matches Source::Output)
        .data(index_bytes)
        .build()
}

/// Create a recipe argument with input data reference
pub fn create_input_data_reference(index: u32) -> RecipeArgument {
    let index_bytes = Bytes::new_builder()
        .set(index.to_le_bytes().iter().copied().map(Into::into).collect())
        .build();
    RecipeArgument::new_builder()
        .arg_type(Byte::new(1u8)) // 1 = input_data_reference (matches Source::Input)
        .data(index_bytes)
        .build()
}

/// Create a recipe argument with cell dep data reference
pub fn create_cell_dep_data_reference(index: u32) -> RecipeArgument {
    let index_bytes = Bytes::new_builder()
        .set(index.to_le_bytes().iter().copied().map(Into::into).collect())
        .build();
    RecipeArgument::new_builder()
        .arg_type(Byte::new(3u8)) // 3 = cell_dep_data_reference (matches Source::CellDep)
        .data(index_bytes)
        .build()
}

/// Create a recipe argument with header reference
pub fn create_header_reference(index: u32) -> RecipeArgument {
    let index_bytes = Bytes::new_builder()
        .set(index.to_le_bytes().iter().copied().map(Into::into).collect())
        .build();
    RecipeArgument::new_builder()
        .arg_type(Byte::new(4u8)) // 4 = header_reference (matches Source::HeaderDep)
        .data(index_bytes)
        .build()
}

/// Create a recipe argument with a reference to data at a specific source and index
/// 
/// # Arguments
/// * `source` - The source type from ckb_std (Input, Output, CellDep, HeaderDep)
/// * `index` - The index within the source
/// 
/// # Examples
/// ```rust,no_run
/// use ckb_std::ckb_constants::Source;
/// use ckb_deterministic::transaction_recipe::create_recipe_with_reference;
/// 
/// // Reference to output data at index 0
/// let arg = create_recipe_with_reference(Source::Output, 0);
/// 
/// // Reference to cell dep data at index 2
/// let arg = create_recipe_with_reference(Source::CellDep, 2);
/// ```
pub fn create_recipe_with_reference(source: Source, index: u32) -> RecipeArgument {
    // Use Source enum value directly as arg_type (they now match)
    let arg_type = match source {
        Source::Input => 1u8,      // matches Source::Input = 1
        Source::Output => 2u8,     // matches Source::Output = 2
        Source::CellDep => 3u8,    // matches Source::CellDep = 3
        Source::HeaderDep => 4u8,  // matches Source::HeaderDep = 4
        Source::GroupInput | Source::GroupOutput => {
            panic!("Group sources are not supported for recipe references")
        }
    };
    
    let index_bytes = Bytes::new_builder()
        .set(index.to_le_bytes().iter().copied().map(Into::into).collect())
        .build();
        
    RecipeArgument::new_builder()
        .arg_type(Byte::new(arg_type))
        .data(index_bytes)
        .build()
}

/// Create a TransactionRecipe with flexible arguments
/// 
/// # Examples
/// 
/// ```rust,no_run
/// # use ckb_deterministic::errors::Error;
/// # fn example() -> Result<(), Error> {
/// use ckb_deterministic::transaction_recipe::{
///     create_inline_argument, create_output_data_reference,
///     create_cell_dep_data_reference, create_recipe_with_args
/// };
/// 
/// // Simple recipe with one output data reference
/// let recipe = create_recipe_with_args(
///     "Protocol.update",
///     vec![create_output_data_reference(0)]
/// )?;
/// 
/// // Complex recipe with multiple argument types  
/// let recipe = create_recipe_with_args(
///     "AMM.swap",
///     vec![
///         create_inline_argument(b"token_a"),
///         create_inline_argument(b"token_b"),
///         create_output_data_reference(1),
///         create_cell_dep_data_reference(0),
///     ]
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn create_recipe_flexible(method_name: &str, args: Vec<RecipeArgument>) -> Result<TransactionRecipe, Error> {
    create_recipe_with_args(method_name, args)
}

/// Create a TransactionRecipe with multiple arguments of different types
pub fn create_recipe_with_args(method_name: &str, args: Vec<RecipeArgument>) -> Result<TransactionRecipe, Error> {
    let method_path_bytes = Bytes::new_builder()
        .set(method_name.as_bytes().iter().copied().map(Into::into).collect())
        .build();
    
    let arguments = RecipeArgumentVec::new_builder()
        .set(args)
        .build();
    
    Ok(TransactionRecipe::new_builder()
        .method_path(method_path_bytes)
        .arguments(arguments)
        .build())
}

/// Check if witness data at index contains a transaction recipe
pub fn has_transaction_recipe_at(index: usize) -> bool {
    match parse_transaction_recipe_at(index) {
        Ok(Some(_)) => true,
        _ => false,
    }
}

/// Find the first witness index that contains a valid transaction recipe
/// Searches through all witnesses in order and returns the index of the first valid recipe
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

/// Resolve a RecipeArgument to get the actual data
/// For inline data, returns the data directly
/// For references, loads the data from the specified source
pub fn resolve_recipe_argument(arg: &RecipeArgument) -> Result<Vec<u8>, Error> {
    let arg_type_byte = arg.arg_type();
    let arg_type_value = u8::from(arg_type_byte);
    
    match arg_type_value {
        0 => {
            // inline_data - return the data directly
            Ok(arg.data().raw_data().to_vec())
        }
        1 => {
            // input_data_reference
            let index = u32::from_le_bytes(
                arg.data().raw_data()[0..4]
                    .try_into()
                    .map_err(|_| Error::Encoding)?
            );
            high_level::load_cell_data(index as usize, Source::Input)
                .map_err(|_| Error::IndexOutOfBound)
        }
        2 => {
            // output_data_reference
            let index = u32::from_le_bytes(
                arg.data().raw_data()[0..4]
                    .try_into()
                    .map_err(|_| Error::Encoding)?
            );
            high_level::load_cell_data(index as usize, Source::Output)
                .map_err(|_| Error::IndexOutOfBound)
        }
        3 => {
            // cell_dep_data_reference
            let index = u32::from_le_bytes(
                arg.data().raw_data()[0..4]
                    .try_into()
                    .map_err(|_| Error::Encoding)?
            );
            high_level::load_cell_data(index as usize, Source::CellDep)
                .map_err(|_| Error::IndexOutOfBound)
        }
        4 => {
            // header_reference - headers don't have data, return the header hash
            let index = u32::from_le_bytes(
                arg.data().raw_data()[0..4]
                    .try_into()
                    .map_err(|_| Error::Encoding)?
            );
            // Load header and return its hash
            let header = high_level::load_header(index as usize, Source::HeaderDep)
                .map_err(|_| Error::IndexOutOfBound)?;
            // Return the raw header bytes
            Ok(header.as_slice().to_vec())
        }
        _ => Err(Error::Encoding),
    }
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

/// Parameters for creating a transaction recipe with dependencies
pub struct RecipeParams<'a> {
    pub method_name: &'a str,
    pub arguments: &'a [Vec<u8>],
    pub cell_deps: Option<&'a [CellDepParams]>,
    pub header_deps: Option<&'a [[u8; 32]]>,
}

/// Parameters for creating a cell dependency
pub struct CellDepParams {
    pub tx_hash: [u8; 32],
    pub index: u32,
    pub dep_type: u8, // 0 = code, 1 = dep_group
}

/// Create a TransactionRecipe with optional dependencies
pub fn create_transaction_recipe_with_deps(params: RecipeParams) -> Result<TransactionRecipe, Error> {
    // Convert method path name to bytes
    let method_path_bytes = Bytes::new_builder()
        .set(params.method_name.as_bytes().iter().copied().map(Into::into).collect())
        .build();
    
    // Convert arguments to RecipeArgumentVec (default to inline_data type)
    let mut args_builder = RecipeArgumentVec::new_builder();
    for arg in params.arguments {
        let arg_bytes = Bytes::new_builder()
            .set(arg.iter().copied().map(Into::into).collect())
            .build();
        let recipe_arg = RecipeArgument::new_builder()
            .arg_type(Byte::new(0u8)) // 0 = inline_data
            .data(arg_bytes)
            .build();
        args_builder = args_builder.push(recipe_arg);
    }
    let arguments = args_builder.build();
    
    // Build cell deps if provided
    let cell_deps = if let Some(deps) = params.cell_deps {
        let mut deps_builder = CellDepVec::new_builder();
        for dep_param in deps {
            let tx_hash = Byte32::new_builder()
                .set({
                    let mut bytes = [Byte::default(); 32];
                    for (i, &b) in dep_param.tx_hash.iter().enumerate() {
                        bytes[i] = Byte::new(b);
                    }
                    bytes
                })
                .build();
            let index = Uint32::new_builder()
                .set([
                    Byte::new((dep_param.index & 0xff) as u8),
                    Byte::new(((dep_param.index >> 8) & 0xff) as u8),
                    Byte::new(((dep_param.index >> 16) & 0xff) as u8),
                    Byte::new(((dep_param.index >> 24) & 0xff) as u8),
                ])
                .build();
            let out_point = OutPoint::new_builder()
                .tx_hash(tx_hash)
                .index(index)
                .build();
            let cell_dep = CellDep::new_builder()
                .out_point(out_point)
                .dep_type(molecule::prelude::Byte::new(dep_param.dep_type))
                .build();
            deps_builder = deps_builder.push(cell_dep);
        }
        CellDepVecOpt::new_builder()
            .set(Some(deps_builder.build()))
            .build()
    } else {
        CellDepVecOpt::new_builder().build()
    };
    
    // Build header deps if provided
    let header_deps = if let Some(deps) = params.header_deps {
        let mut deps_builder = Byte32Vec::new_builder();
        for hash in deps {
            let byte32 = Byte32::new_builder()
                .set({
                    let mut bytes = [Byte::default(); 32];
                    for (i, &b) in hash.iter().enumerate() {
                        bytes[i] = Byte::new(b);
                    }
                    bytes
                })
                .build();
            deps_builder = deps_builder.push(byte32);
        }
        Byte32VecOpt::new_builder()
            .set(Some(deps_builder.build()))
            .build()
    } else {
        Byte32VecOpt::new_builder().build()
    };
    
    Ok(TransactionRecipe::new_builder()
        .method_path(method_path_bytes)
        .arguments(arguments)
        .cell_deps(cell_deps)
        .header_deps(header_deps)
        .build())
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
    
    #[test]
    fn test_create_recipe_with_deps() {
        let args = vec![b"arg1".to_vec()];
        let cell_deps = vec![
            CellDepParams {
                tx_hash: [1u8; 32],
                index: 0,
                dep_type: 0, // code
            },
            CellDepParams {
                tx_hash: [2u8; 32],
                index: 1,
                dep_type: 1, // dep_group
            },
        ];
        let header_deps = vec![[3u8; 32], [4u8; 32]];
        
        let params = RecipeParams {
            method_name: "UDT.transfer",
            arguments: &args,
            cell_deps: Some(&cell_deps),
            header_deps: Some(&header_deps),
        };
        
        let recipe = create_transaction_recipe_with_deps(params).unwrap();
        
        // Test basic fields
        assert_eq!(recipe.method_path_name().unwrap(), "UDT.transfer");
        assert_eq!(recipe.arguments_vec().len(), 1);
        
        // Test cell deps
        let deps = recipe.cell_deps().to_opt().unwrap();
        assert_eq!(deps.len(), 2);
        
        // Test header deps
        let hdeps = recipe.header_deps().to_opt().unwrap();
        assert_eq!(hdeps.len(), 2);
    }
}