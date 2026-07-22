from fastapi import FastAPI
from pydantic import TypeAdapter
from starlette.responses import JSONResponse


def run() -> str:
    app = FastAPI(title="Sifr API")
    value = TypeAdapter(int).validate_python("42")
    response = JSONResponse({"ok": True}, status_code=201)
    if value != 42 or app.title != "Sifr API" or response.status_code != 201:
        raise RuntimeError("FastAPI/Pydantic full example did not validate expected results")
    if response.body != b'{"ok":true}':
        raise RuntimeError("FastAPI response body did not match")
    return "sifr-python-interop:fastapi-pydantic:value=42:title=Sifr API:status=201"
