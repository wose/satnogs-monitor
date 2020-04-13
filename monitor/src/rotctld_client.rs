use crate::Result;

use std::io::Write;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;

pub struct RotCtldClient {
    reader: BufReader<TcpStream>,
    writer: TcpStream,
}

impl RotCtldClient {
    pub fn new(address: &str) -> Result<Self> {
        let stream = TcpStream::connect(address)?;
        stream.set_read_timeout(Some(std::time::Duration::new(1, 0)))?;
        let reader = BufReader::new(stream.try_clone()?);

        Ok(RotCtldClient {
            reader,
            writer: stream,
        })
    }

    pub fn position(&mut self) -> Result<(f64, f64)> {
        write!(&self.writer, "p\n")?;

        let mut azimuth = String::new();
        let mut elevation = String::new();
        self.reader.read_line(&mut azimuth)?;
        if azimuth.starts_with("RPRT") {
            azimuth.clear();
            azimuth.push_str("-1");
            elevation.push_str("-1");
        } else {
            self.reader.read_line(&mut elevation)?;
        }
        let azimuth = azimuth.trim().parse()?;
        let elevation = elevation.trim().parse()?;

        Ok((azimuth, elevation))
    }
}
