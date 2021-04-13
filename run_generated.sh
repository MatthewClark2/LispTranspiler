echo "Arguments Provided: " $*

(echo "Building Rust..."
cargo run $@ | clang-format > out.c) &&
if [ ! -d tmp ]; then
	mkdir tmp
fi &&
cd tmp &&
(echo "Building C library ...";
cmake .. -DCMAKE_BUILD_TYPE=Debug -DCOMPILE_GENERATED_CODE=True &&
make out &&
echo "Executing generated program ... "
./out &&
cd ..) || cd ..

