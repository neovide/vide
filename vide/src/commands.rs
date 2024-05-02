use std::sync::Arc;

use glam::Vec4;
use ordered_float::OrderedFloat;

#[derive(Clone, Debug)]
pub enum DrawCommand {
    Grid {
        id: u64,
        command: GridCommand,
    },
    Cursor(Option<Cursor>),
    Font {
        fallback_list: Vec<String>,
        size: f64,
        bold: bool,
        italic: bool,
        subpixel: bool,
    },
    LineSpace(i64),
    DefaultStyle(Style),
}

#[derive(Clone, Debug)]
pub enum GridCommand {
    Position {
        left: u64,
        top: u64,
        width: u64,
        height: u64,
        floating_order: Option<u64>,
    },
    DrawLine(Vec<LineFragment>),
    Clear,
    Show,
    Hide,
    Close,
    Scroll {
        delta: f64,
    },
}

#[derive(Clone, Debug)]
pub struct LineFragment {
    pub text: String,
    pub left: u64,
    pub top: u64,
    pub style: Option<Arc<Style>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    pub colors: Colors,
    pub reverse: bool,
    pub italic: bool,
    pub bold: bool,
    pub strikethrough: bool,
    pub blend: u8,
    pub underline: Option<UnderlineStyle>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Colors {
    pub foreground: Option<Vec4>,
    pub background: Option<Vec4>,
    pub special: Option<Vec4>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum UnderlineStyle {
    UnderLine,
    UnderDouble,
    UnderDash,
    UnderDot,
    UnderCurl,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CursorShape {
    Block,
    Horizontal(OrderedFloat<f32>),
    Vertical(OrderedFloat<f32>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cursor {
    pub parent_grid: u64,
    pub left: u64,
    pub top: u64,
    pub shape: CursorShape,
    pub blinkwait: Option<u64>,
    pub blinkon: Option<u64>,
    pub blinkoff: Option<u64>,
    pub style: Option<Arc<Style>>,
    pub double_width: bool,
    pub content_under_cursor: (String, Option<Arc<Style>>),
}
