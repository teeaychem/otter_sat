import pathlib
import subprocess
import glob

CURRENT_DIR = pathlib.Path(__file__).parent
BUILD_DIR = CURRENT_DIR.joinpath("build")
VERBOSE = False

def compare_results(cnf, line_skip, minisat, otter_sat):
    if VERBOSE:
        print('\n'.join(minisat))
        print('\n'.join(otter_sat))

    if minisat[line_skip:] != otter_sat[line_skip:]:
        print(f"\tFAIL: {cnf}")
        print('\n\t'.join(minisat))
        print('\n\t'.join(otter_sat))
    else:
        print("\tPASS")

def is_comment(line):
    return len(line) > 0 and line[0] == 'c'

def build_binaries():

    print("CMake setup…")

    cmake_result = subprocess.run(["cmake", "-S", ".", "-B", "build"], capture_output=True, text=True)
    if cmake_result.returncode != 0:
        print(cmake_result.stderr)

    print("Building…")

    make_result = subprocess.run(["make"], cwd=BUILD_DIR, capture_output=True, text=True)
    if make_result.returncode != 0:
        print(make_result.stderr)

    print("Build okay!")

def test_backbone():

    backbone_test_directory = CURRENT_DIR.joinpath(pathlib.Path("ipasir", "app", "genipabones", "inputs"))

    for cnf in glob.glob(f"{backbone_test_directory}/*.cnf"):

        print(f"Checking: {cnf}")

        minisat_output = []
        otter_sat_output = []

        minisat = subprocess.run(["./genipabones_minisat", cnf], cwd=BUILD_DIR, capture_output=True, text=True)
        for line in minisat.stdout.split('\n'):
            if not is_comment(line):
                minisat_output.append(line)

        otter_sat = subprocess.run(["./genipabones_otter_sat", cnf], cwd=BUILD_DIR, capture_output=True, text=True)
        for line in otter_sat.stdout.split('\n'):
            if not is_comment(line):
                otter_sat_output.append(line)

        compare_results(cnf, 0, minisat_output, otter_sat_output)

def test_essentials():

    essentials_test_directory = CURRENT_DIR.joinpath(pathlib.Path("ipasir", "app", "genipaessentials", "inputs"))

    for cnf in glob.glob(f"{essentials_test_directory}/*.cnf"):

        print(f"Checking: {cnf}")

        minisat_output = []
        otter_sat_output = []

        minisat = subprocess.run(["./genipaessentials_minisat", cnf], cwd=BUILD_DIR, capture_output=True, text=True)
        for line in minisat.stdout.split('\n'):
            if not is_comment(line):
                minisat_output.append(line)

        otter_sat = subprocess.run(["./genipaessentials_otter_sat", cnf], cwd=BUILD_DIR, capture_output=True, text=True)
        for line in otter_sat.stdout.split('\n'):
            if not is_comment(line):
                otter_sat_output.append(line)

        compare_results(cnf, 0, minisat_output, otter_sat_output)


def test_portfolio():

    portfolio_test_directory = CURRENT_DIR.joinpath(pathlib.Path("ipasir", "app", "genipafolio", "inputs"))
    stdout_prefix = "c [genipafolio]"
    stdout_prefix_len = len(stdout_prefix)

    for cnf in glob.glob(f"{portfolio_test_directory}/*.cnf"):

        minisat_output = []
        otter_sat_output = []

        minisat = subprocess.run(["./genipafolio_minisat", cnf], cwd=BUILD_DIR, capture_output=True, text=True)
        for line in minisat.stdout.split('\n'):
            if len(line) > stdout_prefix_len and line[:stdout_prefix_len] == stdout_prefix:
                minisat_output.append(line)

        otter_sat = subprocess.run(["./genipafolio_otter_sat", cnf], cwd=BUILD_DIR, capture_output=True, text=True)
        for line in otter_sat.stdout.split('\n'):
            if len(line) > stdout_prefix_len and line[:stdout_prefix_len] == stdout_prefix:
                otter_sat_output.append(line)

        compare_results(cnf, 2, minisat_output, otter_sat_output)


def test_longest_simple_path():

    lsp_test_directory = CURRENT_DIR.joinpath(pathlib.Path("ipasir", "app", "genipalsp", "inputs"))
    lsp_prefix = "[X.XXX]"
    lsp_prefix_len = len(lsp_prefix)

    for cnf in glob.glob(f"{lsp_test_directory}/*.grid"):

        print(f"Checking: {cnf}")

        minisat_output = []
        otter_sat_output = []

        minisat = subprocess.run(["./genipalsp_minisat", cnf], cwd=BUILD_DIR, capture_output=True, text=True)
        for line in minisat.stdout.split('\n'):
            if len(line) > lsp_prefix_len and line[0] != 'c':
                minisat_output.append(line[lsp_prefix_len:])

        otter_sat = subprocess.run(["./genipalsp_otter_sat", cnf], cwd=BUILD_DIR, capture_output=True, text=True)
        for line in otter_sat.stdout.split('\n'):
            if len(line) > lsp_prefix_len and line[0] != 'c':
                otter_sat_output.append(line[lsp_prefix_len:])

        compare_results(cnf, 3, minisat_output, otter_sat_output)


build_binaries()

test_backbone()
test_essentials()
test_portfolio()
test_longest_simple_path()
