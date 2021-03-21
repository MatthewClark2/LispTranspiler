#ifndef LISP_DATA_H
#define LISP_DATA_H

#include <stdint.h>
#include <stddef.h>
#include "stdlisp.h"


// Note(matthew-c21): This approach to typing forces specific in-built types. For what I'm doing now, that's fine, but I
//  may want to modify this in the future to accommodate user defined types and polymorphic behavior.
// TODO(matthew-c21): Expand with new types as they are added.
typedef struct LispDatum* (*LispFunction)(struct LispDatum**, uint32_t);

/** The ordering of values of numeric types is important for determining type promotion. If type a > b, then b may be
 * promoted to a. The ordering of non-numeric types is arbitrary, and should never be used for the same purpose. */
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

struct LispDatum* new_lambda(LispFunction f, struct LispDatum** captures, uint32_t n_captures, char* name);

struct LispDatum* new_cons(struct LispDatum* car, struct LispDatum* cdr);

void discard_datum(struct LispDatum* x);

// TODO(matthew-c21): I want nil to be a static constant, but I'm not sure how to deal with const-correctness.
struct LispDatum* get_nil();

void simplify(struct LispDatum* x);

int truthy(const struct LispDatum* x);

// NOTE(matthew-c21): While these functions could just be a `from_string(char*, LispDataType)`, this method avoids the
//  possibility of mis-tagged unions being generated.
struct LispDatum* new_symbol(const char* content);
struct LispDatum* new_symbol_from_copy(char* content, uint32_t length);

#endif //LISP_DATA_H
