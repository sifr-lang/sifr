from pathlib import Path
from tempfile import TemporaryDirectory

from fastapi import BackgroundTasks, Depends, FastAPI, Response
from pydantic import TypeAdapter
from starlette.applications import Starlette
from starlette.requests import Request
from starlette.responses import JSONResponse, PlainTextResponse
from starlette.routing import Route
from starlette.testclient import TestClient


def _record_frontend_task(completed: list[str]) -> None:
    completed.append("frontend")


def _verify_frontend() -> None:
    completed: list[str] = []

    def frontend_dependency(
        response: Response, background_tasks: BackgroundTasks
    ) -> None:
        response.headers["x-sifr-frontend"] = "served"
        background_tasks.add_task(_record_frontend_task, completed)

    with TemporaryDirectory() as directory:
        Path(directory, "index.html").write_text("Sifr frontend", encoding="utf-8")
        app = FastAPI(title="Sifr API", dependencies=[Depends(frontend_dependency)])
        app.frontend("/ui", directory=directory, check_dir=True)

        with TestClient(app) as client:
            frontend = client.get("/ui")

    if frontend.status_code != 200 or frontend.text != "Sifr frontend":
        raise RuntimeError("FastAPI frontend did not serve the static application")
    if frontend.headers.get("x-sifr-frontend") != "served" or not completed:
        raise RuntimeError("FastAPI frontend dependency effects were not preserved")


def _verify_body_limit() -> None:
    async def body_length(request: Request) -> PlainTextResponse:
        return PlainTextResponse(str(len(await request.body())))

    app = Starlette(
        routes=[Route("/upload", body_length, methods=["POST"])], max_body_size=4
    )
    with TestClient(app) as client:
        accepted = client.post("/upload", content=b"1234")
        rejected = client.post("/upload", content=b"12345")

    if accepted.status_code != 200 or accepted.text != "4":
        raise RuntimeError("Starlette rejected a body at the configured limit")
    if rejected.status_code != 413 or rejected.text != "Content Too Large":
        raise RuntimeError("Starlette did not enforce the configured body limit")


def run() -> str:
    app = FastAPI(title="Sifr API")
    value = TypeAdapter(int).validate_python("42")
    response = JSONResponse({"ok": True}, status_code=201)
    if value != 42 or app.title != "Sifr API" or response.status_code != 201:
        raise RuntimeError(
            "FastAPI/Pydantic full example did not validate expected results"
        )
    if response.body != b'{"ok":true}':
        raise RuntimeError("FastAPI response body did not match")
    _verify_frontend()
    _verify_body_limit()
    return (
        "sifr-python-interop:fastapi-pydantic:value=42:title=Sifr API:status=201:"
        "frontend=served:dependency=applied:max-body=413"
    )
