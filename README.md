# libronda

This project was created as a way to speed up conda's low-level operations. 
It was intended to cover graph computations. It hasn't gotten that far. Right now,
it builds a rust binary that can be used with a python wrapper. It parses version 
strings, compares versions, and to some extent, evaluates version constraints.

The code should compile, but the test suite does not pass. Some of the test failures
are because the implementation isn't there yet. Other test failures are because of subtle
behavior differences between conda's comparison code and the code here. If this project ever
gets good enough to really consider using, we may want to revisit the version parsing/comparison 
rules.
