#include "postgres.h"

#include "libpq/libpq.h"
#include "miscadmin.h"
#include "postmaster/postmaster.h"
#include "tcop/tcopprot.h"

__thread const char *debug_query_string;
__thread CommandDest whereToSendOutput = DestDebug;
#if PG_VERSION_NUM < 160000
__thread bool ClientAuthInProgress = false;
#else
bool ClientAuthInProgress = false;
#endif
const PQcommMethods *PqCommMethods = NULL;

#if PG_VERSION_NUM < 180000
__thread int max_stack_depth = 100;
__thread char *stack_base_ptr = NULL;
static __thread long max_stack_depth_bytes = 100 * 1024L;
#endif

void
ProcessInterrupts(void)
{
}

#if PG_VERSION_NUM < 180000
pg_stack_base_t
set_stack_base(void)
{
	char *old = stack_base_ptr;
	stack_base_ptr = (char *) __builtin_frame_address(0);
	return old;
}

void
restore_stack_base(pg_stack_base_t base)
{
	stack_base_ptr = base;
}

bool
stack_is_too_deep(void)
{
	char stack_top;
	long depth;

	if (stack_base_ptr == NULL)
		stack_base_ptr = (char *) __builtin_frame_address(0);
	depth = (long) (stack_base_ptr - &stack_top);
	if (depth < 0)
		depth = -depth;
	return depth > max_stack_depth_bytes;
}

void
check_stack_depth(void)
{
	if (stack_is_too_deep())
		ereport(ERROR,
				(errcode(ERRCODE_STATEMENT_TOO_COMPLEX),
				 errmsg("stack depth limit exceeded"),
				 errhint("Reduce query nesting depth.")));
}
#endif
