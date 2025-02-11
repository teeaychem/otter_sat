import os
import pathlib
import subprocess
import unittest

from utils import BUILD_DIR, CURRENT_DIR, build_binaries


def test_portfolio(solver, cnf):
    solve_instance = subprocess.run(
        [f"./genipafolio_{solver}", cnf],
        cwd=BUILD_DIR,
        capture_output=True,
        text=True,
    )

    return solve_instance.returncode


class TestPortfolio(unittest.TestCase):
    solver = "otter_sat"

    cnf_dir = CURRENT_DIR.joinpath(
        pathlib.Path("ipasir", "app", "genipafolio", "inputs")
    )

    def test_sat_100(self):
        path = self.cnf_dir.joinpath("sat100.cnf")
        result = test_portfolio(self.solver, path)
        self.assertEqual(result, 10)

    def test_sat_250(self):
        path = self.cnf_dir.joinpath("sat250.cnf")
        result = test_portfolio(self.solver, path)
        self.assertEqual(result, 10)

    @unittest.skipIf(int(os.getenv("TEST_LEVEL", 0)) < 1, "Expensive")
    def test_unsat_250(self):
        path = self.cnf_dir.joinpath("unsat250.cnf")
        result = test_portfolio(self.solver, path)
        self.assertEqual(result, 20)


if __name__ == "__main__":
    build_binaries()
    unittest.main()
