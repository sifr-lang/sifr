"""Semantic parent joins through actual native source/to_def observations."""
from include_source_authority import require
from include_source_declarations import stable,exact_declaration,association


def parent_relation(owner,native,owners,stage,roots,records,inputs,api,native_owners,seen=()):
    key=stable(owner['identity'])
    require(key not in seen,'cyclic original semantic owner parent relation',api)
    parents=[o for o in owners if stable(o['identity'])==stable(owner['parent'])]
    require(len(parents)<=1,'ambiguous original compiler semantic parent',api)
    context=native['context'];references=[]
    for p in context['semantic_parents']:
        references.append((p['syntax'],p['roundtrip'],p['kind'],p['owner']))
    if context['module_declaration'] is not None:
        p=context['module_declaration'];references.append((p['syntax'],p['roundtrip'],'Module',native['module']))
    for p in context['module_ancestors']:
        references.append((p['syntax'],p['roundtrip'],'Module',p['owner']))
        # Out-of-line module definitions are SourceFiles. Their separately
        # enumerated declaration source belongs to the same actual Module ID.
        # Keep both roles; the definition range cannot replace the declaration.
        for declaration in native_owners:
            if declaration.get('owner')==p['owner'] and declaration.get('kind')=='MODULE':
                references.append((declaration['syntax'],declaration['roundtrip'],'Module',p['owner']))
    if parents:
        parent=parents[0];matches=[]
        for reference,roundtrip,kind,native_owner in references:
            if not roundtrip:continue
            try:
                relation=roots.relation(reference)
                edge=exact_declaration(parent,relation,roots,records,inputs,api)
            except (ValueError,api.Unsupported):continue
            source_owners=[p for p in native_owners if p.get('syntax')==reference and p.get('owner')==native_owner]
            if len(source_owners)!=1:continue
            try:ancestry=parent_relation(parent,source_owners[0],owners,stage,roots,records,inputs,api,native_owners,(*seen,key))
            except api.Unsupported:continue
            matches.append({'syntax':reference,'kind':kind,'native_owner':native_owner,'compiler_parent':parent['identity'],'compiler_owner':owners.index(parent),'native_owner_index':native_owners.index(source_owners[0]),'compiler_association':association(parent,stage,api),'source':edge,'ancestry':ancestry})
        require(len(matches)==1,'missing/ambiguous exact required semantic parent source/to_def relation',api)
        return matches[0]
    # The actual crate-root resolver association has no parent. A SourceFile
    # native module source/to_def is the separate native root authority.
    root=[a for a in stage['resolver_associations'] if stable(a['identity'])==stable(owner['parent']) and a['parent'] is None]
    require(len(root)==1 and root[0]['node_id']==0,'missing actual final compiler crate-root owner association',api)
    selected=inputs['selected_cargo_root'];candidates=[]
    source=native['module_source']
    if source['roundtrip'] and source['kind']=='SOURCE_FILE' and source['aggregate_original'] is not None:
        if source['aggregate_original']['file']==selected:candidates.append({'native_source':source})
    for ancestor in context['module_ancestors']:
        if ancestor['roundtrip'] and ancestor['kind']=='SOURCE_FILE':
            original=roots.root(ancestor['syntax']['root'])['nodes'][ancestor['syntax']['node']]['aggregate_original']
            origin_root=roots.root(ancestor['syntax']['root'])
            if original is not None and origin_root['origin_files'][original['file']]['path']==selected:candidates.append({'native_source':ancestor})
    require(len(candidates)==1 and selected in inputs['files'],'missing/ambiguous selected original Cargo root/native crate-root source/to_def relation',api)
    source=candidates[0]['native_source']
    return {'kind':'CrateRoot','compiler_parent':owner['parent'],'compiler_association':root[0],'native_module':native['module'],'native_source':source,'selected_cargo_root':selected}
