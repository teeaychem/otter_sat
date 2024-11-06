use otter_lib::{
    config::Config,
    io::files::{default_on_dir, formula_report},
    types::gen::Report,
};

use super::*;
fn dimacs_path() -> PathBuf {
    satlib_collection("DIMACS")
}

#[test]
fn aim() {
    let mut satisfiable = 0;
    let mut unsatisfiable = 0;

    let aim_path = satlib_collection("DIMACS").join(Path::new("AIM"));
    let formulas = fs::read_dir(aim_path).unwrap_or_else(|_| panic!("formulas missing"));

    for formula in formulas.flatten() {
        let formula_path = formula.path();

        let formula_check = formula_path
            .extension()
            .is_some_and(|ext| ext == "cnf" || ext == "gz");

        if formula_check {
            let formula_name = formula_path.to_str().unwrap().to_owned();

            if formula_name.contains("yes") {
                assert_eq!(
                    Report::Satisfiable,
                    formula_report(formula.path(), &Config::default())
                );
                satisfiable += 1;
            }

            if formula_name.contains("no") {
                assert_eq!(
                    Report::Unsatisfiable,
                    formula_report(formula.path(), &Config::default())
                );
                unsatisfiable += 1;
            }
        }
    }
    assert_eq!(satisfiable, 48);
    assert_eq!(unsatisfiable, 24);
}

mod cfa {
    use otter_lib::io::files::default_on_dir;

    use super::*;

    fn cfa_path(dir: &str) -> PathBuf {
        dimacs_path().join(Path::new("CFA").join(dir))
    }

    #[test]
    fn bf() {
        default_on_dir(cfa_path("BF"), &Config::default(), Report::Unsatisfiable);
    }

    #[test]
    fn ssa() {
        let unsatisfiable = [
            "ssa0432-003.cnf.gz",
            "ssa2670-130.cnf.gz",
            "ssa2670-141.cnf.gz",
            "ssa6288-047.cnf.gz",
        ];

        let mut unsat_count = 0;
        for formula in unsatisfiable {
            assert_eq!(
                Report::Unsatisfiable,
                formula_report(cfa_path("SSA").join(formula), &Config::default())
            );
            unsat_count += 1;
        }
        assert_eq!(unsat_count, unsatisfiable.len());

        let satisfiable = [
            "ssa7552-038.cnf.gz",
            "ssa7552-158.cnf.gz",
            "ssa7552-159.cnf.gz",
            "ssa7552-160.cnf.gz",
        ];
        let mut sat_count = 0;
        for formula in satisfiable {
            assert_eq!(
                Report::Satisfiable,
                formula_report(cfa_path("SSA").join(formula), &Config::default())
            );
            sat_count += 1;
        }
        assert_eq!(sat_count, satisfiable.len());
    }
}

#[test]
fn dubois() {
    default_on_dir(
        dimacs_path().join("DUBOIS"),
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
            formula_report(
                dimacs_path().join("GCP").join("g125.17.cnf.gz"),
                &Config::default()
            )
        );
    }

    #[test]
    #[ignore = "expensive"]
    fn one_two_five_one_eight() {
        assert_eq!(
            Report::Satisfiable,
            formula_report(
                dimacs_path().join("GCP").join("g125.18.cnf.gz"),
                &Config::default()
            )
        );
    }

    #[test]
    #[ignore = "expensive"]
    fn two_five_zero_one_five() {
        assert_eq!(
            Report::Satisfiable,
            formula_report(
                dimacs_path().join("GCP").join("g250.15.cnf.gz"),
                &Config::default()
            )
        );
    }

    #[test]
    #[ignore = "expensive"]
    fn two_five_zero_two_nine() {
        assert_eq!(
            Report::Satisfiable,
            formula_report(
                dimacs_path().join("GCP").join("g250.29.cnf.gz"),
                &Config::default()
            )
        );
    }
}

#[test]
fn hanoi() {
    default_on_dir(
        dimacs_path().join("HANOI"),
        &Config::default(),
        Report::Satisfiable,
    );
}

#[test]
fn inductive_inference() {
    default_on_dir(
        dimacs_path().join("II"),
        &Config::default(),
        Report::Satisfiable,
    );
}

#[test]
fn jnh() {
    let satisfiable = [
        "jnh1.cnf.gz",
        "jnh7.cnf.gz",
        "jnh12.cnf.gz",
        "jnh17.cnf.gz",
        "jnh201.cnf.gz",
        "jnh204.cnf.gz",
        "jnh205.cnf.gz",
        "jnh207.cnf.gz",
        "jnh209.cnf.gz",
        "jnh210.cnf.gz",
        "jnh212.cnf.gz",
        "jnh213.cnf.gz",
        "jnh217.cnf.gz",
        "jnh218.cnf.gz",
        "jnh220.cnf.gz",
        "jnh301.cnf.gz",
        "jnh212.cnf.gz",
    ];

    let mut sat_count = 0;
    let mut unsat_count = 0;

    let aim_path = satlib_collection("DIMACS").join(Path::new("JNH"));
    let formulas = fs::read_dir(aim_path).unwrap_or_else(|_| panic!("formulas missing"));

    for formula in formulas.flatten() {
        let formula_path = formula.path();

        let formula_check = formula_path
            .extension()
            .is_some_and(|ext| ext == "cnf" || ext == "gz");

        if formula_check {
            let file = formula
                .path()
                .as_path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned();

            if satisfiable.contains(&file.as_str()) {
                assert_eq!(
                    Report::Satisfiable,
                    formula_report(formula.path(), &Config::default())
                );
                sat_count += 1;
            } else {
                assert_eq!(
                    Report::Unsatisfiable,
                    formula_report(formula.path(), &Config::default())
                );
                unsat_count += 1;
            }
        }
    }
    assert_eq!(sat_count, 16);
    assert_eq!(unsat_count, 34);
}

#[test]
#[ignore = "expensive"]
fn lran() {
    default_on_dir(
        dimacs_path().join("LRAN"),
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
            formulas.push(format!("par8-{index}.cnf.gz"));
        }

        let mut ok_count = 0;
        for formula in &formulas {
            assert_eq!(
                Report::Satisfiable,
                formula_report(
                    dimacs_path().join("PARITY").join(formula),
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
            formulas.push(format!("par16-{index}.cnf.gz"));
        }

        let mut ok_count = 0;
        for formula in &formulas {
            assert_eq!(
                Report::Satisfiable,
                formula_report(
                    dimacs_path().join("PARITY").join(formula),
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
            formulas.push(format!("par32-{index}.cnf.gz"));
        }

        let mut ok_count = 0;
        for formula in &formulas {
            assert_eq!(
                Report::Satisfiable,
                formula_report(
                    dimacs_path().join("PARITY").join(formula),
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
        let formulas = ["hole6.cnf.gz", "hole7.cnf.gz", "hole8.cnf.gz"];

        let mut ok_count = 0;
        for formula in formulas {
            assert_eq!(
                Report::Unsatisfiable,
                formula_report(
                    dimacs_path().join("PHOLE").join(formula),
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
        let formulas = ["hole9.cnf.gz", "hole10.cnf.gz"];

        let mut ok_count = 0;
        for formula in formulas {
            assert_eq!(
                Report::Unsatisfiable,
                formula_report(
                    dimacs_path().join("PHOLE").join(formula),
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
    default_on_dir(
        dimacs_path().join("PRET"),
        &Config::default(),
        Report::Unsatisfiable,
    );
}
