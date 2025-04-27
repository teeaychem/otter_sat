use otter_sat::config::Config;
use otter_tests::{
    frat::{frat_dir_test, frat_verify},
    general::cnf_lib_subdir,
};

use std::path::PathBuf;

mod frat_setup {

    use super::*;

    #[test]
    fn frat_setup_check() {
        let file_path = cnf_lib_subdir(vec!["frat", "tt.cnf"]);

        let mut config = Config::default();
        config.subsumption.value = false;

        assert!(frat_verify(file_path.clone(), config));

        // let mut config = Config::default();
        // config.subsumption = true;

        // assert!(
        //     !frat_verify(file_path, config),
        //     "Unless subsumption proofs…"
        // );
    }
}

mod frat_satlib {

    use super::*;
    fn satlib_dir() -> PathBuf {
        cnf_lib_subdir(vec!["SATLIB"])
    }

    mod uniform_random {

        use super::*;
        fn uniform_random_dir() -> PathBuf {
            satlib_dir().join("uniform_random")
        }

        #[test]
        fn uf50_218_1000() {
            let dir = uniform_random_dir().join("UF50.218.1000").join("unsat");
            assert_eq!(frat_dir_test(dir), 1000);
        }

        #[test]
        #[ignore = "expensive"]
        fn uf200_860_100() {
            let dir = uniform_random_dir().join("UF200.860.100").join("unsat");
            assert_eq!(frat_dir_test(dir), 99);
        }

        #[test]
        #[ignore = "expensive"]
        fn uf225_960_100() {
            let dir = uniform_random_dir().join("UF225.960.100").join("unsat");
            assert_eq!(frat_dir_test(dir), 100);
        }

        #[test]
        #[ignore = "expensive"]
        fn uf250_1065_100() {
            let dir = uniform_random_dir().join("UF250.1065.100").join("unsat");
            assert_eq!(frat_dir_test(dir), 100);
        }
    }

    mod quasigroup {
        use super::*;
        fn quasigroup_dir() -> PathBuf {
            satlib_dir().join("quasigroup")
        }

        #[test]
        #[ignore = "slower than other quasigroup tests"]
        fn qg3() {
            let mut config = Config::default();
            config.subsumption.value = false;
            assert!(frat_verify(quasigroup_dir().join("qg3-09.cnf.xz"), config));
        }

        #[test]
        fn qg4() {
            let mut config = Config::default();
            config.subsumption.value = false;
            assert!(frat_verify(quasigroup_dir().join("qg4-08.cnf.xz"), config));
        }

        // #[rustfmt::skip]
        // #[test]
        // #[ignore = "slower than other quasigroup tests"]
        // fn qg5() {
        //     let mut config = Config::default();
        //     config.subsumption.value = false;
        //     assert!(frat_verify(quasigroup_dir().join("qg5-09.cnf.xz"), config.clone()));
        //     assert!(frat_verify(quasigroup_dir().join("qg5-10.cnf.xz"), config.clone()));
        //     assert!(frat_verify(quasigroup_dir().join("qg5-12.cnf.xz"), config.clone()));
        //     assert!(frat_verify(quasigroup_dir().join("qg5-13.cnf.xz"), config.clone()));
        // }

        #[rustfmt::skip]
        #[test]
        fn qg6() {
            let mut config = Config::default();
            config.subsumption.value = false;
            assert!(frat_verify(quasigroup_dir().join("qg6-10.cnf.xz"), config.clone()));
            assert!(frat_verify(quasigroup_dir().join("qg6-11.cnf.xz"), config.clone()));
            assert!(frat_verify(quasigroup_dir().join("qg6-12.cnf.xz"), config.clone()));
        }

        #[rustfmt::skip]
        #[test]
        fn qg7() {
            let mut config = Config::default();
            config.subsumption.value = false;
            assert!(frat_verify(quasigroup_dir().join("qg7-10.cnf.xz"), config.clone()));
            assert!(frat_verify(quasigroup_dir().join("qg7-11.cnf.xz"), config.clone()));
            assert!(frat_verify(quasigroup_dir().join("qg7-12.cnf.xz"), config.clone()));
        }
    }

    mod dimacs {
        use super::*;
        fn dimacs_dir() -> PathBuf {
            satlib_dir().join("DIMACS")
        }

        #[test]
        fn dubois() {
            let dir = dimacs_dir().join("DUBOIS");
            assert_eq!(frat_dir_test(dir), 13);
        }

        mod circuit {
            use super::*;
            fn circuit_dir() -> PathBuf {
                dimacs_dir().join("CFA")
            }

            #[rustfmt::skip]
                #[test]
                fn bf() {
                    let bf_dir = circuit_dir().join("BF");

                    let mut config = Config::default();
                    config.subsumption.value = false;

                    assert!(frat_verify(bf_dir.join("bf0432-007.cnf.xz"), config.clone()));
                    assert!(frat_verify(bf_dir.join("bf1355-075.cnf.xz"), config.clone()));
                    assert!(frat_verify(bf_dir.join("bf1355-638.cnf.xz"), config.clone()));
                    assert!(frat_verify(bf_dir.join("bf2670-001.cnf.xz"), config.clone()));
                }

            #[rustfmt::skip]
                #[test]
                fn ssa() {
                    let ssa_dir = circuit_dir().join("SSA");

                    let mut config = Config::default();
                    config.subsumption.value = false;

                    assert!(frat_verify(ssa_dir.join("ssa0432-003.cnf.xz"), config.clone()));
                    assert!(frat_verify(ssa_dir.join("ssa2670-130.cnf.xz"), config.clone()));
                    assert!(frat_verify(ssa_dir.join("ssa2670-141.cnf.xz"), config.clone()));
                    assert!(frat_verify(ssa_dir.join("ssa6288-047.cnf.xz"), config.clone()));
                }
        }

        mod pigeon {
            use super::*;

            fn phole_dir() -> PathBuf {
                dimacs_dir().join("PHOLE")
            }

            #[test]
            fn hole678() {
                let mut config = Config::default();
                config.subsumption.value = false;
                let files = ["hole6.cnf.xz", "hole7.cnf.xz", "hole8.cnf.xz"];
                for file in files {
                    assert!(frat_verify(phole_dir().join(file), config.clone()));
                }
            }

            #[test]
            #[ignore = "expensive unsat"]
            fn hole9() {
                let mut config = Config::default();
                config.subsumption.value = false;
                assert!(frat_verify(phole_dir().join("hole9.cnf.xz"), config));
            }

            #[test]
            #[ignore = "expensive unsat"]
            fn hole10() {
                let mut config = Config::default();
                config.subsumption.value = false;
                assert!(frat_verify(phole_dir().join("hole10.cnf.xz"), config));
            }
        }
    }
}
