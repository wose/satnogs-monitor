use chrono::prelude::*;
use circular_queue::CircularQueue;
use failure::ResultExt;
use log::{debug, trace};
use satnogs_network_client::{Client, StationStatus};
use termion::input::{MouseTerminal, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use tui::{backend::*, text::Span, text::Spans};
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::symbols::{DOT, Marker};
use tui::widgets::canvas::{Canvas, Context, Line, Map, MapResolution, Points};
use tui::widgets::{Block, Borders, Paragraph};
use tui::Frame;
use tui::Terminal;

use tui::widgets::{Axis, Chart, Dataset};

use std::f64::consts;
use std::io;
use std::sync::mpsc::{sync_channel, Receiver, RecvTimeoutError, SyncSender};
use std::thread;

use crate::event::Event;
use crate::job::Job;
use crate::satnogs;
use crate::settings::Settings;
use crate::state::State;
use crate::station::Station;
use crate::widgets::{InfoBar, Waterfall, WaterfallLegend};

use crate::Result;

const COL_LIGHT_BG: Color = Color::DarkGray;
const COL_DARK_CYAN: Color = Color::DarkGray;
const COL_WHITE: Color = Color::White;

type LogQueue = CircularQueue<(DateTime<Utc>, log::Level, String)>;
type TermBackend = TermionBackend<MouseTerminal<RawTerminal<io::Stdout>>>;

pub struct Ui {
    events: Receiver<Event>,
    logs: LogQueue,
    last_job_update: std::time::Instant,
    network: satnogs::Connection,
    sender: SyncSender<Event>,
    settings: Settings,
    show_logs: bool,
    shutdown: bool,
    size: Rect,
    state: State,
    terminal: Terminal<TermBackend>,
    ticks: u32,
    waterfall_data: Vec<(i64, Vec<f32>)>,
    waterfall_frequencies: Vec<f32>,
    waterfall_obs_id: u64,
}

impl Ui {
    pub fn new(settings: Settings, _client: Client, state: State) -> Result<Self> {
        let (sender, reciever) = sync_channel(100);

        // Must be called before any threads are launched
        let winch_send = sender.clone();
        let mut signals = ::signal_hook::iterator::Signals::new(&[::libc::SIGWINCH])
            .context("couldn't register resize signal handler")?;
        thread::spawn(move || {
            for _ in signals.pending() {
                let _ = winch_send.send(Event::Resize);
            }
        });

        let input_send = sender.clone();
        thread::spawn(move || {
            for event in ::std::io::stdin().events() {
                if let Ok(ev) = event {
                    let _ = input_send.send(Event::Input(ev));
                }
            }
        });

        let tick_send = sender.clone();
        thread::spawn(move || {
            while tick_send.send(Event::Tick).is_ok() {
                thread::sleep(std::time::Duration::new(1, 0));
            }
        });

        let stdout = io::stdout()
            .into_raw_mode()
            .context("failed to put stdout into raw mode")?;
        let stdout = MouseTerminal::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend).context("failed to create terminal")?;

        terminal.clear().context("failed to clear terminal")?;
        terminal.hide_cursor().context("failed to hide cursor")?;

        let ui = Self {
            events: reciever,
            last_job_update: std::time::Instant::now(),
            logs: CircularQueue::with_capacity(100),
            network: satnogs::Connection::new(sender.clone(), settings.api_endpoint.clone()),
            sender,
            settings,
            show_logs: false,
            shutdown: false,
            size: Rect::default(),
            state,
            terminal,
            ticks: 0,
            waterfall_obs_id: 0,
            waterfall_frequencies: vec![],
            waterfall_data: vec![],
        };

        Ok(ui)
    }

    pub fn sender(&self) -> SyncSender<Event> {
        self.sender.clone()
    }

    fn next_station(&mut self) {
        self.state.next_station();
    }

    fn prev_station(&mut self) {
        self.state.prev_station();
    }

    fn draw(&mut self) -> Result<()> {
        let size = self
            .terminal
            .size()
            .context("Failed to get terminal size")?;
        if self.size != size {
            self.terminal
                .resize(size)
                .context("Failed to resize terminal")?;
            self.size = size;
        }

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([Constraint::Length(2), Constraint::Min(0)].as_ref())
            .split(self.size);

        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(40), Constraint::Min(0)].as_ref())
            .split(rows[1]);

        let log_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(10)].as_ref())
            .split(self.size)[1];

        let station = self.state.get_active_station();

        let logs = &self.logs;
        let show_logs = self.show_logs;
        let ground_tracks = self.settings.ui.ground_track_num as usize;
        let sat_footprint = self.settings.ui.sat_footprint;
        let spectrum_plot = self.settings.ui.spectrum_plot;
        let rot_thresholds = (
            self.settings.ui.rotator_warn,
            self.settings.ui.rotator_error,
        );
        let db_range = [self.settings.ui.db_min, self.settings.ui.db_max];
        let state = &self.state;
        let waterfall = self.settings.ui.waterfall;
        let waterfall_data = &self.waterfall_data;
        let waterfall_frequencies = &self.waterfall_frequencies;
        let waterfall_zoom = self.settings.waterfall_zoom;

        self.terminal
            .draw(|mut f| {
                let info_bar = InfoBar::new(state)
                    .style(Style::default().fg(Color::White).bg(Color::DarkGray))
                    .block(Block::default());
                f.render_widget(info_bar, rows[0]);

                let mut rect = render_station_view(&mut f, body[0], &station);
                rect = render_next_job_view(&mut f, rect, &station);
                if let Some(job) = station.jobs.iter().next() {
                    rect = render_polar_plot(&mut f, rect, &job);
                }
                rect = render_satellite_view(&mut f, rect, state, rot_thresholds);
                rect = render_future_jobs_view(&mut f, rect, &station);

                // to create the rest of the border we add an empty paragraph
                let empty: Vec<Spans> = Vec::new();
                let fill = Paragraph::new(empty)
                    .block(
                        Block::default()
                            .borders(Borders::RIGHT)
                            .border_style(Style::default().fg(COL_DARK_CYAN)),
                    );
                f.render_widget(fill, rect);

                // render main area on the right
                rect = body[1];
                if !waterfall_data.is_empty() && !waterfall_frequencies.is_empty() {
                    let layout = Layout::default().direction(Direction::Vertical);

                    rect = match (spectrum_plot, waterfall) {
                        (true, false) => {
                            let area = layout
                                .constraints(
                                    [Constraint::Percentage(50), Constraint::Min(0)].as_ref(),
                                )
                                .split(rect);

                            render_spectrum_plot(
                                &mut f,
                                area[1],
                                &waterfall_frequencies,
                                &waterfall_data,
                                db_range,
                                waterfall_zoom,
                            );

                            area[0]
                        }
                        (false, true) => {
                            let area = layout
                                .constraints(
                                    [Constraint::Percentage(50), Constraint::Min(0)].as_ref(),
                                )
                                .split(rect);

                            render_waterfall(
                                &mut f,
                                area[1],
                                &waterfall_frequencies,
                                &waterfall_data,
                                db_range,
                            );

                            area[0]
                        }

                        (true, true) => {
                            let area = layout
                                .constraints(
                                    [
                                        Constraint::Percentage(50),
                                        Constraint::Percentage(25),
                                        Constraint::Min(0),
                                    ]
                                    .as_ref(),
                                )
                                .split(rect);

                            render_spectrum_plot(
                                &mut f,
                                area[1],
                                &waterfall_frequencies,
                                &waterfall_data,
                                db_range,
                                waterfall_zoom,
                            );
                            render_waterfall(
                                &mut f,
                                area[2],
                                &waterfall_frequencies,
                                &waterfall_data,
                                db_range,
                            );

                            area[0]
                        }
                        _ => rect,
                    };
                }

                render_map_view(&mut f, rect, &station, ground_tracks, sat_footprint);

                if show_logs {
                    render_log_view(&mut f, log_area, logs);
                }
            })
            .context("Failed to draw to terminal")?;

        Ok(())
    }

    fn handle_input(&mut self, event: &::termion::event::Event) {
        use termion::event::Event::*;
        use termion::event::Key::*;

        match *event {
            Key(Ctrl('c')) => self.shutdown = true,
            Key(Char('f')) => self.settings.ui.sat_footprint = !self.settings.ui.sat_footprint,
            Key(Char('l')) => self.show_logs = !self.show_logs,
            Key(Char('\t')) => self.next_station(),
            Key(Ctrl('\t')) => self.prev_station(),
            Key(Char('q')) => self.shutdown = true,
            Key(Char('+')) => {
                if self.settings.waterfall_zoom < 10.0 {
                    self.settings.waterfall_zoom += 0.5;
                }
            }
            Key(Char('-')) => {
                if self.settings.waterfall_zoom > 1.0 {
                    self.settings.waterfall_zoom -= 0.5;
                }
            }
            Key(key) => {
                debug!("Key Event: {:?}", key);
            }
            _ => {}
        }
    }

    fn handle_event(&mut self, event: Event) {
        match event {
            Event::CommandResponse(data) => match data {
                satnogs::Data::Jobs(station_id, jobs) => {
                    self.state.update_jobs(station_id, jobs);
                    self.state
                        .update_vessel_position(self.settings.ui.ground_track_num);
                }
            },
            Event::Resize => debug!("Terminal size changed"),
            Event::Input(event) => {
                self.handle_input(&event);
            }
            Event::Log((level, message)) => {
                self.logs.push((Utc::now(), level, message));
            }
            Event::RotatorPosition(azimuth, elevation) => {
                self.state.rotator_position = Some((azimuth, elevation));
            }
            Event::SystemInfo(local_stations, sys_info) => {
                trace!("Got system info for stations {:?}", local_stations);
                for id in local_stations {
                    self.state
                        .stations
                        .entry(id)
                        .and_modify(|station| station.update_sys_info(sys_info.clone()));
                }
            }
            Event::Tick => {
                self.handle_tick();
            }
            Event::WaterfallCreated(obs_id, frequencies) => {
                self.waterfall_obs_id = obs_id;
                self.waterfall_frequencies = frequencies;
            }
            Event::WaterfallData(seconds, data) => {
                self.waterfall_data.push((seconds, data));
            }
            Event::WaterfallClosed(_obs_id) => {
                self.waterfall_data.clear();
                self.waterfall_frequencies.clear();
                self.waterfall_obs_id = 0;
            }
        }
    }

    fn handle_tick(&mut self) {
        if self.last_job_update.elapsed().as_secs() >= self.settings.job_update_interval {
            self.update_jobs();
        }

        self.ticks += 1;
        if self.ticks % 5 == 0 {
            self.state
                .update_vessel_position(self.settings.ui.ground_track_num);
        }

        if self.ticks % 60 == 0 {
            for job in self.state.stations.values_mut() {
                job.remove_finished_jobs();
            }
        }
    }

    fn update_jobs(&mut self) {
        trace!("Requesting jobs update");

        for id in self.state.stations.keys() {
            self.network.send(satnogs::Command::GetJobs(*id)).unwrap();
        }
        self.last_job_update = std::time::Instant::now();
    }

    pub fn run(mut self) -> Result<()> {
        use std::time::{Duration, Instant};

        self.update_jobs();
        self.draw()?;

        while let Ok(event) = self.events.recv() {
            self.handle_event(event);

            let start_instant = Instant::now();
            while let Some(remaining_time) =
                Duration::from_millis(16).checked_sub(start_instant.elapsed())
            {
                let event = match self.events.recv_timeout(remaining_time) {
                    Ok(ev) => ev,
                    Err(RecvTimeoutError::Timeout) => break,
                    Err(_) => {
                        self.shutdown = true;
                        break;
                    }
                };

                self.handle_event(event);
            }

            self.draw()?;

            if self.shutdown {
                break;
            }
        }

        Ok(())
    }
}

