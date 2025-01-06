use std::fmt::Display;
use std::io::{stdout, Write};

use otter_sat::config::Config;

use termion::{cursor::DetectCursorPos, raw::IntoRawMode};

pub struct ContextWindow {
    pub location: (u16, u16),
    column: u16,
    pub iterations: usize,
    pub confclits: usize,
}

#[derive(Clone, Copy)]
pub enum WindowItem {
    Iterations,
    Conflicts,
    Chosen,
    Ratio,
    Time,
}

impl Default for ContextWindow {
    fn default() -> Self {
        let mut stdout = std::io::stdout().into_raw_mode().unwrap();
        let location = stdout.cursor_pos().unwrap();

        ContextWindow {
            location,
            column: 14,
            iterations: 0,
            confclits: 0,
        }
    }
}

impl ContextWindow {
    fn get_offset(&self, item: WindowItem) -> (u16, u16) {
        let bottom = self.location.1;
        let the_row = match item {
            WindowItem::Iterations => bottom - 5,
            WindowItem::Conflicts => bottom - 4,
            WindowItem::Ratio => bottom - 3,
            WindowItem::Chosen => bottom - 2,
            WindowItem::Time => bottom - 1,
        };
        (self.column, the_row)
    }

    #[allow(unused_must_use)]
    pub fn update_item(&self, item: WindowItem, output: impl Display) {
        let mut stdout = stdout();
        let (x, y) = self.get_offset(item);

        write!(
            stdout,
            "{}{}",
            termion::cursor::Goto(x, y),
            termion::clear::UntilNewline
        );

        match item {
            WindowItem::Ratio => writeln!(stdout, "{output:.4}"),
            _ => writeln!(stdout, "{output}"),
        };

        write!(stdout, "{}", termion::cursor::Goto(0, self.location.1));
    }

    pub fn update_position(&mut self) {
        // self.location = cursor::position().expect("Unable to display stats");
        let mut stdout = std::io::stdout().into_raw_mode().unwrap();
        self.location = stdout.cursor_pos().unwrap();
    }

    #[allow(unused_must_use)]
    pub fn draw_window(&mut self, config: &Config) {
        let mut stdout = stdout();
        writeln!(stdout, "c 🦦");
        writeln!(stdout, "c CHOICE POLARITY LEAN {}", config.polarity_lean);
        if let Some(limit) = config.time_limit {
            writeln!(stdout, "c TIME LIMIT: {:.2?}", limit);
        }
        writeln!(stdout, "c ITERATIONS");
        writeln!(stdout, "c CONFLCITS");
        writeln!(stdout, "c C/I RATIO");
        writeln!(stdout, "c CHOICES");
        writeln!(stdout, "c TIME");

        self.update_position();
    }
}
