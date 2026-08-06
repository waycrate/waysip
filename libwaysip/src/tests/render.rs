use std::os::unix::net::UnixStream;

use wayland_backend::client::Backend;
use wayland_client::{Connection, Proxy};

use crate::render::*;
use crate::state::{self, LayerSurfaceInfo};
use crate::{BoxInfo, Color, Position, Size, Style};

fn sample_color() -> Color {
    Color {
        r: 0.1,
        g: 0.2,
        b: 0.3,
        a: 0.4,
    }
}

fn dummy_conn() -> Connection {
    let (client, server) = UnixStream::pair().expect("unix stream");
    Box::leak(Box::new(server));
    let backend = Backend::connect(client).expect("backend");
    Connection::from_backend(backend)
}

fn inert<T: Proxy>(conn: &Connection) -> T {
    T::inert(conn.backend().downgrade())
}

/// A `LayerSurfaceInfo` backed by an inert (no live compositor needed) set
/// of Wayland proxies and a real, reasonably-sized cairo surface, so the
/// drawing methods below have real pixels to paint into.
fn layer_surface_info(conn: &Connection) -> LayerSurfaceInfo {
    let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 200, 200).unwrap();
    let cairo_t = cairo::Context::new(&surface).unwrap();
    state::LayerSurfaceInfo {
        layer: inert(conn),
        wl_surface: inert(conn),
        cursor_surface: inert(conn),
        buffer: inert(conn),
        cursor_buffer: None,
        cairo_t,
        stride: 200 * 4,
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

#[test]
fn draw_ui_plain_background() {
    let mut file = tempfile::tempfile().unwrap();
    let UiInit { stride, .. } = draw_ui(&mut file, (10, 10), sample_color(), None);
    assert!(stride >= 10 * 4);
}

#[test]
fn draw_ui_with_matching_frozen_background() {
    let mut file = tempfile::tempfile().unwrap();
    let frozen = cairo::ImageSurface::create(cairo::Format::ARgb32, 10, 10).unwrap();
    let UiInit { stride, .. } = draw_ui(&mut file, (10, 10), sample_color(), Some(&frozen));
    assert!(stride >= 10 * 4);
}

#[test]
fn draw_ui_with_mismatched_frozen_background_scales() {
    let mut file = tempfile::tempfile().unwrap();
    let frozen = cairo::ImageSurface::create(cairo::Format::ARgb32, 5, 5).unwrap();
    let UiInit { stride, .. } = draw_ui(&mut file, (20, 20), sample_color(), Some(&frozen));
    assert!(stride >= 20 * 4);
}

// --- LayerSurfaceInfo drawing methods ---
//
// None of these need a live compositor: `wl_surface.attach/damage/commit`
// are one-way protocol requests that succeed fine against an inert proxy
// (see `dispatch.rs` for the same technique), and the actual drawing is
// real cairo/pango work against a real (if disconnected-from-any-screen)
// surface.

#[test]
fn init_commit_does_not_panic() {
    let conn = dummy_conn();
    let info = layer_surface_info(&conn);
    info.init_commit();
}

#[test]
fn redraw_select_screen_when_selected_draws_label() {
    let conn = dummy_conn();
    let info = layer_surface_info(&conn);
    info.redraw_select_screen(
        true,
        Size {
            width: 200,
            height: 200,
        },
        Position { x: 0, y: 0 },
        "DP-1",
        "Some Monitor",
    );
}

#[test]
fn redraw_select_screen_when_not_selected_paints_background_only() {
    let conn = dummy_conn();
    let info = layer_surface_info(&conn);
    info.redraw_select_screen(
        false,
        Size {
            width: 200,
            height: 200,
        },
        Position { x: 0, y: 0 },
        "DP-1",
        "Some Monitor",
    );
}

#[test]
fn redraw_minimal_does_not_panic() {
    let conn = dummy_conn();
    let mut info = layer_surface_info(&conn);
    info.redraw(
        Position { x: 10.0, y: 10.0 },
        Position { x: 60.0, y: 60.0 },
        Position { x: 0, y: 0 },
        Size {
            width: 200,
            height: 200,
        },
        false,
        None,
        true,
        false,
    );
}

#[test]
fn redraw_full_featured_does_not_panic() {
    let conn = dummy_conn();
    let mut info = layer_surface_info(&conn);
    let boxes = vec![BoxInfo {
        start_x: 5.0,
        start_y: 5.0,
        end_x: 30.0,
        end_y: 30.0,
    }];
    info.redraw(
        Position { x: 10.0, y: 10.0 },
        Position { x: 60.0, y: 60.0 },
        Position { x: 0, y: 0 },
        Size {
            width: 200,
            height: 200,
        },
        true,
        Some(&boxes),
        false,
        true,
    );
}

#[test]
fn redraw_with_frozen_background_does_not_panic() {
    let conn = dummy_conn();
    let mut info = layer_surface_info(&conn);
    info.frozen_bg = Some(cairo::ImageSurface::create(cairo::Format::ARgb32, 200, 200).unwrap());
    info.redraw(
        Position { x: 10.0, y: 10.0 },
        Position { x: 60.0, y: 60.0 },
        Position { x: 0, y: 0 },
        Size {
            width: 200,
            height: 200,
        },
        true,
        None,
        true,
        false,
    );
}

#[test]
fn redraw_reuses_prev_selection_clip_on_second_call() {
    let conn = dummy_conn();
    let mut info = layer_surface_info(&conn);
    let size = Size {
        width: 200,
        height: 200,
    };
    // First call has no prev_selection yet (the `else` branch of the
    // clip-rect computation); the second exercises the `Some(prev)` branch.
    info.redraw(
        Position { x: 10.0, y: 10.0 },
        Position { x: 60.0, y: 60.0 },
        Position { x: 0, y: 0 },
        size,
        false,
        None,
        false,
        false,
    );
    assert!(info.prev_selection.is_some());
    info.redraw(
        Position { x: 15.0, y: 15.0 },
        Position { x: 70.0, y: 70.0 },
        Position { x: 0, y: 0 },
        size,
        false,
        None,
        false,
        false,
    );
}
