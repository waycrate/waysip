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
    #[cfg(feature = "benchmark")]
    bench: bool,
    #[cfg(feature = "freeze")]
    freeze: bool,
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

    #[cfg(feature = "benchmark")]
    pub fn with_bench(mut self) -> Self {
        self.bench = true;
        self
    }

    /// Freeze the screen so the visible desktop stays static while selecting.
    #[cfg(feature = "freeze")]
    pub fn with_freeze(mut self) -> Self {
        self.freeze = true;
        self
    }

    /// get the selected area
    pub fn get(self) -> Result<Option<state::AreaInfo>, WaySipError> {
        #[cfg(feature = "benchmark")]
        let bench = self.bench;
        #[cfg(feature = "freeze")]
        let freeze = self.freeze;

        match self.conn {
            Some(connection) => get_area_inner(
                &connection,
                self.selection_type,
                self.style,
                self.predefined_boxes,
                self.aspect_ratio,
                #[cfg(feature = "benchmark")]
                bench,
                #[cfg(feature = "freeze")]
                freeze,
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
                    #[cfg(feature = "benchmark")]
                    bench,
                    #[cfg(feature = "freeze")]
                    freeze,
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
    #[cfg(feature = "benchmark")] bench: bool,
    #[cfg(feature = "freeze")] freeze: bool,
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
        .roundtrip(&mut state)
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
        .roundtrip(&mut state)
        .map_err(WaySipError::DispatchError)?; // then make a dispatch

    // you will find you get the outputs, but if you do not
    // do the step before, you get empty list

    #[cfg(feature = "freeze")]
    let frozen_backgrounds = if freeze {
        capture_frozen_backgrounds(connection, &state.wloutput_infos)?
    } else {
        Vec::new()
    };
    #[cfg(feature = "freeze")]
    let mut frozen_backgrounds = frozen_backgrounds.into_iter();

    let layer_shell = globals
        .bind::<ZwlrLayerShellV1, _, _>(&qh, 3..=4, ())
        .map_err(WaySipError::NotSupportedProtocol)?;

    // so it is the same way, to get surface detach to protocol, first get the shell, like wmbase
    // or layer_shell or session-shell, then get `surface` from the wl_surface you get before, and
    // set it
    // finally thing to remember is to commit the surface, make the shell to init.
    for wloutput in state.wloutput_infos.iter() {
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
            "osk".to_owned(),
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
        #[cfg(feature = "freeze")]
        let frozen_bg = frozen_backgrounds.next().flatten();
        #[cfg(not(feature = "freeze"))]
        let frozen_bg: Option<cairo::ImageSurface> = None;

        let mut file = tempfile::tempfile().unwrap();
        let UiInit {
            context: cairo_t,
            stride,
        } = render::draw_ui(
            &mut file,
            (init_w, init_h),
            style.background_color,
            frozen_bg.as_ref(),
        );
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
            prev_selection: None,
            margin: std::cell::OnceCell::new(),
            frozen_bg,
        });
    }
    state.shm = Some(shm);
    state.qh = Some(qh);

    #[cfg(feature = "benchmark")]
    {
        state.bench = bench;
    }

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

/// Takes a screenshot of every output before the selection UI is shown, so
/// the visible desktop can be kept static ("frozen") while the user selects.
#[cfg(feature = "freeze")]
fn capture_frozen_backgrounds(
    connection: &Connection,
    outputs: &[state::WlOutputInfo],
) -> Result<Vec<Option<cairo::ImageSurface>>, WaySipError> {
    let wayshot_conn =
        libwayshot::WayshotConnection::from_connection(connection.clone()).map_err(|e| {
            WaySipError::InitFailed(format!("Failed to initialize screenshot backend: {e}"))
        })?;
    let available = wayshot_conn.get_all_outputs();

    Ok(outputs
        .iter()
        .map(|wloutput| -> Option<cairo::ImageSurface> {
            let output_info = available
                .iter()
                .find(|info| info.name == wloutput.get_name())?;
            let image = wayshot_conn
                .screenshot_single_output(output_info, false)
                .ok()?;
            image_to_argb_surface(image)
        })
        .collect())
}

/// Converts an [`image::DynamicImage`] into a premultiplied-alpha
/// `cairo::ImageSurface` (`ARgb32`) suitable for use as a paint source.
#[cfg(feature = "freeze")]
fn image_to_argb_surface(image: image::DynamicImage) -> Option<cairo::ImageSurface> {
    let rgba = image.to_rgba8();
    let width = rgba.width() as i32;
    let height = rgba.height() as i32;
    if width == 0 || height == 0 {
        return None;
    }

    let stride = cairo::Format::ARgb32.stride_for_width(width as u32).ok()?;
    let src = rgba.as_raw();
    let mut data = vec![0u8; stride as usize * height as usize];

    for y in 0..height as usize {
        let row = y * stride as usize;
        for x in 0..width as usize {
            let si = (y * width as usize + x) * 4;
            let r = src[si] as u32;
            let g = src[si + 1] as u32;
            let b = src[si + 2] as u32;
            let a = src[si + 3] as u32;
            // cairo's ARgb32 stores premultiplied-alpha pixels.
            let pr = r * a / 255;
            let pg = g * a / 255;
            let pb = b * a / 255;
            let pixel = (a << 24) | (pr << 16) | (pg << 8) | pb;

            let di = row + x * 4;
            data[di..di + 4].copy_from_slice(&pixel.to_ne_bytes());
        }
    }

    cairo::ImageSurface::create_for_data(data, cairo::Format::ARgb32, width, height, stride).ok()
}
