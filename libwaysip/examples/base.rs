use libwaysip::{SelectionType, WaysipConnection, get_area};
use wayland_client::{
    Connection, Dispatch, QueueHandle,
    globals::{GlobalListContents, registry_queue_init},
    protocol::wl_registry,
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
    let (globals, _) = registry_queue_init::<State>(&connection).unwrap();

    println!(
        "{:?}",
        get_area(
            Some(WaysipConnection {
                connection: &connection,
                globals: &globals
            }),
            SelectionType::Area
        )
    );
}
