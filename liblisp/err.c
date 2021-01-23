#include <stdio.h>
#include <stdlib.h>

#include "err.h"

/**
 * Error state value. This value should be treated as readonly by client code. Should multi-threaded behavior ever
 * become a thing, this will need to be more complex.
 */
static enum Cause GlobalErrorState = None;

static enum ErrorBehavior GlobalErrorBehavior = LogOnly;

static void destroy_and_exit() {
  // TODO(matthew-c21): If necessary, add resource handles here to be closed before exiting.
  exit(-1);
}

static char* cause_string(enum Cause cause) {
  switch (cause) {
    case ZeroDivision:
      return "Division by Zero";
    case Math:
      return "Math Exception";
    case Generic:
      return "Runtime Exception";
    case Type:
      return "Type Mismatch Exception";
    case Argument:
      return "Invalid Argument Exception";
    default:
      return "Unknown exception";
  }
}

void* raise(enum Cause cause, const char* msg) {
  fprintf(stderr, "%s: %s\n", cause_string(cause), msg);
  GlobalErrorState = cause;

  switch (GlobalErrorBehavior) {
    case LogAndQuit:
      destroy_and_exit();
      break;
    case LogOnly:
      break;
    default:
      fprintf(stderr, "Illegal error behavior detected. Exiting immediately.");
      destroy_and_exit();
  }

  return NULL;
}
