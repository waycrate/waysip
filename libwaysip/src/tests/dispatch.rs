//! Unit tests for `WaysipState`'s `Dispatch` impls, using "inert" proxy
//! objects (`Proxy::inert`) backed by a locally paired `UnixStream` instead
//! of a real compositor. Nothing here needs a live Wayland server: the
//! backend is a genuine, working client backend, it just has nobody
//! listening on the other end of the socket, so one-way protocol requests
//! (bind, commit, attach, ...) succeed locally without ever needing a reply.
//! This mirrors the technique used by `libwayshot`'s own dispatch tests.
//!
//! Tests that need a real compositor round-trip (registry/xdg_output info
//! actually being filled in by a server) live in `live_connection.rs`
//! instead.

use std::os::unix::net::UnixStream;

use wayland_backend::client::Backend;
use wayland_client::protocol::{
    wl_buffer::{self, WlBuffer},
    wl_callback::{self, WlCallback},
    wl_keyboard, wl_output, wl_pointer, wl_registry, wl_seat,
    wl_shm::WlShm,
};
use wayland_client::{Connection, Dispatch, Proxy, WEnum};
use wayland_protocols::wp::cursor_shape::v1::client::wp_cursor_shape_manager_v1::WpCursorShapeManagerV1;
use wayland_protocols::xdg::shell::client::xdg_wm_base;
use wayland_protocols::xdg::xdg_output::zv1::client::zxdg_output_v1;
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1;

use crate::state::{self, Corner, DragTarget, SelectionType, WaysipState};
use crate::{BoxInfo, Position, Size, Style};

fn dummy_conn() -> Connection {
    let (client, server) = UnixStream::pair().expect("unix stream");
    Box::leak(Box::new(server));
    let backend = Backend::connect(client).expect("backend");
    Connection::from_backend(backend)
}

fn inert<T: Proxy>(conn: &Connection) -> T {
    T::inert(conn.backend().downgrade())
}

fn base_state() -> WaysipState {
    WaysipState::new(SelectionType::Area)
}

fn cairo_context() -> cairo::Context {
    let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 4, 4).unwrap();
    cairo::Context::new(&surface).unwrap()
}

fn layer_surface_info(conn: &Connection) -> state::LayerSurfaceInfo {
    state::LayerSurfaceInfo {
        layer: inert(conn),
        wl_surface: inert(conn),
        cursor_surface: inert(conn),
        buffer: inert(conn),
        cursor_buffer: None,
        cairo_t: cairo_context(),
        stride: 16,
        inited: false,
        buffer_busy: false,
        style: Style::default(),
        pango_layout: std::cell::OnceCell::new(),
        font_desc_bold: std::cell::OnceCell::new(),
        font_desc_normal: std::cell::OnceCell::new(),
        prev_selection: None,
        margin: std::cell::OnceCell::new(),
        frozen_bg: None,
    }
}

fn output_with_xdg_info(
    conn: &Connection,
    start: (i32, i32),
    size: (i32, i32),
) -> state::WlOutputInfo {
    let wl_output: wl_output::WlOutput = inert(conn);
    let zxdg_output: zxdg_output_v1::ZxdgOutputV1 = inert(conn);
    let info = state::WlOutputInfo::new(wl_output);
    let mut xdg_info = state::ZXdgOutputInfo::new(zxdg_output);
    xdg_info.start_position = Position {
        x: start.0,
        y: start.1,
    };
    xdg_info.size = Size {
        width: size.0,
        height: size.1,
    };
    info.xdg_output_info.set(xdg_info).unwrap();
    info
}

// --- wl_keyboard ---

