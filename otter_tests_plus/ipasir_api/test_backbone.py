import os
import pathlib
import re
import subprocess
import unittest

from utils import BUILD_DIR, CURRENT_DIR, build_binaries


def test_backbone(solver, cnf):
    solve_instance = subprocess.run(
        [f"./genipabones_{solver}", cnf],
        cwd=BUILD_DIR,
        capture_output=True,
        text=True,
    )

    for line in solve_instance.stdout.split("\n"):
        m = re.match("c Found (\\d+) backbones", line)
        if m:
            return int(m.group(1))


class TestBackbone(unittest.TestCase):
    solver = "otter_sat"

    cnf_dir = CURRENT_DIR.joinpath(
        pathlib.Path("ipasir", "app", "genipabones", "inputs")
    )

    def test_sat_100(self):
        path = self.cnf_dir.joinpath("sat100.cnf")
        result = test_backbone(self.solver, path)
        self.assertEqual(result, 14)

    @unittest.skipIf(int(os.getenv("TEST_LEVEL", 0)) < 1, "Expensive")
    def test_sat_250(self):
        path = self.cnf_dir.joinpath("sat250.cnf")
        result = test_backbone(self.solver, path)
        self.assertEqual(result, 2)


if __name__ == "__main__":
    build_binaries()
    unittest.main()
