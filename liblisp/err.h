#ifndef LISP_ERR_H
#define LISP_ERR_H

enum Cause {
  None = 0, Type, Argument, ZeroDivision, Math, Generic
};

enum ErrorBehavior {
  /**
   * Primarily used for debugging purposes. Behavior of the program when encountering an error in this state is
   * undefined.
   */
  LogOnly,

  /**
   * Standard behavior that logs an error to `stderr` and quits.
   */
  LogAndQuit
};

/**
 * Error state value. This value should be treated as readonly by client code.
 */
enum Cause GlobalErrorState;

/**
 * Basic means of expounding on runtime errors. Prints cause and message to stderr. The behavior after this point is
 * based on what the currently set error behavior is. Use `return raise(...)` in the same place you would use `raise E`.
 * The behavior is quite a bit different, but it establishes the same point. This function can also double as a logging
 * tool if no cause is given.
 * @param cause the reason for the raise. If `None`, no exit will occur, but details will still be printed to `stderr`.
 * @param msg extra details to be printed. If NULL, only the cause will be printed.
 * @return NULL if the global error behavior is set to log only. This method does not return otherwise.
 */
void* raise(enum Cause cause, const char* msg);

/**
 * Change what happens when an error is raised. Mostly used for debugging purposes.
 */
void set_global_error_behavior(enum ErrorBehavior behavior);

#endif //LISP_ERR_H
