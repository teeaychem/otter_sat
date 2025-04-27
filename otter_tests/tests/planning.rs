mod planning {

    use otter_sat::{config::Config, reports::Report};
    use otter_tests::general::{cnf_lib_subdir, silent_on_directory};

    #[test]
    fn logistics() {
        silent_on_directory(
            cnf_lib_subdir(vec!["SATLIB", "planning", "logistics"]),
            &Config::default(),
            Report::Satisfiable,
        );
    }

    #[test]
    fn blocksworld() {
        silent_on_directory(
            cnf_lib_subdir(vec!["SATLIB", "planning", "blocksworld"]),
            &Config::default(),
            Report::Satisfiable,
        );
    }
}
