/// Universal Cell Classification System for CKB
///
/// This module provides a project-agnostic framework for classifying and collecting
/// cells in CKB transactions. It's designed to work with any project, not just CKBoost.
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{packed::Script, prelude::Entity},
    high_level::{
        load_cell_data, load_cell_lock, load_cell_lock_hash, load_cell_type, load_cell_type_hash,
    },
};
extern crate alloc;
use alloc::{
    boxed::Box,
    collections::BTreeMap,
    format,
    string::String,
    vec::Vec,
};
use crate::{debug_info, debug_trace};

use crate::known_scripts::KnownScript;

/// Represents a cell with all its metadata
#[derive(Debug, Clone)]
pub struct CellInfo {
    pub source: Source,
    pub index: usize,
    pub data: Vec<u8>,
    pub lock: Script,
    pub lock_hash: [u8; 32],
    pub type_script: Option<Script>,
    pub type_hash: Option<[u8; 32]>,
}

/// Cell classification result - completely generic
#[derive(Debug, Clone, PartialEq)]
pub enum CellClass {
    /// Simple CKB cell (no type script)
    SimpleCKB,
    /// Known cell type with string identifier (e.g., "xudt", "spore")
    Known(String),
    /// Custom cell type with byte identifier for efficiency (e.g., project-specific cells)
    Custom(String),
    /// Unidentified cell that doesn't match any classification rules
    Unidentified,
}

impl CellClass {
    /// Create a known cell class from a string
    pub fn known(name: &str) -> Self {
        CellClass::Known(name.into())
    }

    /// Create a custom cell class from bytes
    pub fn custom(id: impl Into<String>) -> Self {
        CellClass::Custom(id.into())
    }

    /// Check if this is a simple CKB cell
    pub fn is_simple_ckb(&self) -> bool {
        matches!(self, CellClass::SimpleCKB)
    }

    /// Check if this is a known cell type
    pub fn is_known(&self, name: &str) -> bool {
        matches!(self, CellClass::Known(n) if n == name)
    }

    /// Check if this is a custom cell type
    pub fn is_custom(&self, id: &str) -> bool {
        matches!(self, CellClass::Custom(i) if i == id)
    }

    /// Check if this is an unidentified cell
    pub fn is_unidentified(&self) -> bool {
        matches!(self, CellClass::Unidentified)
    }
}

/// Classification rule for cells
pub enum ClassificationRule {
    /// Match by exact type hash
    TypeHash {
        hash: [u8; 32],
        class: CellClass,
    },
    /// Match by type code hash
    TypeCodeHash {
        code_hash: [u8; 32],
        class: CellClass,
    },
    /// Match by exact lock hash
    LockHash {
        hash: [u8; 32],
        class: CellClass,
    },
    /// Match by lock code hash
    LockCodeHash {
        code_hash: [u8; 32],
        class: CellClass,
    },
    /// Match by known script
    KnownScript {
        script: KnownScript,
        class: CellClass,
    },
    /// Custom predicate function
    Custom {
        predicate: fn(&CellInfo) -> bool,
        class: CellClass,
        name: String,
    },
}

