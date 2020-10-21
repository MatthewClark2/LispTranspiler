cmake_minimum_required(VERSION 3.10)
project(lisp C)

set(CMAKE_C_STANDARD 11)

add_subdirectory(cutest-1.5)
include_directories(cutest-1.5)
add_subdirectory(liblisp)

if (COMPILE_GENERATED_CODE)
    add_executable(out out.c)
    include_directories(liblisp)
    target_link_libraries(out lisp)
endif()
