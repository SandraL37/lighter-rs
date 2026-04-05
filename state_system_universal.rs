//! Production-grade universal interaction state system.
//!
//! Design goals:
//! - Universal: works for any element kind (div/text/img/custom).
//! - Typed: no string matching, no property-group enum.
//! - Performant: interned `PropertyId` keys via `slotmap`.
//! - Safe: optional validation path with explicit error types.
//! - Ergonomic: builders for state patches and transition config.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use slotmap::{SecondaryMap, SlotMap, new_key_type};

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct InteractionState: u8 {
        const HOVER    = 1 << 0;
        const ACTIVE   = 1 << 1;
        const FOCUS    = 1 << 2;
        const DISABLED = 1 << 3;
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct DirtyFlags: u8 {
        const PAINT  = 1 << 0;
        const LAYOUT = 1 << 1;
        const OTHER  = 1 << 2;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const TRANSPARENT: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn lerp(self, other: Color, t: f32) -> Self {
        Self {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub matrix: [f32; 6],
}

impl Transform {
    pub const IDENTITY: Transform = Transform {
        matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    };

    pub fn scale(s: f32) -> Self {
        Self {
            matrix: [s, 0.0, 0.0, s, 0.0, 0.0],
        }
    }
}

new_key_type! {
    pub struct PropertyId;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PropertyKeyDef {
    Node(NodeProperty),
    Div(DivProperty),
    Text(TextProperty),
    Img(ImgProperty),
    Custom(CustomPropertyKey),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeProperty {
    Opacity,
    Transform,
    TransformScale,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DivProperty {
    BackgroundColor,
    CornerRadius,
    Width,
    Height,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TextProperty {
    Content,
    Color,
    FontSize,
    FontFamily,
    FontWeight,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImgProperty {
    Source,
    Tint,
    FitMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CustomPropertyKey {
    pub namespace: Arc<str>,
    pub name: Arc<str>,
}

impl CustomPropertyKey {
    pub fn new(namespace: impl Into<Arc<str>>, name: impl Into<Arc<str>>) -> Self {
        Self {
            namespace: namespace.into(),
            name: name.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    F32,
    I32,
    U16,
    Bool,
    Text,
    Color,
    Transform,
}

#[derive(Debug, Clone, Copy)]
pub struct PropertyMeta {
    pub kind: ValueKind,
    pub dirty: DirtyFlags,
}

#[derive(Debug, Default)]
pub struct PropertyRegistry {
    keys: SlotMap<PropertyId, PropertyKeyDef>,
    key_index: HashMap<PropertyKeyDef, PropertyId>,
    meta: SecondaryMap<PropertyId, PropertyMeta>,
}

impl PropertyRegistry {
    pub fn intern(&mut self, key: PropertyKeyDef, meta: PropertyMeta) -> PropertyId {
        if let Some(id) = self.key_index.get(&key) {
            self.meta.insert(*id, meta);
            return *id;
        }
        let id = self.keys.insert(key.clone());
        self.key_index.insert(key, id);
        self.meta.insert(id, meta);
        id
    }

    pub fn resolve_key(&self, id: PropertyId) -> Option<&PropertyKeyDef> {
        self.keys.get(id)
    }

    pub fn meta(&self, id: PropertyId) -> Option<PropertyMeta> {
        self.meta.get(id).copied()
    }

    pub fn kind(&self, id: PropertyId) -> Option<ValueKind> {
        self.meta(id).map(|m| m.kind)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    F32(f32),
    I32(i32),
    U16(u16),
    Bool(bool),
    Text(Arc<str>),
    Color(Color),
    Transform(Transform),
}

impl PropertyValue {
    pub fn kind(&self) -> ValueKind {
        match self {
            PropertyValue::F32(_) => ValueKind::F32,
            PropertyValue::I32(_) => ValueKind::I32,
            PropertyValue::U16(_) => ValueKind::U16,
            PropertyValue::Bool(_) => ValueKind::Bool,
            PropertyValue::Text(_) => ValueKind::Text,
            PropertyValue::Color(_) => ValueKind::Color,
            PropertyValue::Transform(_) => ValueKind::Transform,
        }
    }

    fn lerp(&self, to: &PropertyValue, t: f32) -> Option<PropertyValue> {
        match (self, to) {
            (PropertyValue::F32(a), PropertyValue::F32(b)) => Some(PropertyValue::F32(a + (b - a) * t)),
            (PropertyValue::Color(a), PropertyValue::Color(b)) => Some(PropertyValue::Color(a.lerp(*b, t))),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateError {
    UnknownProperty(PropertyId),
    KindMismatch {
        property: PropertyId,
        expected: ValueKind,
        got: ValueKind,
    },
    NonInterpolableProperty(PropertyId),
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Easing {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseOutCubic,
}

impl Easing {
    pub fn apply(self, t: f32) -> f32 {
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Easing::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TransitionSpec {
    pub duration: Duration,
    pub delay: Duration,
    pub easing: Easing,
}

impl TransitionSpec {
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            delay: Duration::ZERO,
            easing: Easing::Linear,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TransitionConfig {
    per_property: SecondaryMap<PropertyId, TransitionSpec>,
    fallback: Option<TransitionSpec>,
}

impl TransitionConfig {
    pub fn get(&self, property: PropertyId) -> Option<TransitionSpec> {
        self.per_property.get(property).copied().or(self.fallback)
    }

    pub fn set_property(&mut self, property: PropertyId, spec: TransitionSpec) {
        self.per_property.insert(property, spec);
    }
}

#[derive(Debug, Clone, Default)]
pub struct TransitionBuilder {
    config: TransitionConfig,
}

impl TransitionBuilder {
    pub fn property(mut self, property: PropertyId, duration: Duration) -> Self {
        self.config
            .per_property
            .insert(property, TransitionSpec::new(duration));
        self
    }

    pub fn all(mut self, duration: Duration) -> Self {
        self.config.fallback = Some(TransitionSpec::new(duration));
        self
    }

    pub fn ease(mut self, property: PropertyId, easing: Easing) -> Self {
        if let Some(spec) = self.config.per_property.get_mut(property) {
            spec.easing = easing;
        }
        self
    }

    pub fn delay(mut self, property: PropertyId, delay: Duration) -> Self {
        if let Some(spec) = self.config.per_property.get_mut(property) {
            spec.delay = delay;
        }
        self
    }

    pub fn build(self) -> TransitionConfig {
        self.config
    }
}

pub type StyleMap = HashMap<PropertyId, PropertyValue>;
pub type PatchMap = HashMap<PropertyId, PropertyValue>;

#[derive(Debug, Clone, Default)]
pub struct StatePatches {
    pub hover: PatchMap,
    pub active: PatchMap,
    pub focus: PatchMap,
    pub disabled: PatchMap,
}

impl StatePatches {
    pub fn resolve(&self, state: InteractionState) -> PatchMap {
        if state.contains(InteractionState::DISABLED) {
            return self.disabled.clone();
        }

        let mut cap = 0usize;
        if state.contains(InteractionState::FOCUS) {
            cap += self.focus.len();
        }
        if state.contains(InteractionState::HOVER) {
            cap += self.hover.len();
        }
        if state.contains(InteractionState::ACTIVE) {
            cap += self.active.len();
        }

        let mut out = HashMap::with_capacity(cap);
        if state.contains(InteractionState::FOCUS) {
            merge_patch_into(&mut out, &self.focus);
        }
        if state.contains(InteractionState::HOVER) {
            merge_patch_into(&mut out, &self.hover);
        }
        if state.contains(InteractionState::ACTIVE) {
            merge_patch_into(&mut out, &self.active);
        }
        out
    }
}

#[inline]
fn merge_patch_into(dst: &mut PatchMap, src: &PatchMap) {
    for (k, v) in src {
        dst.insert(*k, v.clone());
    }
}

pub fn apply_patch_fast(
    style: &mut StyleMap,
    patch: &PatchMap,
    classify_dirty: impl Fn(PropertyId) -> DirtyFlags,
) -> DirtyFlags {
    let mut dirty = DirtyFlags::empty();
    for (k, v) in patch {
        style.insert(*k, v.clone());
        dirty |= classify_dirty(*k);
    }
    dirty
}

pub fn apply_patch_checked(
    style: &mut StyleMap,
    patch: &PatchMap,
    registry: &PropertyRegistry,
) -> Result<DirtyFlags, StateError> {
    let mut dirty = DirtyFlags::empty();
    for (k, v) in patch {
        let meta = registry.meta(*k).ok_or(StateError::UnknownProperty(*k))?;
        let got = v.kind();
        if meta.kind != got {
            return Err(StateError::KindMismatch {
                property: *k,
                expected: meta.kind,
                got,
            });
        }
        style.insert(*k, v.clone());
        dirty |= meta.dirty;
    }
    Ok(dirty)
}

#[derive(Debug, Clone)]
pub struct AnimationTrack {
    pub property: PropertyId,
    pub from: PropertyValue,
    pub to: PropertyValue,
    pub duration: Duration,
    pub delay: Duration,
    pub easing: Easing,
    pub elapsed: Duration,
}

impl AnimationTrack {
    pub fn tick(&mut self, dt: Duration) {
        self.elapsed += dt;
    }

    pub fn is_complete(&self) -> bool {
        self.elapsed >= self.duration + self.delay
    }

    pub fn progress(&self) -> f32 {
        if self.duration.is_zero() {
            return 1.0;
        }
        if self.elapsed <= self.delay {
            return 0.0;
        }
        let active = self.elapsed.saturating_sub(self.delay);
        let t = (active.as_secs_f32() / self.duration.as_secs_f32()).clamp(0.0, 1.0);
        self.easing.apply(t)
    }

    pub fn current_value(&self) -> PropertyValue {
        let t = self.progress();
        self.from
            .lerp(&self.to, t)
            .unwrap_or_else(|| if t >= 1.0 { self.to.clone() } else { self.from.clone() })
    }
}

#[derive(Debug, Clone, Default)]
pub struct AnimationState {
    pub tracks: SecondaryMap<PropertyId, AnimationTrack>,
}

impl AnimationState {
    pub fn is_animating(&self) -> bool {
        self.tracks.iter().next().is_some()
    }

    pub fn tick(&mut self, dt: Duration) -> bool {
        for (_, track) in self.tracks.iter_mut() {
            track.tick(dt);
        }
        let completed: Vec<PropertyId> = self
            .tracks
            .iter()
            .filter_map(|(k, t)| if t.is_complete() { Some(k) } else { None })
            .collect();
        for id in completed {
            self.tracks.remove(id);
        }
        self.is_animating()
    }

    pub fn apply_to(&self, style: &mut StyleMap) {
        for (id, track) in self.tracks.iter() {
            style.insert(id, track.current_value());
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationMode {
    BestEffort,
    Strict,
}

pub fn create_animations(
    current: &StyleMap,
    target: &StyleMap,
    transitions: &TransitionConfig,
    mode: AnimationMode,
) -> Result<AnimationState, StateError> {
    let mut state = AnimationState::default();

    for (property, from) in current {
        let Some(to) = target.get(property) else {
            continue;
        };
        if from == to {
            continue;
        }

        let Some(spec) = transitions.get(*property) else {
            continue;
        };

        if from.lerp(to, 0.5).is_none() {
            if mode == AnimationMode::Strict {
                return Err(StateError::NonInterpolableProperty(*property));
            }
            continue;
        }

        state.tracks.insert(
            *property,
            AnimationTrack {
                property: *property,
                from: from.clone(),
                to: to.clone(),
                duration: spec.duration,
                delay: spec.delay,
                easing: spec.easing,
                elapsed: Duration::ZERO,
            },
        );
    }

    Ok(state)
}

pub const fn ms(v: u64) -> Duration {
    Duration::from_millis(v)
}

#[derive(Debug, Clone, Copy)]
pub struct LighterPropertyIds {
    pub node_opacity: PropertyId,
    pub node_transform: PropertyId,
    pub node_transform_scale: PropertyId,
    pub div_background_color: PropertyId,
    pub div_corner_radius: PropertyId,
    pub text_content: PropertyId,
    pub text_color: PropertyId,
    pub text_font_size: PropertyId,
    pub text_font_family: PropertyId,
    pub text_font_weight: PropertyId,
}

impl LighterPropertyIds {
    pub fn intern(registry: &mut PropertyRegistry) -> Self {
        Self {
            node_opacity: registry.intern(
                PropertyKeyDef::Node(NodeProperty::Opacity),
                PropertyMeta {
                    kind: ValueKind::F32,
                    dirty: DirtyFlags::PAINT,
                },
            ),
            node_transform: registry.intern(
                PropertyKeyDef::Node(NodeProperty::Transform),
                PropertyMeta {
                    kind: ValueKind::Transform,
                    dirty: DirtyFlags::PAINT,
                },
            ),
            node_transform_scale: registry.intern(
                PropertyKeyDef::Node(NodeProperty::TransformScale),
                PropertyMeta {
                    kind: ValueKind::F32,
                    dirty: DirtyFlags::PAINT,
                },
            ),
            div_background_color: registry.intern(
                PropertyKeyDef::Div(DivProperty::BackgroundColor),
                PropertyMeta {
                    kind: ValueKind::Color,
                    dirty: DirtyFlags::PAINT,
                },
            ),
            div_corner_radius: registry.intern(
                PropertyKeyDef::Div(DivProperty::CornerRadius),
                PropertyMeta {
                    kind: ValueKind::F32,
                    dirty: DirtyFlags::PAINT,
                },
            ),
            text_content: registry.intern(
                PropertyKeyDef::Text(TextProperty::Content),
                PropertyMeta {
                    kind: ValueKind::Text,
                    dirty: DirtyFlags::PAINT | DirtyFlags::LAYOUT,
                },
            ),
            text_color: registry.intern(
                PropertyKeyDef::Text(TextProperty::Color),
                PropertyMeta {
                    kind: ValueKind::Color,
                    dirty: DirtyFlags::PAINT,
                },
            ),
            text_font_size: registry.intern(
                PropertyKeyDef::Text(TextProperty::FontSize),
                PropertyMeta {
                    kind: ValueKind::F32,
                    dirty: DirtyFlags::PAINT | DirtyFlags::LAYOUT,
                },
            ),
            text_font_family: registry.intern(
                PropertyKeyDef::Text(TextProperty::FontFamily),
                PropertyMeta {
                    kind: ValueKind::Text,
                    dirty: DirtyFlags::PAINT | DirtyFlags::LAYOUT,
                },
            ),
            text_font_weight: registry.intern(
                PropertyKeyDef::Text(TextProperty::FontWeight),
                PropertyMeta {
                    kind: ValueKind::U16,
                    dirty: DirtyFlags::PAINT | DirtyFlags::LAYOUT,
                },
            ),
        }
    }
}

pub mod lighter_adapters {
    use super::*;
    use crate::core::{
        arena::node::NodeStyle as LighterNodeStyle,
        reactive::dirty::DirtyFlags as LighterDirtyFlags,
        style::{Color as LighterColor, Transform as LighterTransform},
    };
    use crate::elements::{
        div::style::DivStyle,
        text::style::{FontWeight, TextStyle},
    };

    #[inline]
    fn to_u(c: LighterColor) -> Color {
        Color {
            r: c.r,
            g: c.g,
            b: c.b,
            a: c.a,
        }
    }

    #[inline]
    fn to_l(c: Color) -> LighterColor {
        LighterColor {
            r: c.r,
            g: c.g,
            b: c.b,
            a: c.a,
        }
    }

    #[inline]
    fn transform_to_u(t: LighterTransform) -> Transform {
        Transform { matrix: t.matrix }
    }

    #[inline]
    fn transform_to_l(t: Transform) -> LighterTransform {
        LighterTransform { matrix: t.matrix }
    }

    pub fn map_node(node: &LighterNodeStyle, ids: &LighterPropertyIds) -> StyleMap {
        let mut out = StyleMap::with_capacity(3);
        out.insert(ids.node_opacity, PropertyValue::F32(node.opacity));
        if let Some(t) = node.transform {
            out.insert(ids.node_transform, PropertyValue::Transform(transform_to_u(t)));
            out.insert(ids.node_transform_scale, PropertyValue::F32(t.matrix[0]));
        }
        out
    }

    pub fn apply_node(node: &mut LighterNodeStyle, patch: &PatchMap, ids: &LighterPropertyIds) -> LighterDirtyFlags {
        let mut dirty = LighterDirtyFlags::empty();
        for (id, value) in patch {
            if *id == ids.node_opacity {
                if let PropertyValue::F32(v) = value {
                    node.opacity = *v;
                    dirty |= LighterDirtyFlags::PAINT;
                }
            } else if *id == ids.node_transform {
                if let PropertyValue::Transform(t) = value {
                    node.transform = Some(transform_to_l(*t));
                    dirty |= LighterDirtyFlags::PAINT;
                }
            } else if *id == ids.node_transform_scale {
                if let PropertyValue::F32(s) = value {
                    node.transform = Some(LighterTransform {
                        matrix: [*s, 0.0, 0.0, *s, 0.0, 0.0],
                    });
                    dirty |= LighterDirtyFlags::PAINT;
                }
            }
        }
        dirty
    }

    pub fn map_div(div: &DivStyle, ids: &LighterPropertyIds) -> StyleMap {
        let mut out = StyleMap::with_capacity(2);
        out.insert(
            ids.div_background_color,
            PropertyValue::Color(to_u(div.background_color)),
        );
        out.insert(ids.div_corner_radius, PropertyValue::F32(div.corner_radius));
        out
    }

    pub fn apply_div(div: &mut DivStyle, patch: &PatchMap, ids: &LighterPropertyIds) -> LighterDirtyFlags {
        let mut dirty = LighterDirtyFlags::empty();
        for (id, value) in patch {
            if *id == ids.div_background_color {
                if let PropertyValue::Color(c) = value {
                    div.background_color = to_l(*c);
                    dirty |= LighterDirtyFlags::PAINT;
                }
            } else if *id == ids.div_corner_radius {
                if let PropertyValue::F32(r) = value {
                    div.corner_radius = *r;
                    dirty |= LighterDirtyFlags::PAINT;
                }
            }
        }
        dirty
    }

    pub fn map_text(text: &TextStyle, ids: &LighterPropertyIds) -> StyleMap {
        let mut out = StyleMap::with_capacity(5);
        out.insert(ids.text_content, PropertyValue::Text(Arc::clone(&text.content)));
        out.insert(ids.text_color, PropertyValue::Color(to_u(text.color)));
        out.insert(ids.text_font_size, PropertyValue::F32(text.font_size));
        out.insert(
            ids.text_font_family,
            PropertyValue::Text(Arc::clone(&text.font_family)),
        );
        out.insert(ids.text_font_weight, PropertyValue::U16(text.font_weight.0));
        out
    }

    pub fn apply_text(text: &mut TextStyle, patch: &PatchMap, ids: &LighterPropertyIds) -> LighterDirtyFlags {
        let mut dirty = LighterDirtyFlags::empty();
        for (id, value) in patch {
            if *id == ids.text_content {
                if let PropertyValue::Text(v) = value {
                    text.content = Arc::clone(v);
                    dirty |= LighterDirtyFlags::PAINT | LighterDirtyFlags::LAYOUT;
                }
            } else if *id == ids.text_color {
                if let PropertyValue::Color(c) = value {
                    text.color = to_l(*c);
                    dirty |= LighterDirtyFlags::PAINT;
                }
            } else if *id == ids.text_font_size {
                if let PropertyValue::F32(s) = value {
                    text.font_size = *s;
                    dirty |= LighterDirtyFlags::PAINT | LighterDirtyFlags::LAYOUT;
                }
            } else if *id == ids.text_font_family {
                if let PropertyValue::Text(v) = value {
                    text.font_family = Arc::clone(v);
                    dirty |= LighterDirtyFlags::PAINT | LighterDirtyFlags::LAYOUT;
                }
            } else if *id == ids.text_font_weight {
                if let PropertyValue::U16(w) = value {
                    text.font_weight = FontWeight::new(*w);
                    dirty |= LighterDirtyFlags::PAINT | LighterDirtyFlags::LAYOUT;
                }
            }
        }
        dirty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_precedence_active_over_hover() {
        let mut patches = StatePatches::default();
        let p = PropertyId::null();
        patches.hover.insert(p, PropertyValue::F32(1.0));
        patches.active.insert(p, PropertyValue::F32(2.0));

        let merged = patches.resolve(InteractionState::HOVER | InteractionState::ACTIVE);
        assert_eq!(merged.get(&p), Some(&PropertyValue::F32(2.0)));
    }

    #[test]
    fn disabled_is_exclusive() {
        let mut patches = StatePatches::default();
        let p = PropertyId::null();
        patches.hover.insert(p, PropertyValue::F32(1.0));
        patches.disabled.insert(p, PropertyValue::F32(3.0));
        let merged = patches.resolve(InteractionState::HOVER | InteractionState::DISABLED);
        assert_eq!(merged.get(&p), Some(&PropertyValue::F32(3.0)));
    }

    #[test]
    fn checked_patch_validates_kind() {
        let mut registry = PropertyRegistry::default();
        let p = registry.intern(
            PropertyKeyDef::Node(NodeProperty::Opacity),
            PropertyMeta {
                kind: ValueKind::F32,
                dirty: DirtyFlags::PAINT,
            },
        );

        let mut style = StyleMap::new();
        let mut patch = PatchMap::new();
        patch.insert(p, PropertyValue::Bool(true));

        let result = apply_patch_checked(&mut style, &patch, &registry);
        assert!(matches!(result, Err(StateError::KindMismatch { .. })));
    }
}
