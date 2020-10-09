add_executable(lisp_test  test_stdlib.c dummy.c AllTests_gen.c)
target_link_libraries(lisp_test lisp cutest)