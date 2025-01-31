import pathlib
import subprocess
import glob

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
        print('\n\t'.join(minisat))
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

        compare_results(cnf, 0, 0, minisat_output, otter_sat_output)

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

        # Skip signature, common preamble, and report of shared clauses.
        compare_results(cnf, 6, 2, minisat_output, otter_sat_output)


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

        compare_results(cnf, 2, 0, minisat_output, otter_sat_output)


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

        compare_results(cnf, 3, 0, minisat_output, otter_sat_output)


def test_reachability():

    lsp_test_directory = CURRENT_DIR.joinpath(pathlib.Path("ipasir", "app", "genipareach", "inputs"))
    lsp_prefix = ""
    lsp_prefix_len = len(lsp_prefix)

    for cnf in glob.glob(f"{lsp_test_directory}/*.cnf"):

        print(f"Checking: {cnf}")

        minisat_output = []
        otter_sat_output = []

        minisat = subprocess.run(["./genipareach_minisat", cnf], cwd=BUILD_DIR, capture_output=True, text=True)
        for line in minisat.stdout.split('\n'):
            if len(line) > lsp_prefix_len and line[0] != 'c':
                minisat_output.append(line[lsp_prefix_len:])

        otter_sat = subprocess.run(["./genipareach_otter_sat", cnf], cwd=BUILD_DIR, capture_output=True, text=True)
        for line in otter_sat.stdout.split('\n'):
            if len(line) > lsp_prefix_len and line[0] != 'c':
                otter_sat_output.append(line[lsp_prefix_len:])

        compare_results(cnf, 3, 0, minisat_output, otter_sat_output)

def test_check_satunsat():

    lsp_test_directory = CURRENT_DIR.joinpath(pathlib.Path("ipasir", "app", "ipasir-check-satunsat", "inputs"))
    lsp_prefix = ""
    lsp_prefix_len = len(lsp_prefix)

    arg_lists = [
        ["200000", "2", "30"],
        ["100000", "5", "5000"],
        ["200000", "5", "5000"],
        ["300", "3", "300000"],
        ["2000", "5", "30000"],
        ["200000", "2", "30"],
    ]

    for arg_list in arg_lists:

        print(f"Checking: {arg_list}")

        minisat_output = []
        otter_sat_output = []

        minisat = subprocess.run(["./ipasir-check-satunsat_minisat", *arg_list], cwd=BUILD_DIR, capture_output=True, text=True)
        for line in minisat.stdout.split('\n'):
            if len(line) > lsp_prefix_len and line[0] != 'c':
                minisat_output.append(line)

        otter_sat = subprocess.run(["./ipasir-check-satunsat_otter_sat", *arg_list], cwd=BUILD_DIR, capture_output=True, text=True)
        for line in otter_sat.stdout.split('\n'):
            if len(line) > lsp_prefix_len and line[0] != 'c':
                otter_sat_output.append(line)

        result = minisat_output[-1].split(",")[1:] == otter_sat_output[-1].split(",")[1:]
        if result:
            print("\tPASS")
        else:
            print("\tFAIL")
            print(minisat_output[-1])
            print(otter_sat_output[-1])

def test_check_conflict():
    """
    Non-comparitive, as the binary returns an error message.
    """

    lsp_test_directory = CURRENT_DIR.joinpath(pathlib.Path("ipasir", "app", "ipasir-check-conflict", "inputs"))
    lsp_prefix = ""
    lsp_prefix_len = len(lsp_prefix)

    arg_lists = [
        ["1", "1"],
        ["2", "2"],
        ["2", "1"],
        ["1", "2"],
        ["10000", "10002"],
        ["10002", "10000"],
        ["9999", "10002"],
        ["10002", "9999"],
    ]

    for arg_list in arg_lists:

        print(f"Checking: {arg_list}")

        otter_sat_output = []

        otter_sat = subprocess.run(["./ipasir-check-conflict_otter_sat", *arg_list], cwd=BUILD_DIR, capture_output=True, text=True)
        for line in otter_sat.stdout.split('\n'):
            if len(line) > lsp_prefix_len and line[0] != 'c':
                otter_sat_output.append(line)

        result = otter_sat_output[-1][:5] != "error"
        if result:
            print("\tPASS")
        else:
            print("\tFAIL")
            print(otter_sat_output[-1])

def test_check_iterative():
    """
    Non-comparitive, as the binary returns an error message.
    """

    lsp_test_directory = CURRENT_DIR.joinpath(pathlib.Path("ipasir", "app", "ipasir-check-iterative", "inputs"))
    lsp_prefix = ""
    lsp_prefix_len = len(lsp_prefix)

    arg_lists = [
        ["100000", "5", "5000"],
        ["200000", "5", "5000"],
        ["300", "3", "300000"],
        ["2000", "5", "30000"],
    ]

    for arg_list in arg_lists:

        print(f"Checking: {arg_list}")

        minisat_output = []
        otter_sat_output = []

        otter_sat = subprocess.run(["./ipasir-check-iterative_otter_sat", *arg_list], cwd=BUILD_DIR, capture_output=True, text=True)
        for line in otter_sat.stdout.split('\n'):
            if len(line) > lsp_prefix_len and line[0] != 'c':
                otter_sat_output.append(line)

        result = otter_sat_output[-2][:5] != "error"
        if result:
            print("\tPASS")
        else:
            print("\tFAIL")
            print(otter_sat_output[-2])



build_binaries()

test_backbone()
# test_essentials()
# test_portfolio()
# test_longest_simple_path()
# test_reachability()
# test_check_conflict()
# test_check_iterative()
# test_check_satunsat()
