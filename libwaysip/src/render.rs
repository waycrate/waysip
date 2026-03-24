use super::state::LayerSurfaceInfo;
use crate::{BoxInfo, Size, utils::Position};
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
            let (txt1_x, txt1_y) = pangolayout.pixel_size();
            cairoinfo.save().unwrap();
            cairoinfo.move_to(10., 60.);
            pangocairo::functions::show_layout(cairoinfo, pangolayout);
            cairoinfo.restore().unwrap();

            let pos_txt = format!("pos: {start_x}, {start_y}");
            pangolayout.set_text(&pos_txt);
            let (txt2_x, txt2_y) = pangolayout.pixel_size();
            cairoinfo.save().unwrap();
            cairoinfo.move_to(10., txt1_y as f64 + 65.0);
            pangocairo::functions::show_layout(cairoinfo, pangolayout);
            cairoinfo.restore().unwrap();

            cairoinfo.save().unwrap();
            cairoinfo.set_operator(cairo::Operator::DestOver);
            cairoinfo.rectangle(
                5.0,
                55.0,
                txt1_x.max(txt2_x) as f64 + 10.0,
                (txt1_y + txt2_y) as f64 + 16.0,
            );
            cairoinfo.set_source_rgba(
                self.style.background_color.r,
                self.style.background_color.g,
                self.style.background_color.b,
                self.style.background_color.a,
            );
            cairoinfo.fill().unwrap();

            cairoinfo.restore().unwrap();
            cairoinfo.set_operator(cairo::Operator::Over);
        } else {
            cairoinfo.set_source_rgba(
                self.style.background_color.r,
                self.style.background_color.g,
                self.style.background_color.b,
                self.style.background_color.a,
            );
            cairoinfo.paint().unwrap();
        }

        self.wl_surface.attach(Some(&self.buffer), 0, 0);
        self.wl_surface.damage(0, 0, width, height);
        self.wl_surface.commit();
    }

    pub fn redraw(
        &mut self,
        Position {
            x: start_pos_x,
            y: start_pos_y,
        }: Position<f64>,
        end_pos: Position<f64>,
        Position {
            x: start_x,
            y: start_y,
        }: Position,
        Size { width, height }: Size,
        draw_text: bool,
        opt_boxes: Option<&Vec<BoxInfo>>,
    ) {
        let cairoinfo = &self.cairo_t;

        let current_sel = {
            let rx1 = (start_pos_x - start_x as f64).min(end_pos.x - start_x as f64);
            let ry1 = (start_pos_y - start_y as f64).min(end_pos.y - start_y as f64);
            let rx2 = (start_pos_x - start_x as f64).max(end_pos.x - start_x as f64);
            let ry2 = (start_pos_y - start_y as f64).max(end_pos.y - start_y as f64);
            [rx1, ry1, rx2, ry2]
        };

        let border_margin = self.style.border_weight + 2.0;

        let (text_margin_w, text_margin_h) = *self.margin.get_or_init(|| {
            if !draw_text {
                return (
                    self.style.border_weight + 2.0,
                    self.style.border_weight + 2.0,
                );
            }
            let layout = self
                .pango_layout
                .get_or_init(|| pangocairo::functions::create_layout(cairoinfo));
            let desc = self.font_desc_bold.get_or_init(|| {
                let mut d = pango::FontDescription::new();
                d.set_family(self.style.font_name.as_str());
                d.set_weight(pango::Weight::Bold);
                d.set_size(self.style.font_size * pango::SCALE);
                d
            });
            layout.set_font_description(Some(desc));
            // Worst-case label: max 5-digit coords
            layout.set_text("-99999,-99999, 99999x99999");
            let (tw, th) = layout.pixel_size();
            // 10px offset from selection corner + text size + padding(2px)
            (tw as f64 + 12.0, th as f64 + 12.0)
        });

        let clip_rect = if let Some(p) = self.prev_selection.as_ref() {
            [
                p[0].min(current_sel[0]) - border_margin,
                p[1].min(current_sel[1]) - border_margin,
                p[2].max(current_sel[2]) + text_margin_w,
                p[3].max(current_sel[3]) + text_margin_h,
            ]
        } else {
            [
                current_sel[0] - border_margin,
                current_sel[1] - border_margin,
                current_sel[2] + text_margin_w,
                current_sel[3] + text_margin_h,
            ]
        };

        let cx = clip_rect[0].max(0.0);
        let cy = clip_rect[1].max(0.0);
        let cw = (clip_rect[2] - cx).min(width as f64 - cx).max(0.0);
        let ch = (clip_rect[3] - cy).min(height as f64 - cy).max(0.0);

        let dx = cx.floor() as i32;
        let dy = cy.floor() as i32;
        let dw = (cx + cw).ceil() as i32 - dx;
        let dh = (cy + ch).ceil() as i32 - dy;

        cairoinfo.save().unwrap();
        cairoinfo.rectangle(dx as f64, dy as f64, dw as f64, dh as f64);
        cairoinfo.clip();
        cairoinfo.set_operator(cairo::Operator::Source);
        cairoinfo.set_source_rgba(
            self.style.background_color.r,
            self.style.background_color.g,
            self.style.background_color.b,
            self.style.background_color.a,
        );
        cairoinfo.paint().unwrap();
        cairoinfo.restore().unwrap();

        cairoinfo.set_operator(cairo::Operator::Source);

        if let Some(boxes) = opt_boxes {
            for box_info in boxes {
                let bstart_x = box_info.start_x - start_x as f64;
                let bstart_y = box_info.start_y - start_y as f64;
                let bend_x = box_info.end_x - start_x as f64;
                let bend_y = box_info.end_x - start_y as f64;
                let bwidth = bend_x - box_info.start_x;
                let bheight = bend_y - box_info.start_y;
                cairoinfo.rectangle(bstart_x, bstart_y, bwidth, bheight);
                cairoinfo.set_source_rgba(
                    self.style.box_color.r,
                    self.style.box_color.g,
                    self.style.box_color.b,
                    self.style.box_color.a,
                );
                cairoinfo.fill_preserve().unwrap();
                cairoinfo.stroke().unwrap();
            }
        }

        let relate_start_x = start_pos_x - start_x as f64;
        let relate_start_y = start_pos_y - start_y as f64;
        let relate_end_x = end_pos.x - start_x as f64;
        let relate_end_y = end_pos.y - start_y as f64;
        let rlwidth = relate_end_x - relate_start_x;
        let rlheight = relate_end_y - relate_start_y;

        cairoinfo.rectangle(relate_start_x, relate_start_y, rlwidth, rlheight);
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

        if draw_text {
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
                (end_pos.x - start_pos_x) as i32,
                (end_pos.y - start_pos_y) as i32
            );

            pangolayout.set_text(text.as_str());
            cairoinfo.save().unwrap();
            cairoinfo.move_to(relate_end_x + 10., relate_end_y + 10.);
            pangocairo::functions::show_layout(cairoinfo, pangolayout);
            cairoinfo.restore().unwrap();
        }

        self.wl_surface.attach(Some(&self.buffer), 0, 0);
        self.wl_surface.damage(dx, dy, dw, dh);
        self.wl_surface.commit();

        self.prev_selection = Some(current_sel);
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
    background_color: crate::Color,
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
