import subprocess
from utils import BUILD_DIR, build_binaries, is_comment

"""
This test uses harcoded expectations from buidling the satunsat with minisat.
The parameters are as suggested from examples.txt.
"""

def satunsat(x, y, z, sat, unsat):
    print(f"Checking SAT/UNSAT {x} {y} {z}")

    otter_sat_output = []
    otter_sat = subprocess.run(
        ["./ipasir-check-satunsat_otter_sat", str(x), str(y), str(z)],
        cwd=BUILD_DIR,
        capture_output=True,
        text=True,
    )
    for line in otter_sat.stdout.split("\n"):
        if not is_comment(line):
            otter_sat_output.append(line)

    result_info = otter_sat_output[-2].split(",")
    sat_info = result_info[1].split()[0]
    unsat_info = result_info[2].split()[0]

    if sat_info == str(sat) and unsat_info == str(unsat):
        print("PASS")
    else:
        print("FAIL")
        print(sat_info, str(sat))
        print(unsat_info, str(unsat))


build_binaries()

satunsat(300, 3, 300000, 2, 297)
satunsat(2000, 5, 30000, 1, 1998)
satunsat(20000, 2, 30, 8, 19991)
satunsat(200000, 2, 30, 16, 199983)
satunsat(100000, 5, 5000, 2, 99997)
satunsat(200000, 5, 5000, 2, 199997)
