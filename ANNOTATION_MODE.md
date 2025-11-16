# Annotation Mode

Annotation mode allows you to select an area and then fine-tune the selection using keyboard controls before finalizing the selection.

## Usage

Enable annotation mode with the `-A` flag:

```bash
waysip -A
```

## Default Keybindings

Once you've made an initial selection by dragging on the screen, you enter annotation mode where you can adjust your selection:

- **W**: Move selection up
- **A**: Move selection left
- **S**: Move selection down
- **D**: Move selection right
- **Enter**: Confirm and finalize the selection
- **Escape**: Cancel and exit

## Custom Keybindings

You can customize the keybindings using the `--keybindings` flag with the format:

```bash
waysip -A --keybindings "left:key,right:key,up:key,down:key,confirm:key,cancel:key"
```

Where `key` is the Linux kernel keycode (e.g., 30 for A, 32 for D, 17 for W, 31 for S).

### Example with Custom Keybindings

```bash
# Use arrow keys instead of WASD
waysip -A --keybindings "left:105,right:106,up:103,down:108,confirm:28,cancel:1"
```

Common keycodes:
- 1: Escape
- 17: W
- 28: Enter
- 30: A
- 31: S
- 32: D
- 103: Up Arrow
- 105: Left Arrow
- 106: Right Arrow
- 108: Down Arrow

## How It Works

1. Start waysip in annotation mode: `waysip -A`
2. Click and drag to select an initial area (or click once for a point)
3. After releasing the mouse, the selection enters annotation mode
4. Use keyboard controls to adjust the selection boundaries
5. Press Enter to confirm or Escape to cancel
6. The final selection coordinates are output to stdout in the specified format

## Use Cases

- **Precise screenshots**: Select an area roughly, then fine-tune the exact boundaries
- **Scripting**: Combine with output processing for automated area selection workflows
- **Accessibility**: Allow keyboard-only selection after initial mouse input

## Combining with Other Modes

Annotation mode works independently and cannot be combined with:
- Point selection (`-p`)
- Screen selection (`-o`, `-i`)
- Predefined boxes (`-r`)

You can still use other customization options like colors (`-b`, `-c`, `-s`), fonts (`-F`, `-S`), and output format (`-f`).
