import subprocess
import unittest

from utils import BUILD_DIR, build_binaries, is_comment, printv


def check_iterative(solver, x, y, z):
    solver_output = []

    solver = subprocess.run(
        [f"./ipasir-check-iterative_{solver}", str(x), str(y), str(z)],
        cwd=BUILD_DIR,
        capture_output=True,
        text=True,
    )
    for line in solver.stdout.split("\n"):
        if not is_comment(line):
            solver_output.append(line)

    return solver_output[-2][:5] != "error"


class TestIterative(unittest.TestCase):
    solver = "otter_sat"

    def test_300_3_300000(self):
        self.assertTrue(check_iterative(self.solver, 300, 3, 300000))

    def test_2000_5_30000(self):
        self.assertTrue(check_iterative(self.solver, 2000, 5, 30000))

    def test_100000_5_5000(self):
        self.assertTrue(check_iterative(self.solver, 100000, 5, 5000))

    def test_200000_5_5000(self):
        self.assertTrue(check_iterative(self.solver, 200000, 5, 5000))


if __name__ == "__main__":
    build_binaries()
    unittest.main()
