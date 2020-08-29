#include "CuTest.h"
#include "../data.h"

// PREDICATES
void Test_int_equality(CuTest* tc) {
  struct LispDatum* a = new_integer(32);
  struct LispDatum* b = new_integer(32);

  CuAssert(tc, "32 = 32", eqv(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_integer(-9);
  b = new_integer(-9);

  CuAssert(tc, "-9 = -9", eqv(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_integer(0);
  b = new_integer(0);

  CuAssert(tc, "0 = 0", eqv(a, b));

  discard_datum(a);
  discard_datum(b);
}

/*
void int_inequality(CuTest* ct) {}

void float_equality(CuTest* ct) {}

void float_inequality(CuTest* ct) {}

// Note: Be sure to make sure that creating a rational number simplifies it.
void rational_equality(CuTest* ct) {}

void rational_inequality(CuTest* ct) {}

void complex_equality(CuTest* ct) {}

void complex_inequality(CuTest* ct) {}

void cons_equality(CuTest* ct) {}

void cons_inequality(CuTest* ct) {}

// MATH
void int_addition(CuTest* ct) {}
*/

