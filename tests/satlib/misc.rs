use otter_lib::{
    config::Config,
    context::Report,
    io::files::{default_on_dir, formula_report},
};

use crate::satlib::*;

mod planning {
    use otter_lib::{config::Config, context::Report, io::files::default_on_dir};

    use super::*;
    #[test]
    fn logistics() {
        default_on_dir(
            satlib_collection("planning").join("logistics"),
            &Config::default(),
            Report::Satisfiable,
        );
    }

    #[test]
    fn blocksworld() {
        default_on_dir(
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
        use otter_lib::{config::Config, context::Report, io::files::default_on_dir};

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
