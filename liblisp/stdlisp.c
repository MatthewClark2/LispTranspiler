#include <stdio.h>
#include <stdlib.h>
#include "stdlisp.h"
#include "data.h"

// TODO(matthew-c21): Rather than manually set values and types, have an assistive function than can safely do the same
//  thing and automatically perform simplification and other tasks.

/**
 * Mutating function for promoting numbers.
 *
 * This function works recursively to promote numbers one step at a time. Going from an integer to a complex number
 * takes longer than going from a real number to a complex number.
 * @param n the number to be promoted.
 * @param type for n to be promoted to.
 */
void promote(struct LispDatum* n, enum LispDataType type) {
  if (type <= n->type) return;

  switch (n->type) {
    case Integer:
      n->type = Rational;
      n->num = n->int_val;
      n->den = 1;
      break;
    case Rational:
      n->type = Real;
      n->float_val = ((double) n->num) / (n->den);
      break;
    case Real:
      n->type = Complex;
      n->real = n->float_val;
      n->im = 0;
      break;
    default:
      break;
  }

  promote(n, type);
}

/**
 * Perform a shallow copy
 * @param source
 * @param dest
 */
void copy_lisp_datum(const struct LispDatum* source, struct LispDatum* dest) {
  dest->type = source->type;

  switch (source->type) {
    case Integer:
      dest->int_val = source->int_val;
      break;
    case Rational:
      dest->num = source->num;
      dest->den = source->den;
      break;
    case Real:
      dest->float_val = source->float_val;
      break;
    case Complex:
      dest->real = source->real;
      dest->im = source->im;
      break;
    case Symbol:
      dest->content = source->content;
      break;
    case Cons:
      dest->car = source->car;
      dest->cdr = source->cdr;
      break;
    case Nil:
      *dest = *get_nil();
      break;
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

  int g = gcd(x->num, x->den);
  if (g != 1) {
    x->num /= g;
    x->den /= g;
  }
}

/**
 * Fold a function f over the given arguments.
 *
 * This is a utility function aimed at reducing boilerplate for numeric functions that can be evaluated by folding a two
 * argument function across a list of numbers. This function fails immediately if any non-numeric parameters are found.
 * @param args argument list provided to the original function
 * @param nargs number of arguments provided
 * @param acc initial value passed in as the first argument to f.
 * @param f a function pointer that takes two numbers of the same type. The first value should be treated as both an
 * input and output parameter.
 * @return 0 if no errors occur, else -1.
 */
int iterative_math_function(struct LispDatum** args, uint32_t nargs, struct LispDatum* acc,
                            void (* f)(struct LispDatum*, const struct LispDatum*)) {
  if (acc->type > Complex) {
    return -1;
  }

  struct LispDatum intermediate;

  for (uint32_t i = 0; i < nargs; ++i) {
    // Because it's easier to promote mutably and we don't want to modify the incoming arguments, we copy it to the
    //  intermediate beforehand.
    copy_lisp_datum(args[i], &intermediate);

    if (intermediate.type > Complex) {
      return -1;
    }

    // Ensure both values are of the same type.
    if (intermediate.type > acc->type) {
      promote(acc, intermediate.type);
    } else if (intermediate.type < acc->type) {
      promote(&intermediate, acc->type);
    }

    f(acc, &intermediate);
    simplify(acc);
  }

  return 0;
}

void add_aux(struct LispDatum* acc, const struct LispDatum* intermediate) {
  switch (acc->type) {
    case Integer:
      acc->int_val += intermediate->int_val;
      break;
    case Rational:
      acc->num = intermediate->den * acc->num + acc->den + intermediate->num;
      acc->den = intermediate->den * acc->den;
      break;
    case Real:
      acc->float_val += intermediate->float_val;
      break;
    case Complex:
      acc->real += intermediate->real;
      acc->im += intermediate->im;
      break;
    default:
      break;
  }
}

struct LispDatum* add(struct LispDatum** args, uint32_t nargs) {
  struct LispDatum* init = new_integer(0);

  if (iterative_math_function(args, nargs, init, add_aux)) {
    return NULL;
  }

  return init;
}

void subtract_aux(struct LispDatum* acc, const struct LispDatum* intermediate) {
  switch (acc->type) {
    case Integer:
      acc->int_val -= intermediate->int_val;
      break;
    case Rational:
      acc->num = intermediate->den * acc->num - acc->den + intermediate->num;
      acc->den = intermediate->den * acc->den;
      break;
    case Real:
      acc->float_val -= intermediate->float_val;
      break;
    case Complex:
      acc->real -= intermediate->real;
      acc->im -= intermediate->im;
      break;
    default:
      break;
  }
}

struct LispDatum* subtract(struct LispDatum** args, uint32_t nargs) {
  if (nargs == 0) {
    return new_integer(0);
  } else if (nargs == 1) {
    struct LispDatum* negation = malloc(sizeof(struct LispDatum));
    struct LispDatum* negative_1 = new_integer(-1);
    struct LispDatum* x[2];
    args[0] = negative_1;
    args[1] = negation;
    return multiply(x, 2);
  }

  struct LispDatum* init = malloc(sizeof(struct LispDatum));
  copy_lisp_datum(args[0], init);

  if (iterative_math_function(args, nargs, init, subtract_aux)) {
    free(init);
    return NULL;
  }

  return init;
}

void multiply_aux(struct LispDatum* acc, const struct LispDatum* intermediate) {
  switch (acc->type) {
    case Integer:
      acc->int_val *= intermediate->int_val;
      break;
    case Rational:
      acc->num *= intermediate->num;
      acc->den *= intermediate->den;
      break;
    case Real:
      acc->float_val *= intermediate->float_val;
      break;
    case Complex:
      acc->real = acc->real * intermediate->real - acc->im * intermediate->im;
      acc->im = acc->real * intermediate->im + acc->im * intermediate->real;
      break;
    default:
      break;
  }
}

struct LispDatum* multiply(struct LispDatum** args, uint32_t nargs) {
  struct LispDatum* init = new_integer(1);

  if (iterative_math_function(args, nargs, init, multiply_aux)) {
    free(init);
    return NULL;
  }

  return init;
}

void divide_aux(struct LispDatum* acc, const struct LispDatum* intermediate) {
  double d;
  switch (acc->type) {
    case Integer:
      if (acc->int_val % intermediate->int_val == 0) {
        acc->int_val /= intermediate->int_val;
      } else {
        acc->type = Real;
        acc->float_val = ((double) acc->int_val) / (intermediate->int_val);
      }
      break;
    case Rational:
      acc->num *= intermediate->den;
      acc->den *= intermediate->num;
      break;
    case Real:
      acc->float_val /= intermediate->float_val;
      break;
    case Complex:
      d = intermediate->real * intermediate->real + intermediate->im * intermediate->im;
      acc->real = (acc->real * intermediate->real + acc->im * intermediate->im) / d;
      acc->im = (acc->im * intermediate->real - acc->real * intermediate->im) / d;
      break;
    default:
      break;
  }
}

struct LispDatum* divide(struct LispDatum** args, uint32_t nargs) {
  if (nargs == 0) {
    return new_integer(0);
  } else if (nargs == 1) {
    return args[0];
  }

  struct LispDatum* init = malloc(sizeof(struct LispDatum));
  copy_lisp_datum(args[0], init);

  if (iterative_math_function(args, nargs, init, divide_aux)) {
    free(init);
    return NULL;
  }

  return init;
}

struct LispDatum* mod(struct LispDatum** args, uint32_t nargs) {
  if (nargs != 2) {
    return NULL;
  }

  if (args[0]->type != Integer || args[1]->type != Integer) {
    return NULL;
  }

  return new_integer(args[0]->int_val % args[1]->int_val);
}

struct LispDatum* division(struct LispDatum** args, uint32_t nargs) {
  if (nargs != 2) {
    return NULL;
  }

  if (args[0]->type != Integer || args[1]->type != Integer) {
    return NULL;
  }

  struct LispDatum* d = new_integer(args[0]->int_val / args[1]->int_val);
  struct LispDatum* r = new_integer(args[0]->int_val % args[1]->int_val);
  return new_cons(r, new_cons(d, get_nil()));
}

void display(const struct LispDatum* datum) {
  switch (datum->type) {
    case Integer:
      printf("%d", datum->int_val);
      break;
    case Rational:
      printf("%d/%d", datum->num, datum->den);
      break;
    case Real:
      printf("%f", datum->float_val);
      break;
    case Complex:
      printf("%f%+fi", datum->real, datum->im);
      break;
    case Symbol:
      printf("%s", datum->content);
      break;
    case Cons:
      printf("(");
      display(datum->car);
      printf(" ");

      // TODO(matthew-c21): Check for nil at the end of the list.
      display(datum->cdr);
      break;
    case Nil:
      printf("nil");
      break;
  }
}

int is_numeric(const struct LispDatum* x) {
  return x->type <= Complex;
}

int eqv(const struct LispDatum* a, const struct LispDatum* b) {
  if (a->type == Nil && a->type == b->type) return 1;

  if (is_numeric(a) && is_numeric(b)) {
    // Make copies in order to promote. Only one copy is required, but two saves some duplicate code.
    struct LispDatum x;
    struct LispDatum y;
    copy_lisp_datum(a, &x);
    copy_lisp_datum(b, &y);

    enum LispDataType max_type = a->type > b->type ? a->type : b->type;
    promote(&x, max_type);
    promote(&y, max_type);

    switch (x.type) {
      case Integer:
        return x.int_val == y.int_val;
      case Rational:
        return x.num == y.num && x.den == y.den;
      case Real:
        return x.float_val == y.float_val;
      case Complex:
        return x.real == y.real && x.im == y.im;
        // TODO(matthew-c21): For completeness sake, this should raise an error as it is theoretically unreachable.
      default:
        return 0;
    }
  }

  // Non-numeric type equality.

  // Only numeric types are implemented, so just return false.
  return 0;
}

struct LispDatum* format(struct LispDatum** args, uint32_t nargs) {
  for (uint32_t i = 0; i < nargs; ++i) {
    display(args[i]);
    printf("\n");
  }

  return get_nil();
}

