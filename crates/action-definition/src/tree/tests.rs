use super::*;
use crate::actions::{click::Click, message_box::MessageBox, test::Test};

#[test]
fn default_tree_contains_only_root() {
    let tree = ActionTree::default();
    assert_eq!(tree.rows, vec![tree.root]);
    assert_eq!(tree.row_of[tree.root], 0);
    assert_eq!(tree.node_row(tree.root).unwrap(), 0);
    assert!(matches!(
        tree.map[tree.root].payload,
        NodePayload::Static(Static::Root),
    ));
    assert!(tree.map[tree.root].children.is_empty());
}

#[test]
fn append_action_links_into_parent_and_indexes_row() {
    let mut tree = ActionTree::default();
    let action = tree
        .append_new_action(&Click::DEFINITION, tree.root)
        .unwrap();
    assert_eq!(tree.map[tree.root].children, vec![action]);
    assert_eq!(tree.map[action].parent_id, Some(tree.root));
    assert_eq!(tree.rows, vec![tree.root, action]);
    assert_eq!(tree.row_of[action], 1);
    assert_eq!(tree.node_row(action).unwrap(), 1);
}

#[test]
fn set_node_label_updates_node_and_label_lookup() {
    let mut tree = ActionTree::default();
    let action = tree
        .append_new_action(&Click::DEFINITION, tree.root)
        .unwrap();

    tree.set_node_label(action, "click_target").unwrap();

    assert_eq!(tree.map[action].label(), Some("click_target"));
    assert_eq!(tree.node_by_label("click_target"), Some(action));
}

#[test]
fn set_node_label_replaces_old_label_lookup() {
    let mut tree = ActionTree::default();
    let action = tree
        .append_new_action(&Click::DEFINITION, tree.root)
        .unwrap();

    tree.set_node_label(action, "old_label").unwrap();
    tree.set_node_label(action, "new_label").unwrap();

    assert_eq!(tree.map[action].label(), Some("new_label"));
    assert_eq!(tree.node_by_label("old_label"), None);
    assert_eq!(tree.node_by_label("new_label"), Some(action));
}

#[test]
fn set_node_label_rejects_duplicate_label_on_another_node() {
    let mut tree = ActionTree::default();
    let first = tree
        .append_new_action(&Click::DEFINITION, tree.root)
        .unwrap();
    let second = tree
        .append_new_action(&Click::DEFINITION, tree.root)
        .unwrap();

    tree.set_node_label(first, "shared_label").unwrap();
    let error = tree.set_node_label(second, "shared_label").unwrap_err();

    assert!(matches!(error, Error::DuplicateLabel(label) if label == "shared_label"));
    assert_eq!(tree.map[first].label(), Some("shared_label"));
    assert_eq!(tree.map[second].label(), None);
    assert_eq!(tree.node_by_label("shared_label"), Some(first));
}

#[test]
fn clear_node_label_removes_node_label_and_lookup() {
    let mut tree = ActionTree::default();
    let action = tree
        .append_new_action(&Click::DEFINITION, tree.root)
        .unwrap();

    tree.set_node_label(action, "temporary_label").unwrap();
    tree.clear_node_label(action).unwrap();

    assert_eq!(tree.map[action].label(), None);
    assert_eq!(tree.node_by_label("temporary_label"), None);
}

#[test]
fn empty_label_clears_node_label_and_lookup() {
    let mut tree = ActionTree::default();
    let action = tree
        .append_new_action(&Click::DEFINITION, tree.root)
        .unwrap();

    tree.set_node_label(action, "temporary_label").unwrap();
    tree.set_node_label(action, "").unwrap();

    assert_eq!(tree.map[action].label(), None);
    assert_eq!(tree.node_by_label("temporary_label"), None);
    assert_eq!(tree.node_by_label(""), None);
}

#[test]
fn node_by_label_returns_none_for_unknown_label() {
    let tree = ActionTree::default();

    assert_eq!(tree.node_by_label("missing_label"), None);
}

