mod parity {
    use otter_sat::{config::Config, reports::Report};
    use otter_tests::general::{cnf_lib_subdir, silent_formula_report};

    #[test]
    fn eight() {
        let mut formulas = Vec::new();
        for index in 1..6 {
            formulas.push(format!("par8-{index}.cnf.xz"));
        }

        let mut ok_count = 0;
        for formula in &formulas {
            assert_eq!(
                Report::Satisfiable,
                silent_formula_report(
                    cnf_lib_subdir(vec!["SATLIB", "DIMACS", "PARITY"]).join(formula),
                    &Config::default()
                )
            );
            ok_count += 1;
        }
        assert_eq!(ok_count, formulas.len());
    }

    #[test]
    fn sixteen() {
        let mut formulas = Vec::new();
        for index in 1..6 {
            formulas.push(format!("par16-{index}.cnf.xz"));
        }

        let mut ok_count = 0;
        for formula in &formulas {
            assert_eq!(
                Report::Satisfiable,
                silent_formula_report(
                    cnf_lib_subdir(vec!["SATLIB", "DIMACS", "PARITY"]).join(formula),
                    &Config::default()
                )
            );
            ok_count += 1;
        }
        assert_eq!(ok_count, formulas.len());
    }

    #[test]
    #[ignore = "expensive"]
    fn thirty_two() {
        let mut formulas = Vec::new();
        for index in 1..6 {
            formulas.push(format!("par32-{index}.cnf.xz"));
        }

        let mut ok_count = 0;
        for formula in &formulas {
            assert_eq!(
                Report::Satisfiable,
                silent_formula_report(
                    cnf_lib_subdir(vec!["SATLIB", "DIMACS", "PARITY"]).join(formula),
                    &Config::default()
                )
            );
            ok_count += 1;
        }
        assert_eq!(ok_count, formulas.len());
    }
}