impl ClassificationRule {
    /// Check if this rule matches the given cell
    pub fn matches(&self, cell: &CellInfo) -> Result<Option<CellClass>, crate::errors::Error> {
        match self {
            ClassificationRule::TypeHash { hash, class } => {
                        if cell.type_hash == Some(*hash) {
                            Ok(Some(class.clone()))
                        } else {
                            Ok(None)
                        }
                    }
            ClassificationRule::TypeCodeHash { code_hash, class } => {
                        match &cell.type_script {
                            Some(type_script) => {
                                if type_script.code_hash().as_slice() == code_hash {
                                    Ok(Some(class.clone()))
                                } else {
                                    Ok(None)
                                }
                            }
                            None => Ok(None)
                        }
                    }
            ClassificationRule::LockHash { hash, class } => {
                        if cell.lock_hash == *hash {
                            Ok(Some(class.clone()))
                        } else {
                            Ok(None)
                        }
                    }
            ClassificationRule::LockCodeHash { code_hash, class } => {
                        if cell.lock.code_hash().as_slice() == code_hash {
                            Ok(Some(class.clone()))
                        } else {
                            Ok(None)
                        }
                    }
            ClassificationRule::Custom {
                        predicate, class, ..
                    } => {
                        if predicate(cell) {
                            Ok(Some(class.clone()))
                        } else {
                            Ok(None)
                        }
                    }
            ClassificationRule::KnownScript { script, class } => {
                // Check both type script and lock script based on the script type
                if script.is_type_script() {
                    match &cell.type_script {
                        Some(type_script) => {
                            let expected_code_hash = script.code_hash()?;
                            if type_script.code_hash().as_slice() == expected_code_hash {
                                Ok(Some(class.clone()))
                            } else {
                                Ok(None)
                            }
                        }
                        None => Ok(None)
                    }
                } else if script.is_lock_script() {
                    let expected_code_hash = script.code_hash()?;
                    if cell.lock.code_hash().as_slice() == expected_code_hash {
                        Ok(Some(class.clone()))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
        }
    }

    /// Get human-readable name for this rule
    pub fn name(&self) -> String {
        match self {
            ClassificationRule::TypeHash { hash, .. } => {
                format!("TypeHash({:02x?})", &hash[0..4])
            }
            ClassificationRule::TypeCodeHash { code_hash, .. } => {
                format!("TypeCodeHash({:02x?})", &code_hash[0..4])
            }
            ClassificationRule::LockHash { hash, .. } => {
                format!("LockHash({:02x?})", &hash[0..4])
            }
            ClassificationRule::LockCodeHash { code_hash, .. } => {
                format!("LockCodeHash({:02x?})", &code_hash[0..4])
            }
            ClassificationRule::Custom { name, .. } => {
                format!("Custom({})", name)
            }
            ClassificationRule::KnownScript { script, .. } => {
                format!("KnownScript({})", script.identifier())
            }
        }
    }
}

/// Trait for cell classification strategies
pub trait CellClassifier {
    /// Classify a cell based on its properties
    fn classify(&self, cell: &CellInfo) -> Result<CellClass, crate::errors::Error>;

    /// Get the human-readable name of this classifier
    fn name(&self) -> &str;

    /// Get priority for this classifier (higher = earlier evaluation)
    fn priority(&self) -> u8 {
        100
    }
    
    /// Check if we should load data for a cell with this type code hash
    /// Returns false for unrecognized types that might be depGroups
    fn should_load_data_for_type(&self, type_code_hash: &[u8; 32]) -> bool {
        // Default implementation: don't load data for unknown types
        // Classifiers can override this to specify which types they recognize
        false
    }
}

/// Rule-based cell classifier using a list of classification rules
pub struct RuleBasedClassifier {
    rules: Vec<ClassificationRule>,
    name: String,
}

impl RuleBasedClassifier {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            rules: Vec::new(),
            name: name.into(),
        }
    }

    /// Add a classification rule
    pub fn add_rule(mut self, rule: ClassificationRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Add a type hash rule
    pub fn add_type_hash(self, hash: [u8; 32], class: CellClass) -> Self {
        self.add_rule(ClassificationRule::TypeHash { hash, class })
    }

    /// Add a type code hash rule
    pub fn add_type_code_hash(self, code_hash: [u8; 32], class: CellClass) -> Self {
        self.add_rule(ClassificationRule::TypeCodeHash { code_hash, class })
    }

    /// Add a lock hash rule
    pub fn add_lock_hash(self, hash: [u8; 32], class: CellClass) -> Self {
        self.add_rule(ClassificationRule::LockHash { hash, class })
    }

    /// Add a lock code hash rule
    pub fn add_lock_code_hash(self, code_hash: [u8; 32], class: CellClass) -> Self {
        self.add_rule(ClassificationRule::LockCodeHash { code_hash, class })
    }

    pub fn add_known_script(self, script: KnownScript, class: CellClass) -> Self {
        self.add_rule(ClassificationRule::KnownScript {
            script,
            class,
        })
    }

    /// Add a custom predicate rule
    pub fn add_custom(
        self,
        name: impl Into<String>,
        predicate: fn(&CellInfo) -> bool,
        class: CellClass,
    ) -> Self {
        self.add_rule(ClassificationRule::Custom {
            predicate,
            class,
            name: name.into(),
        })
    }

}

impl CellClassifier for RuleBasedClassifier {
    fn classify(&self, cell: &CellInfo) -> Result<CellClass, crate::errors::Error> {
        // Check if it's a simple CKB cell (no type script)
        if cell.type_script.is_none() {
            return Ok(CellClass::SimpleCKB);
        }
        
        // Evaluate rules in order until we find a match
        for rule in &self.rules {
            match rule.matches(cell)? {
                Some(class) => return Ok(class),
                None => {}
            }
        }
        Ok(CellClass::Unidentified)
    }

    fn name(&self) -> &str {
        &self.name
    }
    
    fn should_load_data_for_type(&self, type_code_hash: &[u8; 32]) -> bool {
        // Check if any of our rules recognize this type code hash
        for rule in &self.rules {
            match rule {
                ClassificationRule::TypeCodeHash { code_hash, .. } => {
                    if code_hash == type_code_hash {
                        // We recognize this type, so we should load its data
                        return true;
                    }
                }
                _ => {
                    // Other rule types don't apply for this check
                }
            }
        }
        // We don't recognize this type, so don't load its data
        // This is important for depGroups which don't have normal cell data
        false
    }
}

/// Cell collection results organized by classification
#[derive(Debug, Default)]
pub struct ClassifiedCells {
    pub simple_ckb_cells: Vec<CellInfo>,
    pub known_cells: BTreeMap<String, Vec<CellInfo>>,
    pub custom_cells: BTreeMap<String, Vec<CellInfo>>,
    pub unidentified_cells: Vec<CellInfo>,
}

impl ClassifiedCells {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a cell to the appropriate classification
    pub fn add_cell(&mut self, cell: CellInfo, classification: CellClass) {
        match classification {
            CellClass::SimpleCKB => {
                self.simple_ckb_cells.push(cell);
            }
            CellClass::Known(name) => {
                self.known_cells
                    .entry(name)
                    .or_insert_with(Vec::new)
                    .push(cell);
            }
            CellClass::Custom(id) => {
                self.custom_cells
                    .entry(id)
                    .or_insert_with(Vec::new)
                    .push(cell);
            }
            CellClass::Unidentified => {
                self.unidentified_cells.push(cell);
            }
        }
    }

    /// Get simple CKB cells
    pub fn get_simple_ckb(&self) -> &Vec<CellInfo> {
        &self.simple_ckb_cells
    }

    /// Get cells of a known type
    pub fn get_known(&self, name: &str) -> Option<&Vec<CellInfo>> {
        self.known_cells.get(name)
    }

    /// Get cells of a custom type
    pub fn get_custom(&self, id: &str) -> Option<&Vec<CellInfo>> {
        self.custom_cells.get(id)
    }

    /// Check if there are any unidentified cells
    pub fn has_unidentified_cells(&self) -> bool {
        !self.unidentified_cells.is_empty()
    }

    /// Check if the collection is empty (no cells of any type)
    pub fn is_empty(&self) -> bool {
        self.simple_ckb_cells.is_empty()
            && self.known_cells.is_empty()
            && self.custom_cells.is_empty()
            && self.unidentified_cells.is_empty()
    }

    /// Get total count of all cells
    pub fn total_cell_count(&self) -> usize {
        self.simple_ckb_cells.len()
            + self.known_cells.values().map(|v| v.len()).sum::<usize>()
            + self.custom_cells.values().map(|v| v.len()).sum::<usize>()
            + self.unidentified_cells.len()
    }

    /// Get count of identified cells only
    pub fn identified_cell_count(&self) -> usize {
        self.total_cell_count() - self.unidentified_cells.len()
    }

    /// Get summary of cell counts by type
    pub fn summary(&self) -> ClassificationSummary {
        ClassificationSummary {
            simple_ckb_count: self.simple_ckb_cells.len(),
            known_types: self.known_cells.keys().cloned().collect(),
            known_counts: self
                .known_cells
                .iter()
                .map(|(k, v)| (k.clone(), v.len()))
                .collect(),
            custom_types: self.custom_cells.keys().cloned().collect(),
            custom_counts: self
                .custom_cells
                .iter()
                .map(|(k, v)| (k.clone(), v.len()))
                .collect(),
            unidentified_count: self.unidentified_cells.len(),
            total_count: self.total_cell_count(),
        }
    }
}

/// Summary of classification results
#[derive(Debug)]
pub struct ClassificationSummary {
    pub simple_ckb_count: usize,
    pub known_types: Vec<String>,
    pub known_counts: Vec<(String, usize)>,
    pub custom_types: Vec<String>,
    pub custom_counts: Vec<(String, usize)>,
    pub unidentified_count: usize,
    pub total_count: usize,
}

/// Multi-classifier that combines multiple classifiers
pub struct MultiClassifier {
    classifiers: Vec<Box<dyn CellClassifier>>,
    name: String,
}

impl MultiClassifier {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            classifiers: Vec::new(),
            name: name.into(),
        }
    }

