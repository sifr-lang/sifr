"""Genuine producer assertions, separate source membership and compiler identity."""
import json
from pathlib import Path
from include_source_authority import read_originals
from include_source_tokens import Roots
from include_source_relation import LOST,NOT_CLAIMED
from include_source import verify


def inventories(test,proof,receipt,authority,api):
    source,semantic,stage=read_originals(authority)
    compiler=semantic['compiler'];native=source['native'];identity=proof['compiler_semantic_owner_identity'];membership=proof['source_native_attribute_membership']
    result=verify(proof,receipt,authority,api)
    test.require(result['diagnostic_relation'] and result['semantic_export'] is False,'authenticated diagnostic relation without semantic export')
    test.require(all(proof[field]==NOT_CLAIMED for field in LOST),'abandoned compiler-original attachment/AttrId/lineage/survival never claimed')
    test.require(len(identity['complete_owner_dispositions'])==len(compiler['declaration_owners']),'complete independent original compiler inventory/dispositions')
    test.require(len(membership['native_dispositions'])==len(native['owners']),'complete independent native owner inventory/dispositions')
    test.require(identity['required_owner_indices']==[j['compiler_owner'] for j in identity['correspondences']],'every required original owner has exactly one correspondence')
    test.require(len(membership['include_roots'])==len(native['include_contexts']),'every independent actual include context reconciles, including an authenticated empty inventory')
    test.require(len(stage['lowered_attribute_owners'])>0 and stage['lowered_owner_enumeration']=='actual-hir-crate-items-owners-including-crate-root','all actual HIR owners retain transformed observations')
    test.require(stage['accepted_proof'] is False and stage['expanded_attributes'] is not None,'expanded AttrId and empty lowerings remain transformed-stage observations')
    for join in identity['correspondences']:
        sem=join['compiler_semantic_owner_identity'];owner=compiler['declaration_owners'][join['compiler_owner']];member=join['source_native_attribute_membership']
        test.require(sem['association']['identity']==owner['identity'] and sem['association']['local_def_id']==owner['local_def_id'] and sem['association']['hir_id']==owner['hir_id'],'independent final compiler semantic identity / actual prelower NodeId association')
        test.require(member['tokens'] and member['subnodes'],'complete nontrivia token and every native subnode relation')
        test.require(sem['declaration']['compiler_declaration']==owner['source'],'original compiler declaration remains separate from physical whole node')
    return source,semantic,stage


def named_join(proof,semantic,suffix,api):
    owners=semantic['compiler']['declaration_owners']
    selected=[j for j in proof['compiler_semantic_owner_identity']['correspondences'] if owners[j['compiler_owner']]['identity']['path'].endswith(suffix)]
    if len(selected)!=1:raise api.Unsupported('missing/ambiguous genuine acceptance anchor: '+suffix)
    return selected[0],owners[selected[0]['compiler_owner']]


def attribute_anchor(test,proof,authority,suffix,original_range,physical_range,attribute_range,api):
    source,semantic,_=read_originals(authority);join,owner=named_join(proof,semantic,suffix,api)
    membership=join['source_native_attribute_membership'];roots=Roots(source['native']['syntax_inventory']['roots'],source['native']['physical'])
    direct=[a for a in membership['source_attributes'] if a['relationship']=='direct']
    test.require([owner['source']['start'],owner['source']['end']]==original_range,'fresh original compiler declaration anchor')
    test.require(membership['physical_range']==physical_range,'whole physical attributed node remains separate')
    test.require(len(direct)==1 and direct[0]['style']=='outer','complete direct source/native outer attribute membership')
    physical=roots.root(membership['physical']['root']);attribute=physical['nodes'][direct[0]['physical_attribute_node']]
    test.require(attribute['range']==attribute_range and attribute['kind']=='ATTR','exact source/native direct attribute boundary and kind')
    test.require(any(n['kind']=='TOKEN_TREE' for n in membership['subnodes']) and any(n['kind']=='PATH' for n in membership['subnodes']),'attribute path and arguments have actual complete subnode origins')
    test.require(join['compiler_semantic_owner_identity']['association']['node_id'] is not None,'actual compiler declaration semantic identity is independent of direct source attribute membership')


def generics(test,proof,receipt,authority,api):
    source,semantic,_=inventories(test,proof,receipt,authority,api)
    roots=proof['source_native_attribute_membership']['include_roots'];joins=proof['compiler_semantic_owner_identity']['correspondences']
    test.require(len(roots)==2 and roots[0]['membership']['file']==roots[1]['membership']['file'],'genuine reused physical source has two independently enumerated includes')
    test.require(roots[0]['membership']['native']['root']!=roots[1]['membership']['native']['root'],'left/right native include identities stay separate despite cached descent')
    test.require(all(r['membership']['tokens'] and r['membership']['subnodes'] and r['membership']['source_attributes'] for r in roots),'complete ordered source attribute/token/subnode membership in each include context')
    semantics=[j['compiler_semantic_owner_identity'] for j in joins if j['native_owner'] is not None]
    test.require(any(s['generics'] and s['generics']['parent_count']>0 and s['parameter_correspondences'] for s in semantics),'actual inherited and method-own parameters both retain compiler indices')
    test.require(any(s['binder_use_correspondences']['binders'] for s in semantics),'genuine nonempty supported compiler HRTB binders')
    test.require(any(s['binder_use_correspondences']['uses'] for s in semantics),'actual HIR lifetime uses reconcile full native occurrence inventory')
    test.require(any(s['outlives_correspondences'] for s in semantics),'genuine original outlives clauses retain typed compiler region authority')
    test.require(any(e['semantic_identity']['disposition']=='exact-local-compiler-native-trait-owner' for s in semantics for e in s['trait_correspondences']),'resolved local trait has its own exact compiler/native source and semantic parent identity')
    test.require(any(e['compiler'].get('resolved',{}).get('kind')=='LateBound' for s in semantics for e in s['binder_use_correspondences']['uses']),'nonempty HRTB uses retain actual compiler target/depth/index')
    return {'owners':len(joins),'roots':len(roots),'attributes':sum(len(r['membership']['source_attributes']) for r in roots),'outlives':sum(len(s['outlives_correspondences']) for s in semantics),'binders':sum(len(s['binder_use_correspondences']['binders']) for s in semantics)}