fn render_waterfall<T: Backend>(
    t: &mut Frame<T>,
    rect: Rect,
    _frequencies: &[f32],
    data: &[(i64, Vec<f32>)],
    db_range: [f32; 2],
) {
    let min = format!("{:>4.0}", db_range[0]);
    let mid = format!("{:>4.0}", (db_range[0] + db_range[1]) / 2.0);
    let max = format!("{:>4.0}", db_range[1]);
    let labels = [&min, &mid, &max];

    let legend = WaterfallLegend::default()
        .labels(&labels)
        .labels_style(Style::default().fg(Color::DarkGray));

    let waterfall = Waterfall::default()
        .data(data)
        .bounds(db_range)
        .block(
            Block::default()
                .title(Span::styled("Waterfall", Style::default().fg(Color::Yellow)))
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .legend(legend);

    t.render_widget(waterfall, rect);
}

fn render_spectrum_plot<T: Backend>(
    t: &mut Frame<T>,
    rect: Rect,
    frequencies: &[f32],
    data: &[(i64, Vec<f32>)],
    db_range: [f32; 2],
    zoom: f32,
) {
    let title = format!("Spectrum (x{:.*})", 1, zoom);

    let freq_1 = Span::styled(format!("{}", (frequencies.first().unwrap() / 1000.0 / zoom).floor()), Style::default().fg(Color::DarkGray));
    let freq_2 = Span::styled(format!("{}", (frequencies.first().unwrap() / 1000.0 / 2.0 / zoom).floor()), Style::default().fg(Color::DarkGray));
    let freq_3 = Span::styled(format!("{}", 0), Style::default().fg(Color::DarkGray));
    let freq_4 = Span::styled(format!("{}", (frequencies.last().unwrap() / 1000.0 / 2.0 / zoom).ceil()), Style::default().fg(Color::DarkGray));
    let freq_5 = Span::styled(format!("{}", (frequencies.last().unwrap() / 1000.0 / zoom).ceil()), Style::default().fg(Color::DarkGray));
    let x_labels = vec![freq_1, freq_2, freq_3, freq_4, freq_5];

    let db_min = Span::styled(format!("{:>6.0}", db_range[0]), Style::default().fg(Color::DarkGray));
    let db_mid = Span::styled(format!("{:>6.0}", (db_range[0] + db_range[1]) / 2.0), Style::default().fg(Color::DarkGray));
    let db_max = Span::styled(format!("{:>6.0}", db_range[1]), Style::default().fg(Color::DarkGray));
    let y_labels = vec![db_min, db_mid, db_max];

    let data = frequencies
        .iter()
        .zip(&data.last().unwrap().1)
        .map(|(x, y)| (*x as f64, *y as f64))
        .collect::<Vec<_>>();

    let datasets = vec![
        Dataset::default()
            .marker(Marker::Braille)
            .style(Style::default().fg(Color::Cyan))
            .data(&data)
    ];

    let spectrum = Chart::new(datasets)
        .block(
            Block::default()
                .title(Span::styled(title ,Style::default().fg(Color::Yellow)))
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .x_axis(
            Axis::default()
                .title(Span::styled("Frequency (kHz)", Style::default().fg(Color::DarkGray)))
                .style(Style::default().fg(Color::DarkGray))
                .bounds([
                    (*frequencies.first().unwrap() / zoom) as f64,
                    (*frequencies.last().unwrap() / zoom) as f64,
                ])
                .labels(x_labels)
        )
        .y_axis(
            Axis::default()
                .title(Span::styled("Power (dB)", Style::default().fg(Color::DarkGray)))
                .style(Style::default().fg(Color::DarkGray))
                .bounds([db_range[0] as f64, db_range[1] as f64])
                .labels(y_labels)
        );

    t.render_widget(spectrum, rect);
}

fn render_map_view<T: Backend>(
    t: &mut Frame<T>,
    rect: Rect,
    station: &Station,
    ground_tracks: usize,
    footprint: bool,
) {
    let map = Canvas::default()
        .paint(|ctx| {
            ctx.draw(&Map {
                color: COL_LIGHT_BG,
                resolution: MapResolution::High,
            });

            ctx.print(station.info.lng, station.info.lat, DOT, Color::LightCyan);

            if let Some(job) = station.jobs.iter().next() {
//                let vessel_name = format!("■─{}", job.vessel_name());
                ctx.print(
                    job.sat().lon_deg,
                    job.sat().lat_deg,
                    r#"■"#,
                    Color::LightRed,
                );
                ctx.layer();
                let mut ground_track = Points::default();
                // plot future orbits first so the current orbit will be drawn on top
                ground_track.color = Color::Cyan;
                ground_track.coords =
                    &job.vessel.ground_track[job.vessel.ground_track.len() / ground_tracks..];
                ctx.draw(&ground_track);

                ctx.layer();
                ground_track.color = Color::Yellow;
                ground_track.coords =
                    &job.vessel.ground_track[..job.vessel.ground_track.len() / ground_tracks];
                ctx.draw(&ground_track);

                if footprint {
                    ctx.layer();
                    let footprint = Points {
                        coords: &job.vessel.footprint,
                        color: Color::Green,
                    };
                    ctx.draw(&footprint);
                }
            }
        })
        .x_bounds([-180.0, 180.0])
        .y_bounds([-90.0, 90.0]);

    t.render_widget(map, rect);
}

fn render_polar_plot<T: Backend>(t: &mut Frame<T>, rect: Rect, job: &Job) -> Rect {
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(rect.width / 2), Constraint::Min(0)].as_ref())
        .split(rect);

    let polar_plot = Canvas::default()
        .paint(|ctx| {
            ctx.draw(&Line {
                x1: -100.0,
                y1: 0.0,
                x2: 100.0,
                y2: 0.0,
                color: COL_LIGHT_BG,
            });
            ctx.draw(&Line {
                x1: 0.0,
                y1: -100.0,
                x2: 0.0,
                y2: 100.0,
                color: COL_LIGHT_BG,
            });
            draw_arc(ctx, COL_LIGHT_BG, (0.0, 0.0), 100.0, 0.0, 360.0, 360);
            draw_arc(ctx, COL_LIGHT_BG, (0.0, 0.0), 100.0 / 3.0, 0.0, 360.0, 360);
            draw_arc(ctx, COL_LIGHT_BG, (0.0, 0.0), 200.0 / 3.0, 0.0, 360.0, 360);

            ctx.layer();
            ctx.print(-110.0, 0.0, "W", Color::Yellow);
            ctx.print(110.0, 0.0, "E", Color::Yellow);
            ctx.print(0.0, 120.0, "N", Color::Yellow);
            ctx.print(0.0, -110.0, "S", Color::Yellow);

            ctx.layer();
            let polar_track = &job
                .vessel
                .polar_track
                .iter()
                .map(|point| azel2xy(point))
                .collect::<Vec<(f64, f64)>>();

            let aos_point = polar_track.first();
            let los_point = polar_track.last();

            let points = Points {
                coords: polar_track,
                color: Color::Cyan,
            };
            ctx.draw(&points);

            if let Some(aos_point) = aos_point {
                ctx.print(aos_point.0, aos_point.1, DOT, Color::Green);
            }

            if let Some(los_point) = los_point {
                ctx.print(los_point.0, los_point.1, DOT, Color::Red);
            }

            let now = Utc::now();
            if now >= job.start() && now <= job.end() {
                let position = azel2xy(&(job.sat().az_deg, job.sat().el_deg));
                ctx.print(position.0, position.1, "■", Color::LightRed);
            }
        })
        .x_bounds([-120.0, 120.0])
        .y_bounds([-120.0, 120.0])
        .block(
            Block::default()
                .borders(Borders::RIGHT)
                .border_style(Style::default().fg(COL_DARK_CYAN)),
        );

    t.render_widget(polar_plot, area[0]);

    area[1]
}

