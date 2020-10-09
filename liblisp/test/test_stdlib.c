#include <stddef.h>

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

void Test_int_inequality(CuTest* tc) {
  struct LispDatum* a = new_integer(10);
  struct LispDatum* b = new_integer(12);

  CuAssert(tc, "10 != 12", !eqv(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_integer(-6);
  b = new_integer(-7);

  CuAssert(tc, "-6 != -7", !eqv(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_integer(6);
  b = new_integer(-6);

  CuAssert(tc, "6 != -6", !eqv(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_integer(-7);
  b = new_integer(7);

  CuAssert(tc, "-7 != 7", !eqv(a, b));

  discard_datum(a);
  discard_datum(b);
}

void Test_float_equality(CuTest* tc) {
  tc = NULL;
}

void Test_float_inequality(CuTest* tc) {
  struct LispDatum* a = new_real(10.0);
  struct LispDatum* b = new_real(12.0);

  CuAssert(tc, "10.0 != 12.0", !eqv(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_real(10.0);
  b = new_real(10.01);

  CuAssert(tc, "10.0 != 10.01", !eqv(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_real(-841.0);
  b = new_real(-76.0);

  CuAssert(tc, "-841.0 != -76.0", !eqv(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_real(-841.25);
  b = new_real(-841.25005);

  CuAssert(tc, "-841.25 != -841.25005", !eqv(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_real(-841.25);
  b = new_real(841.25);

  CuAssert(tc, "-841.25 != 841.25", !eqv(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_real(841.25);
  b = new_real(-841.25);

  CuAssert(tc, "841.25 != -841.25", !eqv(a, b));

  discard_datum(a);
  discard_datum(b);
}

// Note: Be sure to make sure that creating a rational number simplifies it.
void Test_rational_equality(CuTest* tc) {
  tc = NULL;
}

void Test_rational_inequality(CuTest* tc) {
  tc = NULL;
}

void Test_complex_equality(CuTest* tc) {
  tc = NULL;
}

void Test_complex_inequality(CuTest* tc) {
  tc = NULL;
}

void Test_promotion_equality_int(CuTest* tc) {
  struct LispDatum* a = new_integer(1);
  struct LispDatum* b = new_real(1.0);

  CuAssert(tc, "1 = 1.0", eqv(a, b));

  discard_datum(b);

  b = new_rational(1, 1);

  CuAssert(tc, "1 = 1/1", eqv(a, b));

  discard_datum(b);

  b = new_complex(1.0, 0);

  CuAssert(tc, "1 = 1+0i", eqv(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_integer(-1);
  b = new_real(-1.0);

  CuAssert(tc, "-1 = -1.0", eqv(a, b));

  discard_datum(b);

  b = new_rational(-1, 1);

  CuAssert(tc, "-1 = -1/1", eqv(a, b));

  discard_datum(b);

  b = new_complex(-1.0, 0);

  CuAssert(tc, "-1 = -1+0i", eqv(a, b));

  discard_datum(a);
  discard_datum(b);
}

void Test_promotion_equality_rational(CuTest* tc) {
  struct LispDatum* a = new_rational(1, 1);
  struct LispDatum* b = new_real(1.0);

  CuAssert(tc, "1/1 = 1.0", eqv(a, b));

  discard_datum(b);

  b = new_complex(1.0, 0);

  CuAssert(tc, "1/1 = 1+0i", eqv(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_rational(-1, 1);
  b = new_real(-1.0);

  CuAssert(tc, "-1/1 = -1.0", eqv(a, b));

  discard_datum(b);

  b = new_complex(-1.0, 0);

  CuAssert(tc, "-1/1 = -1+0i", eqv(a, b));

  discard_datum(a);
  discard_datum(b);
}

void Test_promotion_equality_real(CuTest* tc) {
  struct LispDatum* a = new_real(1.0);
  struct LispDatum* b = new_complex(1.0, 0);

  CuAssert(tc, "1.0 = 1.0+0i", eqv(a, b));

  discard_datum(b);

  b = new_complex(-1.0, 0);

  CuAssert(tc, "-1.0 = -1+0i", eqv(a, b));

  discard_datum(a);
  discard_datum(b);
}

void Test_cons_equality(CuTest* tc) {
  tc = NULL;
}

void Test_cons_inequality(CuTest* tc) {
  tc = NULL;
}

// MATH
void Test_int_addition(CuTest* tc) {
  CuAssert(tc, "", 0);
}

// + ++ -- +- -+ +++ ---
void Test_int_int_multiplication(CuTest* tc) {
  struct LispDatum* args[3];
  args[0] = new_integer(5);
  args[1] = new_integer(7);
  args[1] = new_integer(0);

  CuAssert(tc, "5 * 7 = 35", eqv(multiply(args, 2), new_integer(35)));

  args[0]->int_val = -5;
  args[1]->int_val = -7;

  CuAssert(tc, "-5 * -7 = 35", eqv(multiply(args, 2), new_integer(35)));

  args[0]->int_val = 5;
  args[1]->int_val = -7;

  CuAssert(tc, "5 * -7 = -35", eqv(multiply(args, 2), new_integer(-35)));

  args[0]->int_val = -5;
  args[1]->int_val = 7;

  CuAssert(tc, "-5 * 7 = -35", eqv(multiply(args, 2), new_integer(-35)));

  args[0]->int_val = 2;
  args[1]->int_val = 2;
  args[2]->int_val = 2;

  CuAssert(tc, "2 * 2 * 2 = 8", eqv(multiply(args, 3), new_integer(8)));

  args[0]->int_val = -2;
  args[1]->int_val = -2;
  args[2]->int_val = -2;

  CuAssert(tc, "-2 * -2 * -2 = -8", eqv(multiply(args, 3), new_integer(-8)));
}

void Test_int_real_multiplication(CuTest* tc) {
  struct LispDatum* args[3];
  args[0] = new_integer(5);
  args[1] = new_real(7.0);
  args[2] = new_real(2.0);

  CuAssert(tc, "5 * 7 = 35", eqv(multiply(args, 2), new_real(35.0)));

  args[0]->int_val = -5;
  args[1]->float_val = -7.0;

  CuAssert(tc, "-5 * -7 = 35", eqv(multiply(args, 2), new_real(35.0)));

  args[0]->int_val = 5;
  args[1]->float_val = -7;

  CuAssert(tc, "5 * -7 = -35", eqv(multiply(args, 2), new_real(-35.0)));

  args[0]->int_val = -5;
  args[1]->float_val = 7;

  CuAssert(tc, "-5 * 7 = -35", eqv(multiply(args, 2), new_real(-35.0)));

  args[0]->int_val = 2;
  args[1]->float_val = 2;
  args[2]->float_val = 2;

  CuAssert(tc, "2 * 2 * 2 = 8", eqv(multiply(args, 3), new_real(8.0)));

  args[0]->int_val = -2;
  args[1]->float_val = -2;
  args[2]->float_val = -2;

  CuAssert(tc, "-2 * -2 * -2 = -8", eqv(multiply(args, 3), new_real(-8.0)));
}

void Test_int_rational_multiplication(CuTest* tc) {
  // TODO(matthew-c21): Expand with relevant math.
  struct LispDatum* args[3];
  args[0] = new_integer(5);
  args[1] = new_rational(7, 1);
  args[2] = new_rational(2, 1);

  CuAssert(tc, "5 * 7 = 35", eqv(multiply(args, 2), new_rational(35, 1)));

  args[0]->int_val = -5;
  args[1]->num = -7;

  CuAssert(tc, "-5 * -7 = 35", eqv(multiply(args, 2), new_rational(35, 1)));

  args[0]->int_val = 5;
  args[1]->num = -7;

  CuAssert(tc, "5 * -7 = -35", eqv(multiply(args, 2), new_rational(-35, 1)));

  args[0]->int_val = -5;
  args[1]->num = 7;

  CuAssert(tc, "-5 * 7 = -35", eqv(multiply(args, 2), new_rational(-35, 1)));

  args[0]->int_val = 2;
  args[1]->num = 2;
  args[2]->num = 2;

  CuAssert(tc, "2 * 2 * 2 = 8", eqv(multiply(args, 3), new_rational(8, 1)));

  args[0]->int_val = -2;
  args[1]->num = -2;
  args[2]->num = -2;

  CuAssert(tc, "-2 * -2 * -2 = -8", eqv(multiply(args, 3), new_rational(-8, 1)));
}

void Test_int_complex_multiplication(CuTest* tc) {
  tc = NULL;
}

void Test_real_int_multiplication(CuTest* tc) {
  tc = NULL;
}

void Test_real_real_multiplication(CuTest* tc) {
  tc = NULL;
}

void Test_real_rational_multiplication(CuTest* tc) {
  tc = NULL;
}

void Test_real_complex_multiplication(CuTest* tc) {
  tc = NULL;
}

void Test_rational_int_multiplication(CuTest* tc) {
  tc = NULL;
}

void Test_rational_real_multiplication(CuTest* tc) {
  tc = NULL;
}

void Test_rational_rational_multiplication(CuTest* tc) {
  tc = NULL;
}

void Test_rational_complex_multiplication(CuTest* tc) {
  tc = NULL;
}

void Test_complex_int_multiplication(CuTest* tc) {
  tc = NULL;
}

void Test_complex_real_multiplication(CuTest* tc) {
  tc = NULL;
}

void Test_complex_rational_multiplication(CuTest* tc) {
  tc = NULL;
}

void Test_complex_complex_multiplication(CuTest* tc) {
  tc = NULL;
}
