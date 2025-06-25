use libwaysip::{SelectionType, WaySip};
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

    let passing_data = libwaysip::state::PassingData {
        background_color: [0.0, 0.0, 0.0, 1.0],
        foreground_color: [1.0, 1.0, 1.0, 1.0],
        border_text_color: [1.0, 1.0, 1.0, 1.0],
        border_size: 2.0,
        font_size: 12,
        font_name: "sans-serif".to_string(),
    };

    println!(
        "{:?}",
        WaySip::new()
            .with_connection(connection)
            .with_selection_type(SelectionType::Area, passing_data)
            .get()
    );
}
