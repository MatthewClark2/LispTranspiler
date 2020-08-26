#include "stdlisp.c"
#include "stdlisp.h"

int main() {
  struct LispDatum destination;

  copy_lisp_datum(get_nil(), &destination);
  display(&destination);
}
