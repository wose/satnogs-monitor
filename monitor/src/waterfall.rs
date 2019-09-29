use std::cmp::max;

use unicode_width::UnicodeWidthStr;

use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::Style;
use tui::symbols;
use tui::widgets::{Block, Borders, Widget};

/// An X or Y axis for the chart widget
pub struct Axis<'a, L>
where
    L: AsRef<str> + 'a,
{
    /// Title displayed next to axis end
    title: Option<&'a str>,
    /// Style of the title
    title_style: Style,
    /// Bounds for the axis (all data points outside these limits will not be represented)
    bounds: [f64; 2],
    /// A list of labels to put to the left or below the axis
    labels: Option<&'a [L]>,
    /// The labels' style
    labels_style: Style,
    /// The style used to draw the axis itself
    style: Style,
}

impl<'a, L> Default for Axis<'a, L>
where
    L: AsRef<str>,
{
    fn default() -> Axis<'a, L> {
        Axis {
            title: None,
            title_style: Default::default(),
            bounds: [0.0, 0.0],
            labels: None,
            labels_style: Default::default(),
            style: Default::default(),
        }
    }
}

impl<'a, L> Axis<'a, L>
where
    L: AsRef<str>,
{
    pub fn title(mut self, title: &'a str) -> Axis<'a, L> {
        self.title = Some(title);
        self
    }

    pub fn title_style(mut self, style: Style) -> Axis<'a, L> {
        self.title_style = style;
        self
    }

    pub fn bounds(mut self, bounds: [f64; 2]) -> Axis<'a, L> {
        self.bounds = bounds;
        self
    }

    pub fn labels(mut self, labels: &'a [L]) -> Axis<'a, L> {
        self.labels = Some(labels);
        self
    }

    pub fn labels_style(mut self, style: Style) -> Axis<'a, L> {
        self.labels_style = style;
        self
    }

    pub fn style(mut self, style: Style) -> Axis<'a, L> {
        self.style = style;
        self
    }
}

/// A group of data points
pub struct Dataset<'a> {
    /// Name of the dataset (used in the legend if shown)
    name: &'a str,
    /// A reference to the actual data
    data: &'a [(f64, f64)],
    /// Style used to plot this dataset
    style: Style,
}

impl<'a> Default for Dataset<'a> {
    fn default() -> Dataset<'a> {
        Dataset {
            name: "",
            data: &[],
            style: Style::default(),
        }
    }
}

impl<'a> Dataset<'a> {
    pub fn name(mut self, name: &'a str) -> Dataset<'a> {
        self.name = name;
        self
    }

    pub fn data(mut self, data: &'a [(f64, f64)]) -> Dataset<'a> {
        self.data = data;
        self
    }

    pub fn style(mut self, style: Style) -> Dataset<'a> {
        self.style = style;
        self
    }
}

/// A container that holds all the infos about where to display each elements of the chart (axis,
/// labels, legend, ...).
#[derive(Debug)]
struct WaterfallLayout {
    title_x: Option<(u16, u16)>,
    title_y: Option<(u16, u16)>,
    label_x: Option<u16>,
    label_y: Option<u16>,
    axis_x: Option<u16>,
    axis_y: Option<u16>,
    color_box: Option<Rect>,
    plot_area: Rect,
}

impl Default for WaterfallLayout {
    fn default() -> WaterfallLayout {
        WaterfallLayout {
            title_x: None,
            title_y: None,
            label_x: None,
            label_y: None,
            axis_x: None,
            axis_y: None,
            color_box: None,
            plot_area: Rect::default(),
        }
    }
}

/// A widget to plot one or more dataset in a cartesian coordinate system
///
/// # Examples
///
/// ```
/// # extern crate tui;
/// # use tui::widgets::{Block, Borders, Chart, Axis, Dataset, Marker};
/// # use tui::style::{Style, Color};
/// # fn main() {
/// Chart::default()
///     .block(Block::default().title("Chart"))
///     .x_axis(Axis::default()
///         .title("X Axis")
///         .title_style(Style::default().fg(Color::Red))
///         .style(Style::default().fg(Color::Gray))
///         .bounds([0.0, 10.0])
///         .labels(&["0.0", "5.0", "10.0"]))
///     .y_axis(Axis::default()
///         .title("Y Axis")
///         .title_style(Style::default().fg(Color::Red))
///         .style(Style::default().fg(Color::Gray))
///         .bounds([0.0, 10.0])
///         .labels(&["0.0", "5.0", "10.0"]))
///     .datasets(&[Dataset::default()
///                     .name("data1")
///                     .marker(Marker::Dot)
///                     .style(Style::default().fg(Color::Cyan))
///                     .data(&[(0.0, 5.0), (1.0, 6.0), (1.5, 6.434)]),
///                 Dataset::default()
///                     .name("data2")
///                     .marker(Marker::Braille)
///                     .style(Style::default().fg(Color::Magenta))
///                     .data(&[(4.0, 5.0), (5.0, 8.0), (7.66, 13.5)])]);
/// # }
/// ```
pub struct Waterfall<'a, LX, LY>
where
    LX: AsRef<str> + 'a,
    LY: AsRef<str> + 'a,
{
    /// A block to display around the widget eventually
    block: Option<Block<'a>>,
    /// The horizontal axis
    x_axis: Axis<'a, LX>,
    /// The vertical axis
    y_axis: Axis<'a, LY>,
    /// A reference to the datasets
    datasets: &'a [Dataset<'a>],
    /// The widget base style
    style: Style,
}

