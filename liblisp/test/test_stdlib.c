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
  struct LispDatum* a = new_real(5.0);
  struct LispDatum* b = new_real(5.0);

  CuAssert(tc, "5.0 = 5.0", eqv(a, b));

  a->float_val *= -1;
  b->float_val *= -1;

  CuAssert(tc, "-5.0 = -5.0", eqv(a, b));

  a->float_val *= 0;
  b->float_val *= 0;

  CuAssert(tc, "0 = 0", eqv(a, b));

  discard_datum(a);
  discard_datum(b);
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

void Test_rational_equality(CuTest* tc) {
  struct LispDatum* a = new_rational(2, 3);
  struct LispDatum* b = new_rational(4, 6);

  CuAssert(tc, "2/3 = 4/6", eqv(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_rational(1,2);
  b = new_rational(1,2);

  CuAssert(tc, "1/2 = 1/2", eqv(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_rational(5, 6);
  b = new_rational(-5, -6);

  CuAssert(tc, "5/6 = -5/-6", eqv(a, b));

  discard_datum(a);
  discard_datum(b);
}

void Test_rational_inequality(CuTest* tc) {
  CuAssert(tc, "1/2 != -1/2", !eqv(new_rational(1, 2), new_rational(-1, 2)));
  CuAssert(tc, "3/4 != 1/2", !eqv(new_rational(3, 4), new_rational(1, 2)));
}

void Test_complex_equality(CuTest* tc) {
  CuAssert(tc, "1+2i = 1+2i", eqv(new_complex(1, 2), new_complex(1, 2)));
}

void Test_complex_inequality(CuTest* tc) {
  CuAssert(tc, "1+2i != -1+2i", !eqv(new_complex(1, 2), new_complex(-1, 2)));
  CuAssert(tc, "3+4i != 1+2i", !eqv(new_complex(3, 4), new_complex(1, 2)));
  CuAssert(tc, "1-i != 1+i", !eqv(new_complex(1, -1), new_complex(1, 1)));
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

  discard_datum(a);
  discard_datum(b);

  a = new_real(-1.0);
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
  struct LispDatum* args[2];
  args[0] = new_integer(-3);
  args[1] = new_integer(2);

  CuAssert(tc, "-3 + 2 = -1", eqv(add(args, 2), new_integer(-1)));

  args[1] = new_rational(4, 5);
  CuAssert(tc, "-3 + 4/5 = -11/5", eqv(add(args, 2), new_rational(-11, 5)));

  args[1] = new_real(3.5);
  CuAssert(tc, "-3 + 3.5 = 0.5", eqv(add(args, 2), new_real(0.5)));

  args[1] = new_complex(3, 1);
  CuAssert(tc, "-3 + 3+i = i", eqv(add(args, 2), new_complex(0, 1)));
}

void Test_real_addition(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_real(-3);
  args[1] = new_integer(2);

  CuAssert(tc, "-3.0 + 2 = -1", eqv(add(args, 2), new_real(-1)));

  args[1] = new_rational(4, 5);
  CuAssert(tc, "-3.0 + 4/5 = -2.2", eqv(add(args, 2), new_real(-2.2)));

  args[1] = new_real(3.5);
  CuAssert(tc, "-3.0 + 3.5 = 0.5", eqv(add(args, 2), new_real(0.5)));

  args[1] = new_complex(3, 1);
  CuAssert(tc, "-3.0 + 3+i = i", eqv(add(args, 2), new_complex(0, 1)));
}

void Test_rational_addition(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_rational(-3, 1);
  args[1] = new_integer(2);

  CuAssert(tc, "-3/1 + 2 = -1", eqv(add(args, 2), new_rational(-1, 1)));

  args[1] = new_rational(4, 5);
  CuAssert(tc, "-3/1 + 4/5 = -11/5", eqv(add(args, 2), new_rational(-11, 5)));

  args[1] = new_real(3.5);
  CuAssert(tc, "-3/1 + 3.5 = 0.5", eqv(add(args, 2), new_real(0.5)));

  args[1] = new_complex(3, 1);
  CuAssert(tc, "-3/1 + 3+i = i", eqv(add(args, 2), new_complex(0, 1)));
}

void Test_complex_addition(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_complex(-3, 0);
  args[1] = new_integer(2);

  CuAssert(tc, "-3+0i + 2 = -1", eqv(add(args, 2), new_complex(-1, 0)));

  args[1] = new_rational(4, 5);
  CuAssert(tc, "-3+0i + 4/5 = -11/5", eqv(add(args, 2), new_complex(-2.2, 0)));

  args[1] = new_real(3.5);
  CuAssert(tc, "-3+0i + 3.5 = 0.5", eqv(add(args, 2), new_complex(0.5, 0)));

  args[1] = new_complex(3, 1);
  CuAssert(tc, "-3+0i + 3+i = i", eqv(add(args, 2), new_complex(0, 1)));
}

void Test_int_subtraction(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_integer(16);
  args[1] = new_integer(4);

  CuAssert(tc, "16 - 4 = 12", eqv(subtract(args, 2), new_integer(12)));

  args[1] = new_rational(9, 2);
  CuAssert(tc, "16 - 9/2 = 23/2", eqv(subtract(args, 2), new_rational(23, 2)));

  args[1] = new_real(-4.5);
  CuAssert(tc, "16 - -4.5 = 20.5", eqv(subtract(args, 2), new_real(20.5)));

  args[1] = new_complex(9, 2);
  CuAssert(tc, "16 - 9+2i = 7-2i", eqv(subtract(args, 2), new_complex(7, -2)));
}

void Test_real_subtraction(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_real(16);
  args[1] = new_integer(4);

  CuAssert(tc, "16.0 - 4 = 12.0", eqv(subtract(args, 2), new_real(12)));

  args[1] = new_rational(9, 2);
  CuAssert(tc, "16.0 - 9/2 = 11.5", eqv(subtract(args, 2), new_real(11.5)));

  args[1] = new_real(-4.5);
  CuAssert(tc, "16.0 - -4.5 = 20.5", eqv(subtract(args, 2), new_real(20.5)));

  args[1] = new_complex(9, 2);
  CuAssert(tc, "16.0 - 9+2i = 7-2i", eqv(subtract(args, 2), new_complex(7, -2)));
}

void Test_rational_subtraction(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_rational(16, 1);
  args[1] = new_integer(4);

  CuAssert(tc, "16/1 - 4 = 12/1", eqv(subtract(args, 2), new_rational(12, 1)));

  args[1] = new_rational(9, 2);
  CuAssert(tc, "16/1 - 9/2 = 21/2", eqv(subtract(args, 2), new_rational(23, 2)));

  args[1] = new_real(-4.5);
  CuAssert(tc, "16/1 - -4.5 = 20.5", eqv(subtract(args, 2), new_real(20.5)));

  args[1] = new_complex(9, 2);
  CuAssert(tc, "16/1 - 9+2i = 7-2i", eqv(subtract(args, 2), new_complex(7, -2)));
}

void Test_complex_subtraction(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_complex(16, 0);
  args[1] = new_integer(4);

  CuAssert(tc, "16/1 - 4 = 12/1", eqv(subtract(args, 2), new_complex(12, 0)));

  args[1] = new_rational(9, 2);
  CuAssert(tc, "16/1 - 9/2 = 23/2", eqv(subtract(args, 2), new_complex(11.5, 0)));

  args[1] = new_real(-4.5);
  CuAssert(tc, "16/1 - -4.5 = 20.5", eqv(subtract(args, 2), new_complex(20.5, 0)));

  args[1] = new_complex(9, 2);
  CuAssert(tc, "16/1 - 9+2i = 7-2i", eqv(subtract(args, 2), new_complex(7, -2)));
}

void Test_int_division(CuTest* tc) {
  tc = NULL;
}

void Test_real_division(CuTest* tc) {
  tc = NULL;
}

void Test_rational_division(CuTest* tc) {
  tc = NULL;
}

void Test_complex_division(CuTest* tc) {
  tc = NULL;
}

void Test_int_int_multiplication(CuTest* tc) {
  struct LispDatum* args[3];
  args[0] = new_integer(5);
  args[1] = new_integer(7);
  args[2] = new_integer(0);

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
  args[1] = new_rational(7, 3);
  args[2] = new_rational(2, 1);

  CuAssert(tc, "5 * 7 = 35", eqv(multiply(args, 2), new_rational(35, 3)));

  args[0]->int_val = -5;
  args[1]->num = -7;

  CuAssert(tc, "-5 * -7 = 35", eqv(multiply(args, 2), new_rational(35, 3)));

  args[0]->int_val = 5;
  args[1]->num = -7;

  CuAssert(tc, "5 * -7 = -35", eqv(multiply(args, 2), new_rational(-35, 3)));

  args[0]->int_val = -5;
  args[1]->num = 7;

  CuAssert(tc, "-5 * 7 = -35", eqv(multiply(args, 2), new_rational(-35, 3)));

  args[0]->int_val = 2;
  args[1]->num = 2;
  args[2]->num = 2;

  CuAssert(tc, "2 * 2 * 2 = 8", eqv(multiply(args, 3), new_rational(8, 3)));

  args[0]->int_val = -2;
  args[1]->num = -2;
  args[2]->num = -2;

  CuAssert(tc, "-2 * -2 * -2 = -8", eqv(multiply(args, 3), new_rational(-8, 3)));
}

void Test_int_complex_multiplication(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_integer(16);
  args[1] = new_complex(2, 3);

  CuAssert(tc, "16 * 2+3i = 32 + 48i", eqv(multiply(args, 2), new_complex(32, 48)));

  args[0]->int_val = -1;
  args[1]->im = -4;

  CuAssert(tc, "-1 * 2-4i = -2+4i", eqv(multiply(args, 2), new_complex(-2, 4)));
}

void Test_real_int_multiplication(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_real(2.5);
  args[1] = new_integer(4);

  CuAssert(tc, "4 * 2.5 = 10.0", eqv(multiply(args, 2), new_real(10)));
  tc = NULL;
}

void Test_real_real_multiplication(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_real(2.5);
  args[1] = new_real(1.75);

  CuAssert(tc, "1.75 * 2.5 = 4.375", eqv(multiply(args, 2), new_real(2.5 * 1.75)));
}

void Test_real_rational_multiplication(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_real(2.5);
  args[1] = new_rational(7, 4);

  CuAssert(tc, "7/4 * 2.5 = 4.375", eqv(multiply(args, 2), new_real(2.5 * 1.75)));
}

void Test_real_complex_multiplication(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_real(-2.5);
  args[1] = new_complex(3.25, -4.5);

  CuAssert(tc, "3.25-4.5i * -2.5 = -8.125+11.25i", eqv(multiply(args, 2), new_complex(-8.125, 11.25)));
}

void Test_rational_int_multiplication(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_rational(2, 3);
  args[1] = new_integer(5);

  struct LispDatum* result = multiply(args, 2);
  CuAssertIntEquals(tc, 10, result->num);
  CuAssertIntEquals(tc, 3, result->den);
}

void Test_rational_real_multiplication(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_rational(-2, 5);
  args[1] = new_real(2.5);

  CuAssert(tc, "-2/5 * 2.5 = -1.0", eqv(multiply(args, 2), new_real(-1.0)));
}

void Test_rational_rational_multiplication(CuTest* tc) {
  struct LispDatum* args[3];
  args[0] = new_rational(2, 3);
  args[1] = new_rational(3, 4);

  struct LispDatum* result = multiply(args, 2);
  CuAssert(tc, "2/3 * 3/4 is rational", result->type = Rational);
  CuAssertIntEquals(tc, 1, result->num);
  CuAssertIntEquals(tc, 2, result->den);
}

void Test_rational_complex_multiplication(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_rational(-7, 2);
  args[1] = new_complex(2, -5.2);

  CuAssert(tc, "-7/2 * 2-5.2i = -7+18.2i", eqv(multiply(args, 2), new_complex(-7, 18.2)));
}

void Test_complex_int_multiplication(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_complex(-2, 5);
  args[1] = new_integer(4);

  CuAssert(tc, "-2+5i * 4 = -8+20i", eqv(multiply(args, 2), new_complex(-8, 20)));
}

void Test_complex_real_multiplication(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_complex(-2, 5);
  args[1] = new_real(-0.5);

  CuAssert(tc, "-2+5i * -0.5 = 1-2.5i", eqv(multiply(args, 2), new_complex(1, -2.5)));
}

void Test_complex_rational_multiplication(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_complex(-2, 5);
  args[1] = new_rational(-1, 2);

  CuAssert(tc, "-2+5i * -1/2 = 1-2.5i", eqv(multiply(args, 2), new_complex(1, -2.5)));
}

void Test_complex_complex_multiplication(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_complex(-2, 5);
  args[1] = new_complex(-2, -5);

  CuAssert(tc, "-2+5i * -2-5i = -4", eqv(multiply(args, 2), new_complex(29, 0)));
}

void Test_multiplication(CuTest* tc) {
  struct LispDatum* args[4];
  args[0] = new_integer(3);
  args[1] = new_rational(1, 2);
  args[2] = new_real(-3.5);
  args[3] = new_complex(-4.25, 6.125);

  CuAssert(tc, "3 * 1/2 * -3.5 * -4.25+6.125i = ", eqv(multiply(args, 4), new_complex(22.3125, -32.15625)));
}

void Test_empty_list(CuTest* tc) {
  struct LispDatum* empty_list = list(NULL, 0);
  CuAssert(tc, "(list) != nil", !eqv(get_nil(), empty_list));
  CuAssertPtrEquals(tc, NULL, empty_list->car);
  CuAssertPtrEquals(tc, NULL, empty_list->cdr);
}

void Test_single_element_list(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = get_nil();
  args[1] = get_nil();
  struct LispDatum* mono_list = list(args, 1);
  CuAssertPtrEquals(tc, get_nil(), mono_list->car);
  CuAssertPtrEquals(tc, NULL, mono_list->cdr);

  struct LispDatum* duo_list = list(args, 2);
  CuAssertPtrEquals(tc, get_nil(), duo_list->car);
  CuAssertPtrEquals(tc, get_nil(), duo_list->cdr->car);
  CuAssertPtrEquals(tc, NULL, duo_list->cdr->cdr);
}

void Test_nil_cadr(CuTest* tc) {
  struct LispDatum* nil = get_nil();

  CuAssert(tc, "(cdr nil) == ()", eqv(cdr(&nil, 1), list(NULL, 0)));
  // TODO(matthew-c21) Find a way to test for exceptions that circumvents the natural method of just closing the program
}
