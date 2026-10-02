"""Actual producers qualify only the bounded original dependency feasibility."""
import copy
import json
import os
from pathlib import Path
import tempfile
import unittest
import maintainability_builtin_input as builtin
import source_binder as bridge


class SourceBinderFeasibilityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.target=Path(os.environ["SIFR_BUILTIN_TARGET_DIR"]).resolve()
        cls.component=Path(os.environ["SIFR_BUILTIN_COMPONENT_RECEIPT"])
        cls.identity=builtin.tool_identity(cls.component,cls.target)
        cls.evidence=Path(os.environ["SIFR_BUILTIN_EVIDENCE_DIR"])
        key=builtin.digest(builtin.encoded([str(builtin.ROOT),builtin.run(["git","rev-parse","HEAD"]).stdout.strip(),cls.identity,{k:builtin.digest(v.encode()) for k,v in os.environ.items() if not k.startswith("SIFR_BUILTIN_")}]))
        cls.capture_dir=cls.evidence / "source-binder-prepared" / key
        cls.proof,cls.receipt,cls.authority=bridge.capture(cls.capture_dir,cls.target,cls.identity,builtin)

    def setUp(self):self.assertions=0
    def require(self,value,reason):
        self.assertions+=1
        self.assertTrue(value,reason)
    def tearDown(self):print(f"{self.id()}: assertions={self.assertions}; evidence={self.capture_dir}")
    def reject(self,value,label,receipt=None,authority=None):
        receipt=copy.deepcopy(receipt or self.receipt)
        receipt["proof_digest"]=builtin.digest(builtin.encoded(value))
        self.assertions+=1
        with self.assertRaises(builtin.Unsupported):bridge.verify(value,receipt,authority if authority is not None else self.authority,builtin)
        (self.capture_dir / ("negative-"+label+".json")).write_bytes(builtin.encoded(value))

    def test_original_syn_step_dependency_hir_is_available(self):
        originals=json.loads(self.authority.originals)
        build=originals["original-build.json"]
        invocation=originals["dependency-invocation.json"]
        raw=originals["syn-raw.json"]
        self.require(bridge.verify(self.proof,self.receipt,self.authority,builtin)["feasibility"],"bounded proof has authenticated original authority")
        self.require(build["status"]==0 and any(m.get("reason")=="build-finished" and m["success"] for m in build["messages"]),"normal selected Cargo success")
        self.require(invocation["compiler_status"]==0 and "RUSTC_BOOTSTRAP" not in invocation["environment"],"actual dependency invocation succeeds without bootstrap")
        self.require(build["archive_sha256"]=="12df2e0110f65b775f769bb17ef989067a1d931b2eb822bd4346631eeada89f9","locked original registry archive")
        self.require(len(raw["declaration_owners"])==len(originals["independent-inventory.json"]["owners"])>1,"complete independently held compiler owner universe before selection")
        self.require(stable(self.proof["declaration_owners"][0]["identity"]) in {stable(c["target"]) for c in originals["caller-raw.json"]["calls"]},"cross-capture relation uses identical crate/DefPathHash")
        self.require(self.proof["semantic_export"] is False,"feasibility never publishes a semantic adapter export")

    def test_syn_step_hir_binder_uses_join_exact_ra_source(self):
        links=self.proof["source_correspondences"]
        declaration=[l for l in links if l["role"]=="declaration"]
        uses=[l for l in links if l["role"]=="use"]
        inherited=[l for l in links if l["role"]=="inherited-use"]
        self.require(len(declaration)==1 and len(uses)==2 and len(inherited)==1,"all four actual named occurrences survive")
        self.require(all(l["target"]==declaration[0]["target"] for l in uses),"both actual compiler uses resolve to original binder declaration")
        self.require(all(l["resolved"]["kind"]=="LateBound" and l["resolved"]["depth"]==0 and l["resolved"]["index"]==0 for l in uses),"compiler late-bound depth/index preserved")
        self.require(inherited[0]["target"]!=declaration[0]["target"] and inherited[0]["resolved"]["kind"]=="EarlyBound","inherited lifetime identity remains distinct")
        self.require(all(l["ra"]["resolved"] is None for l in declaration+uses) and inherited[0]["ra"]["resolved"]["relation"]=="inherited","HRTB identity comes solely from original compiler HIR")
        self.require(sorted(l["ra"]["range"] for l in links)==[[35876,35878],[35898,35900],[35902,35904],[35928,35930]],"ranges independently reauthenticated by actual producers")
        self.require(len(self.proof["trait_constraints"])==1 and self.proof["trait_constraints"][0]["ra"]["roundtrip"],"each explicit resolved trait path joins")
        self.require(self.proof["declaration_owners"][0]["generics"]["parent_count"]==1,"inherited and method-own generic parameters preserved")
        self.require(all(b["compiler_map_disposition"]=="present" for b in self.proof["binders"]),"supported original compiler binder queries only")
        self.require(bridge.verify(self.proof,self.receipt,self.authority,builtin)["source_correspondences"]==4,"exact original relation admitted")
        self.real_fixtures()

    def test_unavailable_or_ambiguous_original_bridge_stops(self):
        for field in ("dependency_invocation","source_files","declaration_owners","binders","lifetime_occurrences","trait_constraints","source_correspondences","independent_inventory"):
            value=copy.deepcopy(self.proof);value[field]=[]
            self.reject(value,"missing-"+field)
        for field in ("declaration_owners","source_correspondences","trait_constraints"):
            value=copy.deepcopy(self.proof);value[field].append(copy.deepcopy(value[field][0]))
            self.reject(value,"ambiguous-"+field)
        value=copy.deepcopy(self.proof);value["source_correspondences"][1]["resolved"]["depth"]=9
        self.reject(value,"forged-depth")
        value=copy.deepcopy(self.proof);value["declaration_owners"][0]["identity"]=copy.deepcopy(value["declaration_owners"][1]["identity"])
        self.reject(value,"wrong-owner")
        value=copy.deepcopy(self.proof);value["trait_constraints"][0]["compiler"]["resolved"]["hash"]="forged-trait"
        self.reject(value,"wrong-trait")
        receipt=copy.deepcopy(self.receipt);receipt["capture_status"]=101
        self.reject(self.proof,"compiler-failure",receipt=receipt)
        self.reject(self.proof,"replaced-authority",authority=bridge.OriginalAuthority(self.authority.originals,self.authority.inputs,self.authority.seal))
        self.assertions+=1
        with self.assertRaises(builtin.Unsupported):bridge.verify(self.proof,self.receipt,None,builtin)
        self.require(bridge.verify(self.proof,self.receipt,self.authority,builtin)["semantic_export"] is False,"intact independent originals remain valid after all mutations")
        for name in ("dependency-invocation.json","syn-raw.json","ra-source.json"):
            path=self.capture_dir/name
            retained=self.capture_dir/("retained-original-"+name)
            original=path.read_bytes()
            path.rename(retained)
            try:self.reject(self.proof,"actual-unavailable-"+name)
            finally:retained.rename(path)
            self.require(path.read_bytes()==original,"actual original authority bytes retained after unavailable-input rejection")
        self.real_fixtures()

    def real_fixtures(self):
        directory=Path(tempfile.mkdtemp(prefix="real-source-binder-",dir=self.capture_dir))
        source=builtin.TOOL / "fixtures/source_binder/independent.rs"
        env=os.environ.copy()
        for key in ("RUSTC_BOOTSTRAP","RUSTC_WRAPPER","RUSTC_WORKSPACE_WRAPPER"):env.pop(key,None)
        args=["--crate-name","source_binder_fixture","--crate-type","lib","--edition","2024","--emit","metadata","--out-dir",str(directory),str(source)]
        control=builtin.run(["rustc",*args],env=env,log=directory / "compiler-control.log")
        self.require(control.returncode==0,"real independent-binder original compiler control")
        replay=env.copy();replay["SIFR_BUILTIN_SOURCE_BINDER"]=str(directory / "raw.json")
        builtin.run([str(self.target / "debug/sifr_maintainability_builtin_input"),*args],env=replay,log=directory / "compiler-capture.log")
        raw=json.loads((directory / "raw.json").read_text())
        result=builtin.run([str(self.target / "debug/ra_common"),"--source-binder-syntax",str(source)],env=env,log=directory / "ra-syntax.log")
        syntax=json.loads(result.stdout);(directory / "ra-syntax.json").write_bytes(builtin.encoded(syntax))
        owners=[o for o in raw["declaration_owners"] if o["identity"]["kind"]=="Fn"]
        self.require(len(owners)==len(syntax["functions"])==2,"all original fixture owners preserved")
        declarations=[]
        for owner in owners:
            parameter=[p for p in owner["parameters"] if p["origin"]=="Binder"][0]
            lifetime=[l for l in owner["lifetime_occurrences"] if l["syntax"]=="ExplicitBound"][0]
            self.require(stable(parameter["identity"])==stable(lifetime["target"])==stable(lifetime["resolved"]["target"]),"real fixture use/compiler declaration relation")
            declarations.append(stable(parameter["identity"]))
            function=bridge.unique([f for f in syntax["functions"] if f["range"]==[owner["source"]["start"],owner["source"]["end"]]],"fixture owner AST interval",builtin)
            for span in (parameter["source"],lifetime["source"]):
                occurrence=bridge.unique([l for l in function["lifetimes"] if l["range"]==[span["start"],span["end"]]],"fixture lifetime AST interval",builtin)
                self.require(source.read_bytes()[span["start"]:span["end"]].decode()==occurrence["token"]==span["snippet"],"actual Unicode-prefix original byte position and unique RA token interval")
        self.require(declarations[0]!=declarations[1],"independent same-spelled real binders retain distinct compiler identity")
        bad=builtin.TOOL / "fixtures/source_binder/invalid_shadow.rs"
        bad_args=[*args[:-1],str(bad)]
        failed=builtin.run(["rustc",*bad_args],env=env,log=directory / "invalid-shadow-control.log",allowed=(1,))
        self.require(failed.returncode==1 and "E0496" in failed.stderr,"invalid lexical shadowing is a genuine compiler-error negative")
        replay["SIFR_BUILTIN_SOURCE_BINDER"]=str(directory / "failed-raw.json")
        failed=builtin.run([str(self.target / "debug/sifr_maintainability_builtin_input"),*bad_args],env=replay,log=directory / "invalid-shadow-capture.log",allowed=(101,))
        self.require(failed.returncode==101 and "E0496" in failed.stderr and not (directory / "failed-raw.json").exists(),"compiler failure cannot publish original local HIR feasibility")


def stable(identity):return identity["crate"],identity["hash"]