fn azel2xy(point: &(f64, f64)) -> (f64, f64) {
    let az = point.0.to_radians();
    let el = point.1.to_radians();

    let radius = 100.0 - (2.0 * 100.0 * el) / consts::PI;
    let x = radius * az.sin();
    let y = radius * az.cos();

    (x, y)
}

fn draw_arc(
    ctx: &mut Context,
    color: Color,
    center: (f64, f64),
    radius: f64,
    a_min: f64,
    a_max: f64,
    segments: usize,
) {
    let mut points = vec![];

    for segment in 0..=segments {
        let angle = a_min + (segment as f64 / segments as f64) * (a_max - a_min);
        points.push((
            center.0 + angle.cos() * radius,
            center.1 + angle.sin() * radius,
        ));
    }

    let points = Points {
        coords: &points.as_slice(),
        color,
    };

    ctx.draw(&points);
}

fn render_station_view<T: Backend>(t: &mut Frame<T>, rect: Rect, station: &Station) -> Rect {
    let mut lines = 4u16;

    let station_status = match station.info.status {
        StationStatus::Online => "ONLINE",
        StationStatus::Offline => "OFFLINE",
        StationStatus::Testing => "TESTING",
    };
    let mut station_info = vec![
        Spans::from(Span::styled("Station Status", Style::default().fg(Color::Yellow))),
        Spans::default(),
        Spans::from(vec![
        Span::styled("Observation  ", Style::default().fg(Color::Cyan)),
        Span::styled(
            format!("{:>19}", station_status),
            Style::default().fg(Color::Yellow),
        ),
    ])];

    let sys_info = &station.sys_info;
    if let Some(cpu_load) = &sys_info.cpu_load {
        let load = 100.0
            - cpu_load
                .iter()
                .fold(0.0, |acc, load| acc + load.idle * 100.0)
                / cpu_load.len() as f32;

        station_info.extend_from_slice(&[Spans::from(vec![
            Span::styled("CPU          ", Style::default().fg(Color::Cyan)),
            Span::styled(format!("{:>19.1} ", load), Style::default().fg(COL_WHITE)),
            Span::styled("%", Style::default().fg(Color::LightGreen)),
        ])]);
        lines += 1;
    }

    if let Some(temp) = sys_info.cpu_temp {
        station_info.extend_from_slice(&[Spans::from(vec![
            Span::styled("CPU Temp     ", Style::default().fg(Color::Cyan)),
            Span::styled(format!("{:19.1}", temp), Style::default().fg(COL_WHITE)),
            Span::styled(" °C", Style::default().fg(Color::LightGreen)),
        ])]);
        lines += 1;
    }

    if let Some(mem) = &sys_info.mem {
        station_info.extend_from_slice(&[Spans::from(vec![
            Span::styled("Mem          ", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!(
                    "{:19.1}",
                    100.0 - (mem.free.as_u64() as f32 / mem.total.as_u64() as f32) * 100.0
                ),
                Style::default().fg(COL_WHITE),
            ),
            Span::styled(" %", Style::default().fg(Color::LightGreen)),
        ])]);
        lines += 1;
    }

    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(lines), Constraint::Min(0)].as_ref())
        .split(rect);

    let station_par = Paragraph::new(station_info)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::RIGHT)
                .border_style(Style::default().fg(COL_DARK_CYAN)),
        );

    t.render_widget(station_par, area[0]);

    area[1]
}

