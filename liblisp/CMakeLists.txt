set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Werror -Wextra -Wall -Wpedantic")

add_library(lisp STATIC data.c stdlisp.c err.c)
add_executable(scratch scratch.c)
target_link_libraries(scratch lisp)

# Generally not a great installation location, but it's required to be used with Cargo.
install(TARGETS lisp DESTINATION .)

add_subdirectory(test)