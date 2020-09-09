cargo run > out.c &&
if [ ! -d tmp ]; then 
	mkdir tmp
fi &&
cd tmp &&
(cmake .. -DCOMPILE_GENERATED_CODE=True &&
make out &&
./out &&
cd ..) || cd ..

