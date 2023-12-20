use libwaysip::{WaySipKind, WaysipEv};
fn main() {
    let ev = WaysipEv::new().unwrap();
    println!("{:?}", ev.get_area(WaySipKind::Area));
}
