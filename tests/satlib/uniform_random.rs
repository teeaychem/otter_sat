use super::*;
use otter_lib::{
    config::Config,
    dispatch::report::{self},
    io::files::silent_on_directory,
};

mod ThreeSAT {

    use otter_lib::io::files::silent_on_split_directory;

    use super::*;
    fn unform_random_path() -> PathBuf {
        satlib_path().join("uniform_random")
    }

    #[test]
    fn uniform_random_3_20_91() {
        silent_on_directory(
            unform_random_path().join("UF20.91"),
            &Config::default(),
            report::Solve::Satisfiable,
        );
    }

    mod group_50_218 {
        use super::*;
        #[test]
        fn sat() {
            silent_on_directory(
                unform_random_path().join("UF50.218.1000").join("sat"),
                &Config::default(),
                report::Solve::Satisfiable,
            );
        }

        #[test]
        fn unsat() {
            silent_on_directory(
                unform_random_path().join("UF50.218.1000").join("unsat"),
                &Config::default(),
                report::Solve::Unsatisfiable,
            );
        }
    }

    #[test]
    #[ignore]
    fn uniform_random_3_225_960() {
        silent_on_split_directory(
            unform_random_path().join("UF225.960.100"),
            &Config::default(),
        );
    }

    #[test]
    #[ignore]
    fn uniform_random_3_250_1065() {
        silent_on_split_directory(
            unform_random_path().join("UF250.1065.100"),
            &Config::default(),
        );
    }
}

mod ThreeSAT_and_backbone_minimal_subinstances {
    use super::*;

    #[test]
    fn rti_k3_n100_m429() {
        silent_on_directory(
            satlib_collection("backbone").join("RTI_k3_n100_m429"),
            &Config::default(),
            report::Solve::Satisfiable,
        );
    }

    #[test]
    fn bms_k3_n100_m429() {
        silent_on_directory(
            satlib_collection("backbone").join("BMS_k3_n100_m429"),
            &Config::default(),
            report::Solve::Satisfiable,
        );
    }
}
