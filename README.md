# lispc

lispc is a Rust crate aimed at generating C code from a roughly Scheme flavored LISP.

## Tests

The compilation modules all use Rust's default testing facilities. The C runtime uses the CuTest framework for unit
testing, the details of which are provided in the `cutest-1.5` directory.

## Building

The Rust portion of the project uses stable Cargo. The C portion is built with CMake.
