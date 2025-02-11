import os
import pathlib
import re
import subprocess
import unittest

from utils import BUILD_DIR, CURRENT_DIR, build_binaries, is_comment


def test_lsp(solver, cnf):
    solve_instance = subprocess.run(
        [f"./genipalsp_{solver}", cnf],
        cwd=BUILD_DIR,
        capture_output=True,
        text=True,
    )

    for line in solve_instance.stdout.split("\n"):
        if not is_comment(line):
            m = re.search("Longest path has (\\d+) vertices", line)
            if m:
                return int(m.group(1))


class TestLSP(unittest.TestCase):
    solver = "minisat"

    cnf_dir = CURRENT_DIR.joinpath(pathlib.Path("ipasir", "app", "genipalsp", "inputs"))

    def test_maze_10(self):
        result = test_lsp(self.solver, self.cnf_dir.joinpath("maze-10-0.5-8.grid"))
        self.assertEqual(result, 25)

    @unittest.skipIf(int(os.getenv("TEST_LEVEL", 0)) < 1, "Expensive")
    def test_maze_12(self):
        result = test_lsp(self.solver, self.cnf_dir.joinpath("maze-12-0.3-1.grid"))
        self.assertEqual(result, 71)

    @unittest.skipIf(int(os.getenv("TEST_LEVEL", 0)) < 1, "Expensive")
    def test_maze_13(self):
        result = test_lsp(self.solver, self.cnf_dir.joinpath("maze-13-0.3-1.grid"))
        self.assertEqual(result, 81)


if __name__ == "__main__":
    build_binaries()
    unittest.main()
