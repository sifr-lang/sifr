"""Closed projection from three intact independent diagnostic inventories."""
from include_source_authority import require
from include_source_tokens import Roots
from include_source_declarations import stable,syntax_kind,exact_declaration,association,declaration_tokens
from include_source_parents import parent_relation
from include_source_normalization import path,authenticate_source
from include_source_semantics import parameter_edges,binders_and_uses,trait_edges
import include_source_encoding as canonical

SCHEMA='sifr-maintainability-include-source-relation-v1'
LOST=('original_rustc_attr_id','compiler_original_attribute_attachment','exhaustive_consumed_item_lineage','preconsumption_to_final_survival')
NOT_CLAIMED='not_claimed_by_authorized_contract'


def project(source,semantic,stage,inputs,api):
    native=source['native'];compiler=semantic['compiler'];owners=compiler['declaration_owners']
    require(native['semantic_export'] is False and compiler['semantic_export'] is False and stage['semantic_export'] is False,'semantic export is not admitted by diagnostic contract',api)
    require(stage['schema']=='development-transformed-attribute-inventory-v2' and all(stage.get(k)==NOT_CLAIMED for k in LOST),'false compiler-original attribute/consumption/survival claim',api)
    require(stage['capture_stage']=='rustc-after-expansion-resolver-for-lowering' and stage['serialization_stage']=='rustc-after-analysis','unknown transformed attribute capture stage',api)
    roots=Roots(native['syntax_inventory']['roots'],native['physical'])
    descriptors=native['syntax_inventory']['roots'];contexts=native['include_contexts']
    require({c['root']['root'] for c in contexts}=={i for i,r in enumerate(descriptors) if r['is_include']},'incomplete independent original include context inventory',api)
    require(len({c['root']['root'] for c in contexts})==len(contexts),'duplicate original include context',api)
    root_edges=[]
    for context in contexts:
        require(context['context']['include_ancestors'],'missing actual original include invocation ancestry',api)
        root_edges.append({'context':context,'membership':roots.relation(context['root'],True)})
    included_files={e['membership']['file'] for e in root_edges}
    for physical in native['physical']:
        from pathlib import Path
        require(physical['file'] in inputs['files'],'complete physical source inventory lacks original raw input authority',api)
        require(roots.root(physical['syntax']['root'])['text'].encode()==Path(physical['file']).read_bytes(),'physical original AST/raw source byte inventory conflict',api)
    records=compiler['source_files'];normalizations=[]
    for record in records:
        if path(inputs['root'],record['file']) in included_files:normalizations.append(authenticate_source(record,inputs,api))
    require({s['file'] for s in normalizations}==included_files,'missing complete original included SourceFile/raw normalization authority',api)
    native_relations=[];declaration_index={}
    for index,observed in enumerate(native['owners']):
        if observed.get('is_include'):
            require(observed['roundtrip'],'required original native owner has no source/to_def roundtrip',api)
            membership=roots.relation(observed['syntax']);native_relations.append((index,observed,membership))
            tokens=declaration_tokens(roots,membership['physical'])
            require(tokens,'required original native declaration has empty nonattribute token inventory',api)
            key=(membership['file'],observed['kind'],tokens[0]['range'][0],tokens[-1]['range'][1])
            declaration_index.setdefault(key,[]).append((index,observed,membership))
    required=[i for i,o in enumerate(owners) if o['source']['kind']=='original' and path(inputs['root'],o['source']['file']) in included_files]
    joins=[];imports=[];used_native=set()
    for index in required:
        owner=owners[index]
        if owner['identity']['kind']=='Use':imports.append(index);continue
        matches=[]
        source_span=owner['source'];key=(path(inputs['root'],source_span['file']),syntax_kind(owner),source_span['start'],source_span['end'])
        for ni,observed,membership in declaration_index.get(key,[]):
            try:
                declaration=exact_declaration(owner,membership,roots,records,inputs,api)
                parent=parent_relation(owner,observed,owners,stage,roots,records,inputs,api,native['owners'])
            except (ValueError,api.Unsupported):continue
            matches.append({'compiler_owner':index,'native_owner':ni,'source_native_attribute_membership':membership,'compiler_semantic_owner_identity':{'association':association(owner,stage,api),'declaration':declaration,'parent_relation':parent,'parameters':owner['parameters'],'binders':owner['binders'],'uses':owner['lifetime_occurrences'],'traits':owner['trait_constraints'],'implemented_trait':owner['implemented_trait'],'generics':owner['generics'],'signature':owner['signature'],'predicates':owner['predicates']}})
        require(len(matches)==1,'missing/ambiguous required original include semantic owner correspondence: '+owner['identity']['raw'],api)
        require(matches[0]['native_owner'] not in used_native,'multiple original compiler owners claim one native source owner',api)
        used_native.add(matches[0]['native_owner']);joins.append(matches[0])
    all_parameters=[];parent_frames={}
    def add_parent_frames(frame):
        if 'compiler_owner' in frame:
            key=(frame['compiler_owner'],frame['native_owner_index']);parent_frames[key]=frame
        if 'ancestry' in frame:add_parent_frames(frame['ancestry'])
    for joined in joins:add_parent_frames(joined['compiler_semantic_owner_identity']['parent_relation'])
    for (compiler_index,native_index),frame in parent_frames.items():
        if compiler_index not in required:
            frame['parameter_correspondences']=parameter_edges(owners[compiler_index],native['owners'][native_index],roots,records,inputs,api)
            all_parameters.extend(frame['parameter_correspondences'])
    for joined in joins:
        owner=owners[joined['compiler_owner']];observed=native['owners'][joined['native_owner']]
        parameters=parameter_edges(owner,observed,roots,records,inputs,api)
        joined['compiler_semantic_owner_identity']['parameter_correspondences']=parameters
        all_parameters.extend(parameters)
    used_source_starts=set()
    def source_starts(value):
        if isinstance(value,dict):
            if value.get('kind')=='original' and 'source_file_start_pos' in value:used_source_starts.add(value['source_file_start_pos'])
            for v in value.values():source_starts(v)
        elif isinstance(value,list):
            for v in value:source_starts(v)
    for index in set(required)|{key[0] for key in parent_frames}:source_starts(owners[index])
    normalized_starts={s['normalization_authority']['start_pos'] for s in normalizations}
    for record in records:
        if record['start_pos'] in used_source_starts-normalized_starts:normalizations.append(authenticate_source(record,inputs,api))
    require(used_source_starts<={s['normalization_authority']['start_pos'] for s in normalizations},'required semantic source lacks complete original normalization/raw hash authority',api)
    for joined in joins:
        owner=owners[joined['compiler_owner']];observed=native['owners'][joined['native_owner']]
        joined['compiler_semantic_owner_identity']['binder_use_correspondences']=binders_and_uses(owner,observed,joined['source_native_attribute_membership'],owners,all_parameters,roots,records,inputs,api)
        joined['compiler_semantic_owner_identity']['trait_correspondences']=trait_edges(owner,observed,joined['source_native_attribute_membership'],roots,records,inputs,api)
        from include_source_constraints import local_trait_identity,outlives
        for edge in joined['compiler_semantic_owner_identity']['trait_correspondences']:
            target=edge.get('compiler_identity',edge.get('compiler',{}).get('resolved'))
            edge['semantic_identity']=local_trait_identity(target,edge['native'],owners,stage,native['owners'],roots,records,inputs,api)
        joined['compiler_semantic_owner_identity']['outlives_correspondences']=outlives(owner,observed,joined['source_native_attribute_membership'],owners,all_parameters,roots,records,inputs,api)
    # Imports have actual compiler identities and exact native source subnodes.
    # Their semantic parent is a genuine declaration; imported targets never
    # substitute for the import's compiler owner or pretend to be import ToDef.
    for index in imports:
        owner=owners[index];parents=[j for j in joins if stable(owners[j['compiler_owner']]['identity'])==stable(owner['parent'])]
        require(len(parents)==1,'required original import lacks exact admitted native semantic parent',api)
        parent=parents[0];observed=native['owners'][parent['native_owner']];reference=observed['syntax'];root=roots.root(reference['root']);matches=[]
        for node in roots.descendants(root,reference['node']):
            if root['nodes'][node]['kind'] not in {'USE','USE_TREE'}:continue
            try:
                membership=roots.relation({'root':reference['root'],'node':node})
                declaration=exact_declaration(owner,membership,roots,records,inputs,api)
            except (ValueError,api.Unsupported):continue
            matches.append({'compiler_owner':index,'native_owner':None,'disposition':'original-import-source-subnode-under-actual-semantic-parent','source_native_attribute_membership':membership,'compiler_semantic_owner_identity':{'association':association(owner,stage,api),'declaration':declaration,'parent_compiler_owner':parent['compiler_owner'],'parent_native_owner':parent['native_owner']}})
        require(len(matches)==1,'missing/ambiguous exact original import source-subnode disposition',api)
        joins.append(matches[0])
    require(used_native=={i for i,_,_ in native_relations},'unreconciled independently enumerated native include owner',api)
    dispositions=[{'compiler_owner':i,'disposition':'admitted-exact-include-source' if i in required else ('compiler-'+o['source']['kind'] if o['source']['kind']!='original' else 'original-owner-outside-include-relation')} for i,o in enumerate(owners)]
    native_dispositions=[{'native_owner':i,'disposition':'admitted-exact-include-source' if i in used_native else o.get('disposition','native-owner-outside-include-relation')} for i,o in enumerate(native['owners'])]
    from include_source_dependency import project as dependency_project
    exact_dependency=dependency_project(source,semantic,stage,inputs,api)
    syn_relation=None
    if semantic.get('original_syn') is not None:
        import source_binder
        syn_relation=source_binder._project({**semantic['original_syn'],'caller-raw.json':compiler},inputs,api,semantic['context'])
    return {'schema':SCHEMA,'semantic_export':False,**{k:NOT_CLAIMED for k in LOST},'source_native_attribute_membership':{'capture_binding':canonical.digest(source),'normalizations':normalizations,'include_roots':root_edges,'physical_inventory':native['physical'],'native_dispositions':native_dispositions},'compiler_semantic_owner_identity':{'capture_binding':canonical.digest(semantic),'complete_owner_dispositions':dispositions,'required_owner_indices':required,'correspondences':sorted(joins,key=lambda j:j['compiler_owner']),'original_syn_correspondence':syn_relation,'exact_original_dependency_source':exact_dependency},'transformed_attribute_observations':{'capture_binding':canonical.digest(stage),'stage':stage},'input_binding':canonical.digest(inputs)}
