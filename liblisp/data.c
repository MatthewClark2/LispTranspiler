#include <stdlib.h>
#include <string.h>
#include "data.h"
#include "err.h"

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

struct LispDatum* new_symbol(const char* content) {
  struct LispDatum* x = malloc(sizeof(struct LispDatum));
  x->type = Symbol;
  strcpy(x->content, content);
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
  switch (x->type) {
    case Integer:
    case Rational:
    case Real:
    case Complex:
      free(x);
      break;
    case String:
      free(x->content);
      free(x);
      break;
    case Symbol:
    case Keyword:
      free(x->label);
      free(x);
      break;
    case Cons:
      discard_datum(x->car);
      discard_datum(x->cdr);
      free(x);
      break;
    case Bool:
    case Nil:
      break;
    case Lambda:
      for (uint32_t i = 0; i < x->n_captures; ++i) {
        discard_datum(x->captures[i]);
      }
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

  // Check for division by 0.
  if (x->den == 0) {
    raise(ZeroDivision, "Division by 0 in simplification of rational number");
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

struct LispDatum* new_string(const char* s) {
  struct LispDatum* string = malloc(sizeof(struct LispDatum));
  string->type = String;

  size_t len = strlen(s);
  string->length = len;
  string->content = malloc(sizeof(char) * len + 1);
  strncpy(string->content, s, len);
  string->content[len] = 0;

  return string;
}

struct LispDatum* get_true() {
  static struct LispDatum true = {.type = Bool, .boolean = 1};
  return &true;
}

struct LispDatum* get_false() {
  static struct LispDatum false = {.type = Bool, .boolean = 0};
  return &false;
}

int truthy(const struct LispDatum* x) {
  return x != get_false();
}

struct LispDatum* new_keyword(const char* s) {
  struct LispDatum* keyword = new_symbol(s);
  keyword->type = Keyword;

  return keyword;
}

struct LispDatum* new_lambda(LispFunction f, struct LispDatum** captures, uint32_t n_captures, char* name) {
  struct LispDatum* lambda = malloc(sizeof(struct LispDatum));
  lambda->type = Lambda;
  lambda->f = f;

  // The only named lambdas are those created at compile time, meaning they have a statically stored name.
  lambda->name = name;

  if (captures == NULL) {
    lambda->captures = captures;
    lambda->n_captures = n_captures;
  } else {
    lambda->captures = NULL;
    lambda->n_captures = 0;
  }

  return lambda;
}
