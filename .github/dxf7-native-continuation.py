"""One bounded DXF.7 continuation: retain passed stages; run failed/unreached stages."""
import hashlib,importlib.util,json,os,subprocess,sys,tarfile
from pathlib import Path
workspace=Path(os.environ["GITHUB_WORKSPACE"])
source=workspace/"source";validation=workspace/"validation"
sys.path.insert(0,str(source/"scripts/distribution"))
from qualify_native_package import NativePackage,digest,require
spec=importlib.util.spec_from_file_location("current_native_dxf",validation/"scripts/distribution/native_dxf_contracts.py")
helper=importlib.util.module_from_spec(spec);spec.loader.exec_module(helper)
artifacts=workspace/"native-inputs";previous=workspace/"previous-inputs"
commit=os.environ["SOURCE_COMMIT"];target=os.environ["TARGET"]
prior=workspace/"prior-evidence/native-qualification-evidence.tar.gz"
with tarfile.open(prior) as tar:
 raw=tar.extractfile("native-package.json").read();prior_report=json.loads(raw)
 manifest=json.load(tar.extractfile("evidence-index.json"))
 for row in manifest:
  require(hashlib.sha256(tar.extractfile(row["path"]).read()).hexdigest()==row["sha256"],"prior evidence digest mismatch")
 cache=json.load(tar.extractfile("dxf-cache/report.json"))
 trace=json.load(tar.extractfile("dxf-trace/trace/trace-v1.json"))
 identity=json.load(tar.extractfile("dxf-identity.stdout"))
require(prior_report["source_commit"]==commit and prior_report["target"]==target,"prior source/target mismatch")
require(prior_report["status"]=="fail" and prior_report["rows"][-1]["id"]=="dxf-0-run" and "unavailable" in prior_report["failure"],"continuation requires the specific admitted fixture failure")
require(len(cache["calls"])==34 and trace["compiler_identity"]==identity["compiler_build_id"],"prior package checks incomplete")
index=json.loads((artifacts/"qualification-artifact-index.json").read_text())
require(index["source_commit"]==commit and index["workflow"]["run_id"]==int(os.environ["ORIGINAL_RUN_ID"]) and index["workflow"]["run_attempt"]==1,"canonical index mismatch")
archive=artifacts/f"sifr-0.1.0-{target}.tar.gz";installer=artifacts/"sifr-installer-0.1.0"
for path in [archive,installer,artifacts/f"qualification-{target}.json"]:
 entries=[r for r in index["artifacts"] if r["name"]==path.name]
 require(len(entries)==1 and digest(path)==entries[0]["sha256"],"canonical input digest mismatch")
owner=NativePackage(artifacts,installer,"0.1.0",target,commit,Path(os.environ["RUNNER_TEMP"])/"sifr-native-continuation",previous,previous/"sifr-installer-0.0.0","0.0.0")
owner.report["continuation"]={"validation_source":os.environ["GITHUB_SHA"],"protocol_sha256":digest(__file__),"helper_sha256":digest(validation/"scripts/distribution/native_dxf_contracts.py"),"prior_archive_sha256":digest(prior),"prior_report_sha256":hashlib.sha256(raw).hexdigest(),"prior_status":"fail","retained_stages":"exact package transitions, transaction rollback, relocation, 34 cache calls and private trace/product identity checks","new_stages":"five source selectors under canonical owned TMPDIR; previously unreached native profiles/loader and source/installed metadata corpus","no_package_rebuild":True}
require(owner.report["previous_installer_sha256"]==prior_report["previous_installer_sha256"],"previous installer identity mismatch")
try:
 package=owner.output/"exact-package";package.mkdir()
 with tarfile.open(archive) as tar:tar.extractall(package,filter="data")
 binary=package/"bin/sifr";expected=json.loads((artifacts/f"qualification-{target}.json").read_text())
 require(digest(binary)==expected["binary_sha256"]==cache["binary_sha256"],"resumed bytes differ from original installed package")
 owner.report["archive_sha256"]=digest(archive);owner.report["binary_sha256"]=digest(binary)
 owner.run("rustc-identity",["rustc","-vV"])
 owner.run("cargo-identity",["cargo","-vV"])
 owner.integrity("resume-integrity",binary)
 owner.report["source_tests"]=helper.qualify_source(owner,source)
 owner.native_profiles(binary)
 owner.metadata_corpus(binary)
 subprocess.run(["git","diff","--quiet","HEAD"],cwd=source,check=True)
 owner.report["status"]="pass"
except BaseException as error:
 owner.report["status"]="fail";owner.report["failure"]=str(error);raise
finally:owner.save()
