use std::{
    path::PathBuf,
    process::{self},
    thread,
};

const FRAT_RS_PATH: &str = "./frat-rs";

use crossbeam::channel::unbounded;
use otter_lib::{config::Config, context::Context, dispatch::Dispatch};

fn frat_verify(file_path: PathBuf, config: Config) -> bool {
    let mut frat_path_string = file_path.clone().to_str().unwrap().to_owned();
    frat_path_string.push_str(".frat");
    let frat_path = PathBuf::from(&frat_path_string);

    let (tx, rx) = unbounded::<Dispatch>();

    let listener_handle = {
        let frat_path = frat_path.clone();
        thread::spawn(|| otter_lib::dispatch::receivers::frat_receiver(rx, frat_path))
    };

    let mut the_context = Context::from_config(config, tx);

    match load_dimacs(&mut the_context, &file_path) {
        Ok(()) => {}
        Err(e) => {
            panic!("c Error loading file: {e:?}")
        }
    };

    let _result = the_context.solve();
    the_context.report_active();

    drop(the_context);

    let _ = listener_handle.join();

    let mut frat_process = process::Command::new(FRAT_RS_PATH);
    frat_process.arg("elab");
    frat_process.arg(frat_path_string.clone());
    frat_process.arg("-m"); // keep the intermediate file in memory

    let output = match frat_process.output() {
        Ok(out) => out,
        Err(e) => panic!("{e:?}"),
    };

    let _ = std::fs::remove_file(frat_path);
    match output.status.code() {
        Some(0) => true,
        _ => {
            println!("{output:?}");
            false
        }
    }
}

fn frat_dir_test(dir: String) -> usize {
    let mut counter = 0;
    for entry in glob::glob(format!("{dir}/*.xz").as_str()).expect("bad glob") {
        let formula = entry.unwrap();
        let config = Config {
            subsumption: false,
            ..Default::default()
        };
        if frat_verify(formula, config) {
            counter += 1
        } else {
            break;
        }
    }
    counter
}

use otter_tests::{cnf_lib_subdir, load_dimacs};

#[cfg(test)]
mod frat_tests {

    use super::*;

    #[test]
    fn frat_setup_check() {
        let file_path = cnf_lib_subdir(vec!["frat", "tt.cnf"]);

        let config = Config {
            subsumption: false,
            ..Default::default()
        };

        assert!(frat_verify(file_path.clone(), config));

        let config = Config {
            subsumption: true,
            ..Default::default()
        };

        assert!(
            !frat_verify(file_path, config),
            "Unless subsumption proofs…"
        );
    }

    #[allow(non_snake_case)]
    mod SATLIB {
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
                assert_eq!(frat_dir_test(dir.to_str().unwrap().to_owned()), 1000);
            }

            #[test]
            #[ignore = "expensive"]
            fn uf200_860_100() {
                let dir = uniform_random_dir().join("UF200.860.100").join("unsat");
                assert_eq!(frat_dir_test(dir.to_str().unwrap().to_owned()), 99);
            }

            #[test]
            #[ignore = "expensive"]
            fn uf225_960_100() {
                let dir = uniform_random_dir().join("UF225.960.100").join("unsat");
                assert_eq!(frat_dir_test(dir.to_str().unwrap().to_owned()), 100);
            }

            #[test]
            #[ignore = "expensive"]
            fn uf250_1065_100() {
                let dir = uniform_random_dir().join("UF250.1065.100").join("unsat");
                assert_eq!(frat_dir_test(dir.to_str().unwrap().to_owned()), 100);
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
                let config = Config {
                    subsumption: false,
                    ..Default::default()
                };
                assert!(frat_verify(quasigroup_dir().join("qg3-09.cnf.xz"), config));
            }

            #[test]
            fn qg4() {
                let config = Config {
                    subsumption: false,
                    ..Default::default()
                };
                assert!(frat_verify(quasigroup_dir().join("qg4-08.cnf.xz"), config));
            }

            #[rustfmt::skip]
            #[test]
            #[ignore = "slower than other quasigroup tests"]
            fn qg5() {
                let config = Config {
                    subsumption: false,
                    ..Default::default()
                };
                assert!(frat_verify(quasigroup_dir().join("qg5-09.cnf.xz"), config.clone()));
                assert!(frat_verify(quasigroup_dir().join("qg5-10.cnf.xz"), config.clone()));
                assert!(frat_verify(quasigroup_dir().join("qg5-12.cnf.xz"), config.clone()));
                assert!(frat_verify(quasigroup_dir().join("qg5-13.cnf.xz"), config.clone()));
            }

            #[rustfmt::skip]
            #[test]
            fn qg6() {
                let config = Config {
                    subsumption: false,
                    ..Default::default()
                };
                assert!(frat_verify(quasigroup_dir().join("qg6-10.cnf.xz"), config.clone()));
                assert!(frat_verify(quasigroup_dir().join("qg6-11.cnf.xz"), config.clone()));
                assert!(frat_verify(quasigroup_dir().join("qg6-12.cnf.xz"), config.clone()));
            }

            #[rustfmt::skip]
            #[test]
            fn qg7() {
                let config = Config {
                    subsumption: false,
                    ..Default::default()
                };
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
            fn DUBOIS() {
                let dir = dimacs_dir().join("DUBOIS");
                assert_eq!(frat_dir_test(dir.to_str().unwrap().to_owned()), 13);
            }

            mod curcuit {
                use super::*;
                fn circuit_dir() -> PathBuf {
                    dimacs_dir().join("CFA")
                }

                #[rustfmt::skip]
                #[test]
                fn bf() {
                    let bf_dir = circuit_dir().join("BF");

                    let config = Config {
                        subsumption: false,
                        ..Default::default()
                    };

                    assert!(frat_verify(bf_dir.join("bf0432-007.cnf.xz"), config.clone()));
                    assert!(frat_verify(bf_dir.join("bf1355-075.cnf.xz"), config.clone()));
                    assert!(frat_verify(bf_dir.join("bf1355-638.cnf.xz"), config.clone()));
                    assert!(frat_verify(bf_dir.join("bf2670-001.cnf.xz"), config.clone()));
                }

                #[rustfmt::skip]
                #[test]
                fn ssa() {
                    let ssa_dir = circuit_dir().join("SSA");

                    let config = Config {
                        subsumption: false,
                        ..Default::default()
                    };

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

                #[rustfmt::skip]
                #[test]
                fn hole678() {
                    let config = Config { subsumption: false, ..Default::default() };
                    assert!(frat_verify(phole_dir().join("hole6.cnf.xz"), config.clone()));
                    assert!(frat_verify(phole_dir().join("hole7.cnf.xz"), config.clone()));
                    assert!(frat_verify(phole_dir().join("hole8.cnf.xz"), config.clone()));
                }

                #[rustfmt::skip]
                #[test]
                #[ignore = "expensive unsat"]
                fn hole9() {
                    let config = Config { subsumption: false, ..Default::default() };
                    assert!(frat_verify(phole_dir().join("hole9.cnf.xz"), config));
                }

                #[rustfmt::skip]
                #[test]
                #[ignore = "expensive unsat"]
                fn hole10() {
                    let config = Config { subsumption: false, ..Default::default() };
                    assert!(frat_verify(phole_dir().join("hole10.cnf.xz"), config));
                }
            }
        }
    }
}
