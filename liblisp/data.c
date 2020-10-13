#include <stdlib.h>
#include <string.h>
#include "data.h"

struct LispDatum* new_integer(int32_t i) {
  struct LispDatum* x = malloc(sizeof(struct LispDatum));
  x->type = Integer;
  x->int_val = i;
  return x;
}

struct LispDatum* new_real(double d) {
  struct LispDatum* x = malloc(sizeof(struct LispDatum));
  x->type = Real;
  x->float_val = d;
  return x;
}

struct LispDatum* new_rational(int32_t a, int32_t b) {
  struct LispDatum* x = malloc(sizeof(struct LispDatum));
  x->type = Rational;
  x->num = a;
  x->den = b;
  simplify(x);
  return x;
}

struct LispDatum* new_complex(double r, double i) {
  struct LispDatum* x = malloc(sizeof(struct LispDatum));
  x->type = Complex;
  x->real = r;
  x->im = i;
  return x;
}

struct LispDatum* new_symbol(char* content) {
  struct LispDatum* x = malloc(sizeof(struct LispDatum));
  x->type = Symbol;
  x->content = content;
  return x;
}

struct LispDatum* new_symbol_from_copy(char* content, uint32_t length) {
  struct LispDatum* x = malloc(sizeof(struct LispDatum));
  x->type = Symbol;
  x->content = malloc(length);

  strncpy(x->content, content, length);

  return x;
}

// TODO(matthew-c21): Whenever garbage collection is implemented, this should update the reference count.
struct LispDatum* new_cons(struct LispDatum* car, struct LispDatum* cdr) {
  struct LispDatum* x = malloc(sizeof(struct LispDatum));
  x->type = Cons;
  x->car = car;
  x->cdr = cdr;
  return x;
}

struct LispDatum* get_nil() {
  // Essentially, what this does is create a single instance of NIL which is then shared
  static struct LispDatum x = {.type =  Nil, .int_val = 0};
  return &x;
}

// TODO(matthew-c21): Update for garbage collection later on.
void discard_datum(struct LispDatum* x) {
  if (x->type != Nil) {
    free(x);
  }
}

/**
 * Euclid's GCD algorithm, directly copied from [this answer](https://stackoverflow.com/a/19738969).
 */
int gcd(int a, int b) {
  int temp;
  while (b != 0) {
    temp = a % b;

    a = b;
    b = temp;
  }
  return a;
}

/**
 * Reduce a reducible LispDatum (rational,).
 * @param x the value to be simplified.
 */
void simplify(struct LispDatum* x) {
  if (x->type != Rational) {
    return;
  }

  // Reduce
  int g = gcd(x->num, x->den);
  if (g != 1) {
    x->num /= g;
    x->den /= g;
  }

  // Ensure the numerator contains the sign.
  if (x->den < 0) {
    x->num *= -1;
    x->den *= -1;
  }
}
