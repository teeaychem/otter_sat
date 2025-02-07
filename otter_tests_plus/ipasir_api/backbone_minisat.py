import glob
import pathlib
import subprocess
from utils import BUILD_DIR, CURRENT_DIR, build_binaries, is_comment, compare_results


def test_backbone():
    backbone_test_directory = CURRENT_DIR.joinpath(
        pathlib.Path("ipasir", "app", "genipabones", "inputs")
    )

    for cnf in sorted(glob.glob(f"{backbone_test_directory}/*.cnf")):
        print(f"Checking: {cnf}")

        minisat_output = []
        otter_sat_output = []

        minisat = subprocess.run(
            ["./genipabones_minisat", cnf],
            cwd=BUILD_DIR,
            capture_output=True,
            text=True,
        )
        for line in minisat.stdout.split("\n"):
            if not is_comment(line):
                minisat_output.append(line)

        otter_sat = subprocess.run(
            ["./genipabones_otter_sat", cnf],
            cwd=BUILD_DIR,
            capture_output=True,
            text=True,
        )
        for line in otter_sat.stdout.split("\n"):
            if not is_comment(line):
                otter_sat_output.append(line)

        compare_results(cnf, 0, 0, minisat_output, otter_sat_output)


build_binaries()
test_backbone()
