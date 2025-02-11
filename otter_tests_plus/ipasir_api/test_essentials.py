import os
import pathlib
import subprocess
import unittest

from utils import BUILD_DIR, CURRENT_DIR, build_binaries


def test_essentials(solver, cnf):
    solve_instance = subprocess.run(
        [f"./genipaessentials_{solver}", cnf],
        cwd=BUILD_DIR,
        capture_output=True,
        text=True,
    )

    essential_atoms = []

    for atom in solve_instance.stdout.splitlines()[-2].split():
        try:
            essential_atoms.append(int(atom))
        except Exception:
            return []

    return essential_atoms


class TestEssentials(unittest.TestCase):
    solver = "otter_sat"

    cnf_dir = CURRENT_DIR.joinpath(
        pathlib.Path("ipasir", "app", "genipaessentials", "inputs")
    )

    def test_sat_100(self):
        path = self.cnf_dir.joinpath("sat100.cnf")
        essential_atoms = [1, 3, 4, 6, 8, 9, 10, 11, 14, 15, 17, 18, 21, 25, 27, 28, 33, 34, 37, 38, 39, 42, 43, 48, 50, 52, 55, 56, 58, 59, 60, 65, 66, 68, 69, 70, 75, 77, 80, 81, 89, 92, 93, 94, 97, 100]

        result = test_essentials(self.solver, path)
        self.assertEqual(result, essential_atoms)

    @unittest.skipIf(int(os.getenv("TEST_LEVEL", 0)) < 1, "Expensive")
    def test_sat_250(self):
        path = self.cnf_dir.joinpath("sat250.cnf")

        essential_atoms = [1, 2, 3, 4, 6, 9, 11, 12, 13, 15, 19, 22, 25, 27, 28, 30, 31, 32, 35, 37, 40, 41, 42, 43, 46, 47, 50, 51, 52, 54, 55, 56, 57, 58, 59, 61, 62, 64, 65, 67, 68, 69, 70, 75, 76, 78, 81, 83, 84, 85, 86, 87, 88, 89, 90, 92, 93, 94, 97, 103, 107, 108, 109, 111, 112, 113, 114, 120, 121, 122, 124, 125, 126, 129, 134, 137, 138, 140, 141, 142, 144, 146, 147, 149, 150, 153, 154, 155, 158, 159, 160, 161, 162, 164, 166, 168, 171, 172, 175, 176, 178, 180, 182, 183, 185, 186, 187, 188, 189, 190, 192, 196, 197, 199, 200, 201, 202, 203, 204, 205, 207, 208, 209, 210, 211, 218, 220, 221, 222, 223, 226, 230, 232, 233, 234, 236, 237, 238, 239, 240, 241, 242, 243, 244, 245, 247, 249, 250]

        result = test_essentials(self.solver, path)
        self.assertEqual(result, essential_atoms)

    def test_unsat_250(self):
        path = self.cnf_dir.joinpath("unsat250.cnf")
        essential_atoms = []

        result = test_essentials(self.solver, path)
        self.assertEqual(result, essential_atoms)


if __name__ == "__main__":
    build_binaries()
    unittest.main()
