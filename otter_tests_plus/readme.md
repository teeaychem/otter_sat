# otter_tests_plus

This folder contains tests for otter_sat which do not (reasonably) fit into the otter_tests crate.

# tests

Note, these tests assume `otter_tests_plus` at the default location within the `otter_sat` repository.

## ipasir_api

Tests of the IPASIR API (version 1 (one)) found at [https://github.com/biotomas/ipasir](https://github.com/biotomas/ipasir).
Included here as the build otter_sat suitable for C linking and in turn link the library to a handful of C/++ applications for testing.

Each test is for a specific example application from the IPASIR repo, with expected results derived from calling the same test with `MiniSAT` over `otter_sat`.[^1]

[^1]: MiniSAT is used (over the default picosat-961, etc.) as compilation via CMake is fairly simple.

The tests may be run by:

0. Ensuring the included `CMakeLists.txt` file will find the appropriate instance of the otter_sat library when compiled (inspect the `LIBRARIES` variable).
1. Cloning the IPASIR repo into `ipasir_api` (with structure `ipasir_api/ipasir/ipasir.h`, etc.)
2. Calling `python setup.py` to build relevant binaries, or do so directly using `CMakeLists.txt`.
3. Calling `python -m unittest [-v] [test_specific.py]`.

Skipped tests can be enabled by passing `TEST_LEVEL=1`, e.g. `TEST_LEVEL=1 py -m unittest -v test_iterative.py` will run all the iterative tests.
