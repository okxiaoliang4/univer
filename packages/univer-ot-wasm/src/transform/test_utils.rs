#[cfg(test)]
pub mod test_utils {
    use crate::types::MutationInfoInternal;
    use serde_json;

    /// Helper function to create MutationInfoInternal from serde_json::Value for testing
    /// This works in both WASM and non-WASM test environments
    pub fn create_mutation_info(id: String, params: serde_json::Value) -> MutationInfoInternal {
        MutationInfoInternal { id, params }
    }

    /// Helper function to convert MutationInfoInternal back to serde_json::Value for testing
    pub fn mutation_info_to_json(mi: &MutationInfoInternal) -> serde_json::Value {
        mi.params.clone()
    }
}
