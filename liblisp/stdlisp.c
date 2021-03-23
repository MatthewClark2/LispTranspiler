#include <stdio.h>
#include <stdlib.h>
#include <string.h>
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
    set_global_error_behavior(LogAndQuit);
    raise(Generic, "Fatal programming error occurred causing illegal call to `push`.");
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
    case Keyword:
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
    case Bool:
      dest->boolean = source->boolean;
      break;
    case Lambda:
      dest->f = source->f;
      dest->captures = source->captures;
      dest->n_captures = source->n_captures;
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
    return raise(Argument, "Too few calls to subtract.");
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

  if (datum_cmp(intermediate, &zero)) {
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
    return raise(Argument, "Incorrect number of arguments passed to mod.");
  }

  if (args[0]->type != Integer || args[1]->type != Integer) {
    return raise(Math, "Cannot perform modulus operation on non-integer values.");
  }

  return new_integer(args[0]->int_val % args[1]->int_val);
}

struct LispDatum* division(struct LispDatum** args, uint32_t nargs) {
  if (nargs != 2) {
    return raise(Argument, "Incorrect number of arguments passed to mod.");
  }

  if (args[0]->type != Integer || args[1]->type != Integer) {
    return raise(Math, "Cannot perform division algorithm on non-integer values.");
  }

  struct LispDatum* d = new_integer(args[0]->int_val / args[1]->int_val);
  struct LispDatum* r = new_integer(args[0]->int_val % args[1]->int_val);
  return new_cons(r, new_cons(d, get_nil()));
}

void display(struct LispDatum* datum) {
  struct LispDatum* read_ptr = datum;

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
    case Keyword:
      printf(":%s", datum->label);
      break;
    case Cons:
      printf("(");
      while (is_occupied_node(read_ptr)) {
        display(read_ptr->car);

        if (is_occupied_node(read_ptr->cdr)) printf(" ");
        read_ptr = read_ptr->cdr;
      }

      if (read_ptr != NULL) {
        printf(" . ");
        display(read_ptr);
      }

      printf(")");
      break;
    case Nil:
      printf("nil");
      break;
    case String:
      printf("%s", datum->content);
      break;
    case Bool:
      printf("%s", datum->boolean ? "#t" : "#f");
      break;
    case Lambda:
      if (datum->name == NULL) {
        printf("<anonymous function at 0x%p>", (void*)datum);
      } else {
        printf("<function %s>", datum->name);
      }
      break;
  }
}

int is_numeric(const struct LispDatum* x) {
  return x->type <= Complex;
}

int datum_cmp(const struct LispDatum* a, const struct LispDatum* b) {
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
  } else if (a->type == b->type) {
    // TODO(matthew-c21): Cons, symbol, keyword equality all missing.
    switch (a->type) {
      case String:
        return a->length == b->length && strncmp(a->content, b->content, a->length) == 0;
      case Symbol:
      case Keyword:
        return strcmp(a->label, b->label) == 0;
      case Bool:  // Nil and Bool are both static, and must be equal to themselves.
      case Nil:
      case Lambda:
        // There's no use comparing lambdas, so an individual lambda is only equal to itself.
        return a == b;
      case Cons:
        if (a->car == NULL && b->car == NULL) {
          return 1;
        }

        return datum_cmp(a->car, b->car) && datum_cmp(a->cdr, b->cdr);
      case Integer:
      case Real:
      case Rational:
      case Complex:
        raise(Generic, "Invalid program state. Contact the developer.");
        break;
    }
  }

  // Technically unreachable.
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
  if (nargs != 1) {
    return raise(Argument, "`car` takes a single argument.");
  } else if (args[0]->type != Cons) {
    return raise(Type, "`car` expected proper list argument");
  }

  return args[0]->car;
}

struct LispDatum* cdr(struct LispDatum** args, uint32_t nargs) {
  if (nargs != 1) {
    return raise(Argument, "`cdr` expects exactly one argument");
  } else if (args[0]->type != Nil && args[0]->type != Cons) {
    return raise(Type, "`cdr` expected a list valued argument.");
  }

  if (is_occupied_node(args[0])) {
    return args[0]->cdr;
  }

  return list(NULL, 0);
}

struct LispDatum* length(struct LispDatum** args, uint32_t nargs) {
  if (nargs != 1) {
    return raise(Argument, "`length` takes a single argument.");
  } else if (args[0]->type == Nil) {
    return new_integer(0);
  } else if (args[0]->type == String) {
    return new_integer(args[0]->length);
  } else if (args[0]->type != Cons) {
      return raise(Type, "`length` expected list argument");
  }

