use super::*;

mod ThreeSAT {
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

    mod group_50_218 {
        use super::*;
        #[test]
        fn sat() {
            default_on_dir(
                unform_random_path().join("UF50.218.1000").join("sat"),
                &Config::default(),
                Report::Satisfiable,
            );
        }

        #[test]
        fn unsat() {
            default_on_dir(
                unform_random_path().join("UF50.218.1000").join("unsat"),
                &Config::default(),
                Report::Unsatisfiable,
            );
        }
    }

    #[test]
    #[ignore]
    fn uniform_random_3_225_960() {
        default_on_split_dir(
            unform_random_path().join("UF225.960.100"),
            &Config::default(),
        );
    }

    #[test]
    #[ignore]
    fn uniform_random_3_250_1065() {
        default_on_split_dir(
            unform_random_path().join("UF250.1065.100"),
            &Config::default(),
        );
    }
}

mod ThreeSAT_and_backbone_minimal_subinstances {
    use super::*;

    #[test]
    fn rti_k3_n100_m429() {
        default_on_dir(
            satlib_collection("backbone").join("RTI_k3_n100_m429"),
            &Config::default(),
            Report::Satisfiable,
        );
    }

    #[test]
    fn bms_k3_n100_m429() {
        default_on_dir(
            satlib_collection("backbone").join("BMS_k3_n100_m429"),
            &Config::default(),
            Report::Satisfiable,
        );
    }
}
