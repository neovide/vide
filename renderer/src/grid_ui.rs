#[derive(Clone, Debug)]
pub enum DrawCommand {
    CloseWindow(u64),
    Window {
        window_id: u64,
        command: WindowDrawCommand,
    },
    UpdateCursor(Option<Cursor>),
    FontChanged {
        fallback_list: Vec<String>,
        size: f64,
        bold: bool,
        italic: bool,
        subpixel: bool,
    },
    LineSpaceChanged(i64),
    DefaultStyleChanged(Style),
}

#[derive(Clone, Debug)]
pub enum WindowDrawCommand {
    Position {
        window_position: (f64, f64),
        window_size: (u64, u64),
        floating_order: Option<u64>,
    },
    DrawLine(Vec<LineFragment>),
    Clear,
    Show,
    Hide,
    Close,
    Viewport {
        scroll_delta: f64,
    },
}

#[derive(Clone, Debug)]
pub struct LineFragment {
    pub text: String,
    pub window_left: u64,
    pub window_top: u64,
    pub style: Option<Arc<Style>>,
}

#[derive(new, Debug, Clone, PartialEq)]
pub struct Style {
    pub colors: Colors,
    #[new(default)]
    pub reverse: bool,
    #[new(default)]
    pub italic: bool,
    #[new(default)]
    pub bold: bool,
    #[new(default)]
    pub strikethrough: bool,
    #[new(default)]
    pub blend: u8,
    #[new(default)]
    pub underline: Option<UnderlineStyle>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CursorShape {
    Block,
    Horizontal(f32),
    Vertical(f32),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cursor {
    pub parent_window_id: u64,
    pub window_left: u64,
    pub window_top: u64,
    pub shape: CursorShape,
    pub blinkwait: Option<u64>,
    pub blinkon: Option<u64>,
    pub blinkoff: Option<u64>,
    pub style: Option<Arc<Style>>,
    pub double_width: bool,
    pub content_under_cursor: (String, Option<Arc<Style>>),
}
