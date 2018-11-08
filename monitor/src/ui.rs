use chrono::prelude::*;
use circular_queue::CircularQueue;
use log::{debug, info, trace, warn};
use termion::input::{MouseTerminal, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::symbols::DOT;
use tui::widgets::canvas::{Canvas, Map, MapResolution, Points};
use tui::widgets::{Block, Borders, Paragraph, Text, Widget};
use tui::Terminal;
use unicode_width::UnicodeWidthStr;

use std::io;
use std::sync::mpsc::{sync_channel, Receiver, RecvTimeoutError, SyncSender};
use std::thread;
//use std::time::Duration;

use crate::event::Event;
use crate::satnogs;
use crate::settings::Settings;
use crate::station::{Station, StationStatus};
use crate::vessel::Vessel;

use crate::Result;

//const COL_DARK_BG: Color = Color::Rgb(0x10, 0x10, 0x10);
const COL_LIGHT_BG: Color = Color::Rgb(0x77, 0x77, 0x77);
const COL_CYAN: Color = Color::Rgb(0x0C, 0x7C, 0x73);
const COL_DARK_CYAN: Color = Color::Rgb(0x09, 0x35, 0x33);
/*
const COL_LIGHT_CYAN: Color = Color::Rgb(0x04, 0xF1, 0xF1);
const COL_DARK_GREEN: Color = Color::Rgb(0x14, 0x22, 0x1A);
const COL_LIGHT_GREEN: Color = Color::Rgb(0x32, 0x4D, 0x38);
const COL_DARK_RED: Color = Color::Rgb(0x39, 0x08, 0x0C);
const COL_LIGHT_RED: Color = Color::Rgb(0x77, 0x06, 0x0C);
*/
const COL_WHITE: Color = Color::Rgb(0xFA, 0xFA, 0xFA);

type Backend = TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<io::Stdout>>>>;

pub struct Ui {
    active_station: usize,
    events: Receiver<Event>,
    jobs: Vec<satnogs_network_client::Job>,
    logs: CircularQueue<(DateTime<Utc>, log::Level, String)>,
    last_job_update: std::time::Instant,
    network: satnogs::Connection,
    sender: SyncSender<Event>,
    show_logs: bool,
    shutdown: bool,
    size: Rect,
    stations: Vec<Station>,
    terminal: Terminal<Backend>,
    ticks: u32,
    vessels: Vec<Vessel>,
}

impl Ui {
    pub fn new(settings: &Settings) -> Self {
        let (sender, reciever) = sync_channel(100);

        // Must be called before any threads are launched
        let winch_send = sender.clone();
        let signals = ::signal_hook::iterator::Signals::new(&[::libc::SIGWINCH])
            .expect("Couldn't register resize signal handler");
        thread::spawn(move || {
            for _ in &signals {
                let _ = winch_send.send(Event::Resize);
            }
        });

        let send = sender.clone();
        thread::spawn(move || {
            for event in ::std::io::stdin().events() {
                if let Ok(ev) = event {
                    let _ = send.send(Event::Input(ev));
                }
            }
        });

        let send = sender.clone();
        thread::spawn(move || {
            while send.send(Event::Tick).is_ok() {
                thread::sleep(std::time::Duration::new(1, 0));
            }
        });

        let stdout = io::stdout()
            .into_raw_mode()
            .expect("Failted to put stdout into raw mode");
        let stdout = MouseTerminal::from(stdout);
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend).expect("Failed to create terminal");

        terminal.clear().unwrap();
        terminal.hide_cursor().unwrap();

        let stations: Vec<_> = settings
            .stations
            .iter()
            .map(|sc|
                Station {
                active: true,
                id: sc.satnogs_id,
                lat: None,
                lng: None,
                name: sc.name.to_string(),
                observations: vec![],
                    status: StationStatus::Idle,
                }
            )
            .collect();

        let mut ui = Self {
            active_station: 0,
            events: reciever,
            jobs: vec![],
            last_job_update: std::time::Instant::now(),
            logs: CircularQueue::with_capacity(100),
            network: satnogs::Connection::new(sender.clone()),
            sender: sender,
            show_logs: false,
            shutdown: false,
            size: Rect::default(),
            stations: stations,
            terminal: terminal,
            ticks: 0,
            vessels: vec![],
        };

        for station in &ui.stations {
            ui.network.send(satnogs::Command::GetStationInfo(station.id)).unwrap();
        }
        ui
    }

    pub fn sender(&self) -> SyncSender<Event> {
        self.sender.clone()
    }

    fn next_station(&mut self) {
        self.active_station = if self.active_station + 1 >= self.stations.len() {
            0
        } else {
            self.active_station + 1
        }
    }

    fn prev_station(&mut self) {
        self.active_station = if self.active_station == 0 {
            self.stations.len()
        } else {
            self.active_station - 1
        }
    }

    fn update_vessel_position(&mut self) {
        for vessel in &mut self.vessels {
            vessel.update_position();
        }
    }

    pub fn update_ground_tracks(&mut self) {
        for vessel in &mut self.vessels {
            vessel.update_ground_track();
        }
    }

    fn draw(&mut self) {
        let size = self.terminal.size().expect("Failed to get terminal size");
        if self.size != size {
            self.terminal
                .resize(size)
                .expect("Failed to resize terminal");
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

        let header = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(27)].as_ref())
            .split(rows[0]);

        let mut tabs = vec![];
        tabs.push(Text::styled(
            "🛰  ",
            Style::default().fg(COL_WHITE).bg(COL_DARK_CYAN),
        ));
        tabs.push(Text::styled(
            " NETWORK ",
            Style::default().fg(COL_WHITE).bg(COL_DARK_CYAN),
        ));
        tabs.push(Text::raw("         "));
        for station in &self.stations {
            format_station(&station, 1, &mut tabs);
            tabs.push(Text::raw("   "));
        }

        let decal = (0..9).map(|_| "▀").collect::<String>();
        tabs.push(Text::raw("\n"));
        tabs.push(Text::raw("   "));
        tabs.push(Text::styled(decal, Style::default().fg(COL_DARK_CYAN)));
        tabs.push(Text::raw("         "));
        for station in &self.stations {
            format_station(&station, 2, &mut tabs);
            tabs.push(Text::raw("   "));
        }

        let utc: DateTime<Utc> = Utc::now();
        let jobs = &self.jobs;
        let vessels = &self.vessels;
        let logs = &self.logs;
        let show_logs = self.show_logs;
        let station = self.stations.get(self.active_station);

        self.terminal
            .draw(|mut f| {
                // the "Tabs"
                Paragraph::new(tabs.iter())
                    .alignment(Alignment::Left)
                    .style(Style::default().fg(Color::White))
                    .render(&mut f, header[0]);

                // UTC clock
                let decal = (0..25).map(|_| "▀").collect::<String>();

                Paragraph::new(
                    [
                        Text::styled(
                            utc.format("    %F").to_string(),
                            Style::default().fg(COL_WHITE).bg(COL_DARK_CYAN),
                        ),
                        Text::styled(
                            utc.format(" %Z ").to_string(),
                            Style::default().fg(COL_WHITE).bg(COL_DARK_CYAN),
                        ),
                        Text::styled(
                            utc.format("%T").to_string(),
                            Style::default().fg(COL_WHITE).bg(COL_DARK_CYAN),
                        ),
                        Text::raw("\n   "),
                        Text::styled(decal, Style::default().fg(COL_DARK_CYAN)),
                    ].into_iter(),
                ).alignment(Alignment::Left)
                    .render(&mut f, header[1]);

                // left bar
                // - station status
                // - current observation info

                let mut station_info = vec![
                    Text::styled("Station Status\n\n", Style::default().fg(Color::Yellow)),
                    Text::styled("Observation  ", Style::default().fg(Color::Cyan)),
                    Text::styled("               IDLE\n", Style::default().fg(Color::Yellow)),
                    Text::styled("CPU          ", Style::default().fg(Color::Cyan)),
                    Text::styled("                 11", Style::default().fg(COL_WHITE)),
                    Text::styled(" %\n", Style::default().fg(Color::LightGreen)),
                    Text::styled("CPU Temp     ", Style::default().fg(Color::Cyan)),
                    Text::styled("               54.3", Style::default().fg(COL_WHITE)),
                    Text::styled(" °C\n", Style::default().fg(Color::LightGreen)),
                    Text::styled("MEM          ", Style::default().fg(Color::Cyan)),
                    Text::styled("                 28", Style::default().fg(COL_WHITE)),
                    Text::styled(" %\n", Style::default().fg(Color::LightGreen)),
                    Text::styled("FS /tmp      ", Style::default().fg(Color::Cyan)),
                    Text::styled("                 53", Style::default().fg(COL_WHITE)),
                    Text::styled(" %\n", Style::default().fg(Color::LightGreen)),
                    Text::raw("\n"),
                ];

                let mut jobs_rev = jobs.iter().rev();
                if let Some(job) = jobs_rev.next() {
                    let delta_t = Utc::now() - job.start;
                    station_info.extend_from_slice(&[
                        Text::styled("Next Job", Style::default().fg(Color::Yellow)),
                        Text::styled(
                            format!(
                                "                {:4}'{:2}\"\n\n",
                                delta_t.num_minutes(),
                                (delta_t.num_seconds() % 60).abs()
                            ),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Text::styled("ID           ", Style::default().fg(Color::Cyan)),
                        Text::styled(format!("{:>19}\n", job.id), Style::default().fg(COL_WHITE)),
                        Text::styled("Vessel       ", Style::default().fg(Color::Cyan)),
                        Text::styled(
                            format!("{:>19}\n", job.tle0),
                            Style::default().fg(COL_WHITE),
                        ),
                        Text::styled("Start        ", Style::default().fg(Color::Cyan)),
                        Text::styled(
                            format!("{:>19}\n", job.start.format("%Y-%m-%d %H:%M:%S")),
                            Style::default().fg(COL_WHITE),
                        ),
                        Text::styled("End          ", Style::default().fg(Color::Cyan)),
                        Text::styled(
                            format!("{:>19}\n", job.end.format("%Y-%m-%d %H:%M:%S")),
                            Style::default().fg(COL_WHITE),
                        ),
                        Text::styled("Mode         ", Style::default().fg(Color::Cyan)),
                        Text::styled(
                            format!("{:>19}\n", job.mode),
                            Style::default().fg(COL_WHITE),
                        ),
                        Text::styled("Frequency    ", Style::default().fg(Color::Cyan)),
                        Text::styled(
                            format!("{:19.3}", job.frequency as f64 / 1_000_000.0),
                            Style::default().fg(COL_WHITE),
                        ),
                        Text::styled(" Mhz\n\n", Style::default().fg(Color::LightGreen)),
                    ]);
                } else {
                    station_info.push(Text::styled(
                        "Next Job\n\n",
                        Style::default().fg(Color::Yellow),
                    ));
                    station_info.push(Text::styled("None\n\n", Style::default().fg(Color::Red)));
                }

                station_info.push(Text::styled(
                    format!("Future Jobs ({})\n\n", jobs.len()),
                    Style::default().fg(Color::Yellow),
                ));

                if jobs.is_empty() {
                    station_info.push(Text::styled("None\n\n", Style::default().fg(Color::Red)));
                } else {
                    let mut jobs_rev = jobs_rev.take(5);
                    while let Some(job) = jobs_rev.next() {
                        let delta_t = Utc::now() - job.start;
                        station_info.extend_from_slice(&[
                            Text::styled(
                                format!("#{:<7}─┬", job.id),
                                Style::default().fg(Color::Cyan),
                            ),
                            Text::styled(
                                format!("{:>26}", job.tle0),
                                Style::default().fg(Color::Yellow),
                            ),
                            Text::styled("┐\n", Style::default().fg(Color::Cyan)),
                            Text::styled(
                                format!(
                                    "{:4}'{:2}\"",
                                    delta_t.num_minutes(),
                                    (delta_t.num_seconds() % 60).abs()
                                ),
                                Style::default().fg(Color::DarkGray),
                            ),
                            Text::styled(" └", Style::default().fg(Color::Cyan)),
                            Text::styled(
                                format!("{:>8} ", job.mode),
                                Style::default().fg(COL_WHITE),
                            ),
                            Text::styled(
                                format!("{:13.3}", job.frequency as f64 / 1_000_000.0),
                                Style::default().fg(COL_WHITE),
                            ),
                            Text::styled(" Mhz", Style::default().fg(Color::LightGreen)),
                            Text::styled("┘\n", Style::default().fg(Color::Cyan)),
                        ]);
                    }
                }

                station_info.extend_from_slice(&[
                    Text::styled("\n\nObservation\n\n", Style::default().fg(Color::Yellow)),
                    Text::styled("ID           ", Style::default().fg(Color::Cyan)),
                    Text::styled("            #309482", Style::default().fg(COL_WHITE)),
                    Text::styled("\n", Style::default().fg(Color::LightGreen)),
                    Text::styled("Satellite    ", Style::default().fg(Color::Cyan)),
                    Text::styled("          25544 ISS", Style::default().fg(COL_WHITE)),
                    Text::styled("\n", Style::default().fg(Color::LightGreen)),
                    Text::styled("Observer     ", Style::default().fg(Color::Cyan)),
                    Text::styled("               wose", Style::default().fg(COL_WHITE)),
                    Text::styled("\n", Style::default().fg(Color::LightGreen)),
                    Text::styled("Start        ", Style::default().fg(Color::Cyan)),
                    Text::styled("2018-11-02 02:13:41", Style::default().fg(COL_WHITE)),
                    Text::styled("\n", Style::default().fg(Color::LightGreen)),
                    Text::styled("End          ", Style::default().fg(Color::Cyan)),
                    Text::styled("2018-11-02 02:23:10", Style::default().fg(COL_WHITE)),
                    Text::styled("\n", Style::default().fg(Color::LightGreen)),
                    Text::styled("Transmitter  ", Style::default().fg(Color::Cyan)),
                    Text::styled("  Mode U/V FM Voice", Style::default().fg(COL_WHITE)),
                    Text::styled("\n", Style::default().fg(Color::LightGreen)),
                    Text::styled("Frequency    ", Style::default().fg(Color::Cyan)),
                    Text::styled("            145.800", Style::default().fg(COL_WHITE)),
                    Text::styled(" MHz\n", Style::default().fg(Color::LightGreen)),
                    Text::styled("Rise         ", Style::default().fg(Color::Cyan)),
                    Text::styled("              199.0", Style::default().fg(COL_WHITE)),
                    Text::styled(" °\n", Style::default().fg(Color::LightGreen)),
                    Text::styled("Max          ", Style::default().fg(Color::Cyan)),
                    Text::styled("               14.0", Style::default().fg(COL_WHITE)),
                    Text::styled(" °\n", Style::default().fg(Color::LightGreen)),
                    Text::styled("Set          ", Style::default().fg(Color::Cyan)),
                    Text::styled("               79.0", Style::default().fg(COL_WHITE)),
                    Text::styled(" °\n", Style::default().fg(Color::LightGreen)),
                    Text::styled("\n\nSatellite\n\n", Style::default().fg(Color::Yellow)),
                ]);
                if vessels.is_empty() {
                    station_info.push(Text::styled("None\n\n", Style::default().fg(Color::Red)));
                } else {
                    station_info.extend_from_slice(&[
                        Text::styled("Orbit    ", Style::default().fg(Color::Cyan)),
                        Text::styled(
                            format!("{:>19}", vessels[0].sat().orbit_nr),
                            Style::default().fg(COL_WHITE),
                        ),
                        Text::styled("\n", Style::default().fg(Color::LightGreen)),
                        Text::styled("Latitude     ", Style::default().fg(Color::Cyan)),
                        Text::styled(
                            format!("{:>19.3}", vessels[0].sat().lat_deg),
                            Style::default().fg(COL_WHITE),
                        ),
                        Text::styled(" °\n", Style::default().fg(Color::LightGreen)),
                        Text::styled("Longitude    ", Style::default().fg(Color::Cyan)),
                        Text::styled(
                            format!("{:>19.3}", vessels[0].sat().lon_deg),
                            Style::default().fg(COL_WHITE),
                        ),
                        Text::styled(" °\n", Style::default().fg(Color::LightGreen)),
                        Text::styled("Altitude     ", Style::default().fg(Color::Cyan)),
                        Text::styled(
                            format!("{:>19.3}", vessels[0].sat().alt_km),
                            Style::default().fg(COL_WHITE),
                        ),
                        Text::styled(" km\n", Style::default().fg(Color::LightGreen)),
                        Text::styled("Velocity     ", Style::default().fg(Color::Cyan)),
                        Text::styled(
                            format!("{:>19.3}", vessels[0].sat().vel_km_s),
                            Style::default().fg(COL_WHITE),
                        ),
                        Text::styled(" km/s\n", Style::default().fg(Color::LightGreen)),
                        Text::styled("Range        ", Style::default().fg(Color::Cyan)),
                        Text::styled(
                            format!("{:>19.3}", vessels[0].sat().range_km),
                            Style::default().fg(COL_WHITE),
                        ),
                        Text::styled(" km\n", Style::default().fg(Color::LightGreen)),
                        Text::styled("Range Rate   ", Style::default().fg(Color::Cyan)),
                        Text::styled(
                            format!("{:>19.3}", vessels[0].sat().range_rate_km_sec),
                            Style::default().fg(COL_WHITE),
                        ),
                        Text::styled(" km/s\n", Style::default().fg(Color::LightGreen)),
                    ]);
                }

                Paragraph::new(station_info.iter())
                    .alignment(Alignment::Left)
                    .block(
                        Block::default()
                            .borders(Borders::RIGHT)
                            .border_style(Style::default().fg(COL_DARK_CYAN)),
                    )
                    .render(&mut f, body[0]);

                // map with current obs vessel
                Canvas::default()
//                    .block(Block::default().borders(Borders::LEFT).border_style(Style::default().fg(COL_DARK_CYAN)))
                    .paint(|ctx| {
                        ctx.draw(&Map {
                            color: COL_LIGHT_BG,
                            resolution: MapResolution::High,
                        });
                        if let Some(station) = station {
                            if let (Some(lat), Some(lng)) = (station.lat, station.lng) {
                                ctx.print(lng, lat, DOT, Color::LightCyan);
                            }
                        }
                        for vessel in vessels {
                            let marker = format!("■─{}", vessel.name());
                            ctx.print(vessel.sat().lon_deg,
                                      vessel.sat().lat_deg,
                                      marker,
                                      Color::LightRed);
                            let mut ground_track = Points::default();
                            ground_track.color = Color::Yellow;
                            ground_track.coords = &vessel.ground_track;
                            ctx.layer();
                            ctx.draw(&ground_track);
                        }
                    })
                    .x_bounds([-180.0, 180.0])
                    .y_bounds([-90.0, 90.0])
                    .render(&mut f, body[1]);
                //.render(&mut f, obs_rt_info[1]);

                if show_logs {
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
                    ).alignment(Alignment::Left)
                        .block(
                            Block::default()
                                .borders(Borders::RIGHT | Borders::LEFT | Borders::TOP)
                                .border_style(Style::default().fg(COL_DARK_CYAN))
                                .title("Log")
                                .title_style(Style::default().fg(Color::Yellow)),
                        )
                        .render(&mut f, log_area);
                }
            })
            .expect("Failed to draw to terminal");
    }

    fn handle_input(&mut self, event: &::termion::event::Event) {
        use termion::event::Event::*;
        use termion::event::Key::*;
        //        use termion::event::{MouseButton, MouseEvent};

        match *event {
            Key(Ctrl('c')) => self.shutdown = true,
            Key(Char('l')) => self.show_logs = !self.show_logs,
            Key(Char('\t')) => self.next_station(),
            Key(Char('q')) => self.shutdown = true,
            Key(key) => {
                debug!("Key Event: {:?}", key);
            },
            _ => {}
        }
    }

    fn handle_event(&mut self, event: Event) {
        match event {
            Event::CommandResponse(data) => match data {
                satnogs::Data::Jobs(station_id, jobs) => {
                    if station_id == self.stations[self.active_station].id {
                        info!("Got {} jobs for station {}", jobs.len(), station_id);
                        self.vessels.clear();
                        if let Some(job) = jobs.iter().rev().next() {
                            self.vessels.clear();
                            self.vessels
                                .push(Vessel::new(&job.tle0, &job.tle1, &job.tle2));
                            self.update_ground_tracks();
                            self.update_vessel_position();
                        }
                    }
                    self.jobs = jobs;
                }
                satnogs::Data::Observations(_) => info!("Got observations update"),
                satnogs::Data::StationInfo(station_id, info) => {
                    info!("Got info for station {}", station_id);
                    if let Some(station) = self.stations.iter_mut().find(|station| station.id == station_id) {
                        station.lat = Some(info.lat);
                        station.lng = Some(info.lng);
                    }
                }
            },
            Event::Resize => {
                debug!("Terminal size changed")
            }
            Event::Input(event) => {
                self.handle_input(&event);
            }
            Event::Log((level, message)) => {
                self.logs.push((Utc::now(), level, message));
            }
            Event::NoSatnogsNetworkConnection => {
                warn!("No connection to SatNOGS network");
            }
            Event::Shutdown => self.shutdown = true,
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
            self.update_vessel_position();
        }

        if self.ticks % 60 == 0 {
            self.update_ground_tracks();
        }
    }

    fn update_jobs(&mut self) {
        trace!("Requesting jobs update");

        for station in &self.stations {
            self.network
                .send(satnogs::Command::GetJobs(Some(station.id as i64)))
                .unwrap();
        }
        self.last_job_update = std::time::Instant::now();
    }

    pub fn run(mut self) {
        use std::time::{Duration, Instant};

        self.update_jobs();
        self.draw();

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

            self.draw();

            if self.shutdown {
                break;
            }
        }
    }
}

