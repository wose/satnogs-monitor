use chrono::prelude::*;
use circular_queue::CircularQueue;
use failure::ResultExt;
use log::{debug, trace};
use satnogs_network_client::{Client, StationStatus};
use termion::input::{MouseTerminal, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use tui::backend::*;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::symbols::DOT;
use tui::widgets::canvas::{Canvas, Map, MapResolution, Points};
use tui::widgets::{Block, Borders, Paragraph, Text, Widget};
use tui::Frame;
use tui::Terminal;

use std::io;
use std::sync::mpsc::{sync_channel, Receiver, RecvTimeoutError, SyncSender};
use std::thread;

use crate::event::Event;
use crate::satnogs;
use crate::settings::Settings;
use crate::state::State;
use crate::station::Station;
use crate::widgets::InfoBar;

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
}

impl Ui {
    pub fn new(settings: Settings, _client: Client, state: State) -> Result<Self> {
        let (sender, reciever) = sync_channel(100);

        // Must be called before any threads are launched
        let winch_send = sender.clone();
        let signals = ::signal_hook::iterator::Signals::new(&[::libc::SIGWINCH])
            .context("couldn't register resize signal handler")?;
        thread::spawn(move || {
            for _ in &signals {
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
            sender: sender,
            settings,
            show_logs: false,
            shutdown: false,
            size: Rect::default(),
            state: state,
            terminal: terminal,
            ticks: 0,
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
        let state = &self.state;

        self.terminal
            .draw(|mut f| {
                InfoBar::new(state)
                    .style(Style::default().fg(Color::White).bg(Color::DarkGray))
                    .block(Block::default())
                    .render(&mut f, rows[0]);

                let rect = render_station_view(&mut f, body[0], &station);
                let rect = render_next_job_view(&mut f, rect, &station);
                let rect = render_satellite_view(&mut f, rect, &station);
                let rect = render_future_jobs_view(&mut f, rect, &station);

                // to create the rest of the border we add an empty paragraph
                Paragraph::new([].iter())
                    .block(
                        Block::default()
                            .borders(Borders::RIGHT)
                            .border_style(Style::default().fg(COL_DARK_CYAN)),
                    )
                    .render(&mut f, rect);

                render_map_view(&mut f, body[1], &station, ground_tracks);

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
            Key(Char('l')) => self.show_logs = !self.show_logs,
            Key(Char('\t')) => self.next_station(),
            Key(Ctrl('\t')) => self.prev_station(),
            Key(Char('q')) => self.shutdown = true,
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
        }
    }

    fn handle_tick(&mut self) {
        if self.last_job_update.elapsed().as_secs() >= 600 {
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

        for (id, _) in &self.state.stations {
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

fn render_map_view<T: Backend>(
    t: &mut Frame<T>,
    rect: Rect,
    station: &Station,
    ground_tracks: usize,
) {
    Canvas::default()
        .paint(|ctx| {
            ctx.draw(&Map {
                color: COL_LIGHT_BG,
                resolution: MapResolution::High,
            });

            ctx.print(station.info.lng, station.info.lat, DOT, Color::LightCyan);

            if let Some(job) = station.jobs.iter().next() {
                let marker = format!("■─{}", job.vessel_name());
                ctx.print(
                    job.sat().lon_deg,
                    job.sat().lat_deg,
                    marker,
                    Color::LightRed,
                );
                ctx.layer();
                let mut ground_track = Points::default();
                // plot future orbits first so the current orbit will be drawn on top
                ground_track.color = Color::Cyan;;
                ground_track.coords =
                    &job.vessel.ground_track[job.vessel.ground_track.len() / ground_tracks..];
                ctx.draw(&ground_track);

                ctx.layer();
                ground_track.color = Color::Yellow;
                ground_track.coords =
                    &job.vessel.ground_track[..job.vessel.ground_track.len() / ground_tracks];
                ctx.draw(&ground_track);
            }
        })
        .x_bounds([-180.0, 180.0])
        .y_bounds([-90.0, 90.0])
        .render(t, rect);
}

fn render_station_view<T: Backend>(t: &mut Frame<T>, rect: Rect, station: &Station) -> Rect {
    let mut lines = 4u16;

    let station_status = match station.info.status {
        StationStatus::Online => "ONLINE",
        StationStatus::Offline => "OFFLINE",
        StationStatus::Testing => "TESTING",
    };
    let mut station_info = vec![
        Text::styled("Station Status\n\n", Style::default().fg(Color::Yellow)),
        Text::styled("Observation  ", Style::default().fg(Color::Cyan)),
        Text::styled(
            format!("{:>19}\n", station_status),
            Style::default().fg(Color::Yellow),
        ),
    ];

    let sys_info = &station.sys_info;
    if let Some(cpu_load) = &sys_info.cpu_load {
        let load = 100.0
            - cpu_load
                .iter()
                .fold(0.0, |acc, load| acc + load.idle * 100.0)
                / cpu_load.len() as f32;

        station_info.extend_from_slice(&[
            Text::styled("CPU          ", Style::default().fg(Color::Cyan)),
            Text::styled(format!("{:>19.1} ", load), Style::default().fg(COL_WHITE)),
            Text::styled("%\n", Style::default().fg(Color::LightGreen)),
        ]);
        lines += 1;
    }

    if let Some(temp) = sys_info.cpu_temp {
        station_info.extend_from_slice(&[
            Text::styled("CPU Temp     ", Style::default().fg(Color::Cyan)),
            Text::styled(format!("{:19.1}", temp), Style::default().fg(COL_WHITE)),
            Text::styled(" °C\n", Style::default().fg(Color::LightGreen)),
        ]);
        lines += 1;
    }

    if let Some(mem) = &sys_info.mem {
        station_info.extend_from_slice(&[
            Text::styled("Mem          ", Style::default().fg(Color::Cyan)),
            Text::styled(
                format!(
                    "{:19.1}",
                    100.0 - (mem.free.as_u64() as f32 / mem.total.as_u64() as f32) * 100.0
                ),
                Style::default().fg(COL_WHITE),
            ),
            Text::styled(" %\n", Style::default().fg(Color::LightGreen)),
        ]);
        lines += 1;
    }

    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(lines), Constraint::Min(0)].as_ref())
        .split(rect);

    Paragraph::new(station_info.iter())
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::RIGHT)
                .border_style(Style::default().fg(COL_DARK_CYAN)),
        )
        .render(t, area[0]);

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
            Text::styled("Next Job", Style::default().fg(Color::Yellow)),
            Text::styled(
                format!(
                    "                {:+4}'{:2}\"\n\n",
                    delta_t.num_minutes(),
                    (delta_t.num_seconds() % 60).abs()
                ),
                time_style,
            ),
            Text::styled("ID           ", Style::default().fg(Color::Cyan)),
            Text::styled(
                format!("{:>19}\n", job.id()),
                Style::default().fg(COL_WHITE),
            ),
            Text::styled("Vessel       ", Style::default().fg(Color::Cyan)),
            Text::styled(
                format!("{:>19}\n", job.vessel_name()),
                Style::default().fg(COL_WHITE),
            ),
            Text::styled("Start        ", Style::default().fg(Color::Cyan)),
            Text::styled(
                format!("{:>19}\n", job.start().format("%Y-%m-%d %H:%M:%S")),
                Style::default().fg(COL_WHITE),
            ),
            Text::styled("End          ", Style::default().fg(Color::Cyan)),
            Text::styled(
                format!("{:>19}\n", job.end().format("%Y-%m-%d %H:%M:%S")),
                Style::default().fg(COL_WHITE),
            ),
            Text::styled("Mode         ", Style::default().fg(Color::Cyan)),
            Text::styled(
                format!("{:>19}\n", job.mode()),
                Style::default().fg(COL_WHITE),
            ),
            Text::styled("Frequency    ", Style::default().fg(Color::Cyan)),
            Text::styled(
                format!("{:19.3}", job.frequency_mhz()),
                Style::default().fg(COL_WHITE),
            ),
            Text::styled(" MHz\n\n", Style::default().fg(Color::LightGreen)),
            Text::styled("Rise         ", Style::default().fg(Color::Cyan)),
            Text::styled(
                format!("{:19.3}", job.observation.rise_azimuth),
                Style::default().fg(COL_WHITE),
            ),
            Text::styled(" °\n", Style::default().fg(Color::LightGreen)),
            Text::styled("Max          ", Style::default().fg(Color::Cyan)),
            Text::styled(
                format!("{:19.3}", job.observation.max_altitude),
                Style::default().fg(COL_WHITE),
            ),
            Text::styled(" °\n", Style::default().fg(Color::LightGreen)),
            Text::styled("Set          ", Style::default().fg(Color::Cyan)),
            Text::styled(
                format!("{:19.3}", job.observation.set_azimuth),
                Style::default().fg(COL_WHITE),
            ),
            Text::styled(" °\n", Style::default().fg(Color::LightGreen)),
        ]);

        13
    } else {
        job_info.push(Text::styled(
            "Next Job\n\n",
            Style::default().fg(Color::Yellow),
        ));
        job_info.push(Text::styled("None\n", Style::default().fg(Color::Red)));

        4
    };

    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(lines), Constraint::Min(0)].as_ref())
        .split(rect);

    Paragraph::new(job_info.iter())
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::RIGHT)
                .border_style(Style::default().fg(COL_DARK_CYAN)),
        )
        .render(t, area[0]);

    area[1]
}