  int len = 0;
  struct LispDatum* ptr = args[0];

  while (is_occupied_node(ptr)) {
    ++len;
    ptr = ptr->cdr;
  }

  // The 0 length check helps account for empty lists that never get assigned to their cdr.
  if (len != 0 && ptr != NULL) {
    return raise(Type, "`length` expected list argument. Received pair.");
  }

  return new_integer(len);
}

struct LispDatum* cons(struct LispDatum** args, uint32_t nargs) {
  if (nargs != 2) {
    return raise(Argument, "`cons` takes exactly two arguments.");
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
  // Ensure type of all arguments.
  for (uint32_t i = 0; i < nargs; ++i) {
    if (args[i]->type != Cons && args[i]->type != Nil) {
      return raise(Type, "Expected list in argument to `append`.");
    }
  }

  // Special cases for small argument numbers.
  if (nargs == 0) {
    return list(NULL, 0);
  } else if (nargs == 1) {
    return args[0];
  }

  struct LispDatum* combination = malloc(sizeof(struct LispDatum));
  combination->type = Cons;

  struct LispDatum* write_ptr = combination;

  int initial_write = 1;

  for (uint32_t i = 0; i < nargs - 1; ++i) {
    struct LispDatum* idx = args[i];

    if (idx->type == Nil) {
      continue;
    }

    // check type of each idx. If any pairs appear, toss them. If a Nil appears, it should be skipped.
    while (is_occupied_node(idx)) {
      push(write_ptr, idx->car);

      if (initial_write) {
        initial_write = 0;
      } else {
        write_ptr = write_ptr->cdr;
      }

      idx = idx->cdr;
    }

    if (idx != NULL) {
      return raise(Type, "Non-terminal arguments to `append` should be proper lists");
    }
  }

  // If it's an occupied list (proper or otherwise), push it at the end. It must otherwise be empty or nil.
  if (is_occupied_node(args[nargs - 1])) {
    write_ptr->cdr = args[nargs-1];
  }

  // Otherwise it's nil and nothing needs to be done.
  return combination;
}

struct LispDatum* reverse(struct LispDatum** args, uint32_t nargs) {
  if (nargs != 1) {
    return raise(Argument, "`reverse` takes exactly one argument");
  } else if (args[0]->type == Nil) {
    return args[0];
  } else if (args[0]->type != Cons) {
    return raise(Type, "`reverse` expected list argument");
  }

  // Handle case of empty and singleton list.
  if (args[0]->car == NULL || args[0]->cdr == NULL) {
    return args[0];
  }

  struct LispDatum* reversal = NULL;
  struct LispDatum* idx = args[0];

  while (is_occupied_node(idx)) {
    reversal = new_cons(idx->car, reversal);
    idx = idx->cdr;
  }

  if (idx != NULL) {
    return raise(Type, "`reverse` expects a proper list");
  }

  if (reversal == NULL) {
    return list(NULL, 0);
  }

  return reversal;
}

struct LispDatum* eqv(struct LispDatum** args, uint32_t nargs) {
  int truthy = 1;

  for (uint32_t i = 0; i + 1 < nargs; ++i) {
    truthy = truthy && datum_cmp(args[i], args[i + 1]);
  }

