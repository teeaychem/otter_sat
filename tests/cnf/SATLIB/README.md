All formulas in this directory are SATLIB benchmark problems from [https://www.cs.ubc.ca/~hoos/SATLIB/benchm.html](https://www.cs.ubc.ca/~hoos/SATLIB/benchm.html)

The structure is rearranged a little, though html descriptions from the SATLIB page have been cloned and then placed where relevant.

Changes made:

- DIMACS/DUBOIS/dubois100.cnf
  - Terminating 0's have been added if a line broken to a new clause without a delimiter.
    - …as the library does not assume a new line terminates a clause
- DIMACS/PRET/*.cnf
  - The final 0 from each formula has been removed
    - …as the library assumes an empty clause is equivalent to ⊥
