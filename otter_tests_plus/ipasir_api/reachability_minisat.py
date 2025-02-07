import glob
import pathlib
import subprocess
from utils import BUILD_DIR, CURRENT_DIR, compare_results, build_binaries, is_comment


def test_reachability():
    lsp_test_directory = CURRENT_DIR.joinpath(
        pathlib.Path("ipasir", "app", "genipareach", "inputs")
    )

    for cnf in glob.glob(f"{lsp_test_directory}/*.cnf"):
        print(f"Checking: {cnf}")

        minisat_output = []
        otter_sat_output = []

        minisat = subprocess.run(
            ["./genipareach_minisat", cnf],
            cwd=BUILD_DIR,
            capture_output=True,
            text=True,
        )
        for line in minisat.stdout.split("\n"):
            if not is_comment(line):
                minisat_output.append(line)

        otter_sat = subprocess.run(
            ["./genipareach_otter_sat", cnf],
            cwd=BUILD_DIR,
            capture_output=True,
            text=True,
        )
        for line in otter_sat.stdout.split("\n"):
            if not is_comment(line):
                otter_sat_output.append(line)

        compare_results(cnf, 3, 0, minisat_output, otter_sat_output)


build_binaries()
test_reachability()
