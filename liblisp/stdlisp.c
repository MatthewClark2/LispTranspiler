#include <stdio.h>
#include <stdlib.h>
#include "stdlisp.h"
#include "data.h"

/**
 * Mutating function for promoting numbers.
 *
 * This function works recursively to promote numbers one step at a time. Going from an integer to a complex number
 * takes longer than going from a real number to a complex number.
 * @param n the number to be promoted.
 * @param type for n to be promoted to.
 */
void promote(struct LispDatum* n, enum LispDataType type) {
  if (type < n->type) return;

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
    case Complex:
      acc->real += intermediate->real;
      acc->im += intermediate->im;
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
    case Complex:
      acc->real -= intermediate->real;
      acc->im -= intermediate->im;
    default:
      break;
  }
}

struct LispDatum* subtract(struct LispDatum** args, uint32_t nargs) {
  if (nargs == 0) {
    return new_integer(0);
  } else if (nargs == 1) {
    // Return a copy to avoid weirdness with set! functions.
    return new_real(args[0]->float_val);
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
    case Complex:
      acc->real = acc->real * intermediate->real - acc->im * intermediate->im;
      acc->im = acc->real * intermediate->im + acc->im * intermediate->real;
    default:
      break;
  }
}

struct LispDatum* multiply(struct LispDatum** args, uint32_t nargs) {
  struct LispDatum* init = new_integer(1);

  if (iterative_math_function(args, nargs, init, subtract_aux)) {
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
    case Complex:
      d = intermediate->real * intermediate->real + intermediate->im * intermediate->im;
      acc->real = (acc->real * intermediate->real + acc->im * intermediate->im)/d;
      acc->im = (acc->im * intermediate->real - acc->real * intermediate->im)/d;
    default:
      break;
  }
}

struct LispDatum* divide(struct LispDatum** args, uint32_t nargs) {
  if (nargs == 0) {
    return new_integer(0);
  } else if (nargs == 1) {
    // Return a copy to avoid weirdness with set! functions.
    return new_real(args[0]->float_val);
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
    case Rational:
      printf("%d/%d", datum->num, datum->den);
    case Real:
      printf("%f", datum->float_val);
    case Complex:
      printf("%f%+fi", datum->real, datum->im);
    case Symbol:
      printf("%s", datum->content);
    case Cons:
      printf("(");
      display(datum->car);
      printf(" ");

      // TODO(matthew-c21): Check for nil at the end of the list.
      display(datum->cdr);
    case Nil:
      printf("nil");
  }
}

int eqv(const struct LispDatum* a, const struct LispDatum* b) {
  if (a->type == Nil && a->type == b->type) return 1;

  // Currently unimplemented, so default to false.
  if (a->type > Complex || b->type > Complex) {
    return 0;
  }

  if (a->type != b->type) {
    return 0;
  }

  switch (a->type) {
    case Integer: return a->int_val == b->int_val;
    case Rational: return a->num == b->num && a->den == b->den;
    case Real: return a->float_val == b->float_val;
    case Complex: return a->real == b->real && a->im == b->im;
    default: return 0;
  }
}