  return truthy ? get_true() : get_false();
}

static int cmp(struct LispDatum* a, struct LispDatum* b) {
  if (is_numeric(a) && is_numeric(b)) {
    struct LispDatum x;
    copy_lisp_datum(a, &x);

    struct LispDatum y;
    copy_lisp_datum(b, &y);

    promote(&x, y.type);
    promote(&y, x.type);

    switch (x.type) {
      case Integer:
        return x.int_val == y.int_val ? 0 : x.int_val > y.int_val ? 1 : -1;
      case Rational:
        return x.num == y.num && x.den == y.den ? 0 : (double) x.num / x.den > (double) y.num / y.den ? 1 : -1;
      case Real:
        return x.float_val == y.float_val ? 0 : x.float_val > y.float_val ? 1 : -1;
      case Complex:
        if (x.real == y.real) {
          return x.im == y.im ? 0 : x.im > y.im ? 1 : -1;
        } else {
          return  x.real > y.real ? 1 : -1;
        }
      default:
        raise(Generic, "invalid state reached during cmp");
        return 0;
    }
  } else {
    raise(Generic, "invalid state reached during cmp");
    return 0;
  }
}

static struct LispDatum*
comparator(struct LispDatum** args, uint32_t nargs, int (* valid)(struct LispDatum*, struct LispDatum*)) {
  int is_true = 1;

  for (uint32_t i = 0; i + 1 < nargs; ++i) {
    // TODO(matthew-c21): Redundant checks are redundant.
    if (!is_numeric(args[i]) || !is_numeric(args[i + 1])) {
      return raise(Generic, "Compared values must be numeric.");
    }

    is_true = is_true && valid(args[i], args[i + 1]);
  }

  return is_true ? get_true() : get_false();
}

int less_than_aux(struct LispDatum* a, struct LispDatum* b) {
  return cmp(a, b) < 0;
}

struct LispDatum* less_than(struct LispDatum** args, uint32_t nargs) {
  return comparator(args, nargs, less_than_aux);
}

int num_equals_aux(struct LispDatum* a, struct LispDatum* b) {
  return cmp(a, b) == 0;
}

struct LispDatum* num_equals(struct LispDatum** args, uint32_t nargs) {
  return comparator(args, nargs, num_equals_aux);
}

int greater_than_aux(struct LispDatum* a, struct LispDatum* b) {
  return cmp(a, b) > 0;
}

struct LispDatum* greater_than(struct LispDatum** args, uint32_t nargs) {
  return comparator(args, nargs, greater_than_aux);
}

int less_than_eql_aux(struct LispDatum* a, struct LispDatum* b) {
  return cmp(a, b) <= 0;
}

struct LispDatum* less_than_eql(struct LispDatum** args, uint32_t nargs) {
  return comparator(args, nargs, less_than_eql_aux);
}

int greater_than_eql_aux(struct LispDatum* a, struct LispDatum* b) {
  return cmp(a, b) >= 0;
}

struct LispDatum* greater_than_eql(struct LispDatum** args, uint32_t nargs) {
  return comparator(args, nargs, greater_than_eql_aux);
}

// NOTE(matthew-c21): The implementation of the following functions assumes that values are immutable. Bugs may ensure
//  if that assumption is violated.
struct LispDatum* logical_and(struct LispDatum** args, uint32_t nargs) {
  struct LispDatum* last = get_true();

  for (uint32_t i = 0; i < nargs; ++i) {
    if (truthy(args[i])) {  // Direct pointer comparison is bad unless the pointer is static.
      last = args[i];
    } else {
      return get_false();
    }
  }

  return last;
}

struct LispDatum* logical_or(struct LispDatum** args, uint32_t nargs) {
  for (uint32_t i = 0; i < nargs; ++i) {
    if (truthy(args[i])) {  // Direct pointer comparison is bad unless the pointer is static.
      return args[i];
    }
  }

  return get_false();
}

struct LispDatum* logical_not(struct LispDatum** args, uint32_t nargs) {
  if (nargs != 1) {
    return raise(Generic, "Wrong number of arguments passed to not");
  }

  return truthy(args[0]) ? get_false() : get_true();
}

struct LispDatum* apply(struct LispDatum** args, uint32_t nargs) {
  if (nargs != 2) {
    return raise(Argument, "`apply` requires exactly two arguments.");
  } else if (args[0]->type != Lambda || ((args[1]->type != Cons) && args[1]->type != Nil)) {
    return raise(Type, "Expected `lambda` and `cons` type arguments to `apply`.");
  }

  // This is a double traversal, which is inefficient, but saves the hassle of allocating and freeing memory.
  struct LispDatum* i = length(&args[1], 1);
  int len = i->int_val;

  free(i);  // TODO(matthew-c21): Clean up when the garbage collector is implemented.

  // TODO(matthew-c21): Are variadic arguments handled at compile time or run time?
  //  Answer: Do it at runtime. Native functions can probably work with the given array as is, but generated functions
  //  will just need to interface through a list, which should just require a `rest = list(args + x, nargs - x)`, where
  //  x is the number of named arguments.
  struct LispDatum* f_args[len];
  uint32_t j = 0;
  struct LispDatum* ptr;

  // Collect the list into an array.
  for (ptr = args[1]; ptr != NULL && ptr->type == Cons; ptr = ptr->cdr) {
    f_args[j++] = ptr->car;
  }

  // Improper list, so we give up.
  if (ptr != NULL) {
    return raise(Type, "`apply` requires a proper list.");
  }

  return args[0]->f(f_args, j);
}

struct LispDatum* funcall(struct LispDatum** args, uint32_t nargs) {
  if (nargs == 0) {
    return raise(Argument, "`funcall` requires at least one argument.");
  } else if (args[0]->type != Lambda) {
    return raise(Type, "Expected lambda.");
  }

  // If there's no other arguments, we want to avoid the risk of indexing past the array.
  return args[0]->f(nargs == 1 ? NULL : args + 1, nargs - 1);
}

