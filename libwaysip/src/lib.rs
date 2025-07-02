mod dispatch;
mod render;

pub mod error;
pub mod state;
mod utils;
pub use utils::*;

use error::WaySipError;
use render::UiInit;
pub use state::{AreaInfo, BoxInfo, SelectionType};
use std::os::unix::prelude::AsFd;
use wayland_client::{
    Connection,
    globals::registry_queue_init,
    protocol::{
        wl_compositor::WlCompositor,
        wl_seat::WlSeat,
        wl_shm::{self, WlShm},
    },
};
use wayland_cursor::{CursorImageBuffer, CursorTheme};
use wayland_protocols::{
    wp::cursor_shape::v1::client::wp_cursor_shape_manager_v1::WpCursorShapeManagerV1,
    xdg::xdg_output::zv1::client::zxdg_output_manager_v1::ZxdgOutputManagerV1,
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{Layer, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{self, Anchor},
};

fn get_cursor_buffer(connection: &Connection, shm: &WlShm) -> Option<CursorImageBuffer> {
    let mut cursor_theme = CursorTheme::load(connection, shm.clone(), 23).ok()?;
    let mut cursor = cursor_theme.get_cursor("crosshair");
    if cursor.is_none() {
        cursor = cursor_theme.get_cursor("left_ptr");
    }
    Some(cursor?[0].clone())
}

#[derive(Debug, Default)]
pub struct WaySip {
    conn: Option<Connection>,
    selection_type: SelectionType,
    style: Style,
    predefined_boxes: Option<Vec<state::BoxInfo>>,
    aspect_ratio: Option<(f64, f64)>,
}

impl WaySip {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_connection(mut self, conn: Connection) -> Self {
        self.conn = Some(conn);
        self
    }

    pub fn with_selection_type(mut self, selection_type: SelectionType) -> Self {
        self.selection_type = selection_type;
        self
    }

    pub fn with_background_color(mut self, color: Color) -> Self {
        self.style.background_color = color;
        self
    }

    pub fn with_foreground_color(mut self, color: Color) -> Self {
        self.style.foreground_color = color;
        self
    }

    pub fn with_border_text_color(mut self, color: Color) -> Self {
        self.style.border_text_color = color;
        self
    }
    pub fn with_box_color(mut self, color: Color) -> Self {
        self.style.box_color = color;
        self
    }
    pub fn with_border_weight(mut self, border_weight: f64) -> Self {
        self.style.border_weight = border_weight;
        self
    }
    pub fn with_font_size(mut self, font_size: i32) -> Self {
        self.style.font_size = font_size;
        self
    }
    pub fn with_font_name(mut self, font_name: String) -> Self {
        self.style.font_name = font_name;
        self
    }

    pub fn with_predefined_boxes(mut self, boxes: Vec<state::BoxInfo>) -> Self {
        self.predefined_boxes = Some(boxes);
        self
    }

    pub fn with_aspect_ratio(mut self, width: f64, height: f64) -> Self {
        self.aspect_ratio = Some((width, height));
        self
    }

    /// get the selected area
    pub fn get(self) -> Result<Option<state::AreaInfo>, WaySipError> {
        match self.conn {
            Some(connection) => get_area_inner(
                &connection,
                self.selection_type,
                self.style,
                self.predefined_boxes,
                self.aspect_ratio,
            ),
            None => {
                let connection = Connection::connect_to_env()
                    .map_err(|e| WaySipError::InitFailed(e.to_string()))?;

                get_area_inner(
                    &connection,
                    self.selection_type,
                    self.style,
                    self.predefined_boxes,
                    self.aspect_ratio,
                )
            }
        }
    }
}

fn get_area_inner(
    connection: &Connection,
    selection_type: SelectionType,
    style: Style,
    boxes: Option<Vec<state::BoxInfo>>,
    aspect_ratio: Option<(f64, f64)>,
) -> Result<Option<state::AreaInfo>, WaySipError> {
    let (globals, _) = registry_queue_init::<state::WaysipState>(connection)
        .map_err(|e| WaySipError::InitFailed(e.to_string()))?;
    let mut state = state::WaysipState::new(selection_type);

    state.predefined_boxes = boxes;
    state.aspect_ratio = aspect_ratio;

    let mut event_queue = connection.new_event_queue::<state::WaysipState>();
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

    let cursor_buffer = get_cursor_buffer(connection, &shm);

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

    for wloutput in state.wloutput_infos.iter_mut() {
        let zwloutput = xdg_output_manager.get_xdg_output(wloutput.get_output(), &qh, ());
        wloutput
            .xdg_output_info
            .set(state::ZXdgOutputInfo::new(zwloutput))
            .expect("should be set only once");
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
    for (index, wloutput) in state.wloutput_infos.iter().enumerate() {
        let wl_surface = wmcompositer.create_surface(&qh, ()); // and create a surface. if two or more,
        // we need to create more
        let zwlinfo = wloutput.xdg_output_info();
        let Size {
            width: init_w,
            height: init_h,
        } = zwlinfo.size;
        // this example is ok for both xdg_surface and layer_shell

        let layer = layer_shell.get_layer_surface(
            &wl_surface,
            Some(wloutput.get_output()),
            Layer::Overlay,
            format!("nobody_{index}"),
            &qh,
            (),
        );
        layer.set_anchor(Anchor::all());
        layer.set_exclusive_zone(-1);
        layer.set_keyboard_interactivity(zwlr_layer_surface_v1::KeyboardInteractivity::OnDemand);
        layer.set_size(init_w as u32, init_h as u32);

        wl_surface.commit(); // so during the init Configure of the shell, a buffer, at least a buffer is needed.
        // and if you need to reconfigure it, you need to commit the wl_surface again
        // so because this is just an example, so we just commit it once
        // like if you want to reset anchor or KeyboardInteractivity or resize, commit is needed
        let mut file = tempfile::tempfile().unwrap();
        let UiInit {
            context: cairo_t,
            stride,
        } = render::draw_ui(&mut file, (init_w, init_h), style.background_color);
        let pool = shm.create_pool(file.as_fd(), init_w * init_h * 4, &qh, ());

        let buffer =
            pool.create_buffer(0, init_w, init_h, stride, wl_shm::Format::Argb8888, &qh, ());

        let cursor_surface = wmcompositer.create_surface(&qh, ()); // and create a surface. if two or more,
        state.wl_surfaces.push(state::LayerSurfaceInfo {
            layer,
            wl_surface,
            cursor_surface,
            buffer,
            cursor_buffer: cursor_buffer.clone(),
            cairo_t,
            inited: false,
            buffer_busy: true,
            stride,
            style: style.clone(),
            pango_layout: std::cell::OnceCell::new(),
            font_desc_bold: std::cell::OnceCell::new(),
            font_desc_normal: std::cell::OnceCell::new(),
        });
    }
    state.shm = Some(shm);

    state.qh = Some(qh);
    while state.running {
        event_queue
            .blocking_dispatch(&mut state)
            .map_err(WaySipError::DispatchError)?;
    }

    layer_shell.destroy();
    for surface in &state.wl_surfaces {
        surface.layer.destroy();
        surface.wl_surface.destroy();
        surface.cursor_surface.destroy();
        surface.buffer.destroy();
    }
    state.wl_surfaces.clear();
    Ok(state.area_info())
}
