use chrono::{DateTime, Utc};
use futures::{Stream, StreamExt};
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
    task::Poll,
    time::Duration,
};

#[derive(Clone, Copy)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Purple,
    Cyan,
    White,
    Fixed(u8),
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Color::Black => f.write_str("40"),
            Color::Red => f.write_str("41"),
            Color::Green => f.write_str("42"),
            Color::Yellow => f.write_str("43"),
            Color::Blue => f.write_str("44"),
            Color::Purple => f.write_str("45"),
            Color::Cyan => f.write_str("46"),
            Color::White => f.write_str("47"),
            Color::Fixed(num) => write!(f, "48;5;{}", num),
        }
    }
}

impl FromStr for Color {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let color = match s.trim().to_ascii_lowercase().as_str() {
            "black" => Color::Black,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "purple" => Color::Purple,
            "cyan" => Color::Cyan,
            "white" => Color::White,
            s if s.chars().all(|c| c.is_ascii_digit()) => {
                Color::Fixed(s.parse::<u8>().map_err(|e| e.to_string())?)
            }
            _ => return Err("Unknown color".to_owned()),
        };
        Ok(color)
    }
}

enum Ansi {
    ClearScreen,
    Goto(u16, u16),
    SetColor(Color),
    ResetStyle,
}

impl Display for Ansi {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Ansi::ClearScreen => f.write_str("\u{001b}c"),
            Ansi::Goto(x, y) => write!(f, "\x1B[{};{}H", y + 1, x + 1),
            Ansi::SetColor(c) => write!(f, "\x1B[{}m", c),
            Ansi::ResetStyle => f.write_str("\x1B[0m"),
        }
    }
}

const NUMBER_WIDTH: u16 = 6;
const NUMBER_HEIGHT: u16 = 5;
const NUMBER_TABLE: [[bool; NUMBER_WIDTH as usize / 2 * NUMBER_HEIGHT as usize]; 10] = {
    const X: bool = true;
    const O: bool = false;
    [
        [X, X, X, X, O, X, X, O, X, X, O, X, X, X, X], // 0
        [O, O, X, O, O, X, O, O, X, O, O, X, O, O, X], // 1
        [X, X, X, O, O, X, X, X, X, X, O, O, X, X, X], // 2
        [X, X, X, O, O, X, X, X, X, O, O, X, X, X, X], // 3
        [X, O, X, X, O, X, X, X, X, O, O, X, O, O, X], // 4
        [X, X, X, X, O, O, X, X, X, O, O, X, X, X, X], // 5
        [X, X, X, X, O, O, X, X, X, X, O, X, X, X, X], // 6
        [X, X, X, O, O, X, O, O, X, O, O, X, O, O, X], // 7
        [X, X, X, X, O, X, X, X, X, X, O, X, X, X, X], // 8
        [X, X, X, X, O, X, X, X, X, O, O, X, X, X, X], // 9
    ]
};

fn format_number(
    f: &mut Formatter<'_>,
    number: usize,
    left: u16,
    top: u16,
    color: Color,
) -> fmt::Result {
    let table = &NUMBER_TABLE[number];

    let mut i = 0;
    let mut filled = false;

    for y in 0..NUMBER_HEIGHT {
        write!(f, "{}", Ansi::Goto(left, top + y))?;

        for _ in 0..NUMBER_WIDTH / 2 {
            if table[i] {
                if !filled {
                    write!(f, "{}", Ansi::SetColor(color))?;
                    filled = true;
                }
            } else if filled {
                write!(f, "{}", Ansi::ResetStyle)?;
                filled = false;
            }

            f.write_str("  ")?;
            i += 1;
        }
    }

    write!(f, "{}", Ansi::ResetStyle)
}

struct Clockface {
    datetime: DateTime<Utc>,
    color: Color,
}

impl Display for Clockface {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        const LEFT_MARGIN: u16 = 0;
        const TOP_MARGIN: u16 = 1;

        let mut unix_time = self.datetime.timestamp();
        let mut digits = Vec::new();
        while unix_time > 0 {
            digits.push(unix_time % 10);
            unix_time /= 10;
        }

        const STEP: u16 = NUMBER_WIDTH + 1;
        for (i, d) in digits.iter().rev().enumerate() {
            format_number(
                f,
                *d as usize,
                LEFT_MARGIN + i as u16 * STEP,
                TOP_MARGIN,
                self.color,
            )?;
        }

        let datetime_str = self
            .datetime
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        write!(
            f,
            "{}{}{}",
            Ansi::Goto(
                LEFT_MARGIN
                    + (digits.len() as u16 * STEP).saturating_sub(datetime_str.len() as u16) / 2,
                TOP_MARGIN + NUMBER_HEIGHT + 1
            ),
            datetime_str,
            Ansi::Goto(0, TOP_MARGIN + NUMBER_HEIGHT + 2)
        )
    }
}

pub fn clock_stream(color: Color) -> impl Stream<Item = String> {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    let stream = futures::stream::poll_fn(move |cx| match interval.poll_tick(cx) {
        Poll::Ready(_) => {
            let response = Clockface {
                datetime: Utc::now(),
                color,
            }
            .to_string();
            Poll::Ready(Some(response))
        }
        Poll::Pending => Poll::Pending,
    });
    futures::stream::once(async { Ansi::ClearScreen.to_string() }).chain(stream)
}