fn render_next_job_view<T: Backend>(t: &mut Frame<T>, rect: Rect, station: &Station) -> Rect {
    let mut jobs_rev = station.jobs.iter();
    let mut job_info = vec![];

    let lines = if let Some(job) = jobs_rev.next() {
        let delta_t = Utc::now() - job.start();
        let time_style = if delta_t >= time::Duration::zero() {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        job_info.extend_from_slice(&[
            Spans::from(vec![
                Span::styled("Next Job", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!(
                        "                {:+4}'{:2}\"",
                        delta_t.num_minutes(),
                        (delta_t.num_seconds() % 60).abs()
                    ),
                    time_style,
                ),
            ]),
            Spans::default(),
            Spans::from(vec![
                Span::styled("ID           ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{:>19}", job.id()),
                    Style::default().fg(COL_WHITE),
                ),
            ]),
            Spans::from(vec![
                Span::styled("Vessel       ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{:>19}", job.vessel_name()),
                    Style::default().fg(COL_WHITE),
                ),
            ]),
            Spans::from(vec![
                Span::styled("Start        ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{:>19}", job.start().format("%Y-%m-%d %H:%M:%S")),
                    Style::default().fg(COL_WHITE),
                ),
            ]),
            Spans::from(vec![
                Span::styled("End          ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{:>19}", job.end().format("%Y-%m-%d %H:%M:%S")),
                    Style::default().fg(COL_WHITE),
                ),
            ]),
            Spans::from(vec![
                Span::styled("Mode         ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{:>19}", job.mode()),
                    Style::default().fg(COL_WHITE),
                ),
            ]),
            Spans::from(vec![
                Span::styled("Frequency    ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{:19.3}", job.frequency_mhz()),
                    Style::default().fg(COL_WHITE),
                ),
                Span::styled(" MHz", Style::default().fg(Color::LightGreen)),
            ]),
            Spans::default(),
            Spans::from(vec![
                Span::styled("Rise         ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{:19.3}", job.observation.rise_azimuth),
                    Style::default().fg(COL_WHITE),
                ),
                Span::styled(" °", Style::default().fg(Color::LightGreen)),
            ]),
            Spans::from(vec![
                Span::styled("Max          ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{:19.3}", job.observation.max_altitude),
                    Style::default().fg(COL_WHITE),
                ),
                Span::styled(" °", Style::default().fg(Color::LightGreen)),
            ]),
            Spans::from(vec![
                Span::styled("Set          ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{:19.3}", job.observation.set_azimuth),
                    Style::default().fg(COL_WHITE),
                ),
                Span::styled(" °", Style::default().fg(Color::LightGreen)),
            ]),
        ]);

        13
    } else {
        job_info.extend_from_slice(&[
            Spans::from(Span::styled(
                "Next Job",
                Style::default().fg(Color::Yellow),
            )),
            Spans::default(),
            Spans::from(Span::styled("None\n", Style::default().fg(Color::Red))),
        ]);

        4
    };

    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(lines), Constraint::Min(0)].as_ref())
        .split(rect);

    let job_info = Paragraph::new(job_info)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::RIGHT)
                .border_style(Style::default().fg(COL_DARK_CYAN)),
        );

    t.render_widget(job_info, area[0]);

    area[1]
}

