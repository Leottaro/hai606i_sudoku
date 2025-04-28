use macroquad::prelude::*;

use super::Button;

impl Button {
    pub fn new(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        text: String,
        clicked: bool,
        scale_factor: f32,
    ) -> Self {
        Button {
            x,
            y,
            width,
            height,
            enabled: true,
            clickable: true,
            text,
            clicked,
            hover: false,
            scale_factor,
        }
    }

    pub fn x(&self) -> f32 {
        self.x * self.scale_factor
    }

    pub fn y(&self) -> f32 {
        self.y * self.scale_factor
    }

    pub fn width(&self) -> f32 {
        self.width * self.scale_factor
    }

    pub fn height(&self) -> f32 {
        self.height * self.scale_factor
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

    pub fn set_clickable(&mut self, clickable: bool) {
        self.clickable = clickable;
    }

    pub fn set_clicked(&mut self, clicked: bool) {
        self.clicked = clicked;
    }

    pub fn set_hover(&mut self, hover: bool) {
        self.hover = hover;
    }

    pub fn set_scale_factor(&mut self, scale_factor: f32) {
        self.scale_factor = scale_factor;
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }

    pub async fn draw(&self, font: Font) {
        let color = Color::from_hex(0xe4ebf2);
        let hover_color = Color::from_hex(0xd0dbe7);
        let clicked_color = Color::from_hex(0xc2ddf8);
        let clicked_hovered_color = Color::from_hex(0x9ac5f8);
        let blocked_color = Color::from_hex(0x818294);

        let mut text_color = Color::from_hex(0x000000);
        if !self.clickable {
            text_color = Color::from_hex(0xffffff);
        }

        let actual_color = if self.clickable {
            if self.clicked {
                if self.hover {
                    clicked_hovered_color
                } else {
                    clicked_color
                }
            } else if self.hover {
                hover_color
            } else {
                color
            }
        } else {
            blocked_color
        };

        draw_rectangle(
            self.x * self.scale_factor,
            self.y * self.scale_factor,
            self.width * self.scale_factor,
            self.height * self.scale_factor,
            actual_color,
        );
        let font_size = (((self.height * self.scale_factor) as u16) * 2) / 8;
        let text = self.text.clone();
        let text_dimensions = measure_text(&text, Some(&font), font_size, 1.0);
        let text_x = self.x * self.scale_factor
            + (self.width * self.scale_factor - text_dimensions.width) / 2.0;
        let text_y = self.y * self.scale_factor
            + (self.height * self.scale_factor + text_dimensions.height) / 2.0;
        draw_text_ex(
            &text,
            text_x,
            text_y,
            TextParams {
                font: Some(&font),
                font_size,
                color: text_color,
                ..Default::default()
            },
        );
    }
}
