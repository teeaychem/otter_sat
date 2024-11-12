use std::path::PathBuf;

use clap::ArgMatches;

use crate::cli::config::ConfigIO;

impl ConfigIO {
    pub fn from_args(args: &ArgMatches) -> Self {
        let mut the_config = ConfigIO::default();

        if let Ok(Some(value)) = args.try_get_one::<bool>("core") {
            the_config.show_core = *value
        };
        if let Ok(Some(value)) = args.try_get_one::<bool>("stats") {
            the_config.show_stats = *value;
        };
        if let Ok(Some(value)) = args.try_get_one::<bool>("valuation") {
            the_config.show_valuation = *value
        };

        if let Ok(Some(detail)) = args.try_get_one::<u8>("detail") {
            the_config.detail = *detail
        };

        match args.get_many::<PathBuf>("paths") {
            None => {
                println!("c No files");
                std::process::exit(1);
            }
            Some(paths) => the_config.files = paths.cloned().collect(),
        }

        if let Ok(Some(value)) = args.try_get_one::<bool>("FRAT") {
            the_config.frat = *value
        };

        if let Ok(Some(path)) = args.try_get_one::<PathBuf>("FRAT_path") {
            the_config.frat = true;
            the_config.frat_path = Some(path.to_owned());
        };

        if the_config.frat && the_config.frat_path.is_none() {
            the_config.frat_path = Some(frat_default(&the_config));
        }

        the_config
    }
}

fn frat_default(config_io: &ConfigIO) -> PathBuf {
    let the_path = config_io.files.first().unwrap().clone();
    let frat_file = format!("{}.frat", the_path.file_name().unwrap().to_str().unwrap());
    let mut frat_path = std::env::current_dir().unwrap();
    frat_path.push(frat_file);

    // std::process::exit(2);
    // frat_path.push_str(".frat");
    PathBuf::from(&frat_path)
}
