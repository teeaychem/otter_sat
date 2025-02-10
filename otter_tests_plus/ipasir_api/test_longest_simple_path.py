import unittest
import pathlib
import re
import subprocess
from utils import BUILD_DIR, CURRENT_DIR, build_binaries, is_comment


def test_lsp(solver, cnf, expected_vertex_count):
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
                return m.group(1) == str(expected_vertex_count)

    return False


class TestReachability(unittest.TestCase):
    solver = "otter_sat"
    build_binaries()

    lsp_dir = CURRENT_DIR.joinpath(pathlib.Path("ipasir", "app", "genipalsp", "inputs"))

    def test_maze_10(self):
        result = test_lsp(self.solver, self.lsp_dir.joinpath("maze-10-0.5-8.grid"), 25)
        self.assertTrue(result)

    def test_maze_12(self):
        result = test_lsp(self.solver, self.lsp_dir.joinpath("maze-12-0.3-1.grid"), 71)
        self.assertTrue(result)

    def test_maze_13(self):
        result = test_lsp(self.solver, self.lsp_dir.joinpath("maze-13-0.3-1.grid"), 81)
        self.assertTrue(result)


if __name__ == "__main__":
    build_binaries()
    unittest.main()
