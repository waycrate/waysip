use crate::{
    Position, Size,
    state::{self, LayerSurfaceInfo, WaysipState},
};
use wayland_client::{
    Connection, Dispatch, Proxy, WEnum, delegate_noop,
    globals::GlobalListContents,
    protocol::{
        wl_buffer::{self, WlBuffer},
        wl_callback::{self, WlCallback},
        wl_compositor::WlCompositor,
        wl_keyboard, wl_output, wl_pointer, wl_registry,
        wl_seat::{self},
        wl_shm::WlShm,
        wl_shm_pool::WlShmPool,
        wl_surface::WlSurface,
    },
};
use wayland_protocols::{
    wp::cursor_shape::v1::client::{
        wp_cursor_shape_device_v1::{self, WpCursorShapeDeviceV1},
        wp_cursor_shape_manager_v1::WpCursorShapeManagerV1,
    },
    xdg::{
        shell::client::{xdg_toplevel::XdgToplevel, xdg_wm_base},
        xdg_output::zv1::client::{zxdg_output_manager_v1::ZxdgOutputManagerV1, zxdg_output_v1},
    },
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::ZwlrLayerShellV1,
    zwlr_layer_surface_v1::{self},
};

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for WaysipState {
    fn event(
        state: &mut Self,
        surface: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: <zwlr_layer_surface_v1::ZwlrLayerSurfaceV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        if let zwlr_layer_surface_v1::Event::Configure {
            serial,
            width,
            height,
        } = event
        {
            surface.ack_configure(serial);

            state.ensure_buffer(surface, (width, height));
            state.ensure_init(surface);
            state.redraw(surface);
        }
    }
}

impl Dispatch<zxdg_output_v1::ZxdgOutputV1, ()> for WaysipState {
    fn event(
        state: &mut Self,
        proxy: &zxdg_output_v1::ZxdgOutputV1,
        event: <zxdg_output_v1::ZxdgOutputV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        let Some(info) = state
            .wloutput_infos
            .iter_mut()
            .find(|info| info.zxdg_output() == proxy)
        else {
            return;
        };
        let info = info.xdg_output_info_mut();
        match event {
            zxdg_output_v1::Event::LogicalSize { width, height } => {
                info.size = Size { width, height }
            }
            zxdg_output_v1::Event::LogicalPosition { x, y } => {
                info.start_position = Position { x, y }
            }
            zxdg_output_v1::Event::Name { name } => info.name = name,
            zxdg_output_v1::Event::Description { description } => info.description = description,
            _ => {}
        }
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for state::WaysipState {
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

impl Dispatch<wl_registry::WlRegistry, ()> for state::WaysipState {
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
            state.wloutput_infos.push(state::WlOutputInfo::new(output));
        }
    }
}

impl Dispatch<wl_output::WlOutput, ()> for state::WaysipState {
    fn event(
        state: &mut Self,
        wl_output: &wl_output::WlOutput,
        event: <wl_output::WlOutput as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        let output = state
            .wloutput_infos
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
                output.size = Size { width, height };
            }

