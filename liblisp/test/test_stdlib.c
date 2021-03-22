#include <stddef.h>

#include "../err.h"
#include "CuTest.h"
#include "../data.h"

#define AssertThrows(expression, err) CuAssertPtrEquals(tc, NULL, (expression)); \
CuAssertTrue(tc, GlobalErrorState == (err)); \
raise(None, NULL);

static void assert_empty_list(CuTest* tc, const char* msg, struct LispDatum* alist) {
  CuAssert(tc, "Value is a list", alist->type == Cons);
  CuAssert(tc, msg, alist->car == NULL && alist->cdr == NULL);

}

// PREDICATES
// TODO(matthew-c21): List and string equality testing
void Test_int_equality(CuTest* tc) {
  struct LispDatum* a = new_integer(32);
  struct LispDatum* b = new_integer(32);

  CuAssert(tc, "32 = 32", datum_cmp(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_integer(-9);
  b = new_integer(-9);

  CuAssert(tc, "-9 = -9", datum_cmp(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_integer(0);
  b = new_integer(0);

  CuAssert(tc, "0 = 0", datum_cmp(a, b));

  discard_datum(a);
  discard_datum(b);
}

void Test_int_inequality(CuTest* tc) {
  struct LispDatum* a = new_integer(10);
  struct LispDatum* b = new_integer(12);

  CuAssert(tc, "10 != 12", !datum_cmp(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_integer(-6);
  b = new_integer(-7);

  CuAssert(tc, "-6 != -7", !datum_cmp(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_integer(6);
  b = new_integer(-6);

  CuAssert(tc, "6 != -6", !datum_cmp(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_integer(-7);
  b = new_integer(7);

  CuAssert(tc, "-7 != 7", !datum_cmp(a, b));

  discard_datum(a);
  discard_datum(b);
}

void Test_float_equality(CuTest* tc) {
  struct LispDatum* a = new_real(5.0);
  struct LispDatum* b = new_real(5.0);

  CuAssert(tc, "5.0 = 5.0", datum_cmp(a, b));

  a->float_val *= -1;
  b->float_val *= -1;

  CuAssert(tc, "-5.0 = -5.0", datum_cmp(a, b));

  a->float_val *= 0;
  b->float_val *= 0;

  CuAssert(tc, "0 = 0", datum_cmp(a, b));

  discard_datum(a);
  discard_datum(b);
}

void Test_float_inequality(CuTest* tc) {
  struct LispDatum* a = new_real(10.0);
  struct LispDatum* b = new_real(12.0);

  CuAssert(tc, "10.0 != 12.0", !datum_cmp(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_real(10.0);
  b = new_real(10.01);

  CuAssert(tc, "10.0 != 10.01", !datum_cmp(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_real(-841.0);
  b = new_real(-76.0);

  CuAssert(tc, "-841.0 != -76.0", !datum_cmp(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_real(-841.25);
  b = new_real(-841.25005);

  CuAssert(tc, "-841.25 != -841.25005", !datum_cmp(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_real(-841.25);
  b = new_real(841.25);

  CuAssert(tc, "-841.25 != 841.25", !datum_cmp(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_real(841.25);
  b = new_real(-841.25);

  CuAssert(tc, "841.25 != -841.25", !datum_cmp(a, b));

  discard_datum(a);
  discard_datum(b);
}

void Test_rational_equality(CuTest* tc) {
  struct LispDatum* a = new_rational(2, 3);
  struct LispDatum* b = new_rational(4, 6);

  CuAssert(tc, "2/3 = 4/6", datum_cmp(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_rational(1, 2);
  b = new_rational(1, 2);

  CuAssert(tc, "1/2 = 1/2", datum_cmp(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_rational(5, 6);
  b = new_rational(-5, -6);

  CuAssert(tc, "5/6 = -5/-6", datum_cmp(a, b));

  discard_datum(a);
  discard_datum(b);
}

void Test_rational_inequality(CuTest* tc) {
  CuAssert(tc, "1/2 != -1/2", !datum_cmp(new_rational(1, 2), new_rational(-1, 2)));
  CuAssert(tc, "3/4 != 1/2", !datum_cmp(new_rational(3, 4), new_rational(1, 2)));
}

void Test_complex_equality(CuTest* tc) {
  CuAssert(tc, "1+2i = 1+2i", datum_cmp(new_complex(1, 2), new_complex(1, 2)));
}

void Test_complex_inequality(CuTest* tc) {
  CuAssert(tc, "1+2i != -1+2i", !datum_cmp(new_complex(1, 2), new_complex(-1, 2)));
  CuAssert(tc, "3+4i != 1+2i", !datum_cmp(new_complex(3, 4), new_complex(1, 2)));
  CuAssert(tc, "1-i != 1+i", !datum_cmp(new_complex(1, -1), new_complex(1, 1)));
}

void Test_promotion_equality_int(CuTest* tc) {
  struct LispDatum* a = new_integer(1);
  struct LispDatum* b = new_real(1.0);

  CuAssert(tc, "1 = 1.0", datum_cmp(a, b));

  discard_datum(b);

  b = new_rational(1, 1);

  CuAssert(tc, "1 = 1/1", datum_cmp(a, b));

  discard_datum(b);

  b = new_complex(1.0, 0);

  CuAssert(tc, "1 = 1+0i", datum_cmp(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_integer(-1);
  b = new_real(-1.0);

  CuAssert(tc, "-1 = -1.0", datum_cmp(a, b));

  discard_datum(b);

  b = new_rational(-1, 1);

  CuAssert(tc, "-1 = -1/1", datum_cmp(a, b));

  discard_datum(b);

  b = new_complex(-1.0, 0);

  CuAssert(tc, "-1 = -1+0i", datum_cmp(a, b));

  discard_datum(a);
  discard_datum(b);
}

void Test_promotion_equality_rational(CuTest* tc) {
  struct LispDatum* a = new_rational(1, 1);
  struct LispDatum* b = new_real(1.0);

  CuAssert(tc, "1/1 = 1.0", datum_cmp(a, b));

  discard_datum(b);

  b = new_complex(1.0, 0);

  CuAssert(tc, "1/1 = 1+0i", datum_cmp(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_rational(-1, 1);
  b = new_real(-1.0);

  CuAssert(tc, "-1/1 = -1.0", datum_cmp(a, b));

  discard_datum(b);

  b = new_complex(-1.0, 0);

  CuAssert(tc, "-1/1 = -1+0i", datum_cmp(a, b));

  discard_datum(a);
  discard_datum(b);
}

void Test_promotion_equality_real(CuTest* tc) {
  struct LispDatum* a = new_real(1.0);
  struct LispDatum* b = new_complex(1.0, 0);

  CuAssert(tc, "1.0 = 1.0+0i", datum_cmp(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_real(-1.0);
  b = new_complex(-1.0, 0);

  CuAssert(tc, "-1.0 = -1+0i", datum_cmp(a, b));

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

  CuAssert(tc, "-3 + 2 = -1", datum_cmp(add(args, 2), new_integer(-1)));

  args[1] = new_rational(4, 5);
  CuAssert(tc, "-3 + 4/5 = -11/5", datum_cmp(add(args, 2), new_rational(-11, 5)));

  args[1] = new_real(3.5);
  CuAssert(tc, "-3 + 3.5 = 0.5", datum_cmp(add(args, 2), new_real(0.5)));

  args[1] = new_complex(3, 1);
  CuAssert(tc, "-3 + 3+i = i", datum_cmp(add(args, 2), new_complex(0, 1)));
}

void Test_real_addition(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_real(-3);
  args[1] = new_integer(2);

  CuAssert(tc, "-3.0 + 2 = -1", datum_cmp(add(args, 2), new_real(-1)));

  args[1] = new_rational(4, 5);
  CuAssert(tc, "-3.0 + 4/5 = -2.2", datum_cmp(add(args, 2), new_real(-2.2)));

  args[1] = new_real(3.5);
  CuAssert(tc, "-3.0 + 3.5 = 0.5", datum_cmp(add(args, 2), new_real(0.5)));

  args[1] = new_complex(3, 1);
  CuAssert(tc, "-3.0 + 3+i = i", datum_cmp(add(args, 2), new_complex(0, 1)));
}

void Test_rational_addition(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_rational(-3, 1);
  args[1] = new_integer(2);

  CuAssert(tc, "-3/1 + 2 = -1", datum_cmp(add(args, 2), new_rational(-1, 1)));

  args[1] = new_rational(4, 5);
  CuAssert(tc, "-3/1 + 4/5 = -11/5", datum_cmp(add(args, 2), new_rational(-11, 5)));

  args[1] = new_real(3.5);
  CuAssert(tc, "-3/1 + 3.5 = 0.5", datum_cmp(add(args, 2), new_real(0.5)));

  args[1] = new_complex(3, 1);
  CuAssert(tc, "-3/1 + 3+i = i", datum_cmp(add(args, 2), new_complex(0, 1)));
}

void Test_complex_addition(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_complex(-3, 0);
  args[1] = new_integer(2);

  CuAssert(tc, "-3+0i + 2 = -1", datum_cmp(add(args, 2), new_complex(-1, 0)));

  args[1] = new_rational(4, 5);
  CuAssert(tc, "-3+0i + 4/5 = -11/5", datum_cmp(add(args, 2), new_complex(-2.2, 0)));

  args[1] = new_real(3.5);
  CuAssert(tc, "-3+0i + 3.5 = 0.5", datum_cmp(add(args, 2), new_complex(0.5, 0)));

  args[1] = new_complex(3, 1);
  CuAssert(tc, "-3+0i + 3+i = i", datum_cmp(add(args, 2), new_complex(0, 1)));
}

void Test_int_subtraction(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_integer(16);
  args[1] = new_integer(4);

  CuAssert(tc, "16 - 4 = 12", datum_cmp(subtract(args, 2), new_integer(12)));

  args[1] = new_rational(9, 2);
  CuAssert(tc, "16 - 9/2 = 23/2", datum_cmp(subtract(args, 2), new_rational(23, 2)));

  args[1] = new_real(-4.5);
  CuAssert(tc, "16 - -4.5 = 20.5", datum_cmp(subtract(args, 2), new_real(20.5)));

  args[1] = new_complex(9, 2);
  CuAssert(tc, "16 - 9+2i = 7-2i", datum_cmp(subtract(args, 2), new_complex(7, -2)));
}

void Test_real_subtraction(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_real(16);
  args[1] = new_integer(4);

  CuAssert(tc, "16.0 - 4 = 12.0", datum_cmp(subtract(args, 2), new_real(12)));

  args[1] = new_rational(9, 2);
  CuAssert(tc, "16.0 - 9/2 = 11.5", datum_cmp(subtract(args, 2), new_real(11.5)));

  args[1] = new_real(-4.5);
  CuAssert(tc, "16.0 - -4.5 = 20.5", datum_cmp(subtract(args, 2), new_real(20.5)));

  args[1] = new_complex(9, 2);
  CuAssert(tc, "16.0 - 9+2i = 7-2i", datum_cmp(subtract(args, 2), new_complex(7, -2)));
}

void Test_rational_subtraction(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_rational(16, 1);
  args[1] = new_integer(4);

  CuAssert(tc, "16/1 - 4 = 12/1", datum_cmp(subtract(args, 2), new_rational(12, 1)));

  args[1] = new_rational(9, 2);
  CuAssert(tc, "16/1 - 9/2 = 21/2", datum_cmp(subtract(args, 2), new_rational(23, 2)));

  args[1] = new_real(-4.5);
  CuAssert(tc, "16/1 - -4.5 = 20.5", datum_cmp(subtract(args, 2), new_real(20.5)));

  args[1] = new_complex(9, 2);
  CuAssert(tc, "16/1 - 9+2i = 7-2i", datum_cmp(subtract(args, 2), new_complex(7, -2)));
}

void Test_complex_subtraction(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_complex(16, 0);
  args[1] = new_integer(4);

  CuAssert(tc, "16/1 - 4 = 12/1", datum_cmp(subtract(args, 2), new_complex(12, 0)));

  args[1] = new_rational(9, 2);
  CuAssert(tc, "16/1 - 9/2 = 23/2", datum_cmp(subtract(args, 2), new_complex(11.5, 0)));

  args[1] = new_real(-4.5);
  CuAssert(tc, "16/1 - -4.5 = 20.5", datum_cmp(subtract(args, 2), new_complex(20.5, 0)));

  args[1] = new_complex(9, 2);
  CuAssert(tc, "16/1 - 9+2i = 7-2i", datum_cmp(subtract(args, 2), new_complex(7, -2)));
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

  CuAssert(tc, "5 * 7 = 35", datum_cmp(multiply(args, 2), new_integer(35)));

  args[0]->int_val = -5;
  args[1]->int_val = -7;

  CuAssert(tc, "-5 * -7 = 35", datum_cmp(multiply(args, 2), new_integer(35)));

  args[0]->int_val = 5;
  args[1]->int_val = -7;

  CuAssert(tc, "5 * -7 = -35", datum_cmp(multiply(args, 2), new_integer(-35)));

  args[0]->int_val = -5;
  args[1]->int_val = 7;

  CuAssert(tc, "-5 * 7 = -35", datum_cmp(multiply(args, 2), new_integer(-35)));

  args[0]->int_val = 2;
  args[1]->int_val = 2;
  args[2]->int_val = 2;

  CuAssert(tc, "2 * 2 * 2 = 8", datum_cmp(multiply(args, 3), new_integer(8)));

  args[0]->int_val = -2;
  args[1]->int_val = -2;
  args[2]->int_val = -2;

  CuAssert(tc, "-2 * -2 * -2 = -8", datum_cmp(multiply(args, 3), new_integer(-8)));
}

void Test_int_real_multiplication(CuTest* tc) {
  struct LispDatum* args[3];
  args[0] = new_integer(5);
  args[1] = new_real(7.0);
  args[2] = new_real(2.0);

  CuAssert(tc, "5 * 7 = 35", datum_cmp(multiply(args, 2), new_real(35.0)));

  args[0]->int_val = -5;
  args[1]->float_val = -7.0;

  CuAssert(tc, "-5 * -7 = 35", datum_cmp(multiply(args, 2), new_real(35.0)));

  args[0]->int_val = 5;
  args[1]->float_val = -7;

  CuAssert(tc, "5 * -7 = -35", datum_cmp(multiply(args, 2), new_real(-35.0)));

  args[0]->int_val = -5;
  args[1]->float_val = 7;

  CuAssert(tc, "-5 * 7 = -35", datum_cmp(multiply(args, 2), new_real(-35.0)));

  args[0]->int_val = 2;
  args[1]->float_val = 2;
  args[2]->float_val = 2;

  CuAssert(tc, "2 * 2 * 2 = 8", datum_cmp(multiply(args, 3), new_real(8.0)));

  args[0]->int_val = -2;
  args[1]->float_val = -2;
  args[2]->float_val = -2;

  CuAssert(tc, "-2 * -2 * -2 = -8", datum_cmp(multiply(args, 3), new_real(-8.0)));
}

void Test_int_rational_multiplication(CuTest* tc) {
  // TODO(matthew-c21): Expand with relevant math.
  struct LispDatum* args[3];
  args[0] = new_integer(5);
  args[1] = new_rational(7, 3);
  args[2] = new_rational(2, 1);

  CuAssert(tc, "5 * 7 = 35", datum_cmp(multiply(args, 2), new_rational(35, 3)));

  args[0]->int_val = -5;
  args[1]->num = -7;

  CuAssert(tc, "-5 * -7 = 35", datum_cmp(multiply(args, 2), new_rational(35, 3)));

  args[0]->int_val = 5;
  args[1]->num = -7;

  CuAssert(tc, "5 * -7 = -35", datum_cmp(multiply(args, 2), new_rational(-35, 3)));

  args[0]->int_val = -5;
  args[1]->num = 7;

  CuAssert(tc, "-5 * 7 = -35", datum_cmp(multiply(args, 2), new_rational(-35, 3)));

  args[0]->int_val = 2;
  args[1]->num = 2;
  args[2]->num = 2;

  CuAssert(tc, "2 * 2 * 2 = 8", datum_cmp(multiply(args, 3), new_rational(8, 3)));

  args[0]->int_val = -2;
  args[1]->num = -2;
  args[2]->num = -2;

  CuAssert(tc, "-2 * -2 * -2 = -8", datum_cmp(multiply(args, 3), new_rational(-8, 3)));
}

void Test_int_complex_multiplication(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_integer(16);
  args[1] = new_complex(2, 3);

  CuAssert(tc, "16 * 2+3i = 32 + 48i", datum_cmp(multiply(args, 2), new_complex(32, 48)));

  args[0]->int_val = -1;
  args[1]->im = -4;

  CuAssert(tc, "-1 * 2-4i = -2+4i", datum_cmp(multiply(args, 2), new_complex(-2, 4)));
}

void Test_real_int_multiplication(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_real(2.5);
  args[1] = new_integer(4);

  CuAssert(tc, "4 * 2.5 = 10.0", datum_cmp(multiply(args, 2), new_real(10)));
  tc = NULL;
}

void Test_real_real_multiplication(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_real(2.5);
  args[1] = new_real(1.75);

  CuAssert(tc, "1.75 * 2.5 = 4.375", datum_cmp(multiply(args, 2), new_real(2.5 * 1.75)));
}

void Test_real_rational_multiplication(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_real(2.5);
  args[1] = new_rational(7, 4);

  CuAssert(tc, "7/4 * 2.5 = 4.375", datum_cmp(multiply(args, 2), new_real(2.5 * 1.75)));
}

void Test_real_complex_multiplication(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_real(-2.5);
  args[1] = new_complex(3.25, -4.5);

  CuAssert(tc, "3.25-4.5i * -2.5 = -8.125+11.25i", datum_cmp(multiply(args, 2), new_complex(-8.125, 11.25)));
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

  CuAssert(tc, "-2/5 * 2.5 = -1.0", datum_cmp(multiply(args, 2), new_real(-1.0)));
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

  CuAssert(tc, "-7/2 * 2-5.2i = -7+18.2i", datum_cmp(multiply(args, 2), new_complex(-7, 18.2)));
}

void Test_complex_int_multiplication(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_complex(-2, 5);
  args[1] = new_integer(4);

  CuAssert(tc, "-2+5i * 4 = -8+20i", datum_cmp(multiply(args, 2), new_complex(-8, 20)));
}

void Test_complex_real_multiplication(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_complex(-2, 5);
  args[1] = new_real(-0.5);

  CuAssert(tc, "-2+5i * -0.5 = 1-2.5i", datum_cmp(multiply(args, 2), new_complex(1, -2.5)));
}

void Test_complex_rational_multiplication(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_complex(-2, 5);
  args[1] = new_rational(-1, 2);

  CuAssert(tc, "-2+5i * -1/2 = 1-2.5i", datum_cmp(multiply(args, 2), new_complex(1, -2.5)));
}

void Test_complex_complex_multiplication(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_complex(-2, 5);
  args[1] = new_complex(-2, -5);

  CuAssert(tc, "-2+5i * -2-5i = -4", datum_cmp(multiply(args, 2), new_complex(29, 0)));
}

void Test_multiplication(CuTest* tc) {
  struct LispDatum* args[4];
  args[0] = new_integer(3);
  args[1] = new_rational(1, 2);
  args[2] = new_real(-3.5);
  args[3] = new_complex(-4.25, 6.125);

  CuAssert(tc, "3 * 1/2 * -3.5 * -4.25+6.125i = ", datum_cmp(multiply(args, 4), new_complex(22.3125, -32.15625)));
}

void Test_bool_equality(CuTest* tc) {
  struct LispDatum* true = get_true();
  struct LispDatum* false = get_false();

  CuAssert(tc, "#t = #t", datum_cmp(true, get_true()));
  CuAssert(tc, "#f = #f", datum_cmp(false, get_false()));

  CuAssertIntEquals(tc, 1, true->boolean ? 1 : 0);
  CuAssertIntEquals(tc, 0, false->boolean ? 1 : 0);

  CuAssertIntEquals(tc, Bool, true->type);
  CuAssertIntEquals(tc, Bool, false->type);
}

void Test_bool_inequality(CuTest* tc) {
  struct LispDatum* true = get_true();
  struct LispDatum* false = get_false();
  struct LispDatum* true_string = new_string("");
  struct LispDatum* false_string = new_string("abc");
  struct LispDatum* true_num = new_integer(1);
  struct LispDatum* false_num = new_integer(0);

  CuAssert(tc, "#t != 1", !datum_cmp(true, true_num));
  CuAssert(tc, "#t != \"abc\"", !datum_cmp(true, true_string));
  CuAssert(tc, "#f != 0", !datum_cmp(false, false_num));
  CuAssert(tc, "#f != \"\"", !datum_cmp(false, false_string));
  CuAssert(tc, "#t != nil", !datum_cmp(true, get_nil()));
  CuAssert(tc, "#f != nil", !datum_cmp(false, get_nil()));
}

void Test_discard_statics(CuTest* tc) {
  struct LispDatum* true = get_true();
  struct LispDatum* false = get_false();
  struct LispDatum* nil = get_nil();

  discard_datum(true);
  discard_datum(false);
  discard_datum(nil);

  true = get_true();
  false = get_false();
  nil = get_nil();

  CuAssert(tc, "true exists", true->boolean);
  CuAssert(tc, "false exists", !false->boolean);
  CuAssert(tc, "nil exists", nil->type == Nil);
}

void Test_string_equality(CuTest* tc) {
  struct LispDatum* a = new_string("abc");
  struct LispDatum* b = new_string("abc");

  CuAssert(tc, "abc = abc", datum_cmp(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_string("");
  b = new_string("");

  CuAssert(tc, "'' = ''", datum_cmp(a, b));
}

void Test_string_inequality(CuTest* tc) {
  struct LispDatum* a = new_string("abc");
  struct LispDatum* b = new_string("ABC");

  CuAssert(tc, "abc != ABC", !datum_cmp(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_string("abc");
  b = new_string("abcd");

  CuAssert(tc, "abc != abcd", !datum_cmp(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_string("zabc");
  b = new_string("abc");

  CuAssert(tc, "zabc != abc", !datum_cmp(a, b));

  discard_datum(a);
  discard_datum(b);

  a = new_string("");
  b = new_string("abc");

  CuAssert(tc, "'' != abc", !datum_cmp(a, b));
}

void Test_variable_eqv(CuTest* tc) {
  struct LispDatum* a = new_integer(0);
  struct LispDatum* b = new_rational(0, 1);
  struct LispDatum* c = new_real(0.0);
  struct LispDatum* d = new_complex(1.0, 0.0);

  struct LispDatum* args[4];
  args[0] = a;
  args[1] = b;
  args[2] = c;
  args[3] = d;

  CuAssert(tc, "empty true", datum_cmp(eqv(args, 0), get_true()));
  CuAssert(tc, "single value true", datum_cmp(eqv(args, 1), get_true()));
  CuAssert(tc, "0 = 0/1", datum_cmp(eqv(args, 2), get_true()));
  CuAssert(tc, "0 = 0/1 = 0.0", datum_cmp(eqv(args, 3), get_true()));
  CuAssert(tc, "0 = 0/1 = 0.0 != 1+0i", datum_cmp(eqv(args, 4), get_false()));
}

void Test_empty_list(CuTest* tc) {
  struct LispDatum* empty_list = list(NULL, 0);
  CuAssert(tc, "(list) != nil", !datum_cmp(get_nil(), empty_list));
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

  assert_empty_list(tc, "(cdr nil) == ()", cdr(&nil, 1));

  struct LispDatum* null_result = car(&nil, 1);
  CuAssert(tc, "(car nil) fails", null_result == NULL);
  CuAssert(tc, "Error state raised", GlobalErrorState);

  raise(None, "Error state reset.");
}

void Test_car(CuTest* tc) {
  struct LispDatum* args[3];

  for (int i = 0; i < 3; ++i) {
    args[i] = new_integer(i + 1);
  }

  struct LispDatum* alist = list(args, 3);

  // Neither `list` nor `car` should produce a copy of it's elements, so this is a stronger condition than `eqv`.
  struct LispDatum* result = car(&alist, 1);
  CuAssert(tc, "(car '(1 2 3)) = 1", result == args[0]);
}

void Test_cdr(CuTest* tc) {
  struct LispDatum* args[3];

  for (int i = 0; i < 3; ++i) {
    args[i] = new_integer(i + 1);
  }

  struct LispDatum* alist = list(args, 3);

  struct LispDatum* output = cdr(&alist, 1);

  CuAssert(tc, "result is a list", output->type == Cons);
  CuAssert(tc, "(car (cdr '(1 2 3)) == 2", output->car == args[1]);
  CuAssert(tc, "(car (cdr (cdr '(1 2 3))) == 3", output->cdr->car == args[2]);
  CuAssert(tc, "(cdr (cdr (cdr '(1 2 3))) == NULL", output->cdr->cdr == NULL);
}

void Test_cadr_errors(CuTest* tc) {
  // Fail without enough arguments.
  CuAssertPtrEquals(tc, NULL, car(NULL, 0));
  CuAssert(tc, "Too few arguments to `car` fail", GlobalErrorState == Argument);

  raise(None, NULL);

  CuAssertPtrEquals(tc, NULL, cdr(NULL, 0));
  CuAssert(tc, "Too few arguments to `cdr` fail", GlobalErrorState == Argument);

  raise(None, NULL);

  // Fail with too many arguments. NULL is used because the arguments shouldn't even be accessed.
  CuAssertPtrEquals(tc, NULL, car(NULL, 0));
  CuAssert(tc, "Too few arguments to `car` fail", GlobalErrorState == Argument);

  raise(None, NULL);

  CuAssertPtrEquals(tc, NULL, cdr(NULL, 2));
  CuAssert(tc, "Too many arguments to `cdr` fail", GlobalErrorState == Argument);

  raise(None, NULL);

  // Fail with non-list arguments.
  struct LispDatum* bad_arg = new_string("fubar");

  CuAssertPtrEquals(tc, NULL, car(&bad_arg, 1));
  CuAssert(tc, "`car` fails with non-list arg", GlobalErrorState == Type);

  raise(None, NULL);

  CuAssertPtrEquals(tc, NULL, cdr(&bad_arg, 1));
  CuAssert(tc, "`cdr` fails with non-list arg", GlobalErrorState == Type);

  raise(None, NULL);
}

void Test_length(CuTest* tc) {
  struct LispDatum* nil = get_nil();
  // Nil
  struct LispDatum* l = length(&nil, 1);
  CuAssert(tc, "(len nil) == 0", l->type == Integer);
  CuAssert(tc, "(len nil) == 0", l->int_val == 0);

  // Empty List
  struct LispDatum* empty_list = list(NULL, 0);
  l = length(&empty_list, 1);
  CuAssertTrue(tc, l->type == Integer);
  CuAssertTrue(tc, l->int_val == 0);

  // Standard list
  struct LispDatum* args[2];
  args[0] = new_integer(1);
  args[1] = new_integer(2);

  struct LispDatum* alist = list(args, 2);

  l = length(&alist, 1);
  CuAssertTrue(tc,l->type == Integer);
  CuAssertTrue(tc, l->int_val == 2);

  // String
  discard_datum(args[0]);
  discard_datum(args[1]);

  args[0] = new_string("hello");

  l = length(args, 1);
  CuAssertTrue(tc, l->type == Integer);
  CuAssertTrue(tc, l->int_val == 5);
}

void Test_length_errors(CuTest* tc) {
  // Not enough args
  AssertThrows(length(NULL, 0), Argument);

  // Too many args
  AssertThrows(length(NULL, 2), Argument);

  // Non-list value.
  struct LispDatum* arg = new_integer(0);
  AssertThrows(length(&arg, 1), Type);

  // Pair
  struct LispDatum* args[2];
  args[0] = new_integer(1);
  args[1] = new_integer(2);

  struct LispDatum* pair = cons(args, 2);
  AssertThrows(length(&pair, 1), Type);
}

void Test_append(CuTest* tc) {
  // Boilerplate for rest of calls.
  struct LispDatum* digits[10];

  for (int i = 0; i < 10; ++i) {
    digits[i] = new_integer(i);
  }

  // 0-3
  struct LispDatum* list1 = list(digits, 4);

  // 4-6
  struct LispDatum* list2 = list(digits + 4, 3);

  // 7-9
  struct LispDatum* list3 = list(digits + 7, 3);

  struct LispDatum* args[4];
  args[0] = list1;
  args[1] = list2;
  args[2] = list3;

  // No arguments - empty list.
  struct LispDatum* result = append(NULL, 0);
  CuAssertTrue(tc, result->type == Cons);
  CuAssertPtrEquals(tc, NULL, result->car);
  CuAssertPtrEquals(tc, NULL, result->cdr);

  // One nil argument returns nil.
  struct LispDatum* nil = get_nil();
  result = append(&nil, 1);
  CuAssertPtrEquals(tc, get_nil(), result);

  // One argument - returned as is without creating new list.
  result = append(&list1, 1);
  CuAssertTrue(tc, result == list1);

  // Generic case.
  result = append(args, 3);

  CuAssertTrue(tc, result->type == Cons);

  struct LispDatum* idx = result;

  for (int i = 0; i < 10; ++i) {
    CuAssertTrue(tc, idx->car == digits[i]);
    idx = idx->cdr;
  }

  CuAssertPtrEquals(tc, args[3], idx);

  // Nil in middle of list.
  args[3] = args[2];
  args[2] = get_nil();
  result = append(args, 4);

  CuAssertTrue(tc, result->type == Cons);

  idx = result;

  for (int i = 0; i < 10; ++i) {
    CuAssertTrue(tc, idx->car == digits[i]);
    idx = idx->cdr;
  }

  CuAssertPtrEquals(tc, NULL, idx);
}

void Test_append_errors(CuTest* tc) {
  struct LispDatum* args[3];
  args[0] = new_integer(0);
  args[1] = list(NULL, 0);

  AssertThrows(append(args, 1), Type);

  // Swap and add nil.
  args[2] = args[0];
  args[0] = args[1];
  args[1] = args[2];
  args[2] = get_nil();

  AssertThrows(append(args, 3), Type);
}

void Test_cons(CuTest* tc) {
  struct LispDatum* args[2];
  args[0] = new_integer(1);
  args[1] = new_string("asdf");

  struct LispDatum* pair = cons(args, 2);

  CuAssertTrue(tc, pair->type == Cons);
  CuAssertTrue(tc, pair->car == args[0]);
  CuAssertTrue(tc, pair->cdr == args[1]);
}

void Test_cons_errors(CuTest* tc) {
  // Too few
  AssertThrows(cons(NULL, 0), Argument);
  AssertThrows(cons(NULL, 1), Argument);

  // Too many
  AssertThrows(cons(NULL, 3), Argument);
}

void Test_reverse(CuTest* tc) {
  // Nil
  struct LispDatum* nil = get_nil();
  CuAssertPtrEquals(tc, get_nil(), reverse(&nil, 1));

  // Empty
  struct LispDatum* empty = list(NULL, 0);
  CuAssertPtrEquals(tc, empty, reverse(&empty, 1));

  // Single Element
  struct LispDatum* monolith = list(&nil, 1);
  CuAssertPtrEquals(tc, monolith, reverse(&monolith, 1));

  // Default
  struct LispDatum* items[4];
  for (int i = 0; i < 4; ++i) {
    items[i] = new_integer(i);
  }

  struct LispDatum* item_list = list(items, 4);

  struct LispDatum* reversed = reverse(&item_list, 1);
  CuAssertTrue(tc, reversed->type == Cons);

  struct LispDatum* idx = reversed;

  for (int i = 0; i < 4; ++i) {
    CuAssertPtrEquals(tc, items[3-i], idx->car);
    idx = idx->cdr;
  }

  CuAssertTrue(tc, idx == NULL);
}

void Test_reverse_errors(CuTest* tc) {
  // Too few
  AssertThrows(reverse(NULL, 0), Argument);

  // Too many
  AssertThrows(reverse(NULL, 2), Argument);

  // Wrong type
  struct LispDatum* bad_arg = new_string("fubar");
  AssertThrows(reverse(&bad_arg, 1), Type)

  // Pair
  struct LispDatum* pair_args[2];
  pair_args[0] = bad_arg;
  pair_args[1] = get_nil();

  struct LispDatum* bad_pair = cons(pair_args, 2);
  AssertThrows(reverse(&bad_pair, 1), Type);
}