#[test]
fn action_tree_serializes_only_persistent_state() {
    let mut tree = ActionTree::default();
    let action = tree
        .append_new_action(&Test::DEFINITION, tree.root)
        .unwrap();
    let true_branch = tree.map[action].children[0];
    let nested = tree
        .append_new_action(&Click::DEFINITION, true_branch)
        .unwrap();

    tree.set_node_label(action, "condition").unwrap();
    tree.set_node_comment(nested, "Nested click").unwrap();
    tree.set_node_collapsed(action, true).unwrap();

    let json = serde_json::to_value(&tree).expect("serialize tree");
    let object = json.as_object().expect("tree serializes as object");

    assert_eq!(object.len(), 2);
    assert!(object.contains_key("map"));
    assert!(object.contains_key("root"));
    assert!(!object.contains_key("label_map"));
    assert!(!object.contains_key("rows"));
    assert!(!object.contains_key("row_of"));

    assert_json_key_absent(&json, "depth");
}

#[test]
fn action_tree_deserialization_rebuilds_indexes_and_depths() {
    let mut tree = ActionTree::default();
    let action = tree
        .append_new_action(&Test::DEFINITION, tree.root)
        .unwrap();
    let true_branch = tree.map[action].children[0];
    let false_branch = tree.map[action].children[1];
    let nested = tree
        .append_new_action(&Click::DEFINITION, true_branch)
        .unwrap();

    tree.set_node_label(action, "condition").unwrap();
    tree.set_node_label(nested, "click_target").unwrap();
    tree.set_node_collapsed(action, true).unwrap();

    let mut json = serde_json::to_value(&tree).expect("serialize tree");
    inject_legacy_depth_fields(&mut json, 99);

    let deserialized: ActionTree = serde_json::from_value(json).expect("deserialize tree");

    assert_eq!(deserialized.rows, tree.rows);
    for (row, &node_id) in tree.rows.iter().enumerate() {
        assert_eq!(deserialized.node_row(node_id).unwrap(), row);
    }
    assert_eq!(deserialized.node_by_label("condition"), Some(action));
    assert_eq!(deserialized.node_by_label("click_target"), Some(nested));

    assert_eq!(deserialized.map[deserialized.root].depth(), 0);
    assert_eq!(deserialized.map[action].depth(), 1);
    assert_eq!(deserialized.map[true_branch].depth(), 2);
    assert_eq!(deserialized.map[false_branch].depth(), 2);
    assert_eq!(deserialized.map[nested].depth(), 3);
    assert!(deserialized.map[action].is_collapsed());
}

#[test]
fn set_and_clear_node_comment_updates_node_metadata() {
    let mut tree = ActionTree::default();
    let action = tree
        .append_new_action(&Click::DEFINITION, tree.root)
        .unwrap();

    tree.set_node_comment(action, "Click the submit button")
        .unwrap();
    assert_eq!(tree.map[action].comment(), Some("Click the submit button"));

    tree.clear_node_comment(action).unwrap();
    assert_eq!(tree.map[action].comment(), None);
}

#[test]
fn empty_comment_clears_node_comment() {
    let mut tree = ActionTree::default();
    let action = tree
        .append_new_action(&Click::DEFINITION, tree.root)
        .unwrap();

    tree.set_node_comment(action, "Click the submit button")
        .unwrap();
    tree.set_node_comment(action, "").unwrap();

    assert_eq!(tree.map[action].comment(), None);
}

#[test]
fn append_action_with_branches_lays_out_subtree_in_preorder() {
    let mut tree = ActionTree::default();
    let action = tree
        .append_new_action(&Test::DEFINITION, tree.root)
        .unwrap();
    let branches = tree.map[action].children.clone();
    assert_eq!(branches.len(), 2);
    assert_eq!(tree.rows, vec![tree.root, action, branches[0], branches[1]]);
    assert_eq!(tree.row_of[branches[0]], 2);
    assert_eq!(tree.row_of[branches[1]], 3);
    assert_eq!(tree.node_row(action).unwrap(), 1);
    assert_eq!(tree.node_row(branches[0]).unwrap(), 2);
    assert_eq!(tree.node_row(branches[1]).unwrap(), 3);
    for &branch in &branches {
        assert_eq!(tree.map[branch].parent_id, Some(action));
    }
}

