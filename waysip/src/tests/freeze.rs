use image::{DynamicImage, Rgba, RgbaImage};

use crate::freeze::image_to_argb_surface;

fn solid_image(width: u32, height: u32, pixel: [u8; 4]) -> DynamicImage {
    let buf = RgbaImage::from_fn(width, height, |_, _| Rgba(pixel));
    DynamicImage::ImageRgba8(buf)
}

#[test]
fn converts_opaque_image() {
    let img = solid_image(2, 2, [10, 20, 30, 255]);
    let surface = image_to_argb_surface(img).unwrap();
    assert_eq!(surface.width(), 2);
    assert_eq!(surface.height(), 2);
}

#[test]
fn zero_sized_image_returns_none() {
    let img = solid_image(0, 0, [0, 0, 0, 0]);
    assert!(image_to_argb_surface(img).is_none());
}

#[test]
fn premultiplies_alpha() {
    let img = solid_image(1, 1, [200, 100, 50, 128]);
    let mut surface = image_to_argb_surface(img).unwrap();
    let bytes = surface.data().unwrap();
    // cairo's ARgb32 stores native-endian premultiplied 0xAARRGGBB.
    let pixel = u32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let a = (pixel >> 24) & 0xff;
    let r = (pixel >> 16) & 0xff;
    let g = (pixel >> 8) & 0xff;
    let b = pixel & 0xff;
    assert_eq!(a, 128);
    assert_eq!(r, 200u32 * 128 / 255);
    assert_eq!(g, 100u32 * 128 / 255);
    assert_eq!(b, 50u32 * 128 / 255);
}

// --- live-compositor test ---
//
// `capture_backgrounds` needs a real `libwayshot::WayshotConnection` to talk
// wlr_screencopy to an actual compositor, so unlike everything else in this
// file, it can't be exercised with a fake/inert proxy. CI starts a headless
// wlroots compositor (see `.github/workflows/test-coverage.yml`) and points
// `WAYLAND_DISPLAY` at it before running tests; locally or in any other CI
// job there's no compositor, so this skips itself at runtime instead of
// failing.

#[test]
fn capture_backgrounds_returns_image_for_a_real_output() {
    if std::env::var_os("WAYLAND_DISPLAY").is_none() {
        eprintln!("skipping: no WAYLAND_DISPLAY set (requires a live compositor)");
        return;
    }

    let conn = libwayshot::WayshotConnection::new().expect("should connect to the CI compositor");
    let output = conn.get_all_outputs()[0].clone();

    let provider =
        crate::freeze::capture_backgrounds().expect("should init the screenshot backend");
    let surface = provider(&output.wl_output, &output.name)
        .expect("should capture a background for a real output");

    assert!(surface.width() > 0);
    assert!(surface.height() > 0);
}