#[test]
fn keyboard_escape_aborts_selection() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let keyboard: wl_keyboard::WlKeyboard = inert(&conn);
    let mut state = base_state();
    state.start_pos = Some(Position { x: 1.0, y: 1.0 });
    state.end_pos = Some(Position { x: 2.0, y: 2.0 });

    <WaysipState as Dispatch<wl_keyboard::WlKeyboard, ()>>::event(
        &mut state,
        &keyboard,
        wl_keyboard::Event::Key {
            serial: 0,
            time: 0,
            key: 1,
            state: WEnum::Value(wl_keyboard::KeyState::Pressed),
        },
        &(),
        &conn,
        &qh,
    );

    assert!(state.start_pos.is_none());
    assert!(state.end_pos.is_none());
    assert!(!state.running);
}

#[test]
fn keyboard_escape_release_is_ignored() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let keyboard: wl_keyboard::WlKeyboard = inert(&conn);
    let mut state = base_state();

    <WaysipState as Dispatch<wl_keyboard::WlKeyboard, ()>>::event(
        &mut state,
        &keyboard,
        wl_keyboard::Event::Key {
            serial: 0,
            time: 0,
            key: 1,
            state: WEnum::Value(wl_keyboard::KeyState::Released),
        },
        &(),
        &conn,
        &qh,
    );

    assert!(state.running);
}

#[test]
fn keyboard_confirm_key_while_editing_stops_running() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let keyboard: wl_keyboard::WlKeyboard = inert(&conn);
    let mut state = base_state();
    state.editing = true;
    state.confirm_key = 28;

    <WaysipState as Dispatch<wl_keyboard::WlKeyboard, ()>>::event(
        &mut state,
        &keyboard,
        wl_keyboard::Event::Key {
            serial: 0,
            time: 0,
            key: 28,
            state: WEnum::Value(wl_keyboard::KeyState::Pressed),
        },
        &(),
        &conn,
        &qh,
    );

    assert!(!state.running);
}

#[test]
fn keyboard_confirm_key_while_not_editing_is_ignored() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let keyboard: wl_keyboard::WlKeyboard = inert(&conn);
    let mut state = base_state();
    state.confirm_key = 28;

    <WaysipState as Dispatch<wl_keyboard::WlKeyboard, ()>>::event(
        &mut state,
        &keyboard,
        wl_keyboard::Event::Key {
            serial: 0,
            time: 0,
            key: 28,
            state: WEnum::Value(wl_keyboard::KeyState::Pressed),
        },
        &(),
        &conn,
        &qh,
    );

    assert!(state.running);
}

#[test]
fn keyboard_unrelated_key_is_ignored() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let keyboard: wl_keyboard::WlKeyboard = inert(&conn);
    let mut state = base_state();

    <WaysipState as Dispatch<wl_keyboard::WlKeyboard, ()>>::event(
        &mut state,
        &keyboard,
        wl_keyboard::Event::Key {
            serial: 0,
            time: 0,
            key: 99,
            state: WEnum::Value(wl_keyboard::KeyState::Pressed),
        },
        &(),
        &conn,
        &qh,
    );

    assert!(state.running);
}

// --- wl_registry ---

#[test]
fn registry_global_wl_output_registers_output() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let registry: wl_registry::WlRegistry = inert(&conn);
    let mut state = base_state();

    <WaysipState as Dispatch<wl_registry::WlRegistry, ()>>::event(
        &mut state,
        &registry,
        wl_registry::Event::Global {
            name: 1,
            interface: "wl_output".to_string(),
            version: 4,
        },
        &(),
        &conn,
        &qh,
    );

    assert_eq!(state.wloutput_infos.len(), 1);
}

#[test]
fn registry_global_other_interface_is_ignored() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let registry: wl_registry::WlRegistry = inert(&conn);
    let mut state = base_state();

    <WaysipState as Dispatch<wl_registry::WlRegistry, ()>>::event(
        &mut state,
        &registry,
        wl_registry::Event::Global {
            name: 1,
            interface: "wl_compositor".to_string(),
            version: 4,
        },
        &(),
        &conn,
        &qh,
    );

    assert!(state.wloutput_infos.is_empty());
}