#[test]
fn appending_a_second_sibling_keeps_preorder() {
    let mut tree = ActionTree::default();
    let first = tree
        .append_new_action(&Test::DEFINITION, tree.root)
        .unwrap();
    let first_branches = tree.map[first].children.clone();
    let second = tree
        .append_new_action(&Click::DEFINITION, tree.root)
        .unwrap();

    assert_eq!(tree.map[tree.root].children, vec![first, second]);
    assert_eq!(
        tree.rows,
        vec![
            tree.root,
            first,
            first_branches[0],
            first_branches[1],
            second
        ],
    );
    assert_eq!(tree.row_of[second], 4);
}

#[test]
fn nodes_record_their_depth_in_the_tree() {
    let mut tree = ActionTree::default();
    assert_eq!(tree.map[tree.root].depth(), 0);

    let action = tree
        .append_new_action(&Test::DEFINITION, tree.root)
        .unwrap();
    assert_eq!(tree.map[action].depth(), 1);

    let branch = tree.map[action].children[0];
    assert_eq!(tree.map[branch].depth(), 2);

    let nested = tree.append_new_action(&Click::DEFINITION, branch).unwrap();
    assert_eq!(tree.map[nested].depth(), 3);
}

#[test]
fn nodes_are_expanded_and_show_all_rows_by_default() {
    let mut tree = ActionTree::default();
    let action = tree
        .append_new_action(&Test::DEFINITION, tree.root)
        .unwrap();

    assert!(!tree.map[action].is_collapsed());
    assert_eq!(tree.visible_rows(), tree.rows()[1..].to_vec());
}

#[test]
fn collapsing_a_node_hides_its_descendants_from_visible_rows() {
    let mut tree = ActionTree::default();
    let action = tree
        .append_new_action(&Test::DEFINITION, tree.root)
        .unwrap();
    // The action has branch children that are visible while expanded.
    assert!(tree.map[action].has_children());

    tree.set_node_collapsed(action, true).unwrap();

    // The action stays visible, but its subtree is gone.
    assert_eq!(tree.visible_rows(), vec![action]);
    assert!(tree.map[action].is_collapsed());

    tree.set_node_collapsed(action, false).unwrap();
    assert_eq!(tree.visible_rows(), tree.rows()[1..].to_vec());
}

#[test]
fn collapsing_a_node_hides_deeper_descendants_not_just_children() {
    let mut tree = ActionTree::default();
    let action = tree
        .append_new_action(&Test::DEFINITION, tree.root)
        .unwrap();
    let true_branch = tree.map[action].children[0];
    let nested = tree
        .append_new_action(&Click::DEFINITION, true_branch)
        .unwrap();

    // Collapsing the top action must hide the branch (a child) *and* the
    // nested action (a grandchild): visibility depends on all ancestors.
    tree.set_node_collapsed(action, true).unwrap();

    let visible = tree.visible_rows();
    assert_eq!(visible, vec![action]);
    assert!(!visible.contains(&true_branch));
    assert!(!visible.contains(&nested));
}

#[test]
fn ancestors_walk_from_parent_up_to_root() {
    let mut tree = ActionTree::default();
    let action = tree
        .append_new_action(&Test::DEFINITION, tree.root)
        .unwrap();
    let branch = tree.map[action].children[0];
    let nested = tree.append_new_action(&Click::DEFINITION, branch).unwrap();

    assert_eq!(
        tree.ancestors(nested).collect::<Vec<_>>(),
        vec![branch, action, tree.root],
    );
    assert!(tree.ancestors(tree.root).next().is_none());

    assert!(tree.is_ancestor(action, nested));
    assert!(tree.is_ancestor(tree.root, nested));
    assert!(!tree.is_ancestor(nested, action));
    // Strict: a node is not its own ancestor.
    assert!(!tree.is_ancestor(action, action));
}

fn drag_tree() -> (ActionTree, NodeId, NodeId, NodeId, NodeId, NodeId, NodeId) {
    let mut tree = ActionTree::default();
    let test = tree
        .append_new_action(&Test::DEFINITION, tree.root)
        .unwrap();
    let true_branch = tree.map[test].children[0];
    let false_branch = tree.map[test].children[1];
    let true_click = tree
        .append_new_action(&Click::DEFINITION, true_branch)
        .unwrap();
    let false_click = tree
        .append_new_action(&Click::DEFINITION, false_branch)
        .unwrap();
    let final_click = tree
        .append_new_action(&Click::DEFINITION, tree.root)
        .unwrap();

    (
        tree,
        test,
        true_branch,
        true_click,
        false_branch,
        false_click,
        final_click,
    )
}

