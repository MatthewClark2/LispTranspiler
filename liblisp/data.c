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
