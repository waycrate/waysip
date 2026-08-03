//! Captures a screenshot of every output (via `libwayshot`) so it can be
//! used as the static backdrop for the selection overlay, giving the effect
//! of a frozen screen while selecting.

use std::collections::HashMap;

use wayland_client::protocol::wl_output::{Transform, WlOutput};

/// Takes a screenshot of every output up front and returns a
/// [`libwaysip::BackgroundProvider`]-compatible closure that looks up the
/// pre-captured image for a given output by name.
///
/// Returns `None` (falling back to the normal, non-frozen overlay) if the
/// screenshot backend itself couldn't be initialised.
pub fn capture_backgrounds()
-> Option<impl Fn(&WlOutput, &str) -> Option<cairo::ImageSurface> + 'static> {
    let wayshot_conn = match libwayshot::WayshotConnection::new() {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Warning: freeze: failed to initialize screenshot backend: {e}");
            return None;
        }
    };

    let mut backgrounds: HashMap<String, cairo::ImageSurface> = HashMap::new();
    for output_info in wayshot_conn.get_all_outputs() {
        let image = match wayshot_conn.screenshot_single_output(output_info, false) {
            Ok(image) => image,
            Err(e) => {
                eprintln!(
                    "Warning: freeze: failed to capture output {}: {e}",
                    output_info.name
                );
                continue;
            }
        };
        // `screenshot_single_output` returns the raw, un-rotated buffer
        // (always "landscape"); apply the output's transform ourselves so
        // portrait/rotated outputs aren't rendered squashed or sideways.
        let image = rotate_image(image, output_info.transform);
        match image_to_argb_surface(image) {
            Some(surface) => {
                backgrounds.insert(output_info.name.clone(), surface);
            }
            None => {
                eprintln!(
                    "Warning: freeze: failed to convert screenshot for output {}",
                    output_info.name
                );
            }
        }
    }

    Some(move |_wl_output: &WlOutput, name: &str| backgrounds.get(name).cloned())
}

/// Rotates/flips a raw screencopy buffer to account for the output's
/// transform.
fn rotate_image(image: image::DynamicImage, transform: Transform) -> image::DynamicImage {
    match transform {
        Transform::_90 => image::imageops::rotate90(&image).into(),
        Transform::_180 => image::imageops::rotate180(&image).into(),
        Transform::_270 => image::imageops::rotate270(&image).into(),
        Transform::Flipped => image::imageops::flip_horizontal(&image).into(),
        Transform::Flipped90 => {
            image::imageops::rotate90(&image::imageops::flip_horizontal(&image)).into()
        }
        Transform::Flipped180 => {
            image::imageops::rotate180(&image::imageops::flip_horizontal(&image)).into()
        }
        Transform::Flipped270 => {
            image::imageops::rotate270(&image::imageops::flip_horizontal(&image)).into()
        }
        _ => image,
    }
}

/// Converts an [`image::DynamicImage`] into a premultiplied-alpha
/// `cairo::ImageSurface` (`ARgb32`) suitable for use as a paint source.
fn image_to_argb_surface(image: image::DynamicImage) -> Option<cairo::ImageSurface> {
    let rgba = image.to_rgba8();
    let width = rgba.width() as i32;
    let height = rgba.height() as i32;
    if width == 0 || height == 0 {
        return None;
    }

    let stride = cairo::Format::ARgb32.stride_for_width(width as u32).ok()?;
    let src = rgba.as_raw();
    let mut data = vec![0u8; stride as usize * height as usize];

    for y in 0..height as usize {
        let row = y * stride as usize;
        for x in 0..width as usize {
            let si = (y * width as usize + x) * 4;
            let r = src[si] as u32;
            let g = src[si + 1] as u32;
            let b = src[si + 2] as u32;
            let a = src[si + 3] as u32;
            // cairo's ARgb32 stores premultiplied-alpha pixels.
            let pr = r * a / 255;
            let pg = g * a / 255;
            let pb = b * a / 255;
            let pixel = (a << 24) | (pr << 16) | (pg << 8) | pb;

            let di = row + x * 4;
            data[di..di + 4].copy_from_slice(&pixel.to_ne_bytes());
        }
    }

    cairo::ImageSurface::create_for_data(data, cairo::Format::ARgb32, width, height, stride).ok()
}
