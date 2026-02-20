use std::sync::mpsc::Sender;

use iced::widget::{Button, Column, button, column, image as iced_image, row, scrollable, text};
use iced::{Alignment, Element, Length, Task};
use iced_image::Handle;
use iced_layershell::to_layer_message;
use iced_runtime::task;
use libwayshot::WayshotConnection;
use libwayshot::output::OutputInfo;
use libwayshot::region::TopLevel;
use std::sync::Arc;

use crate::fl;
use crate::gui_selector::GUISelection;

#[derive(Debug, Clone, Copy, PartialEq)]
enum ViewMode {
    Screens,
    Windows,
}

pub(crate) struct IcedSelector {
    mode: ViewMode,
    toplevels: Vec<(TopLevel, Option<Handle>)>,
    outputs: Vec<(OutputInfo, Option<Handle>)>,
    sender: Sender<GUISelection>,
}

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
pub(crate) enum Message {
    ShowScreens,
    ShowWindows,
    ScreenSelected(usize),
    WindowSelected(usize),
    OutputScreenshot(usize, Handle),
    ToplevelScreenshot(usize, Handle),
}

impl IcedSelector {
    pub(crate) fn new(
        sender: Sender<GUISelection>,
        conn: Arc<WayshotConnection>,
    ) -> (Self, Task<Message>) {
        let toplevels_info = conn.get_all_toplevels().to_vec();
        let outputs_info = conn.get_all_outputs().to_vec();

        // Initialize IcedSelector instance with outputs and toplevels obtained
        // through Wayshot, alongside their screenshot (obtained asynchronously)
        let toplevels_tasks = toplevels_info.iter().enumerate().map(|(i, t)| {
            let toplevel = t.clone();
            let conn = conn.clone();
            task::blocking(move |sender| {
                // can fail if toplevel capture is not supported
                parse_screenshot_and_send(conn.screenshot_toplevel(&toplevel, false), sender, i);
            })
            .map(|(i, s)| Message::ToplevelScreenshot(i, s))
        });

        let outputs_tasks = outputs_info.iter().enumerate().map(|(i, o)| {
            let output = o.clone();
            let conn = conn.clone();
            task::blocking(move |sender| {
                parse_screenshot_and_send(conn.screenshot_single_output(&output, false), sender, i);
            })
            .map(|(i, s)| Message::OutputScreenshot(i, s))
        });

        let all_tasks = Task::batch(toplevels_tasks.chain(outputs_tasks));

        (
            Self {
                mode: ViewMode::Screens,
                toplevels: toplevels_info.into_iter().map(|t| (t, None)).collect(),
                outputs: outputs_info.into_iter().map(|o| (o, None)).collect(),
                sender,
            },
            all_tasks,
        )
    }

    pub(crate) fn namespace() -> String {
        String::from("selection") // same as slurp
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
            Message::ScreenSelected(index) => {
                let _ = self
                    .sender
                    .send(GUISelection::Output(self.outputs[index].0.clone()));
                iced::exit()
            }
            Message::WindowSelected(index) => {
                let _ = self
                    .sender
                    .send(GUISelection::Toplevel(self.toplevels[index].0.clone()));
                iced::exit()
            }
            Message::OutputScreenshot(index, screenshot_handle) => {
                self.outputs[index].1 = Some(screenshot_handle);
                Task::none()
            }
            Message::ToplevelScreenshot(index, screenshot_handle) => {
                self.toplevels[index].1 = Some(screenshot_handle);
                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub(crate) fn view(&self) -> Element<'_, Message> {
        let selector = row![
            button(text(fl!("screens")).center().width(Length::Fill))
                .on_press(Message::ShowScreens)
                .width(Length::Fill)
                .style(if self.mode == ViewMode::Screens {
                    button::primary
                } else {
                    button::secondary
                }),
            button(text(fl!("windows")).center().width(Length::Fill))
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

        let choices: Element<'_, Message> = match self.mode {
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

        column![selector, scrollable(choices).height(Length::Fill)]
            .padding(20)
            .spacing(10)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn build_button<'a>(
    label: String,
    screenshot: Option<Handle>,
    message: Message,
) -> Button<'a, Message> {
    let button_content: Element<'a, Message> = match screenshot {
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

fn parse_screenshot_and_send(
    capture: Result<image::DynamicImage, libwayshot::Error>,
    mut sender: futures_channel::mpsc::Sender<(usize, Handle)>,
    elem_index: usize,
) {
    if let Ok(screenshot) = capture {
        let rgba_image = screenshot.to_rgba8();
        let handle = Handle::from_rgba(
            rgba_image.width(),
            rgba_image.height(),
            rgba_image.into_raw(),
        );
        let _ = sender.try_send((elem_index, handle));
    }
}
