"""Complete original callable/source proof for the retained syn::step bridge."""
from include_source_authority import require
from include_source_declarations import stable,association,exact_declaration
from include_source_parents import parent_relation
from include_source_tokens import Roots
from include_source_normalization import authenticate_source
from include_source_semantics import parameter_edges,binders_and_uses,trait_edges,span_partition
from include_source_constraints import local_trait_identity,outlives


def project(source,semantic,stage,inputs,api):
    original=semantic['original_syn']
    if original is None:
        require(stage['dependency_inventory'] is None and source['native']['dependency_bridge'] is None,'unexpected dependency authority in a context without the actual extern',api)
        return None
    native=source['native'];raw=original['syn-raw.json'];dependency_stage=stage['dependency_inventory'];cpp=semantic['compiler']
    require(dependency_stage['context']['crate']==raw['crate'] and dependency_stage['context']['cfg']==raw['cfg'] and dependency_stage['context']['target']==semantic['context']['target'] and dependency_stage['context']['test'] is False,'actual dependency compiler/stage context conflict',api)
    require(dependency_stage['semantic_export'] is False and dependency_stage['accepted_proof'] is False and all(dependency_stage[k]=='not_claimed_by_authorized_contract' for k in ('original_rustc_attr_id','compiler_original_attribute_attachment','exhaustive_consumed_item_lineage','preconsumption_to_final_survival')),'false dependency compiler-original attribute/lineage claim',api)
    require(dependency_stage['schema']==stage['schema'] and dependency_stage['capture_stage']==stage['capture_stage'] and dependency_stage['serialization_stage']==stage['serialization_stage'],'unknown actual dependency transformed stage',api)
    bridge=native['dependency_bridge'];require(len(bridge)==1,'missing/ambiguous complete original native callable bridge',api);bridge=bridge[0]
    roots=Roots(native['syntax_inventory']['roots'],native['physical']);caller=roots.relation(bridge['caller'])
    candidates=[]
    for call in cpp['calls']:
        if call['source']['kind']!='original':continue
        try:partition=span_partition(call['source'],None,caller,roots,cpp['source_files'],inputs,api)
        except api.Unsupported:continue
        if partition['physical_tokens']==[e['physical_token'] for e in caller['tokens']] and [call['source']['start'],call['source']['end']]==caller['physical_range']:candidates.append((call,partition))
    require(len(candidates)==1,'missing/ambiguous complete actual original compiler/native callable expression',api)
    call,call_partition=candidates[0]
    owners=raw['declaration_owners'];target=[o for o in owners if stable(o['identity'])==stable(call['target'])];parents=[o for o in owners if stable(o['identity'])==stable(call['parent'])]
    require(len(target)==len(parents)==1 and target[0]['parent']==parents[0]['identity'],'actual compiler callable target/parent identity conflict',api)
    owner,parent=target[0],parents[0]
    dependency_inputs={**inputs,'selected_cargo_root':next(t['src_path'] for t in original['original-build.json']['syn_package']['targets'] if 'lib' in t['kind'])}
    native_owners=[bridge['owner'],bridge['parent'],*bridge['module_owners']]
    all_parameters=[];joins=[]
    for compiler,observed in ((parent,bridge['parent']),(owner,bridge['owner'])):
        require(observed['roundtrip'],'actual original native callable/parent source has no ToDef roundtrip',api)
        membership=roots.relation(observed['syntax']);parameters=parameter_edges(compiler,observed,roots,raw['source_files'],dependency_inputs,api);all_parameters.extend(parameters)
        joins.append({'compiler_identity':compiler['identity'],'native_owner':observed['owner'],'membership':membership,'association':association(compiler,dependency_stage,api),'declaration':exact_declaration(compiler,membership,roots,raw['source_files'],dependency_inputs,api),'parent':parent_relation(compiler,observed,owners,dependency_stage,roots,raw['source_files'],dependency_inputs,api,native_owners),'parameters':parameters})
    for join,compiler,observed in zip(joins,(parent,owner),(bridge['parent'],bridge['owner'])):
        membership=join['membership'];join['binder_uses']=binders_and_uses(compiler,observed,membership,owners,all_parameters,roots,raw['source_files'],dependency_inputs,api)
        join['traits']=trait_edges(compiler,observed,membership,roots,raw['source_files'],dependency_inputs,api)
        for edge in join['traits']:
            identity=edge.get('compiler_identity',edge.get('compiler',{}).get('resolved'));edge['semantic_identity']=local_trait_identity(identity,edge['native'],owners,dependency_stage,native_owners,roots,raw['source_files'],dependency_inputs,api)
        join['outlives']=outlives(compiler,observed,membership,owners,all_parameters,roots,raw['source_files'],dependency_inputs,api)
    caller_owners=[o for o in cpp['declaration_owners'] if stable(o['identity'])==stable(call['owner'])]
    require(len(caller_owners)==1 and bridge['caller_owner']['roundtrip'],'missing actual original caller function identity/source',api);caller_owner=caller_owners[0]
    caller_membership=roots.relation(bridge['caller_owner']['syntax'])
    caller_edge={'compiler_owner':caller_owner['identity'],'native_owner':bridge['caller_owner']['owner'],'association':association(caller_owner,stage,api),'membership':caller_membership,'declaration':exact_declaration(caller_owner,caller_membership,roots,cpp['source_files'],inputs,api),'parent':parent_relation(caller_owner,bridge['caller_owner'],cpp['declaration_owners'],stage,roots,cpp['source_files'],inputs,api,native['owners'])}
    source_files=[]
    # Complete original dependency inventory stays independent; only the two
    # actual resolved callable owners are admitted by this bounded bridge.
    for record in raw['source_files']:
        if record['file']==owner['source']['file']:source_files.append(authenticate_source(record,inputs,api))
    require(source_files,'missing authenticated original callable SourceFile',api)
    return {'caller':{'compiler':call,'native_source_membership':caller,'source_partition':call_partition,'semantic_owner':caller_edge},'declaration_owners':joins,'source_files':source_files,'complete_original_owner_dispositions':[{'compiler_owner':i,'disposition':'admitted-original-resolved-callable-source' if o in (owner,parent) else 'outside-bounded-original-callable-relation'} for i,o in enumerate(owners)]}
