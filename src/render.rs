use cairo::Format;
use memmap2::MmapMut;
use std::fs::File;
// FIXME: this wll never be right
pub fn draw_ui(tmp: &mut File, (width, height): (i32, i32)) {
    let cairo_fmt = Format::ARgb32;
    let stride = cairo_fmt.stride_for_width(width as u32).unwrap();
    tmp.set_len((stride * height) as u64).unwrap();
    let mmmap: MmapMut = unsafe { MmapMut::map_mut(&*tmp).unwrap() };

    let surface =
        cairo::ImageSurface::create_for_data(mmmap, cairo_fmt, width, height, stride).unwrap();
    //cairo::ImageSurface::create(format, width, height)
    let cr = cairo::Context::new(&surface).unwrap();
    cr.set_source_rgba(0.4_f64, 0.4_f64, 0.4_f64, 0.4);
    cr.paint().unwrap();
}