fn render_satellite_view<T: Backend>(
    t: &mut Frame<T>,
    rect: Rect,
    state: &State,
    rot_thresholds: (f64, f64),
) -> Rect {
    let mut sat_info = vec![];
    let station = state.get_active_station();
    let jobs = &station.jobs;

    sat_info.push(Spans::from(Span::styled(
        "Satellite\n\n",
        Style::default().fg(Color::Yellow),
    )));

    let lines = if jobs.is_empty() {
        sat_info.push(Spans::from(Span::styled("None", Style::default().fg(Color::Red))));

        4
    } else {
        let job = jobs.iter().next().unwrap();
        sat_info.extend_from_slice(&[
            Spans::from(vec![
                Span::styled("Orbit        ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{:>19}", job.sat().orbit_nr),
                    Style::default().fg(COL_WHITE),
                ),
            ]),
            Spans::from(vec![
                Span::styled("Latitude     ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{:>19.3}", job.sat().lat_deg),
                    Style::default().fg(COL_WHITE),
                ),
                Span::styled(" °", Style::default().fg(Color::LightGreen)),
            ]),
            Spans::from(vec![
                Span::styled("Longitude    ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{:>19.3}", job.sat().lon_deg),
                    Style::default().fg(COL_WHITE),
                ),
                Span::styled(" °", Style::default().fg(Color::LightGreen)),
            ]),
            Spans::from(vec![
                Span::styled("Altitude     ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{:>19.3}", job.sat().alt_km),
                    Style::default().fg(COL_WHITE),
                ),
                Span::styled(" km\n", Style::default().fg(Color::LightGreen)),
            ]),
            Spans::from(vec![
                Span::styled("Velocity     ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{:>19.3}", job.sat().vel_km_s),
                    Style::default().fg(COL_WHITE),
                ),
                Span::styled(" km/s", Style::default().fg(Color::LightGreen)),
            ]),
            Spans::from(vec![
                Span::styled("Range        ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{:>19.3}", job.sat().range_km),
                    Style::default().fg(COL_WHITE),
                ),
                Span::styled(" km", Style::default().fg(Color::LightGreen)),
            ]),
            Spans::from(vec![
                Span::styled("Range Rate   ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{:>19.3}", job.sat().range_rate_km_sec),
                    Style::default().fg(COL_WHITE),
                ),
                Span::styled(" km/s", Style::default().fg(Color::LightGreen)),
            ]),
        ]);

        if let Some((azimuth, elevation)) = state.rotator_position {
            let az_diff = (azimuth - job.sat().az_deg).abs();
            let el_diff = (elevation - job.sat().el_deg).abs();

            let rotator_color = match az_diff.max(el_diff) {
                delta if delta < rot_thresholds.0 => COL_WHITE,
                delta if delta < rot_thresholds.1 => Color::Yellow,
                _ => Color::Red,
            };

            sat_info.extend_from_slice(&[
                Spans::from(vec![
                    Span::styled("Azimuth     ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        format!("{:>8.3}", azimuth),
                        Style::default().fg(rotator_color),
                    ),
                    Span::styled(" °   ", Style::default().fg(Color::LightGreen)),
                    Span::styled(
                        format!("{:>7.3}", job.sat().az_deg),
                        Style::default().fg(COL_WHITE),
                    ),
                    Span::styled(" °", Style::default().fg(Color::LightGreen)),
                ]),
                Spans::from(vec![
                    Span::styled("Elevation   ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        format!("{:>8.3}", elevation),
                        Style::default().fg(rotator_color),
                    ),
                    Span::styled(" °   ", Style::default().fg(Color::LightGreen)),
                    Span::styled(
                        format!("{:>7.3}", job.sat().el_deg),
                        Style::default().fg(COL_WHITE),
                    ),
                    Span::styled(" °", Style::default().fg(Color::LightGreen)),
                ]),
            ]);
        } else {
            sat_info.extend_from_slice(&[
                Spans::from(vec![
                    Span::styled("Azimuth      ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        format!("{:>19.3}", job.sat().az_deg),
                        Style::default().fg(COL_WHITE),
                    ),
                    Span::styled(" °", Style::default().fg(Color::LightGreen)),
                ]),
                Spans::from(vec![
                    Span::styled("Elevation    ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        format!("{:>19.3}", job.sat().el_deg),
                        Style::default().fg(COL_WHITE),
                    ),
                    Span::styled(" °", Style::default().fg(Color::LightGreen)),
                ]),
            ]);
        }

        12
    };

    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(lines), Constraint::Min(0)].as_ref())
        .split(rect);

    let sat_info = Paragraph::new(sat_info)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::RIGHT)
                .border_style(Style::default().fg(COL_DARK_CYAN)),
        );

    t.render_widget(sat_info, area[0]);

    area[1]
}

