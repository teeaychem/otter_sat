mod graph_colouring {
    use otter_sat::{config::Config, reports::Report};
    use otter_tests::general::{cnf_lib_subdir, silent_formula_report, silent_on_directory};

    use std::path::PathBuf;

    #[test]
    #[ignore = "expensive"]
    fn one_two_five() {
        assert_eq!(
            Report::Satisfiable,
            silent_formula_report(
                cnf_lib_subdir(vec!["SATLIB", "DIMACS", "GCP", "g125.17.cnf.xz"]),
                &Config::default()
            )
        );

        assert_eq!(
            Report::Satisfiable,
            silent_formula_report(
                cnf_lib_subdir(vec!["SATLIB", "DIMACS", "GCP", "g125.18.cnf.xz"]),
                &Config::default()
            )
        );

        assert_eq!(
            Report::Satisfiable,
            silent_formula_report(
                cnf_lib_subdir(vec!["SATLIB", "DIMACS", "GCP", "g250.15.cnf.xz"]),
                &Config::default()
            )
        );

        assert_eq!(
            Report::Satisfiable,
            silent_formula_report(
                cnf_lib_subdir(vec!["SATLIB", "DIMACS", "GCP", "g250.29.cnf.xz"]),
                &Config::default()
            )
        );
    }

    mod morphed {

        use otter_tests::general::silent_on_directory;

        use super::*;

        fn morphed_path() -> PathBuf {
            cnf_lib_subdir(vec!["SATLIB", "graph_colouring", "morphed"])
        }

        #[macro_export]
        macro_rules! morphed_test {
            ( $name:ident,  $n:literal ) => {
                silent_on_directory(
                    morphed_path().join(format!("SW100-8-{}", $n)),
                    &Config::default(),
                    Report::Satisfiable,
                );
            };
        }

        #[test]
        fn sw100() {
            morphed_test!(sw100_8_0, 0);
            morphed_test!(sw100_8_1, 1);
            morphed_test!(sw100_8_2, 2);
            morphed_test!(sw100_8_3, 3);
            morphed_test!(sw100_8_4, 4);
            morphed_test!(sw100_8_5, 5);
            morphed_test!(sw100_8_6, 6);
            morphed_test!(sw100_8_7, 7);
            morphed_test!(sw100_8_8, 8);
        }

        #[test]
        fn sw100_8_p0() {
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
            cnf_lib_subdir(vec!["SATLIB", "graph_colouring", "flat"])
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
