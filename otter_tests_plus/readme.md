# otter_tests_plus

This folder contains tests for otter_sat which do not (reasonably) fit into the otter_tests crate.

# tests

## ipasir_api

Tests of the IPASIR API (version 1 (one)) found at [https://github.com/biotomas/ipasir](https://github.com/biotomas/ipasir).

The tests may be run by:

1. Cloning the IPASIR repo into `ipasir_api` (the folder should be named `ipasir`, and so there will be `ipasir_api/ipasir/ipasir.h`, etc.)
2. Modifying `compare_minisat.py` to include the tests of choice.
3. Calling `compare_minisat.py`.

In short, `compare_minisat.py` uses the included CMake file to build binaries of the example applications in IPASIR repository and compares the output of the applications.
MiniSAT is used (over the default picosat-961, etc.) as compilation via CMake is fairly simple.

Note, these tests assume `otter_tests_plus` at the default location within the `otter_sat` repository.
