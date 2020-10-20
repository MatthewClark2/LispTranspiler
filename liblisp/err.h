#ifndef LISP_ERR_H
#define LISP_ERR_H

enum Cause {
  ZeroDivision, Math, Generic
};

/**
 * Basic means of expounding on runtime errors. Prints cause and message to stderr, then exits program. While no value
 * is actually returned from this function, it ostensibly returns a void pointer so it can be used as `return raise()`.
 * @param msg extra details to be printed. If NULL, only the cause will be printed.
 * @return nothing, as the program exits.
 */
void* raise(enum Cause cause, const char* msg);

#endif //LISP_ERR_H
