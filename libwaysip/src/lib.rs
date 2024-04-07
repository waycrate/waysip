mod dispatch;
mod error;
mod render;
pub mod state;

pub use error::WaySipError;

use std::os::unix::prelude::AsFd;


use wayland_client::{
    globals::{registry_queue_init},
    protocol::{
        wl_compositor::WlCompositor,
        wl_seat::{WlSeat},
        wl_shm::{self, WlShm},
    },
    Connection,
};



use wayland_protocols::xdg::xdg_output::zv1::client::{
    zxdg_output_manager_v1::ZxdgOutputManagerV1,
};

use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{Layer, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{self, Anchor},
};

use wayland_protocols::wp::cursor_shape::v1::client::{
    wp_cursor_shape_manager_v1::WpCursorShapeManagerV1,
};

use wayland_cursor::CursorImageBuffer;
use wayland_cursor::CursorTheme;
fn get_cursor_buffer(connection: &Connection, shm: &WlShm) -> Option<CursorImageBuffer> {
    let mut cursor_theme = CursorTheme::load(connection, shm.clone(), 23).ok()?;
    let mut cursor = cursor_theme.get_cursor("crosshair");
    if cursor.is_none() {
        cursor = cursor_theme.get_cursor("left_ptr");
    }
    Some(cursor?[0].clone())
}

/// get the selected area
pub fn get_area(kind: state::WaySipKind) -> Result<Option<state::AreaInfo>, WaySipError> {
    let connection =
        Connection::connect_to_env().map_err(|e| WaySipError::InitFailed(e.to_string()))?;
    let (globals, _) = registry_queue_init::<state::BaseState>(&connection)
        .map_err(|e| WaySipError::InitFailed(e.to_string()))?; // We just need the
                                                               // global, the
                                                               // event_queue is
                                                               // not needed, we
                                                               // do not need
                                                               // state::BaseState after
                                                               // this anymore

    let mut state = state::SecondState::new(kind);

    let mut event_queue = connection.new_event_queue::<state::SecondState>();
    let qh = event_queue.handle();

    let wmcompositer = globals
        .bind::<WlCompositor, _, _>(&qh, 1..=5, ())
        .map_err(WaySipError::NotSupportedProtocol)?; // so the first
                                                      // thing is to
                                                      // get WlCompositor

    let cursor_manager = globals
        .bind::<WpCursorShapeManagerV1, _, _>(&qh, 1..=1, ())
        .ok();

    let shm = globals
        .bind::<WlShm, _, _>(&qh, 1..=1, ())
        .map_err(WaySipError::NotSupportedProtocol)?;

    let cursor_buffer = get_cursor_buffer(&connection, &shm);

    if cursor_manager.is_none() && cursor_buffer.is_none() {
        return Err(WaySipError::CursorThemeFetchFailed);
    }

    state.cursor_manager = cursor_manager;

    globals
        .bind::<WlSeat, _, _>(&qh, 1..=1, ())
        .map_err(WaySipError::NotSupportedProtocol)?;

    let _ = connection.display().get_registry(&qh, ()); // so if you want WlOutput, you need to
                                                        // register this

    event_queue
        .blocking_dispatch(&mut state)
        .map_err(WaySipError::DispatchError)?; // then make a dispatch

    let xdg_output_manager = globals
        .bind::<ZxdgOutputManagerV1, _, _>(&qh, 1..=3, ())
        .map_err(WaySipError::NotSupportedProtocol)?;

    for wloutput in state.outputs.iter() {
        let zwloutput = xdg_output_manager.get_xdg_output(wloutput.get_output(), &qh, ());
        state
            .zxdgoutputs
            .push(state::ZXdgOutputInfo::new(zwloutput));
    }

    event_queue
        .blocking_dispatch(&mut state)
        .map_err(WaySipError::DispatchError)?; // then make a dispatch

    // you will find you get the outputs, but if you do not
    // do the step before, you get empty list

    let layer_shell = globals
        .bind::<ZwlrLayerShellV1, _, _>(&qh, 3..=4, ())
        .map_err(WaySipError::NotSupportedProtocol)?;

    // so it is the same way, to get surface detach to protocol, first get the shell, like wmbase
    // or layer_shell or session-shell, then get `surface` from the wl_surface you get before, and
    // set it
    // finally thing to remember is to commit the surface, make the shell to init.
    for (index, (wloutput, zwlinfo)) in state
        .outputs
        .iter()
        .zip(state.zxdgoutputs.iter())
        .enumerate()
    {
        let wl_surface = wmcompositer.create_surface(&qh, ()); // and create a surface. if two or more,
                                                               // we need to create more
        let (init_w, init_h) = (zwlinfo.width, zwlinfo.height);
        // this example is ok for both xdg_surface and layer_shell

        let layer = layer_shell.get_layer_surface(
            &wl_surface,
            Some(wloutput.get_output()),
            Layer::Overlay,
            format!("nobody_{index}"),
            &qh,
            (),
        );
        layer.set_anchor(Anchor::Top | Anchor::Left | Anchor::Right | Anchor::Bottom);
        layer.set_exclusive_zone(-1);
        layer.set_keyboard_interactivity(zwlr_layer_surface_v1::KeyboardInteractivity::OnDemand);
        layer.set_size(init_w as u32, init_h as u32);

        wl_surface.commit(); // so during the init Configure of the shell, a buffer, atleast a buffer is needed.
                             // and if you need to reconfigure it, you need to commit the wl_surface again
                             // so because this is just an example, so we just commit it once
                             // like if you want to reset anchor or KeyboardInteractivity or resize, commit is needed
        let mut file = tempfile::tempfile().unwrap();
        let cairo_t = render::draw_ui(&mut file, (init_w, init_h));
        let pool = shm.create_pool(file.as_fd(), init_w * init_h * 4, &qh, ());

        let buffer = pool.create_buffer(
            0,
            init_w,
            init_h,
            init_w * 4,
            wl_shm::Format::Argb8888,
            &qh,
            (),
        );

        let cursor_suface = wmcompositer.create_surface(&qh, ()); // and create a surface. if two or more,
        state.wl_surfaces.push(state::LayerSurfaceInfo {
            layer,
            wl_surface,
            cursor_suface,
            buffer,
            cursor_buffer: cursor_buffer.clone(),
            cairo_t,
        });
    }

    while state.running {
        event_queue
            .blocking_dispatch(&mut state)
            .map_err(WaySipError::DispatchError)?;
    }

    Ok(state.area_info())
}
