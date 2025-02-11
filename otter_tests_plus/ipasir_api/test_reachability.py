import unittest
import pathlib
import re
import subprocess
from utils import BUILD_DIR, CURRENT_DIR, build_binaries


def test_reachability(solver, cnf, step):
    solve_instance = subprocess.run(
        [f"./genipareach_{solver}", cnf],
        cwd=BUILD_DIR,
        capture_output=True,
        text=True,
    )

    for line in solve_instance.stdout.split("\n"):
        m = re.search("solved at step nr. (\\d+)", line)
        if m:
            return m.group(1) == str(step)

    return False


class TestReachability(unittest.TestCase):
    solver = "otter_sat"

    cnf_dir = CURRENT_DIR.joinpath(
        pathlib.Path("ipasir", "app", "genipareach", "inputs")
    )

    def test_floortile_4_3_2(self):
        path = self.cnf_dir.joinpath("Floortile_p01-4-3-2.dimspec.cnf")
        result = test_reachability(self.solver, path, 8)
        self.assertTrue(result)

    def test_floortile_5_4_2(self):
        path = self.cnf_dir.joinpath("Floortile_p01-5-4-2.dimspec.cnf")
        result = test_reachability(self.solver, path, 13)
        self.assertTrue(result)

    def test_maintenance(self):
        path = self.cnf_dir.joinpath(
            "Maintenance_maintenance.1.3.060.180.5-002.dimspec.cnf"
        )
        result = test_reachability(self.solver, path, 1)
        self.assertTrue(result)


if __name__ == "__main__":
    build_binaries()
    unittest.main()
