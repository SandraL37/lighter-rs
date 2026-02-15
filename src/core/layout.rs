mod engine;

pub use engine::{ComputedLayout, LayoutCache, LayoutStyle, UnroundedLayout, compute_layout};

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

#[derive(Debug, Clone, Copy, PartialEq)]
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
            location: Point::new(x, y),
            size: Size::wh(width, height),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T: Clone> Point<T> {
    pub fn new(x: T, y: T) -> Self {
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
            margin: Margin::uniform(DefiniteDimensionAuto::Points(0.0)),
            padding: Padding::uniform(DefiniteDimension::Points(0.0)),
            gap: Gap::uniform(DefiniteDimension::Points(0.0)),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Stretch,
            flex_wrap: FlexWrap::NoWrap,
        }
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
            margin: Margin::uniform(DefiniteDimensionAuto::Points(0.0)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dimension {
    Auto,
    Points(f32),
    Percent(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DefiniteDimension {
    Points(f32),
    Percent(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DefiniteDimensionAuto {
    Auto,
    Points(f32),
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

pub trait PointsDimension {
    fn points(value: f32) -> Self;
}

impl PointsDimension for Dimension {
    #[inline(always)]
    fn points(value: f32) -> Self {
        Dimension::Points(value)
    }
}

impl PointsDimension for DefiniteDimension {
    #[inline(always)]
    fn points(value: f32) -> Self {
        DefiniteDimension::Points(value)
    }
}

impl PointsDimension for DefiniteDimensionAuto {
    #[inline(always)]
    fn points(value: f32) -> Self {
        DefiniteDimensionAuto::Points(value)
    }
}

pub fn px<T: PointsDimension>(value: f32) -> T {
    T::points(value)
}

pub trait PercentDimension {
    fn percent(value: f32) -> Self;
}

impl PercentDimension for Dimension {
    #[inline(always)]
    fn percent(value: f32) -> Self {
        Dimension::Percent(value)
    }
}

impl PercentDimension for DefiniteDimension {
    #[inline(always)]
    fn percent(value: f32) -> Self {
        DefiniteDimension::Percent(value)
    }
}

impl PercentDimension for DefiniteDimensionAuto {
    #[inline(always)]
    fn percent(value: f32) -> Self {
        DefiniteDimensionAuto::Percent(value)
    }
}

pub fn percent<T: PercentDimension>(value: f32) -> T {
    T::percent(value)
}

pub trait AutoDimension {
    fn auto() -> Self;
}

impl AutoDimension for Dimension {
    #[inline(always)]
    fn auto() -> Self {
        Dimension::Auto
    }
}

impl AutoDimension for DefiniteDimensionAuto {
    #[inline(always)]
    fn auto() -> Self {
        DefiniteDimensionAuto::Auto
    }
}

pub fn auto<T: AutoDimension>() -> T {
    T::auto()
}
