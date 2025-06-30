use libwaysip::{Color, SelectionType, WaySip};
use wayland_client::{
    Connection, Dispatch, QueueHandle, globals::GlobalListContents, protocol::wl_registry,
};

struct State {}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for State {
    fn event(
        _: &mut State,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
    }
}

fn main() {
    let connection = Connection::connect_to_env().unwrap();

    println!(
        "{:?}",
        WaySip::new()
            .with_connection(connection)
            .with_selection_type(SelectionType::Area)
            .with_background_color(Color::default())
            .with_foreground_color(Color::default())
            .with_border_text_color(Color::default())
            .with_box_color(Color::default())
            .with_border_weight(2.0)
            .with_font_size(12)
            .with_font_name("Sans".to_string())
            .with_predefined_boxes(Vec::new())
            .with_aspect_ratio(16.0, 9.0)
            .get()
    );
}
