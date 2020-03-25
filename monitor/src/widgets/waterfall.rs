use super::viridis::VIRIDIS;

use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, Widget},
};

#[derive(Default)]
pub struct WaterfallLayout {
    legend_area: Option<Rect>,
    data_area: Rect,
}

pub struct WaterfallLegend<'a, L>
where
    L: AsRef<str> + 'a,
{
    labels: Option<&'a [L]>,
    labels_style: Style,
}

impl<'a, L> Default for WaterfallLegend<'a, L>
where
    L: AsRef<str>,
{
    fn default() -> Self {
        WaterfallLegend {
            labels: None,
            labels_style: Default::default(),
        }
    }
}

impl<'a, L> WaterfallLegend<'a, L>
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

pub struct Waterfall<'a, L>
where
    L: AsRef<str> + 'a,
{
    block: Option<Block<'a>>,
    bounds: [f32; 2],
    data: &'a [(i64, Vec<f32>)],
    legend: Option<WaterfallLegend<'a, L>>,
}

impl<'a, L> Default for Waterfall<'a, L>
where
    L: AsRef<str> + 'a,
{
    fn default() -> Self {
        Waterfall {
            block: None,
            bounds: [-100.0, 0.0],
            data: Default::default(),
            legend: None,
        }
    }
}

impl<'a, L> Waterfall<'a, L>
where
    L: AsRef<str>,
{
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn bounds(mut self, bounds: [f32; 2]) -> Self {
        self.bounds = bounds;
        self
    }

    pub fn data(mut self, data: &'a [(i64, Vec<f32>)]) -> Self {
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
        let y = area.bottom() - 1;

        if self.legend.is_some() && y > area.top() {
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
            let color_step = 255.0 / (area.height as f32 * 2.0);
            for line in 0..area.height {
                let top_index =
                    ((255.0 - (color_step * line as f32 * 2.0).abs().floor()) as usize).min(255);
                let bottom_index = ((255.0
                    - (color_step * line as f32 * 2.0 + color_step).abs().floor())
                    as usize)
                    .min(255);

                buf.set_string(
                    area.left() + 4,
                    area.top() + line,
                    "▀▀",
                    Style::default()
                        .fg(VIRIDIS[top_index])
                        .bg(VIRIDIS[bottom_index]),
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

        if self.data.is_empty() {
            return;
        }

        let area = layout.data_area;
        let bin_size = self.data[0].1.len() / (area.width as usize);

        const PIX: &str = "▀";

        let lines = area.height as usize * 2;
        let rows = self.data.iter().rev().take(lines);

        let db_range = self.bounds[1] - self.bounds[0];

        for (row, chunk) in rows
            .collect::<Vec<&(i64, Vec<f32>)>>()
            .chunks(2)
            .enumerate()
        {
            let mut chunk = chunk.iter();
            if let Some((_timestamp, row_data)) = chunk.next() {
                let columns = row_data
                    .chunks(bin_size)
                    .map(|chunk| chunk.iter().fold(self.bounds[0], |res, val| res.max(*val)))
                    .collect::<Vec<f32>>();

                let styles = if let Some((_timestamp, row_data)) = chunk.next() {
                    columns
                        .iter()
                        .zip(
                            row_data
                                .chunks(bin_size)
                                .map(|chunk| {
                                    chunk.iter().fold(self.bounds[0], |res, val| res.max(*val))
                                })
                                .collect::<Vec<f32>>(),
                        )
                        .map(|(first, second)| {
                            Style::default()
                                .fg(VIRIDIS[255
                                    - ((255.0 / db_range * (first - self.bounds[1])).abs().floor()
                                        as usize)
                                        .min(255)])
                                .bg(
                                    VIRIDIS[255
                                        - ((255.0 / db_range * (second - self.bounds[1]))
                                            .abs()
                                            .floor()
                                            as usize)
                                            .min(255)],
                                )
                        })
                        .collect::<Vec<_>>()
                } else {
                    columns
                        .iter()
                        .map(|db| {
                            Style::default().fg(VIRIDIS[255
                                - ((255.0 / db_range * (db - self.bounds[1])).abs().floor()
                                    as usize)
                                    .min(255)])
                        })
                        .collect::<Vec<_>>()
                };

                // we do not interpolate between pixels so we just zoom slightly in and display
                // the area.width pixel in the center of the waterfall
                let start = (styles.len() - area.width as usize) / 2;
                for (column, style) in styles
                    .iter()
                    .skip(start)
                    .take(area.width as usize)
                    .enumerate()
                {
                    buf.set_string(
                        area.left() + column as u16,
                        area.top() + row as u16,
                        PIX,
                        *style,
                    );
                }
            }
        }
    }
}
