use std::sync::LazyLock;

use iced::alignment::Vertical;
use iced::widget::{Row, column, row, space, svg};


use crate::ui::components::album_display::get_cover_art;
use crate::ui::components::load_icon;
use crate::ui::ellipsize::ellipsized_text;
use crate::{Album, Message};

load_icon!(placeholder_album);

pub fn album_heading(album: &Album) -> Row<'_, Message> {
    row![
        get_cover_art(album, 200f32).unwrap_or(
            svg(placeholder_album.clone())
                .height(200f32)
                .width(100f32)
                .into()
        ),
        column![
            ellipsized_text(album.name.clone()).size(24),
            space().height(3),
            ellipsized_text(
                album
                    .artists
                    .iter()
                    .map(|x| x.name.clone())
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        ]
    ]
    .align_y(Vertical::Center)
    .spacing(10)
}
