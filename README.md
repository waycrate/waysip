# Waysip

it is a crate like slurp, to select area on wayland, which support layershell.

usage

```rust
use libwaysip::{get_area, WaySipKind};
fn main() {
    println!("{:?}", get_area(WaySipKind::Area));
}
```
