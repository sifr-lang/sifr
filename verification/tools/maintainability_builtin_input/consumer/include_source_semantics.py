"""Compiler binding meaning joined to exact native declaration/use source nodes."""
from include_source_authority import require
from include_source_declarations import stable,exact_source_node
from include_source_normalization import source_for


def span_partition(span,owner,membership,roots,records,inputs,api):
    filename=source_for(span,records,inputs,api)
    require(filename==membership['file'],'semantic span outside authenticated original owner file',api)
    physical=roots.root(membership['physical']['root']);tokens=roots.tokens(physical,membership['physical']['node'])
    intersect=[t for t in tokens if t['range'][0]<span['end'] and span['start']<t['range'][1]]
    require(all(span['start']<=t['range'][0]<t['range'][1]<=span['end'] for t in intersect),'semantic compiler span cuts a physical original token',api)
    edges=[e for e in membership['tokens'] if e['physical_token'] in {t['ordinal'] for t in intersect}]
    require(len(edges)==len(intersect),'semantic span lacks complete public native token origins',api)
    return {'compiler_source':span,'physical_tokens':[t['ordinal'] for t in intersect],'public_token_edges':edges,'disposition':'exact-original-token-partition' if intersect else 'compiler-empty-source-interval'}


def parameter_edges(owner,native,roots,records,inputs,api):
    expected=[p for p in owner['parameters'] if p['origin']=='Generics' and p['source_parameter_disposition']!='compiler-elided-lifetime-parameter']
    generics=native['semantics']['generics'];observed=[] if generics is None else generics['parameters']
    order=owner['generic_parameter_order'];self_parameter=bool(owner['identity']['kind']=='Trait' and order and order['has_self'])
    require(len(observed)==len(expected)+int(self_parameter),'independent original compiler/native explicit generic parameter inventory conflict',api)
    edges=[]
    for index,p in enumerate(observed):
        require(p['index']==index and p['owner']==generics['owner'],'actual native generic parameter ownership/order conflict',api)
        if self_parameter and index==0:
            source=p['source'];first=order['parameters'][0]
            require(first['index']==0 and first['kind']['kind']=='type' and p['kind']=='type' and p['implicit'] and source is not None and source['disposition']=='implicit-trait-self-owner-source' and source['roundtrip'],'implicit trait Self actual owner-source disposition conflict',api)
            require(source['syntax']==native['syntax'],'implicit trait Self belongs to wrong actual native trait owner',api)
            edges.append({'compiler_parameter':first,'native_parameter':p,'disposition':'implicit-trait-self-owner-source'});continue
        cp=expected[index-int(self_parameter)];source=p['source']
        if cp['source_parameter_disposition']=='compiler-implicit-impl-trait-parameter':
            require(p['kind']=='type' and p['implicit'],'compiler/native implicit impl Trait parameter conflict',api)
            source_edge=exact_source_node(cp['source'],native['syntax'],roots,records,inputs,api,{'IMPL_TRAIT_TYPE'})
            if source is not None:require(source['roundtrip'],'available actual implicit parameter source does not roundtrip',api)
            disposition='compiler-implicit-impl-trait-parameter'
        else:
            require(source is not None and source['roundtrip'] and source['disposition']=='explicit-parameter-source','missing original explicit native generic parameter source/to_def',api)
            name=cp['name_source'] or cp['source']
            source_edge=exact_source_node(name,source['syntax'],roots,records,inputs,api,{'NAME','LIFETIME'})
            require((cp['identity']['kind']=='LifetimeParam')==(p['kind']=='lifetime') and (cp['identity']['kind']=='TyParam')==(p['kind']=='type'),'actual compiler/native generic parameter kind conflict',api)
            disposition='explicit-source-parameter'
        generic=[g for g in (order['parameters'] if order is not None else []) if stable(g['identity'])==stable(cp['identity'])]
        require(len(generic)<=1,'ambiguous actual compiler early generic parameter index',api)
        if generic:require(generic[0]['index']==index+(order['parent_count'] if order is not None else 0),'compiler/native own versus inherited generic index conflict',api)
        edges.append({'compiler_parameter':cp,'native_parameter':p,'source':source_edge,'generic_parameter_authority':generic,'disposition':disposition})
    return edges


