use iced::widget::{Column, Row, button, column, container};
use iced::{Element, Length};
use iced_aw::ContextMenu;
use std::time::Duration;

use super::interface::SIZES;
use crate::{App, Message, Song, ellipsize::ellipsized_text};

fn format_duration(duration: &Duration) -> String {
    if duration.as_secs() >= 3600 {
        format!(
            "{}:{:02}:{:02}",
            duration.as_secs() / 3600,
            duration.as_secs() % 3600 / 60,
            duration.as_secs() % 60
        )
    } else {
        format!("{}:{:02}", duration.as_secs() / 60, duration.as_secs() % 60)
    }
}

pub fn song_row(song: &Song) -> Row<'_, Message> {
    Row::from_iter(
        [
            song.track_number.to_string(),
            song.title.clone(),
            song.album.clone(),
            song.artists
                .iter()
                .map(|x| x.name.clone())
                .collect::<Vec<String>>()
                .join(", "),
            format_duration(&song.length),
        ]
        .into_iter()
        .zip(SIZES)
        .map(|(x, w)| {
            column![
                ellipsized_text(x)
                    .size(15.0)
                    .height(17.0)
                    .width(Length::FillPortion(w))
            ]
            .padding(5)
            .into()
        }),
    )
    .height(50)
}

impl App {
    pub fn display_queue(&self) -> Element<'_, Message> {
        if self.queue.is_empty() {
            return iced::widget::space().into();
        }

        Column::from_iter(
            self.queue.iter().map(
                |song| button(
                    column![
                        ellipsized_text(&song.title),
                        ellipsized_text(song.artists.iter().map(|x| x.name.clone()).collect::<Vec<_>>().join(", "))
                    ]
                ).width(Length::Fill).into()
            )
        ).width(Length::FillPortion(2)).into()
    }

    pub fn map_songs(&self) -> Vec<Element<'_, Message>> {
        self.albums
            .iter()
            .flat_map(|album| album.iter())
            .map(|song| {
                container(ContextMenu::new(
                    button(song_row(song))
                        .on_press(Message::Play(song.clone()))
                        .style(|theme: &iced::Theme, status: button::Status| {
                            let _palette = theme.extended_palette();
                            button::Style {
                                ..button::subtle(theme, status)
                            }
                        })
                        .padding(0),
                    || column![].into(),
                ))
                .into()
            })
            .collect()
    }
}
