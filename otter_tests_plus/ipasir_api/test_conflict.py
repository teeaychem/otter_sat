import subprocess
import unittest

from utils import BUILD_DIR, build_binaries, is_comment


def check_conflict(solver, x, y):
    solver_output = []

    solver = subprocess.run(
        [f"./ipasir-check-conflict_{solver}", str(x), str(y)],
        cwd=BUILD_DIR,
        capture_output=True,
        text=True,
    )
    for line in solver.stdout.split("\n"):
        if not is_comment(line):
            solver_output.append(line)

    return solver_output[-1][:5] != "error"


class TestConflict(unittest.TestCase):
    solver = "otter_sat"

    def test_small(self):
        self.assertTrue(check_conflict(self.solver, 1, 1))
        self.assertTrue(check_conflict(self.solver, 2, 1))
        self.assertTrue(check_conflict(self.solver, 1, 2))
        self.assertTrue(check_conflict(self.solver, 2, 2))

    def test_large(self):
        self.assertTrue(check_conflict(self.solver, 10000, 10002))
        self.assertTrue(check_conflict(self.solver, 10002, 10000))
        self.assertTrue(check_conflict(self.solver, 9999, 10002))
        self.assertTrue(check_conflict(self.solver, 10002, 9999))


if __name__ == "__main__":
    build_binaries()
    unittest.main()
