# Annotation Mode - Developer Guide

## Code Architecture

### Keybindings Structure

The core of the feature is the `Keybindings` struct in `libwaysip/src/state.rs`:

```rust
#[derive(Debug, Clone)]
pub struct Keybindings {
    pub move_left: u32,
    pub move_right: u32,
    pub move_up: u32,
    pub move_down: u32,
    pub confirm: u32,
    pub cancel: u32,
}

impl Default for Keybindings {
    fn default() -> Self {
        Self {
            move_left: 30,   // A key
            move_right: 32,  // D key
            move_up: 17,     // W key
            move_down: 31,   // S key
            confirm: 28,     // Enter key
            cancel: 1,       // Escape key
        }
    }
}
```

### Integration with WaysipState

The `WaysipState` tracks annotation mode and keybindings:

```rust
pub struct WaysipState {
    // ... other fields ...
    pub annotation_mode: bool,
    pub keybindings: Keybindings,
}

impl WaysipState {
    pub fn new(selection_type: SelectionType) -> Self {
        WaysipState {
            // ... initialization ...
            annotation_mode: false,
            keybindings: Keybindings::default(),
        }
    }
}
```

### Keyboard Event Handling

The dispatch handler uses keybindings from state:

```rust
impl Dispatch<wl_keyboard::WlKeyboard, ()> for state::WaysipState {
    fn event(state: &mut Self, /* ... */) {
        if let wl_keyboard::Event::Key { key, state: key_state, .. } = event {
            if key_state != WEnum::Value(wl_keyboard::KeyState::Pressed) {
                return;
            }
            
            let keybindings = state.keybindings.clone();
            
            if state.annotation_mode {
                if key == keybindings.move_left {
                    // Adjust selection left
                    if let Some(ref mut start) = state.start_pos {
                        start.x -= 1.0;
                    }
                    if let Some(ref mut end) = state.end_pos {
                        end.x -= 1.0;
                    }
                    state.redraw_current_surface();
                    state.commit();
                }
                // ... similar for other directions ...
            }
        }
    }
}
```

### WaySip Builder API

Use the builder pattern to configure keybindings:

```rust
let custom_keybindings = Keybindings {
    move_left: 105,      // Left arrow
    move_right: 106,     // Right arrow
    move_up: 103,        // Up arrow
    move_down: 108,      // Down arrow
    confirm: 28,         // Enter
    cancel: 1,           // Escape
};

let area = WaySip::new()
    .with_selection_type(SelectionType::Annotation)
    .with_keybindings(custom_keybindings)
    .get()?;
```

## Linux Key Codes Reference

Common keycodes used in annotation mode:

```
1      Escape
17     W
28     Return/Enter
30     A
31     S
32     D
57     Space
103    Up Arrow
105    Left Arrow
106    Right Arrow
108    Down Arrow
111    Delete
113    Minus
121    Equals
```

## Extending the Feature

### Adding New Actions in Annotation Mode

To add a new action (e.g., grow/shrink selection):

1. **Add to Keybindings struct** (state.rs):
```rust
pub struct Keybindings {
    // ... existing fields ...
    pub grow: u32,
    pub shrink: u32,
}

impl Default for Keybindings {
    fn default() -> Self {
        Self {
            // ... existing bindings ...
            grow: 61,      // = key
            shrink: 121,   // - key
        }
    }
}
```

2. **Add CLI argument** (main.rs):
```rust
#[derive(Parser)]
struct Args {
    // ... existing fields ...
    /// Custom keybindings
    #[arg(long, value_name = "keybindings")]
    keybindings: Option<String>,
}
```

3. **Handle in keyboard event** (dispatch.rs):
```rust
if key == keybindings.grow {
    if let Some(ref mut end) = state.end_pos {
        end.x += 5.0;
        end.y += 5.0;
    }
    state.redraw_current_surface();
    state.commit();
}
```

### Parsing Custom Keybindings from String

To support custom keybindings from command-line arguments:

```rust
fn parse_keybindings(kb_string: &str) -> Result<Keybindings, String> {
    let mut kb = Keybindings::default();
    
    for pair in kb_string.split(',') {
        let (key, code) = pair.split_once(':')
            .ok_or_else(|| "Invalid format".to_string())?;
        let code: u32 = code.parse()
            .map_err(|_| "Invalid keycode".to_string())?;
        
        match key {
            "left" => kb.move_left = code,
            "right" => kb.move_right = code,
            "up" => kb.move_up = code,
            "down" => kb.move_down = code,
            "confirm" => kb.confirm = code,
            "cancel" => kb.cancel = code,
            _ => return Err(format!("Unknown action: {}", key)),
        }
    }
    
    Ok(kb)
}
```

## Testing Annotation Mode

### Manual Testing Checklist

- [ ] Default keybindings work (W/A/S/D)
- [ ] Selection moves in correct direction for each key
- [ ] Multiple keys pressed sequentially move selection multiple times
- [ ] Enter confirms selection and outputs correct coordinates
- [ ] Escape cancels without outputting
- [ ] Custom keybindings via `--keybindings` flag work
- [ ] Selection doesn't go outside screen boundaries
- [ ] Redraw properly updates visual feedback
- [ ] Works with color customization flags
- [ ] Output format options respected

### Integration Testing

```bash
# Test default annotation mode
waysip -A

# Test with custom colors
waysip -A -b "00000080" -c "FF0000FF"

# Test with custom keybindings
waysip -A --keybindings "left:105,right:106,up:103,down:108,confirm:28,cancel:1"

# Test output format
waysip -A -f "%x,%y %wx%h\n"
```

## Performance Considerations

1. **Keybindings Clone**: Keybindings are cloned in keyboard handler for thread safety
2. **Redraw Optimization**: Redraw is only called when position actually changes
3. **Event Queue**: Keyboard events are processed through the standard event queue

## Future Enhancements

1. **Config File Support**: Load keybindings from `~/.config/waysip/keybindings.toml`
2. **Movement Speed**: Allow adjustable pixel per keypress (currently 1 pixel)
3. **Selection Modes**: Different modes (move, resize, rotate) switchable via keys
4. **Visual Feedback**: Display current selection dimensions while in annotation mode
5. **Macro Support**: Repeat last action with a key combination
