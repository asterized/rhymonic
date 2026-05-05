use iced::widget::{Column, Row, button, column, container, responsive, rule, scrollable, space};
use iced::{Element, Length, Theme};
use iced_aw::ContextMenu;

use crate::ui::ellipsize::ellipsized_text;
use crate::{Message, Song};

use std::sync::Arc;
use std::time::Duration;

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

pub const SIZES: [u16; 5] = [2, 8, 5, 5, 3];

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

const COLUMNS: [&'static str; 5] = ["Track", "Title", "Album", "Artists", "Duration"];
const ROW_HEIGHT: f32 = 50.0;

pub fn display_song(song: &Arc<Song>) -> Element<'_, Message> {
    ContextMenu::new(
        container(
            button(song_row(song))
                .on_press(Message::Play(song.clone()))
                .padding(0)
                .style(|theme, status| {
                    let mut base = button::background(theme, status);
                    base.border = iced::border::width(0);

                    base
                })
        )
        .height(ROW_HEIGHT),

        || column![
            button("Add to queue").on_press(Message::Queue(song.clone()))
        ].into()
    )
        .into()
}

pub fn display_songs(
    songs: &Vec<Arc<Song>>,
    scroll_offset: f32,
    _connected: bool,
) -> Element<'_, Message> {
    column![
        container(
            Row::from_iter(
                std::iter::once(space().width(3.5).into())
                    .chain(COLUMNS.iter().zip(SIZES.iter()).flat_map(|(column, size)| {
                        [
                            container(ellipsized_text(*column).width(Length::FillPortion(*size)))
                                .into(),
                            rule::vertical(1f32).into(),
                        ]
                    }))
                    .take(COLUMNS.len() * 2)
            )
            .spacing(7)
        )
        .height(Length::Shrink)
        .style(|theme: &Theme| container::background(
            theme.extended_palette().background.weaker.color
        )),
        ContextMenu::new(
            responsive(move |viewport| {
                let start = (scroll_offset / ROW_HEIGHT) as usize;
                let visible_rows = (viewport.height / ROW_HEIGHT) as usize + 1;
                let end = (start + visible_rows).min(songs.len());

                let bottom_space: f32 = songs.len().saturating_sub(end) as f32 * ROW_HEIGHT;

                scrollable(
                    column![
                        space().height(start as f32 * ROW_HEIGHT),
                        Column::from_iter(songs[start..end].iter().map(display_song)),
                        space().height(bottom_space),
                    ]
                    .height(Length::Fill),
                )
                .height(Length::Fill)
                .on_scroll(|position| Message::ScrollPosition(position.absolute_offset().y))
                .into()
            })
            .height(Length::Fill),
            || column![
                button("Import song").on_press(Message::BeginImportSong),
                button("Import folder").on_press(Message::BeginImportDir)
            ]
            .into()
        )
    ]
    .into()
}
