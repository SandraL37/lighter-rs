use std::sync::Arc;
use std::time::Duration;

use crate::{
    core::{
        arena::{
            NodeArena,
            node::{EventHandlers, InteractionState, NodeId, NodeKind, NodeProps, NodePropsExt},
        },
        error::*,
        event::MouseEvents,
        layout::{ContainerStylePropsExt, LayoutStyle, LeafStylePropsExt},
        reactive::{
            bind::{DeferredBinding, bind_field},
            dirty::DirtyFlags,
            signal::MaybeSignal,
        },
        style::{Color, Transform},
    },
    elements::Element,
};

pub struct Div {
    node_props: NodeProps,
    layout_props: LayoutStyle,
    div_props: DivProps,
    children: Vec<Box<dyn Element>>,
    deferred_bindings: Vec<DeferredBinding>,
    event_handlers: EventHandlers,
}

impl std::fmt::Debug for Div {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Div")
            .field("node_props", &self.node_props)
            .field("layout_props", &self.layout_props)
            .field("div_props", &self.div_props)
            .finish()
    }
}

pub trait ChildrenExt: Sized {
    fn children_mut(&mut self) -> &mut Vec<Box<dyn Element>>;

    fn child(mut self, child: impl Element + 'static) -> Self {
        self.children_mut().push(Box::new(child));
        self
    }
}

pub trait DivPropsExt: Sized {
    fn div_ctx(&mut self) -> (&mut DivProps, &mut Vec<DeferredBinding>);

    fn bg(mut self, color: impl Into<MaybeSignal<Color>>) -> Self {
        let (props, bindings) = self.div_ctx();
        bind_field(
            &mut props.background_color,
            bindings,
            color,
            DirtyFlags::PAINT,
            |data, _, val| {
                data.kind.as_div_mut().background_color = val;
            },
        );
        self
    }

    fn rounded(mut self, radius: impl Into<MaybeSignal<f32>>) -> Self {
        let (props, bindings) = self.div_ctx();
        bind_field(
            &mut props.corner_radius,
            bindings,
            radius,
            DirtyFlags::PAINT,
            |data, _, val| {
                data.kind.as_div_mut().corner_radius = val;
            },
        );
        self
    }

    fn when_bg(mut self, state: InteractionState, color: Color) -> Self {
        self = self.when(state, |s| s.bg(color));
        self
    }

    fn style(mut self, apply: impl FnOnce(DivStylePatch) -> DivStylePatch) -> Self {
        let (props, _) = self.div_ctx();
        let current = std::mem::take(&mut props.base_style);
        props.base_style = apply(current);
        self
    }

    fn when(
        mut self,
        state: InteractionState,
        apply: impl FnOnce(DivStylePatch) -> DivStylePatch,
    ) -> Self {
        let (props, _) = self.div_ctx();
        props.state_styles.push((state, apply(DivStylePatch::default())));
        self
    }

    #[inline]
    fn hover_bg(self, color: Color) -> Self {
        self.when_bg(InteractionState::HOVER, color)
    }

    #[inline]
    fn active_bg(self, color: Color) -> Self {
        self.when_bg(InteractionState::ACTIVE, color)
    }

    #[inline]
    fn hover(self, apply: impl FnOnce(DivStylePatch) -> DivStylePatch) -> Self {
        self.when(InteractionState::HOVER, apply)
    }

    #[inline]
    fn active(self, apply: impl FnOnce(DivStylePatch) -> DivStylePatch) -> Self {
        self.when(InteractionState::ACTIVE, apply)
    }

    #[inline]
    fn focus(self, apply: impl FnOnce(DivStylePatch) -> DivStylePatch) -> Self {
        self.when(InteractionState::FOCUS, apply)
    }

    #[inline]
    fn disabled(self, apply: impl FnOnce(DivStylePatch) -> DivStylePatch) -> Self {
        self.when(InteractionState::DISABLED, apply)
    }

    fn transition(mut self, apply: impl FnOnce(DivTransition) -> DivTransition) -> Self {
        let (props, _) = self.div_ctx();
        let current = props.transition.unwrap_or_default();
        props.transition = Some(apply(current));
        self
    }
}

