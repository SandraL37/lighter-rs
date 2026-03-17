mod engine;

use std::ops::Add;

use crate::core::{
    arena::{ NodeArena, node::{ NodeData, NodeId } },
    layout::engine::{ ComputedLayout, LayoutCache, LayoutStyle, UnroundedLayout, compute_layout },
    reactive::{ bind::{ DeferredBinding, bind_field }, dirty::DirtyFlags, signal::MaybeSignal },
    render::Renderer,
};

pub struct LayoutContext<'a, R: Renderer> {
    pub root: NodeId,
    pub arena: &'a mut NodeArena,
    pub renderer: &'a mut R,
}

impl<'a, R: Renderer> LayoutContext<'a, R> {
    pub fn get_children(&self, node_id: impl Into<NodeId>) -> &Vec<NodeId> {
        self.arena
            .get_children(node_id.into())
            .expect(
                "Layout engine error: Malformed NodeArena. Tried to access children of a dropped node."
            )
    }

    pub fn get_child_id(&self, parent_node_id: impl Into<NodeId>, child_index: usize) -> NodeId {
        *self
            .get_children(parent_node_id)
            .get(child_index)
            .expect(
                "Layout engine error: Malformed NodeArena. Tried to access a dropped child of a node."
            )
    }

    pub fn get_layout(&self, node_id: impl Into<NodeId>) -> &NodeLayout {
        self.arena
            .get_layout(node_id.into())
            .expect(
                "Layout engine error: Malformed NodeArena. Tried to access the layout of a dropped node."
            )
    }

    pub fn get_layout_mut(&mut self, node_id: impl Into<NodeId>) -> &mut NodeLayout {
        self.arena
            .get_layout_mut(node_id.into())
            .expect(
                "Layout engine error: Malformed NodeArena. Tried to access the layout of a dropped node."
            )
    }

    pub fn get_data(&self, node_id: impl Into<NodeId>) -> &NodeData {
        self.arena
            .get_data(node_id.into())
            .expect(
                "Layout engine error: Malformed NodeArena. Tried to access the data of a dropped node."
            )
    }

    pub fn compute_layout(&mut self, available_space: Size<AvailableSpace>) {
        compute_layout(self, self.root, available_space);
    }
}

#[derive(Debug, Default, Clone)]
pub struct NodeLayout {
    pub style: LayoutStyle,
    pub unrounded: UnroundedLayout,
    pub computed: ComputedLayout,
    pub cache: LayoutCache,
}

