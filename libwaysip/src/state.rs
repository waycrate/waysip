use std::{mem::ManuallyDrop, os::fd::AsFd};

use wayland_client::{
    QueueHandle,
    protocol::{
        wl_buffer::WlBuffer,
        wl_output::WlOutput,
        wl_shm::{self, WlShm},
        wl_surface::WlSurface,
    },
};
use wayland_cursor::CursorImageBuffer;
use wayland_protocols::{
    wp::cursor_shape::v1::client::wp_cursor_shape_manager_v1::WpCursorShapeManagerV1,
    xdg::xdg_output::zv1::client::zxdg_output_v1,
};
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::ZwlrLayerSurfaceV1;

use crate::{
    Point, Size,
    render::{self, UiInit},
};

/// You are allow to choose three actions of waysip, include area selection, point selection, and
/// select screen
#[derive(Debug, Clone, Copy, Default)]
pub enum SelectionType {
    #[default]
    Area,
    Point,
    Screen,
}

#[derive(Debug)]
pub struct ZXdgOutputInfo {
    pub zxdg_output: zxdg_output_v1::ZxdgOutputV1,
    pub width: i32,
    pub height: i32,
    pub start_x: i32,
    pub start_y: i32,
    pub name: String,
    pub description: String,
}

impl ZXdgOutputInfo {
    pub fn new(zxdgoutput: zxdg_output_v1::ZxdgOutputV1) -> Self {
        Self {
            zxdg_output: zxdgoutput,
            width: 0,
            height: 0,
            start_x: 0,
            start_y: 0,
            name: "".to_string(),
            description: "".to_string(),
        }
    }
    pub fn get_screen_info(&self, output_info: WlOutputInfo) -> ScreenInfo {
        ScreenInfo {
            output_info,
            start_x: self.start_x,
            start_y: self.start_y,
            width: self.width,
            height: self.height,
            name: self.name.clone(),
            description: self.description.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WlOutputInfo {
    pub output: WlOutput,
    pub description: String,
    pub name: String,
    pub size: (i32, i32),
}

impl WlOutputInfo {
    pub fn new(output: WlOutput) -> Self {
        Self {
            output,
            description: "".to_string(),
            name: "".to_string(),
            size: (0, 0),
        }
    }

    pub fn get_output(&self) -> &WlOutput {
        &self.output
    }

    pub fn get_size(&self) -> Size {
        self.size.into()
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_description(&self) -> &str {
        &self.description
    }
}

/// tell the screen info, include description, size and the name. and include the current wloutput
/// binded by the screen
#[derive(Debug)]
pub struct ScreenInfo {
    pub output_info: WlOutputInfo,
    pub start_x: i32,
    pub start_y: i32,
    pub width: i32,
    pub height: i32,
    pub name: String,
    pub description: String,
}

impl ScreenInfo {
    /// get the binding output
    pub fn get_outputinfo(&self) -> &WlOutputInfo {
        &self.output_info
    }

    /// get the logical size of the screen
    pub fn get_size(&self) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    /// get the name of the screen
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// get the description of the screen
    pub fn get_description(&self) -> &str {
        &self.description
    }

    /// get the logical position of the screen
    pub fn get_position(&self) -> Point {
        Point {
            x: self.start_x,
            y: self.start_y,
        }
    }
}

#[derive(Debug)]
pub struct WaysipState {
    pub outputs: Vec<WlOutputInfo>,
    pub zxdg_outputs: Vec<ZXdgOutputInfo>,
    pub running: bool,
    pub selection_type: SelectionType,
    pub wl_surfaces: Vec<LayerSurfaceInfo>,
    pub current_pos: (f64, f64),
    pub start_pos: Option<(f64, f64)>,
    pub end_pos: Option<(f64, f64)>,
    pub current_screen: usize,
    pub cursor_manager: Option<WpCursorShapeManagerV1>,
    pub shm: Option<WlShm>,
    pub qh: Option<QueueHandle<Self>>,
}

impl WaysipState {
    pub fn new(selection_type: SelectionType) -> Self {
        WaysipState {
            outputs: Vec::new(),
            zxdg_outputs: Vec::new(),
            running: true,
            selection_type,
            wl_surfaces: Vec::new(),
            current_pos: (0., 0.),
            start_pos: None,
            end_pos: None,
            current_screen: 0,
            cursor_manager: None,
            qh: None,
            shm: None,
        }
    }

    pub fn is_area(&self) -> bool {
        matches!(self.selection_type, SelectionType::Area)
    }

    pub fn is_screen(&self) -> bool {
        matches!(self.selection_type, SelectionType::Screen)
    }

    pub fn ensure_buffer(&mut self, surface: &ZwlrLayerSurfaceV1, (width, height): (u32, u32)) {
        let Some(surface_info) = self
            .wl_surfaces
            .iter_mut()
            .find(|info| info.layer == *surface)
        else {
            return;
        };
        if surface_info.buffer_busy {
            return;
        }
        let mut file = tempfile::tempfile().unwrap();
        let qh = self.qh.as_ref().unwrap();
        let width = width as i32;
        let height = height as i32;
        let UiInit {
            context: cairo_t,
            stride,
        } = render::draw_ui(&mut file, (width, width));
        let pool =
            self.shm
                .as_ref()
                .unwrap()
                .create_pool(file.as_fd(), width * height * 4, qh, ());

        let buffer =
            pool.create_buffer(0, width, height, stride, wl_shm::Format::Argb8888, qh, ());
        unsafe {
            let old_buffer = ManuallyDrop::take(&mut surface_info.buffer);
            old_buffer.destroy();
            let old_cairo_t = ManuallyDrop::take(&mut surface_info.cairo_t);
            drop(old_cairo_t);
        }
        surface_info.buffer = ManuallyDrop::new(buffer);
        surface_info.cairo_t = ManuallyDrop::new(cairo_t);
        surface_info.buffer_busy = true;
        surface_info.inited = false;
    }

    pub fn ensure_init(&mut self, surface: &ZwlrLayerSurfaceV1) {
        let Some(surface_info) = self
            .wl_surfaces
            .iter_mut()
            .find(|info| info.layer == *surface)
        else {
            return;
        };
        if surface_info.inited {
            return;
        }
        surface_info.init_commit();
        surface_info.inited = true;
    }

    pub fn commit(&self) {
        let surface = &self.wl_surfaces[self.current_screen];
        surface
            .wl_surface
            .frame(self.qh.as_ref().unwrap(), self.current_screen);
        surface.wl_surface.commit();
    }

    pub fn redraw_current_surface(&self) {
        let surface_info = &self.wl_surfaces[self.current_screen];
        self.redraw(&surface_info.layer);
    }

    pub fn redraw(&self, surface: &ZwlrLayerSurfaceV1) {
        let Some(screen_index) = self
            .wl_surfaces
            .iter()
            .position(|info| info.layer == *surface)
        else {
            return;
        };

        let info = &self.wl_surfaces[screen_index];
        let ZXdgOutputInfo {
            width,
            height,
            start_x,
            start_y,
            name,
            description,
            ..
        } = &self.zxdg_outputs[screen_index];

        if self.is_screen() {
            info.redraw_select_screen(
                self.current_screen == screen_index,
                (*width, *height),
                (*start_x, *start_y),
                name,
                description,
            );
        } else {
            if self.start_pos.is_none() {
                return;
            }
            let (pos_x, pos_y) = self.start_pos.unwrap();
            info.redraw(
                (pos_x, pos_y),
                self.current_pos,
                (*start_x, *start_y),
                (*width, *height),
            );
        }
    }

    pub fn area_info(&self) -> Option<AreaInfo> {
        if self.start_pos.is_none() || self.end_pos.is_none() {
            return None;
        }
        let (start_x, start_y) = self.start_pos.unwrap();
        let (end_x, end_y) = self.end_pos.unwrap();
        let output = self.outputs[self.current_screen].clone();
        let info = &self.zxdg_outputs[self.current_screen];
        Some(AreaInfo {
            start_x,
            start_y,
            end_x,
            end_y,
            screen_info: info.get_screen_info(output),
        })
    }
}

#[derive(Debug)]
pub struct LayerSurfaceInfo {
    pub layer: ZwlrLayerSurfaceV1,
    pub wl_surface: WlSurface,
    pub cursor_surface: WlSurface,
    pub buffer: ManuallyDrop<WlBuffer>,
    pub cursor_buffer: Option<CursorImageBuffer>,
    pub cairo_t: ManuallyDrop<cairo::Context>,
    pub stride: i32,
    pub inited: bool,
    pub buffer_busy: bool,
}

/// describe the information of the area
#[derive(Debug)]
pub struct AreaInfo {
    pub start_x: f64,
    pub start_y: f64,
    pub end_x: f64,
    pub end_y: f64,

    pub screen_info: ScreenInfo,
}

impl AreaInfo {
    /// provide the width of the area as f64
    pub fn width_f64(&self) -> f64 {
        (self.end_x - self.start_x).abs()
    }

    pub fn size(&self) -> Size {
        Size {
            width: self.width(),
            height: self.height(),
        }
    }

    pub fn size_f(&self) -> Size<f64> {
        Size {
            width: self.height_f64(),
            height: self.width_f64(),
        }
    }

    /// provide the width of the area as i32
    pub fn width(&self) -> i32 {
        self.width_f64() as i32
    }

    /// provide the height of the area as f64
    pub fn height_f64(&self) -> f64 {
        (self.end_y - self.start_y).abs()
    }

    /// provide the width of the area as i32
    pub fn height(&self) -> i32 {
        self.height_f64() as i32
    }

    /// calculate the real start position
    pub fn left_top_point(&self) -> Point {
        Point {
            x: self.start_x.min(self.end_x) as i32,
            y: (self.start_y.min(self.end_y)) as i32,
        }
    }

    /// you can get the info of the chosen screen
    pub fn selected_screen_info(&self) -> &ScreenInfo {
        &self.screen_info
    }
}
