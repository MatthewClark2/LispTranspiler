cmake_minimum_required(VERSION 3.10)
project(lisp C)

set(CMAKE_C_STANDARD 99)
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Werror -Wextra -Wall -Wpedantic")

add_library(lisp STATIC lisp.h lisp.c)

# Generally not a great installation location, but it's required to be used with Cargo.
install(TARGETS lisp DESTINATION .)