"""Audited measurement-only runner closure shared by independent endpoints."""
RUNTIME_PATHS = tuple('verification/runner/sifr_verify/'+name for name in (
    '__init__.py', 'process_execution.py', 'process_supervisor.py', 'process_disk_budget.py',
))
