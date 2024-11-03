#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::fs;
use std::path::{Path, PathBuf};

use otter_lib::{
    config::Config,
    context::{self, Context, Report},
    io::files::*,
    structures::{
        literal::{Literal, LiteralSource},
        variable::list::VariableList,
    },
};

fn cnf_path() -> PathBuf {
    Path::new(".").join("tests").join("cnf")
}

fn satlib_path() -> PathBuf {
    cnf_path().join("SATLIB")
}

fn satlib_collection(collection: &str) -> PathBuf {
    satlib_path().join(Path::new(collection))
}

mod uniform_random_3SAT {
    use super::*;
    fn unform_random_path() -> PathBuf {
        satlib_path().join("uniform_random")
    }

    #[test]
    fn uniform_random_3_20_91() {
        default_on_dir(
            unform_random_path().join("UF20.91"),
            &Config::default(),
            Report::Satisfiable,
        );
    }

    #[test]
    fn uniform_random_3_50_128() {
        default_on_split_dir(
            unform_random_path().join("UF50.218.1000"),
            &Config::default(),
        );
    }

    #[test]
    fn uniform_random_3_225_960() {
        default_on_split_dir(
            unform_random_path().join("UF225.960.100"),
            &Config::default(),
        );
    }
    #[test]
    fn uniform_random_3_250_1065() {
        default_on_split_dir(
            unform_random_path().join("UF250.1065.100"),
            &Config::default(),
        );
    }
}

mod random_3SAT_and_backbone_minimal_subinstances {
    use super::*;

    #[test]
    fn rti_k3_n100_m429() {
        default_on_dir(
            satlib_collection("RTI_k3_n100_m429"),
            &Config::default(),
            Report::Satisfiable,
        );
    }

    #[test]
    fn bms_k3_n100_m429() {
        default_on_dir(
            satlib_collection("BMS_k3_n100_m429"),
            &Config::default(),
            Report::Satisfiable,
        );
    }
}

mod planning {
    use super::*;
    #[test]
    fn logistics() {
        default_on_dir(
            satlib_collection("logistics"),
            &Config::default(),
            Report::Satisfiable,
        );
    }

    #[test]
    fn blocksworld() {
        default_on_dir(
            satlib_collection("blocksworld"),
            &Config::default(),
            Report::Satisfiable,
        );
    }
}

mod graph_colouring {
    use super::*;
    fn colouring_path() -> PathBuf {
        satlib_path().join("graph_colouring")
    }
    mod morphed {
        use super::*;
        fn morphed_path() -> PathBuf {
            colouring_path().join("morphed")
        }

        #[macro_export]
        macro_rules! morphed_test {
            ( $name:ident,  $n:literal ) => {
                #[test]
                fn $name() {
                    default_on_dir(
                        morphed_path().join(format!("SW100-8-{}", $n)),
                        &Config::default(),
                        Report::Satisfiable,
                    );
                }
            };
        }

        morphed_test!(SW100_8_0, 0);
        morphed_test!(SW100_8_1, 1);
        morphed_test!(SW100_8_2, 2);
        morphed_test!(SW100_8_3, 3);
        morphed_test!(SW100_8_4, 4);
        morphed_test!(SW100_8_5, 5);
        morphed_test!(SW100_8_6, 6);
        morphed_test!(SW100_8_7, 7);
        morphed_test!(SW100_8_8, 8);

        #[test]
        fn SW100_8_p0() {
            default_on_dir(
                morphed_path().join("SW100-8-p0"),
                &Config::default(),
                Report::Satisfiable,
            );
        }
    }

    mod flat {
        use super::*;
        fn flat_path() -> PathBuf {
            colouring_path().join("flat")
        }

        #[macro_export]
        macro_rules! flat_test {
            ( $name:ident,  $n:literal, $m:literal ) => {
                #[test]
                fn $name() {
                    default_on_dir(
                        flat_path().join(format!("flat{}-{}", $n, $m)),
                        &Config::default(),
                        Report::Satisfiable,
                    );
                }
            };
        }