#[test]
fn registry_global_remove_is_ignored() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let registry: wl_registry::WlRegistry = inert(&conn);
    let mut state = base_state();

    <WaysipState as Dispatch<wl_registry::WlRegistry, ()>>::event(
        &mut state,
        &registry,
        wl_registry::Event::GlobalRemove { name: 1 },
        &(),
        &conn,
        &qh,
    );

    assert!(state.wloutput_infos.is_empty());
}

// --- wl_output ---

#[test]
fn wl_output_name_event_sets_name() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let wl_output: wl_output::WlOutput = inert(&conn);
    let mut state = base_state();
    state
        .wloutput_infos
        .push(state::WlOutputInfo::new(wl_output.clone()));

    <WaysipState as Dispatch<wl_output::WlOutput, ()>>::event(
        &mut state,
        &wl_output,
        wl_output::Event::Name {
            name: "DP-1".to_string(),
        },
        &(),
        &conn,
        &qh,
    );

    assert_eq!(state.wloutput_infos[0].name, "DP-1");
}

#[test]
fn wl_output_mode_event_sets_size() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let wl_output: wl_output::WlOutput = inert(&conn);
    let mut state = base_state();
    state
        .wloutput_infos
        .push(state::WlOutputInfo::new(wl_output.clone()));

    <WaysipState as Dispatch<wl_output::WlOutput, ()>>::event(
        &mut state,
        &wl_output,
        wl_output::Event::Mode {
            flags: WEnum::Value(wl_output::Mode::Current),
            width: 1920,
            height: 1080,
            refresh: 60000,
        },
        &(),
        &conn,
        &qh,
    );

    assert_eq!(state.wloutput_infos[0].size.width, 1920);
    assert_eq!(state.wloutput_infos[0].size.height, 1080);
}

// --- zxdg_output_v1 ---

#[test]
fn zxdg_output_updates_matching_output() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let wl_output: wl_output::WlOutput = inert(&conn);
    let zxdg_output: zxdg_output_v1::ZxdgOutputV1 = inert(&conn);
    let output_info = state::WlOutputInfo::new(wl_output);
    output_info
        .xdg_output_info
        .set(state::ZXdgOutputInfo::new(zxdg_output.clone()))
        .unwrap();
    let mut state = base_state();
    state.wloutput_infos.push(output_info);

    <WaysipState as Dispatch<zxdg_output_v1::ZxdgOutputV1, ()>>::event(
        &mut state,
        &zxdg_output,
        zxdg_output_v1::Event::LogicalSize {
            width: 1920,
            height: 1080,
        },
        &(),
        &conn,
        &qh,
    );
    <WaysipState as Dispatch<zxdg_output_v1::ZxdgOutputV1, ()>>::event(
        &mut state,
        &zxdg_output,
        zxdg_output_v1::Event::LogicalPosition { x: 10, y: 20 },
        &(),
        &conn,
        &qh,
    );
    <WaysipState as Dispatch<zxdg_output_v1::ZxdgOutputV1, ()>>::event(
        &mut state,
        &zxdg_output,
        zxdg_output_v1::Event::Name {
            name: "DP-1".to_string(),
        },
        &(),
        &conn,
        &qh,
    );

    let info = state.wloutput_infos[0].xdg_output_info();
    assert_eq!(info.size.width, 1920);
    assert_eq!(info.size.height, 1080);
    assert_eq!(info.start_position.x, 10);
    assert_eq!(info.start_position.y, 20);
    assert_eq!(info.name, "DP-1");
}

#[test]
fn zxdg_output_event_for_unmatched_proxy_is_ignored() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let zxdg_output: zxdg_output_v1::ZxdgOutputV1 = inert(&conn);
    let mut state = base_state();

    <WaysipState as Dispatch<zxdg_output_v1::ZxdgOutputV1, ()>>::event(
        &mut state,
        &zxdg_output,
        zxdg_output_v1::Event::LogicalSize {
            width: 1920,
            height: 1080,
        },
        &(),
        &conn,
        &qh,
    );

    assert!(state.wloutput_infos.is_empty());
}

