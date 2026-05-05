use iced::widget::{Column, Scrollable, button, container, row, scrollable, svg};
use iced::{Alignment, Length};

use std::sync::LazyLock;

use crate::ui::components::load_inline;
use crate::ui::ellipsize::ellipsized_text;
use crate::{Message, Page};

const PAGES: [Page; 3] = [Page::Songs, Page::Queue, Page::Albums];
const ICONS: [LazyLock<svg::Handle>; 3] = [
    load_inline!(music),
    load_inline!(queue),
    load_inline!(album),
];

pub fn sidebar<'a>(current_page: &'a Page) -> Scrollable<'a, Message> {
    scrollable(Column::from_iter(
        PAGES
            .into_iter()
            .zip(ICONS.into_iter())
            .map(|(page, page_icon)| {
                button(
                    container(
                        row![
                            svg(page_icon.clone()).width(Length::Shrink),
                            ellipsized_text(format!("{:?}", &page))
                        ]
                        .spacing(10),
                    )
                    .align_x(Alignment::Start)
                    .padding(0),
                )
                .on_press(Message::SetPage(page.clone()))
                .style(move |t, s| {
                    button::background(t, s).with_background({
                        let palette = t.extended_palette();

                        if let Page::Album(_) = current_page
                            && &page == &Page::Albums
                        {
                            palette.primary.base.color
                        } else if &page == current_page {
                            palette.primary.base.color
                        } else {
                            palette.background.neutral.color
                        }
                    })
                })
                .width(Length::Fill)
                .height(Length::Fixed(30f32))
                .into()
            }),
    ))
}
