use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Component {
    Text(TextComponent),
    Score(ScoreComponent),
    Selector(SelectorComponent),
    Keybind(KeybindComponent),
    Translation(TranslationComponent),
}

impl Component {
    pub fn append(mut self, child: Component) -> Self {
        match &mut self {
            Component::Text(c) => c.extra.push(child),
            Component::Score(c) => c.extra.push(child),
            Component::Selector(c) => c.extra.push(child),
            Component::Keybind(c) => c.extra.push(child),
            Component::Translation(c) => c.extra.push(child),
        }
        self
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Style {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub underlined: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strikethrough: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obfuscated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadow_color: Option<[f32; 4]>,
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bold(mut self, val: bool) -> Self {
        self.bold = Some(val);
        self
    }

    pub fn color(mut self, val: impl Into<String>) -> Self {
        self.color = Some(val.into());
        self
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TextComponent {
    pub text: String,
    #[serde(flatten)]
    pub style: Style,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub extra: Vec<Component>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TranslationComponent {
    pub translate: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with: Option<Vec<Component>>,
    #[serde(flatten)]
    pub style: Style,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub extra: Vec<Component>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ScoreComponent {
    pub score: ScoreData,
    #[serde(flatten)]
    pub style: Style,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub extra: Vec<Component>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ScoreData {
    pub name: String,
    pub objective: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SelectorComponent {
    pub selector: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub separator: Option<Box<Component>>,
    #[serde(flatten)]
    pub style: Style,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub extra: Vec<Component>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct KeybindComponent {
    pub keybind: String,
    #[serde(flatten)]
    pub style: Style,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub extra: Vec<Component>,
}

pub trait ComponentTrait {
    fn append(&mut self, child: Component) -> &mut Self
    where
        Self: Sized;
    fn style(&mut self, style: &Style) -> &mut Self
    where
        Self: Sized;
}

macro_rules! impl_component_trait {
    ( $( $ty:ident ),* $(,)? ) => {
        $(
            impl ComponentTrait for $ty {
                fn append(&mut self, child: Component) -> &mut Self {
                    self.extra.push(child);
                    self
                }

                fn style(&mut self, style: &Style) -> &mut Self {
                    self.style = style.clone();
                    self
                }
            }
        )*
    };
}

impl_component_trait!(
    TextComponent,
    ScoreComponent,
    SelectorComponent,
    KeybindComponent,
    TranslationComponent
);

#[macro_export]
macro_rules! keybind {
    ($key:expr) => {
        rustmine_lib::component::Component::Keybind(
            rustmine_lib::component::KeybindComponent {
                keybind: $key,
                style: rustmine_lib::component::Style::default(),
                extra: vec![],
            },
        )
    };
}

#[macro_export]
macro_rules! translation {
    ($key:expr) => {
        rustmine_lib::component::Component::Translation(
            rustmine_lib::component::TranslationComponent {
                key: $key,
                style: rustmine_lib::component::Style::default(),
                extra: vec![],
            },
        )
    };
}

#[macro_export]
macro_rules! text {
    ($txt:expr) => {
        rustmine_lib::component::Component::Text(
            rustmine_lib::component::TextComponent {
                text: $txt.to_string(),
                style: rustmine_lib::component::Style::default(),
                extra: vec![],
            },
        )
    };
}

#[macro_export]
macro_rules! score {
    ($name:expr, $objective:expr) => {
        rustmine_lib::component::Component::Score(
            rustmine_lib::component::ScoreComponent {
                score: rustmine_lib::component::ScoreData {
                    name: $name.to_string(),
                    objective: $objective.to_string(),
                },
                style: rustmine_lib::component::Style::default(),
                extra: vec![],
            },
        )
    };
}

#[macro_export]
macro_rules! selector {
    ($sel:expr) => {
        rustmine_lib::component::Component::Selector(
            rustmine_lib::component::SelectorComponent {
                selector: $sel.to_string(),
                separator: None,
                style: rustmine_lib::component::Style::default(),
                extra: vec![],
            },
        )
    };
}

#[macro_export]
macro_rules! styled {
    ($component:expr, { $($field:ident : $value:expr),* $(,)? }) => {{
        let mut c = $component;
        match &mut c {
            rustmine_lib::component::Component::Text(t) => {
                $( t.style.$field = Some($value); )*
            }
            rustmine_lib::component::Component::Score(s) => {
                $( s.style.$field = Some($value); )*
            }
            rustmine_lib::component::Component::Selector(sel) => {
                $( sel.style.$field = Some($value); )*
            }
            rustmine_lib::component::Component::Keybind(k) => {
                $( k.style.$field = Some($value); )*
            }
            rustmine_lib::component::Component::Translation(t) => {
                $( t.style.$field = Some($value); )*
            }
        }
        c
    }};
}
