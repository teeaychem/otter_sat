import glob
import pathlib
import subprocess
from utils import BUILD_DIR, CURRENT_DIR, compare_results, build_binaries


def test_longest_simple_path():
    lsp_test_directory = CURRENT_DIR.joinpath(
        pathlib.Path("ipasir", "app", "genipalsp", "inputs")
    )
    lsp_prefix = "[X.XXX]"
    lsp_prefix_len = len(lsp_prefix)

    for cnf in sorted(glob.glob(f"{lsp_test_directory}/*.grid")):
        print(f"Checking: {cnf}")

        minisat_output = []
        otter_sat_output = []

        minisat = subprocess.run(
            ["./genipalsp_minisat", cnf], cwd=BUILD_DIR, capture_output=True, text=True
        )
        for line in minisat.stdout.split("\n"):
            if len(line) > lsp_prefix_len and line[0] != "c":
                minisat_output.append(line[lsp_prefix_len:])

        otter_sat = subprocess.run(
            ["./genipalsp_otter_sat", cnf],
            cwd=BUILD_DIR,
            capture_output=True,
            text=True,
        )
        for line in otter_sat.stdout.split("\n"):
            if len(line) > lsp_prefix_len and line[0] != "c":
                otter_sat_output.append(line[lsp_prefix_len:])

        compare_results(cnf, 3, 0, minisat_output, otter_sat_output)


build_binaries()
test_longest_simple_path()
