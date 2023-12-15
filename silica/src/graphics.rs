use taffy::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum ThemeColor {
    Background,
    ButtonNormal,
    ButtonOver,
    ButtonPress,
    ButtonDisable,
    Border,
    Foreground,
}

impl ThemeColor {
    pub fn to_rgba(self) -> [f32; 4] {
        match self {
            ThemeColor::Background => [0.847, 0.847, 0.847, 1.0],
            ThemeColor::ButtonNormal | ThemeColor::ButtonDisable => [0.784, 0.784, 0.784, 1.0],
            ThemeColor::ButtonOver => [0.722, 0.722, 0.722, 1.0],
            ThemeColor::ButtonPress => [0.659, 0.659, 0.659, 1.0],
            ThemeColor::Border => [0.094, 0.094, 0.094, 1.0],
            ThemeColor::Foreground => [0.094, 0.094, 0.094, 1.0],
        }
    }
}

#[derive(Clone)]
pub struct VisualStyle {
    pub background: Option<ThemeColor>,
    pub border: Option<ThemeColor>,
    pub foreground: Option<ThemeColor>,
}

impl VisualStyle {
    pub const BUTTON: VisualStyle = VisualStyle {
        background: Some(ThemeColor::ButtonNormal),
        border: Some(ThemeColor::Border),
        foreground: Some(ThemeColor::Foreground),
    };
}

impl Default for VisualStyle {
    fn default() -> Self {
        VisualStyle {
            background: None,
            border: Some(ThemeColor::Border),
            foreground: Some(ThemeColor::Foreground),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HorizontalAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerticalAlign {
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Clone)]
pub struct TextSection {
    pub font: String,
    pub text: String,
    pub h_align: HorizontalAlign,
    pub v_align: VerticalAlign,
}

impl Default for TextSection {
    fn default() -> Self {
        TextSection {
            font: "sans 14".to_owned(),
            text: String::new(),
            h_align: HorizontalAlign::Left,
            v_align: VerticalAlign::Center,
        }
    }
}

pub trait GraphicsContext {
    fn save(&mut self);
    fn restore(&mut self);
    fn translate(&mut self, tx: f32, ty: f32);

    fn set_color(&mut self, color: ThemeColor);
    fn draw_rect(&mut self, size: Size<f32>);
    fn draw_border(&mut self, size: Size<f32>, border: Rect<LengthPercentage>);
    fn draw_text(&mut self, size: Size<f32>, text: &TextSection);
}
