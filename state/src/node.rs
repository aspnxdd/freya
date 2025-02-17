use dioxus_native_core::node_ref::{AttributeMask, NodeMask, NodeView};
use dioxus_native_core::state::{NodeDepState, ParentDepState, State};
use dioxus_native_core_macro::{sorted_str_slice, State};
use skia_safe::Color;

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub enum SizeMode {
    #[default]
    Auto,
    Percentage(f32),
    Manual(f32),
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub enum DirectionMode {
    #[default]
    Vertical,
    Horizontal,
    Both,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FontStyle {
    pub color: Color,
    pub font_family: String,
    pub font_size: f32,
}

impl Default for FontStyle {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            font_family: "Fira Sans".to_string(),
            font_size: 16.0,
        }
    }
}

#[derive(Debug, Clone, State, Default)]
pub struct NodeState {
    #[node_dep_state()]
    pub size: Size,
    #[node_dep_state()]
    pub style: Style,
    #[parent_dep_state(font_style)]
    pub font_style: FontStyle,
}

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub struct Size {
    pub width: SizeMode,
    pub height: SizeMode,
    pub min_height: SizeMode,
    pub min_width: SizeMode,
    pub padding: (f32, f32, f32, f32),
    pub scroll_y: f32,
    pub scroll_x: f32,
    pub direction: DirectionMode,
}

impl Size {
    pub fn expanded() -> Self {
        Self {
            width: SizeMode::Percentage(100.0),
            height: SizeMode::Percentage(100.0),
            min_height: SizeMode::Manual(0.0),
            min_width: SizeMode::Manual(0.0),
            padding: (0.0, 0.0, 0.0, 0.0),
            scroll_y: 0.0,
            scroll_x: 0.0,
            direction: DirectionMode::Both,
        }
    }
}

/// Font style are inherited by default if not specified otherwise by some of the supported attributes.
impl ParentDepState for FontStyle {
    type Ctx = ();
    type DepState = Self;

    const NODE_MASK: NodeMask =
        NodeMask::new_with_attrs(AttributeMask::Static(&sorted_str_slice!([
            "color",
            "font_size",
            "font_family"
        ])));

    fn reduce<'a>(
        &mut self,
        node: NodeView,
        parent: Option<&'a Self::DepState>,
        _ctx: &Self::Ctx,
    ) -> bool {
        let mut font_style = parent.map(|c| c.clone()).unwrap_or_default();

        for attr in node.attributes() {
            match attr.name {
                "color" => {
                    let new_color = parse_color(&attr.value.to_string());
                    if let Some(new_color) = new_color {
                        font_style.color = new_color;
                    }
                }
                "font_family" => {
                    font_style.font_family = attr.value.to_string();
                }
                "font_size" => {
                    if let Ok(font_size) = attr.value.to_string().parse() {
                        font_style.font_size = font_size;
                    }
                }
                _ => {}
            }
        }
        let changed = &font_style != self;
        *self = font_style;
        changed
    }
}

impl NodeDepState<()> for Size {
    type Ctx = ();

    const NODE_MASK: NodeMask =
        NodeMask::new_with_attrs(AttributeMask::Static(&sorted_str_slice!([
            "width",
            "height",
            "min_height",
            "min_width",
            "padding",
            "scroll_y",
            "scroll_x",
            "direction"
        ])));

    fn reduce<'a>(&mut self, node: NodeView, _sibling: (), _ctx: &Self::Ctx) -> bool {
        let mut width = SizeMode::default();
        let mut height = SizeMode::default();
        let mut min_height = SizeMode::default();
        let mut min_width = SizeMode::default();
        let mut padding = (0.0, 0.0, 0.0, 0.0);
        let mut scroll_y = 0.0;
        let mut scroll_x = 0.0;
        let mut direction = DirectionMode::Vertical;

        for a in node.attributes() {
            match a.name {
                "width" => {
                    let attr = a.value.to_string();
                    if let Some(new_width) = parse_size(&attr) {
                        width = new_width;
                    }
                }
                "height" => {
                    let attr = a.value.to_string();
                    if let Some(new_height) = parse_size(&attr) {
                        height = new_height;
                    }
                }
                "min_height" => {
                    let attr = a.value.to_string();
                    if let Some(new_min_height) = parse_size(&attr) {
                        min_height = new_min_height;
                    }
                }
                "min_width" => {
                    let attr = a.value.to_string();
                    if let Some(new_min_width) = parse_size(&attr) {
                        min_width = new_min_width;
                    }
                }
                "padding" => {
                    let total_padding: f32 = a.value.to_string().parse().unwrap();
                    let padding_for_side = total_padding / 2.0;
                    padding.0 = padding_for_side;
                    padding.1 = padding_for_side;
                    padding.2 = padding_for_side;
                    padding.3 = padding_for_side;
                }
                "scroll_y" => {
                    let scroll: f32 = a.value.to_string().parse().unwrap();
                    scroll_y = scroll;
                }
                "scroll_x" => {
                    let scroll: f32 = a.value.to_string().parse().unwrap();
                    scroll_x = scroll;
                }
                "direction" => {
                    direction = if a.value.to_string() == "horizontal" {
                        DirectionMode::Horizontal
                    } else if a.value.to_string() == "both" {
                        DirectionMode::Both
                    } else {
                        DirectionMode::Vertical
                    };
                }
                _ => {
                    println!("Unsupported attribute <{}>", a.name);
                }
            }
        }

