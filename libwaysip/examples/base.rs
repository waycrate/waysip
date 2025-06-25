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

    println!(
        "{:?}",
        WaySip::new()
            .with_connection(connection)
            .with_selection_type(SelectionType::Area)
            .get()
    );
}