def binders_and_uses(owner,native,membership,owners,all_parameter_edges,roots,records,inputs,api):
    parameters={}
    for o in owners:
        for p in o['parameters']:
            key=stable(p['identity']);require(key not in parameters,'duplicate independently enumerated original compiler parameter identity',api);parameters[key]=p
    binders=[]
    for b in owner['binders']:
        require(b['compiler_map_disposition']=='present' and b['variables'] is not None,'required original compiler binder map unavailable',api)
        require(all(v['ordinal']==i and v['kind'] in {'NamedRegion','AnonymousRegion'} for i,v in enumerate(b['variables'])),'unsupported original compiler binder variable inventory',api)
        require(all(stable(p) in parameters for p in b['parameters']),'orphan original compiler binder declaration',api)
        binders.append({'compiler':b,'source':span_partition(b['source'],owner,membership,roots,records,inputs,api)})
    declarations=[];accounted=set();uses=[]
    for p in owner['parameters']:
        span=(p['name_source'] or p['source']) if p['source_parameter_disposition']=='explicit-source-parameter' else p['source']
        edge=span_partition(span,owner,membership,roots,records,inputs,api)
        if p['source_parameter_disposition']=='explicit-source-parameter':
            node=exact_source_node(span,native['syntax'],roots,records,inputs,api,{'LIFETIME','NAME'})
            declarations.append({'compiler_parameter':p,'source':node})
            for i,l in enumerate(native['semantics']['lifetimes']):
                if l['syntax']==node['native']:accounted.add(i)
        else:declarations.append({'compiler_parameter':p,'source':edge,'disposition':p['source_parameter_disposition']})
    for use in owner['lifetime_occurrences']:
        resolved=use['resolved'];require(resolved['kind'] in {'EarlyBound','LateBound','Free','StaticLifetime'},'unsupported/missing original compiler lifetime binding',api)
        target=use['target'];parameter=None
        if target is not None:
            require(stable(target) in parameters and resolved.get('target')==target,'orphan/inconsistent original compiler lifetime target',api);parameter=parameters[stable(target)]
        elif resolved['kind']!='StaticLifetime':require(False,'missing original compiler lifetime declaration identity',api)
        source=span_partition(use['source'],owner,membership,roots,records,inputs,api)
        if resolved['kind']=='LateBound':
            matches=[b for b in owner['binders'] if target in b['parameters'] and resolved['index']<len(b['variables']) and (b['variables'][resolved['index']]['kind']=='AnonymousRegion' or b['variables'][resolved['index']].get('declaration')==target)]
            require(len(matches)==1,'missing/ambiguous actual compiler binder/index/declaration relation',api)
            source['compiler_binder']=matches[0]['id']
        if use['syntax']=='Implicit':
            require(parameter is None or parameter['source_parameter_disposition']=='compiler-elided-lifetime-parameter','implicit lifetime has wrong actual compiler declaration disposition',api)
            uses.append({'compiler':use,'source':source,'disposition':'compiler-implicit-lifetime-with-authenticated-owner-source'});continue
        node=exact_source_node(use['source'],native['syntax'],roots,records,inputs,api,{'LIFETIME'})
        candidates=[(i,l) for i,l in enumerate(native['semantics']['lifetimes']) if l['syntax']==node['native']]
        require(len(candidates)==1,'missing/ambiguous independently enumerated native lifetime occurrence',api)
        ni,ra=candidates[0];accounted.add(ni)
        if resolved['kind']=='StaticLifetime':require(ra['resolved'] is None,'static lifetime cannot claim a native parameter declaration',api);disposition='compiler-static-lifetime'
        elif parameter['source_parameter_disposition']=='compiler-elided-lifetime-parameter':
            require(use['syntax']=='ExplicitAnonymous','elided declaration has wrong actual lifetime syntax disposition',api);disposition='compiler-explicit-anonymous-lifetime'
        elif parameter['origin']=='Binder':
            require(resolved['kind']=='LateBound','genuine HRTB use lost compiler late-bound identity',api);disposition='compiler-hir-binder-identity-with-exact-native-source'
        else:
            require(ra['resolved'] is not None and ra['resolved']['source'] is not None and ra['resolved']['source']['roundtrip'],'ordinary explicit/inherited native lifetime identity unavailable',api)
            matches=[e for e in all_parameter_edges if e.get('compiler_parameter',{}).get('identity')==parameter['identity'] and e['native_parameter']['owner']==ra['resolved']['owner'] and e['native_parameter']['index']==ra['resolved']['index'] and e['native_parameter']['source']['syntax']==ra['resolved']['source']['syntax']]
            require(len(matches)==1,'resolved native lifetime target/owner/index conflicts with original compiler declaration',api);source['parameter_correspondence']=matches[0];disposition='resolved-ordinary-or-inherited-parameter'
        uses.append({'compiler':use,'source':node,'binding':source,'native':ra,'disposition':disposition})
    require(accounted==set(range(len(native['semantics']['lifetimes']))),'incomplete independent native declaration/use lifetime multiset',api)
    return {'declarations':declarations,'binders':binders,'uses':uses}


