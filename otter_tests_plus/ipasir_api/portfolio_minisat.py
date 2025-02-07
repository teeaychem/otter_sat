import glob
import pathlib
import subprocess
from utils import BUILD_DIR, CURRENT_DIR, build_binaries, compare_results


def test_portfolio():
    portfolio_test_directory = CURRENT_DIR.joinpath(
        pathlib.Path("ipasir", "app", "genipafolio", "inputs")
    )
    stdout_prefix = "c [genipafolio]"
    stdout_prefix_len = len(stdout_prefix)

    for cnf in glob.glob(f"{portfolio_test_directory}/*.cnf"):
        minisat_output = []
        otter_sat_output = []

        minisat = subprocess.run(
            ["./genipafolio_minisat", cnf],
            cwd=BUILD_DIR,
            capture_output=True,
            text=True,
        )
        for line in minisat.stdout.split("\n"):
            if (
                len(line) > stdout_prefix_len
                and line[:stdout_prefix_len] == stdout_prefix
            ):
                minisat_output.append(line)

        otter_sat = subprocess.run(
            ["./genipafolio_otter_sat", cnf],
            cwd=BUILD_DIR,
            capture_output=True,
            text=True,
        )
        for line in otter_sat.stdout.split("\n"):
            if (
                len(line) > stdout_prefix_len
                and line[:stdout_prefix_len] == stdout_prefix
            ):
                otter_sat_output.append(line)

        compare_results(cnf, 2, 0, minisat_output, otter_sat_output)


build_binaries()
test_portfolio()
