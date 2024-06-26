use gpui::{
    div, rems, IntoElement, ParentElement, Rems, RenderOnce, SharedString, Styled, WindowContext,
};
use settings::Settings;
use theme::{ActiveTheme, ThemeSettings};

use crate::rems_from_px;

#[derive(Debug, Default, Clone)]
pub enum UiTextSize {
    /// The default size for UI text.
    ///
    /// `0.825rem` or `14px` at the default scale of `1rem` = `16px`.
    ///
    /// Note: The absolute size of this text will change based on a user's `ui_scale` setting.
    #[default]
    Default,
    /// The large size for UI text.
    ///
    /// `1rem` or `16px` at the default scale of `1rem` = `16px`.
    ///
    /// Note: The absolute size of this text will change based on a user's `ui_scale` setting.
    Large,

    /// The small size for UI text.
    ///
    /// `0.75rem` or `12px` at the default scale of `1rem` = `16px`.
    ///
    /// Note: The absolute size of this text will change based on a user's `ui_scale` setting.
    Small,

    /// The extra small size for UI text.
    ///
    /// `0.625rem` or `10px` at the default scale of `1rem` = `16px`.
    ///
    /// Note: The absolute size of this text will change based on a user's `ui_scale` setting.
    XSmall,
}

impl UiTextSize {
    pub fn rems(self) -> Rems {
        match self {
            Self::Large => rems_from_px(16.),
            Self::Default => rems_from_px(14.),
            Self::Small => rems_from_px(12.),
            Self::XSmall => rems_from_px(10.),
        }
    }
}

/// The size of a [`Headline`] element
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default)]
pub enum HeadlineSize {
    XSmall,
    Small,
    #[default]
    Medium,
    Large,
    XLarge,
}

impl HeadlineSize {
    pub fn size(self) -> Rems {
        match self {
            // Based on the Major Second scale
            Self::XSmall => rems(0.88),
            Self::Small => rems(1.0),
            Self::Medium => rems(1.125),
            Self::Large => rems(1.27),
            Self::XLarge => rems(1.43),
        }
    }

    pub fn line_height(self) -> Rems {
        match self {
            Self::XSmall => rems(1.6),
            Self::Small => rems(1.6),
            Self::Medium => rems(1.6),
            Self::Large => rems(1.6),
            Self::XLarge => rems(1.6),
        }
    }
}

#[derive(IntoElement)]
pub struct Headline {
    size: HeadlineSize,
    text: SharedString,
}

impl RenderOnce for Headline {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let ui_font = ThemeSettings::get_global(cx).ui_font.family.clone();

        div()
            .font_family(ui_font)
            .line_height(self.size.line_height())
            .text_size(self.size.size())
            .text_color(cx.theme().colors().text)
            .child(self.text)
    }
}

impl Headline {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            size: HeadlineSize::default(),
            text: text.into(),
        }
    }

    pub fn size(mut self, size: HeadlineSize) -> Self {
        self.size = size;
        self
    }
}
