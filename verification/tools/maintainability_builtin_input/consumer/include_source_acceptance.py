"""Four exact amended include-source proof cases using genuine producers."""
import gc
import json
import os
from pathlib import Path
import time
import unittest
import uuid
import maintainability_builtin_input as b
import include_source_capture as capture
import include_source_assertions as assertions
import include_source_negatives as negatives
from include_source_authority import read_originals
from include_source import verify
import include_source_union as union
import include_source_encoding as canonical

CONTEXTS=tuple(capture.context(p,t) for p in ('sifr_codegen','sifr_lowering') for t in (False,True))
VARIANTS=('hrtb','alpha','crlf','bom','unicode')
ORIGINAL_CASES={
 'BuiltinCapabilityTests':('test_builtin_bodies_calls_and_auxiliary_origins','test_hygiene_and_same_spelled_methods_preserve_trait_origin','test_expansion_ast_hir_mapping_is_owned_and_complete','test_component_context_and_input_drift_fail_closed','test_live_rust_ir_builtin_surface_has_complete_dispositions'),
 'BuiltinInventoryTests':('test_hir_ty_owner_inventory_is_independent_of_ast_projection','test_coordinated_fixture_owner_removals_fail_semantic_admission','test_ra_invocation_owned_common_members_fail_without_compiler_impl','test_coordinated_live_owner_removals_fail_semantic_admission'),
 'BuiltinExtensionTests':('test_default_eq_hash_ordering_bodies_and_members','test_lifetime_receiver_and_method_generics_preserve_constraints','test_extended_inventory_owner_and_constraint_removals_fail_closed','test_live_owned_derive_inventory_has_complete_dispositions'),
}


def cache_key(root,ctx,identity):
    return b.digest(b.encoded([str(root),b.run(['git','rev-parse','HEAD'],cwd=root).stdout.strip(),ctx,identity,{k:b.digest(v.encode()) for k,v in os.environ.items() if not k.startswith('SIFR_BUILTIN_')}]))


def prepared(root,ctx,target,identity,evidence):
    output=evidence/'include-source-prepared'/cache_key(root,ctx,identity)
    return capture.capture(output,target,identity,b,ctx,root),output


def signature(proof,authority):
    """Compare complete source edges; IDs remain intact in both verified proofs.

    Only already proved local owner/parameter identities are mapped to their
    independent compiler inventory ordinal. No semantic identity is derived
    from spelling, source ranges or this repeat comparison.
    """
    source,semantic,_=read_originals(authority);owners=semantic['compiler']['declaration_owners']
    identities={b.encoded(o['identity']):['owner',i] for i,o in enumerate(owners)}
    for i,o in enumerate(owners):
        for pi,p in enumerate(o['parameters']):identities[b.encoded(p['identity'])]=['parameter',i,pi]
    root=Path(semantic['original_control']['cwd'])
    def normalize(v):
        if isinstance(v,dict):
            if {'crate','hash','kind','path','raw'}<=set(v):return identities.get(b.encoded(v),v)
            return {k:normalize(x) for k,x in v.items() if k not in {'source_file_start_pos','source_hash','original_source_hash'}}
        if isinstance(v,list):return [normalize(x) for x in v]
        if isinstance(v,str) and v.startswith(str(root)+'/'):return '<ROOT>/'+v[len(str(root))+1:]
        return v
    result=[]
    for j in proof['compiler_semantic_owner_identity']['correspondences']:
        membership=j['source_native_attribute_membership'];sem=j['compiler_semantic_owner_identity']
        # Actual root ordinals/RA IDs can differ across independent DBs. Every
        # token/subnode's original range/kind and complete source partitions are
        # compared, while each proof separately establishes all native owners.
        result.append({'compiler_owner':j['compiler_owner'],'source':normalize(owners[j['compiler_owner']]['source']),'kind':owners[j['compiler_owner']]['source_owner_kind'],'tokens':membership['tokens'],'subnodes':[{'kind':n['kind'],'aggregate_original':normalize(n['aggregate_original']),'contextual_original':normalize(n['contextual_original'])} for n in membership['subnodes']],'attributes':membership['source_attributes'],'physical_trivia':normalize(membership['physical_trivia']),'physical_range':membership['physical_range'],'association':normalize(sem['association']),'compiler_parameters':normalize(sem.get('parameters',[])),'compiler_binders':normalize(sem.get('binders',[])),'compiler_uses':normalize(sem.get('uses',[])),'compiler_traits':normalize(sem.get('traits',[]))})
    return result


def signature_record(proof,receipt,authority,path):
    # Keep every normalized source edge, but release the object graph before
    # independently loading the repeat's complete originals and projection.
    verify(proof,receipt,authority,b)
    canonical.write(path,signature(proof,authority))
    return (str(path),canonical.file_digest(path),Path(path).stat().st_size)