fn render_future_jobs_view<T: Backend>(t: &mut Frame<T>, rect: Rect, station: &Station) -> Rect {
    let mut jobs_info = vec![];
    let mut lines = 4u16;

    jobs_info.push(Spans::from(Span::styled(
        format!("Future Jobs ({})\n\n", station.jobs.len()),
        Style::default().fg(Color::Yellow),
    )));

    if station.jobs.is_empty() {
        jobs_info.push(Spans::from(Span::styled("None\n", Style::default().fg(Color::Red))));
    } else {
        let jobs_rev = station
            .jobs
            .iter()
            .take((rect.height as usize).saturating_sub(2) / 2);

        for job in jobs_rev {
            let delta_t = Utc::now() - job.start();
            jobs_info.extend_from_slice(&[
                Spans::from(vec![
                    Span::styled(
                        format!("#{:<7}─┬", job.id()),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(
                        format!("{:>26}", job.vessel_name()),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::styled("┐", Style::default().fg(Color::Cyan)),
                ]),
                Spans::from(vec![
                    Span::styled(
                        format!(
                            "{:5}'{:2}\"",
                            delta_t.num_minutes(),
                            (delta_t.num_seconds() % 60).abs()
                        ),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled("└", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        format!("{:>10} ", job.mode()),
                        Style::default().fg(COL_WHITE),
                    ),
                    Span::styled(
                        format!("{:11.3}", job.frequency_mhz()),
                        Style::default().fg(COL_WHITE),
                    ),
                    Span::styled(" MHz", Style::default().fg(Color::LightGreen)),
                    Span::styled("┘", Style::default().fg(Color::Cyan)),
                ]),
            ]);

            lines += 2;
        }
    }

    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(lines), Constraint::Min(0)].as_ref())
        .split(rect);

    let jobs_info = Paragraph::new(jobs_info)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::RIGHT)
                .border_style(Style::default().fg(COL_DARK_CYAN)),
        );

    t.render_widget(jobs_info, area[0]);

    area[1]
}

