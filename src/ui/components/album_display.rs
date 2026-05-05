use std::sync::LazyLock;

use iced::widget::{Row, button, column, image as wimage, scrollable, svg};

use iced::{Element, Length};

use crate::ui::Page;
use crate::ui::components::load_icon;
use crate::ui::ellipsize::ellipsized_text;
use crate::{Album, Message};

fn get_cover_art(album: &Album) -> Option<Element<'_, Message>> {
    let f = album.songs.first()?;
    let art_bytes = f.image.clone()?;

    Some(wimage(art_bytes).height(100f32).into())
}

load_icon!(placeholder_album);

pub fn album_display<'a>(albums: impl Iterator<Item = &'a Album>) -> Element<'a, Message> {
    scrollable(
        Row::from_iter(
            albums
                .enumerate()
                .map(|(index, album): (usize, &'a Album)| {
                    button(column![
                        get_cover_art(album).unwrap_or(
                            svg(placeholder_album.clone())
                                .height(100f32)
                                .width(100f32)
                                .into()
                        ),
                        ellipsized_text(&album.name)
                        .width(100.0)
                        .height(20.0)
                        .size(16),
                        ellipsized_text(
                            album
                                .artists
                                .iter()
                                .map(|artist| artist.name.clone())
                                .collect::<Vec<String>>()
                                .join(", ")
                        )
                        .width(100.0)
                        .height(27.0)
                        .size(14)
                    ])
                    .on_press(Message::SetPage(Page::Album(index)))
                    .style(|theme, status| {
                        let mut s = button::subtle(theme, status);

                        s.border = iced::Border {
                            color: iced::Color::TRANSPARENT,
                            width: 2.0,
                            radius: iced::border::radius(20)
                        };

                        s
                    })
                    .into()
                }),
        )
        .spacing(20)
        .width(Length::Fill)
        .wrap(),
    )
    .height(Length::Fill)
    .into()
}