impl<'a, LX, LY> Default for Waterfall<'a, LX, LY>
where
    LX: AsRef<str>,
    LY: AsRef<str>,
{
    fn default() -> Waterfall<'a, LX, LY> {
        Waterfall {
            block: None,
            x_axis: Axis::default(),
            y_axis: Axis::default(),
            style: Default::default(),
            datasets: &[],
        }
    }
}

impl<'a, LX, LY> Waterfall<'a, LX, LY>
where
    LX: AsRef<str>,
    LY: AsRef<str>,
{
    pub fn block(mut self, block: Block<'a>) -> Waterfall<'a, LX, LY> {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: Style) -> Waterfall<'a, LX, LY> {
        self.style = style;
        self
    }

    pub fn x_axis(mut self, axis: Axis<'a, LX>) -> Waterfall<'a, LX, LY> {
        self.x_axis = axis;
        self
    }

    pub fn y_axis(mut self, axis: Axis<'a, LY>) -> Waterfall<'a, LX, LY> {
        self.y_axis = axis;
        self
    }

    pub fn datasets(mut self, datasets: &'a [Dataset<'a>]) -> Waterfall<'a, LX, LY> {
        self.datasets = datasets;
        self
    }

    /// Compute the internal layout of the waterfall given the area. If the area is too small some
    /// elements may be automatically hidden
    fn layout(&self, area: Rect) -> WaterfallLayout {
        let mut layout = WaterfallLayout::default();
        if area.height == 0 || area.width == 0 {
            return layout;
        }
        let mut x = area.left();
        let mut y = area.bottom() - 1;

        if self.x_axis.labels.is_some() && y > area.top() {
            layout.label_x = Some(y);
            y -= 1;
        }

        if let Some(y_labels) = self.y_axis.labels {
            let mut max_width = y_labels
                .iter()
                .fold(0, |acc, l| max(UnicodeWidthStr::width(l.as_ref()), acc))
                as u16;
            if let Some(x_labels) = self.x_axis.labels {
                if !x_labels.is_empty() {
                    max_width = max(
                        max_width,
                        UnicodeWidthStr::width(x_labels[0].as_ref()) as u16,
                    );
                }
            }
            if x + max_width < area.right() {
                layout.label_y = Some(x);
                x += max_width;
            }
        }

        if self.x_axis.labels.is_some() && y > area.top() {
            layout.axis_x = Some(y);
            y -= 1;
        }

        if self.y_axis.labels.is_some() && x + 1 < area.right() {
            layout.axis_y = Some(x);
            x += 1;
        }

        if x < area.right() && y > 1 {
            layout.plot_area = Rect::new(x, area.top(), area.right() - x, y - area.top() + 1);
        }

        if let Some(title) = self.x_axis.title {
            let w = UnicodeWidthStr::width(title) as u16;
            if w < layout.plot_area.width && layout.plot_area.height > 2 {
                layout.title_x = Some((x + layout.plot_area.width - w, y));
            }
        }

        if let Some(title) = self.y_axis.title {
            let w = UnicodeWidthStr::width(title) as u16;
            if w + 1 < layout.plot_area.width && layout.plot_area.height > 2 {
                layout.title_y = Some((x + 1, area.top()));
            }
        }

        if let Some(inner_width) = self
            .datasets
            .iter()
            .map(|d| UnicodeWidthStr::width(d.name) as u16)
            .max()
        {
            let legend_width = inner_width + 2;
            let legend_height = self.datasets.len() as u16 + 2;
            if legend_width < layout.plot_area.width / 3
                && legend_height < layout.plot_area.height / 3
                && inner_width > 0
            {
                layout.color_box = Some(Rect::new(
                    layout.plot_area.right() - legend_width,
                    layout.plot_area.top(),
                    legend_width,
                    legend_height,
                ));
            }
        }
        layout
    }
}