fn render_log_view<T: Backend>(t: &mut Frame<T>, rect: Rect, logs: &LogQueue) {
    let block = Block::default()
        .borders(Borders::RIGHT | Borders::LEFT | Borders::TOP)
        .border_style(Style::default().fg(COL_DARK_CYAN))
        .title(Span::styled("Log", Style::default().fg(Color::Yellow)));

    let inner = block.inner(rect);
    let empty_line = (0..inner.width).map(|_| " ").collect::<String>() + "\n";

    // clear background of the log window
    let lines = (0..inner.height)
        .map(|_| Spans::from(Span::raw(&empty_line)))
        .collect::<Vec<_>>();

    let bg = Paragraph::new(lines);

    t.render_widget(bg, inner);
    let logs = logs.iter()
                   .take(9)
                   .collect::<Vec<_>>()
                   .iter()
                   .rev()
                   .map(|(time, level, message)| {
                       let style = match level {
                           log::Level::Warn => Style::default().fg(Color::Yellow),
                           log::Level::Error => Style::default().fg(Color::Red),
                           _ => Style::default(),
                       };

                       Spans::from(vec![
                           Span::raw(format!("{}", time)),
                           Span::styled(format!(" {:8} ", level), style),
                           Span::raw(format!("{}", message)),
                       ])
                   })
                   .collect::<Vec<_>>();

    let log = Paragraph::new(logs)
        .block(block);

    t.render_widget(log, rect);
}
