//! `apply_format` is pure string formatting over an `AreaInfo`, but
//! `AreaInfo::screen_info` embeds a real `WlOutput` proxy. We don't need a
//! live compositor to get one though: an "inert" proxy backed by a locally
//! paired `UnixStream` (same technique as libwaysip's own dispatch tests)
//! is enough, since `apply_format` never actually sends it any protocol
//! requests.

use std::os::unix::net::UnixStream;

use libwaysip::state::ScreenInfo;
use libwaysip::{AreaInfo, BoxInfo, Position, Size};
use wayland_backend::client::Backend;
use wayland_client::protocol::wl_output::WlOutput;
use wayland_client::{Connection, Proxy};

use crate::utils::apply_format;

fn dummy_conn() -> Connection {
    let (client, server) = UnixStream::pair().expect("unix stream");
    Box::leak(Box::new(server));
    let backend = Backend::connect(client).expect("backend");
    Connection::from_backend(backend)
}

#[allow(clippy::too_many_arguments)]
fn area_info(
    conn: &Connection,
    box_info: BoxInfo,
    screen_pos: (i32, i32),
    screen_size: (i32, i32),
    output_size: (i32, i32),
    name: &str,
    description: &str,
) -> AreaInfo {
    AreaInfo {
        box_info,
        screen_info: ScreenInfo {
            position: Position {
                x: screen_pos.0,
                y: screen_pos.1,
            },
            screen_size: Size {
                width: screen_size.0,
                height: screen_size.1,
            },
            wl_output: WlOutput::inert(conn.backend().downgrade()),
            output_size: Size {
                width: output_size.0,
                height: output_size.1,
            },
            name: name.to_string(),
            description: description.to_string(),
        },
        effective_selection_type: None,
        #[cfg(feature = "benchmark")]
        timestamps_total: Vec::new(),
    }
}

fn box_info(start_x: f64, start_y: f64, end_x: f64, end_y: f64) -> BoxInfo {
    BoxInfo {
        start_x,
        start_y,
        end_x,
        end_y,
    }
}

#[test]
fn basic_placeholders_use_selection_bounds() {
    let conn = dummy_conn();
    let info = area_info(
        &conn,
        box_info(10.0, 20.0, 110.0, 170.0),
        (0, 0),
        (1920, 1080),
        (1920, 1080),
        "DP-1",
        "Some Monitor",
    );
    let out = apply_format(&info, "%x,%y %wx%h", false);
    assert_eq!(out, "10,20 100x150");
}

#[test]
fn relative_placeholders_are_clamped_to_screen_bounds() {
    let conn = dummy_conn();
    // Screen starts at (100,100), sized 200x200; selection spills past the
    // screen's right/bottom edge.
    let info = area_info(
        &conn,
        box_info(250.0, 250.0, 400.0, 400.0),
        (100, 100),
        (200, 200),
        (200, 200),
        "DP-1",
        "",
    );
    let out = apply_format(&info, "%X,%Y %Wx%H", false);
    assert_eq!(out, "150,150 50x50");
}

#[test]
fn output_name_and_description_placeholders() {
    let conn = dummy_conn();
    let info = area_info(
        &conn,
        box_info(0.0, 0.0, 10.0, 10.0),
        (0, 0),
        (100, 100),
        (100, 100),
        "DP-1",
        "My Monitor",
    );
    let out = apply_format(&info, "%o|%l|%d", false);
    assert_eq!(out, "DP-1|DP-1|My Monitor");
}

#[test]
fn wloutput_size_placeholders() {
    let conn = dummy_conn();
    let info = area_info(
        &conn,
        box_info(0.0, 0.0, 10.0, 10.0),
        (0, 0),
        (1920, 1080),
        (3840, 2160),
        "DP-1",
        "",
    );
    let out = apply_format(&info, "%Lx%T", false);
    assert_eq!(out, "3840x2160");
}

#[test]
fn screen_mode_uses_screen_bounds_instead_of_selection() {
    let conn = dummy_conn();
    let info = area_info(
        &conn,
        box_info(999.0, 999.0, 1000.0, 1000.0),
        (5, 5),
        (800, 600),
        (800, 600),
        "DP-1",
        "",
    );
    let out = apply_format(&info, "%x,%y %wx%h", true);
    assert_eq!(out, "5,5 800x600");
}

#[test]
fn literal_percent_and_escapes() {
    let conn = dummy_conn();
    let info = area_info(
        &conn,
        box_info(0.0, 0.0, 1.0, 1.0),
        (0, 0),
        (10, 10),
        (10, 10),
        "DP-1",
        "",
    );
    let out = apply_format(&info, r"100%%\n\\end", false);
    assert_eq!(out, "100%\n\\end");
}

#[test]
fn unknown_percent_specifier_passes_through() {
    let conn = dummy_conn();
    let info = area_info(
        &conn,
        box_info(0.0, 0.0, 1.0, 1.0),
        (0, 0),
        (10, 10),
        (10, 10),
        "DP-1",
        "",
    );
    let out = apply_format(&info, "%z", false);
    assert_eq!(out, "z");
}

#[test]
fn zero_size_selection_clamps_to_one_pixel() {
    let conn = dummy_conn();
    let info = area_info(
        &conn,
        box_info(5.0, 5.0, 5.0, 5.0),
        (0, 0),
        (100, 100),
        (100, 100),
        "DP-1",
        "",
    );
    let out = apply_format(&info, "%wx%h", false);
    assert_eq!(out, "1x1");
}
