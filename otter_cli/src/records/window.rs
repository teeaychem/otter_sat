use crate::window::ContextWindow;

use otter_lib::dispatch::{
    stat::{self},
    Dispatch,
};

pub fn window_writer<'w>(window: &'w mut ContextWindow) -> Box<dyn FnMut(&Dispatch) + 'w> {
    let handler = |dispatch: &Dispatch| match &dispatch {
        Dispatch::SolveComment(comment) => {
            window.location.1 -= 1;
            println!("c {}", comment)
        }
        Dispatch::SolveReport(report) => println!("s {}", report.to_string().to_uppercase()),
        Dispatch::Parser(msg) => {
            window.location.1 -= 1;
            println!("c {msg}")
        }
        Dispatch::Stats(stat) => {
            use crate::window::WindowItem;
            match stat {
                stat::Count::ICD(i, c, d) => {
                    window.update_item(WindowItem::Iterations, i);
                    window.update_item(WindowItem::Chosen, d);
                    window.update_item(WindowItem::Conflicts, c);
                    window.update_item(WindowItem::Ratio, *c as f64 / *i as f64);
                    window.flush();
                }

                stat::Count::Time(t) => window.update_item(WindowItem::Time, format!("{:.2?}", t)),
            }
        }
        Dispatch::Finish => window.flush(),
        Dispatch::BCP(_)
        | Dispatch::Resolution(_)
        | Dispatch::VariableDB(_)
        | Dispatch::VariableDBReport(_)
        | Dispatch::ClauseDB(_)
        | Dispatch::ClauseDBReport(_)
        | Dispatch::Level(_) => {}
    };
    Box::new(handler)
}
