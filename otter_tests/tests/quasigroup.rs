mod quasigroup {
    use otter_sat::{config::Config, reports::Report};
    use otter_tests::general::{cnf_lib_subdir, silent_formula_report};

    #[test]
    fn sat() {
        let collection_path = cnf_lib_subdir(vec!["SATLIB", "quasigroup"]);
        let satisfiable_formulas = [
            "qg1-07.cnf.xz",
            "qg1-08.cnf.xz",
            "qg2-07.cnf.xz",
            "qg2-08.cnf.xz",
            "qg3-08.cnf.xz",
            "qg4-09.cnf.xz",
            "qg5-11.cnf.xz",
            "qg6-09.cnf.xz",
            "qg7-09.cnf.xz",
            "qg7-13.cnf.xz",
        ];
        let mut sat_count = 0;
        for formula in &satisfiable_formulas {
            assert_eq!(
                Report::Satisfiable,
                silent_formula_report(collection_path.join(formula), &Config::default())
            );
            sat_count += 1;
        }
        assert_eq!(sat_count, satisfiable_formulas.len());
    }

    #[test]
    fn unsat_3() {
        let collection_path = cnf_lib_subdir(vec!["SATLIB", "quasigroup"]);
        let unsatisfiable_formulas = ["qg3-09.cnf.xz"];
        let mut unsat_count = 0;
        for formula in &unsatisfiable_formulas {
            assert_eq!(
                Report::Unsatisfiable,
                silent_formula_report(collection_path.join(formula), &Config::default())
            );
            unsat_count += 1;
        }
        assert_eq!(unsat_count, unsatisfiable_formulas.len());
    }

    #[test]
    fn unsat_4() {
        let collection_path = cnf_lib_subdir(vec!["SATLIB", "quasigroup"]);
        let unsatisfiable_formulas = ["qg4-08.cnf.xz"];
        let mut unsat_count = 0;
        for formula in &unsatisfiable_formulas {
            assert_eq!(
                Report::Unsatisfiable,
                silent_formula_report(collection_path.join(formula), &Config::default())
            );
            unsat_count += 1;
        }
        assert_eq!(unsat_count, unsatisfiable_formulas.len());
    }

    #[test]
    fn unsat_5() {
        let collection_path = cnf_lib_subdir(vec!["SATLIB", "quasigroup"]);
        let unsatisfiable_formulas = [
            "qg5-09.cnf.xz",
            "qg5-10.cnf.xz",
            "qg5-12.cnf.xz",
            "qg5-13.cnf.xz",
        ];
        let mut unsat_count = 0;
        for formula in &unsatisfiable_formulas {
            assert_eq!(
                Report::Unsatisfiable,
                silent_formula_report(collection_path.join(formula), &Config::default())
            );
            unsat_count += 1;
        }
        assert_eq!(unsat_count, unsatisfiable_formulas.len());
    }

    #[test]
    fn unsat_6() {
        let collection_path = cnf_lib_subdir(vec!["SATLIB", "quasigroup"]);
        let unsatisfiable_formulas = ["qg6-10.cnf.xz", "qg6-11.cnf.xz", "qg6-12.cnf.xz"];
        let mut unsat_count = 0;
        for formula in &unsatisfiable_formulas {
            assert_eq!(
                Report::Unsatisfiable,
                silent_formula_report(collection_path.join(formula), &Config::default())
            );
            unsat_count += 1;
        }
        assert_eq!(unsat_count, unsatisfiable_formulas.len());
    }

    #[test]
    fn unsat_7() {
        let collection_path = cnf_lib_subdir(vec!["SATLIB", "quasigroup"]);
        let unsatisfiable_formulas = ["qg7-10.cnf.xz", "qg7-11.cnf.xz", "qg7-12.cnf.xz"];
        let mut unsat_count = 0;
        for formula in &unsatisfiable_formulas {
            assert_eq!(
                Report::Unsatisfiable,
                silent_formula_report(collection_path.join(formula), &Config::default())
            );
            unsat_count += 1;
        }
        assert_eq!(unsat_count, unsatisfiable_formulas.len());
    }
}
