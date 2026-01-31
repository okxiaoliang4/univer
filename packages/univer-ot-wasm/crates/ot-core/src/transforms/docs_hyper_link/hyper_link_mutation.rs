use crate::registry::TransformRegistry;

/// Register transforms for document hyperlink mutations
///
/// Document hyperlinks are position-based within the document text,
/// using character offsets rather than cell coordinates.
///
/// Transform strategies:
/// - Position shifts based on text insertion/deletion
/// - Different from sheets hyperlinks (which use row/col coordinates)
///
/// This is currently out of scope for the sheets-focused OT implementation.
pub fn register_transforms(_registry: &mut TransformRegistry) {
    // TODO: Implement document hyperlink transforms:
    // - Position adjustments based on text edits
    // - Hyperlink overlap resolution
    // - Interaction with rich text formatting
    //
    // Note: This requires coordination with document OT system
}
