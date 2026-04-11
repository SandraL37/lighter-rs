use crate::{
    core::{
        animation::{ Duration, Easing, TransitionDir, TransitionSpec },
        interaction::InteractionState,
        reactive::signal::{ MaybeSignal, Signal },
        style::Color,
    },
    elements::text::style::FontWeight,
};

#[derive(Clone, Copy, Debug)]
pub enum PropertyValue<T> {
    Static(T),
}

impl<T> PropertyValue<T> {
    pub fn get(self) -> T {
        match self {
            PropertyValue::Static(v) => v,
        }
    }
}

impl<T> From<T> for PropertyValue<T> {
    fn from(v: T) -> Self {
        PropertyValue::Static(v)
    }
}

#[derive(Clone, Debug)]
pub struct StateValue<T> {
    pub value: PropertyValue<T>,
    pub transition: TransitionSpec,
}

impl<T> StateValue<T> {
    pub fn instant(v: impl Into<PropertyValue<T>>) -> Self {
        StateValue {
            value: v.into(),
            transition: TransitionSpec::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Property<T> {
    pub base: PropertyValue<T>,
    pub hover: Option<StateValue<T>>,
    pub active: Option<StateValue<T>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComputedState {
    Base = 0,
    Hover = 1,
    Active = 2,
}

impl<T> Property<T> {
    pub fn base(value: impl Into<PropertyValue<T>>) -> Self {
        Self {
            base: value.into(),
            hover: None,
            active: None,
        }
    }

    pub fn hover(self, v: impl Into<PropertyValue<T>>) -> StateBuilder<T> {
        StateBuilder::new(self, ComputedState::Hover, v.into())
    }

    pub fn active(self, v: impl Into<PropertyValue<T>>) -> StateBuilder<T> {
        StateBuilder::new(self, ComputedState::Active, v.into())
    }

    pub(crate) fn resolve(&self, state: InteractionState) -> &PropertyValue<T> {
        let computed_state = ComputedState::from(state);
        let slot = match computed_state {
            ComputedState::Base => None,
            ComputedState::Hover => self.hover.as_ref(),
            // If active is not defined, keep hover styling while pressed.
            ComputedState::Active => self.active.as_ref().or(self.hover.as_ref()),
        };
        slot.map(|sv| &sv.value).unwrap_or(&self.base)
    }

    fn set_state(&mut self, state: ComputedState, sv: StateValue<T>) {
        match state {
            ComputedState::Base => {
                self.base = sv.value;
            }
            ComputedState::Hover => {
                self.hover = Some(sv);
            }
            ComputedState::Active => {
                self.active = Some(sv);
            }
        }
    }
}

impl<T> From<T> for Property<T> {
    fn from(value: T) -> Self {
        Property::base(PropertyValue::Static(value))
    }
}

pub struct StateBuilder<T> {
    property: Property<T>,
    state: ComputedState,
    sv: StateValue<T>,
}

impl<T> StateBuilder<T> {
    fn new(property: Property<T>, state: ComputedState, value: PropertyValue<T>) -> Self {
        StateBuilder {
            property,
            state,
            sv: StateValue::instant(value),
        }
    }

    pub fn hover(self, v: impl Into<PropertyValue<T>>) -> StateBuilder<T> {
        let prop = self.finalize();
        prop.hover(v)
    }

    pub fn active(self, v: impl Into<PropertyValue<T>>) -> StateBuilder<T> {
        let prop = self.finalize();
        prop.active(v)
    }

    #[inline(always)]
    pub fn transition(self, duration: Duration, easing: Easing) -> Self {
        self.enter(duration, easing).exit(duration, easing)
    }

    pub fn enter(mut self, duration: Duration, easing: Easing) -> Self {
        self.sv.transition.enter = Some(TransitionDir { duration, easing });
        self
    }

    pub fn exit(mut self, duration: Duration, easing: Easing) -> Self {
        self.sv.transition.exit = Some(TransitionDir { duration, easing });
        self
    }

    fn finalize(mut self) -> Property<T> {
        self.property.set_state(self.state, self.sv);
        self.property
    }
}

impl<T> From<StateBuilder<T>> for Property<T> {
    fn from(value: StateBuilder<T>) -> Self {
        value.finalize()
    }
}

pub type StyleProp<T> = MaybeSignal<Property<T>>;

pub trait IntoStyleProp<T> {
    fn into_style_prop(self) -> StyleProp<T>;
}

impl<T> IntoStyleProp<T> for T {
    fn into_style_prop(self) -> StyleProp<T> {
        MaybeSignal::Static(Property::base(PropertyValue::Static(self)))
    }
}

impl<T> IntoStyleProp<T> for Property<T> {
    fn into_style_prop(self) -> StyleProp<T> {
        MaybeSignal::Static(self)
    }
}

impl<T> IntoStyleProp<T> for StateBuilder<T> {
    fn into_style_prop(self) -> StyleProp<T> {
        MaybeSignal::Static(self.finalize())
    }
}

impl<T> IntoStyleProp<T> for Signal<Property<T>> {
    fn into_style_prop(self) -> StyleProp<T> {
        MaybeSignal::Signal(self)
    }
}

impl<T: Clone> IntoStyleProp<T> for Signal<T> {
    fn into_style_prop(self) -> StyleProp<T> {
        MaybeSignal::Signal(self.map(Property::base))
    }
}

pub trait StateExt<T>: Sized + Into<T> {
    fn hover(self, v: T) -> StateBuilder<T> {
        Property::base(PropertyValue::Static(self.into())).hover(v)
    }
    fn active(self, v: T) -> StateBuilder<T> {
        Property::base(PropertyValue::Static(self.into())).active(v)
    }
}

impl StateExt<f32> for f32 {}
impl StateExt<Color> for Color {}
impl StateExt<Signal<Color>> for Signal<Color> {}
impl StateExt<FontWeight> for FontWeight {}
