use iced::widget::{
    Column, Row, button, column, row, scrollable, slider, svg, text as _text
};
use iced::{Element, Length, Padding};

use crate::ellipsize::ellipsized_text;
use crate::ui::helpers::{map_songs, display_queue};
use crate::{App, MediaEvent, MediaSignal, Message};

pub const SIZES: [u16; 5] = [2, 8, 5, 5, 3];
pub const COLUMNS: [&'static str; 5] = ["Track", "Title", "Album", "Artists", "Duration"];

impl App {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::SetPosition(position) => {
                let _ = self.channel.try_send(MediaSignal::NewPosition(position));
            }

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

                MediaEvent::Sync(duration) => {
                    self.position = duration.as_secs_f64();
                }

                _ => {}
            },

            Message::SetPage(page) => {
                self.page = page;
            },

            _ => {}
        };
    }

    pub fn view(&self) -> Element<'_, Message> {
        if !self.connected {
            return _text("Loading...").into();
        }

        let tbl = map_songs(&self.songs);
        let songs = scrollable(Column::from_vec(tbl).width(Length::Fill)).height(Length::Fill);

        let duration = if self.queue.len() <= self.queue_position {
            0
        }
        else {
            self.queue[self.queue_position].length.as_secs()
        };

        row![
            display_queue(&self.queue),
            column![
                Row::from_iter(COLUMNS.into_iter().enumerate().map(|(i, x)| {
                    ellipsized_text(x.to_string())
                        .size(15.0)
                        .width(Length::FillPortion(SIZES[i]))
                        .into()
                }))
                    .padding(5),

                songs,

                slider(
                    0f64..=(duration as f64),
                    self.position,
                    Message::SetPosition
                ),

                row![
                    row![].width(Length::FillPortion(3)),

                    row![
                        button(svg("resources/step-backward.svg"))
                            .style(button::background)
                            .padding(Padding::from([15, 0])),
                        button(svg("resources/play.svg"))
                            .on_press(Message::Send(MediaSignal::PlayPause))
                            .style(button::background)
                            .padding(Padding::from([12, 0])),
                        button(svg("resources/step-forward.svg"))
                            .on_press(Message::Send(MediaSignal::Next))
                            .style(button::background)
                            .padding(Padding::from([15, 0]))
                    ]
                        .height(52)
                        .width(Length::FillPortion(1)),

                    row![].width(Length::FillPortion(3))
                ]
            ]
            .width(Length::FillPortion(8))
        ]
        .into()
    }
}
