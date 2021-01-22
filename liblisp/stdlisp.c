#include <stdio.h>
#include <stdlib.h>
#include "stdlisp.h"
#include "data.h"
#include "err.h"

/**
 * Determine if a datum refers to an occupied (not {NULL, NULL}) Cons pair.
 */
static int is_occupied_node(const struct LispDatum* d) {
  return d != NULL && (d->type == Cons && d->car != NULL);
}

// TODO(matthew-c21): When garbage collection is added, this sort of behavior should be invoked alongside the gc to
//    increase reference counts.
/**
 * Push a new value to the end of a list. Assumes that the given node is already at the end of the list.
 */
static void push(struct LispDatum* node, struct LispDatum* value) {
  if (node == NULL || node->type != Cons || node->cdr != NULL) {
    fprintf(stderr, "Fatal exeption occurred.");
    exit(-1);
  }

  // Pushing to an empty node
  if (node->car == NULL) {
    node->car = value;
  } else {
    node->cdr = malloc(sizeof(struct LispDatum));
    node->cdr->type = Cons;
    node->cdr->car = value;
    node->cdr->cdr = NULL;
  }
}

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

    args = args + 1;  // The first argument does not need to be subtracted from itself.
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

  // TODO(matthew-c21): Non-numeric type equality.

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

// Several of the following functions start with the same check. However, since there is no proper means of jumping from
//  one part of execution to another, there's no good way eliminate this redundancy.

struct LispDatum* car(struct LispDatum** args, uint32_t nargs) {
  if (nargs != 1 || args[0]->type != Cons) {
    return raise(Generic, "`car` takes a single list argument.");
  }

  return args[0]->car;
}

struct LispDatum* cdr(struct LispDatum** args, uint32_t nargs) {
  if (nargs != 1 && (args[0]->type != Nil && args[0]->type != Cons)) {
    return raise(Generic, "`cdr` takes exactly one list argument.");
  }

  if (is_occupied_node(args[0])) {
    return args[0]->cdr;
  }

  return list(NULL, 0);
}

struct LispDatum* length(struct LispDatum** args, uint32_t nargs) {
  if (nargs != 1 || args[0]->type != Cons) {
    return raise(Generic, "`length` takes a single list argument.");
  }

  int len = 0;
  struct LispDatum* ptr = args[0];

  while (is_occupied_node(ptr)) {
    ++len;
    ptr = ptr->cdr;
  }

  return new_integer(len);
}

struct LispDatum* cons(struct LispDatum** args, uint32_t nargs) {
  if (nargs != 2) {
    return raise(Generic, "`cons` takes exactly two arguments.");
  }

  return new_cons(args[0], args[1]);
}

struct LispDatum* list(struct LispDatum** args, uint32_t nargs) {
  struct LispDatum* alist = malloc(sizeof(struct LispDatum));
  alist->type = Cons;

  if (nargs == 0) {
    alist->car = alist->cdr = NULL;
    return alist;
  }

  struct LispDatum* write_ptr = alist;
  int initial_write = 1;

  for (uint32_t i = 0; i < nargs; ++i) {
    push(write_ptr, args[i]);
    if (initial_write) {
      initial_write = 0;
    } else {
      write_ptr = write_ptr->cdr;
    }
  }

  return alist;
}

struct LispDatum* append(struct LispDatum** args, uint32_t nargs) {
  struct LispDatum* combination = malloc(sizeof(struct LispDatum));

  struct LispDatum* write_ptr = combination;

  int initial_write = 1;

  // Ensure type of all arguments.
  for (uint32_t i = 0; i < nargs; ++i) {
    if (args[i]->type != Cons) {
      free(combination);
      return raise(Generic, "Found non-list value in `append`.");
    }
  }

  for (uint32_t i = 0; i < nargs; ++i) {
    struct LispDatum* idx = args[i];

    while (!is_occupied_node(idx)) {
      if (initial_write) {
        initial_write = 0;
        write_ptr->car = idx->car;
      } else {
        push(write_ptr, idx->car);
        write_ptr = write_ptr->cdr;
      }
    }
  }

  return combination;
}

struct LispDatum* reverse(struct LispDatum** args, uint32_t nargs) {
  if (nargs != 1 || args[0]->type != Cons) {
    return raise(Generic, "`reverse` takes a single list argument");
  }

  struct LispDatum* reversal = list(NULL, 0);
  struct LispDatum* write_ptr = reversal;

  int initial_write = 1;

  struct LispDatum* idx = args[0];

  while (is_occupied_node(idx)) {
    push(write_ptr, idx->car);
    idx = idx->car;

    if (initial_write) {
      initial_write = 0;
    } else {
      write_ptr = write_ptr->cdr;
    }
  }

  return reversal;
}