        let changed = (width != self.width)
            || (height != self.height)
            || (min_height != self.min_height)
            || (min_width != self.min_width)
            || (padding != self.padding)
            || (direction != self.direction)
            || (scroll_x != self.scroll_x)
            || (scroll_y != self.scroll_y);
        *self = Self {
            width,
            height,
            min_height,
            min_width,
            padding,
            scroll_y,
            scroll_x,
            direction,
        };
        changed
    }
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct ShadowSettings {
    pub x: f32,
    pub y: f32,
    pub intensity: u8,
    pub size: f32,
    pub color: Color,
}

#[derive(Default, Clone, Debug)]
pub struct Style {
    pub background: Color,
    pub relative_layer: i16,
    pub shadow: ShadowSettings,
    pub radius: f32,
    pub image_data: Option<Vec<u8>>,
}

impl NodeDepState<()> for Style {
    type Ctx = ();

    const NODE_MASK: NodeMask =
        NodeMask::new_with_attrs(AttributeMask::Static(&sorted_str_slice!([
            "background",
            "layer",
            "shadow",
            "radius",
            "image_data"
        ])));

    fn reduce<'a>(&mut self, node: NodeView, _sibling: (), _ctx: &Self::Ctx) -> bool {
        let mut background = Color::TRANSPARENT;
        let mut relative_layer = 0;
        let mut shadow = ShadowSettings::default();
        let mut radius = 0.0;
        let mut image_data = None;

        for attr in node.attributes() {
            match attr.name {
                "background" => {
                    let new_back = parse_color(&attr.value.to_string());
                    if let Some(new_back) = new_back {
                        background = new_back;
                    }
                }
                "layer" => {
                    let new_relative_layer: Option<i16> = attr.value.to_string().parse().ok();
                    if let Some(new_relative_layer) = new_relative_layer {
                        relative_layer = new_relative_layer;
                    }
                }
                "shadow" => {
                    let new_shadow = parse_shadow(&attr.value.to_string());

                    if let Some(new_shadow) = new_shadow {
                        shadow = new_shadow;
                    }
                }
                "radius" => {
                    let new_radius: Option<f32> = attr.value.to_string().parse().ok();

                    if let Some(new_radius) = new_radius {
                        radius = new_radius;
                    }
                }
                "image_data" => {
                    let bytes = attr.value.as_bytes();
                    image_data = bytes.map(|v| v.to_vec());
                }
                _ => {
                    println!("Unsupported attribute <{}>", attr.name);
                }
            }
        }

        let changed = (background != self.background)
            || (relative_layer != self.relative_layer)
            || (shadow != self.shadow)
            || (radius != self.radius)
            || (image_data != self.image_data);

        *self = Self {
            background,
            relative_layer,
            shadow,
            radius,
            image_data,
        };
        changed
    }
}

fn parse_shadow(value: &str) -> Option<ShadowSettings> {
    let value = value.to_string();
    let mut shadow_values = value.split_ascii_whitespace();
    Some(ShadowSettings {
        x: shadow_values.nth(0)?.parse().ok()?,
        y: shadow_values.nth(0)?.parse().ok()?,
        intensity: shadow_values.nth(0)?.parse().ok()?,
        size: shadow_values.nth(0)?.parse().ok()?,
        color: parse_color(shadow_values.nth(0)?)?,
    })
}

fn parse_rgb(color: &str) -> Option<Color> {
    let color = color.replace("rgb(", "").replace(")", "");
    let mut colors = color.split(",");

    let r = colors.nth(0)?.trim().parse().ok()?;
    let g = colors.nth(0)?.trim().parse().ok()?;
    let b = colors.nth(0)?.trim().parse().ok()?;
    Some(Color::from_rgb(r, g, b))
}

fn parse_color(color: &str) -> Option<Color> {
    match color {
        "red" => Some(Color::RED),
        "green" => Some(Color::GREEN),
        "blue" => Some(Color::BLUE),
        "yellow" => Some(Color::YELLOW),
        "black" => Some(Color::BLACK),
        "gray" => Some(Color::GRAY),
        "white" => Some(Color::WHITE),
        _ => parse_rgb(color),
    }
}

fn parse_size(size: &str) -> Option<SizeMode> {
    if size == "stretch" {
        Some(SizeMode::Percentage(100.0))
    } else if size == "auto" {
        Some(SizeMode::Auto)
    } else if size.contains("%") {
        Some(SizeMode::Percentage(size.replace("%", "").parse().ok()?))
    } else {
        Some(SizeMode::Manual(size.parse().ok()?))
    }
}
