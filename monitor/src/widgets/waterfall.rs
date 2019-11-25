use super::viridis::VIRIDIS;

use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Widget},
};

pub struct Waterfall<'a> {
    block: Option<Block<'a>>,
    
}

impl<'a> Widget for Waterfall<'a> {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        
    }
}
