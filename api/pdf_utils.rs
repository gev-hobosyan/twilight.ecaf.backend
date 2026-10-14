use krilla::color::rgb::Color;
use krilla::geom::{Path, PathBuilder, Point};
use krilla::num::NormalizedF32;
use krilla::paint::{Fill, FillRule, Paint, Stroke};
use krilla::surface::Surface;
use krilla::text::{Font, TextDirection};

pub fn text_size(font_size: f32, text: &str) -> (f32, f32) {
    let chars = text.len() as f32;

    let height = font_size * 1.5;
    let width = chars * (font_size * 0.45);

    (width, height)
}

pub fn draw_title(surface: &mut Surface, pw: f32, ph: f32, text: &str, font_size: f32, font: Font) -> (f32, f32) {
    draw_centered_text(surface, 0.0, 2.0, pw, ph, text, font_size, font)
}

fn draw_centered_text(surface: &mut Surface, x: f32, y: f32, w: f32, h: f32, text: &str, font_size: f32, font: Font) -> (f32, f32) {
    let (text_width, text_height) = text_size(font_size, &text);

    let x = x + (w - text_width) / 2.0;
    let y = y + text_height;

    surface.draw_text(
        Point::from_xy(x, y),
        font,
        font_size,
        text,
        false,
        TextDirection::Auto
    );

    (x, y)
}

pub fn draw_table<T>(
    surface: &mut Surface,
    pw: f32,
    py: f32,
    headers: Vec<&str>,
    data: Vec<T>,
    font_size: f32,
    font: Font,
    start_y: f32) -> Option<String>
where T: TableDrawable{
    let column_count = headers.len();

    let column_width = (pw - 40.0) / column_count as f32;
    let row_height = font_size * 2.0 + 4.0;

    let header_y = start_y + row_height;

    let header = {
        let mut pb = PathBuilder::new();
        pb.move_to(20.0, start_y);
        pb.line_to(pw - 20.0, start_y);

        for i in 0..column_count + 1 {
            let x = 20.0 + column_width * (i as f32);
            pb.move_to(x, start_y);
            pb.line_to(x, header_y);
        }

        pb.move_to(20.0, header_y);
        pb.line_to(pw - 20.0, header_y);

        pb.close();

        pb.finish().unwrap()
    };

    surface.set_fill(
        Some(Fill {
            paint: Paint::from(Color::new(0, 0, 0)),
            opacity: NormalizedF32::ONE,
            rule: FillRule::EvenOdd
        })
    );

    let mut line_stroke = Stroke::default();
    line_stroke.width = 1.0;

    for i in 0..column_count {
        let text = headers[i];

        draw_centered_text(surface, 15.0 + column_width * (i as f32), start_y - 3.0, column_width, header_y - start_y, text, 18.0, font.clone());
    }

    let mut el_y = header_y;

    surface.set_stroke(Some(line_stroke.clone()));
    surface.draw_path(&header);

    for element in data {
        surface.set_stroke(Some(line_stroke.clone()));

        let el_data = element.into_vec();

        let y = el_y + row_height;

        let path = {
            let mut pb = PathBuilder::new();
            for i in 0..el_data.len() + 1 {
                let x = 20.0 + column_width * (i as f32);
                pb.move_to(x, el_y);
                pb.line_to(x, y);
            }

            pb.move_to(20.0, y);
            pb.line_to(pw - 20.0, y);

            pb.close();
            pb.finish().unwrap()
        };

        surface.draw_path(&path);

        for i in 0..el_data.len() {
            let text = &el_data[i];

            surface.set_stroke(None);

            draw_centered_text(surface, 17.0 + column_width * (i as f32), el_y - 3.0, column_width, y - el_y, text.as_str(), 16.0, font.clone());
        }

        el_y = y;
    }

    None
}

pub trait TableDrawable {
    fn into_vec(self) -> Vec<String>;
}