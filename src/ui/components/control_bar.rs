use iced::alignment::Vertical;
use iced::border::Radius;
use iced::widget::{Column, button, column, row, slider, space, svg};
use iced::{Background, Border, Color, Length, Padding, Theme};

use std::sync::LazyLock;

use crate::ui::components::load_icon;
use crate::{MediaControl, Message};

fn control_button(theme: &Theme, status: button::Status) -> button::Style {
    button::Style {
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Radius::from(7),
        },

        background: Some(Background::Color({
            match status {
                button::Status::Hovered => Color::from_rgb8(0x3D, 0x3D, 0x3D),
                button::Status::Pressed => Color::from_rgb8(0x42, 0x42, 0x42),
                _ => Color::from_rgb8(0x36, 0x3B, 0x3C),
            }
        })),

        ..button::subtle(theme, status)
    }
}

load_icon!(step_backward);
load_icon!(step_forward);
load_icon!(pause);
load_icon!(play);

pub fn control_bar<'a>(playing: bool, duration: f64, position: f64) -> Column<'a, Message> {
    column![
        slider(
            0f64..=(duration as f64),
            position,
            Message::SetPosition
        )
        .width(Length::Fill),

        row![
            space().width(Length::FillPortion(1)),
            button(svg(step_backward.clone()))
                .style(control_button)
                .padding(Padding::from([15, 0]))
                .height(50.0)
                .width(50.0),
            button({
                if playing {
                    svg(pause.clone())
                } else {
                    svg(play.clone())
                }
            })
            .on_press(Message::Control(MediaControl::PlayPause))
            .style(control_button)
            .padding(Padding::from([12, 0]))
            .height(50.0)
            .width(50.0),
            button(svg(step_forward.clone()))
            .on_press(Message::Control(MediaControl::Next))
            .style(control_button)
            .padding(Padding::from([15, 0]))
            .height(50.0)
            .width(50.0),
            space().width(Length::FillPortion(1))
        ]
        .spacing(20.0)
        .height(60.0)
        .padding(Padding::ZERO.bottom(5.0))
        .align_y(Vertical::Center)
    ]
}
