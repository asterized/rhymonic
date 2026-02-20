use iced::widget::{
    Column, Row, column, row, scrollable, text as _text,
};
use iced::{Element, Length};

use crate::ellipsize::ellipsized_text;
use crate::{App, MediaEvent, MediaSignal, Message};

pub const SIZES: [u16; 5] = [2, 8, 5, 5, 3];
pub const COLUMNS: [&'static str; 5] = ["Track", "Title", "Album", "Artists", "Duration"];

impl App {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Queue(x) => {
                self.queue.push(x.clone());
                self.channel
                    .try_send(MediaSignal::AddSong(x))
                    .expect("could not send message");
            }
            Message::Send(x) => self.channel.try_send(x).expect("could not send message"),
            Message::Play(x) => {
                self.queue.clear();
                let _ = self.channel.try_send(MediaSignal::PlaySong(x.clone()));
                self.queue.push(x);
            }

            Message::Media(x) => match x {
                MediaEvent::Connect(channel) => {
                    self.channel = channel;
                    self.connected = true;
                }
                MediaEvent::EndedSong => {
                    self.queue_position += 1;
                    let _ = self.channel.try_send(MediaSignal::PlaySong(
                        self.queue[self.queue_position].clone(),
                    ));
                }
                _ => {}
            },
        };
    }

    pub fn view(&self) -> Element<'_, Message> {
        if !self.connected {
            return _text("Loading...").into();
        }
        let tbl = Column::from_iter(self.map_songs());
        let songs: Element<'_, Message> = scrollable(tbl).into();

        column![row![
            self.display_queue(),
            column![
                Row::from_iter(COLUMNS.into_iter().enumerate().map(|(i, x)| {
                    ellipsized_text(x.to_string())
                        .size(15.0)
                        .width(Length::FillPortion(SIZES[i]))
                        .into()
                }))
                .padding(5),
                songs
            ]
            .width(Length::FillPortion(8))
        ]]
        .into()
    }
}
