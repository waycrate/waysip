use std::sync::mpsc::{self, Sender};
use std::time::Duration;

use iced::widget::{Button, Column, button, column, image as iced_image, row, scrollable, text};
use iced::{Alignment, Element, Length, Task, Theme};
use iced_layershell::daemon;
use iced_layershell::reexport::{Anchor, KeyboardInteractivity};
use iced_layershell::settings::{LayerShellSettings, Settings};
use iced_layershell::to_layer_message;
use libwayshot::WayshotConnection;
use libwayshot::output::OutputInfo;
use libwayshot::region::TopLevel;

use crate::gui_selector::GUISelection;

pub fn launch() -> GUISelection {
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

    rx.recv_timeout(Duration::from_secs(10))
        .unwrap_or(GUISelection::Failed)
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ViewMode {
    Screens,
    Windows,
}

pub(crate) struct IcedSelector {
    mode: ViewMode,
    toplevels: Vec<(TopLevel, Option<iced_image::Handle>)>,
    outputs: Vec<(OutputInfo, Option<iced_image::Handle>)>,
    sender: Sender<GUISelection>,
}

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
pub(crate) enum Message {
    ShowScreens,
    ShowWindows,
    ScreenSelected(usize),
    WindowSelected(usize),
}

impl IcedSelector {
    pub(crate) fn new(sender: Sender<GUISelection>) -> Self {
        WayshotConnection::new()
            .map(|mut conn| {
                let toplevels_info = conn.get_all_toplevels().to_vec();
                let outputs_info = conn.get_all_outputs().to_vec();

                // initialize IcedSelector struct with outputs and toplevels obtained 
                // through Wayshot, alongside their screenshot
                let toplevels: Vec<(TopLevel, Option<iced_image::Handle>)> = toplevels_info
                    .into_iter()
                    .map(|t| {
                        (
                            t.clone(),
                            // can fail if toplevel capture is not supported
                            match conn.screenshot_toplevel(t, false) {
                                Ok(screenshot) => {
                                    let rgba_image = screenshot.to_rgba8();
                                    Some(iced_image::Handle::from_rgba(
                                        rgba_image.width(),
                                        rgba_image.height(),
                                        rgba_image.into_raw(),
                                    ))
                                },
                                _ => Option::None,
                            }
                        )
                    })
                    .collect();

                let outputs: Vec<(OutputInfo, Option<iced_image::Handle>)> = outputs_info
                    .into_iter()
                    .map(|o| {
                        (
                            o.clone(),
                            match conn.screenshot_single_output(&o, false) {
                                Ok(screenshot) => {
                                    let rgba_image = screenshot.to_rgba8();
                                    Some(iced_image::Handle::from_rgba(
                                        rgba_image.width(),
                                        rgba_image.height(),
                                        rgba_image.into_raw(),
                                    ))
                                },
                                _ => Option::None,
                            }
                        )
                    })
                    .collect();

                Self {
                    mode: ViewMode::Screens,
                    toplevels,
                    outputs,
                    sender,
                }
            })
            .expect("Couldn't establish a Wayshot connection")
    }

    pub(crate) fn title(&self, _id: iced::window::Id) -> Option<String> {
        Some(String::from("Waysip - Area Selector"))
    }

    pub(crate) fn namespace() -> String {
        String::from("waysip_area_selector")
    }

    pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ShowScreens => {
                self.mode = ViewMode::Screens;
                Task::none()
            }
            Message::ShowWindows => {
                self.mode = ViewMode::Windows;
                Task::none()
            }
            Message::ScreenSelected(id) => {
                let _ = self.sender.send(
                    self.outputs
                        .get(id)
                        .map_or(GUISelection::Failed, |o| GUISelection::Output(o.0.clone())),
                );
                iced::exit()
            }
            Message::WindowSelected(id) => {
                let _ =
                    self.sender
                        .send(self.toplevels.get(id).map_or(GUISelection::Failed, |t| {
                            GUISelection::Toplevel(t.0.clone())
                        }));
                iced::exit()
            }
            _ => Task::none(),
        }
    }

    pub(crate) fn view(&self, _id: iced::window::Id) -> Element<'_, Message> {
        let selector = row![
            button(text("Screens").center().width(Length::Fill))
                .on_press(Message::ShowScreens)
                .width(Length::Fill)
                .style(if self.mode == ViewMode::Screens {
                    button::primary
                } else {
                    button::secondary
                }),
            button(text("Windows").center().width(Length::Fill))
                .on_press(Message::ShowWindows)
                .width(Length::Fill)
                .style(if self.mode == ViewMode::Windows {
                    button::primary
                } else {
                    button::secondary
                }),
        ]
        .align_y(Alignment::Center)
        .spacing(10)
        .padding(20)
        .width(Length::Fill);

        let content: Element<'_, Message> = match self.mode {
            ViewMode::Screens => {
                Column::with_children(self.outputs.iter().enumerate().map(|(i, e)| {
                    build_button(e.0.name.clone(), e.1.clone(), Message::ScreenSelected(i)).into()
                }))
            }
            ViewMode::Windows => {
                Column::with_children(self.toplevels.iter().enumerate().map(|(i, e)| {
                    build_button(e.0.title.clone(), e.1.clone(), Message::WindowSelected(i)).into()
                }))
            }
        }
        .spacing(10)
        .into();

        column![selector, scrollable(content).height(Length::Fill)]
            .padding(20)
            .spacing(10)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

// Helper function to build buttons with iced
fn build_button<'a>(
    label: String,
    content: Option<iced_image::Handle>,
    message: Message,
) -> Button<'a, Message> {
    let button_content: Element<'a, Message> = match content {
        Some(image_handle) => column![
            text(label).center().width(Length::Fill),
            iced_image(image_handle)
                .width(Length::Fill)
                .height(Length::Fixed(100.0))
        ]
        .align_x(Alignment::Center)
        .spacing(5)
        .into(),
        _ => text(label).center().width(Length::Fill).into(),
    };
    button(button_content)
    .on_press(message)
    .width(Length::Fill)
    .style(button::subtle)
    .padding(10)
}