    /// Add a classifier (higher priority classifiers are evaluated first)
    pub fn add_classifier(mut self, classifier: Box<dyn CellClassifier>) -> Self {
        self.classifiers.push(classifier);
        // Sort by priority (highest first)
        self.classifiers
            .sort_by(|a, b| b.priority().cmp(&a.priority()));
        self
    }
}

impl CellClassifier for MultiClassifier {
    fn classify(&self, cell: &CellInfo) -> Result<CellClass, crate::errors::Error> {
        // Try each classifier in priority order
        for classifier in &self.classifiers {
            let result = classifier.classify(cell)?;
            if !result.is_unidentified() {
                return Ok(result);
            }
        }
        Ok(CellClass::Unidentified)
    }

    fn name(&self) -> &str {
        &self.name
    }
}



/// Main cell collector that orchestrates the classification process
pub struct CellCollector<C: CellClassifier> {
    classifier: C,
    strict_mode: bool,
}

impl<C: CellClassifier> CellCollector<C> {
    pub fn new(classifier: C) -> Self {
        Self {
            classifier,
            strict_mode: true,
        }
    }

    /// Enable or disable strict mode (reject transactions with unidentified cells)
    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    /// Collect and classify cells from a specific source
    pub fn collect_from_source(
        &self,
        source: Source,
    ) -> Result<ClassifiedCells, crate::errors::Error> {
        debug_trace!(
            "Collecting cells from source: {:?} using classifier: {}",
            source,
            self.classifier.name()
        );
        let mut classified = ClassifiedCells::new();
        let mut index = 0;

        // Special handling for CellDep source to avoid loading data from depGroups
        if source == Source::CellDep {
            // For CellDeps, we check the type script first to determine if we need to load data
            // This avoids trying to load data from depGroups which causes memory allocation errors
            loop {
                // First check if the cell exists by trying to load its lock
                match load_cell_lock(index, source) {
                    Ok(lock) => {
                        // Load the type script to check if this is a cell we care about
                        let type_script = load_cell_type(index, source)?;
                        
                        // Check if this cell has a type script and if we recognize it
                        let should_load_data = match type_script {
                            Some(ref ts) => {
                                // Check if the classifier recognizes this type script
                                // This will be true for protocol, campaign, user cells etc.
                                // but false for unknown types and importantly, depGroups
                                let code_hash = ts.code_hash();
                                let type_code_hash_slice = code_hash.as_slice();
                                let mut type_code_hash = [0u8; 32];
                                type_code_hash.copy_from_slice(type_code_hash_slice);
                                self.classifier.should_load_data_for_type(&type_code_hash)
                            }
                            None => {
                                // No type script, it's a simple CKB cell in CellDeps
                                // We don't need to load data for simple cells in CellDeps
                                false
                            }
                        };
                        
                        let data = if should_load_data {
                            // Try to load data, but if it fails (shouldn't happen for known types),
                            // use empty data
                            load_cell_data(index, source).unwrap_or_else(|_| {
                                debug_info!("Failed to load data for CellDep {} despite having recognized type", index);
                                Vec::new()
                            })
                        } else {
                            // Don't load data for unrecognized types or depGroups
                            debug_info!("Skipping data load for CellDep {} (unrecognized or depGroup)", index);
                            Vec::new()
                        };
                        
                        match self.load_cell_info(source, index, data) {
                            Ok(cell_info) => {
                                let classification = self.classifier.classify(&cell_info)?;
                                debug_info!("Cell {} classified as: {:?}", index, classification);
                                classified.add_cell(cell_info, classification);
                            }
                            Err(e) => {
                                debug_info!("Failed to load cell info for CellDep {}: {:?}", index, e);
                                // Skip this cell and continue
                            }
                        }
                        index += 1;
                    }
                    Err(_) => {
                        // No more cells
                        break;
                    }
                }
            }
        } else {
            // For Input and Output sources, use the original logic
            while let Ok(data) = load_cell_data(index, source) {
                let cell_info = self.load_cell_info(source, index, data)?;
                let classification = self.classifier.classify(&cell_info)?;

                debug_info!("Cell {} classified as: {:?}", index, classification);
                classified.add_cell(cell_info, classification);

                index += 1;
            }
        }

        debug_info!(
            "Collected {} total cells, {} unidentified",
            classified.total_cell_count(),
            classified.unidentified_cells.len()
        );

        // In strict mode, reject transactions with unidentified cells
        if self.strict_mode && classified.has_unidentified_cells() {
            debug_info!(
                "Strict mode: rejecting transaction due to {} unidentified cells",
                classified.unidentified_cells.len()
            );
            return Err(crate::errors::Error::UnidentifiedCells);
        }

        Ok(classified)
    }

