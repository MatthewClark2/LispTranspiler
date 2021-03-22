#ifndef LISP_STDLISP_H
#define LISP_STDLISP_H

#include "data.h"

/**
 * Sum all values provided.
 *
 * If no arguments are supplied, return 0. If non-numeric arguments are supplied, raise an error.
 */
struct LispDatum* add(struct LispDatum** args, uint32_t nargs);

/**
 * Subtract the 2nd, 3rd, etc., arguments from the first.
 *
 * If no arguments are supplied, return 0. If one argument is supplied, return the same value. If non-numeric arguments
 * are supplied, raise an error.
 */
struct LispDatum* subtract(struct LispDatum** args, uint32_t nargs);

/**
 * Multiply all provided arguments.
 *
 * If no arguments are supplied, return 1. If non-numeric arguments are supplied, raise an error.
 */
struct LispDatum* multiply(struct LispDatum** args, uint32_t nargs);

/**
 * Divide the first argument by all subsequent arguments.
 *
 * If no arguments are supplied, return 0. If only one argument is supplied, return the same value. If non-numeric
 * arguments are supplied, raise an error.
 */
struct LispDatum* divide(struct LispDatum** args, uint32_t nargs);

/**
 * Given integers a and b, return the smallest integer m such that a = b(mod m).
 *
 * Takes exactly two integer arguments. If anything else is provided, raise an error.
 */
struct LispDatum* mod(struct LispDatum** args, uint32_t nargs);

/**
 * Given integers a and b, return a nil terminated list containing two numbers x and y such that y<a and a = bx + y.
 *
 * Takes exactly two integer arguments. If anything else is provided, raise an error.
 */
struct LispDatum* division(struct LispDatum** args, uint32_t nargs);

struct LispDatum* format(struct LispDatum** args, uint32_t nargs);

void display(struct LispDatum* datum);

int datum_cmp(const struct LispDatum* a, const struct LispDatum* b);

/**
 * Determines if two objects are strictly equal.
 *
 * This is equivalent to the eqv? predicate found in Scheme. See the R7RS spec for more information.
 */
struct LispDatum* eqv(struct LispDatum** args, uint32_t nargs);

// Comparative functions and logical manipulation
// TODO(matthew-c21): Most comparators can be discarded once user generated functions are in order.
struct LispDatum* less_than(struct LispDatum** args, uint32_t nargs);
struct LispDatum* num_equals(struct LispDatum** args, uint32_t nargs);
struct LispDatum* greater_than(struct LispDatum** args, uint32_t nargs);
struct LispDatum* less_than_eql(struct LispDatum** args, uint32_t nargs);
struct LispDatum* greater_than_eql(struct LispDatum** args, uint32_t nargs);

/**
 * Returns false if any of the non-terminal elements are false. Otherwise returns the final element.
 */
struct LispDatum* logical_and(struct LispDatum** args, uint32_t nargs);

/**
 * Returns the first non-falsy argument if one exists. If none exist, returns false.
 */
struct LispDatum* logical_or(struct LispDatum** args, uint32_t nargs);

/**
 * Takes a single argument, and returns its logical inverse.
 */
struct LispDatum* logical_not(struct LispDatum** args, uint32_t nargs);

// LIST FUNCTIONS

/**
 * Obtain the first element of a list. Fails if the argument is not a list, or if it is empty as the first element of an
 * empty list is not defined.
 */
struct LispDatum* car(struct LispDatum** args, uint32_t nargs);

/**
 * Obtain the linked child nodes in a list. When used on an empty list or nil, returns an empty list. When used on an
 * improper list, e.g. `(cdr '(a . b))`, returns the second item - in this case the symbol 'b'.
 */
struct LispDatum* cdr(struct LispDatum** args, uint32_t nargs);

/**
 * Obtains the length of a proper list or string. Fails on other types, or when not receiving exactly one argument.
 * @throws Type error if given improper arguments.
 * @throws Argument error if not given exactly one argument
 */
struct LispDatum* length(struct LispDatum** args, uint32_t nargs);

/**
 * Wrapper for the new_cons factory function. Takes exactly two arguments, and constructs a new cons cell out of them.
 */
struct LispDatum* cons(struct LispDatum** args, uint32_t nargs);

/**
 * Creates a linked list structure using the provided arguments. If `nargs` is 0, then the args array is not evaluated
 * at all, meaning that `list(NULL, 0)` always returns a valid, empty list.
 */
struct LispDatum* list(struct LispDatum** args, uint32_t nargs);

/**
 * Combines multiple lists together. Only joins top level lists. Returns an empty list if no lists are provided. Returns
 * the given list if only one list is provided. This has an interesting quirk that `(append nil)` returns nil instead of
 * an empty list. It will fail if given a single argument of any other type, however.
 *
 * Example: (append (1 2) (3 4)) ==> (1 2 3 4)
 */
struct LispDatum* append(struct LispDatum** args, uint32_t nargs);

/**
 * Constructs a list in reverse order. This items referred to in the new reversed list are the same as in the
 * non-reversed list. For a list too short to reverse (nil, empty, single element), the original argument is returned.
 *
 * @throws Argument exception if anything other than one argument is passed.
 * @throws Type exception if the argument is neither a proper list nor nil.
 */
struct LispDatum* reverse(struct LispDatum** args, uint32_t nargs);

// TODO(matthew-c21): Test these functions. They are very important.
/**
 * Takes a function and a list, applying the list as arguments to the function, and returning the result.
 *
 * The following lines are (roughly) equivalent:
 *      `(apply + (list 1 2))`
 *      `(+ 1 2)`
 */
struct LispDatum* apply(struct LispDatum** args, uint32_t nargs);

/**
 * Variadic means of calling a function. Similar to `apply`, but takes any number of arguments bypassing the need to
 * pass arguments via list.
 *
 * The following two lines are effectively equivalent:
 *      `(funcall (lambda (x y) (* x y)) 1 2)`
 *      `(apply (lambda (x y) (* x y)) (list 1 2))`
 *
 * The biggest difference is that funcall doesn't require the memory overhead of creating a list, while apply can take
 * lists made at runtime, making it more flexible.
 */
struct LispDatum* funcall(struct LispDatum** args, uint32_t nargs);

#endif //LISP_STDLISP_H
