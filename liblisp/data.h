#ifndef LISP_DATA_H
#define LISP_DATA_H

#include <stdint.h>
#include <stddef.h>
#include "stdlisp.h"


/** Function pointer specifically designed to manage LISPy calling conventions.  */
typedef struct LispDatum* (*LispFunction)(struct LispDatum**, uint32_t);

// Note(matthew-c21): This approach to typing forces specific in-built types. For what I'm doing now, that's fine, but I
//  may want to modify this in the future to accommodate user defined types and polymorphic behavior.
// TODO(matthew-c21): Expand with new types as they are added.
/**
 * The ordering of values of numeric types is important for determining type promotion. If type a > b, then b may be
 * promoted to a. The ordering of non-numeric types is arbitrary, and should never be used for the same purpose.
 */
enum LispDataType {
  Integer = 0, Rational = 1, Real = 2, Complex = 3, String, Symbol, Bool, Cons, Nil, Lambda, Keyword
};

/** Since LISP is a dynamically typed language, this struct exists as a way to produce that same behavior. */
struct LispDatum {
  enum LispDataType type;

  union {
    struct { int32_t num; int32_t den; };  // rational
    int32_t int_val; // integer
    double float_val; // real
    struct { double real; double im; };  // complex

    /** Symbols own their own strings. */
    char* label;  // symbol/keyword

    struct { char* content; size_t length; }; // strings

    int boolean;

    /** Cons cells do not make copies or transfer the ownership of the referred data. */
    struct { struct LispDatum* car; struct LispDatum* cdr; };  // cons

    /**
     * The "name" field is non-NULL only for static lambdas, meaning those associated with native C functions or those
     * created using the `defun` special form.
     *
     * The behavior regarding a mismatch between the actual size of the captures array and the n_captures count is
     * undefined.
     */
    struct { LispFunction f; struct LispDatum** captures; uint32_t n_captures; char* name; };
  };
};

// TODO(matthew-c21): Later on, these should be modified to connect to the garbage collector.
// NOTE(matthew-c21): None of these `new` functions do any kind of validation
struct LispDatum* new_integer(int32_t i);
struct LispDatum* new_real(double d);
struct LispDatum* new_rational(int32_t a, int32_t b);
struct LispDatum* new_complex(double r, double i);

struct LispDatum* get_true();
struct LispDatum* get_false();

/**
 * Strings passed to this function should be null terminated. Since all new strings are either string literals
 * (automatically terminated) or composed from terminated strings, it is safe to assume that all strings that exist at
 * runtime are null terminated. This should be tested within string functions, specifically ones like concat.
 */
struct LispDatum* new_string(const char* s);
// TODO(matthew-c21): Deprecate this function and replace it with a `keyword` function that returns interned keywords.
struct LispDatum* new_keyword(const char* s);

/**
 * Construct an anonymous function based around a static function f.
 *
 * @param captures an array containing the values captured by the lambda expression. An empty capture array should be
 *        NULL.
 * @param n_captures the size of the captures array. If captures is NULL, this value is ignored, and the resulting
 *        lambda will record having 0 captures.
 */
struct LispDatum* new_lambda(LispFunction f, struct LispDatum** captures, uint32_t n_captures, char* name);

/**
 * Constructs a new pair given car and cdr. If cdr is nil, this creates a properly terminated list only containing car.
 */
struct LispDatum* new_cons(struct LispDatum* car, struct LispDatum* cdr);

void discard_datum(struct LispDatum* x);

struct LispDatum* get_nil();

void simplify(struct LispDatum* x);

/**
 * Determine whether or not a value is truthy. As mentioned in the README, only false and nil are considered to be
 * false.
 */
int truthy(const struct LispDatum* x);

// NOTE(matthew-c21): While these functions could just be a `from_string(char*, LispDataType)`, this method avoids the
//  possibility of mis-tagged unions being generated.
struct LispDatum* new_symbol(const char* content);

#endif //LISP_DATA_H
