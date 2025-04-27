mod phole {
    use otter_sat::{config::Config, reports::Report};
    use otter_tests::general::{cnf_lib_subdir, silent_formula_report};

    #[test]
    fn normal() {
        let formulas = ["hole6.cnf.xz", "hole7.cnf.xz", "hole8.cnf.xz"];

        let mut ok_count = 0;
        for formula in formulas {
            assert_eq!(
                Report::Unsatisfiable,
                silent_formula_report(
                    cnf_lib_subdir(vec!["SATLIB", "DIMACS", "PHOLE"]).join(formula),
                    &Config::default()
                )
            );
            ok_count += 1;
        }
        assert_eq!(ok_count, formulas.len());
    }

    #[test]
    fn tough_nine() {
        assert_eq!(
            Report::Unsatisfiable,
            silent_formula_report(
                cnf_lib_subdir(vec!["SATLIB", "DIMACS", "PHOLE", "hole9.cnf.xz"]),
                &Config::default()
            )
        );
    }

    #[test]
    #[ignore = "expensive"]
    fn tough_ten() {
        assert_eq!(
            Report::Unsatisfiable,
            silent_formula_report(
                cnf_lib_subdir(vec!["SATLIB", "DIMACS", "PHOLE", "hole10.cnf.xz"]),
                &Config::default()
            )
        );
    }
}
