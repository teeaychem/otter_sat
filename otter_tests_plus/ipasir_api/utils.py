import pathlib
import subprocess

CURRENT_DIR = pathlib.Path(__file__).parent
BUILD_DIR = CURRENT_DIR.joinpath("build")


def is_comment(line):
    return len(line) > 0 and line[0] == "c"


def build_binaries():
    """
    Builds all the binaries to a 'build' subfolder.
    Equivalent to directly calling (e.g.): mkdir build && cd build && cmake .. && make
    """

    print("CMake setup…")

    cmake_result = subprocess.run(
        ["cmake", "-S", ".", "-B", "build"], capture_output=True, text=True
    )
    if cmake_result.returncode != 0:
        print(cmake_result.stderr)

    print("Building…")

    make_result = subprocess.run(
        ["make"], cwd=BUILD_DIR, capture_output=True, text=True
    )
    if make_result.returncode != 0:
        print(make_result.stderr)

    print("Build okay!")
