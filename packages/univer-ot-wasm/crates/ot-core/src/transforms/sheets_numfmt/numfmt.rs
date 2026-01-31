use crate::registry::TransformRegistry;

/// Register transforms for number format mutations
///
/// Mutations handled:
/// - SetNumfmtMutation: sheet.mutation.set.numfmt
/// - RemoveNumfmtMutation: sheet.mutation.remove.numfmt
///
/// See: .claude/skills/univer-ot-dev/references/mutations/numfmt_mutations.md
pub fn register_transforms(_registry: &mut TransformRegistry) {
    // TODO: Implement SetNumfmtMutation transforms:
    // - Self-transform: Complex range overlap arithmetic
    // - vs InsertRow/Col: Shift ranges in ref_map
    // - vs RemoveRows/Cols: Shift/remove ranges
    // - vs MoveRows/Cols/Range: Complex range adjustments

    // TODO: Implement RemoveNumfmtMutation transforms:
    // - Self-transform: Union of ranges (idempotent)
    // - vs InsertRow/Col: Shift ranges
    // - vs RemoveRows/Cols: Shift/remove ranges

    // TODO: Implement cross-mutation transforms:
    // - SetNumfmt vs RemoveNumfmt: Conflict resolution
}
