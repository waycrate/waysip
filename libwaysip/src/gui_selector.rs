use std::sync::{Arc, mpsc};
use std::time::Duration;

use clap::ValueEnum;
use iced::Theme;
use iced_layershell::application;
use iced_layershell::reexport::{Anchor, KeyboardInteractivity};
use iced_layershell::settings::{LayerShellSettings, Settings};
use libwayshot::WayshotConnection;
use libwayshot::output::OutputInfo;
use libwayshot::region::TopLevel;

use crate::iced_selector::IcedSelector;

// ValueEnum allows client code to use clap for the theme selection
#[derive(ValueEnum, Clone, Default)]
pub enum WaySipTheme {
    Light,
    #[default]
    Dark,
    Dracula,
    Nord,
    SolarizedLight,
    SolarizedDark,
    GruvboxLight,
    GruvboxDark,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
    TokyoNight,
    TokyoNightStorm,
    TokyoNightLight,
    KanagawaWave,
    KanagawaDragon,
    KanagawaLotus,
    Moonfly,
    Oxocarbon,
    Ferra,
}

impl From<WaySipTheme> for Theme {
    fn from(theme: WaySipTheme) -> Theme {
        match theme {
            WaySipTheme::Light => Theme::Light,
            WaySipTheme::Dark => Theme::Dark,
            WaySipTheme::Dracula => Theme::Dracula,
            WaySipTheme::Nord => Theme::Nord,
            WaySipTheme::SolarizedLight => Theme::SolarizedLight,
            WaySipTheme::SolarizedDark => Theme::SolarizedDark,
            WaySipTheme::GruvboxLight => Theme::GruvboxLight,
            WaySipTheme::GruvboxDark => Theme::GruvboxDark,
            WaySipTheme::CatppuccinLatte => Theme::CatppuccinLatte,
            WaySipTheme::CatppuccinFrappe => Theme::CatppuccinFrappe,
            WaySipTheme::CatppuccinMacchiato => Theme::CatppuccinMacchiato,
            WaySipTheme::CatppuccinMocha => Theme::CatppuccinMocha,
            WaySipTheme::TokyoNight => Theme::TokyoNight,
            WaySipTheme::TokyoNightStorm => Theme::TokyoNightStorm,
            WaySipTheme::TokyoNightLight => Theme::TokyoNightLight,
            WaySipTheme::KanagawaWave => Theme::KanagawaWave,
            WaySipTheme::KanagawaDragon => Theme::KanagawaDragon,
            WaySipTheme::KanagawaLotus => Theme::KanagawaLotus,
            WaySipTheme::Moonfly => Theme::Moonfly,
            WaySipTheme::Oxocarbon => Theme::Oxocarbon,
            WaySipTheme::Ferra => Theme::Ferra,
        }
    }
}

/// Interface struct to start a GUI area selector and retrieve its result
#[derive(Default)]
pub struct AreaSelectorGUI {
    conn: Option<WayshotConnection>,
    theme: WaySipTheme,
}

/// Represents the user's selection made through interaction with the GUI area selector
pub enum GUISelection {
    Toplevel(TopLevel),
    Output(OutputInfo),
    Failed,
}

impl AreaSelectorGUI {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_connection(mut self, conn: WayshotConnection) -> Self {
        self.conn = Some(conn);
        self
    }

    pub fn with_theme(mut self, theme: WaySipTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Launches a GUI area selector
    pub fn launch(self) -> GUISelection {
        let (tx, rx) = mpsc::channel::<GUISelection>();
        let conn = Arc::new(match self.conn {
            Some(conn) => conn,
            None => WayshotConnection::new().expect("Couldn't establish a Wayshot connection"),
        });

        let _ = application(
            move || IcedSelector::new(tx.clone(), conn.clone()),
            IcedSelector::namespace,
            IcedSelector::update,
            IcedSelector::view,
        )
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
        .theme(Into::<Theme>::into(self.theme))
        .run();

        // Gets the selection from the GUI
        rx.recv_timeout(Duration::from_secs(1))
            .unwrap_or(GUISelection::Failed)
    }
}
