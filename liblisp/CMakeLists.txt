cmake_minimum_required(VERSION 3.10)
project(lisp C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Werror -Wextra -Wall -Wpedantic")

add_library(lisp STATIC lisp.c data.c stdlisp.c)
add_executable(scratch scratch.c)
target_link_libraries(scratch lisp)

# Generally not a great installation location, but it's required to be used with Cargo.
install(TARGETS lisp DESTINATION .)