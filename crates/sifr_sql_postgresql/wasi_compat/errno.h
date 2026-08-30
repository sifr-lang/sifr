#ifndef SIFR_WASI_ERRNO_H
#define SIFR_WASI_ERRNO_H

#include_next <errno.h>

/* PostgreSQL names this non-WASI network error in strerror tables. */
#ifndef EHOSTDOWN
#define EHOSTDOWN 77
#endif

#endif
