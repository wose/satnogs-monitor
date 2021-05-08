use chrono::prelude::*;
use satnogs_network_client::StationStatus;
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Widget},
};
use unicode_width::UnicodeWidthStr;

use crate::state::State;

pub struct InfoBar<'a> {
    block: Option<Block<'a>>,
    state: &'a State,
    style: Style,
    online_style: Style,
    testing_style: Style,
    offline_style: Style,
    active_style: Style,
}

impl<'a> InfoBar<'a> {
    pub fn new(state: &'a State) -> Self {
        InfoBar {
            block: None,
            state,
            style: Default::default(),
            online_style: Style::default().fg(Color::LightGreen).bg(Color::DarkGray),
            testing_style: Style::default().fg(Color::Yellow).bg(Color::DarkGray),
            offline_style: Style::default().fg(Color::LightRed).bg(Color::DarkGray),
            active_style: Style::default().fg(Color::LightCyan),
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl<'a> Widget for InfoBar<'a> {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        if area.height < 1 {
            return;
        }

        let space_between_tabs = 3;

        let mut x = area.left();
        for station in self.state.stations.values() {
            let style = match station.info.status {
                StationStatus::Online => self.online_style,
                StationStatus::Testing => self.testing_style,
                StationStatus::Offline => self.offline_style,
            };
            buf.set_string(x, area.top(), " ▲ ", style);

            if area.height > 1 {
                buf.set_string(x, area.top() + 1, "▀▀▀", Style::default().fg(style.fg));
            }

            let title = format!(" {} - {} ", station.info.id, station.info.name);
            let title_width = UnicodeWidthStr::width(title.as_str()) as u16;
            buf.set_string(
                x + 3,
                area.top(),
                title,
                Style::default().fg(Color::White).bg(Color::DarkGray),
            );
            if area.height > 1 {
                let decal_style = match self.state.active_station {
                    id if id == station.info.id => self.active_style,
                    _ => Style::default().fg(Color::DarkGray),
                };

                let decal = (0..title_width).map(|_| "▀").collect::<String>();
                buf.set_string(x + 3, area.top() + 1, decal, decal_style);
            }

            x += 3 + title_width + space_between_tabs;
        }

        let utc: DateTime<Utc> = Utc::now();
        let utc = utc.format(" %F %Z %T").to_string();
        let clock_width = utc.chars().count() as u16;

        if area.right() >= clock_width {
            buf.set_string(
                area.right() - clock_width,
                area.top(),
                utc,
                Style::default().fg(Color::White).bg(Color::DarkGray),
            );
            if area.height > 1 {
                let decal = (0..clock_width).map(|_| "▀").collect::<String>();
                buf.set_string(
                    area.right() - clock_width,
                    area.top() + 1,
                    decal,
                    Style::default().fg(Color::DarkGray),
                );
            }
        }
    }
}