#[test]
fn can_drop_rejects_empty_root_branch_and_duplicate_sources() {
    let (tree, test, true_branch, true_click, ..) = drag_tree();

    assert!(!tree.can_drop(&[], tree.root, DropMode::AppendChild));
    assert!(!tree.can_drop(&[tree.root], true_branch, DropMode::AppendChild));
    assert!(!tree.can_drop(&[true_branch], tree.root, DropMode::AppendChild));
    assert!(!tree.can_drop(&[true_click, true_click], tree.root, DropMode::AppendChild,));
    assert!(tree.can_drop(&[true_click], tree.root, DropMode::AppendChild));
    assert!(tree.can_drop(&[true_click], test, DropMode::Before));
}

#[test]
fn can_drop_rejects_selection_with_ancestor_and_descendant() {
    let (tree, test, _, true_click, ..) = drag_tree();

    assert!(!tree.can_drop(&[test, true_click], tree.root, DropMode::AppendChild));
    assert!(!tree.can_drop(&[true_click, test], tree.root, DropMode::AppendChild));
}

#[test]
fn can_drop_rejects_self_and_own_subtree_targets() {
    let (tree, test, true_branch, true_click, ..) = drag_tree();

    assert!(!tree.can_drop(&[test], test, DropMode::Before));
    assert!(!tree.can_drop(&[test], true_branch, DropMode::AppendChild));
    assert!(!tree.can_drop(&[test], true_click, DropMode::Before));
}

#[test]
fn can_drop_enforces_target_mode_rules() {
    let (tree, test, true_branch, true_click, ..) = drag_tree();

    assert!(tree.can_drop(&[true_click], tree.root, DropMode::AppendChild));
    assert!(tree.can_drop(&[true_click], true_branch, DropMode::AppendChild));
    assert!(!tree.can_drop(&[true_click], test, DropMode::AppendChild));

    assert!(tree.can_drop(&[true_click], test, DropMode::Before));
    assert!(tree.can_drop(&[true_click], test, DropMode::After));
    assert!(!tree.can_drop(&[true_click], tree.root, DropMode::Before));
    assert!(!tree.can_drop(&[true_click], true_branch, DropMode::After));
}

#[test]
fn move_nodes_appends_action_to_container_and_updates_rows_and_depths() {
    let (mut tree, test, true_branch, true_click, false_branch, false_click, final_click) =
        drag_tree();

    tree.move_nodes(&[final_click], true_branch, DropMode::AppendChild)
        .unwrap();

    assert_eq!(tree.map[tree.root].children, vec![test]);
    assert_eq!(
        tree.map[true_branch].children,
        vec![true_click, final_click]
    );
    assert_eq!(tree.map[final_click].parent_id, Some(true_branch));
    assert_eq!(tree.map[final_click].depth(), 3);
    assert_eq!(
        tree.rows,
        vec![
            tree.root,
            test,
            true_branch,
            true_click,
            final_click,
            false_branch,
            false_click,
        ],
    );
}

#[test]
fn move_nodes_before_action_preserves_mixed_parent_selection_order() {
    let (mut tree, test, true_branch, true_click, false_branch, false_click, final_click) =
        drag_tree();

    tree.move_nodes(&[false_click, final_click], true_click, DropMode::Before)
        .unwrap();

    assert_eq!(
        tree.map[true_branch].children,
        vec![false_click, final_click, true_click],
    );
    assert!(tree.map[false_branch].children.is_empty());
    assert_eq!(tree.map[tree.root].children, vec![test]);
    assert_eq!(tree.map[false_click].parent_id, Some(true_branch));
    assert_eq!(tree.map[final_click].parent_id, Some(true_branch));
    assert_eq!(
        tree.rows,
        vec![
            tree.root,
            test,
            true_branch,
            false_click,
            final_click,
            true_click,
            false_branch,
        ],
    );
}

