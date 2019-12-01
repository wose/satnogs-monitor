use super::viridis::VIRIDIS;

use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Widget},
};

#[derive(Default)]
pub struct WaterfallLayout {
    legend_area: Option<Rect>,
    data_area: Rect,
}

#[derive(Default)]
pub struct WaterfallLegend<'a, L>
where
    L: AsRef<str> + 'a
{
    title: Option<&'a str>,
    title_style: Style,
    labels: Option<&'a [L]>,
    labels_style: Style,
}

impl <'a, L> WaterfallLegend<'a, L>
where
    L: AsRef<str>,
{
    pub fn labels(mut self, labels: &'a [L]) -> Self {
        self.labels = Some(labels);
        self
    }

    pub fn labels_style(mut self, style: Style) -> Self {
        self.labels_style = style;
        self
    }
}

#[derive(Default)]
pub struct Waterfall<'a, L>
where
    L: AsRef<str> + 'a
{
    data: &'a [(f32, Vec<f32>)],
    frequencies: &'a [f32],
    block: Option<Block<'a>>,
    legend: Option<WaterfallLegend<'a, L>>
}

impl<'a, L> Waterfall<'a, L>
where
    L: AsRef<str>,
{
    pub fn new(frequencies: &'a [f32], data: &'a [(f32, Vec<f32>)]) -> Self {
        Waterfall {
            legend: None,
            data,
            frequencies,
            block: None,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn data(mut self, data: &'a [(f32, Vec<f32>)]) -> Self {
        self.data = data;
        self
    }

    pub fn legend(mut self, legend: WaterfallLegend<'a, L>) -> Self {
        self.legend = Some(legend);
        self
    }

    fn layout(&self, area: Rect) -> WaterfallLayout {
        let mut layout = WaterfallLayout::default();
        if area.height == 0 || area.width == 0 {
            return layout;
        }

        let mut x = area.left();
        let mut y = area.bottom() - 1;

        if self.legend.is_some() && y > area.top() {
            // -100 ##
            // 7 chars
            layout.legend_area = Some(Rect::new(x, area.top(), x + 7, y - area.top() + 1));
            x += 7;
        }

        if x < area.right() && y > 1 {
            layout.data_area = Rect::new(x, area.top(), area.right() - x, y - area.top() + 1);
        }

        layout
    }
}

impl<'a, L> Widget for Waterfall<'a, L>
where
    L: AsRef<str>,
{
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let area = match self.block {
            Some(ref mut b) => {
                b.draw(area, buf);
                b.inner(area)
            }
            None => area,
        };

        if area.width < 10 || area.height < 5 {
            return;
        }

        let layout = self.layout(area);

        if let Some(area) = layout.legend_area {
            for line in 0..area.height {
                let top_index = ((255.0 - (255.0 / (area.height  as f32 * 2.0) * line as f32 * 2.0).abs().floor()) as usize).min(255);
                let bottom_index = ((255.0 - (255.0 / (area.height  as f32 * 2.0) * line  as f32 * 2.0 + 1.0).abs().floor()) as usize).min(255);

                buf.set_string(
                    area.left() + 4,
                    area.top() + line,
                    "▀▀",
                    Style::default()
                        .fg(VIRIDIS[top_index])
                        .bg(VIRIDIS[bottom_index])
                );
            }

            if let Some(legend) = &self.legend {
                if let Some(labels) = legend.labels {
                    let label_num = labels.len();
                    for (index, label) in labels.iter().enumerate() {
                        let dy = index as u16 * (area.height - 1) / (label_num as u16 - 1);

                        buf.set_string(
                            area.left(),
                            area.bottom() - 1 - dy,
                            format!("{:>4}", label.as_ref()),
                            legend.labels_style,
                        );
                    }
                }
            }
        }


        let area = layout.data_area;
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