impl<'a, LX, LY> Widget for Waterfall<'a, LX, LY>
where
    LX: AsRef<str>,
    LY: AsRef<str>,
{
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let chart_area = match self.block {
            Some(ref mut b) => {
                b.draw(area, buf);
                b.inner(area)
            }
            None => area,
        };

        let layout = self.layout(chart_area);
        let plot_area = layout.plot_area;
        if plot_area.width < 1 || plot_area.height < 1 {
            return;
        }

        self.background(&chart_area, buf, self.style.bg);

        if let Some((x, y)) = layout.title_x {
            let title = self.x_axis.title.unwrap();
            buf.set_string(x, y, title, self.x_axis.style);
        }

        if let Some((x, y)) = layout.title_y {
            let title = self.y_axis.title.unwrap();
            buf.set_string(x, y, title, self.y_axis.style);
        }

        if let Some(y) = layout.label_x {
            let labels = self.x_axis.labels.unwrap();
            let total_width = labels
                .iter()
                .fold(0, |acc, l| UnicodeWidthStr::width(l.as_ref()) + acc)
                as u16;
            let labels_len = labels.len() as u16;
            if total_width < plot_area.width && labels_len > 1 {
                for (i, label) in labels.iter().enumerate() {
                    buf.set_string(
                        plot_area.left() + i as u16 * (plot_area.width - 1) / (labels_len - 1)
                            - UnicodeWidthStr::width(label.as_ref()) as u16,
                        y,
                        label.as_ref(),
                        self.x_axis.labels_style,
                    );
                }
            }
        }

        if let Some(x) = layout.label_y {
            let labels = self.y_axis.labels.unwrap();
            let labels_len = labels.len() as u16;
            for (i, label) in labels.iter().enumerate() {
                let dy = i as u16 * (plot_area.height - 1) / (labels_len - 1);
                if dy < plot_area.bottom() {
                    buf.set_string(
                        x,
                        plot_area.bottom() - 1 - dy,
                        label.as_ref(),
                        self.y_axis.labels_style,
                    );
                }
            }
        }

        if let Some(y) = layout.axis_x {
            for x in plot_area.left()..plot_area.right() {
                buf.get_mut(x, y)
                    .set_symbol(symbols::line::HORIZONTAL)
                    .set_style(self.x_axis.style);
            }
        }

        if let Some(x) = layout.axis_y {
            for y in plot_area.top()..plot_area.bottom() {
                buf.get_mut(x, y)
                    .set_symbol(symbols::line::VERTICAL)
                    .set_style(self.y_axis.style);
            }
        }

        if let Some(y) = layout.axis_x {
            if let Some(x) = layout.axis_y {
                buf.get_mut(x, y)
                    .set_symbol(symbols::line::BOTTOM_LEFT)
                    .set_style(self.x_axis.style);
            }
        }

        for dataset in self.datasets {
            for (x, y) in dataset.data {
                let dy = ((self.y_axis.bounds[1] - y) * f64::from(plot_area.height - 1)
                    / (self.y_axis.bounds[1] - self.y_axis.bounds[0]))
                    as u16;
                let dx = ((x - self.x_axis.bounds[0]) * f64::from(plot_area.width - 1)
                    / (self.x_axis.bounds[1] - self.x_axis.bounds[0]))
                    as u16;

                buf.get_mut(plot_area.left() + dx, plot_area.top() + dy)
                    .set_symbol(symbols::DOT)
                    .set_fg(dataset.style.fg)
                    .set_bg(dataset.style.bg);
            }
        }

        if let Some(legend_area) = layout.color_box {
            Block::default()
                .borders(Borders::ALL)
                .draw(legend_area, buf);
            for (i, dataset) in self.datasets.iter().enumerate() {
                buf.set_string(
                    legend_area.x + 1,
                    legend_area.y + 1 + i as u16,
                    dataset.name,
                    dataset.style,
                );
            }
        }
    }
}
