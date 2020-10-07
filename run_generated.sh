echo "Arguments Provided: " $*

(echo "Building Rust..."
cargo run $@ > out.c) &&
if [ ! -d tmp ]; then
	mkdir tmp
fi &&
cd tmp &&
(echo "Building C library ...";
cmake .. -DCOMPILE_GENERATED_CODE=True &&
make out &&
echo "Executing generated program ... "
./out &&
cd ..) || cd ..

