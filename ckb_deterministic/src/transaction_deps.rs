/// Transaction dependencies (CellDep and HeaderDep) handling
/// 
/// This module provides functionality for working with CKB transaction dependencies
/// including cell dependencies and header dependencies.

use crate::errors::Error;
use crate::generated::{CellDep, CellDepVec};
use ckb_std::{
    high_level::load_header,
    ckb_constants::Source,
    ckb_types::packed::{self, Script},
};
extern crate alloc;
use alloc::{vec::Vec, string::String, ffi};
use molecule::prelude::*;
use core::{
    option::Option::*,
    result::Result,
};

/// Dep type constants
pub const DEP_TYPE_CODE: u8 = 0;
pub const DEP_TYPE_DEP_GROUP: u8 = 1;

/// Load cell dependencies from a transaction
pub fn load_cell_deps() -> Result<Vec<CellDepInfo>, Error> {
    let mut cell_deps = Vec::new();
    let mut index = 0;
    
    loop {
        match load_cell_dep(index) {
            Ok(dep_info) => {
                cell_deps.push(dep_info);
                index += 1;
            }
            Err(_) => break,
        }
    }
    
    Ok(cell_deps)
}

/// Information about a cell dependency
#[derive(Debug, Clone)]
pub struct CellDepInfo {
    pub out_point: OutPointInfo,
    pub dep_type: DepType,
    pub resolved_deps: Vec<OutPointInfo>, // For dep groups, contains resolved members
}

/// OutPoint information
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OutPointInfo {
    pub tx_hash: [u8; 32],
    pub index: u32,
}

/// Dependency type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepType {
    Code,
    DepGroup,
}

impl From<u8> for DepType {
    fn from(value: u8) -> Self {
        match value {
            DEP_TYPE_CODE => DepType::Code,
            DEP_TYPE_DEP_GROUP => DepType::DepGroup,
            _ => DepType::Code, // Default to code for unknown values
        }
    }
}

/// Load a cell dependency by index
fn load_cell_dep(_index: usize) -> Result<CellDepInfo, Error> {
    // In a real implementation, this would use CKB syscalls to load cell dep
    // For now, we'll return an error indicating this needs to be implemented
    // when we have access to the actual transaction structure
    Err(Error::DataError)
}

/// Resolve a dep group to its member cells
pub fn resolve_dep_group(_out_point: &OutPointInfo) -> Result<Vec<OutPointInfo>, Error> {
    // Load the dep group cell data
    // The data should contain a list of OutPoints
    // For now, return empty vector
    Ok(Vec::new())
}

/// Load header dependencies from a transaction
pub fn load_header_deps() -> Result<Vec<[u8; 32]>, Error> {
    let mut header_deps = Vec::new();
    let mut index = 0;
    
    loop {
        match load_header_dep(index) {
            Ok(header_hash) => {
                header_deps.push(header_hash);
                index += 1;
            }
            Err(_) => break,
        }
    }
    
    Ok(header_deps)
}

/// Load a header dependency by index
fn load_header_dep(_index: usize) -> Result<[u8; 32], Error> {
    // In a real implementation, this would use CKB syscalls to load header dep
    // For now, we'll return an error indicating this needs to be implemented
    Err(Error::DataError)
}

/// Extension trait for CellDep
pub trait CellDepExt {
    fn to_info(&self) -> Result<CellDepInfo, Error>;
    fn is_code(&self) -> bool;
    fn is_dep_group(&self) -> bool;
}

impl CellDepExt for CellDep {
    fn to_info(&self) -> Result<CellDepInfo, Error> {
        let out_point = self.out_point();
        let out_point_info = OutPointInfo {
            tx_hash: {
                let raw_data = out_point.tx_hash().raw_data();
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&raw_data);
                arr
            },
            index: {
                let raw_data = out_point.index().raw_data();
                let mut arr = [0u8; 4];
                arr.copy_from_slice(&raw_data);
                u32::from_le_bytes(arr)
            },
        };
        
        let dep_type = DepType::from(self.dep_type().as_slice()[0]);
        
        let resolved_deps = if dep_type == DepType::DepGroup {
            resolve_dep_group(&out_point_info)?
        } else {
            vec![out_point_info.clone()]
        };
        
        Ok(CellDepInfo {
            out_point: out_point_info,
            dep_type,
            resolved_deps,
        })
    }
    
    fn is_code(&self) -> bool {
        self.dep_type().as_slice()[0] == DEP_TYPE_CODE
    }
    
    fn is_dep_group(&self) -> bool {
        self.dep_type().as_slice()[0] == DEP_TYPE_DEP_GROUP
    }
}

/// Extension trait for CellDepVec
pub trait CellDepVecExt {
    fn to_info_vec(&self) -> Result<Vec<CellDepInfo>, Error>;
    fn has_dep(&self, tx_hash: &[u8; 32], index: u32) -> bool;
}

impl CellDepVecExt for CellDepVec {
    fn to_info_vec(&self) -> Result<Vec<CellDepInfo>, Error> {
        let mut infos = Vec::new();
        for i in 0..self.len() {
            if let Some(dep) = self.get(i) {
                infos.push(dep.to_info()?);
            }
        }
        Ok(infos)
    }
    
