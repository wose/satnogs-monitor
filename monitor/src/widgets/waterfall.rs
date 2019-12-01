use super::viridis::VIRIDIS;

use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Widget},
};

pub struct Waterfall<'a> {
    data: &'a [(f32, Vec<f32>)],
    frequencies: &'a [f32],
    block: Option<Block<'a>>,
}

impl<'a> Waterfall<'a> {
    pub fn new(frequencies: &'a [f32], data: &'a [(f32, Vec<f32>)]) -> Self {
        Waterfall {
            data,
            frequencies,
            block: None,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a> Widget for Waterfall<'a> {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let area = match self.block {
            Some(ref mut b) => {
                b.draw(area, buf);
                b.inner(area)
            }
            None => area,
        };

        if area.height < 5 {
            return;
        }

        let bin_size = self.frequencies.len() / (area.width as usize);

        const PIX: &str = "▀";
        // draw the legend
        // text left of legend
        // legend 2 chars width dB top or bottom of legend

        let lines = area.height as usize * 2;
        let columns = area.width;

        let rows = self.data.iter().rev().take(lines);

        let datapoints = self.frequencies.len();
        for (row, chunk) in rows
            .rev()
            .collect::<Vec<&(f32, Vec<f32>)>>()
            .chunks(2)
            .enumerate()
        {
            if let Some((timestamp, row_data)) = chunk.iter().next() {
                let columns = row_data
                    .chunks(bin_size)
                    .map(|chunk| chunk.iter().fold(-100f32, |res, val| res.max(*val)))
                    .collect::<Vec<f32>>();

                let styles = if let Some((timestamp, row_data)) = chunk.iter().next() {
                    columns
                        .iter()
                        .zip(
                            row_data
                                .chunks(bin_size)
                                .map(|chunk| chunk.iter().fold(-100f32, |res, val| res.max(*val)))
                                .collect::<Vec<f32>>(),
                        )
                        .map(|(first, second)|
                             Style::default()
                             .fg(VIRIDIS[255 - ((255.0 / 100.0 * first).abs().floor() as usize).min(255)])
                             .bg(VIRIDIS[255 - ((255.0 / 100.0 * second).abs().floor() as usize).min(255)])
                        )
                        .collect::<Vec<_>>()
                } else {
                    columns
                        .iter()
                        .map(|db| {
                            Style::default()
                                .fg(VIRIDIS[255 - ((255.0 / 100.0 * db).abs().floor() as usize).min(255)])
                        })
                        .collect::<Vec<_>>()
                };

                // we do not interpolate between pixels so we just zoom slightly in and display
                // the area.width pixel in the center of the waterfall
                let start = (styles.len() - area.width as usize) / 2;
                for (column, style) in styles.iter().skip(start).take(area.width  as usize).enumerate() {
                    buf.set_string(
                        area.left() + column as u16,
                        area.top() + row as u16,
                        PIX,
                        *style,
                    );
                }
            }
            //let datapoints = bins.len();
            //let columns = bins.chunks(5).map(|chunk| chunk.iter().sum::<f32>() / datapoints as f32).collect::<Vec<f32>>();
        }

        for data_points in self.data.iter().rev().take(lines) {}
    }
}
