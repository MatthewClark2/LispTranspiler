#include <stdio.h>
#include <stdlib.h>

#include "err.h"

// TODO(matthew-c21): Unit testing.
/**
 * Error state value. This value should be treated as readonly by client code. Should multi-threaded behavior ever
 * become a thing, this will need to be more complex.
 */
enum Cause GlobalErrorState = None;

// This is left static because it shouldn't ever need to be read externally.
// TODO(matthew-c21): This isn't a particularly reasonable default.
static enum ErrorBehavior GlobalErrorBehavior = LogOnly;

static void destroy_and_exit() {
  // TODO(matthew-c21): If necessary, add resource handles here to be closed before exiting.
  exit(-1);
}

static char* cause_string(enum Cause cause) {
  switch (cause) {
    case None:
      return "Info";
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

void set_global_error_behavior(enum ErrorBehavior behavior) {
  GlobalErrorBehavior = behavior;
}