fn render_satellite_view<T: Backend>(t: &mut Frame<T>, rect: Rect, station: &Station) -> Rect {
    let mut sat_info = vec![];
    let jobs = &station.jobs;

    sat_info.push(Text::styled(
        "Satellite\n\n",
        Style::default().fg(Color::Yellow),
    ));

    let lines = if jobs.is_empty() {
        sat_info.push(Text::styled("None\n", Style::default().fg(Color::Red)));

        4
    } else {
        let job = jobs.iter().next().unwrap();
        sat_info.extend_from_slice(&[
            Text::styled("Orbit        ", Style::default().fg(Color::Cyan)),
            Text::styled(
                format!("{:>19}", job.sat().orbit_nr),
                Style::default().fg(COL_WHITE),
            ),
            Text::styled("\n", Style::default().fg(Color::LightGreen)),
            Text::styled("Latitude     ", Style::default().fg(Color::Cyan)),
            Text::styled(
                format!("{:>19.3}", job.sat().lat_deg),
                Style::default().fg(COL_WHITE),
            ),
            Text::styled(" °\n", Style::default().fg(Color::LightGreen)),
            Text::styled("Longitude    ", Style::default().fg(Color::Cyan)),
            Text::styled(
                format!("{:>19.3}", job.sat().lon_deg),
                Style::default().fg(COL_WHITE),
            ),
            Text::styled(" °\n", Style::default().fg(Color::LightGreen)),
            Text::styled("Altitude     ", Style::default().fg(Color::Cyan)),
            Text::styled(
                format!("{:>19.3}", job.sat().alt_km),
                Style::default().fg(COL_WHITE),
            ),
            Text::styled(" km\n", Style::default().fg(Color::LightGreen)),
            Text::styled("Velocity     ", Style::default().fg(Color::Cyan)),
            Text::styled(
                format!("{:>19.3}", job.sat().vel_km_s),
                Style::default().fg(COL_WHITE),
            ),
            Text::styled(" km/s\n", Style::default().fg(Color::LightGreen)),
            Text::styled("Range        ", Style::default().fg(Color::Cyan)),
            Text::styled(
                format!("{:>19.3}", job.sat().range_km),
                Style::default().fg(COL_WHITE),
            ),
            Text::styled(" km\n", Style::default().fg(Color::LightGreen)),
            Text::styled("Range Rate   ", Style::default().fg(Color::Cyan)),
            Text::styled(
                format!("{:>19.3}", job.sat().range_rate_km_sec),
                Style::default().fg(COL_WHITE),
            ),
            Text::styled(" km/s\n", Style::default().fg(Color::LightGreen)),
        ]);

        10
    };

    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(lines), Constraint::Min(0)].as_ref())
        .split(rect);

    Paragraph::new(sat_info.iter())
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::RIGHT)
                .border_style(Style::default().fg(COL_DARK_CYAN)),
        )
        .render(t, area[0]);

    area[1]
}

