add_library(lisp STATIC lisp.c data.c stdlisp.c)
add_executable(scratch scratch.c)
target_link_libraries(scratch lisp)

# Generally not a great installation location, but it's required to be used with Cargo.
install(TARGETS lisp DESTINATION .)

add_subdirectory(test)