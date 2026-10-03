"""Coordinated redigested mutations against intact independent originals."""
import copy
from dataclasses import replace
import json
from pathlib import Path
from include_source import verify,publish
from include_source_authority import read_originals
from include_source_relation import LOST
from include_source_tokens import Roots


def reject(test,proof,receipt,authority,label,change=None,receipt_change=None):
    value=copy.deepcopy(proof);record=copy.deepcopy(receipt)
    if change:change(value)
    if receipt_change:receipt_change(record)
    record['proof_digest']=test.api.digest(test.api.encoded(value))
    destination=test.run_dir/('unpublished-'+label+'.json')
    test.assertions+=1
    with test.assertRaises(test.api.Unsupported):publish(value,record,authority,destination,test.api)
    test.require(not destination.exists() and not destination.with_suffix('.json.tmp').exists(),'no partial publication after '+label)
    (test.run_dir/('negative-'+label+'.json')).write_bytes(test.api.encoded({'proof':value,'receipt':record,'intact_original_inventory':receipt['inputs']['original_inventory'],'accepted_complete_union':False}))


def matrix(test,proof,receipt,authority):
    source,_,_=read_originals(authority);roots=Roots(source['native']['syntax_inventory']['roots'],source['native']['physical'])
    joins=proof['compiler_semantic_owner_identity']['correspondences'];indices=[i for i,j in enumerate(joins) if any(a['relationship']=='direct' for a in j['source_native_attribute_membership']['source_attributes'])]
    test.require(len(indices)==2,'two genuine direct attributed include owners for mutation controls')
    ci=indices[0]
    def member(p):return p['compiler_semantic_owner_identity']['correspondences'][ci]['source_native_attribute_membership']
    def sem(p):return p['compiler_semantic_owner_identity']['correspondences'][ci]['compiler_semantic_owner_identity']
    def attr(p):return next(a for a in member(p)['source_attributes'] if a['relationship']=='direct')
    mutations=[
        ('attribute-remove',lambda p:member(p)['source_attributes'].clear()),
        ('attribute-duplicate',lambda p:member(p)['source_attributes'].append(copy.deepcopy(attr(p)))),
        ('attribute-kind',lambda p:attr(p).update(kind='forged-attribute')),
        ('attribute-style',lambda p:attr(p).update(style='inner')),
        ('attribute-order',lambda p:attr(p).update(ordinal=99)),
        ('attribute-direct-inner-nested',lambda p:attr(p).update(relationship='nested')),
        ('attribute-native-owner',lambda p:attr(p).update(native_owner_node=999999)),
        ('attribute-physical-owner',lambda p:attr(p).update(physical_owner_node=999999)),
        ('attribute-native-membership',lambda p:attr(p).update(native_attribute_node=999999)),
        ('attribute-physical-membership',lambda p:attr(p).update(physical_attribute_node=999999)),
        ('attribute-owner-swap',lambda p:member(p).update(native=copy.deepcopy(p['compiler_semantic_owner_identity']['correspondences'][indices[1]]['source_native_attribute_membership']['native']))),
        ('whole-physical-boundary',lambda p:member(p).update(physical_range=[17,68])),
        ('compiler-declaration-boundary',lambda p:sem(p)['declaration']['compiler_declaration'].update(start=0)),
        ('source-file',lambda p:member(p).update(file='/forged/included.rs')),
        ('native-context',lambda p:member(p)['native'].update(root=999999)),
        ('physical-root',lambda p:member(p)['physical'].update(root=999999)),
        ('token-omission',lambda p:member(p)['tokens'].pop()),
        ('token-duplicate',lambda p:member(p)['tokens'].append(copy.deepcopy(member(p)['tokens'][0]))),
        ('absent-public-origin',lambda p:member(p)['tokens'][0].update(original_intervals=[])),
        ('hull-only',lambda p:member(p).update(tokens=[member(p)['tokens'][0],member(p)['tokens'][-1]],subnodes=[])),
        ('subnode-omission',lambda p:member(p)['subnodes'].pop()),
        ('subnode-forged-ancestor',lambda p:member(p)['subnodes'][0].update(physical_node=999999)),
        ('intervening-physical-trivia',lambda p:member(p)['physical_trivia'].clear()),
        ('intervening-native-trivia',lambda p:member(p)['native_trivia'].clear()),
        ('normalization',lambda p:p['source_native_attribute_membership']['normalizations'][0]['normalization_authority'].update(official_normalization=[] ,original_source_len=0)),
        ('raw-bytes',lambda p:p['source_native_attribute_membership']['normalizations'][0].update(raw_sha256='forged')),
        ('include-ancestor',lambda p:p['source_native_attribute_membership']['include_roots'][0]['context']['context'].update(include_ancestors=[])),
        ('context-omission',lambda p:p['source_native_attribute_membership']['include_roots'].pop()),
        ('compiler-owner',lambda p:sem(p)['association']['identity'].update(hash='forged')),
        ('compiler-owner-swap',lambda p:sem(p)['association'].update(identity=copy.deepcopy(p['compiler_semantic_owner_identity']['correspondences'][indices[1]]['compiler_semantic_owner_identity']['association']['identity']))),
        ('compiler-parent',lambda p:sem(p).update(parent_relation={})),
        ('compiler-node-id',lambda p:sem(p)['association'].update(node_id=999999)),
        ('compiler-local-def',lambda p:sem(p)['association'].update(local_def_id='forged')),
        ('compiler-hir',lambda p:sem(p)['association'].update(hir_id='forged')),
        ('stage-empty-original-absence',lambda p:p['transformed_attribute_observations'].update(original_attributes_absent=True)),
        ('source-mislabelled-compiler-original',lambda p:p.update(compiler_original_attribute_attachment=p['source_native_attribute_membership'])),
        ('unknown-field',lambda p:p.update(fallback=True)),
        ('stale-invocation',lambda p:p['compiler_semantic_owner_identity'].update(capture_binding='stale')),
    ]
    for field in LOST:mutations.append(('false-'+field,lambda p,field=field:p.update({field:'compiler-original-authority'})))
    original=member(proof);physical=roots.root(original['physical']['root']);attribute=attr(proof)
    attribute_nodes=set(roots.descendants(physical,attribute['physical_attribute_node']))
    for kind,label in [('PATH','attribute-path'),('TOKEN_TREE','attribute-arguments')]:
        node=next(n for n in original['subnodes'] if n['physical_node'] in attribute_nodes and n['kind']==kind)
        index=original['subnodes'].index(node)
        mutations.append((label,lambda p,index=index:member(p)['subnodes'][index].update(physical_node=999999)))
    for kind,label in [('L_PAREN','attribute-open-delimiter'),('R_PAREN','attribute-close-delimiter')]:
        token=next(t for t in physical['tokens'] if t['parent'] in attribute_nodes and t['kind']==kind)
        index=next(i for i,e in enumerate(original['tokens']) if e['physical_token']==token['ordinal'])
        mutations.append((label,lambda p,index=index:member(p)['tokens'][index].update(native_tokens=[])))
    none_join=next(i for i,j in enumerate(joins) if any(n['aggregate_original'] is None for n in j['source_native_attribute_membership']['subnodes']))
    def forge_none(p):
        nodes=p['compiler_semantic_owner_identity']['correspondences'][none_join]['source_native_attribute_membership']['subnodes']
        next(n for n in nodes if n['aggregate_original'] is None)['aggregate_original']={'file':'forged','range':[0,1]}
    mutations.append(('forged-exact-aggregate-none',forge_none))
    for label,change in mutations:reject(test,proof,receipt,authority,label,change)
    reject(test,proof,receipt,authority,'failed-original-compiler',receipt_change=lambda r:r.update(capture_status=101))
    reject(test,proof,receipt,authority,'swapped-original-invocation-context',receipt_change=lambda r:r['context'].update(test=not r['context']['test']))
    reject(test,proof,receipt,replace(authority),'replaced-caller-held-authority')
    reject(test,proof,receipt,None,'absent-caller-held-authority')
    test.require(verify(proof,receipt,authority,test.api)['diagnostic_relation'],'intact source/native and final semantic identities remain valid after redigested mutation matrix')


