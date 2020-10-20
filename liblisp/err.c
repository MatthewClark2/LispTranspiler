#include <stdio.h>
#include <stdlib.h>

#include "err.h"

static char* cause_string(enum Cause cause) {
  switch (cause) {
    case ZeroDivision: return "Division by Zero";
    case Math: return "Math Exception";
    case Generic: return "Runtime Exception";
    default: return "Unknown exception";
  }
}

void* raise(enum Cause cause, const char* msg) {
  fprintf(stderr, "%s: %s\n", cause_string(cause), msg);
  exit(-1);
}
