use cairo::{Context, Format};
use memmap2::MmapMut;
use std::fs::File;

use super::LayerSurfaceInfo;

#[allow(unused)]
impl LayerSurfaceInfo {
    pub fn redraw(
        &mut self,
        (pos_x, pos_y): (f64, f64),
        (start_x, start_y): (i32, i32),
        (width, height): (i32, i32),
    ) {
        let cairoinfo = &self.cairo_t;
        cairoinfo.set_source_rgba(0.7_f64, 0.7f64, 0.4_f64, 0.4);
        cairoinfo.paint().unwrap();

        self.wl_surface.attach(Some(&self.buffer), 0, 0);
        self.wl_surface.damage(0, 0, width, height);
        self.wl_surface.commit();
    }
}

pub fn draw_ui(tmp: &mut File, (width, height): (i32, i32)) -> Context {
    let cairo_fmt = Format::ARgb32;
    let stride = cairo_fmt.stride_for_width(width as u32).unwrap();
    tmp.set_len((stride * height) as u64).unwrap();
    let mmmap: MmapMut = unsafe { MmapMut::map_mut(&*tmp).unwrap() };

    let surface =
        cairo::ImageSurface::create_for_data(mmmap, cairo_fmt, width, height, stride).unwrap();
    let cairoinfo = cairo::Context::new(&surface).unwrap();
    cairoinfo.set_source_rgba(0.4_f64, 0.4_f64, 0.4_f64, 0.4);
    cairoinfo.paint().unwrap();
    cairoinfo
}
