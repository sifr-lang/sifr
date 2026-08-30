/*
 * PostgreSQL's generated pg_config.h enables spinlocks, but its portable
 * header has no wasm architecture branch. Components execute single-threaded
 * today; compiler atomics still preserve the lock contract if that changes.
 */
#ifndef S_LOCK_H
#define S_LOCK_H

typedef int slock_t;

static inline int
sifr_wasi_spin_try_lock(volatile slock_t *lock)
{
	slock_t expected = 0;
	return __atomic_compare_exchange_n(lock, &expected, 1, false,
						   __ATOMIC_ACQUIRE, __ATOMIC_RELAXED) ? 0 : 1;
}

#define S_INIT_LOCK(lock) __atomic_store_n((lock), 0, __ATOMIC_RELAXED)
#define S_LOCK_FREE(lock) (__atomic_load_n((lock), __ATOMIC_RELAXED) == 0)
#define S_UNLOCK(lock) __atomic_store_n((lock), 0, __ATOMIC_RELEASE)
#define TAS(lock) sifr_wasi_spin_try_lock(lock)
#define TAS_SPIN(lock) TAS(lock)
#define SPIN_DELAY() ((void) 0)
#define S_LOCK(lock) \
	({ int _delays = 0; while (TAS(lock)) { _delays++; } _delays; })

#endif /* S_LOCK_H */
