import subprocess
from utils import BUILD_DIR, build_binaries, is_comment


def test_check_iterative():
    arg_lists = [
        ["300", "3", "300000"],
        ["2000", "5", "30000"],
        ["100000", "5", "5000"],
        ["200000", "5", "5000"],
    ]

    for arg_list in arg_lists:
        print(f"Checking: {arg_list}")

        otter_sat_output = []

        otter_sat = subprocess.run(
            ["./ipasir-check-iterative_otter_sat", *arg_list],
            cwd=BUILD_DIR,
            capture_output=True,
            text=True,
        )
        for line in otter_sat.stdout.split("\n"):
            if not is_comment(line):
                otter_sat_output.append(line)

        result = otter_sat_output[-2][:5] != "error"
        if result:
            print("\tPASS")
        else:
            print("\tFAIL")
            print(otter_sat_output[-2])


build_binaries()
test_check_iterative()
