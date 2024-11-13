use std::path::PathBuf;

use crossbeam::channel::Sender;

use crate::context::builder::BuildErr;
use crate::dispatch::Dispatch;
use crate::{config::Config, context::Context};

pub fn context_from_path(
    path: PathBuf,
    config: Config,
    sender: Sender<Dispatch>,
) -> Result<Context, BuildErr> {
    let the_path = PathBuf::from(&path);
    let unique_config = config.clone();
    let mut the_context = Context::from_config(unique_config, sender.clone());

    the_context.load_dimacs_file(the_path);

    Ok(the_context)
}