#[test]
fn move_nodes_after_action_preserves_labels() {
    let (mut tree, _, _, true_click, _, false_click, _) = drag_tree();
    tree.set_node_label(false_click, "moved_click").unwrap();

    tree.move_nodes(&[false_click], true_click, DropMode::After)
        .unwrap();

    assert_eq!(tree.map[false_click].label(), Some("moved_click"));
    assert_eq!(tree.node_by_label("moved_click"), Some(false_click));
}

#[test]
fn move_nodes_rechecks_can_drop() {
    let (mut tree, test, true_branch, ..) = drag_tree();

    assert!(matches!(
        tree.move_nodes(&[test], true_branch, DropMode::AppendChild),
        Err(Error::DropNotAllowed),
    ));
}

#[test]
fn copy_subtrees_rejects_static_roots() {
    let (tree, _test, true_branch, ..) = drag_tree();

    assert!(matches!(
        tree.copy_subtrees(&[true_branch]),
        Err(Error::DropNotAllowed),
    ));
}

#[test]
fn paste_subtrees_appends_cloned_subtree_and_preserves_order() {
    let (mut tree, test, true_branch, true_click, false_branch, false_click, final_click) =
        drag_tree();
    tree.set_node_label(test, "decision").unwrap();
    tree.set_node_label(true_click, "accept").unwrap();

    let clipboard = tree.copy_subtrees(&[test]).unwrap();
    let pasted = tree
        .paste_subtrees(&clipboard, tree.root(), DropMode::AppendChild)
        .unwrap();

    assert_eq!(pasted.len(), 1);
    let pasted_test = pasted[0];
    let pasted_branches = tree.map[pasted_test].children.clone();
    assert_eq!(pasted_branches.len(), 2);
    assert_eq!(
        tree.map[tree.root].children,
        vec![test, final_click, pasted_test]
    );
    assert_eq!(
        tree.rows,
        vec![
            tree.root,
            test,
            true_branch,
            true_click,
            false_branch,
            false_click,
            final_click,
            pasted_test,
            pasted_branches[0],
            tree.map[pasted_branches[0]].children[0],
            pasted_branches[1],
            tree.map[pasted_branches[1]].children[0],
        ],
    );
    assert_eq!(tree.map[pasted_test].depth(), 1);
    assert_eq!(tree.map[pasted_branches[0]].depth(), 2);
    assert_eq!(
        tree.map[tree.map[pasted_branches[0]].children[0]].depth(),
        3
    );
}

#[test]
fn paste_subtrees_resolves_label_conflicts() {
    let (mut tree, test, _, true_click, ..) = drag_tree();
    tree.set_node_label(test, "decision").unwrap();
    tree.set_node_label(true_click, "accept").unwrap();

    let clipboard = tree.copy_subtrees(&[test]).unwrap();
    let pasted = tree
        .paste_subtrees(&clipboard, tree.root(), DropMode::AppendChild)
        .unwrap();

    let pasted_test = pasted[0];
    let pasted_true_branch = tree.map[pasted_test].children[0];
    let pasted_true_click = tree.map[pasted_true_branch].children[0];

    assert_eq!(tree.map[pasted_test].label(), Some("decision_copy"));
    assert_eq!(tree.map[pasted_true_click].label(), Some("accept_copy"));
    assert_eq!(tree.node_by_label("decision"), Some(test));
    assert_eq!(tree.node_by_label("decision_copy"), Some(pasted_test));
    assert_eq!(tree.node_by_label("accept"), Some(true_click));
    assert_eq!(tree.node_by_label("accept_copy"), Some(pasted_true_click));
}

#[test]
fn paste_subtrees_after_action_inserts_siblings() {
    let (mut tree, _test, _true_branch, true_click, _false_branch, false_click, final_click) =
        drag_tree();
    let clipboard = tree.copy_subtrees(&[false_click]).unwrap();

    let pasted = tree
        .paste_subtrees(&clipboard, true_click, DropMode::After)
        .unwrap();

    let pasted_click = pasted[0];
    let parent = tree.map[true_click].parent_id.unwrap();
    assert_eq!(tree.map[parent].children, vec![true_click, pasted_click],);
    assert_eq!(tree.map[pasted_click].parent_id, Some(parent));
    assert_eq!(tree.map[pasted_click].depth(), tree.map[true_click].depth());
    assert_eq!(tree.map[tree.root].children.last(), Some(&final_click));
}

