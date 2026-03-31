use libwaysip::{AreaInfo, Position, Size};

pub(crate) fn apply_format(info: &AreaInfo, fmt: &str, screen: bool) -> String {
    let screen_info = info.selected_screen_info();
    let Position { x: sx, y: sy } = screen_info.get_position();
    let Size {
        width: sw,
        height: sh,
    } = screen_info.get_size();
    let Size {
        width: wl_w,
        height: wl_h,
    } = screen_info.get_wloutput_size();

    let (x, y, width, height) = if !screen {
        let Position { x, y } = info.left_top_point();
        let w = info.width().max(1);
        let h = info.height().max(1);
        (x, y, w, h)
    } else {
        let w = sw.max(1);
        let h = sh.max(1);
        (sx, sy, w, h)
    };

    let rel_x = x.saturating_sub(sx);
    let rel_y = y.saturating_sub(sy);
    let rel_width = width.min(sw.saturating_sub(rel_x));
    let rel_height = height.min(sh.saturating_sub(rel_y));

    let out_name = screen_info.get_name();
    let out_description = screen_info.get_description();

    let mut out = String::with_capacity(fmt.len() * 2);
    let mut chars = fmt.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            match chars.next().unwrap_or('%') {
                '%' => out.push('%'),
                'x' => out.push_str(&x.to_string()),
                'y' => out.push_str(&y.to_string()),
                'w' => out.push_str(&width.to_string()),
                'h' => out.push_str(&height.to_string()),
                'X' => out.push_str(&rel_x.to_string()),
                'Y' => out.push_str(&rel_y.to_string()),
                'W' => out.push_str(&rel_width.to_string()),
                'H' => out.push_str(&rel_height.to_string()),
                'o' => out.push_str(out_name),
                'l' => out.push_str(out_name),
                'd' => out.push_str(out_description),
                // Length
                'L' => out.push_str(&wl_w.to_string()),
                // Tall
                'T' => out.push_str(&wl_h.to_string()),
                other => out.push(other),
            }
        } else if c == '\\' {
            match chars.next().unwrap_or('\\') {
                '\\' => out.push('\\'),
                'n' => out.push('\n'),
                other => out.push(other),
            }
        } else {
            out.push(c);
        }
    }
    out
}