def signatures_equal(left,right):
    for path,sha,size in (left,right):
        if Path(path).stat().st_size!=size or canonical.file_digest(path)!=sha:
            raise b.Unsupported('held complete normalized signature byte binding drift')
    with Path(left[0]).open('rb') as first,Path(right[0]).open('rb') as second:
        while True:
            a=first.read(1024*1024);c=second.read(1024*1024)
            if a!=c:return False
            if not a:return True


class IncludeSourceCorrespondenceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.api=b;cls.target=Path(os.environ['SIFR_BUILTIN_TARGET_DIR']).resolve();cls.evidence=Path(os.environ['SIFR_BUILTIN_EVIDENCE_DIR']).resolve()
        cls.identity=b.tool_identity(os.environ['SIFR_BUILTIN_COMPONENT_RECEIPT'],cls.target)
        cls.fixture=b.TOOL/'fixtures/source_binder'
    def setUp(self):
        self.assertions=0;self.started=time.monotonic();self.run_dir=self.evidence/'include-source-assertions'/(self._testMethodName+'-'+str(uuid.uuid4()));self.run_dir.mkdir(parents=True)
    def tearDown(self):
        record={'case':self.id(),'assertions':self.assertions,'candidate':b.run(['git','rev-parse','HEAD']).stdout.strip(),'elapsed_seconds':time.monotonic()-self.started,'semantic_export':False}
        (self.run_dir/'assertion-count.json').write_bytes(b.encoded(record));print(record,'evidence',self.run_dir,flush=True);gc.collect()
    def require(self,value,reason):self.assertions+=1;self.assertTrue(value,reason)
    def get(self,root,ctx):
        result,output=prepared(root,ctx,self.target,self.identity,self.evidence)
        self.last_output=output
        (self.run_dir/('capture-'+b.digest(str(output).encode())+'.json')).write_bytes(b.encoded({'path':str(output),'receipt_sha256':b.digest((output/'receipt.json').read_bytes()),'originals':[original.digest() for original in result[2].originals]}))
        return result
    def fixture_capture(self,name):
        root=self.fixture/('include_probe_'+name);package=json.loads(b.run(['cargo','metadata','--locked','--no-deps','--format-version','1'],cwd=root).stdout)['packages'][0]['name']
        return self.get(root,capture.context(package))

    def test_included_impl_and_adt_have_exact_original_semantic_source(self):
        proof,receipt,authority=self.fixture_capture('attributes');source,semantic,_=assertions.inventories(self,proof,receipt,authority,b)
        for side in ('left','right'):assertions.attribute_anchor(self,proof,authority,side+'::IncludedBinder',[17,68],[0,68],[0,16],b)
        joins=proof['compiler_semantic_owner_identity']['correspondences'];owners=semantic['compiler']['declaration_owners']
        impls=[j for j in joins if owners[j['compiler_owner']]['source_owner_kind']=='Impl']
        self.require(len(impls)==2 and all(any(n['aggregate_original'] is None for n in j['source_native_attribute_membership']['subnodes']) for j in impls),'genuine IncludedBinder impl aggregate original_range_opt None stays None')
        self.require(all(j['compiler_semantic_owner_identity']['binder_use_correspondences']['declarations'] for j in impls),'IncludedBinder actual own lifetime declaration has exact native source')
        self.require(any(j['compiler_semantic_owner_identity'].get('binder_use_correspondences',{}).get('uses') for j in joins),'IncludedBinder actual inherited lifetime uses retain semantic identities')
        del proof,receipt,authority,source,semantic,joins,owners,impls;gc.collect()
        proof,receipt,authority=self.get(b.ROOT,capture.context('sifr_codegen'));_,semantic,_=assertions.inventories(self,proof,receipt,authority,b)
        assertions.attribute_anchor(self,proof,authority,'::SifrIntBindingCollector',[19,81],[0,81],[0,18],b)
        scalar,owner=assertions.named_join(proof,semantic,'::ScalarCallRewriter',b)
        self.require([owner['source']['start'],owner['source']['end']]==[0,307] and [owner['name_source']['start'],owner['name_source']['end']]==[7,25],'fresh ScalarCallRewriter compiler declaration and name anchors')
        stage=json.loads(authority.stage_observations);observed=stage['owners'][scalar['compiler_semantic_owner_identity']['association']['stage_owner']]
        self.require(observed['declaration_tokens_available'] is False,'actual ScalarCallRewriter transformed expanded tokens unavailable disposition')
        self.require(scalar['source_native_attribute_membership']['tokens'] and scalar['source_native_attribute_membership']['subnodes'] and scalar['compiler_semantic_owner_identity']['parent_relation'],'ScalarCallRewriter still has complete exact native token/subnode/owner/parent relation')

    def test_multi_anchor_normalization_and_context_identity_are_lossless(self):
        counts=[]
        for name in VARIANTS:
            proof,receipt,authority=self.fixture_capture(name);counts.append(assertions.generics(self,proof,receipt,authority,b))
            maps=proof['source_native_attribute_membership']['normalizations'];physical=self.fixture/('include_probe_'+name)/'src/included.rs';raw=physical.read_bytes()
            relevant=[m for m in maps if m['file']==str(physical)]
            self.require(len(relevant)==1 and relevant[0]['raw_sha256']==b.digest(raw),'independently authenticated actual '+name+' raw include bytes')
            if name=='crlf':self.require(b'\r\n' in raw and relevant[0]['omitted_bytes'],'actual official CRLF normalization has exhaustive byte witnesses')
            if name=='bom':
                self.require(raw.startswith(b'\xef\xbb\xbf') and relevant[0]['omitted_bytes'][0]['range']==[0,3],'actual official UTF-8 BOM normalization witness')
                dispositions=[r['membership']['normalization_dispositions'] for r in proof['source_native_attribute_membership']['include_roots']]
                self.require(len(dispositions)==2 and all(len(d)==1 and d[0]['physical_token']['range']==[0,3] and d[0]['physical_node']['kind']=='ERROR' and d[0]['official_boundary']=={'position':0,'difference':3,'original_position':3} for d in dispositions),'every actual physical BOM token/subnode has an exact authenticated normalization disposition in each include context')
            if name=='unicode':self.require('Café'.encode() in raw and 'λ'.encode() in raw,'genuine multibyte Unicode declaration and trivia bytes retained')
            if name=='alpha':self.require(b'Container' in raw and b"'parent" in raw and b"'bound" in raw,'genuine alpha-renamed declarations and binder tokens')
            del proof,receipt,authority;gc.collect()
        self.require(all(c==counts[0] for c in counts),'all genuine normalized/renamed cases retain complete owner/attribute/constraint counts')

    def test_coordinated_include_source_mutations_fail_closed(self):
        proof,receipt,authority=self.fixture_capture('attributes');assertions.inventories(self,proof,receipt,authority,b);negatives.matrix(self,proof,receipt,authority);negatives.cache_matrix(self,self.last_output,self.identity)
        for original,binding in zip((authority.source_membership,authority.compiler_semantics,authority.stage_observations),receipt['inputs']['original_inventory']):
            self.require(original==Path(binding['path']).read_bytes(),'caller-held lossless storage preserves the complete actual original bytes')
        source,_,_=read_originals(authority)
        ancestors=source['native']['include_contexts'][0]['context']['include_ancestors']
        saved=list(ancestors)
        try:
            ancestors.clear()
            negatives.reject(self,proof,receipt,authority,'cached-complete-original-decode-mutation')
        finally:ancestors[:]=saved
        self.require(verify(proof,receipt,authority,b)['diagnostic_relation'],'complete decoded original restoration retains the independently held immutable positive')
        del proof,receipt,authority,source,ancestors,saved;gc.collect()
        proof,receipt,authority=self.fixture_capture('hrtb');assertions.generics(self,proof,receipt,authority,b);negatives.semantic_matrix(self,proof,receipt,authority)
        path=self.fixture/'include_probe_hrtb/src/included.rs';original=path.read_bytes()
        try:
            path.write_bytes(original+b'\n// bounded real raw source input mutation\n')
            negatives.reject(self,proof,receipt,authority,'actual-included-source-input-drift')
        finally:path.write_bytes(original)
        self.require(verify(proof,receipt,authority,b)['diagnostic_relation'],'original byte restoration preserves authenticated positive')
        record=json.loads(authority.inputs);configuration=Path(record['root'])/'Cargo.toml';original=configuration.read_bytes()
        try:
            configuration.write_bytes(original+b'\n# bounded actual configuration mutation\n')
            negatives.reject(self,proof,receipt,authority,'actual-cfg-manifest-input-drift')
        finally:configuration.write_bytes(original)
        self.require(verify(proof,receipt,authority,b)['diagnostic_relation'],'intact original source/configuration returns to valid diagnostic control')
        del proof,receipt,authority;gc.collect()
        proof,receipt,authority=self.fixture_capture('bom');assertions.inventories(self,proof,receipt,authority,b)
        def bom(p):return p['source_native_attribute_membership']['include_roots'][0]['membership']['normalization_dispositions']
        negatives.reject(self,proof,receipt,authority,'official-bom-disposition-remove',lambda p:bom(p).clear())
        negatives.reject(self,proof,receipt,authority,'official-bom-physical-token-forge',lambda p:bom(p)[0]['physical_token'].update(range=[0,2]))
        negatives.reject(self,proof,receipt,authority,'official-bom-boundary-forge',lambda p:bom(p)[0]['official_boundary'].update(difference=2))
        negatives.reject(self,proof,receipt,authority,'official-bom-context-swap',lambda p:bom(p)[0].update(native_context=p['source_native_attribute_membership']['include_roots'][1]['membership']['native']))

    def test_required_original_linux_include_context_union_repeats(self):
        repeat=Path(os.environ['SIFR_BUILTIN_REPEAT_ROOT']).resolve()
        self.require(repeat!=b.ROOT and b.run(['git','rev-parse','HEAD'],cwd=repeat).stdout==b.run(['git','rev-parse','HEAD']).stdout,'fresh independently owned unchanged candidate checkout')
        summaries=[];qualified=[]
        for ctx in CONTEXTS:
            proof,receipt,authority=self.get(b.ROOT,ctx);assertions.inventories(self,proof,receipt,authority,b)
            qualified.append(union.qualify(proof,receipt,authority,self.last_output,b))
            label=ctx['package']+('-tests' if ctx['test'] else '-lib')
            before=signature_record(proof,receipt,authority,self.run_dir/(label+'-main-normalized.json'));syn=proof['compiler_semantic_owner_identity']['original_syn_correspondence']
            self.require((syn is not None)==(ctx['package']=='sifr_codegen'),'actual original syn::step joins exactly where caller dependency applies')
            if syn:
                self.require(len(syn['source_correspondences'])==4 and syn['semantic_export'] is False,'complete actual syn::step declaration/HRTB/inherited source correspondences')
                exact=proof['compiler_semantic_owner_identity']['exact_original_dependency_source']
                self.require(len(exact['declaration_owners'])==2 and all(j['membership']['tokens'] and j['membership']['subnodes'] and j['association'] for j in exact['declaration_owners']),'retained syn::step also has complete native original token/subnode and actual compiler NodeId/LocalDefId/HIR owner authority')
            if syn:del exact
            del syn
            summaries.append({'context':ctx,'owners':len(proof['compiler_semantic_owner_identity']['complete_owner_dispositions']),'required':len(proof['compiler_semantic_owner_identity']['correspondences']),'roots':len(proof['source_native_attribute_membership']['include_roots'])})
            del proof,receipt,authority;gc.collect()
            proof,receipt,authority=self.get(repeat,ctx);assertions.inventories(self,proof,receipt,authority,b)
            qualified.append(union.qualify(proof,receipt,authority,self.last_output,b))
            after=signature_record(proof,receipt,authority,self.run_dir/(label+'-repeat-normalized.json'))
            self.require(signatures_equal(before,after),'fresh unchanged complete source/native and independent compiler owner relation normalizes across owned checkouts')
            path=Path(after[0])
            with path.open('r+b') as stream:
                original=stream.read(1);stream.seek(0);stream.write(b' ')
            try:
                self.assertions+=1
                with self.assertRaisesRegex(b.Unsupported,'signature byte binding drift'):signatures_equal(before,after)
            finally:
                with path.open('r+b') as stream:stream.write(original)
            self.require(signatures_equal(before,after),'restored complete normalized signature bytes retain the genuine independently authenticated repeat comparison')
            (self.run_dir/(label+'-normalized-bindings.json')).write_bytes(b.encoded({'main':before,'repeat':after,'compared_every_byte':True}))
            del proof,receipt,authority,before;gc.collect()
        for name in ('attributes',*VARIANTS):
            proof,receipt,authority=self.fixture_capture(name);assertions.inventories(self,proof,receipt,authority,b);qualified.append(union.qualify(proof,receipt,authority,self.last_output,b));del proof,receipt,authority;gc.collect()
        names=['maintainability_builtin_input_tests.'+cls+'.'+case for cls,cases in ORIGINAL_CASES.items() for case in cases]
        fixture_qualification=union.run_fixtures(names,self.run_dir,b)
        self.require(len(fixture_qualification.cases)==13,'entire retained original thirteen-case fixture union actually runs')
        destination=self.run_dir/'complete-context-union.json'
        roots=[self.fixture/('include_probe_'+name) for name in ('attributes',*VARIANTS)]
        self.assertions+=1
        with self.assertRaises(b.Unsupported):union.publish(qualified[:-1],fixture_qualification,destination,b.ROOT,repeat,roots,b)
        self.require(not destination.exists(),'missing required original context prevents partial accepted publication')
        artifact=union.publish(qualified,fixture_qualification,destination,b.ROOT,repeat,roots,b)
        self.require(artifact['accepted_complete_union'] and artifact['semantic_export'] is False,'only complete independently authenticated diagnostic union is published')
