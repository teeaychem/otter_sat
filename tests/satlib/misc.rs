use otter_lib::{
    config::Config,
    io::files::{silent_formula_report, silent_on_directory},
    types::gen::Report,
};

use crate::satlib::*;

mod planning {

    use super::*;
    #[test]
    fn logistics() {
        silent_on_directory(
            satlib_collection("planning").join("logistics"),
            &Config::default(),
            Report::Satisfiable,
        );
    }

    #[test]
    fn blocksworld() {
        silent_on_directory(
            satlib_collection("planning").join("blocksworld"),
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
                    silent_on_directory(
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
            silent_on_directory(
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
                    silent_on_directory(
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
    let pass = silent_on_directory(
        satlib_collection("ais"),
        &Config::default(),
        Report::Satisfiable,
    );
    assert_eq!(pass, 4);
}

#[test]
fn bounded_model_check() {
    silent_on_directory(
        satlib_collection("bmc"),
        &Config::default(),
        Report::Satisfiable,
    );
}

#[test]
fn beijing() {
    let collection_path = satlib_collection("beijing");

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

#[test]
fn quasigroup() {
    let collection_path = satlib_collection("quasigroup");

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

    let unsatisfiable_formulas = [
        "qg5-12.cnf.xz",
        "qg5-13.cnf.xz",
        "qg6-10.cnf.xz",
        "qg6-11.cnf.xz",
        "qg6-12.cnf.xz",
        "qg7-10.cnf.xz",
        "qg7-11.cnf.xz",
        "qg7-12.cnf.xz",
        "qg3-09.cnf.xz",
        "qg4-08.cnf.xz",
        "qg5-09.cnf.xz",
        "qg5-10.cnf.xz",
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
