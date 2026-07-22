from __future__ import annotations

ORDINARY_PYTHON_API_ALLOWED_IMPORTS = {
    "ExitCause",
    "ExitCauseKind",
    "ExitDecision",
    "PythonError",
    "ResourceDiagnostics",
    "resource_diagnostics",
}

POLICY_REJECTION_SEEDS = {
    "raw-object": "from sifr.python import Object, PythonError\n",
    "raw-conversion": "from sifr.python import from_value, to_value\n",
    "python-core": "from sifr.python_core import Object\n",
    "spaced-module": "from sifr . python import Object\n",
    "continued-import": "from sifr.python import PythonError, \\\nObject\n",
    "direct-module": "import sifr.python as python_api\n",
    "from-module": "from sifr import python_core\n",
    "dynamic-trust": "@trust_python_dynamic(\"example\")\ndef example():\n    pass\n",
}