// --- xdg_wm_base ---

#[test]
fn xdg_wm_base_ping_does_not_panic() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let wm_base: xdg_wm_base::XdgWmBase = inert(&conn);
    let mut state = base_state();

    <WaysipState as Dispatch<xdg_wm_base::XdgWmBase, ()>>::event(
        &mut state,
        &wm_base,
        xdg_wm_base::Event::Ping { serial: 7 },
        &(),
        &conn,
        &qh,
    );
}

// --- wl_seat ---

#[test]
fn seat_capabilities_pointer_and_keyboard_do_not_panic() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let seat: wl_seat::WlSeat = inert(&conn);
    let mut state = base_state();

    <WaysipState as Dispatch<wl_seat::WlSeat, ()>>::event(
        &mut state,
        &seat,
        wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(wl_seat::Capability::Keyboard),
        },
        &(),
        &conn,
        &qh,
    );
    <WaysipState as Dispatch<wl_seat::WlSeat, ()>>::event(
        &mut state,
        &seat,
        wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(wl_seat::Capability::Pointer),
        },
        &(),
        &conn,
        &qh,
    );
}

// --- wl_pointer: Button ---

#[test]
fn pointer_button_press_sets_start_pos_for_area_selection() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let pointer: wl_pointer::WlPointer = inert(&conn);
    let mut state = base_state();
    state.qh = Some(qh.clone());
    state.current_pos = Position { x: 12.0, y: 34.0 };

    <WaysipState as Dispatch<wl_pointer::WlPointer, ()>>::event(
        &mut state,
        &pointer,
        wl_pointer::Event::Button {
            serial: 5,
            time: 0,
            button: 272,
            state: WEnum::Value(wl_pointer::ButtonState::Pressed),
        },
        &(),
        &conn,
        &qh,
    );

    assert_eq!(state.start_pos.unwrap().x, 12.0);
    assert_eq!(state.start_pos.unwrap().y, 34.0);
    assert!(state.running);
    assert_eq!(state.last_pointer_serial, Some(5));
}

#[test]
fn pointer_button_press_for_point_selection_finishes_immediately() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let pointer: wl_pointer::WlPointer = inert(&conn);
    let mut state = WaysipState::new(SelectionType::Point);
    state.qh = Some(qh.clone());
    state.current_pos = Position { x: 5.0, y: 6.0 };

    <WaysipState as Dispatch<wl_pointer::WlPointer, ()>>::event(
        &mut state,
        &pointer,
        wl_pointer::Event::Button {
            serial: 1,
            time: 0,
            button: 272,
            state: WEnum::Value(wl_pointer::ButtonState::Pressed),
        },
        &(),
        &conn,
        &qh,
    );

    assert_eq!(state.end_pos.unwrap().x, 5.0);
    assert!(!state.running);
}

#[test]
fn pointer_button_release_for_area_finishes_selection() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let pointer: wl_pointer::WlPointer = inert(&conn);
    let mut state = base_state();
    state.qh = Some(qh.clone());
    state.start_pos = Some(Position { x: 1.0, y: 1.0 });
    state.current_pos = Position { x: 50.0, y: 60.0 };

    <WaysipState as Dispatch<wl_pointer::WlPointer, ()>>::event(
        &mut state,
        &pointer,
        wl_pointer::Event::Button {
            serial: 2,
            time: 0,
            button: 272,
            state: WEnum::Value(wl_pointer::ButtonState::Released),
        },
        &(),
        &conn,
        &qh,
    );

    assert_eq!(state.end_pos.unwrap().x, 50.0);
    assert!(!state.running);
}

