import subprocess
from utils import BUILD_DIR, build_binaries, is_comment


def test_check_satunsat():
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

        minisat = subprocess.run(
            ["./ipasir-check-satunsat_minisat", *arg_list],
            cwd=BUILD_DIR,
            capture_output=True,
            text=True,
        )
        for line in minisat.stdout.split("\n"):
            if not is_comment(line):
                minisat_output.append(line)

        otter_sat = subprocess.run(
            ["./ipasir-check-satunsat_otter_sat", *arg_list],
            cwd=BUILD_DIR,
            capture_output=True,
            text=True,
        )
        for line in otter_sat.stdout.split("\n"):
            if not is_comment(line):
                otter_sat_output.append(line)

        result = (
            minisat_output[-1].split(",")[1:] == otter_sat_output[-1].split(",")[1:]
        )
        if result:
            print("\tPASS")
        else:
            print("\tFAIL")
            print(minisat_output[-1])
            print(otter_sat_output[-1])


build_binaries()
test_check_satunsat()
