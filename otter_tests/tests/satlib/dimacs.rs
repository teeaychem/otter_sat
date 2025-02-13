use otter_sat::{config::Config, reports::Report};
use otter_tests::{cnf_lib_subdir, silent_formula_report, silent_on_directory};

#[test]
fn aim() {
    let mut satisfiable = 0;
    let mut unsatisfiable = 0;

    let aim_path = cnf_lib_subdir(vec!["SATLIB", "DIMACS", "AIM"]);
    let formulas = match std::fs::read_dir(aim_path) {
        Ok(formulas) => formulas,
        Err(_) => panic!("formulas missing"),
    };

    for formula in formulas.flatten() {
        let formula_path = formula.path();

        let formula_check = formula_path
            .extension()
            .is_some_and(|ext| ext == "cnf" || ext == "xz");

        if formula_check {
            if let Some(formula_name) = formula_path.to_str() {
                if formula_name.contains("yes") {
                    assert_eq!(
                        Report::Satisfiable,
                        silent_formula_report(formula.path(), &Config::default())
                    );
                    satisfiable += 1;
                }

                if formula_name.contains("no") {
                    assert_eq!(
                        Report::Unsatisfiable,
                        silent_formula_report(formula.path(), &Config::default())
                    );
                    unsatisfiable += 1;
                }
            }
        }
    }
    assert_eq!(satisfiable, 48);
    assert_eq!(unsatisfiable, 24);
}

mod cfa {
    use super::*;

    // fn cfa_path(dir: &str) -> PathBuf {
    //     dimacs_path().join(Path::new("CFA").join(dir))
    // }

    #[test]
    fn bf() {
        silent_on_directory(
            cnf_lib_subdir(vec!["SATLIB", "DIMACS", "CFA", "BF"]),
            &Config::default(),
            Report::Unsatisfiable,
        );
    }

    #[test]
    fn ssa() {
        let unsatisfiable = [
            "ssa0432-003.cnf.xz",
            "ssa2670-130.cnf.xz",
            "ssa2670-141.cnf.xz",
            "ssa6288-047.cnf.xz",
        ];

        let mut unsat_count = 0;
        for formula in unsatisfiable {
            assert_eq!(
                Report::Unsatisfiable,
                silent_formula_report(
                    cnf_lib_subdir(vec!["SATLIB", "DIMACS", "CFA", "SSA"]).join(formula),
                    &Config::default()
                )
            );
            unsat_count += 1;
        }
        assert_eq!(unsat_count, unsatisfiable.len());

        let satisfiable = [
            "ssa7552-038.cnf.xz",
            "ssa7552-158.cnf.xz",
            "ssa7552-159.cnf.xz",
            "ssa7552-160.cnf.xz",
        ];
        let mut sat_count = 0;
        for formula in satisfiable {
            assert_eq!(
                Report::Satisfiable,
                silent_formula_report(
                    cnf_lib_subdir(vec!["SATLIB", "DIMACS", "CFA", "SSA"]).join(formula),
                    &Config::default()
                )
            );
            sat_count += 1;
        }
        assert_eq!(sat_count, satisfiable.len());
    }
}

#[test]
fn dubois() {
    silent_on_directory(
        cnf_lib_subdir(vec!["SATLIB", "DIMACS", "DUBOIS"]),
        &Config::default(),
        Report::Unsatisfiable,
    );
}

mod graph_colouring {
    use super::*;

    #[test]
    #[ignore = "expensive"]
    fn one_two_five_one_seven() {
        assert_eq!(
            Report::Satisfiable,
            silent_formula_report(
                cnf_lib_subdir(vec!["SATLIB", "DIMACS", "GCP", "g125.17.cnf.xz"]),
                &Config::default()
            )
        );
    }

    #[test]
    #[ignore = "expensive"]
    fn one_two_five_one_eight() {
        assert_eq!(
            Report::Satisfiable,
            silent_formula_report(
                cnf_lib_subdir(vec!["SATLIB", "DIMACS", "GCP", "g125.18.cnf.xz"]),
                &Config::default()
            )
        );
    }

    #[test]
    #[ignore = "expensive"]
    fn two_five_zero_one_five() {
        assert_eq!(
            Report::Satisfiable,
            silent_formula_report(
                cnf_lib_subdir(vec!["SATLIB", "DIMACS", "GCP", "g250.15.cnf.xz"]),
                &Config::default()
            )
        );
    }

