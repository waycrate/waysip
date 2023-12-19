mod error;
mod render;
pub use error::WaySipError;

use std::os::unix::prelude::AsFd;

use wayland_client::{
    delegate_noop,
    globals::{registry_queue_init, GlobalListContents},
    protocol::{
        wl_buffer::WlBuffer,
        wl_compositor::WlCompositor,
        wl_keyboard,
        wl_output::{self, WlOutput},
        wl_pointer, wl_registry,
        wl_seat::{self, WlSeat},
        wl_shm::{self, WlShm},
        wl_shm_pool::WlShmPool,
        wl_surface::WlSurface,
    },
    Connection, Dispatch, Proxy, WEnum,
};

use wayland_protocols::xdg::shell::client::{xdg_toplevel::XdgToplevel, xdg_wm_base};

use wayland_protocols::xdg::xdg_output::zv1::client::{
    zxdg_output_manager_v1::ZxdgOutputManagerV1, zxdg_output_v1,
};

use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{Layer, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{self, Anchor, ZwlrLayerSurfaceV1},
};

use wayland_protocols::wp::cursor_shape::v1::client::{
    wp_cursor_shape_device_v1::{self, WpCursorShapeDeviceV1},
    wp_cursor_shape_manager_v1::WpCursorShapeManagerV1,
};

use wayland_cursor::CursorImageBuffer;
use wayland_cursor::CursorTheme;

#[derive(Debug)]
struct BaseState;

// so interesting, it is just need to invoke once, it just used to get the globals
impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for BaseState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_registry::WlRegistry,
        _event: <wl_registry::WlRegistry as wayland_client::Proxy>::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qh: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

#[derive(Debug)]
struct ZXdgOutputInfo {
    zxdgoutput: zxdg_output_v1::ZxdgOutputV1,
    width: i32,
    height: i32,
    start_x: i32,
    start_y: i32,
    name: String,
    description: String,
}

#[derive(Debug, Clone)]
pub struct WlOutputInfo {
    output: WlOutput,
    description: String,
    name: String,
    size: (i32, i32),
}

impl WlOutputInfo {
    fn new(output: WlOutput) -> Self {
        Self {
            output,
            description: "".to_string(),
            name: "".to_string(),
            size: (0, 0),
        }
    }

    fn get_output(&self) -> &WlOutput {
        &self.output
    }

