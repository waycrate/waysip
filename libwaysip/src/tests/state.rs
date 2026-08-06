use crate::Position;
use crate::error::BoxInfoError;
use crate::state::*;

// --- BoxInfo ---

#[test]
fn box_from_str_valid() {
    let b = BoxInfo::get_box_from_str("10,20 100x50").unwrap();
    assert_eq!(b.start_x, 10.0);
    assert_eq!(b.start_y, 20.0);
    assert_eq!(b.end_x, 110.0);
    assert_eq!(b.end_y, 70.0);
}

#[test]
fn box_from_str_missing_space() {
    let err = BoxInfo::get_box_from_str("10,20100x50").unwrap_err();
    assert!(matches!(err, BoxInfoError::InvalidBoxString(_)));
}

#[test]
fn box_from_str_missing_comma() {
    let err = BoxInfo::get_box_from_str("1020 100x50").unwrap_err();
    assert!(matches!(err, BoxInfoError::InvalidBoxCoordsString(_)));
}

#[test]
fn box_from_str_missing_x_in_size() {
    let err = BoxInfo::get_box_from_str("10,20 10050").unwrap_err();
    assert!(matches!(err, BoxInfoError::InvalidBoxSizeString(_)));
}

#[test]
fn box_from_str_bad_number() {
    let err = BoxInfo::get_box_from_str("a,20 100x50").unwrap_err();
    assert!(matches!(err, BoxInfoError::ParseFloatError(_)));
}

// --- WaysipState selection-type predicates ---

#[test]
fn selection_type_default_is_area() {
    assert!(matches!(SelectionType::default(), SelectionType::Area));
}

#[test]
fn state_predicates() {
    let state = WaysipState::new(SelectionType::Area);
    assert!(state.is_area());
    assert!(!state.is_screen());
    assert!(!state.is_predefined_boxes());
    assert!(!state.is_dimensions_or_output());

    let state = WaysipState::new(SelectionType::Screen);
    assert!(state.is_screen());

    let state = WaysipState::new(SelectionType::PredefinedBoxes);
    assert!(state.is_predefined_boxes());

    let state = WaysipState::new(SelectionType::DimensionsOrOutput);
    assert!(state.is_dimensions_or_output());
}

#[test]
fn effective_selection_type_falls_back() {
    let state = WaysipState::new(SelectionType::Screen);
    assert!(matches!(
        state.effective_selection_type(),
        SelectionType::Screen
    ));
    assert!(state.is_effective_screen());
    assert!(!state.is_effective_area());
}

#[test]
fn effective_selection_type_overridden() {
    let mut state = WaysipState::new(SelectionType::DimensionsOrOutput);
    state.effective_selection_type = Some(SelectionType::Area);
    assert!(state.is_effective_area());
    assert!(!state.is_effective_screen());
}

// --- corners / hit testing ---

fn state_with_rect() -> WaysipState {
    let mut state = WaysipState::new(SelectionType::Area);
    state.start_pos = Some(Position { x: 0.0, y: 0.0 });
    state.end_pos = Some(Position { x: 100.0, y: 100.0 });
    state
}

#[test]
fn corners_none_without_positions() {
    let state = WaysipState::new(SelectionType::Area);
    assert!(state.corners().is_none());
}

#[test]
fn corners_returns_four_points() {
    let state = state_with_rect();
    let corners = state.corners().unwrap();
    assert_eq!(corners.len(), 4);
    assert!(
        corners
            .iter()
            .any(|(c, p)| *c == Corner::Start && p.x == 0.0 && p.y == 0.0)
    );
    assert!(
        corners
            .iter()
            .any(|(c, p)| *c == Corner::End && p.x == 100.0 && p.y == 100.0)
    );
    assert!(
        corners
            .iter()
            .any(|(c, p)| *c == Corner::EndXStartY && p.x == 100.0 && p.y == 0.0)
    );
    assert!(
        corners
            .iter()
            .any(|(c, p)| *c == Corner::StartXEndY && p.x == 0.0 && p.y == 100.0)
    );
}

#[test]
fn hit_test_handle_near_corner() {
    let state = state_with_rect();
    let hit = state.hit_test_handle(Position { x: 2.0, y: 2.0 });
    assert_eq!(hit, Some(Corner::Start));
}

#[test]
fn hit_test_handle_picks_closest() {
    let state = state_with_rect();
    // closer to Start (0,0) than to any other corner
    let hit = state.hit_test_handle(Position { x: 1.0, y: 5.0 });
    assert_eq!(hit, Some(Corner::Start));
}

#[test]
fn hit_test_handle_far_from_all_corners() {
    let state = state_with_rect();
    assert_eq!(state.hit_test_handle(Position { x: 50.0, y: 50.0 }), None);
}

#[test]
fn hit_test_body_when_inside() {
    let state = state_with_rect();
    assert_eq!(
        state.hit_test(Position { x: 50.0, y: 50.0 }),
        Some(DragTarget::Body)
    );
}

#[test]
fn hit_test_corner_takes_priority_over_body() {
    let state = state_with_rect();
    assert_eq!(
        state.hit_test(Position { x: 1.0, y: 1.0 }),
        Some(DragTarget::Corner(Corner::Start))
    );
}