    /// Collect and classify cells from both input and output sources
    pub fn collect_cells(
        &self,
    ) -> Result<(ClassifiedCells, ClassifiedCells, ClassifiedCells), crate::errors::Error> {
        let inputs = self.collect_from_source(Source::Input)?;
        let outputs = self.collect_from_source(Source::Output)?;
        let cell_deps = self.collect_from_source(Source::CellDep)?;
        Ok((inputs, outputs, cell_deps))
    }

    /// Load complete cell information
    fn load_cell_info(
        &self,
        source: Source,
        index: usize,
        data: Vec<u8>,
    ) -> Result<CellInfo, crate::errors::Error> {
        let lock = load_cell_lock(index, source)?;
        let lock_hash = load_cell_lock_hash(index, source)?;
        let type_script = load_cell_type(index, source)?;
        let type_hash = load_cell_type_hash(index, source)?;

        Ok(CellInfo {
            source,
            index,
            data,
            lock,
            lock_hash,
            type_script,
            type_hash,
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_class() {
        let known = CellClass::known("protocol");
        assert!(known.is_known("protocol"));
        assert!(!known.is_known("campaign"));

        let custom = CellClass::custom("custom_id");
        assert!(custom.is_custom("custom_id"));
        assert!(!custom.is_custom("other_id"));

        let unidentified = CellClass::Unidentified;
        assert!(unidentified.is_unidentified());
    }

    #[test]
    fn test_rule_based_classifier() {
        // Create a simple type hash for testing
        let protocol_type_hash = [1u8; 32];
        let campaign_type_hash = [2u8; 32];

        let classifier = RuleBasedClassifier::new("test")
            .add_type_hash(protocol_type_hash, CellClass::known("protocol"))
            .add_type_hash(campaign_type_hash, CellClass::known("campaign"));

        let protocol_cell = CellInfo {
            source: Source::Input,
            index: 0,
            data: b"ProtocolData".to_vec(),
            lock: Script::default(),
            lock_hash: [0u8; 32],
            type_script: Some(Script::default()),
            type_hash: Some(protocol_type_hash),
        };

        let result = classifier.classify(&protocol_cell).unwrap();
        assert!(result.is_known("protocol"));

        let campaign_cell = CellInfo {
            source: Source::Input,
            index: 1,
            data: b"CampaignData".to_vec(),
            lock: Script::default(),
            lock_hash: [0u8; 32],
            type_script: Some(Script::default()),
            type_hash: Some(campaign_type_hash),
        };

        let result2 = classifier.classify(&campaign_cell).unwrap();
        assert!(result2.is_known("campaign"));
    }

    #[test]
    fn test_classified_cells() {
        let mut cells = ClassifiedCells::new();

        let cell = CellInfo {
            source: Source::Input,
            index: 0,
            data: Vec::new(),
            lock: Script::default(),
            lock_hash: [0u8; 32],
            type_script: None,
            type_hash: None,
        };

        cells.add_cell(cell, CellClass::known("protocol"));
        assert_eq!(cells.get_custom("protocol").unwrap().len(), 1);
        assert_eq!(cells.total_cell_count(), 1);
        assert!(!cells.has_unidentified_cells());
    }
}
