add_executable(lisp_test runner.c)
target_link_libraries(lisp_test lisp cutest)