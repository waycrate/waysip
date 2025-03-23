/// Describe the point
#[derive(Debug, Copy, Clone)]
pub struct Point<T = i32> {
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
