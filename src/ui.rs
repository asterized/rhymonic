use iced::widget::{
    Column, Row, button, column, container, mouse_area, row, scrollable, text,
};
use iced::{color, Background, Length};
use iced::{Element, Theme};

use crate::{App, MediaEvent, MediaSignal, Message, Song};

fn song_row(song: &Song) -> Row<'_, Message> {
    Row::from_iter(
        [
            text(song.track_number.to_string()),
            text(song.title.clone()),
            text(String::from("album")),
            text(
                song.artists
                    .iter()
                    .map(|x| x.name.clone())
                    .collect::<Vec<String>>()
                    .join(", "),
            ),
            text(
                song.genres
                    .iter()
                    .map(|x| x.name.clone())
                    .collect::<Vec<String>>()
                    .join(","),
            ),
            text(song.length.as_secs()),
        ]
        .into_iter()
        .zip(SIZES)
        .map(|(x, w)| x.width(Length::FillPortion(w)).wrapping(text::Wrapping::None).into()),
    )
}

fn song_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let base = button::text(theme, status);

    match status {
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(palette.background.neutral.color)),
            ..base
        },
        _ => base,
    }
}

const SIZES: [u16; 6] = [1, 5, 5, 5, 5, 3];
const COLUMNS: [&'static str; 6] = ["Track", "Title", "Artist", "Album", "Genre", "Duration"];

impl App {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Blueify(x) => self.blue = x,
            Message::Play(x) => {
                self.channel.as_mut().unwrap()
                    .try_send(MediaSignal::PlaySong(x))
                    .expect("could not send message");
            },
            Message::Media(x) => {
                match x {
                    MediaEvent::Connect(x) => {
                        println!("start");
                        self.channel = Some(x);
                    }
                    _ => {}
                }
            }
            _ => todo!()
        };
    }

    fn map_songs(&self) -> Vec<Element<'_, Message>> {
        self.songs
            .iter()
            .enumerate()
            .map(|(i, song)| {
                container(mouse_area(song_row(song))
                    .on_press(Message::Blueify(i))
                    .on_double_click(Message::Play(song.clone())))
                    .style(move |theme| {
                        if self.blue == i {
                            container::Style {
                                background: Some(Background::Color(color!(0x1b48a2))),
                                ..container::bordered_box(theme)
                            }
                        } else {
                            container::bordered_box(theme)
                        }
                    })
                    .into()
            })
            .collect()
    }

    pub fn view<'a>(&self) -> Element<'_, Message> {
        let tbl = self.map_songs();
        let songs: Element<'_, Message> = scrollable(Column::from_iter(tbl)).into();

        let topbar = container(
            row![
                button(text("haii")).style(|theme: &Theme, status: button::Status| {
                    let palette = theme.extended_palette();

                    button::Style {
                        background: Some(palette.background.stronger.color.into()),
                        ..button::background(theme, status)
                    }
                })
            ]
            .padding(8),
        )
        .width(Length::Fill)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();

            container::Style {
                background: Some(palette.background.weakest.color.into()),
                border: iced::border::rounded(32),
                ..container::Style::default()
            }
        });

        column![
            topbar,
            column![
                Row::from_iter(
                    COLUMNS
                        .into_iter()
                        .enumerate()
                        .map(|(i, x)| text(x).width(Length::FillPortion(SIZES[i])).into())
                ),
                songs
            ]
        ]
        .into()
    }   
}
