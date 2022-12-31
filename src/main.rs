use iced::widget::{button, column, container, horizontal_space, image, row, text};
use iced::{color, Color, Length};
use iced::{theme, Theme};
use iced::{Alignment, Settings};
use iced::{Element, Sandbox};

mod mouse_area;

use mouse_area::{MouseArea, MouseState};

const CAT_OPEN: &[u8] = include_bytes!("./assets/open.jpeg");
const CAT_CLOSED: &[u8] = include_bytes!("./assets/closed.jpeg");

fn mouse_area<'a, Message, T: Into<Element<'a, Message>>>(
    view: impl Fn(MouseState) -> T + 'a,
) -> MouseArea<'a, Message, iced::Renderer> {
    MouseArea::new(view)
}

fn main() -> iced::Result {
    MyApp::run(Settings::default())
}

enum SpoilersStyle {
    Hidden,
    Shown,
}

impl Default for SpoilersStyle {
    fn default() -> Self {
        SpoilersStyle::Hidden
    }
}

impl container::StyleSheet for SpoilersStyle {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        match self {
            SpoilersStyle::Hidden => container::Appearance {
                background: Some(iced::Background::Color(Color::BLACK)),
                ..Default::default()
            },
            SpoilersStyle::Shown => container::Appearance {
                ..Default::default()
            },
        }
    }
}

fn todo_item<'a>(item_text: &'static str) -> Element<'a, (), iced::Renderer> {
    mouse_area(move |mouse_state| {
        let item_row = row![item_text, horizontal_space(Length::Fill)]
            .align_items(Alignment::Center)
            .height(40.into());

        container(if mouse_state.hovered {
            item_row.push(button("Done").on_press(()))
        } else {
            item_row
        })
        .padding(10)
        .width(Length::Fill)
        .style(theme::Container::Box)
    })
    .into()
}

struct MyApp;

impl Sandbox for MyApp {
    type Message = ();

    fn new() -> MyApp {
        MyApp
    }

    fn title(&self) -> String {
        String::from("Iced MouseArea prototype")
    }

    fn update(&mut self, _message: Self::Message) {
        // This application has no interactions
    }

    fn view(&self) -> Element<Self::Message> {
        container(
            column![
                text("Iced MouseArea Prototype")
                    .size(40)
                    .style(color!(0x888888)),
                mouse_area(|mouse_state| button(if mouse_state.hovered {
                    "hovered"
                } else {
                    "not hovered"
                })
                .on_press(())),
                row![
                    "Spoilers: ",
                    mouse_area(|mouse_state| {
                        container("Pineapple pizza is pretty good").style(theme::Container::Custom(
                            Box::new(if mouse_state.hovered {
                                SpoilersStyle::Shown
                            } else {
                                SpoilersStyle::Hidden
                            }),
                        ))
                    })
                ],
                container(
                    column![
                        text("Todo List").size(30).style(color!(0x777777)),
                        column![
                            todo_item("Do programming"),
                            todo_item("Do more programming")
                        ]
                        .spacing(10),
                    ]
                    .align_items(Alignment::Center)
                    .spacing(20)
                )
                .width(300.into())
                .center_x(),
                mouse_area(|mouse_state| {
                    image(image::Handle::from_memory(if mouse_state.hovered {
                        CAT_OPEN
                    } else {
                        CAT_CLOSED
                    }))
                }),
            ]
            .width(400.into())
            //.align_items(Alignment::Center)
            .spacing(30),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .padding([50, 100])
        .into()
    }
}
