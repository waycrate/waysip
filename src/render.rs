use cairo::{Context, Format};
use memmap2::MmapMut;
use std::fs::File;

use super::LayerSurfaceInfo;

impl LayerSurfaceInfo {
    pub fn redraw(
        &mut self,
        (start_pos_x, start_pos_y): (f64, f64),
        (endpos_x, endpos_y): (f64, f64),
        (start_x, start_y): (i32, i32),
        (width, height): (i32, i32),
    ) {
        let cairoinfo = &self.cairo_t;
        cairoinfo.set_operator(cairo::Operator::Source);
        cairoinfo.set_source_rgba(0.4_f64, 0.4_f64, 0.4_f64, 0.4);
        cairoinfo.paint().unwrap();

        let relate_start_x = start_pos_x - start_x as f64;
        let relate_start_y = start_pos_y - start_y as f64;
        let relate_end_x = endpos_x - start_x as f64;
        let relate_end_y = endpos_y - start_y as f64;
        let rlwidth = relate_end_x - relate_start_x;
        let rlheight = relate_end_y - relate_start_y;

        let start_x = relate_start_x;
        let start_y = relate_start_y;

        cairoinfo.rectangle(start_x, start_y, rlwidth, rlheight);
        cairoinfo.set_source_rgba(0.1, 0.1, 0.1, 0.4);
        cairoinfo.fill().unwrap();

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