#[test]
fn pointer_button_release_with_edit_enabled_starts_editing() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let pointer: wl_pointer::WlPointer = inert(&conn);
    let mut state = base_state();
    state.qh = Some(qh.clone());
    state.edit_enabled = true;
    state.start_pos = Some(Position { x: 1.0, y: 1.0 });
    state.current_pos = Position { x: 50.0, y: 60.0 };

    <WaysipState as Dispatch<wl_pointer::WlPointer, ()>>::event(
        &mut state,
        &pointer,
        wl_pointer::Event::Button {
            serial: 2,
            time: 0,
            button: 272,
            state: WEnum::Value(wl_pointer::ButtonState::Released),
        },
        &(),
        &conn,
        &qh,
    );

    assert!(state.editing);
    assert!(state.running);
}

#[test]
fn pointer_button_press_while_editing_selects_handle() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let pointer: wl_pointer::WlPointer = inert(&conn);
    let mut state = base_state();
    state.qh = Some(qh.clone());
    state.editing = true;
    state.start_pos = Some(Position { x: 0.0, y: 0.0 });
    state.end_pos = Some(Position { x: 100.0, y: 100.0 });
    state.current_pos = Position { x: 2.0, y: 2.0 };

    <WaysipState as Dispatch<wl_pointer::WlPointer, ()>>::event(
        &mut state,
        &pointer,
        wl_pointer::Event::Button {
            serial: 3,
            time: 0,
            button: 272,
            state: WEnum::Value(wl_pointer::ButtonState::Pressed),
        },
        &(),
        &conn,
        &qh,
    );

    assert_eq!(state.active_handle, Some(DragTarget::Corner(Corner::Start)));
}

#[test]
fn pointer_button_release_while_editing_clears_active_handle() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let pointer: wl_pointer::WlPointer = inert(&conn);
    let mut state = base_state();
    state.qh = Some(qh.clone());
    state.editing = true;
    state.active_handle = Some(DragTarget::Body);
    state.move_anchor = Some(state::MoveAnchor {
        grab_pos: Position { x: 0.0, y: 0.0 },
        start_pos: Position { x: 0.0, y: 0.0 },
        end_pos: Position { x: 0.0, y: 0.0 },
    });

    <WaysipState as Dispatch<wl_pointer::WlPointer, ()>>::event(
        &mut state,
        &pointer,
        wl_pointer::Event::Button {
            serial: 4,
            time: 0,
            button: 272,
            state: WEnum::Value(wl_pointer::ButtonState::Released),
        },
        &(),
        &conn,
        &qh,
    );

    assert!(state.active_handle.is_none());
    assert!(state.move_anchor.is_none());
}

// --- wl_pointer: Enter ---

#[test]
fn pointer_enter_sets_current_screen_and_pos() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let pointer: wl_pointer::WlPointer = inert(&conn);
    let mut state = base_state();
    state.qh = Some(qh.clone());

    let output_info = output_with_xdg_info(&conn, (100, 200), (1920, 1080));
    let surface_info = layer_surface_info(&conn);
    let surface_handle = surface_info.wl_surface.clone();
    state.wloutput_infos.push(output_info);
    state.wl_surfaces.push(surface_info);

    <WaysipState as Dispatch<wl_pointer::WlPointer, ()>>::event(
        &mut state,
        &pointer,
        wl_pointer::Event::Enter {
            serial: 9,
            surface: surface_handle,
            surface_x: 5.0,
            surface_y: 7.0,
        },
        &(),
        &conn,
        &qh,
    );

    assert_eq!(state.current_screen, 0);
    assert_eq!(state.current_pos.x, 105.0);
    assert_eq!(state.current_pos.y, 207.0);
    assert_eq!(state.last_pointer_serial, Some(9));
}

#[test]
fn pointer_enter_with_cursor_manager_requests_crosshair_shape() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let pointer: wl_pointer::WlPointer = inert(&conn);
    let mut state = base_state();
    state.qh = Some(qh.clone());
    state.cursor_manager = Some(inert::<WpCursorShapeManagerV1>(&conn));

    let output_info = output_with_xdg_info(&conn, (0, 0), (1920, 1080));
    let surface_info = layer_surface_info(&conn);
    let surface_handle = surface_info.wl_surface.clone();
    state.wloutput_infos.push(output_info);
    state.wl_surfaces.push(surface_info);

    <WaysipState as Dispatch<wl_pointer::WlPointer, ()>>::event(
        &mut state,
        &pointer,
        wl_pointer::Event::Enter {
            serial: 1,
            surface: surface_handle,
            surface_x: 0.0,
            surface_y: 0.0,
        },
        &(),
        &conn,
        &qh,
    );

    assert_eq!(state.cursor_is_crosshair, Some(true));
}