#[derive(Debug, Clone)]
pub struct DivProps {
    pub background_color: Color,
    pub corner_radius: f32, // TODO: make it DefiniteDimension
    pub base_style: DivStylePatch,
    pub state_styles: Vec<(InteractionState, DivStylePatch)>,
    pub transition: Option<DivTransition>,
}

impl Default for DivProps {
    fn default() -> Self {
        DivProps {
            background_color: Color::TRANSPARENT,
            corner_radius: 0.0,
            base_style: DivStylePatch::default(),
            state_styles: Vec::new(),
            transition: None,
        }
    }
}

impl DivProps {
    pub fn resolve_visual(
        &self,
        state: InteractionState,
        node_opacity: f32,
        node_transform: Option<Transform>,
    ) -> DivResolvedVisual {
        let mut out = DivResolvedVisual {
            background_color: self.background_color,
            corner_radius: self.corner_radius,
            opacity: node_opacity,
            transform: node_transform.unwrap_or(Transform::IDENTITY),
        };

        apply_patch(&mut out, &self.base_style);

        // Precedence: disabled > active > hover > focus > base.
        let layers = [
            InteractionState::FOCUS,
            InteractionState::HOVER,
            InteractionState::ACTIVE,
            InteractionState::DISABLED,
        ];

        for layer in layers {
            if !state.contains(layer) {
                continue;
            }

            for (mask, patch) in &self.state_styles {
                if *mask == layer {
                    apply_patch(&mut out, patch);
                }
            }
        }

        out
    }
}

#[derive(Debug, Clone, Default)]
pub struct DivStylePatch {
    pub background_color: Option<Color>,
    pub corner_radius: Option<f32>,
    pub opacity: Option<f32>,
    pub transform: Option<Transform>,
}

