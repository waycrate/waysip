//! Tests that connect to a *real* Wayland compositor over the actual
//! protocol, as opposed to the "inert object" tests in `dispatch.rs` that
//! fake a single dead proxy without a server behind it.
//!
//! There's no way to exercise a real registry/xdg_output round trip without
//! an actual compositor process answering on the other end of the socket.
//! CI starts one (wlroots' headless backend, see
//! `.github/workflows/test-coverage.yml`) and points `WAYLAND_DISPLAY` at
//! it before running tests. Locally, or in any other CI job, there's no
//! compositor, so these skip themselves at runtime instead of failing -
//! `cargo test` must stay green on a plain developer machine.
//!
//! `get_area_inner`'s own blocking event loop isn't exercised here: it
//! waits for real pointer/keyboard input, which the headless CI compositor
//! (no input devices, see `WLR_LIBINPUT_NO_DEVICES=1`) never sends. Instead
//! these replicate just its setup (registry + xdg_output round trip) to
//! prove that part works against a real server.

use wayland_client::Connection;
use wayland_client::globals::registry_queue_init;
use wayland_client::protocol::{wl_compositor::WlCompositor, wl_seat::WlSeat, wl_shm::WlShm};
use wayland_protocols::xdg::xdg_output::zv1::client::zxdg_output_manager_v1::ZxdgOutputManagerV1;
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::ZwlrLayerShellV1;

use crate::Position;
use crate::state::{self, SelectionType, WaysipState};

pub(super) fn skip_without_compositor() -> bool {
    if std::env::var_os("WAYLAND_DISPLAY").is_none() {
        eprintln!("skipping: no WAYLAND_DISPLAY set (requires a live compositor)");
        return true;
    }
    false
}

/// Connects and does the registry + xdg_output round trip that
/// `get_area_inner` does, without its blocking event loop.
fn connected_state_with_outputs() -> (Connection, WaysipState) {
    let connection = Connection::connect_to_env().expect("should connect to the CI compositor");
    let (globals, _) = registry_queue_init::<WaysipState>(&connection)
        .expect("registry init should succeed against a live compositor");
    let mut state = WaysipState::new(SelectionType::Area);
    let mut event_queue = connection.new_event_queue::<WaysipState>();
    let qh = event_queue.handle();

    let _ = connection.display().get_registry(&qh, ());
    event_queue
        .roundtrip(&mut state)
        .expect("first roundtrip should populate outputs");

    let xdg_output_manager = globals
        .bind::<ZxdgOutputManagerV1, _, _>(&qh, 1..=3, ())
        .expect("compositor should support xdg-output");
    for wloutput in state.wloutput_infos.iter_mut() {
        let zwloutput = xdg_output_manager.get_xdg_output(wloutput.get_output(), &qh, ());
        wloutput
            .xdg_output_info
            .set(state::ZXdgOutputInfo::new(zwloutput))
            .expect("should be set only once");
    }
    event_queue
        .roundtrip(&mut state)
        .expect("second roundtrip should populate xdg_output info");

    (connection, state)
}

#[test]
fn connects_and_registers_at_least_one_output() {
    if skip_without_compositor() {
        return;
    }
    let (_connection, state) = connected_state_with_outputs();
    assert!(
        !state.wloutput_infos.is_empty(),
        "expected at least one output (WLR_HEADLESS_OUTPUTS should create one)"
    );
}

#[test]
fn xdg_output_info_is_populated_after_roundtrip() {
    if skip_without_compositor() {
        return;
    }
    let (_connection, state) = connected_state_with_outputs();
    let output = &state.wloutput_infos[0];
    let info = output.xdg_output_info();
    assert!(info.size.width > 0);
    assert!(info.size.height > 0);
}

#[test]
fn required_globals_are_advertised_by_the_compositor() {
    if skip_without_compositor() {
        return;
    }
    let connection = Connection::connect_to_env().expect("should connect to the CI compositor");
    let (globals, _) = registry_queue_init::<WaysipState>(&connection)
        .expect("registry init should succeed against a live compositor");
    let event_queue = connection.new_event_queue::<WaysipState>();
    let qh = event_queue.handle();

    assert!(
        globals.bind::<WlCompositor, _, _>(&qh, 1..=5, ()).is_ok(),
        "wl_compositor should be advertised"
    );
    assert!(
        globals.bind::<WlShm, _, _>(&qh, 1..=1, ()).is_ok(),
        "wl_shm should be advertised"
    );
    assert!(
        globals.bind::<WlSeat, _, _>(&qh, 1..=1, ()).is_ok(),
        "wl_seat should be advertised"
    );
    assert!(
        globals
            .bind::<ZwlrLayerShellV1, _, _>(&qh, 3..=4, ())
            .is_ok(),
        "zwlr_layer_shell_v1 should be advertised"
    );
}

#[test]
fn area_info_computes_correct_geometry_from_real_output() {
    if skip_without_compositor() {
        return;
    }
    let (_connection, mut state) = connected_state_with_outputs();
    state.start_pos = Some(Position { x: 10.0, y: 20.0 });
    state.end_pos = Some(Position { x: 110.0, y: 170.0 });

    let area = state.area_info().expect("both positions are set");
    assert_eq!(area.width(), 100);
    assert_eq!(area.height(), 150);
    let top_left = area.left_top_point();
    assert_eq!(top_left.x, 10);
    assert_eq!(top_left.y, 20);
}