#[test]
fn toggle_node_collapsed_flips_and_returns_the_new_state() {
    let mut tree = ActionTree::default();
    let action = tree
        .append_new_action(&Test::DEFINITION, tree.root)
        .unwrap();

    assert!(tree.toggle_node_collapsed(action).unwrap());
    assert!(tree.map[action].is_collapsed());
    assert!(!tree.toggle_node_collapsed(action).unwrap());
    assert!(!tree.map[action].is_collapsed());
}

#[test]
fn appending_under_a_branch_shifts_later_rows() {
    let mut tree = ActionTree::default();
    let parent = tree
        .append_new_action(&Test::DEFINITION, tree.root)
        .unwrap();
    let true_branch = tree.map[parent].children[0];
    let false_branch = tree.map[parent].children[1];
    let nested = tree
        .append_new_action(&Click::DEFINITION, true_branch)
        .unwrap();

    assert_eq!(
        tree.rows,
        vec![tree.root, parent, true_branch, nested, false_branch],
    );
    assert_eq!(tree.row_of[nested], 3);
    assert_eq!(tree.row_of[false_branch], 4);
    assert_eq!(tree.node_row(nested).unwrap(), 3);
    assert_eq!(tree.node_row(false_branch).unwrap(), 4);
}

#[test]
fn append_action_rejects_invalid_parent() {
    let mut tree = ActionTree::default();
    let stale = tree
        .append_new_action(&Click::DEFINITION, tree.root)
        .unwrap();
    tree.remove(stale).unwrap();
    assert!(tree.append_new_action(&Click::DEFINITION, stale).is_err());
}

#[test]
fn definition_returns_some_for_action_nodes() {
    let mut tree = ActionTree::default();
    let action = tree
        .append_new_action(&Click::DEFINITION, tree.root)
        .unwrap();
    assert!(tree.definition(action).unwrap().is_some());
}

#[test]
fn definition_returns_none_for_static_nodes() {
    let mut tree = ActionTree::default();
    assert!(tree.definition(tree.root).unwrap().is_none());

    let action = tree
        .append_new_action(&Test::DEFINITION, tree.root)
        .unwrap();
    let branch = tree.map[action].children[0];
    assert!(tree.definition(branch).unwrap().is_none());
}

#[test]
fn definition_errors_on_unknown_id() {
    let mut tree = ActionTree::default();
    let stale = tree
        .append_new_action(&Click::DEFINITION, tree.root)
        .unwrap();
    tree.remove(stale).unwrap();
    assert!(tree.definition(stale).is_err());
}

#[test]
fn remove_drops_node_and_reindexes_siblings() {
    let mut tree = ActionTree::default();
    let first = tree
        .append_new_action(&Click::DEFINITION, tree.root)
        .unwrap();
    let second = tree
        .append_new_action(&Click::DEFINITION, tree.root)
        .unwrap();

    tree.remove(first).unwrap();

    assert_eq!(tree.map[tree.root].children, vec![second]);
    assert_eq!(tree.rows, vec![tree.root, second]);
    assert_eq!(tree.row_of[second], 1);
    assert!(!tree.map.contains_key(first));
    assert!(!tree.row_of.contains_key(first));
}

#[test]
fn remove_drops_branch_descendants() {
    let mut tree = ActionTree::default();
    let action = tree
        .append_new_action(&Test::DEFINITION, tree.root)
        .unwrap();
    let branches = tree.map[action].children.clone();

    tree.remove(action).unwrap();

    assert_eq!(tree.rows, vec![tree.root]);
    assert!(tree.map[tree.root].children.is_empty());
    assert!(!tree.map.contains_key(action));
    for branch in branches {
        assert!(!tree.map.contains_key(branch));
        assert!(!tree.row_of.contains_key(branch));
    }
}

#[test]
fn remove_rejects_static_nodes() {
    let mut tree = ActionTree::default();
    assert!(tree.remove(tree.root).is_err());

    let action = tree
        .append_new_action(&Test::DEFINITION, tree.root)
        .unwrap();
    let branch = tree.map[action].children[0];
    assert!(tree.remove(branch).is_err());
}

