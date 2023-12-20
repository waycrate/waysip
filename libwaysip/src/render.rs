use cairo::{Context, Format};
use memmap2::MmapMut;
use std::fs::File;

use super::LayerSurfaceInfo;

const FONT_FAMILY: &str = "Sans";
const FONT_SIZE: i32 = 20;

impl LayerSurfaceInfo {
    pub fn redraw_select_screen(
        &self,
        is_selected: bool,
        (width, height): (i32, i32),
        (start_x, start_y): (i32, i32),
        name: &str,
        descriptin: &str,
    ) {
        let cairoinfo = &self.cairo_t;
        cairoinfo.set_operator(cairo::Operator::Source);
        let color = if is_selected { 0. } else { 0.4 };
        let alpha = if is_selected { 0. } else { 0.5 };
        cairoinfo.set_source_rgba(color, color, color, alpha);
        cairoinfo.paint().unwrap();

        cairoinfo.set_source_rgb(0.3_f64, 0_f64, 0_f64);

        let font_size = FONT_SIZE;
        let pangolayout = pangocairo::create_layout(cairoinfo);
        let mut desc = pango::FontDescription::new();
        desc.set_family(FONT_FAMILY);
        desc.set_weight(pango::Weight::Bold);

        desc.set_size(font_size * pango::SCALE);
        pangolayout.set_font_description(Some(&desc));

        let name_txt = format!("{name}  {descriptin}");
        pangolayout.set_text(&name_txt);
        cairoinfo.save().unwrap();
        cairoinfo.move_to(10., 60.);
        pangocairo::show_layout(cairoinfo, &pangolayout);
        cairoinfo.restore().unwrap();

        let pos_txt = format!("pos: {start_x}, {start_y}");
        pangolayout.set_text(&pos_txt);
        cairoinfo.save().unwrap();
        cairoinfo.move_to(10., 90.);
        pangocairo::show_layout(cairoinfo, &pangolayout);
        cairoinfo.restore().unwrap();

        self.wl_surface.attach(Some(&self.buffer), 0, 0);
        self.wl_surface.damage(0, 0, width, height);
        self.wl_surface.commit();
    }
    pub fn redraw(
        &self,
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

        cairoinfo.set_source_rgb(1_f64, 1_f64, 1_f64);

        let font_size = FONT_SIZE;
        let pangolayout = pangocairo::create_layout(cairoinfo);
        let mut desc = pango::FontDescription::new();
        desc.set_family(FONT_FAMILY);
        desc.set_weight(pango::Weight::Normal);

        desc.set_size(font_size * pango::SCALE);
        pangolayout.set_font_description(Some(&desc));

        let text = format!(
            "{},{}, {}x{}",
            start_pos_x as i32,
            start_pos_y as i32,
            (endpos_x - start_pos_x) as i32,
            (endpos_y - start_pos_y) as i32
        );

        pangolayout.set_text(text.as_str());
        cairoinfo.save().unwrap();
        cairoinfo.move_to(relate_end_x + 10., relate_end_y + 10.);
        pangocairo::show_layout(cairoinfo, &pangolayout);
        cairoinfo.restore().unwrap();

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
