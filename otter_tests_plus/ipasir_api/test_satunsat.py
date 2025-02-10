import subprocess
import unittest

from utils import BUILD_DIR, build_binaries, is_comment, printv

"""
This test uses harcoded expectations from buidling the satunsat with minisat.
The parameters are as suggested from examples.txt.
"""


def check_sat_unsat(solver, x, y, z, sat, unsat):
    solver_output = []
    solver = subprocess.run(
        [f"./ipasir-check-satunsat_{solver}", str(x), str(y), str(z)],
        cwd=BUILD_DIR,
        capture_output=True,
        text=True,
    )
    for line in solver.stdout.split("\n"):
        if not is_comment(line):
            solver_output.append(line)

    result_info = solver_output[-2].split(",")
    sat_info = result_info[1].split()[0]
    unsat_info = result_info[2].split()[0]

    return sat_info == str(sat) and unsat_info == str(unsat)


class TestSatUnsat(unittest.TestCase):
    solver = "otter_sat"

    def test_a(self):
        self.assertTrue(check_sat_unsat(self.solver, 300, 3, 300000, 2, 297))

    def test_b(self):
        self.assertTrue(check_sat_unsat(self.solver, 2000, 5, 30000, 1, 1998))

    def test_c(self):
        self.assertTrue(check_sat_unsat(self.solver, 20000, 2, 30, 8, 19991))

    def test_d(self):
        self.assertTrue(check_sat_unsat(self.solver, 200000, 2, 30, 16, 199983))

    def test_e(self):
        self.assertTrue(check_sat_unsat(self.solver, 100000, 5, 5000, 2, 99997))

    def test_f(self):
        self.assertTrue(check_sat_unsat(self.solver, 200000, 5, 5000, 2, 199997))


if __name__ == "__main__":
    build_binaries()
    unittest.main()