fn render_future_jobs_view<T: Backend>(t: &mut Frame<T>, rect: Rect, station: &Station) -> Rect {
    let mut jobs_info = vec![];
    let mut lines = 4u16;

    jobs_info.push(Text::styled(
        format!("Future Jobs ({})\n\n", station.jobs.len()),
        Style::default().fg(Color::Yellow),
    ));

    if station.jobs.is_empty() {
        jobs_info.push(Text::styled("None\n", Style::default().fg(Color::Red)));
    } else {
        let mut jobs_rev = station
            .jobs
            .iter()
            .take((rect.height as usize).saturating_sub(2) / 2);

        while let Some(job) = jobs_rev.next() {
            let delta_t = Utc::now() - job.start();
            jobs_info.extend_from_slice(&[
                Text::styled(
                    format!("#{:<7}─┬", job.id()),
                    Style::default().fg(Color::Cyan),
                ),
                Text::styled(
                    format!("{:>26}", job.vessel_name()),
                    Style::default().fg(Color::Yellow),
                ),
                Text::styled("┐\n", Style::default().fg(Color::Cyan)),
                Text::styled(
                    format!(
                        "{:5}'{:2}\"",
                        delta_t.num_minutes(),
                        (delta_t.num_seconds() % 60).abs()
                    ),
                    Style::default().fg(Color::DarkGray),
                ),
                Text::styled("└", Style::default().fg(Color::Cyan)),
                Text::styled(
                    format!("{:>10} ", job.mode()),
                    Style::default().fg(COL_WHITE),
                ),
                Text::styled(
                    format!("{:11.3}", job.frequency_mhz()),
                    Style::default().fg(COL_WHITE),
                ),
                Text::styled(" MHz", Style::default().fg(Color::LightGreen)),
                Text::styled("┘\n", Style::default().fg(Color::Cyan)),
            ]);

            lines += 2;
        }
    }

    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(lines), Constraint::Min(0)].as_ref())
        .split(rect);

    Paragraph::new(jobs_info.iter())
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::RIGHT)
                .border_style(Style::default().fg(COL_DARK_CYAN)),
        )
        .render(t, area[0]);

    area[1]
}

