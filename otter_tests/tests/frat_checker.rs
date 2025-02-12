use std::{
    collections::HashSet,
    path::PathBuf,
    process::{self},
};

const FRAT_RS_PATH: &str = "./frat-rs";

use otter_sat::{
    config::Config,
    context::Context,
    db::{clause::db_clause::dbClause, ClauseKey},
    dispatch::frat::Transcriber,
    structures::clause::ClauseSource,
};

fn frat_verify(file_path: PathBuf, config: Config) -> bool {
    let mut frat_path_string = file_path.clone().to_str().unwrap().to_owned();
    frat_path_string.push_str(".frat");
    let frat_path = PathBuf::from(&frat_path_string);
    let transcriber = Transcriber::new(frat_path.clone()).unwrap();
    let tx = std::rc::Rc::new(std::cell::RefCell::new(transcriber));

    let addition_clone = tx.clone();
    let addition_callback = move |clause: &dbClause, source: &ClauseSource| {
        match source {
            ClauseSource::BCP => {
                if let ClauseKey::AdditionUnit(literal) = clause.key() {
                    addition_clone
                        .borrow_mut()
                        .transcribe_bcp(clause.key(), *literal);
                } else {
                    panic!("");
                }
            }

            ClauseSource::Original => addition_clone
                .borrow_mut()
                .transcribe_original_clause(clause.key(), clause.to_vec()),

            ClauseSource::Resolution => addition_clone
                .borrow_mut()
                .transcribe_addition_clause(clause.key(), clause.to_vec()),

            ClauseSource::PureUnit => panic!("X_X"),
        }
        addition_clone.borrow_mut().flush()
    };

    let deletion_clone = tx.clone();
    let deletion_callback = move |clause: &dbClause| {
        deletion_clone
            .borrow_mut()
            .transcribe_deletion(clause.key(), clause.to_vec());

        deletion_clone.borrow_mut().flush()
    };

    let resolution_clone = tx.clone();
    let resolution_presmises_callback = move |premises: &HashSet<ClauseKey>| {
        resolution_clone
            .borrow_mut()
            .transcribe_resolution(premises);
    };

    let unsatisfiable_clone = tx.clone();
    let unsatisfiable_callback = move |_: dbClause| {
        unsatisfiable_clone
            .borrow_mut()
            .transcribe_unsatisfiable_clause();
        unsatisfiable_clone.borrow_mut().flush();
    };

    let mut ctx = Context::from_config(config);

    ctx.clause_db
        .set_callback_delete(Box::new(deletion_callback));
    ctx.clause_db
        .set_callback_addition(Box::new(addition_callback));
    ctx.clause_db
        .set_callback_unsatisfiable(Box::new(unsatisfiable_callback));
    ctx.resolution_buffer
        .set_callback_resolution_premises(Box::new(resolution_presmises_callback));

    match load_dimacs(&mut ctx, &file_path) {
        Ok(()) => {}
        Err(e) => panic!("c Error loading file: {e:?}"),
    };

    let _result = ctx.solve();

    for (_, literal) in ctx.clause_db.all_original_unit_clauses() {
        tx.borrow_mut().transcribe_active_original_unit(literal);
    }

    for (_, literal) in ctx.clause_db.all_addition_unit_clauses() {
        tx.borrow_mut().transcribe_active_addition_unit(literal);
    }

    for (key, clause) in ctx.clause_db.all_binary_clauses() {
        tx.borrow_mut().transcribe_active(key, clause);
    }

    for (key, clause) in ctx.clause_db.all_original_long_clauses() {
        tx.borrow_mut().transcribe_active(key, clause);
    }

    for (key, clause) in ctx.clause_db.all_active_addition_long_clauses() {
        tx.borrow_mut().transcribe_active(key, clause);
    }
    tx.borrow_mut().flush();

    drop(ctx);

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

fn frat_dir_test(dir: PathBuf) -> usize {
    let mut counter = 0;

    if let Some(dir) = dir.to_str() {
        for entry in glob::glob(format!("{dir}/*.xz").as_str()).expect("bad glob") {
            let formula = entry.unwrap();
            let mut config = Config::default();
            config.switch.subsumption = false;

            match frat_verify(formula, config) {
                true => counter += 1,
                false => break,
            }
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

        let mut config = Config::default();
        config.switch.subsumption = false;

        assert!(frat_verify(file_path.clone(), config));

        // let mut config = Config::default();
        // config.switch.subsumption = true;

        // assert!(
        //     !frat_verify(file_path, config),
        //     "Unless subsumption proofs…"
        // );
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
                config.switch.subsumption = false;
                assert!(frat_verify(quasigroup_dir().join("qg3-09.cnf.xz"), config));
            }

            #[test]
            fn qg4() {
                let mut config = Config::default();
                config.switch.subsumption = false;
                assert!(frat_verify(quasigroup_dir().join("qg4-08.cnf.xz"), config));
            }

            #[rustfmt::skip]
            #[test]
            #[ignore = "slower than other quasigroup tests"]
            fn qg5() {
                let mut config = Config::default();
                config.switch.subsumption = false;
                assert!(frat_verify(quasigroup_dir().join("qg5-09.cnf.xz"), config.clone()));
                assert!(frat_verify(quasigroup_dir().join("qg5-10.cnf.xz"), config.clone()));
                assert!(frat_verify(quasigroup_dir().join("qg5-12.cnf.xz"), config.clone()));
                assert!(frat_verify(quasigroup_dir().join("qg5-13.cnf.xz"), config.clone()));
            }

            #[rustfmt::skip]
            #[test]
            fn qg6() {
                let mut config = Config::default();
                config.switch.subsumption = false;
                assert!(frat_verify(quasigroup_dir().join("qg6-10.cnf.xz"), config.clone()));
                assert!(frat_verify(quasigroup_dir().join("qg6-11.cnf.xz"), config.clone()));
                assert!(frat_verify(quasigroup_dir().join("qg6-12.cnf.xz"), config.clone()));
            }

            #[rustfmt::skip]
            #[test]
            fn qg7() {
                let mut config = Config::default();
                config.switch.subsumption = false;
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
                assert_eq!(frat_dir_test(dir), 13);
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

                    let mut config = Config::default();
                    config.switch.subsumption = false;

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
                    config.switch.subsumption = false;

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
                    config.switch.subsumption = false;
                    let files = ["hole6.cnf.xz", "hole7.cnf.xz", "hole8.cnf.xz"];
                    for file in files {
                        assert!(frat_verify(phole_dir().join(file), config.clone()));
                    }
                }

                #[test]
                #[ignore = "expensive unsat"]
                fn hole9() {
                    let mut config = Config::default();
                    config.switch.subsumption = false;
                    assert!(frat_verify(phole_dir().join("hole9.cnf.xz"), config));
                }

                #[test]
                #[ignore = "expensive unsat"]
                fn hole10() {
                    let mut config = Config::default();
                    config.switch.subsumption = false;
                    assert!(frat_verify(phole_dir().join("hole10.cnf.xz"), config));
                }
            }
        }
    }
}
