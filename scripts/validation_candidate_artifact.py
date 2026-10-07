"""Read one immutable GitHub candidate artifact, checking producer and bytes."""
import hashlib
import io
import json
import os
import urllib.request
import zipfile

MAX_ARCHIVE_BYTES = 1024 * 1024


class PublicRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, message, headers, newurl):
        redirected = super().redirect_request(request, fp, code, message, headers, newurl)
        if redirected is not None:
            redirected.remove_header("Authorization")
        return redirected


def archive_bytes(url: str) -> bytes:
    base = os.environ.get("GITHUB_API_URL", "https://api.github.com")
    if not url.startswith(base + "/repos/"):
        raise ValueError("artifact URL is outside the authoritative GitHub API")
    request = urllib.request.Request(url, headers={"Authorization": "Bearer " + os.environ["GH_TOKEN"],
                                                  "Accept": "application/vnd.github+json"})
    with urllib.request.build_opener(PublicRedirect()).open(request, timeout=30) as response:
        raw = response.read(MAX_ARCHIVE_BYTES + 1)
    if len(raw) > MAX_ARCHIVE_BYTES:
        raise ValueError("candidate artifact exceeds its registered bound")
    return raw


def candidate_identity(api, prefix: str, run: dict) -> dict:
    name = f"validation-candidate-{run['run_attempt']}"
    inventory = api(f"{prefix}/actions/runs/{run['id']}/artifacts?name={name}&per_page=100")
    artifacts = inventory["artifacts"]
    if inventory["total_count"] != 1 or len(artifacts) != 1:
        raise ValueError("candidate artifact missing, duplicated or incomplete")
    artifact = artifacts[0]
    producer = artifact.get("workflow_run", {})
    if (artifact["name"] != name or artifact.get("expired") is not False
            or producer.get("id") != run["id"] or producer.get("head_sha") != run["head_sha"]):
        raise ValueError("candidate artifact producer differs from the admitted run")
    raw = archive_bytes(artifact["archive_download_url"])
    if artifact.get("digest") != "sha256:" + hashlib.sha256(raw).hexdigest():
        raise ValueError("candidate artifact bytes differ from GitHub digest")
    with zipfile.ZipFile(io.BytesIO(raw)) as archive:
        if archive.namelist() != ["candidate.json"] or archive.getinfo("candidate.json").file_size > 65536:
            raise ValueError("candidate artifact has an unexpected inventory")
        identity = json.loads(archive.read("candidate.json"))
    if (set(identity) != {"candidate_sha", "run_id", "run_attempt"}
            or identity["run_id"] != run["id"] or identity["run_attempt"] != run["run_attempt"]):
        raise ValueError("candidate identity belongs to a different run/attempt")
    return identity
