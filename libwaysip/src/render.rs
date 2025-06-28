use super::state::LayerSurfaceInfo;
use crate::{Size, utils::Position};
use cairo::{Context, Format};
use memmap2::MmapMut;
use std::fs::File;

impl LayerSurfaceInfo {
    pub fn init_commit(&self) {
        self.wl_surface.attach(Some(&self.buffer), 0, 0);
        self.wl_surface.commit();
    }

    pub fn redraw_select_screen(
        &self,
        is_selected: bool,
        Size { width, height }: Size,
        Position {
            x: start_x,
            y: start_y,
        }: Position,
        name: &str,
        description: &str,
    ) {
        let cairoinfo = &self.cairo_t;
        cairoinfo.set_operator(cairo::Operator::Source);
        if is_selected {
            cairoinfo.set_source_rgba(
                self.style.foreground_color.r,
                self.style.foreground_color.g,
                self.style.foreground_color.b,
                self.style.foreground_color.a,
            );
        } else {
            cairoinfo.set_source_rgba(
                self.style.background_color.r,
                self.style.background_color.g,
                self.style.background_color.b,
                self.style.background_color.a,
            );
        }
        cairoinfo.paint().unwrap();

        cairoinfo.set_source_rgba(
            self.style.border_text_color.r,
            self.style.border_text_color.g,
            self.style.border_text_color.b,
            self.style.border_text_color.a,
        );

        let font_size = self.style.font_size;
        let pangolayout = self
            .pango_layout
            .get_or_init(|| pangocairo::functions::create_layout(cairoinfo));
        let desc = self.font_desc_normal.get_or_init(|| {
            let mut d = pango::FontDescription::new();
            d.set_family(self.style.font_name.as_str());
            d.set_weight(pango::Weight::Normal);
            d.set_size(font_size * pango::SCALE);
            d
        });
        pangolayout.set_font_description(Some(desc));

        let name_txt = format!("{name}  {description}");
        pangolayout.set_text(&name_txt);
        cairoinfo.save().unwrap();
        cairoinfo.move_to(10., 60.);
        pangocairo::functions::show_layout(cairoinfo, pangolayout);
        cairoinfo.restore().unwrap();

        let pos_txt = format!("pos: {start_x}, {start_y}");
        pangolayout.set_text(&pos_txt);
        cairoinfo.save().unwrap();
        cairoinfo.move_to(10., 90.);
        pangocairo::functions::show_layout(cairoinfo, pangolayout);
        cairoinfo.restore().unwrap();

        self.wl_surface.attach(Some(&self.buffer), 0, 0);
        self.wl_surface.damage(0, 0, width, height);
        self.wl_surface.commit();
    }
    pub fn redraw(
        &self,
        Position {
            x: start_pos_x,
            y: start_pos_y,
        }: Position<f64>,
        Position {
            x: endpos_x,
            y: endpos_y,
        }: Position<f64>,
        Position {
            x: start_x,
            y: start_y,
        }: Position,
        Size { width, height }: Size,
    ) {
        let cairoinfo = &self.cairo_t;
        cairoinfo.set_operator(cairo::Operator::Source);
        cairoinfo.set_source_rgba(
            self.style.background_color.r,
            self.style.background_color.g,
            self.style.background_color.b,
            self.style.background_color.a,
        );
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
        cairoinfo.set_source_rgba(
            self.style.foreground_color.r,
            self.style.foreground_color.g,
            self.style.foreground_color.b,
            self.style.foreground_color.a,
        );
        cairoinfo.fill_preserve().unwrap();
        cairoinfo.set_source_rgba(
            self.style.border_text_color.r,
            self.style.border_text_color.g,
            self.style.border_text_color.b,
            self.style.border_text_color.a,
        );
        cairoinfo.set_line_width(self.style.border_weight);
        cairoinfo.stroke().unwrap();

        let font_size = self.style.font_size;
        let pangolayout = self
            .pango_layout
            .get_or_init(|| pangocairo::functions::create_layout(cairoinfo));
        let desc = self.font_desc_bold.get_or_init(|| {
            let mut d = pango::FontDescription::new();
            d.set_family(self.style.font_name.as_str());
            d.set_weight(pango::Weight::Bold);
            d.set_size(font_size * pango::SCALE);
            d
        });
        pangolayout.set_font_description(Some(desc));

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
        pangocairo::functions::show_layout(cairoinfo, pangolayout);
        cairoinfo.restore().unwrap();

        self.wl_surface.attach(Some(&self.buffer), 0, 0);
        self.wl_surface.damage(0, 0, width, height);
        self.wl_surface.commit();
    }
}

#[derive(Debug)]
pub struct UiInit {
    pub context: Context,
    pub stride: i32,
}

// initial bg
pub fn draw_ui(
    tmp: &mut File,
    (width, height): (i32, i32),
    background_color: crate::state::Color,
) -> UiInit {
    let cairo_fmt = Format::ARgb32;
    let stride = cairo_fmt.stride_for_width(width as u32).unwrap();
    tmp.set_len((stride * height) as u64).unwrap();
    let mmmap: MmapMut = unsafe { MmapMut::map_mut(&*tmp).unwrap() };

    let surface =
        cairo::ImageSurface::create_for_data(mmmap, cairo_fmt, width, height, stride).unwrap();
    let cairoinfo = cairo::Context::new(&surface).unwrap();
    cairoinfo.set_source_rgba(
        background_color.r,
        background_color.g,
        background_color.b,
        background_color.a,
    );
    cairoinfo.paint().unwrap();
    UiInit {
        context: cairoinfo,
        stride,
    }
}
