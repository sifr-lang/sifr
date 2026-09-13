"""Exercise maintained service/model bridges without compiler or live services."""
from importlib.util import module_from_spec, spec_from_file_location
from pathlib import Path

from dependency_versions import runtime_version_marker


def check_service_model_features() -> str:
    versions = runtime_version_marker("alembic", "boto3", "psycopg", "pydantic")
    fixtures = Path(__file__).resolve().parents[1] / "fixtures"
    cases = (
        ("sqlalchemy_psycopg", "sqlalchemy_psycopg_example"),
        ("aws_sqs", "boto3_botocore_example"),
        ("fastapi_app", "fastapi_pydantic_example"),
    )
    for fixture, name in cases:
        path = fixtures / fixture / "python_bridges" / f"{name}.py"
        spec = spec_from_file_location(name, path)
        if spec is None or spec.loader is None:
            raise RuntimeError(f"cannot load owned service/model bridge: {path}")
        module = module_from_spec(spec)
        spec.loader.exec_module(module)
        marker = module.run()
        if not marker.startswith("sifr-python-interop:"):
            raise RuntimeError(f"service/model bridge lost its success marker: {name}")
    return versions + " service-model-bridges=3 compiled-evidence=false"