// --- wl_pointer: Motion ---

#[test]
fn pointer_motion_updates_end_pos_for_area_selection() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let pointer: wl_pointer::WlPointer = inert(&conn);
    let mut state = base_state();
    state.qh = Some(qh.clone());
    state
        .wloutput_infos
        .push(output_with_xdg_info(&conn, (0, 0), (1920, 1080)));
    state.start_pos = Some(Position { x: 10.0, y: 10.0 });

    <WaysipState as Dispatch<wl_pointer::WlPointer, ()>>::event(
        &mut state,
        &pointer,
        wl_pointer::Event::Motion {
            time: 0,
            surface_x: 40.0,
            surface_y: 50.0,
        },
        &(),
        &conn,
        &qh,
    );

    assert_eq!(state.current_pos.x, 40.0);
    assert_eq!(state.end_pos.unwrap().x, 40.0);
    assert_eq!(state.end_pos.unwrap().y, 50.0);
}

#[test]
fn pointer_motion_respects_aspect_ratio_height_driven() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let pointer: wl_pointer::WlPointer = inert(&conn);
    let mut state = base_state();
    state.qh = Some(qh.clone());
    state
        .wloutput_infos
        .push(output_with_xdg_info(&conn, (0, 0), (1920, 1080)));
    state.aspect_ratio = Some((16.0, 9.0));
    state.start_pos = Some(Position { x: 0.0, y: 0.0 });

    // width=50, height=200: too tall for 16:9, so height drives the box.
    <WaysipState as Dispatch<wl_pointer::WlPointer, ()>>::event(
        &mut state,
        &pointer,
        wl_pointer::Event::Motion {
            time: 0,
            surface_x: 50.0,
            surface_y: 200.0,
        },
        &(),
        &conn,
        &qh,
    );

    let end = state.end_pos.unwrap();
    assert!((end.x - 200.0 * 16.0 / 9.0).abs() < 0.001);
    assert_eq!(end.y, 200.0);
}

#[test]
fn pointer_motion_respects_aspect_ratio_width_driven() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let pointer: wl_pointer::WlPointer = inert(&conn);
    let mut state = base_state();
    state.qh = Some(qh.clone());
    state
        .wloutput_infos
        .push(output_with_xdg_info(&conn, (0, 0), (1920, 1080)));
    state.aspect_ratio = Some((16.0, 9.0));
    state.start_pos = Some(Position { x: 0.0, y: 0.0 });

    // width=200, height=50: too wide for 16:9, so width drives the box.
    <WaysipState as Dispatch<wl_pointer::WlPointer, ()>>::event(
        &mut state,
        &pointer,
        wl_pointer::Event::Motion {
            time: 0,
            surface_x: 200.0,
            surface_y: 50.0,
        },
        &(),
        &conn,
        &qh,
    );

    let end = state.end_pos.unwrap();
    assert_eq!(end.x, 200.0);
    assert!((end.y - 200.0 * 9.0 / 16.0).abs() < 0.001);
}

#[test]
fn pointer_motion_over_predefined_box_snaps_selection() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let pointer: wl_pointer::WlPointer = inert(&conn);
    let mut state = WaysipState::new(SelectionType::PredefinedBoxes);
    state.qh = Some(qh.clone());
    state
        .wloutput_infos
        .push(output_with_xdg_info(&conn, (0, 0), (1920, 1080)));
    state.predefined_boxes = Some(vec![BoxInfo {
        start_x: 10.0,
        start_y: 10.0,
        end_x: 50.0,
        end_y: 50.0,
    }]);

    <WaysipState as Dispatch<wl_pointer::WlPointer, ()>>::event(
        &mut state,
        &pointer,
        wl_pointer::Event::Motion {
            time: 0,
            surface_x: 20.0,
            surface_y: 20.0,
        },
        &(),
        &conn,
        &qh,
    );

    assert_eq!(state.start_pos.unwrap().x, 10.0);
    assert_eq!(state.end_pos.unwrap().x, 50.0);
}