impl NodeLayout {
    pub fn new(style: impl Into<LayoutStyle>) -> Self {
        Self {
            style: style.into(),
            unrounded: ComputedLayout::default(),
            computed: ComputedLayout::default(),
            cache: LayoutCache::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

impl<T: Copy> Size<T> {
    pub const fn wh(width: T, height: T) -> Self {
        Self { width, height }
    }

    pub fn uniform(value: T) -> Self {
        Self {
            width: value,
            height: value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Insets<T> {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}

impl<T: Copy> Insets<T> {
    pub fn new(top: T, right: T, bottom: T, left: T) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn uniform(value: T) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub fn xy(x: T, y: T) -> Self {
        Self {
            top: y,
            right: x,
            bottom: y,
            left: x,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect<T> {
    pub location: Point<T>,
    pub size: Size<T>,
}

impl<T: Copy> Rect<T> {
    pub fn new(location: Point<T>, size: Size<T>) -> Self {
        Self { location, size }
    }

    pub fn xywh(x: T, y: T, width: T, height: T) -> Self {
        Self {
            location: Point::xy(x, y),
            size: Size::wh(width, height),
        }
    }

    pub fn includes(&self, point: Point<T>) -> bool where T: PartialOrd + Add<Output = T> {
        let max_x = self.location.x + self.size.width;
        let max_y = self.location.y + self.size.height;

        point.x >= self.location.x &&
            point.y >= self.location.y &&
            point.x < max_x &&
            point.y < max_y
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T: Clone> Point<T> {
    pub fn xy(x: T, y: T) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone)]
pub enum LayoutKind {
    Container(ContainerStyle),
    Leaf(LeafStyle),
}

impl Into<LayoutKind> for ContainerStyle {
    fn into(self) -> LayoutKind {
        LayoutKind::Container(self)
    }
}

impl Into<LayoutKind> for LeafStyle {
    fn into(self) -> LayoutKind {
        LayoutKind::Leaf(self)
    }
}

pub type Margin = Insets<DefiniteDimensionAuto>;
pub type Padding = Insets<DefiniteDimension>;
pub type Gap = Size<DefiniteDimension>;

#[derive(Debug, Clone, Copy)]
pub struct ContainerStyle {
    pub size: Size<Dimension>,
    pub min_size: Size<Dimension>,
    pub max_size: Size<Dimension>,

    pub margin: Margin,
    pub padding: Padding,
    pub gap: Gap,

    pub flex_direction: FlexDirection,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub flex_wrap: FlexWrap,
}

impl Default for ContainerStyle {
    fn default() -> Self {
        Self {
            size: Size::wh(Dimension::Auto, Dimension::Auto),
            min_size: Size::wh(Dimension::Auto, Dimension::Auto),
            max_size: Size::wh(Dimension::Auto, Dimension::Auto),
            margin: Margin::uniform(DefiniteDimensionAuto::Pixels(0.0)),
            padding: Padding::uniform(DefiniteDimension::Pixels(0.0)),
            gap: Gap::uniform(DefiniteDimension::Pixels(0.0)),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Stretch,
            flex_wrap: FlexWrap::NoWrap,
        }
    }
}

fn resolve_container_style(
    data: &mut NodeData,
    _layout: &mut NodeLayout
) -> &'static mut ContainerStyle {
    match &mut data.layout_kind {
        LayoutKind::Container(style) => style,
        LayoutKind::Leaf(_) => unreachable!(),
    }
}

fn resolve_leaf_style<'a>(
    data: &'a mut NodeData,
    _layout: &'a mut NodeLayout
) -> &'a mut LeafStyle {
    match &mut data.layout_kind {
        LayoutKind::Container(_) => unreachable!(),
        LayoutKind::Leaf(style) => style,
    }
}

pub trait ContainerStylePropsExt: Sized {
    fn container_style_mut(&mut self) -> &mut ContainerStyle;
    fn bindings_mut(&mut self) -> &mut Vec<DeferredBinding>;

    fn w(mut self, width: impl Into<MaybeSignal<Dimension>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            width,
            DirtyFlags::LAYOUT,
            |style| &mut style.size.width,
            resolve_container_style
        );
        self
    }

    fn h(mut self, height: impl Into<MaybeSignal<Dimension>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            height,
            DirtyFlags::LAYOUT,
            |style| &mut style.size.height,
            resolve_container_style
        );
        self
    }

    fn size(mut self, size: impl Into<MaybeSignal<Dimension>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            size,
            DirtyFlags::LAYOUT,
            |style| { &mut style.size.width },
            resolve_container_style
        );
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            size,
            DirtyFlags::LAYOUT,
            |style| { &mut style.size.height },
            resolve_container_style
        );
        self
    }

    fn max_w(mut self, max_width: impl Into<MaybeSignal<Dimension>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            max_width,
            DirtyFlags::LAYOUT,
            |style| &mut style.max_size.width,
            resolve_container_style
        );
        self
    }

    fn max_h(mut self, max_height: impl Into<MaybeSignal<Dimension>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            max_height,
            DirtyFlags::LAYOUT,
            |style| &mut style.max_size.height,
            resolve_container_style
        );
        self
    }

    fn max_size(mut self, max_size: impl Into<MaybeSignal<Dimension>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            max_size,
            DirtyFlags::LAYOUT,
            |style| &mut style.max_size.width,
            resolve_container_style
        );
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            max_size,
            DirtyFlags::LAYOUT,
            |style| &mut style.max_size.height,
            resolve_container_style
        );
        self
    }

    fn p(mut self, padding: impl Into<MaybeSignal<Padding>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            padding,
            DirtyFlags::LAYOUT,
            |style| &mut style.padding,
            resolve_container_style
        );
        self
    }

    fn m(mut self, margin: impl Into<MaybeSignal<Margin>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            margin,
            DirtyFlags::LAYOUT,
            |style| &mut style.margin,
            resolve_container_style
        );
        self
    }

    fn align(mut self, align_items: impl Into<MaybeSignal<AlignItems>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            align_items,
            DirtyFlags::LAYOUT,
            |style| &mut style.align_items,
            resolve_container_style
        );
        self
    }

    fn justify(mut self, justify_content: impl Into<MaybeSignal<JustifyContent>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            justify_content,
            DirtyFlags::LAYOUT,
            |style| &mut style.justify_content,
            resolve_container_style
        );
        self
    }

    fn gap_x(mut self, gap: impl Into<MaybeSignal<DefiniteDimension>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            gap,
            DirtyFlags::LAYOUT,
            |style| &mut style.gap.width,
            resolve_container_style
        );
        self
    }

    fn gap_y(mut self, gap: impl Into<MaybeSignal<DefiniteDimension>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            gap,
            DirtyFlags::LAYOUT,
            |style| &mut style.gap.height,
            resolve_container_style
        );
        self
    }

    fn gap(mut self, gap: impl Into<MaybeSignal<DefiniteDimension>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            gap,
            DirtyFlags::LAYOUT,
            |style| &mut style.gap.width,
            resolve_container_style
        );
        self
    }

    fn flex_direction(mut self, direction: impl Into<MaybeSignal<FlexDirection>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            direction,
            DirtyFlags::LAYOUT,
            |style| &mut style.flex_direction,
            resolve_container_style
        );
        self
    }

    fn flex_wrap(mut self, wrap: impl Into<MaybeSignal<FlexWrap>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            wrap,
            DirtyFlags::LAYOUT,
            |style| &mut style.flex_wrap,
            resolve_container_style
        );
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LeafStyle {
    pub size: Size<Dimension>,
    pub min_size: Size<Dimension>,
    pub max_size: Size<Dimension>,

    pub margin: Margin,
}