impl DivStylePatch {
    pub fn bg(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn rounded(mut self, radius: f32) -> Self {
        self.corner_radius = Some(radius);
        self
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = Some(opacity);
        self
    }

    pub fn transform(mut self, transform: Transform) -> Self {
        self.transform = Some(transform);
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DivResolvedVisual {
    pub background_color: Color,
    pub corner_radius: f32,
    pub opacity: f32,
    pub transform: Transform,
}

#[derive(Debug, Clone, Copy)]
pub enum TransitionEasing {
    Linear,
    OutCubic,
}

#[derive(Debug, Clone, Copy)]
pub struct DivTransition {
    pub opacity_ms: u32,
    pub transform_ms: u32,
    pub color_ms: u32,
    pub radius_ms: u32,
    pub easing: TransitionEasing,
}

impl Default for DivTransition {
    fn default() -> Self {
        Self {
            opacity_ms: 0,
            transform_ms: 0,
            color_ms: 0,
            radius_ms: 0,
            easing: TransitionEasing::Linear,
        }
    }
}

impl DivTransition {
    pub fn opacity(mut self, ms: u32) -> Self {
        self.opacity_ms = ms;
        self
    }
    pub fn transform(mut self, ms: u32) -> Self {
        self.transform_ms = ms;
        self
    }
    pub fn color(mut self, ms: u32) -> Self {
        self.color_ms = ms;
        self
    }
    pub fn radius(mut self, ms: u32) -> Self {
        self.radius_ms = ms;
        self
    }
    pub fn ease(mut self, easing: TransitionEasing) -> Self {
        self.easing = easing;
        self
    }
}

#[derive(Debug, Clone)]
pub struct DivAnimationState {
    pub from: DivResolvedVisual,
    pub to: DivResolvedVisual,
    pub started: std::time::Instant,
    pub transition: DivTransition,
}

impl DivAnimationState {
    pub fn at(&self, now: std::time::Instant) -> DivResolvedVisual {
        let dt = now.saturating_duration_since(self.started);
        DivResolvedVisual {
            background_color: lerp_color(
                self.from.background_color,
                self.to.background_color,
                ease(progress(dt, self.transition.color_ms), self.transition.easing),
            ),
            corner_radius: lerp_f32(
                self.from.corner_radius,
                self.to.corner_radius,
                ease(progress(dt, self.transition.radius_ms), self.transition.easing),
            ),
            opacity: lerp_f32(
                self.from.opacity,
                self.to.opacity,
                ease(progress(dt, self.transition.opacity_ms), self.transition.easing),
            ),
            transform: lerp_transform(
                self.from.transform,
                self.to.transform,
                ease(progress(dt, self.transition.transform_ms), self.transition.easing),
            ),
        }
    }

    pub fn finished(&self, now: std::time::Instant) -> bool {
        let dt = now.saturating_duration_since(self.started).as_millis() as u32;
        let max_ms = self
            .transition
            .opacity_ms
            .max(self.transition.transform_ms)
            .max(self.transition.color_ms)
            .max(self.transition.radius_ms);
        dt >= max_ms
    }
}

fn progress(dt: Duration, ms: u32) -> f32 {
    if ms == 0 {
        return 1.0;
    }
    (dt.as_secs_f32() / (ms as f32 / 1000.0)).clamp(0.0, 1.0)
}

fn ease(t: f32, easing: TransitionEasing) -> f32 {
    match easing {
        TransitionEasing::Linear => t,
        TransitionEasing::OutCubic => 1.0 - (1.0 - t).powi(3),
    }
}

fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color::rgba(
        lerp_f32(a.r, b.r, t),
        lerp_f32(a.g, b.g, t),
        lerp_f32(a.b, b.b, t),
        lerp_f32(a.a, b.a, t),
    )
}

fn lerp_transform(a: Transform, b: Transform, t: f32) -> Transform {
    let mut out = [0.0; 6];
    for (i, item) in out.iter_mut().enumerate() {
        *item = lerp_f32(a.matrix[i], b.matrix[i], t);
    }
    Transform { matrix: out }
}

fn apply_patch(out: &mut DivResolvedVisual, patch: &DivStylePatch) {
    if let Some(bg) = patch.background_color {
        out.background_color = bg;
    }
    if let Some(radius) = patch.corner_radius {
        out.corner_radius = radius;
    }
    if let Some(opacity) = patch.opacity {
        out.opacity = opacity;
    }
    if let Some(transform) = patch.transform {
        out.transform = transform;
    }
}

impl Element for Div {
    fn build(self: Box<Self>, arena: &mut NodeArena, parent: Option<NodeId>) -> Result<NodeId> {
        let id = arena.create_node(
            NodeKind::Div(Arc::new(self.div_props)),
            self.node_props,
            parent,
            self.layout_props,
            self.event_handlers,
        )?;

        for binding in self.deferred_bindings {
            (binding.0)(id);
        }

        for child in self.children {
            child.build(arena, Some(id))?;
        }

        Ok(id)
    }
}

impl LeafStylePropsExt for Div {
    fn ctx(&mut self) -> (&mut LayoutStyle, &mut Vec<DeferredBinding>) {
        (&mut self.layout_props, &mut self.deferred_bindings)
    }
}

impl ContainerStylePropsExt for Div {}

impl NodePropsExt for Div {
    fn node_ctx(&mut self) -> (&mut NodeProps, &mut Vec<DeferredBinding>) {
        (&mut self.node_props, &mut self.deferred_bindings)
    }
}

impl DivPropsExt for Div {
    fn div_ctx(&mut self) -> (&mut DivProps, &mut Vec<DeferredBinding>) {
        (&mut self.div_props, &mut self.deferred_bindings)
    }
}

impl ChildrenExt for Div {
    fn children_mut(&mut self) -> &mut Vec<Box<dyn Element>> {
        &mut self.children
    }
}

pub fn div() -> Div {
    Div {
        node_props: NodeProps::default(),
        layout_props: LayoutStyle::default(),
        children: Vec::new(),
        div_props: DivProps::default(),
        deferred_bindings: Vec::new(),
        event_handlers: EventHandlers::default(),
    }
}

impl MouseEvents for Div {
    fn event_handlers(&mut self) -> &mut EventHandlers {
        &mut self.event_handlers
    }
}