    #[test]
    #[ignore = "expensive"]
    fn two_five_zero_two_nine() {
        assert_eq!(
            Report::Satisfiable,
            silent_formula_report(
                cnf_lib_subdir(vec!["SATLIB", "DIMACS", "GCP", "g250.29.cnf.xz"]),
                &Config::default()
            )
        );
    }
}

#[test]
fn hanoi() {
    silent_on_directory(
        cnf_lib_subdir(vec!["SATLIB", "DIMACS", "HANOI"]),
        &Config::default(),
        Report::Satisfiable,
    );
}

#[test]
fn inductive_inference() {
    silent_on_directory(
        cnf_lib_subdir(vec!["SATLIB", "DIMACS", "II"]),
        &Config::default(),
        Report::Satisfiable,
    );
}

#[test]
fn jnh() {
    use std::ffi::OsStr;

    let satisfiable = [
        OsStr::new("jnh1.cnf.xz"),
        OsStr::new("jnh7.cnf.xz"),
        OsStr::new("jnh12.cnf.xz"),
        OsStr::new("jnh17.cnf.xz"),
        OsStr::new("jnh201.cnf.xz"),
        OsStr::new("jnh204.cnf.xz"),
        OsStr::new("jnh205.cnf.xz"),
        OsStr::new("jnh207.cnf.xz"),
        OsStr::new("jnh209.cnf.xz"),
        OsStr::new("jnh210.cnf.xz"),
        OsStr::new("jnh212.cnf.xz"),
        OsStr::new("jnh213.cnf.xz"),
        OsStr::new("jnh217.cnf.xz"),
        OsStr::new("jnh218.cnf.xz"),
        OsStr::new("jnh220.cnf.xz"),
        OsStr::new("jnh301.cnf.xz"),
        OsStr::new("jnh212.cnf.xz"),
    ];

    let mut sat_count = 0;
    let mut unsat_count = 0;

    let aim_path = cnf_lib_subdir(vec!["SATLIB", "DIMACS", "JNH"]);
    let formulas = match std::fs::read_dir(aim_path) {
        Ok(formulas) => formulas,
        Err(_) => panic!("formulas missing"),
    };

    for formula in formulas.flatten() {
        match &formula.path().file_name() {
            None => {}
            Some(filename) => {
                let formula_path = formula.path();

                let formula_check = formula_path
                    .extension()
                    .is_some_and(|ext| ext == "cnf" || ext == "xz");

                if formula_check {
                    if satisfiable.contains(filename) {
                        assert_eq!(
                            Report::Satisfiable,
                            silent_formula_report(formula.path(), &Config::default())
                        );
                        sat_count += 1;
                    } else {
                        assert_eq!(
                            Report::Unsatisfiable,
                            silent_formula_report(formula.path(), &Config::default())
                        );
                        unsat_count += 1;
                    }
                }
            }
        }
    }
    assert_eq!(sat_count, 16);
    assert_eq!(unsat_count, 34);
}

// #[test]
// #[ignore = "expensive"]
// fn lran600() {
//     let report = silent_formula_report(
//         cnf_lib_subdir(vec!["SATLIB", "DIMACS", "LRAN"]).join("f600.cnf.xz"),
//         &Config::default(),
//     );
//     assert_eq!(report, Report::Satisfiable);
// }

// #[test]
// #[ignore = "expensive"]
// fn lran1000() {
//     let report = silent_formula_report(
//         cnf_lib_subdir(vec!["SATLIB", "DIMACS", "LRAN"]).join("f1000.cnf.xz"),
//         &Config::default(),
//     );
//     assert_eq!(report, Report::Satisfiable);
// }

// #[test]
// #[ignore = "expensive"]
// fn lran2000() {
//     let report = silent_formula_report(
//         cnf_lib_subdir(vec!["SATLIB", "DIMACS", "LRAN"]).join("f2000.cnf.xz"),
//         &Config::default(),
//     );
//     assert_eq!(report, Report::Satisfiable);
// }

#[test]
#[ignore = "expensive"]
fn lran() {
    silent_on_directory(
        cnf_lib_subdir(vec!["SATLIB", "DIMACS", "LRAN"]),
        &Config::default(),
        Report::Satisfiable,
    );
}

mod partiy {
    use super::*;

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

mod phole {
    use super::*;

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
    #[ignore = "expensive"]
    fn tough() {
        let formulas = ["hole9.cnf.xz", "hole10.cnf.xz"];

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
}

#[test]
fn pret() {
    silent_on_directory(
        cnf_lib_subdir(vec!["SATLIB", "DIMACS", "PRET"]),
        &Config::default(),
        Report::Unsatisfiable,
    );
}
