mod lran {
    use otter_sat::{config::Config, reports::Report};
    use otter_tests::general::{cnf_lib_subdir, silent_formula_report};

    #[test]
    #[ignore = "expensive"]
    fn lran600() {
        let report = silent_formula_report(
            cnf_lib_subdir(vec!["SATLIB", "DIMACS", "LRAN", "f600.cnf.xz"]),
            &Config::default(),
        );
        assert_eq!(report, Report::Satisfiable);
    }

    #[test]
    #[ignore = "expensive"]
    fn lran1000() {
        let report = silent_formula_report(
            cnf_lib_subdir(vec!["SATLIB", "DIMACS", "LRAN", "f1000.cnf.xz"]),
            &Config::default(),
        );
        assert_eq!(report, Report::Satisfiable);
    }

    #[test]
    #[ignore = "expensive"]
    fn lran2000() {
        let report = silent_formula_report(
            cnf_lib_subdir(vec!["SATLIB", "DIMACS", "LRAN", "f2000.cnf.xz"]),
            &Config::default(),
        );
        assert_eq!(report, Report::Satisfiable);
    }
}
