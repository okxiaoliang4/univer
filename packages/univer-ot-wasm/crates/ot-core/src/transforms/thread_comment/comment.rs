use crate::mutations::thread_comment::{
    AddCommentMutation, UpdateCommentMutation, UpdateCommentRefMutation,
    ResolveCommentMutation, DeleteCommentMutation,
};
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, TransformResultRef};
use crate::transforms::constants::*;
use std::sync::Arc;

pub const ADD_COMMENT_ID: MutationId = AddCommentMutation::ID;
pub const UPDATE_COMMENT_ID: MutationId = UpdateCommentMutation::ID;
pub const UPDATE_COMMENT_REF_ID: MutationId = UpdateCommentRefMutation::ID;
pub const RESOLVE_COMMENT_ID: MutationId = ResolveCommentMutation::ID;
pub const DELETE_COMMENT_ID: MutationId = DeleteCommentMutation::ID;

const LOCAL_COMMENT_MUTATIONS: &[MutationId] = &[
    ADD_COMMENT_ID,
    UPDATE_COMMENT_ID,
    UPDATE_COMMENT_REF_ID,
    RESOLVE_COMMENT_ID,
    DELETE_COMMENT_ID,
];

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register symmetric transforms for comment mutations
    registry.register_symmetric_ref(ADD_COMMENT_ID, create_identity());
    registry.register_symmetric_ref(UPDATE_COMMENT_ID, create_lww());
    registry.register_symmetric_ref(UPDATE_COMMENT_REF_ID, create_lww());
    registry.register_symmetric_ref(RESOLVE_COMMENT_ID, create_lww());
    registry.register_symmetric_ref(DELETE_COMMENT_ID, create_identity());

    // Register bidirectional transforms within comment module
    registry.register_bidirectional_ref(ADD_COMMENT_ID, UPDATE_COMMENT_ID, create_identity());
    registry.register_bidirectional_ref(ADD_COMMENT_ID, UPDATE_COMMENT_REF_ID, create_identity());
    registry.register_bidirectional_ref(ADD_COMMENT_ID, RESOLVE_COMMENT_ID, create_identity());
    registry.register_bidirectional_ref(ADD_COMMENT_ID, DELETE_COMMENT_ID, create_identity());
    registry.register_bidirectional_ref(UPDATE_COMMENT_ID, UPDATE_COMMENT_REF_ID, create_identity());
    registry.register_bidirectional_ref(UPDATE_COMMENT_ID, RESOLVE_COMMENT_ID, create_identity());
    registry.register_bidirectional_ref(UPDATE_COMMENT_ID, DELETE_COMMENT_ID, create_identity());
    registry.register_bidirectional_ref(UPDATE_COMMENT_REF_ID, RESOLVE_COMMENT_ID, create_identity());
    registry.register_bidirectional_ref(UPDATE_COMMENT_REF_ID, DELETE_COMMENT_ID, create_identity());
    registry.register_bidirectional_ref(RESOLVE_COMMENT_ID, DELETE_COMMENT_ID, create_identity());

    // Register with all other modules
    for &comment_id in LOCAL_COMMENT_MUTATIONS {
        for &sheet_id in ALL_SHEETS_CORE_MUTATIONS {
            registry.register_identity(comment_id, sheet_id);
        }
        for &dv_id in DATA_VALIDATION_MUTATIONS {
            registry.register_identity(comment_id, dv_id);
        }
        for &cf_id in CONDITIONAL_FORMATTING_MUTATIONS {
            registry.register_identity(comment_id, cf_id);
        }
        for &filter_id in FILTER_MUTATIONS {
            registry.register_identity(comment_id, filter_id);
        }
        for &hl_id in HYPER_LINK_MUTATIONS {
            registry.register_identity(comment_id, hl_id);
        }
        for &note_id in NOTE_MUTATIONS {
            registry.register_identity(comment_id, note_id);
        }
        for &table_id in TABLE_MUTATIONS {
            registry.register_identity(comment_id, table_id);
        }
        for &pt_id in PIVOT_TABLE_MUTATIONS {
            registry.register_identity(comment_id, pt_id);
        }
    }
}

fn create_identity() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        TransformResultRef::identity(m1, m2)
    })
}

fn create_lww() -> TransformFnRef {
    Arc::new(|_m1: &MutationInfo, m2: &MutationInfo| {
        use crate::types::MutationOutcome;
        TransformResultRef {
            m1_prime: MutationOutcome::Removed,
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
