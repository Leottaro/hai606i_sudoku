use macroquad::prelude::*;

use super::{
    display::{DECREASE, INCREASE},
    Button,
};

impl Button {
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

    pub fn stroke(&self) -> bool {
        self.stroke
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
        let hover_color = Color::from_hex(0xd0dbe7).with_alpha(0.25);
        let clicked_color = Color::from_hex(0xc2ddf8).with_alpha(0.75);
        let blocked_color = Color::from_hex(0x818294).with_alpha(0.75);

        let x = self.x * self.scale_factor;
        let y = self.y * self.scale_factor;
        let width = self.width * self.scale_factor;
        let height = self.height * self.scale_factor;

        if self.draw_border {
            draw_rectangle(
                x - 1.,
                y - 1.,
                width + 2.,
                height + 2.,
                Color::from_hex(0),
            );
        }

        draw_rectangle(
            x,
            y,
            width,
            height,
            self.background_color,
        );

        if self.clickable {
            if self.hover {
                draw_rectangle(
                    x,
                    y,
                    width,
                    height,
                    hover_color,
                );
            }

            if self.clicked {
                draw_rectangle(
                    x,
                    y,
                    width,
                    height,
                    clicked_color,
                );
            }
        } else {
            draw_rectangle(
                x,
                y,
                width,
                height,
                blocked_color,
            );
        }

        if self.stroke {
            draw_line(x, y, x + width, y + height, 2.0, Color::from_hex(0xff0000));
            draw_line(x, y + height, x + width, y, 2.0, Color::from_hex(0xff0000));
        }

        if !self.draw_text {
            return;
        }

        let mut text_color = Color::from_hex(0x000000);
        if !self.clickable {
            text_color = Color::from_hex(0xffffff);
        }

        let font_size = if self.text.eq(DECREASE) || self.text.eq(INCREASE) {
            height as u16
        } else {
            (height as u16) / 4
        };
        let text = self.text.clone();
        let text_dimensions = measure_text(&text, Some(&font), font_size, 1.0);
        let text_x = x
            + (width - text_dimensions.width) / 2.0;
        let text_y = y
            + (height + text_dimensions.height) / 2.0;
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

impl Default for Button {
    fn default() -> Self {
        Self {
            x: 0.,
            y: 0.,
            width: 0.,
            height: 0.,
            enabled: true,
            clickable: true,
            text: "Default".to_string(),
            clicked: false,
            hover: false,
            scale_factor: 1.,
            background_color: Color::from_hex(0xe4ebf2),
            draw_text: true,
            draw_border: false,
            stroke: false
        }
    }
}
