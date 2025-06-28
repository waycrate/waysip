use std::{cell::OnceCell, os::fd::AsFd};

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
    Position, Size,
    error::ColorError,
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

#[derive(Debug, Clone)]
pub struct ZXdgOutputInfo {
    pub zxdg_output: zxdg_output_v1::ZxdgOutputV1,
    pub size: Size,
    pub start_position: Position,
    pub name: String,
    pub description: String,
}

impl ZXdgOutputInfo {
    pub fn new(zxdgoutput: zxdg_output_v1::ZxdgOutputV1) -> Self {
        Self {
            zxdg_output: zxdgoutput,
            size: Size {
                width: 0,
                height: 0,
            },
            start_position: Position { x: 0, y: 0 },
            name: "".to_string(),
            description: "".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WlOutputInfo {
    pub output: WlOutput,
    pub description: String,
    pub name: String,
    pub size: Size,
    pub xdg_output_info: OnceCell<ZXdgOutputInfo>,
}

impl WlOutputInfo {
    pub(crate) fn xdg_output_info_mut(&mut self) -> &mut ZXdgOutputInfo {
        self.xdg_output_info.get_mut().expect("should inited")
    }
    pub(crate) fn xdg_output_info(&self) -> &ZXdgOutputInfo {
        self.xdg_output_info.get().expect("should inited")
    }
    pub(crate) fn zxdg_output(&self) -> &zxdg_output_v1::ZxdgOutputV1 {
        &self
            .xdg_output_info
            .get()
            .expect("should inited")
            .zxdg_output
    }
    pub fn new(output: WlOutput) -> Self {
        Self {
            output,
            description: "".to_string(),
            name: "".to_string(),
            size: Size {
                width: 0,
                height: 0,
            },
            xdg_output_info: OnceCell::new(),
        }
    }
    pub fn get_screen_info(&self) -> ScreenInfo {
        let xdg_output_info = self.xdg_output_info();
        ScreenInfo {
            name: self.name.clone(),
            description: self.description.clone(),
            position: xdg_output_info.start_position,
            output_size: self.size,
            wl_output: self.output.clone(),
            screen_size: xdg_output_info.size,
        }
    }
    pub fn get_output(&self) -> &WlOutput {
        &self.output
    }

    pub fn get_size(&self) -> Size {
        self.size
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
    pub position: Position,
    pub screen_size: Size,
    pub wl_output: WlOutput,
    pub output_size: Size,
    pub name: String,
    pub description: String,
}

impl ScreenInfo {
    /// get the binding output
    pub fn get_wloutput(&self) -> &WlOutput {
        &self.wl_output
    }

    /// get the logical size of the wloutput
    pub fn get_wloutput_size(&self) -> Size {
        self.output_size
    }

    /// get the logical size of the screen
    pub fn get_size(&self) -> Size {
        self.screen_size
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
    pub fn get_position(&self) -> Position {
        self.position
    }
}

#[derive(Debug)]
pub struct WaysipState {
    pub wloutput_infos: Vec<WlOutputInfo>,
    pub running: bool,
    pub selection_type: SelectionType,
    pub wl_surfaces: Vec<LayerSurfaceInfo>,
    pub current_pos: Position<f64>,
    pub start_pos: Option<Position<f64>>,
    pub end_pos: Option<Position<f64>>,
    pub current_screen: usize,
    pub cursor_manager: Option<WpCursorShapeManagerV1>,
    pub shm: Option<WlShm>,
    pub qh: Option<QueueHandle<Self>>,
    pub last_redraw: std::time::Instant,
}

impl WaysipState {
    pub fn new(selection_type: SelectionType) -> Self {
        WaysipState {
            wloutput_infos: Vec::new(),
            running: true,
            selection_type,
            wl_surfaces: Vec::new(),
            current_pos: Position { x: 0., y: 0. },
            start_pos: None,
            end_pos: None,
            current_screen: 0,
            cursor_manager: None,
            qh: None,
            shm: None,
            last_redraw: std::time::Instant::now() - std::time::Duration::from_secs(1),
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
        } = render::draw_ui(
            &mut file,
            (width, width),
            surface_info.style.background_color,
        );
        let pool = self
            .shm
            .as_ref()
            .unwrap()
            .create_pool(file.as_fd(), width * height * 4, qh, ());

        let buffer = pool.create_buffer(0, width, height, stride, wl_shm::Format::Argb8888, qh, ());

        let old_buffer = std::mem::replace(&mut surface_info.buffer, buffer);
        let old_cairo_t = std::mem::replace(&mut surface_info.cairo_t, cairo_t);
        old_buffer.destroy();
        drop(old_cairo_t);

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
        let qh = self.qh.as_ref().unwrap();
        for (idx, surface) in self.wl_surfaces.iter().enumerate() {
            surface.wl_surface.frame(qh, idx);
            surface.wl_surface.commit();
        }
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
            size,
            start_position,
            name,
            description,
            ..
        } = self.wloutput_infos[screen_index].xdg_output_info();

        if self.is_screen() {
            for (idx, info) in self
                .wl_surfaces
                .iter()
                .enumerate()
                .filter(|(_, i)| i.inited)
            {
                let ZXdgOutputInfo {
                    size,
                    start_position,
                    ..
                } = &self.wloutput_infos[idx].xdg_output_info();
                info.redraw_select_screen(
                    idx == self.current_screen,
                    *size,
                    *start_position,
                    name,
                    description,
                );
            }
        } else {
            if self.start_pos.is_none() {
                return;
            }
            info.redraw(
                self.start_pos.unwrap(),
                self.current_pos,
                *start_position,
                *size,
            );
        }
    }

    pub fn area_info(&self) -> Option<AreaInfo> {
        if self.start_pos.is_none() || self.end_pos.is_none() {
            return None;
        }
        let Position {
            x: start_x,
            y: start_y,
        } = self.start_pos.unwrap();
        let Position { x: end_x, y: end_y } = self.end_pos.unwrap();
        let output = self.wloutput_infos[self.current_screen].clone();
        Some(AreaInfo {
            start_x,
            start_y,
            end_x,
            end_y,
            screen_info: output.get_screen_info(),
        })
    }
}

#[derive(Debug)]
pub struct LayerSurfaceInfo {
    pub layer: ZwlrLayerSurfaceV1,
    pub wl_surface: WlSurface,
    pub cursor_surface: WlSurface,
    pub buffer: WlBuffer,
    pub cursor_buffer: Option<CursorImageBuffer>,
    pub cairo_t: cairo::Context,
    pub stride: i32,
    pub inited: bool,
    pub buffer_busy: bool,
    pub style: Style,
    pub pango_layout: std::cell::OnceCell<pango::Layout>,
    pub font_desc_bold: std::cell::OnceCell<pango::FontDescription>,
    pub font_desc_normal: std::cell::OnceCell<pango::FontDescription>,
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
    pub fn left_top_point(&self) -> Position {
        Position {
            x: self.start_x.min(self.end_x) as i32,
            y: (self.start_y.min(self.end_y)) as i32,
        }
    }

    /// you can get the info of the chosen screen
    pub fn selected_screen_info(&self) -> &ScreenInfo {
        &self.screen_info
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Default for Color {
    fn default() -> Self {
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.5,
        }
    }
}

impl Color {
    pub fn hex_to_color(colorhex: String) -> Result<Color, ColorError> {
        let stripped_color = colorhex.trim_start_matches('#');

        if stripped_color.len() != 8 || !stripped_color.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ColorError::InvalidColorFormat(colorhex.to_string()));
        }
        let color = Color {
            r: u8::from_str_radix(&stripped_color[0..2], 16)? as f64 / 255.0,
            g: u8::from_str_radix(&stripped_color[2..4], 16)? as f64 / 255.0,
            b: u8::from_str_radix(&stripped_color[4..6], 16)? as f64 / 255.0,
            a: u8::from_str_radix(&stripped_color[6..8], 16)? as f64 / 255.0,
        };
        Ok(color)
    }
}

#[derive(Debug, Clone)]
pub struct Style {
    pub background_color: Color,
    pub foreground_color: Color,
    pub border_text_color: Color,
    pub border_weight: f64,
    pub font_size: i32,
    pub font_name: String,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            background_color: Color {
                r: 0.4,
                g: 0.4,
                b: 0.4,
                a: 0.5,
            }, // #66666680
            foreground_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            }, // #00000000
            border_text_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }, // #000000ff
            border_weight: 1.0,
            font_size: 12,
            font_name: "Sans".to_string(),
        }
    }
}
