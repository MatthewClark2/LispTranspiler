# lispc

lispc is a Rust crate aimed at generating C code from a roughly Scheme flavored LISP. More information can be found in
the C library directory's README.

## Tests

The compilation modules all use Rust's default testing facilities. The C runtime uses the CuTest framework for unit
testing, the details of which are provided in the `cutest-1.5` directory.

## Building

The Rust portion of the project uses stable Cargo. The C portion is built with CMake. In order to compile an executable
from the output generated by `lispc`, use the `run_generated.sh` bash script. A similar Windows command line script will
probably be made in the future. For the sake of transparency, the script:

1. Calls the lispc program, and saves its output to a file called `out.c`.
2. Makes a new directory called `tmp/` in the current directory, and invokes CMake.
3. Invokes `make out`, which compiles an executable for the generated C code.
4. Executes the generated program.
5. Leaves the `tmp` directory.

If the script fails at any point, it leaves the `tmp/` directory.

## TODO(matthew-c21):

Improve documentation for the format of symbols, numbers, keywords, and hashmap literals. Also add more documentation
for the behavior of standard library functions.

Add `do`, `loop`, `let` special forms.

Modify the C runtime to use a reference counting garbage collection system. The garbage collector should probably be
passed between functions as a global-ish variable. This would make it easier to later change the codebase to accept a
general `LispState` type object.

Add another listener that finds uses of natively defined variables and replaces them with the appropriate value. C
functions should be replaced with lambda expressions that refer to them. Should be added when you get around to lambda
expressions.

Prune the AST to remove dead code (literal values outside of an s-expr). This should be the final step in visitor 
application.

Add a factory method that automatically applies visitors in the correct order.

Statically allocate all keywords that appear in the program.

There's a bug that prevents conditional statements from appearing in lambda expressions. Generated symbols are redeclared
every time they're used, meaning that there's no need to try and capture them.

There's a bug that prevents lambdas from being able to capture themselves, as the capture array is put together before
the lambda is defined. This problem could be avoided by added a `recur` form.

Get rid of all the unnecessary semicolons that appear in the generated code.

The `cons` function does not produce the correct behavior when given `nil` in place of a list.

Check Valgrind.