"""Actual local trait identities and compiler predicate region cross-references."""
from include_source_authority import require
from include_source_declarations import stable,association,exact_declaration
from include_source_parents import parent_relation
from include_source_semantics import span_partition


def local_trait_identity(identity,target,owners,stage,native_owners,roots,records,inputs,api):
    matches=[(i,o) for i,o in enumerate(owners) if stable(o['identity'])==stable(identity)]
    require(len(matches)<=1,'ambiguous independently captured compiler trait identity',api)
    if not matches:return {'disposition':'external-compiler-trait-with-exact-authenticated-declaration-source'}
    ci,owner=matches[0]
    require(owner['identity']==identity and owner['source_owner_kind']=='Trait','actual local compiler trait identity/kind conflict',api)
    source=target['source'];ni=[(i,n) for i,n in enumerate(native_owners) if n.get('owner')==target['owner'] and n.get('syntax')==source['syntax']]
    require(len(ni)==1 and source['roundtrip'] and ni[0][1]['roundtrip'],'resolved local trait does not identify its actual native owner/source',api)
    native_index,native=ni[0];membership=roots.relation(source['syntax'])
    return {'disposition':'exact-local-compiler-native-trait-owner','compiler_owner':ci,'native_owner':native_index,'association':association(owner,stage,api),'declaration':exact_declaration(owner,membership,roots,records,inputs,api),'parent':parent_relation(owner,native,owners,stage,roots,records,inputs,api,native_owners)}


def outlives(owner,native,membership,owners,parameter_edges,roots,records,inputs,api):
    predicates=owner['predicates']
    if predicates is None:return []
    by_owner={stable(o['identity']):o for o in owners};parameters={}
    for o in owners:
        for p in o['parameters']:parameters[stable(p['identity'])]=p
    order={};seen=set()
    def generics(o):
        key=stable(o['identity']);require(key not in seen,'cyclic actual compiler generic parent',api);seen.add(key)
        g=o['generic_parameter_order']
        if g is None:return
        if g['parent'] is not None:
            parent=by_owner.get(stable(g['parent']));require(parent is not None and parent['identity']==g['parent'],'missing actual inherited compiler generic owner',api);generics(parent)
        require(set(order)==set(range(g['parent_count'])),'incomplete actual inherited compiler generic index inventory',api)
        for p in g['parameters']:
            require(p['index'] not in order,'duplicate actual compiler generic index',api);order[p['index']]=p
    generics(owner)
    parent=predicates['parent_identity']
    if parent is not None:
        original=by_owner.get(stable(parent));require(original is not None and original['identity']==parent and original['predicates']==predicates['parent'],'actual compiler predicate inheritance identity conflict',api)
    else:require(predicates['parent'] is None,'orphan actual inherited predicate inventory',api)
    edges=[]
    for index,p in enumerate(predicates['own']):
        regions=p['outlives_region_authority']
        if regions is None:continue
        fact=p['fact']['fact'];expected=fact.get('region_outlives')
        if expected is None:
            require('type_outlives' in fact,'region authority on a non-outlives compiler clause',api);expected=[fact['type_outlives'][1]]
        require(len(expected)==len(regions),'actual compiler outlives region inventory conflict',api)
        bindings=[]
        for region,value in zip(regions,expected):
            kind=region['kind'];binding={'compiler_region':region}
            if kind=='EarlyBound':
                parameter=order.get(region['index']);require(parameter is not None and parameter['identity']==region['target'] and parameter['kind']['kind']=='lifetime','actual early outlives declaration/index conflict',api)
                require(value['kind']=='ReEarlyParam' and value['index']==region['index'],'typed outlives region/index observation conflict',api)
                native_parameter=[e for e in parameter_edges if e.get('compiler_parameter',{}).get('identity')==region['target']]
                require(len(native_parameter)==1,'missing/ambiguous exact native outlives lifetime declaration',api);binding['parameter_correspondence']=native_parameter[0]
            elif kind=='LateBound':
                variable=region['index'];bound=p['bound_variables']
                require(0<=variable<len(bound) and value['kind']=='ReBound' and value['variable']==variable and value['depth']==region['depth'],'actual late outlives binder/index/depth conflict',api)
                target=region['target'];decl=bound[variable].get('declaration')
                require(target==decl and (target is None or stable(target) in parameters),'actual late outlives binder/declaration conflict',api);binding['bound_variable']=bound[variable]
            elif kind=='LateParameter':
                require(value['kind']=='ReLateParam' and region['scope']==owner['identity'] and region['target'] is not None and stable(region['target']) in parameters,'actual late outlives scope/declaration conflict',api)
            elif kind=='StaticLifetime':require(value['kind']=='ReStatic','actual static outlives region conflict',api)
            else:require(False,'required declared outlives region authority unavailable: '+kind,api)
            bindings.append(binding)
        source=span_partition(p['source'],owner,membership,roots,records,inputs,api)
        edges.append({'compiler_predicate':index,'compiler':p,'source':source,'region_bindings':bindings})
    return edges
