#ifndef SIFR_WASI_SETJMP_H
#define SIFR_WASI_SETJMP_H

#include_next <setjmp.h>

/* WASI SDK implements plain setjmp through its SJLJ runtime. */
#undef sigsetjmp
#undef siglongjmp
#define sigsetjmp(buffer, save_mask) setjmp(buffer)
#define siglongjmp(buffer, value) longjmp((buffer), (value))

#endif
