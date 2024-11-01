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
    let config = Config::default();
    let collection_path = satlib_collection("beijing");

    let satisfiable_formulas = vec![
        "2bitcomp_5.cnf",
        "2bitmax_6.cnf",
        "3bitadd_31.cnf",
        "3bitadd_32.cnf",
        "3blocks.cnf",
        "4blocks.cnf",
        "4blocksb.cnf",
        "e0ddr2-10-by-5-1.cnf",
        "e0ddr2-10-by-5-4.cnf",
        "enddr2-10-by-5-1.cnf",
        "enddr2-10-by-5-8.cnf",
        "ewddr2-10-by-5-1.cnf",
        "ewddr2-10-by-5-8.cnf",
    ];
    for formula in satisfiable_formulas {
        assert_eq!(
            default_on_path(collection_path.join(formula), &config),
            Report::Satisfiable
        );
    }
}

#[test]
fn quasigroup() {
    let config = Config::default();
    let collection_path = satlib_collection("quasigroup");

    let satisfiable_formulas = vec![
        "qg1-07.cnf",
        "qg1-08.cnf",
        "qg2-07.cnf",
        "qg2-08.cnf",
        "qg3-08.cnf",
        "qg4-09.cnf",
        "qg5-11.cnf",
        "qg6-09.cnf",
        "qg7-09.cnf",
        "qg7-13.cnf",
    ];
    for formula in satisfiable_formulas {
        assert_eq!(
            default_on_path(collection_path.join(formula), &config),
            Report::Satisfiable
        );
    }
    let unsatisfiable_formulas = vec![
        "qg5-12.cnf",
        "qg5-13.cnf",
        "qg6-10.cnf",
        "qg6-11.cnf",
        "qg6-12.cnf",
        "qg7-10.cnf",
        "qg7-11.cnf",
        "qg7-12.cnf",
        "qg3-09.cnf",
        "qg4-08.cnf",
        "qg5-09.cnf",
        "qg5-10.cnf",
    ];
    for formula in unsatisfiable_formulas {
        assert_eq!(
            default_on_path(collection_path.join(formula), &config),
            Report::Unsatisfiable
        );
    }
}
