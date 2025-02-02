use termcolor::{Color, ColorChoice, StandardStream, WriteColor, ColorSpec};
use std::io::Write;

pub struct TermColors;

impl TermColors {
    pub fn new_stream() -> StandardStream {
        StandardStream::stdout(ColorChoice::Always)
    }

    pub fn write_header(
        &self,
        stream: &mut StandardStream,
        label: &str,
        color: Color,
    ) -> Result<(), std::io::Error> {
        stream.set_color(
            termcolor::ColorSpec::new()
                .set_fg(Some(color))
                .set_bold(true)
        )?;
        write!(stream, "[{}]", label)?;
        stream.reset()?;
        Ok(())
    }

    pub fn write_value(
        &self,
        stream: &mut StandardStream,
        value: &str,
    ) -> Result<(), std::io::Error> {
        stream.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
        writeln!(stream, " {}", value)
    }
}

pub fn format_address(address: usize) -> String {
    format!("0x{:016X}", address)
}

pub fn show_credits(stream: &mut StandardStream) -> Result<(), std::io::Error> {
    let colors = TermColors;
    
    stream.set_color(
        ColorSpec::new()
            .set_fg(Some(Color::Magenta))
            .set_bold(true)
            .set_intense(true)
    )?;
    writeln!(stream, "\n╔═════════════════════════════════════════╗")?;
    writeln!(stream, "║    Bufferization | Neoworth Development   ║")?;
    writeln!(stream, "╚═══════════════════════════════════════════╝")?;
    stream.reset()?;
    Ok(())
}