#[test]
fn remove_errors_on_unknown_id() {
    let mut tree = ActionTree::default();
    let stale = tree
        .append_new_action(&Click::DEFINITION, tree.root)
        .unwrap();
    tree.remove(stale).unwrap();
    assert!(tree.remove(stale).is_err());
}

#[test]
fn action_parameter_reads_default_scriptable_value() {
    let mut tree = ActionTree::default();
    let action = tree
        .append_new_action(&MessageBox::DEFINITION, tree.root)
        .unwrap();

    let title = tree.action_parameter(action, "title").unwrap();
    assert_eq!(title["mode"], "static");
    assert_eq!(title["value"], "");
}

#[test]
fn set_action_parameter_round_trips_static_and_script_modes() {
    let mut tree = ActionTree::default();
    let action = tree
        .append_new_action(&MessageBox::DEFINITION, tree.root)
        .unwrap();

    tree.set_action_parameter(
        action,
        "title",
        serde_json::json!({ "mode": "static", "value": "Hello" }),
    )
    .unwrap();
    assert_eq!(
        tree.action_parameter(action, "title").unwrap()["value"],
        "Hello"
    );

    tree.set_action_parameter(
        action,
        "title",
        serde_json::json!({ "mode": "script", "source": "user.name" }),
    )
    .unwrap();
    let title = tree.action_parameter(action, "title").unwrap();
    assert_eq!(title["mode"], "script");
    assert_eq!(title["source"], "user.name");
    // Other parameters are untouched by the patch.
    assert_eq!(tree.action_parameter(action, "buttons").unwrap(), "ok");
}

#[test]
fn set_action_parameter_round_trips_numeric_static_values() {
    let mut tree = ActionTree::default();
    let test = tree
        .append_new_action(&Test::DEFINITION, tree.root)
        .unwrap();
    let click = tree
        .append_new_action(&Click::DEFINITION, tree.root)
        .unwrap();

    tree.set_action_parameter(
        test,
        "percent",
        serde_json::json!({ "mode": "static", "value": 75 }),
    )
    .unwrap();
    assert_eq!(tree.action_parameter(test, "percent").unwrap()["value"], 75,);

    tree.set_action_parameter(
        click,
        "position",
        serde_json::json!({ "mode": "static", "value": { "x": 12, "y": 34 } }),
    )
    .unwrap();
    let position = tree.action_parameter(click, "position").unwrap();
    assert_eq!(position["value"]["x"], 12);
    assert_eq!(position["value"]["y"], 34);
}

#[test]
fn action_parameter_errors_on_non_action_and_unknown_parameter() {
    let mut tree = ActionTree::default();
    let action = tree
        .append_new_action(&MessageBox::DEFINITION, tree.root)
        .unwrap();

    assert!(matches!(
        tree.action_parameter(tree.root, "title"),
        Err(Error::NotAnAction(_)),
    ));
    assert!(matches!(
        tree.action_parameter(action, "nope"),
        Err(Error::UnknownParameter(_)),
    ));
}

#[test]
fn set_action_parameter_errors_on_invalid_value() {
    let mut tree = ActionTree::default();
    let action = tree
        .append_new_action(&MessageBox::DEFINITION, tree.root)
        .unwrap();

    // A bare string is not a valid `Scriptable<String>` encoding.
    assert!(matches!(
        tree.set_action_parameter(action, "title", serde_json::json!("oops")),
        Err(Error::ParameterSerialization(_)),
    ));
}

fn assert_json_key_absent(value: &serde_json::Value, key: &str) {
    match value {
        serde_json::Value::Object(object) => {
            assert!(!object.contains_key(key), "unexpected serialized key {key}");
            for value in object.values() {
                assert_json_key_absent(value, key);
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                assert_json_key_absent(value, key);
            }
        }
        _ => {}
    }
}

fn inject_legacy_depth_fields(value: &mut serde_json::Value, depth: usize) {
    match value {
        serde_json::Value::Object(object) => {
            if let Some(metadata) = object
                .get_mut("metadata")
                .and_then(serde_json::Value::as_object_mut)
            {
                metadata.insert("depth".to_string(), serde_json::json!(depth));
            }
            for value in object.values_mut() {
                inject_legacy_depth_fields(value, depth);
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                inject_legacy_depth_fields(value, depth);
            }
        }
        _ => {}
    }
}
