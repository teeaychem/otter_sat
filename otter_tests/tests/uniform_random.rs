use otter_sat::{config::Config, reports::Report};

use otter_tests::general::{cnf_lib_subdir, silent_on_directory};

#[cfg(test)]
mod three_sat {

    use super::*;

    #[test]
    fn uniform_random_3_20_91() {
        silent_on_directory(
            cnf_lib_subdir(vec!["SATLIB", "uniform_random", "UF20.91"]),
            &Config::default(),
            Report::Satisfiable,
        );
    }

    mod group_50_218 {
        use super::*;
        #[test]
        fn sat() {
            silent_on_directory(
                cnf_lib_subdir(vec!["SATLIB", "uniform_random", "UF50.218.1000", "sat"]),
                &Config::default(),
                Report::Satisfiable,
            );
        }

        #[test]
        fn unsat() {
            silent_on_directory(
                cnf_lib_subdir(vec!["SATLIB", "uniform_random", "UF50.218.1000", "unsat"]),
                &Config::default(),
                Report::Unsatisfiable,
            );
        }
    }

    mod group_225_960 {
        use super::*;
        #[test]
        #[ignore = "expensive"]
        fn sat() {
            silent_on_directory(
                cnf_lib_subdir(vec!["SATLIB", "uniform_random", "UF225.960.100", "sat"]),
                &Config::default(),
                Report::Satisfiable,
            );
        }

        #[test]
        #[ignore = "expensive"]
        fn unsat() {
            silent_on_directory(
                cnf_lib_subdir(vec!["SATLIB", "uniform_random", "UF225.960.100", "unsat"]),
                &Config::default(),
                Report::Unsatisfiable,
            );
        }
    }

    mod group_250_106 {
        use super::*;
        #[test]
        #[ignore = "expensive"]
        fn sat() {
            silent_on_directory(
                cnf_lib_subdir(vec!["SATLIB", "uniform_random", "UF250.1065.100", "sat"]),
                &Config::default(),
                Report::Satisfiable,
            );
        }

        #[test]
        #[ignore = "expensive"]
        fn unsat() {
            silent_on_directory(
                cnf_lib_subdir(vec!["SATLIB", "uniform_random", "UF250.1065.100", "unsat"]),
                &Config::default(),
                Report::Unsatisfiable,
            );
        }
    }
}

#[cfg(test)]
mod three_sat_and_backbone_minimal_subinstances {
    use super::*;

    #[test]
    fn rti_k3_n100_m429() {
        #[cfg(feature = "log")]
        env_logger::init();

        silent_on_directory(
            cnf_lib_subdir(vec!["SATLIB", "backbone", "RTI_k3_n100_m429"]),
            &Config::default(),
            Report::Satisfiable,
        );
    }

    #[test]
    fn bms_k3_n100_m429() {
        silent_on_directory(
            cnf_lib_subdir(vec!["SATLIB", "backbone", "BMS_k3_n100_m429"]),
            &Config::default(),
            Report::Satisfiable,
        );
    }
}