def semantic_matrix(test,proof,receipt,authority):
    joins=proof['compiler_semantic_owner_identity']['correspondences']
    ci=next(i for i,j in enumerate(joins) if j['native_owner'] is not None and j['compiler_semantic_owner_identity']['binder_use_correspondences']['binders'])
    def sem(p):return p['compiler_semantic_owner_identity']['correspondences'][ci]['compiler_semantic_owner_identity']
    def use(p):return next(u for u in sem(p)['binder_use_correspondences']['uses'] if u['compiler']['resolved']['kind']=='LateBound')
    for label,change in (
        ('binder-identity',lambda p:sem(p)['binder_use_correspondences']['binders'][0]['compiler'].update(id='forged')),
        ('binder-map-omission',lambda p:sem(p)['binder_use_correspondences']['binders'].clear()),
        ('use-target',lambda p:use(p)['compiler']['target'].update(hash='forged')),
        ('use-depth',lambda p:use(p)['compiler']['resolved'].update(depth=99)),
        ('use-index',lambda p:use(p)['compiler']['resolved'].update(index=99)),
        ('use-omission',lambda p:sem(p)['binder_use_correspondences']['uses'].clear()),
        ('trait-owner',lambda p:sem(p)['trait_correspondences'][0]['compiler']['resolved'].update(hash='forged')),
        ('trait-omission',lambda p:sem(p)['trait_correspondences'].clear()),
        ('generic-index',lambda p:sem(p)['parameter_correspondences'][0]['native_parameter'].update(index=99)),
        ('outlives-omission',lambda p:sem(p)['outlives_correspondences'].clear()),
    ):reject(test,proof,receipt,authority,label,change)
    test.require(verify(proof,receipt,authority,test.api)['diagnostic_relation'],'intact original semantic controls pass after all semantic swaps/omissions')