    fn has_dep(&self, tx_hash: &[u8; 32], index: u32) -> bool {
        for i in 0..self.len() {
            if let Some(dep) = self.get(i) {
                let out_point = dep.out_point();
                let raw_data = out_point.tx_hash().raw_data();
                let mut hash_arr = [0u8; 32];
                hash_arr.copy_from_slice(&raw_data);
                let dep_tx_hash = hash_arr;
                let index_raw = out_point.index().raw_data();
                let mut index_arr = [0u8; 4];
                index_arr.copy_from_slice(&index_raw);
                let dep_index = u32::from_le_bytes(index_arr);
                
                if &dep_tx_hash == tx_hash && dep_index == index {
                    return true;
                }
            }
        }
        false
    }
}

/// Validate that all required cell deps for known scripts are present
pub fn validate_known_script_deps(
    cell_deps: &[CellDepInfo],
    required_scripts: &[(Script, Vec<(String, u32, DepType)>)],
) -> Result<(), Error> {
    for (_script, required_deps) in required_scripts {
        for (tx_hash_str, index, dep_type) in required_deps {
            // Convert hex string to bytes
            let tx_hash = hex_to_bytes(tx_hash_str)?;
            
            let found = cell_deps.iter().any(|dep| {
                dep.out_point.tx_hash == tx_hash && 
                dep.out_point.index == *index &&
                dep.dep_type == *dep_type
            });
            
            if !found {
                return Err(Error::DataError);
            }
        }
    }
    Ok(())
}

/// Helper to convert hex string to bytes using ckb-std
fn hex_to_bytes(hex: &str) -> Result<[u8; 32], Error> {
    use ckb_std::high_level::decode_hex;
    
    let hex = hex.trim_start_matches("0x");
    
    // Convert to CString for ckb-std decode_hex
    let hex_cstr = ffi::CString::new(hex)
        .map_err(|_| Error::DataError)?;
    
    let decoded = decode_hex(&hex_cstr)
        .map_err(|_| Error::DataError)?;
    
    if decoded.len() != 32 {
        return Err(Error::DataError);
    }
    
    let mut result = [0u8; 32];
    result.copy_from_slice(&decoded);
    Ok(result)
}

/// Check if a header is available via header deps
pub fn is_header_available(header_hash: &[u8; 32], header_deps: &[[u8; 32]]) -> bool {
    header_deps.contains(header_hash)
}

/// Load header by hash if it's in header_deps
pub fn load_header_by_hash(header_hash: &[u8; 32], header_deps: &[[u8; 32]]) -> Result<packed::Header, Error> {
    if !is_header_available(header_hash, header_deps) {
        return Err(Error::DataError);
    }
    
    // Find the index of this header in header_deps
    let index = header_deps.iter().position(|h| h == header_hash)
        .ok_or(Error::DataError)?;
    
    // Load the header using ckb_std
    load_header(index, Source::HeaderDep)
        .map_err(|_| Error::DataError)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated::{Byte32, Uint32, OutPoint as MoleculeOutPoint};
    
    #[test]
    fn test_dep_type_conversion() {
        assert_eq!(DepType::from(0), DepType::Code);
        assert_eq!(DepType::from(1), DepType::DepGroup);
        assert_eq!(DepType::from(99), DepType::Code); // Unknown defaults to Code
    }
    
    #[test]
    fn test_out_point_info() {
        let tx_hash = [1u8; 32];
        let index = 42u32;
        
        let info = OutPointInfo { tx_hash, index };
        assert_eq!(info.tx_hash, tx_hash);
        assert_eq!(info.index, index);
    }
    
    #[test]
    fn test_cell_dep_ext() {
        // Create a CellDep with code type
        let tx_hash = Byte32::new_builder()
            .set({
                let bytes: [molecule::prelude::Byte; 32] = [1u8; 32].map(|b| b.into());
                bytes
            })
            .build();
        let index = Uint32::new_builder()
            .set({
                let bytes: [molecule::prelude::Byte; 4] = [0u8, 0, 0, 1].map(|b| b.into());
                bytes
            })
            .build();
        let out_point = MoleculeOutPoint::new_builder()
            .tx_hash(tx_hash)
            .index(index)
            .build();
        
        let cell_dep = CellDep::new_builder()
            .out_point(out_point)
            .dep_type(molecule::prelude::Byte::new(DEP_TYPE_CODE))
            .build();
        
        assert!(cell_dep.is_code());
        assert!(!cell_dep.is_dep_group());
    }
    
    #[test]
    fn test_header_availability() {
        let header_hash = [1u8; 32];
        let other_hash = [2u8; 32];
        let header_deps = vec![header_hash, [3u8; 32]];
        
        assert!(is_header_available(&header_hash, &header_deps));
        assert!(!is_header_available(&other_hash, &header_deps));
    }
    
    #[test]
    fn test_hex_to_bytes() {
        let hex = "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let result = hex_to_bytes(hex);
        assert!(result.is_ok());
        
        let hex_no_prefix = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let result2 = hex_to_bytes(hex_no_prefix);
        assert!(result2.is_ok());
        
        assert_eq!(result.unwrap(), result2.unwrap());
    }
}