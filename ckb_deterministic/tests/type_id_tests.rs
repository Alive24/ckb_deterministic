//! Tests for the Type ID module
//!
//! These tests demonstrate the usage of Type ID functionality
//! in the ckb_deterministic library.

#[cfg(test)]
mod tests {
    use ckb_deterministic::type_id::calculate_type_id;

    #[test]
    fn test_type_id_calculation_basic() {
        // Example: Calculate Type ID for a transaction
        let input_data = b"example_input_cell_data";
        let output_index = 0;

        let type_id = calculate_type_id(input_data, output_index);

        // Type ID should always be 32 bytes
        assert_eq!(type_id.len(), 32);

        // The same input and output index should always produce the same Type ID
        let type_id2 = calculate_type_id(input_data, output_index);
        assert_eq!(type_id, type_id2);
    }

    #[test]
    fn test_type_id_uniqueness() {
        let input_data = b"test_input";

        // Different output indices produce different Type IDs
        let type_id_0 = calculate_type_id(input_data, 0);
        let type_id_1 = calculate_type_id(input_data, 1);
        assert_ne!(type_id_0, type_id_1);

        // Different input data produces different Type IDs
        let type_id_a = calculate_type_id(b"input_a", 0);
        let type_id_b = calculate_type_id(b"input_b", 0);
        assert_ne!(type_id_a, type_id_b);
    }

    #[test]
    fn test_type_id_deterministic() {
        // Type ID calculation is deterministic
        let input = b"deterministic_test";
        let index = 42;

        // Calculate multiple times
        let results: Vec<[u8; 32]> = (0..10).map(|_| calculate_type_id(input, index)).collect();

        // All results should be identical
        for i in 1..results.len() {
            assert_eq!(results[0], results[i]);
        }
    }
}