    pub fn get_size(&self) -> (i32, i32) {
        self.size
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_description(&self) -> &str {
        &self.description
    }
}

impl ZXdgOutputInfo {
    fn new(zxdgoutput: zxdg_output_v1::ZxdgOutputV1) -> Self {
        Self {
            zxdgoutput,
            width: 0,
            height: 0,
            start_x: 0,
            start_y: 0,
            name: "".to_string(),
            description: "".to_string(),
        }
    }
    fn get_screen_info(&self, output_info: WlOutputInfo) -> ScreenInfo {
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

/// tell the screen info, include description, size and the name. and include the current wloutput
/// binded by the screen
#[derive(Debug)]
pub struct ScreenInfo {
    output_info: WlOutputInfo,
    start_x: i32,
    start_y: i32,
    width: i32,
    height: i32,
    name: String,
    description: String,
}

impl ScreenInfo {
    /// get the binding output
    pub fn get_outputinfo(&self) -> &WlOutputInfo {
        &self.output_info
    }

    /// get the logical size of the screen
    pub fn get_size(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    /// get the name of the screen
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// get the description of the screen
    pub fn get_description(&self) -> &str {
        &self.description
    }

    /// get the logical positon of the screen
    pub fn get_position(&self) -> (i32, i32) {
        (self.start_x, self.start_y)
    }
}

#[derive(Debug)]
struct LayerSurfaceInfo {
    layer: ZwlrLayerSurfaceV1,
    wl_surface: WlSurface,
    cursor_suface: WlSurface,
    buffer: WlBuffer,
    cursor_buffer: Option<CursorImageBuffer>,
    cairo_t: cairo::Context,
}

/// You are allow to choose three actions of waysip, include area selection, point selection, and
/// select sreen
#[derive(Debug, Clone, Copy, Default)]
pub enum WaySipKind {
    #[default]
    Area,
    Point,
    Screen,
}

#[derive(Debug)]
struct SecondState {
    outputs: Vec<WlOutputInfo>,
    zxdgoutputs: Vec<ZXdgOutputInfo>,
    running: bool,
    waysipkind: WaySipKind,
    wl_surfaces: Vec<LayerSurfaceInfo>,
    current_pos: (f64, f64),
    start_pos: Option<(f64, f64)>,
    end_pos: Option<(f64, f64)>,
    current_screen: usize,
    cursor_manager: Option<WpCursorShapeManagerV1>,
}

impl SecondState {
    fn new(waysipkind: WaySipKind) -> Self {
        SecondState {
            outputs: Vec::new(),
            zxdgoutputs: Vec::new(),
            running: true,
            waysipkind,
            wl_surfaces: Vec::new(),
            current_pos: (0., 0.),
            start_pos: None,
            end_pos: None,
            current_screen: 0,
            cursor_manager: None,
        }
    }

    fn is_area(&self) -> bool {
        matches!(self.waysipkind, WaySipKind::Area)
    }

    fn is_screen(&self) -> bool {
        matches!(self.waysipkind, WaySipKind::Screen)
    }
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

    /// caculate the real start position
    pub fn left_top_point(&self) -> (i32, i32) {
        (
            self.start_x.min(self.end_x) as i32,
            (self.start_y.min(self.end_y)) as i32,
        )
    }

    /// you can get the info of the choosed screen
    pub fn selected_screen_info(&self) -> &ScreenInfo {
        &self.screen_info
    }
}

impl SecondState {
    fn redraw_screen(&self) {
        for ((index, info), ZXdgOutputInfo { width, height, .. }) in self
            .wl_surfaces
            .iter()
            .enumerate()
            .zip(self.zxdgoutputs.iter())
        {
            info.redraw_select_screen(self.current_screen == index, (*width, *height));
        }
    }
    fn redraw(&self) {
        if self.start_pos.is_none() {
            return;
        }
        let (pos_x, pos_y) = self.start_pos.unwrap();
        for (
            ZXdgOutputInfo {
                width,
                height,
                start_x,
                start_y,
                ..
            },
            layershell_info,
        ) in self.zxdgoutputs.iter().zip(self.wl_surfaces.iter())
        {
            layershell_info.redraw(
                (pos_x, pos_y),
                self.current_pos,
                (*start_x, *start_y),
                (*width, *height),
            );
        }
    }

    fn area_info(&self) -> Option<AreaInfo> {
        if self.start_pos.is_none() || self.end_pos.is_none() {
            return None;
        }
        let (start_x, start_y) = self.start_pos.unwrap();
        let (end_x, end_y) = self.end_pos.unwrap();
        let output = self.outputs[self.current_screen].clone();
        let info = &self.zxdgoutputs[self.current_screen];
        Some(AreaInfo {
            start_x,
            start_y,
            end_x,
            end_y,
            screen_info: info.get_screen_info(output),
        })
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for SecondState {
    fn event(
        state: &mut Self,
        proxy: &wl_registry::WlRegistry,
        event: <wl_registry::WlRegistry as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        else {
            return;
        };

        if interface == wl_output::WlOutput::interface().name {
            let output = proxy.bind::<wl_output::WlOutput, _, _>(name, version, qh, ());
            state.outputs.push(WlOutputInfo::new(output));
        }
    }
}

impl Dispatch<wl_output::WlOutput, ()> for SecondState {
    fn event(
        state: &mut Self,
        wl_output: &wl_output::WlOutput,
        event: <wl_output::WlOutput as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        let output = state
            .outputs
            .iter_mut()
            .find(|x| x.get_output() == wl_output)
            .unwrap();

        match event {
            wl_output::Event::Name { name } => {
                output.name = name;
            }
            wl_output::Event::Description { description } => {
                output.description = description;
            }
            wl_output::Event::Mode { width, height, .. } => {
                output.size = (width, height);
            }

            _ => (),
        }
    }
}

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for SecondState {
    fn event(
        _state: &mut Self,
        wm_base: &xdg_wm_base::XdgWmBase,
        event: <xdg_wm_base::XdgWmBase as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        if let xdg_wm_base::Event::Ping { serial } = event {
            wm_base.pong(serial);
        }
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for SecondState {
    fn event(
        _state: &mut Self,
        seat: &wl_seat::WlSeat,
        event: <wl_seat::WlSeat as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(capabilities),
        } = event
        {
            if capabilities.contains(wl_seat::Capability::Keyboard) {
                seat.get_keyboard(qh, ());
            }
            if capabilities.contains(wl_seat::Capability::Pointer) {
                seat.get_pointer(qh, ());
            }
        }
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for SecondState {
    fn event(
        state: &mut Self,
        _proxy: &wl_keyboard::WlKeyboard,
        event: <wl_keyboard::WlKeyboard as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_keyboard::Event::Key { key, .. } = event {
            if key == 1 {
                state.running = false;
            }
        }
    }
}

impl Dispatch<wl_pointer::WlPointer, ()> for SecondState {
    fn event(
        dispatch_state: &mut Self,
        pointer: &wl_pointer::WlPointer,
        event: <wl_pointer::WlPointer as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            wl_pointer::Event::Button { state, .. } => {
                match state {
                    WEnum::Value(wl_pointer::ButtonState::Pressed) => {
                        dispatch_state.start_pos = Some(dispatch_state.current_pos);
                        if !dispatch_state.is_area() {
                            dispatch_state.end_pos = Some(dispatch_state.current_pos);
                            dispatch_state.running = false;
                        }
                    }
                    WEnum::Value(wl_pointer::ButtonState::Released) => {
                        dispatch_state.end_pos = Some(dispatch_state.current_pos);
                        // if released, this time select is end
                        dispatch_state.running = false;
                    }
                    _ => {}
                }
                dispatch_state.redraw();
            }
            wl_pointer::Event::Enter {
                serial,
                surface,
                surface_x,
                surface_y,
            } => {
                let Some(LayerSurfaceInfo {
                    cursor_suface,
                    cursor_buffer,
                    ..
                }) = dispatch_state
                    .wl_surfaces
                    .iter()
                    .find(|info| info.wl_surface == surface)
                else {
                    return;
                };
                let current_screen = dispatch_state
                    .wl_surfaces
                    .iter()
                    .position(|info| info.wl_surface == surface)
                    .unwrap();
                dispatch_state.current_screen = current_screen;
                let start_x = dispatch_state.zxdgoutputs[dispatch_state.current_screen].start_x;
                let start_y = dispatch_state.zxdgoutputs[dispatch_state.current_screen].start_y;
                dispatch_state.current_pos =
                    (surface_x + start_x as f64, surface_y + start_y as f64);

                if let Some(ref cursor_manager) = dispatch_state.cursor_manager {
                    let device = cursor_manager.get_pointer(pointer, qh, ());
                    device.set_shape(serial, wp_cursor_shape_device_v1::Shape::Crosshair);
                    device.destroy();
                } else {
                    let cursor_buffer = cursor_buffer.as_ref().unwrap();
                    cursor_suface.attach(Some(cursor_buffer), 0, 0);
                    let (hotspot_x, hotspot_y) = cursor_buffer.hotspot();
                    pointer.set_cursor(
                        serial,
                        Some(cursor_suface),
                        hotspot_x as i32,
                        hotspot_y as i32,
                    );
                    cursor_suface.commit();
                }
                if dispatch_state.is_screen() {
                    dispatch_state.redraw_screen();
                } else {
                    dispatch_state.redraw();
                }
            }
            wl_pointer::Event::Motion {
                surface_x,
                surface_y,
                ..
            } => {
                let start_x = dispatch_state.zxdgoutputs[dispatch_state.current_screen].start_x;
                let start_y = dispatch_state.zxdgoutputs[dispatch_state.current_screen].start_y;
                dispatch_state.current_pos =
                    (surface_x + start_x as f64, surface_y + start_y as f64);
                if dispatch_state.is_area() {
                    dispatch_state.redraw();
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for SecondState {
    fn event(
        state: &mut Self,
        surface: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: <zwlr_layer_surface_v1::ZwlrLayerSurfaceV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        if let zwlr_layer_surface_v1::Event::Configure { serial, .. } = event {
            surface.ack_configure(serial);
            let Some(LayerSurfaceInfo {
                wl_surface, buffer, ..
            }) = state.wl_surfaces.iter().find(|info| info.layer == *surface)
            else {
                return;
            };
            wl_surface.attach(Some(buffer), 0, 0);
            wl_surface.commit();
        }
    }
}

impl Dispatch<zxdg_output_v1::ZxdgOutputV1, ()> for SecondState {
    fn event(
        state: &mut Self,
        proxy: &zxdg_output_v1::ZxdgOutputV1,
        event: <zxdg_output_v1::ZxdgOutputV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        let Some(info) = state
            .zxdgoutputs
            .iter_mut()
            .find(|info| info.zxdgoutput == *proxy)
        else {
            return;
        };
        match event {
            zxdg_output_v1::Event::LogicalSize { width, height } => {
                info.height = height;
                info.width = width;
            }
            zxdg_output_v1::Event::LogicalPosition { x, y } => {
                info.start_x = x;
                info.start_y = y;
            }
            zxdg_output_v1::Event::Name { name } => info.name = name,
            zxdg_output_v1::Event::Description { description } => info.description = description,
            _ => {}
        }
    }
}

delegate_noop!(SecondState: ignore WlCompositor); // WlCompositor is need to create a surface
delegate_noop!(SecondState: ignore WlSurface); // surface is the base needed to show buffer
                                               //
delegate_noop!(SecondState: ignore WlShm); // shm is used to create buffer pool
delegate_noop!(SecondState: ignore XdgToplevel); // so it is the same with layer_shell, private a
                                                 // place for surface
delegate_noop!(SecondState: ignore WlShmPool); // so it is pool, created by wl_shm
delegate_noop!(SecondState: ignore WlBuffer); // buffer show the picture
delegate_noop!(SecondState: ignore ZwlrLayerShellV1); // it is simillar with xdg_toplevel, also the
                                                      // ext-session-shell
delegate_noop!(SecondState: ignore ZxdgOutputManagerV1);

delegate_noop!(SecondState: ignore WpCursorShapeManagerV1);
delegate_noop!(SecondState: ignore WpCursorShapeDeviceV1);

fn get_cursor_buffer(connection: &Connection, shm: &WlShm) -> Option<CursorImageBuffer> {
    let mut cursor_theme = CursorTheme::load(connection, shm.clone(), 23).ok()?;
    let mut cursor = cursor_theme.get_cursor("crosshair");
    if cursor.is_none() {
        cursor = cursor_theme.get_cursor("left_ptr");
    }
    Some(cursor?[0].clone())
}

/// get the selected area
pub fn get_area(kind: WaySipKind) -> Result<Option<AreaInfo>, WaySipError> {
    let connection =
        Connection::connect_to_env().map_err(|e| WaySipError::InitFailed(e.to_string()))?;
    let (globals, _) = registry_queue_init::<BaseState>(&connection)
        .map_err(|e| WaySipError::InitFailed(e.to_string()))?; // We just need the
                                                               // global, the
                                                               // event_queue is
                                                               // not needed, we
                                                               // do not need
                                                               // BaseState after
                                                               // this anymore

    let mut state = SecondState::new(kind);

    let mut event_queue = connection.new_event_queue::<SecondState>();
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
        return Err(WaySipError::NotGetCursorTheme);
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
        state.zxdgoutputs.push(ZXdgOutputInfo::new(zwloutput));
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
        state.wl_surfaces.push(LayerSurfaceInfo {
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