def trait_edges(owner,native,membership,roots,records,inputs,api):
    edges=[];seen=set()
    traits=owner['trait_constraints']
    for trait in traits:
        node=exact_source_node(trait['source'],native['syntax'],roots,records,inputs,api,{'PATH'})
        matches=[(i,t) for i,t in enumerate(native['semantics']['traits']) if t['syntax']==node['native']]
        require(len(matches)==1 and trait['resolved'] is not None,'missing/ambiguous actual compiler/native resolved trait path',api)
        index,target=matches[0];seen.add(index)
        require(target['source'] is not None and target['source']['roundtrip'],'actual resolved native trait lacks source/to_def roundtrip',api)
        declaration=trait['declaration_source'];source_for(declaration,records,inputs,api)
        name=exact_source_node(trait['declaration_name_source'],target['source']['syntax'],roots,records,inputs,api,{'NAME'})
        full=roots.relation(target['source']['syntax'])
        require(name['physical']['root']==full['physical']['root'] and declaration['start']<=trait['declaration_name_source']['start']<trait['declaration_name_source']['end']<=declaration['end'],'original compiler trait declaration/name source conflict',api)
        require(full['file'] in inputs['files'],'whole resolved trait physical source lacks independent input authority',api)
        edges.append({'compiler':trait,'native':target,'use_source':node,'declaration_name_source':name,'whole_native_trait_membership':full,'compiler_declaration_source':declaration,'compiler_declaration_header_partition':span_partition(declaration,owner,full,roots,records,inputs,api)})
    # Implemented trait paths are independent from PolyTraitRef bounds.
    if owner['implemented_trait'] is not None:
        remaining=[(i,t) for i,t in enumerate(native['semantics']['traits']) if i not in seen]
        require(len(remaining)==1,'missing/ambiguous actual native implemented trait path',api)
        index,target=remaining[0];seen.add(index)
        require(target['source'] is not None and target['source']['roundtrip'],'implemented native trait lacks source/to_def roundtrip',api)
        declaration=owner['implemented_trait_source'];source_for(declaration,records,inputs,api)
        name=exact_source_node(owner['implemented_trait_name_source'],target['source']['syntax'],roots,records,inputs,api,{'NAME'})
        edges.append({'compiler_identity':owner['implemented_trait'],'compiler_declaration_source':declaration,'declaration_name_source':name,'native':target,'whole_native_trait_membership':roots.relation(target['source']['syntax']),'compiler_declaration_header_partition':span_partition(declaration,owner,roots.relation(target['source']['syntax']),roots,records,inputs,api)})
    require(seen==set(range(len(native['semantics']['traits']))),'unreconciled independent native trait path inventory',api)
    return edges
