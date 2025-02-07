import pathlib
import subprocess

CURRENT_DIR = pathlib.Path(__file__).parent
BUILD_DIR = CURRENT_DIR.joinpath("build")
VERBOSE = False

def compare_results(cnf, pre_line_skip, post_line_skip,  minisat, otter_sat):
    if VERBOSE:
        print('\n'.join(minisat))
        print('\n'.join(otter_sat))

    if pre_line_skip > 0:
        minisat = minisat[pre_line_skip:]
        otter_sat = otter_sat[pre_line_skip:]
    if post_line_skip > 0:
        minisat = minisat[: -pre_line_skip]
        otter_sat = otter_sat[: -pre_line_skip]

    if minisat != otter_sat:
        print(f"\tFAIL: {cnf}")
        print("MiniSAT:")
        print('\n\t'.join(minisat))
        print("otter_sat:")
        print('\n\t'.join(otter_sat))
    else:
        print("\tPASS")


def is_comment(line):
    return len(line) > 0 and line[0] == 'c'


def build_binaries():
    """
    Builds all the binaries to a 'build' subfolder.
    Equivalent to directly calling (e.g.): mkdir build && cd build && cmake .. && make
    """

    print("CMake setup…")

    cmake_result = subprocess.run(["cmake", "-S", ".", "-B", "build"], capture_output=True, text=True)
    if cmake_result.returncode != 0:
        print(cmake_result.stderr)

    print("Building…")

    make_result = subprocess.run(["make"], cwd=BUILD_DIR, capture_output=True, text=True)
    if make_result.returncode != 0:
        print(make_result.stderr)

    print("Build okay!")
