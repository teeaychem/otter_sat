import subprocess
from utils import BUILD_DIR, build_binaries, is_comment


def test_check_conflict():
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

        otter_sat = subprocess.run(
            ["./ipasir-check-conflict_otter_sat", *arg_list],
            cwd=BUILD_DIR,
            capture_output=True,
            text=True,
        )
        for line in otter_sat.stdout.split("\n"):
            if not is_comment(line):
                otter_sat_output.append(line)

        result = otter_sat_output[-1][:5] != "error"
        if result:
            print("\tPASS")
        else:
            print("\tFAIL")
            print(otter_sat_output[-1])


build_binaries()
test_check_conflict()