fn render_log_view<T: Backend>(t: &mut Frame<T>, rect: Rect, logs: &LogQueue) {
    let block = Block::default()
        .borders(Borders::RIGHT | Borders::LEFT | Borders::TOP)
        .border_style(Style::default().fg(COL_DARK_CYAN))
        .style(Style::default().modifier(tui::style::Modifier::Reset))
        .title("Log")
        .title_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(rect);
    let empty_line = (0..inner.width).map(|_| " ").collect::<String>() + "\n";

    // clear background of the log window
    Paragraph::new(
        (0..inner.height)
            .map(|_| Text::raw(&empty_line))
            .collect::<Vec<_>>()
            .iter(),
    )
    .render(t, inner);

    Paragraph::new(
        logs.iter()
            .take(9)
            .map(|(time, level, message)| {
                let style = match level {
                    log::Level::Warn => Style::default().fg(Color::Yellow),
                    log::Level::Error => Style::default().fg(Color::Red),
                    _ => Style::default(),
                };

                (
                    Text::raw(format!("{}", time)),
                    Text::styled(format!(" {:8} ", level), style),
                    Text::raw(format!("{}\n", message)),
                )
            })
            .collect::<Vec<_>>()
            .iter()
            .rev()
            .fold(Vec::new(), |mut logs, log| {
                logs.push(&log.0);
                logs.push(&log.1);
                logs.push(&log.2);
                logs
            })
            .into_iter(),
    )
    .block(block)
    .render(t, rect);
}
