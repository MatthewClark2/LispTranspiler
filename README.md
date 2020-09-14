# lispc

lispc is a Rust crate aimed at generating C code from a roughly Scheme flavored LISP.

## Tests

The compilation modules all use Rust's default testing facilities. The C runtime uses the CuTest framework for unit
testing, the details of which are provided in the `cutest-1.5` directory.

## Building

The Rust portion of the project uses stable Cargo. The C portion is built with CMake. In order to compile an executable
from the output generated by `lispc`, use the `run_generated.sh` bash script. A similar Windows command line script will
probably be made in the future. For the sake of transparency, the script:

1. Calls the lispc program, and saves its output to a file called `out.c`.
2. Makes a new directory called `tmp/` in the current directory, and invokes CMake.
3. Invokes `make out`, which compiles an exectuble for the generated C code.
4. Executes the generated program.
l
5. Leaves the `tmp` directory.

If the script fails at any point, it leaves the `tmp/` directory.
