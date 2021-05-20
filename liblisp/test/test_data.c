#include "../err.h"
#include "CuTest.h"
#include "../data.h"

// TODO(matthew-c21): Test factory methods.

void Test_nil_false(CuTest* tc) {
  CuAssertTrue(tc, !truthy(get_nil()));
}

void Test_truthy_types(CuTest* tc) {
  // Empty string is true.
  CuAssertTrue(tc, truthy(new_string("")));

  // Zeros are true.
  CuAssertTrue(tc, truthy(new_integer(0)));
  CuAssertTrue(tc, truthy(new_rational(0, 1)));
  CuAssertTrue(tc, truthy(new_complex(0, 0)));
  CuAssertTrue(tc, truthy(new_real(0)));

  // Empty keyword/symbol true.
  CuAssertTrue(tc, truthy(new_symbol("")));
  CuAssertTrue(tc, truthy(new_keyword("")));

  // Empty list true.
  CuAssertTrue(tc, truthy(new_cons(NULL, NULL)));

  // Lambda true.
  CuAssertTrue(tc, truthy(new_lambda(&eqv, NULL, 0, NULL)));
}

void Test_falsy_values(CuTest* tc) {
  CuAssertTrue(tc, !truthy(get_false()));
}
