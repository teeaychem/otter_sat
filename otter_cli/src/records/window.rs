use crate::window::ContextWindow;

use otter_lib::dispatch::{
    library::{
        comment::{self},
        report::{self},
        stat::{self},
    },
    Dispatch,
};

pub fn window_writer<'w>(window: &'w mut ContextWindow) -> Box<dyn FnMut(&Dispatch) + 'w> {
    let handler = |dispatch: &Dispatch| match &dispatch {
        Dispatch::Comment(the_comment) => {
            //
            match the_comment {
                comment::Comment::Solve(solve_comment) => {
                    window.location.1 -= 1;
                    println!("c {}", solve_comment)
                }
            }
        }

        Dispatch::Report(the_report) => {
            //
            match the_report {
                report::Report::Solve(report) => {
                    println!("s {}", report.to_string().to_uppercase())
                }

                report::Report::Finish => window.flush(),
                _ => {}
            }
        }

        Dispatch::Stats(the_stat) => {
            //
            use crate::window::WindowItem;
            match the_stat {
                //
                stat::Stat::Iterations(i) => {
                    window.iterations = *i;
                    window.update_item(WindowItem::Iterations, i)
                }

                stat::Stat::Chosen(c) => window.update_item(WindowItem::Chosen, c),

                stat::Stat::Conflicts(c) => {
                    window.confclits = *c;
                    window.update_item(WindowItem::Conflicts, c)
                }

                stat::Stat::Time(t) => window.update_item(WindowItem::Time, format!("{:.2?}", t)),
            }

            let ratio = window.confclits as f64 / window.iterations as f64;
            window.update_item(WindowItem::Ratio, ratio);
            window.flush();
        }
        Dispatch::Delta(_) => {}
    };
    Box::new(handler)
}
