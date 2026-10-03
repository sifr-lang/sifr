"""Exact declaration partitions; physical membership is not compiler identity."""
from include_source_authority import require
from include_source_normalization import source_for

KINDS={'Fn':'FN','AssocFn':'FN','Struct':'STRUCT','Enum':'ENUM','Union':'UNION','Trait':'TRAIT','TraitAlias':'TRAIT_ALIAS','Const':'CONST','AssocConst':'CONST','Static':'STATIC','TyAlias':'TYPE_ALIAS','AssocTy':'TYPE_ALIAS','Mod':'MODULE','Use':'USE_TREE'}


def stable(identity):
    return identity['crate'],identity['hash']


def syntax_kind(owner):
    if owner['source_owner_kind']=='Macro':return 'MACRO_RULES' if owner['macro_rules'] else 'MACRO_DEF'
    return {'Function':'FN','Struct':'STRUCT','Enum':'ENUM','Union':'UNION','Trait':'TRAIT','TraitAlias':'TRAIT_ALIAS','Impl':'IMPL','Const':'CONST','Static':'STATIC','TypeAlias':'TYPE_ALIAS','Module':'MODULE','Use':'USE_TREE'}.get(owner['source_owner_kind'])


def declaration_tokens(roots,reference):
    root=roots.root(reference['root']);node=reference['node'];attrs=set()
    for attribute in root['nodes'][node]['direct_attributes']:
        attrs.update(t['ordinal'] for t in roots.tokens(root,attribute['node']))
    return [t for t in roots.tokens(root,node) if t['ordinal'] not in attrs]


def exact_declaration(owner,relation,roots,records,inputs,api):
    span=owner['source'];filename=source_for(span,records,inputs,api)
    require(filename==relation['file'],'compiler declaration/physical file identity conflict',api)
    tokens=declaration_tokens(roots,relation['physical'])
    require(tokens and all(span['start']<=t['range'][0]<t['range'][1]<=span['end'] for t in tokens),'compiler declaration fails complete independent token partition',api)
    require(tokens[0]['range'][0]==span['start'] and tokens[-1]['range'][1]==span['end'],'compiler declaration exact boundary conflict',api)
    # Coverage is certified by every token/subnode edge, not this separate span.
    return {'compiler_declaration':span,'whole_physical_owner_range':relation['physical_range'],'declaration_physical_tokens':[t['ordinal'] for t in tokens]}


def association(owner,stage,api):
    identity=stable(owner['identity'])
    candidates=[(i,o) for i,o in enumerate(stage['owners']) if stable(o['identity'])==identity]
    if not candidates and owner['source_owner_kind']=='Use':
        # Nested use-tree definitions are real compiler owners without a
        # separate expanded Item. Their actual resolver association still
        # supplies NodeId/LocalDefId/final HIR and semantic-parent identity.
        rows=[r for r in stage['resolver_associations'] if stable(r['identity'])==identity]
        require(len(rows)==1,'missing/ambiguous actual nested use-tree resolver association',api)
        row=rows[0]
        require(row['identity']==owner['identity'] and row['identity']['kind']=='Use' and row['local_def_id']==owner['local_def_id'] and row['hir_id']==owner['hir_id'] and row['parent']==owner['parent'],'actual nested use-tree NodeId/LocalDefId/HIR/parent conflict',api)
        require(sum(r['node_id']==row['node_id'] for r in stage['resolver_associations'])==1,'ambiguous actual nested use-tree NodeId',api)
        return {'stage_owner':None,'stage_owner_disposition':'actual-resolver-use-definition-without-separate-expanded-item',**row}
    require(len(candidates)==1,'missing/ambiguous independent expanded semantic owner',api)
    index,observed=candidates[0]
    require(observed['identity']==owner['identity'] and observed['local_def_id']==owner['local_def_id'] and observed['parent']==owner['parent'] and observed['declaration_source']==owner['source'],'expanded/analyzed actual owner/parent/declaration conflict',api)
    rows=[r for r in stage['resolver_associations'] if r['node_id']==observed['node_id']]
    require(len(rows)==1,'missing/ambiguous actual NodeId resolver association',api)
    row=rows[0]
    require(row['identity']==owner['identity'] and row['local_def_id']==owner['local_def_id'] and row['hir_id']==owner['hir_id'] and row['parent']==owner['parent'],'actual NodeId/LocalDefId/HIR/final semantic identity conflict',api)
    return {'stage_owner':index,'node_id':row['node_id'],'local_def_id':row['local_def_id'],'hir_id':row['hir_id'],'identity':owner['identity'],'parent':owner['parent']}


def exact_source_node(span,reference,roots,records,inputs,api,kinds=None):
    filename=source_for(span,records,inputs,api);relation=roots.relation(reference)
    require(filename==relation['file'],'semantic source subnode physical file conflict',api)
    root=roots.root(relation['physical']['root']);node=relation['physical']['node']
    found=[i for i in roots.descendants(root,node) if root['nodes'][i]['range']==[span['start'],span['end']] and (kinds is None or root['nodes'][i]['kind'] in kinds)]
    require(len(found)==1,'missing/ambiguous exact semantic source subnode',api)
    physical=found[0]
    edges=[e for e in relation['subnodes'] if e['physical_node']==physical]
    require(len(edges)==1,'missing/ambiguous complete native semantic subnode origin',api)
    return {'compiler_source':span,'physical':{'root':relation['physical']['root'],'node':physical},'native':{'root':reference['root'],'node':edges[0]['native_node']},'kind':root['nodes'][physical]['kind']}