            _ => (),
        }
    }
}

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for state::WaysipState {
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

impl Dispatch<wl_seat::WlSeat, ()> for state::WaysipState {
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

impl Dispatch<wl_keyboard::WlKeyboard, ()> for state::WaysipState {
    fn event(
        state: &mut Self,
        _proxy: &wl_keyboard::WlKeyboard,
        event: <wl_keyboard::WlKeyboard as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_keyboard::Event::Key { key: 1, .. } = event {
            state.running = false;
        }
    }
}

impl Dispatch<wl_pointer::WlPointer, ()> for state::WaysipState {
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
                        if !dispatch_state.is_predefined_boxes() {
                            dispatch_state.start_pos = Some(dispatch_state.current_pos);
                        }
                        if !dispatch_state.is_area() && !dispatch_state.is_predefined_boxes() {
                            dispatch_state.end_pos = Some(dispatch_state.current_pos);
                            dispatch_state.running = false;
                        }
                    }
                    WEnum::Value(wl_pointer::ButtonState::Released) => {
                        if !dispatch_state.is_predefined_boxes() {
                            dispatch_state.end_pos = Some(dispatch_state.current_pos);
                        }
                        dispatch_state.running = false;
                    }
                    _ => {}
                }
                dispatch_state.commit();
            }
            wl_pointer::Event::Enter {
                serial,
                surface,
                surface_x,
                surface_y,
            } => {
                let Some(LayerSurfaceInfo {
                    cursor_surface,
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
                let info =
                    dispatch_state.wloutput_infos[dispatch_state.current_screen].xdg_output_info();
                let start_x = info.start_position.x;
                let start_y = info.start_position.y;
                dispatch_state.current_pos = Position {
                    x: surface_x + start_x as f64,
                    y: surface_y + start_y as f64,
                };

                if let Some(ref cursor_manager) = dispatch_state.cursor_manager {
                    let device = cursor_manager.get_pointer(pointer, qh, ());
                    device.set_shape(serial, wp_cursor_shape_device_v1::Shape::Crosshair);
                    device.destroy();
                } else {
                    let cursor_buffer = cursor_buffer.as_ref().unwrap();
                    cursor_surface.attach(Some(cursor_buffer), 0, 0);
                    let (hotspot_x, hotspot_y) = cursor_buffer.hotspot();
                    pointer.set_cursor(
                        serial,
                        Some(cursor_surface),
                        hotspot_x as i32,
                        hotspot_y as i32,
                    );
                    cursor_surface.commit();
                }

                dispatch_state.commit();
            }
            wl_pointer::Event::Motion {
                surface_x,
                surface_y,
                ..
            } => {
                let info =
                    dispatch_state.wloutput_infos[dispatch_state.current_screen].xdg_output_info();
                let start_x = info.start_position.x;
                let start_y = info.start_position.y;
                dispatch_state.current_pos = Position {
                    x: surface_x + start_x as f64,
                    y: surface_y + start_y as f64,
                };
                dispatch_state.end_pos = None;
                if dispatch_state.is_area() {
                    if let Some(ratio) = dispatch_state.aspect_ratio {
                        let width_rel = ratio.0;
                        let height_rel = ratio.1;
                        let start_pos = dispatch_state
                            .start_pos
                            .unwrap_or(dispatch_state.current_pos);
                        let width = dispatch_state.current_pos.x - start_pos.x;
                        let height = dispatch_state.current_pos.y - start_pos.y;
                        if width_rel / height_rel > width / height {
                            dispatch_state.end_pos = Some(Position {
                                x: start_pos.x + height * width_rel / height_rel,
                                y: start_pos.y + height,
                            });
                        } else {
                            dispatch_state.end_pos = Some(Position {
                                x: start_pos.x + width,
                                y: start_pos.y + width * height_rel / width_rel,
                            });
                        }
                    } else {
                        dispatch_state.end_pos = Some(dispatch_state.current_pos);
                    }

                    let now = std::time::Instant::now();
                    if now.duration_since(dispatch_state.last_redraw)
                        >= std::time::Duration::from_millis(8)
                    {
                        dispatch_state.commit();
                        dispatch_state.last_redraw = now;
                    }
                } else if dispatch_state.is_predefined_boxes() {
                    let current_pos = dispatch_state.current_pos;
                    if let Some(box_info) = dispatch_state
                        .predefined_boxes
                        .as_ref()
                        .unwrap()
                        .iter()
                        .find(|box_info| {
                            current_pos.x >= box_info.start_x
                                && current_pos.x <= box_info.end_x
                                && current_pos.y >= box_info.start_y
                                && current_pos.y <= box_info.end_y
                        })
                    {
                        dispatch_state.end_pos = Some(dispatch_state.current_pos);
                        dispatch_state.start_pos = Some(Position {
                            x: box_info.start_x,
                            y: box_info.start_y,
                        });
                        dispatch_state.end_pos = Some(Position {
                            x: box_info.end_x,
                            y: box_info.end_y,
                        });
                    }
                    let now = std::time::Instant::now();
                    if now.duration_since(dispatch_state.last_redraw)
                        >= std::time::Duration::from_millis(20)
                    // no need to redraw faster as boxes are not moving
                    {
                        dispatch_state.commit();
                        dispatch_state.last_redraw = now;
                    }
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<WlCallback, usize> for state::WaysipState {
    fn event(
        state: &mut Self,
        _proxy: &WlCallback,
        event: <WlCallback as Proxy>::Event,
        screen_index: &usize,
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_callback::Event::Done { .. } = event {
            if *screen_index != state.current_screen {
                return;
            }
            state.redraw_current_surface();
        }
    }
}

impl Dispatch<WlBuffer, ()> for state::WaysipState {
    fn event(
        state: &mut Self,
        buffer: &WlBuffer,
        event: <WlBuffer as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_buffer::Event::Release = event {
            let Some(info) = state
                .wl_surfaces
                .iter_mut()
                .find(|info| info.buffer == *buffer)
            else {
                return;
            };
            info.buffer_busy = false;
        }
    }
}

delegate_noop!(WaysipState: ignore WlCompositor); // WlCompositor is need to create a surface
delegate_noop!(WaysipState: ignore WlSurface); // surface is the base needed to show buffer
//
delegate_noop!(WaysipState: ignore WlShm); // shm is used to create buffer pool
delegate_noop!(WaysipState: ignore XdgToplevel); // so it is the same with layer_shell, private a
// place for surface
delegate_noop!(WaysipState: ignore WlShmPool); // so it is pool, created by wl_shm
delegate_noop!(WaysipState: ignore ZwlrLayerShellV1); // it is similar with xdg_toplevel, also the
// ext-session-shell
delegate_noop!(WaysipState: ignore ZxdgOutputManagerV1);

delegate_noop!(WaysipState: ignore WpCursorShapeManagerV1);
delegate_noop!(WaysipState: ignore WpCursorShapeDeviceV1);
