use libwayshot::WayshotConnection;
use libwaysip::gui_selector::{AreaSelectorGUI, GUISelection};
use wayland_client::Connection;

fn main() {
    let connection =
        WayshotConnection::from_connection(Connection::connect_to_env().unwrap()).unwrap();

    let selector = AreaSelectorGUI::new().with_connection(connection);
    match selector.launch() {
        GUISelection::Output(output) => println!(
            "Selected output with title {} and positioned in {}",
            output.name, output.logical_region
        ),
        GUISelection::Toplevel(toplevel) => println!(
            "Selected toplevel with title {} and app_id {}",
            toplevel.title, toplevel.app_id
        ),
        GUISelection::Failed => println!("GUI selection failed!"),
    }
}