fn format_station(station: &Station, line: u32, data: &mut Vec<Text>) {
    let bg = if station.active {
        COL_CYAN
    } else {
        COL_DARK_CYAN
    };

    match line {
        1 => {
            let status = match station.status {
                StationStatus::Idle => {
                    Text::styled("▲", Style::default().fg(Color::Yellow).bg(bg))
                }
                StationStatus::Observing => {
                    Text::styled("▲", Style::default().fg(Color::LightGreen).bg(bg))
                }
                StationStatus::Offline => {
                    Text::styled("▲", Style::default().fg(Color::LightRed).bg(bg))
                }
            };

            data.extend_from_slice(&[
                Text::styled(" ", Style::default().fg(COL_WHITE).bg(bg)),
                status,
                //            Text::styled("▲", Style::default().fg(COL_LIGHT_GREEN).bg(COL_DARK_CYAN)),
                Text::styled(" ", Style::default().fg(COL_WHITE).bg(bg)),
                Text::styled(
                    format!(" {} ", station),
                    Style::default().fg(COL_WHITE).bg(bg),
                ),
            ]);
        }
        2 => {
            let decal = (0..UnicodeWidthStr::width(format!(" {} ", station).as_str()))
                .map(|_| "▀")
                .collect::<String>();
            data.extend_from_slice(&[
                Text::raw("   "),
                Text::styled(decal, Style::default().fg(bg)),
            ]);
        }
        _ => (),
    }
}