impl Default for LeafStyle {
    fn default() -> Self {
        Self {
            size: Size::wh(Dimension::Auto, Dimension::Auto),
            min_size: Size::wh(Dimension::Auto, Dimension::Auto),
            max_size: Size::wh(Dimension::Auto, Dimension::Auto),
            margin: Margin::uniform(DefiniteDimensionAuto::Pixels(0.0)),
        }
    }
}

pub trait LeafStylePropsExt: Sized {
    fn leaf_style_mut(&mut self) -> &mut LeafStyle;
    fn bindings_mut(&mut self) -> &mut Vec<DeferredBinding>;

    fn w(mut self, width: impl Into<MaybeSignal<Dimension>>) -> Self {
        bind_field(
            self.leaf_style_mut(),
            self.bindings_mut(),
            width,
            DirtyFlags::LAYOUT,
            |style| &mut style.size.width,
            resolve_leaf_style
        );
        self
    }

    fn h(mut self, height: impl Into<MaybeSignal<Dimension>>) -> Self {
        bind_field(
            self.leaf_style_mut(),
            self.bindings_mut(),
            height,
            DirtyFlags::LAYOUT,
            |style| &mut style.size.height,
            resolve_leaf_style
        );
        self
    }

    fn size(mut self, size: impl Into<MaybeSignal<Dimension>>) -> Self {
        bind_field(
            self.leaf_style_mut(),
            self.bindings_mut(),
            size,
            DirtyFlags::LAYOUT,
            |style| { &mut style.size.width },
            resolve_leaf_style
        );
        bind_field(
            self.leaf_style_mut(),
            self.bindings_mut(),
            size,
            DirtyFlags::LAYOUT,
            |style| { &mut style.size.height },
            resolve_leaf_style
        );
        self
    }

    fn max_w(mut self, max_width: impl Into<MaybeSignal<Dimension>>) -> Self {
        bind_field(
            self.leaf_style_mut(),
            self.bindings_mut(),
            max_width,
            DirtyFlags::LAYOUT,
            |style| &mut style.max_size.width,
            resolve_leaf_style
        );
        self
    }

    fn max_h(mut self, max_height: impl Into<MaybeSignal<Dimension>>) -> Self {
        bind_field(
            self.leaf_style_mut(),
            self.bindings_mut(),
            max_height,
            DirtyFlags::LAYOUT,
            |style| &mut style.max_size.height,
            resolve_leaf_style
        );
        self
    }

    fn max_size(mut self, max_size: impl Into<MaybeSignal<Dimension>>) -> Self {
        bind_field(
            self.leaf_style_mut(),
            self.bindings_mut(),
            max_size,
            DirtyFlags::LAYOUT,
            |style| &mut style.max_size.width,
            resolve_leaf_style
        );
        bind_field(
            self.leaf_style_mut(),
            self.bindings_mut(),
            max_size,
            DirtyFlags::LAYOUT,
            |style| &mut style.max_size.height,
            resolve_leaf_style
        );
        self
    }

    fn m(mut self, margin: impl Into<MaybeSignal<Margin>>) -> Self {
        bind_field(
            self.leaf_style_mut(),
            self.bindings_mut(),
            margin,
            DirtyFlags::LAYOUT,
            |style| &mut style.margin,
            resolve_leaf_style
        );
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dimension {
    Auto,
    Pixels(f32),
    Percent(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DefiniteDimension {
    Pixels(f32),
    Percent(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DefiniteDimensionAuto {
    Auto,
    Pixels(f32),
    Percent(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AvailableSpace {
    Definite(f32),
    MinContent,
    MaxContent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyContent {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignItems {
    Start,
    End,
    Center,
    Stretch,
    Baseline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

#[inline(always)]
pub fn px(value: f32) -> DefiniteDimension {
    DefiniteDimension::Pixels(value)
}

#[inline(always)]
pub fn percent(value: f32) -> DefiniteDimension {
    DefiniteDimension::Percent(value)
}

#[inline(always)]
pub fn auto() -> Dimension {
    Dimension::Auto
}

impl From<DefiniteDimension> for Dimension {
    #[inline(always)]
    fn from(d: DefiniteDimension) -> Self {
        match d {
            DefiniteDimension::Pixels(v) => Dimension::Pixels(v),
            DefiniteDimension::Percent(v) => Dimension::Percent(v),
        }
    }
}

impl From<DefiniteDimension> for DefiniteDimensionAuto {
    #[inline(always)]
    fn from(d: DefiniteDimension) -> Self {
        match d {
            DefiniteDimension::Pixels(v) => DefiniteDimensionAuto::Pixels(v),
            DefiniteDimension::Percent(v) => DefiniteDimensionAuto::Percent(v),
        }
    }
}

impl From<DefiniteDimension> for MaybeSignal<Dimension> {
    fn from(d: DefiniteDimension) -> Self {
        MaybeSignal::Static(d.into())
    }
}
