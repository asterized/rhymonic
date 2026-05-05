use std::sync::atomic::Ordering;

use iced::widget::{column, container, row};
use iced::{Border, Element, Length};

use crate::ui::Page;
use crate::ui::components::{album_display, control_bar, display_songs, sidebar};
use crate::{App, Message};

impl App {
    pub fn view(&self) -> Element<'_, Message> {
        let data = match self.page {
            Page::Songs => display_songs(&self.songs, self.scroll_position, self.connected),
            Page::Queue => display_songs(&self.queue, self.scroll_position, self.connected),
            Page::Album(index) => display_songs(
                &self.albums[index].songs,
                self.scroll_position,
                self.connected,
            ),
            Page::Albums => album_display(self.albums.iter()),
        };

        let duration = if self.queue.len() <= self.queue_position {
            0
        } else {
            self.queue[self.queue_position].length.as_millis() as u64
        };

        column![
            row![
                sidebar(&self.page)
                    .width(Length::FillPortion(1))
                    .height(Length::Fill),

                container(data)
                    .style(|theme| container::Style {
                        border: Border {
                            color: theme.extended_palette().background.neutral.color,
                            width: 1.0,
                            radius: 1.0.into(),
                        },
                        ..container::Style::default()
                    })
                    .width(Length::FillPortion(6))
                    .height(Length::Fill)
            ],

            control_bar(
                self.playing,
                duration as f64,
                self.position.load(Ordering::Relaxed) as f64
            )
        ]
        .into()
    }
}
