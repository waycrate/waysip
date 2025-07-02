use crate::error::ColorError;

/// Describe the point
#[derive(Debug, Copy, Clone)]
pub struct Position<T = i32> {
    pub x: T,
    pub y: T,
}

/// Describe the size
#[derive(Debug, Copy, Clone)]
pub struct Size<T = i32> {
    pub width: T,
    pub height: T,
}

impl<T> From<(T, T)> for Size<T>
where
    T: Copy,
{
    fn from(value: (T, T)) -> Self {
        Self {
            width: value.0,
            height: value.1,
        }
    }
}

/// Describe the color of waysip
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Default for Color {
    fn default() -> Self {
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.5,
        }
    }
}

impl Color {
    pub fn hex_to_color(colorhex: String) -> Result<Color, ColorError> {
        let stripped_color = colorhex.trim_start_matches('#');

        if stripped_color.len() != 8 || !stripped_color.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ColorError::InvalidColorFormat(colorhex.to_string()));
        }
        let color = Color {
            r: u8::from_str_radix(&stripped_color[0..2], 16)? as f64 / 255.0,
            g: u8::from_str_radix(&stripped_color[2..4], 16)? as f64 / 255.0,
            b: u8::from_str_radix(&stripped_color[4..6], 16)? as f64 / 255.0,
            a: u8::from_str_radix(&stripped_color[6..8], 16)? as f64 / 255.0,
        };
        Ok(color)
    }
}

/// Current style of the info
#[derive(Debug, Clone)]
pub struct Style {
    pub background_color: Color,
    pub foreground_color: Color,
    pub border_text_color: Color,
    pub box_color: Color,
    pub border_weight: f64,
    pub font_size: i32,
    pub font_name: String,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            background_color: Color {
                r: 0.4,
                g: 0.4,
                b: 0.4,
                a: 0.5,
            }, // #66666680
            foreground_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            }, // #00000000
            border_text_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }, // #000000ff
            box_color: Color {
                r: 0.4,
                g: 0.4,
                b: 0.4,
                a: 0.5,
            }, // #66666680
            border_weight: 1.0,
            font_size: 12,
            font_name: "Sans".to_string(),
        }
    }
}
