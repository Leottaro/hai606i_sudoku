use macroquad::prelude::*;

use super::Button;

impl Button {
    pub fn new(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        enabled: bool,
        text: String,
        font: Font,
    ) -> Self {
        Button {
            x,
            y,
            width,
            height,
            enabled,
            text,
            font,
            clicked: false,
            hover: false,
        }
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn clicked(&self) -> bool {
        self.clicked
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_clicked(&mut self, clicked: bool) {
        self.clicked = clicked;
    }

    pub fn set_hover(&mut self, hover: bool) {
        self.hover = hover;
    }

    pub async fn draw(&self, font: Font) {
        let color = Color::from_hex(0xe4ebf2);
        let hover_color = Color::from_hex(0xf1f5f9);
        let clicked_color = Color::from_hex(0xc2ddf8);
        let clicked_hovered_color = Color::from_hex(0x9ac5f8);

        let actual_color = if self.clicked {
            if self.hover {
                clicked_hovered_color
            } else {
                clicked_color
            }
        } else if self.hover {
            hover_color
        } else {
            color
        };
        draw_rectangle(self.x, self.y, self.width, self.height, actual_color);
        let font_size = self.height as u16 * 2 / 8;
        let text = self.text.clone();
        let text_dimensions = measure_text(&text, Some(&font), font_size, 1.0);
        let text_x = self.x + (self.width - text_dimensions.width) / 2.0;
        let text_y = self.y + (self.height + text_dimensions.height) / 2.0;
        draw_text_ex(
            &text,
            text_x,
            text_y,
            TextParams {
                font: Some(&font),
                font_size: font_size,
                color: Color::from_hex(0x000000),
                ..Default::default()
            },
        );
    }
}