        flat_test!(flat30_60, 30, 60);
        flat_test!(flat50_115, 50, 115);
        flat_test!(flat75_180, 75, 180);
        flat_test!(flat100_239, 100, 239);
        flat_test!(flat125_301, 125, 301);
        flat_test!(flat150_360, 150, 360);
        flat_test!(flat175_417, 175, 417);
        flat_test!(flat200_479, 200, 479);
    }
}

#[test]
fn all_interval_series() {
    default_on_dir(
        satlib_collection("ais"),
        &Config::default(),
        Report::Satisfiable,
    );
}

#[test]
fn bounded_model_check() {
    default_on_dir(
        satlib_collection("bmc"),
        &Config::default(),
        Report::Satisfiable,
    );
}

#[test]
fn beijing() {
    let collection_path = satlib_collection("beijing");

    let satisfiable_formulas = [
        "2bitcomp_5.cnf.gz",
        "2bitmax_6.cnf.gz",
        "3bitadd_31.cnf.gz",
        "3bitadd_32.cnf.gz",
        "3blocks.cnf.gz",
        "4blocks.cnf.gz",
        "4blocksb.cnf.gz",
        "e0ddr2-10-by-5-1.cnf.gz",
        "e0ddr2-10-by-5-4.cnf.gz",
        "enddr2-10-by-5-1.cnf.gz",
        "enddr2-10-by-5-8.cnf.gz",
        "ewddr2-10-by-5-1.cnf.gz",
        "ewddr2-10-by-5-8.cnf.gz",
    ];

    let mut count = 0;
    for formula in satisfiable_formulas {
        assert_eq!(
            Report::Satisfiable,
            formula_report(collection_path.join(formula), &Config::default())
        );
        count += 1;
    }
    assert_eq!(count, satisfiable_formulas.len());
}

#[test]
fn quasigroup() {
    let collection_path = satlib_collection("quasigroup");

    let satisfiable_formulas = [
        "qg1-07.cnf.gz",
        "qg1-08.cnf.gz",
        "qg2-07.cnf.gz",
        "qg2-08.cnf.gz",
        "qg3-08.cnf.gz",
        "qg4-09.cnf.gz",
        "qg5-11.cnf.gz",
        "qg6-09.cnf.gz",
        "qg7-09.cnf.gz",
        "qg7-13.cnf.gz",
    ];
    let mut sat_count = 0;
    for formula in &satisfiable_formulas {
        assert_eq!(
            Report::Satisfiable,
            formula_report(collection_path.join(formula), &Config::default())
        );
        sat_count += 1;
    }
    assert_eq!(sat_count, satisfiable_formulas.len());

    let unsatisfiable_formulas = [
        "qg5-12.cnf.gz",
        "qg5-13.cnf.gz",
        "qg6-10.cnf.gz",
        "qg6-11.cnf.gz",
        "qg6-12.cnf.gz",
        "qg7-10.cnf.gz",
        "qg7-11.cnf.gz",
        "qg7-12.cnf.gz",
        "qg3-09.cnf.gz",
        "qg4-08.cnf.gz",
        "qg5-09.cnf.gz",
        "qg5-10.cnf.gz",
    ];
    let mut unsat_count = 0;
    for formula in &unsatisfiable_formulas {
        assert_eq!(
            Report::Unsatisfiable,
            formula_report(collection_path.join(formula), &Config::default())
        );
        unsat_count += 1;
    }
    assert_eq!(unsat_count, unsatisfiable_formulas.len());
}

mod DIMACS {
    use std::path;

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

    #[test]
    fn gcp() {
        default_on_dir(
            dimacs_path().join("GCP"),
            &Config::default(),
            Report::Satisfiable,
        );
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
    fn lran() {
        default_on_dir(
            dimacs_path().join("LRAN"),
            &Config::default(),
            Report::Satisfiable,
        );
    }

    #[test]
    fn parity() {
        default_on_dir(
            dimacs_path().join("PARITY"),
            &Config::default(),
            Report::Satisfiable,
        );
    }

    #[test]
    fn phole() {
        default_on_dir(
            dimacs_path().join("PHOLE"),
            &Config::default(),
            Report::Unsatisfiable,
        );
    }

    #[test]
    fn pret() {
        default_on_dir(
            dimacs_path().join("PRET"),
            &Config::default(),
            Report::Unsatisfiable,
        );
    }
}
