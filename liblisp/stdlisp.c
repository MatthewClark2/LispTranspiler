#include <stdio.h>
#include <stdlib.h>
#include "stdlisp.h"
#include "data.h"
#include "err.h"

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
      dest->label = source->label;
      break;
    case Cons:
      dest->car = source->car;
      dest->cdr = source->cdr;
      break;
    case Nil:
      *dest = *get_nil();
      break;
    case String:
      dest->content = source->content;
      dest->length = source->length;
      break;
  }
}

void write_zero(struct LispDatum* x) {
  x->type = Integer;
  x->int_val = 0;
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
      acc->num = intermediate->den * acc->num + acc->den * intermediate->num;
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
    return raise(Math, "Addition error.");
  }

  return init;
}

void subtract_aux(struct LispDatum* acc, const struct LispDatum* intermediate) {
  switch (acc->type) {
    case Integer:
      acc->int_val -= intermediate->int_val;
      break;
    case Rational:
      acc->num = intermediate->den * acc->num - acc->den * intermediate->num;
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
  struct LispDatum* init;

  if (nargs == 0) {
    return raise(Generic, "Too few calls to subtract.");
  } else if (nargs == 1) {
    // The argument needs to be negated, so it is essentially being subtracted from 0.
    init = new_integer(0);
  } else {
    init = malloc(sizeof(struct LispDatum));
    copy_lisp_datum(args[0], init);

    args = args+1;  // The first argument does not need to be subtracted from itself.
    nargs -= 1;     // Reduce the number of arguments to compensate.
  }

  if (iterative_math_function(args, nargs, init, subtract_aux)) {
    free(init);
    return raise(Math, "Error during subtraction.");
  }

  return init;
}

void multiply_aux(struct LispDatum* acc, const struct LispDatum* intermediate) {
  double tmp;
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
      tmp = acc->real;
      acc->real = (tmp * intermediate->real) - (acc->im * intermediate->im);
      acc->im = (tmp * intermediate->im) + (acc->im * intermediate->real);
      break;
    default:
      break;
  }
}

struct LispDatum* multiply(struct LispDatum** args, uint32_t nargs) {
  struct LispDatum* init = new_integer(1);

  if (iterative_math_function(args, nargs, init, multiply_aux)) {
    free(init);
    return raise(Math, "Error during multiplication.");
  }

  return init;
}

void divide_aux(struct LispDatum* acc, const struct LispDatum* intermediate) {
  double d;

  struct LispDatum zero;
  write_zero(&zero);

  if (eqv(intermediate, &zero)) {
    raise(ZeroDivision, NULL);
  }

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
    return raise(Math, "Error during division.");
  }

  return init;
}

struct LispDatum* mod(struct LispDatum** args, uint32_t nargs) {
  if (nargs != 2) {
    return raise(Generic, "Incorrect number of arguments passed to mod.");
  }

  if (args[0]->type != Integer || args[1]->type != Integer) {
    return raise(Math, "Cannot perform modulus operation on non-integer values.");
  }

  return new_integer(args[0]->int_val % args[1]->int_val);
}

struct LispDatum* division(struct LispDatum** args, uint32_t nargs) {
  if (nargs != 2) {
    return raise(Generic, "Incorrect number of arguments passed to mod.");
  }

  if (args[0]->type != Integer || args[1]->type != Integer) {
    return raise(Math, "Cannot perform division algorithm on non-integer values.");
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
      printf("%s", datum->label);
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
    case String:
      printf("%s", datum->content);
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
      default:
        raise(Generic, "Non-numeric value undergoing numeric equality test.");
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
    printf(" ");
  }

  printf("\n");

  return get_nil();
}