#[test]
fn pointer_motion_outside_any_predefined_box_leaves_selection_unset() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let pointer: wl_pointer::WlPointer = inert(&conn);
    let mut state = WaysipState::new(SelectionType::PredefinedBoxes);
    state.qh = Some(qh.clone());
    state
        .wloutput_infos
        .push(output_with_xdg_info(&conn, (0, 0), (1920, 1080)));
    state.predefined_boxes = Some(vec![BoxInfo {
        start_x: 10.0,
        start_y: 10.0,
        end_x: 50.0,
        end_y: 50.0,
    }]);

    <WaysipState as Dispatch<wl_pointer::WlPointer, ()>>::event(
        &mut state,
        &pointer,
        wl_pointer::Event::Motion {
            time: 0,
            surface_x: 500.0,
            surface_y: 500.0,
        },
        &(),
        &conn,
        &qh,
    );

    assert!(state.start_pos.is_none());
}

// --- WlCallback (frame done) ---

#[test]
fn frame_callback_for_current_screen_triggers_redraw() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let callback: WlCallback = inert(&conn);
    let mut state = base_state();
    state.redraw_all = true;

    <WaysipState as Dispatch<WlCallback, usize>>::event(
        &mut state,
        &callback,
        wl_callback::Event::Done { callback_data: 123 },
        &0,
        &conn,
        &qh,
    );

    assert!(!state.redraw_all);
}

#[test]
fn frame_callback_for_other_screen_is_ignored() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let callback: WlCallback = inert(&conn);
    let mut state = base_state();
    state.current_screen = 0;
    state.redraw_all = true;

    <WaysipState as Dispatch<WlCallback, usize>>::event(
        &mut state,
        &callback,
        wl_callback::Event::Done { callback_data: 123 },
        &1,
        &conn,
        &qh,
    );

    assert!(state.redraw_all);
}

// --- WlBuffer ---

#[test]
fn buffer_release_clears_busy_flag() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let buffer: WlBuffer = inert(&conn);
    let mut state = base_state();
    let mut surface_info = layer_surface_info(&conn);
    surface_info.buffer = buffer.clone();
    surface_info.buffer_busy = true;
    state.wl_surfaces.push(surface_info);

    <WaysipState as Dispatch<WlBuffer, ()>>::event(
        &mut state,
        &buffer,
        wl_buffer::Event::Release,
        &(),
        &conn,
        &qh,
    );

    assert!(!state.wl_surfaces[0].buffer_busy);
}

// --- zwlr_layer_surface_v1 ---

#[test]
fn layer_surface_configure_creates_buffer_and_marks_inited() {
    let conn = dummy_conn();
    let qh = conn.new_event_queue::<WaysipState>().handle();
    let layer: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1 = inert(&conn);
    let shm: WlShm = inert(&conn);

    let mut state = base_state();
    state.shm = Some(shm);
    state.qh = Some(qh.clone());

    let mut surface_info = layer_surface_info(&conn);
    surface_info.layer = layer.clone();
    surface_info.buffer_busy = false;
    surface_info.inited = false;
    state.wl_surfaces.push(surface_info);

    <WaysipState as Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()>>::event(
        &mut state,
        &layer,
        zwlr_layer_surface_v1::Event::Configure {
            serial: 4,
            width: 100,
            height: 100,
        },
        &(),
        &conn,
        &qh,
    );

    assert!(state.wl_surfaces[0].buffer_busy);
    assert!(state.wl_surfaces[0].inited);
}
