use libwaysip::{get_area, SelectionType};
use wayland_client::{
    globals::{registry_queue_init, GlobalListContents},
    protocol::wl_registry,
    Connection, Dispatch, QueueHandle,
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

    println!("{:?}", get_area(&connection, globals, SelectionType::Area));
}
