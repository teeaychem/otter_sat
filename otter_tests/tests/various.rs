use otter_sat::{config::Config, reports::Report};
use otter_tests::general::{cnf_lib_subdir, silent_formula_report, silent_on_directory};

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

#[test]
fn pret() {
    silent_on_directory(
        cnf_lib_subdir(vec!["SATLIB", "DIMACS", "PRET"]),
        &Config::default(),
        Report::Unsatisfiable,
    );
}

#[test]
fn all_interval_series() {
    let pass = silent_on_directory(
        cnf_lib_subdir(vec!["SATLIB", "ais"]),
        &Config::default(),
        Report::Satisfiable,
    );
    assert_eq!(pass, 4);
}

#[test]
fn bounded_model_check() {
    silent_on_directory(
        cnf_lib_subdir(vec!["SATLIB", "bmc"]),
        &Config::default(),
        Report::Satisfiable,
    );
}

#[test]
fn beijing() {
    let collection_path = cnf_lib_subdir(vec!["SATLIB", "beijing"]);

    let satisfiable_formulas = [
        "2bitcomp_5.cnf.xz",
        "2bitmax_6.cnf.xz",
        "3bitadd_31.cnf.xz",
        "3bitadd_32.cnf.xz",
        "3blocks.cnf.xz",
        "4blocks.cnf.xz",
        "4blocksb.cnf.xz",
        "e0ddr2-10-by-5-1.cnf.xz",
        "e0ddr2-10-by-5-4.cnf.xz",
        "enddr2-10-by-5-1.cnf.xz",
        "enddr2-10-by-5-8.cnf.xz",
        "ewddr2-10-by-5-1.cnf.xz",
        "ewddr2-10-by-5-8.cnf.xz",
    ];

    let mut count = 0;
    for formula in satisfiable_formulas {
        assert_eq!(
            Report::Satisfiable,
            silent_formula_report(collection_path.join(formula), &Config::default())
        );
        count += 1;
    }
    assert_eq!(count, satisfiable_formulas.len());
}
