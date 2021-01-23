#ifndef LISP_ERR_H
#define LISP_ERR_H

// TODO(matthew-c21): Add more descriptive causes.
enum Cause {
  ZeroDivision, Math, Generic
};

// TODO(matthew-c21): Modify to return NULL. Possibly use it to trigger some global error state that needs to be primed
//  in generated code in order to quit the program.
/*
 * i.e.
 *
 * void* raise(cause, msg) { trip_error(cause, msg); return NULL: }
 *
 * void trip_error(cause, msg) { (* details omitted *) if (primed) exit(-1); }
 *
 */


/**
 * Basic means of expounding on runtime errors. Prints cause and message to stderr, then exits program. While no value
 * is actually returned from this function, it ostensibly returns a void pointer so it can be used as `return raise()`.
 * @param msg extra details to be printed. If NULL, only the cause will be printed.
 * @return nothing, as the program exits.
 */
void* raise(enum Cause cause, const char* msg);

#endif //LISP_ERR_H
