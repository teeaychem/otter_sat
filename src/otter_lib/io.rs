use std::fmt::Display;
use std::io::{stdout, Write};

use crate::context::config::Config;

use crossterm::{cursor, terminal, QueueableCommand};

pub struct ContextWindow {
    location: (u16, u16),
    column: u16,
}

#[derive(Debug, Clone, Copy)]
pub enum WindowItem {
    Iterations,
    Conflicts,
    Ratio,
    Time,
}

impl ContextWindow {
    pub fn default() -> Self {
        let location = cursor::position().expect("Unable to display stats");

        ContextWindow {
            location,
            column: 14,
        }
    }
    fn get_offset(&self, item: WindowItem) -> (u16, u16) {
        let bottom = self.location.1;
        let the_row = match item {
            WindowItem::Iterations => bottom - 4,
            WindowItem::Conflicts => bottom - 3,
            WindowItem::Ratio => bottom - 2,
            WindowItem::Time => bottom - 1,
        };
        (self.column, the_row)
    }

    pub fn update_item(&self, item: WindowItem, output: impl Display) {
        let mut stdout = stdout();
        let (x, y) = self.get_offset(item);

        stdout.queue(cursor::SavePosition).unwrap();
        stdout.queue(cursor::MoveTo(x, y)).unwrap();
        stdout
            .queue(terminal::Clear(terminal::ClearType::UntilNewLine))
            .unwrap();
        match item {
            WindowItem::Ratio => stdout.write_all(format!("{output:.4}").as_bytes()).unwrap(),
            _ => stdout.write_all(format!("{output}").as_bytes()).unwrap(),
        }
        stdout.queue(cursor::RestorePosition).unwrap();
    }

    pub fn flush(&self) {
        stdout().flush().unwrap();
    }

    pub fn update_position(&mut self) {
        self.location = cursor::position().expect("Unable to display stats");
    }

    pub fn draw_window(&mut self, config: &Config) {
        println!("c 🦦");
        println!("c CHOICE POLARITY LEAN {}", config.polarity_lean);
        if let Some(limit) = config.time_limit {
            println!("c TIME LIMIT: {:.2?}", limit);
        }
        println!("c ITERATIONS");
        println!("c CONFLCITS");
        println!("c RATIO");
        println!("c TIME");

        self.update_position();
    }

}