#[test]
fn hit_test_none_when_outside() {
    let state = state_with_rect();
    assert_eq!(state.hit_test(Position { x: 200.0, y: 200.0 }), None);
}

#[test]
fn hit_test_none_without_positions() {
    let state = WaysipState::new(SelectionType::Area);
    assert_eq!(state.hit_test(Position { x: 1.0, y: 1.0 }), None);
}

// --- dragging ---

#[test]
fn apply_handle_drag_start_corner() {
    let mut state = state_with_rect();
    state.active_handle = Some(DragTarget::Corner(Corner::Start));
    state.current_pos = Position { x: 5.0, y: 5.0 };
    state.apply_handle_drag();
    assert_eq!(state.start_pos.unwrap().x, 5.0);
    assert_eq!(state.start_pos.unwrap().y, 5.0);
    assert_eq!(state.end_pos.unwrap().x, 100.0);
}

#[test]
fn apply_handle_drag_end_corner() {
    let mut state = state_with_rect();
    state.active_handle = Some(DragTarget::Corner(Corner::End));
    state.current_pos = Position { x: 150.0, y: 150.0 };
    state.apply_handle_drag();
    assert_eq!(state.end_pos.unwrap().x, 150.0);
    assert_eq!(state.end_pos.unwrap().y, 150.0);
}

#[test]
fn apply_handle_drag_end_x_start_y_corner() {
    let mut state = state_with_rect();
    state.active_handle = Some(DragTarget::Corner(Corner::EndXStartY));
    state.current_pos = Position { x: 30.0, y: 40.0 };
    state.apply_handle_drag();
    assert_eq!(state.end_pos.unwrap().x, 30.0);
    assert_eq!(state.start_pos.unwrap().y, 40.0);
    assert_eq!(state.start_pos.unwrap().x, 0.0);
    assert_eq!(state.end_pos.unwrap().y, 100.0);
}

#[test]
fn apply_handle_drag_start_x_end_y_corner() {
    let mut state = state_with_rect();
    state.active_handle = Some(DragTarget::Corner(Corner::StartXEndY));
    state.current_pos = Position { x: 30.0, y: 40.0 };
    state.apply_handle_drag();
    assert_eq!(state.start_pos.unwrap().x, 30.0);
    assert_eq!(state.end_pos.unwrap().y, 40.0);
    assert_eq!(state.start_pos.unwrap().y, 0.0);
    assert_eq!(state.end_pos.unwrap().x, 100.0);
}

#[test]
fn apply_handle_drag_body_moves_whole_rect() {
    let mut state = state_with_rect();
    state.current_pos = Position { x: 10.0, y: 10.0 };
    state.begin_move_drag();
    state.active_handle = Some(DragTarget::Body);
    state.current_pos = Position { x: 15.0, y: 25.0 };
    state.apply_handle_drag();
    assert_eq!(state.start_pos.unwrap().x, 5.0);
    assert_eq!(state.start_pos.unwrap().y, 15.0);
    assert_eq!(state.end_pos.unwrap().x, 105.0);
    assert_eq!(state.end_pos.unwrap().y, 115.0);
}

#[test]
fn apply_handle_drag_noop_without_active_handle() {
    let mut state = state_with_rect();
    state.apply_handle_drag();
    assert_eq!(state.start_pos.unwrap().x, 0.0);
    assert_eq!(state.end_pos.unwrap().x, 100.0);
}

#[test]
fn begin_move_drag_noop_without_positions() {
    let mut state = WaysipState::new(SelectionType::Area);
    state.begin_move_drag();
    assert!(state.move_anchor.is_none());
}

// --- editing/confirm flow ---

#[test]
fn finish_or_start_editing_disabled_stops_running() {
    let mut state = WaysipState::new(SelectionType::Area);
    state.finish_or_start_editing();
    assert!(!state.running);
    assert!(!state.editing);
}

#[test]
fn finish_or_start_editing_enabled_for_area_starts_editing() {
    let mut state = WaysipState::new(SelectionType::Area);
    state.edit_enabled = true;
    state.finish_or_start_editing();
    assert!(state.editing);
    assert!(state.running);
}

#[test]
fn finish_or_start_editing_enabled_for_screen_stops_running() {
    let mut state = WaysipState::new(SelectionType::Screen);
    state.edit_enabled = true;
    state.finish_or_start_editing();
    assert!(!state.editing);
    assert!(!state.running);
}

// --- start pos tracking ---

#[test]
fn set_start_pos_marks_redraw_all_once() {
    let mut state = WaysipState::new(SelectionType::Area);
    assert!(!state.redraw_all);
    state.set_start_pos(Position { x: 1.0, y: 2.0 });
    assert!(state.redraw_all);
    assert_eq!(state.start_pos.unwrap().x, 1.0);

    state.redraw_all = false;
    state.set_start_pos(Position { x: 3.0, y: 4.0 });
    assert!(!state.redraw_all);
    assert_eq!(state.start_pos.unwrap().x, 3.0);
}
