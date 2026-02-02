use crate::mutations::thread_comment::{
    AddCommentMutation, UpdateCommentMutation, UpdateCommentRefMutation,
    ResolveCommentMutation, DeleteCommentMutation,
};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::transform_helpers::{lww_transform, identity_transform};

pub const ADD_COMMENT_ID: MutationId = AddCommentMutation::ID;
pub const UPDATE_COMMENT_ID: MutationId = UpdateCommentMutation::ID;
pub const UPDATE_COMMENT_REF_ID: MutationId = UpdateCommentRefMutation::ID;
pub const RESOLVE_COMMENT_ID: MutationId = ResolveCommentMutation::ID;
pub const DELETE_COMMENT_ID: MutationId = DeleteCommentMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register symmetric transforms for comment mutations
    registry.register_symmetric_ref(ADD_COMMENT_ID, identity_transform());
    registry.register_symmetric_ref(UPDATE_COMMENT_ID, lww_transform());
    registry.register_symmetric_ref(UPDATE_COMMENT_REF_ID, lww_transform());
    registry.register_symmetric_ref(RESOLVE_COMMENT_ID, lww_transform());
    registry.register_symmetric_ref(DELETE_COMMENT_ID, identity_transform());

    // Register bidirectional transforms within comment module
    registry.register_bidirectional_ref(ADD_COMMENT_ID, UPDATE_COMMENT_ID, identity_transform());
    registry.register_bidirectional_ref(ADD_COMMENT_ID, UPDATE_COMMENT_REF_ID, identity_transform());
    registry.register_bidirectional_ref(ADD_COMMENT_ID, RESOLVE_COMMENT_ID, identity_transform());
    registry.register_bidirectional_ref(ADD_COMMENT_ID, DELETE_COMMENT_ID, identity_transform());
    registry.register_bidirectional_ref(UPDATE_COMMENT_ID, UPDATE_COMMENT_REF_ID, identity_transform());
    registry.register_bidirectional_ref(UPDATE_COMMENT_ID, RESOLVE_COMMENT_ID, identity_transform());
    registry.register_bidirectional_ref(UPDATE_COMMENT_ID, DELETE_COMMENT_ID, identity_transform());
    registry.register_bidirectional_ref(UPDATE_COMMENT_REF_ID, RESOLVE_COMMENT_ID, identity_transform());
    registry.register_bidirectional_ref(UPDATE_COMMENT_REF_ID, DELETE_COMMENT_ID, identity_transform());
    registry.register_bidirectional_ref(RESOLVE_COMMENT_ID, DELETE_COMMENT_ID, identity_transform());

    // NOTE: No register_identity calls needed - registry falls back to identity automatically
}