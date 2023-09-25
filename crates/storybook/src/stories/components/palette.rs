use gpui2::elements::div::ScrollState;
use gpui2::{Element, IntoElement, ParentElement, ViewContext};
use ui::{ModifierKey, Palette, PaletteItem};

use crate::story::Story;

#[derive(Element, Default)]
pub struct PaletteStory {}

impl PaletteStory {
    fn render<V: 'static>(&mut self, _: &mut V, cx: &mut ViewContext<V>) -> impl IntoElement<V> {
        Story::container(cx)
            .child(Story::title_for::<_, Palette<V>>(cx))
            .child(Story::label(cx, "Default"))
            .child(Palette::new(ScrollState::default()))
            .child(Story::label(cx, "With Items"))
            .child(
                Palette::new(ScrollState::default())
                    .placeholder("Execute a command...")
                    .items(vec![
                        PaletteItem::new("theme selector: toggle"),
                        // TODO: Add support for chords.
                        // .keybinding(Some("Cmd-k, Cmd-t")),
                        PaletteItem::new("assistant: inline assist")
                            .keybinding(Some(("enter".to_string(), vec![ModifierKey::Command]))),
                        PaletteItem::new("assistant: quote selection")
                            .keybinding(Some((">".to_string(), vec![ModifierKey::Command]))),
                        PaletteItem::new("assistant: toggle focus")
                            .keybinding(Some(("?".to_string(), vec![ModifierKey::Command]))),
                        PaletteItem::new("auto update: check"),
                        PaletteItem::new("auto update: view release notes"),
                        PaletteItem::new("branches: open recent").keybinding(Some((
                            "b".to_string(),
                            vec![ModifierKey::Command, ModifierKey::Alt],
                        ))),
                        PaletteItem::new("chat panel: toggle focus"),
                        PaletteItem::new("cli: install"),
                        PaletteItem::new("client: sign in"),
                        PaletteItem::new("client: sign out"),
                    ]),
            )
    }
}
