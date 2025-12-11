use std::sync::mpsc::{self};
use std::time::Duration;

use iced::Theme;
use iced_layershell::daemon;
use iced_layershell::reexport::{Anchor, KeyboardInteractivity};
use iced_layershell::settings::{LayerShellSettings, Settings};
use libwayshot::output::OutputInfo;
use libwayshot::region::TopLevel;

use crate::iced_selector::IcedSelector;

pub struct AreaSelectorGUI;

pub enum GUISelection {
    Toplevel(TopLevel),
    Output(OutputInfo),
    Failed,
}

impl AreaSelectorGUI {
    pub fn new() -> Self {
        Self {}
    }

    pub fn launch(&self) -> GUISelection {
        let (tx, rx) = mpsc::channel::<GUISelection>();
        let _ = daemon(
            move || IcedSelector::new(tx.clone()),
            IcedSelector::namespace,
            IcedSelector::update,
            IcedSelector::view,
        )
        .title(IcedSelector::title)
        .settings(Settings {
            layer_settings: LayerShellSettings {
                size: Some((400, 400)),
                exclusive_zone: 0,
                anchor: Anchor::Bottom | Anchor::Left | Anchor::Right | Anchor::Top,
                keyboard_interactivity: KeyboardInteractivity::None,
                ..Default::default()
            },
            ..Default::default()
        })
        .theme(Theme::Dark)
        .run();

        rx.recv_timeout(Duration::from_secs(1))
            .unwrap_or(GUISelection::Failed)
    }
}
