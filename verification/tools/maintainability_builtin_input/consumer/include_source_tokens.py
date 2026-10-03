"""Complete ordered token/subnode relations from caller-held original roots."""
from collections import defaultdict
from functools import lru_cache
import json
from pathlib import Path

PARTS={'->','::','..','..=','=>','&&','||','==','!=','<=','>=','<<','>>','+=','-=','*=','/=','%=','&=','|=','^='}

def require(condition,message):
    if not condition:raise ValueError(message)

class Roots:
    def __init__(self,descriptors,physical):
        self.root=lru_cache(maxsize=8)(self.root)
        self._relation=lru_cache(maxsize=128)(self._relation)
        self.descriptors=descriptors
        self.physical={}
        for p in physical:
            key=(p['file'],p['editioned_file'])
            require(key not in self.physical,'duplicate physical file/edition authority')
            self.physical[key]=p['syntax']['root']
    def root(self,index):
        r=json.loads(Path(self.descriptors[index]['path']).read_bytes())
        require(len(r['nodes'])==self.descriptors[index]['node_count'] and len(r['tokens'])==self.descriptors[index]['token_count'],'complete original root cardinality conflict')
        require(r['schema']=='development-complete-syntax-root-inventory-v2' and r['semantic_export'] is False and r['accepted_proof'] is False,'unknown original syntax root schema/status')
        encoded=r['text'].encode();position=r['nodes'][0]['range'][0]
        for token in r['tokens']:
            require(token['range'][0]==position and encoded[token['range'][0]:token['range'][1]]==token['text'].encode(),'incomplete original syntax token/trivia byte partition')
            position=token['range'][1]
        require(position==len(encoded)==r['nodes'][0]['range'][1],'original syntax root token/text boundary conflict')
        members=[[] for _ in r['nodes']]
        for ti,t in enumerate(r['tokens']):
            require(t['ordinal']==ti,'original token ordinal conflict')
            if not t['trivia'] or r['nodes'][t['parent']]['kind']=='DOC_COMMENT':members[t['parent']].append(t['ordinal'])
        for i in reversed(range(len(r['nodes']))):
            n=r['nodes'][i]
            require(n['ordinal']==i,'original node ordinal conflict')
            for c in n['children']:
                require(c>i and r['nodes'][c]['parent']==n['ordinal'],'original ancestor membership conflict')
                members[n['ordinal']].extend(members[c])
            members[n['ordinal']].sort()
        r['_members']=members
        return r
    def tokens(self,r,node):return [r['tokens'][i] for i in r['_members'][node]]
    def descendants(self,r,node):
        result=[];queue=[node]
        while queue:
            i=queue.pop();result.append(i);queue.extend(reversed(r['nodes'][i]['children']))
        return result
    def token_edges(self,nr,ni,pr,pi,file):
        native=self.tokens(nr,ni);physical=self.tokens(pr,pi)
        lookup={tuple(t['range']):t for t in physical};groups=defaultdict(list)
        for t in native:
            origin=t['original'];require(origin is not None,'absent required public token origin')
            require(nr['origin_files'][origin['file']]['path']==file,'cross-file native token origin')
            rng=tuple(origin['range']);target=lookup.get(rng)
            if target is None:
                candidates=[p for p in physical if p['range'][0]<=rng[0]<rng[1]<=p['range'][1]]
                require(len(candidates)==1,'missing/ambiguous physical token origin')
                target=candidates[0]
            groups[target['ordinal']].append(t)
        require(set(groups)=={t['ordinal'] for t in physical},'incomplete independent physical/native token multiset')
        edges=[]
        for p in physical:
            ts=groups[p['ordinal']]
            require(''.join(t['text'] for t in ts)==p['text'],'original public-origin token group bytes conflict')
            intervals=[tuple(t['original']['range']) for t in ts]
            if len(ts)==1:
                require(intervals==[tuple(p['range'])] and ts[0]['kind']==p['kind'],'physical/native original token kind/range conflict')
            else:
                require(p['text'] in PARTS,'unknown token subdivision')
                require(all(i==tuple(p['range']) for i in intervals) or intervals==[(p['range'][0]+i,p['range'][0]+i+1) for i in range(len(ts))],'incomplete public punctuation origin group')
            edges.append({'physical_token':p['ordinal'],'native_tokens':[t['ordinal'] for t in ts],'original_intervals':[list(i) for i in intervals],'kind':p['kind']})
        return edges
    def relation(self,reference,include_root=False):
        return self._relation(reference['root'],reference['node'],include_root)
    def _relation(self,root_index,node_index,include_root):
        # Root indices are actual HirFileId inventories, including distinct
        # include invocations. Physical file identity alone never keys reuse.
        reference={'root':root_index,'node':node_index}
        nr=self.root(reference['root']);ni=reference['node'];native=self.tokens(nr,ni)
        require(native,'empty required owner token inventory')
        files={(nr['origin_files'][t['original']['file']]['path'],nr['origin_files'][t['original']['file']]['file']) for t in native if t['original'] is not None}
        require(len(files)==1,'missing/mixed original native owner files');key=files.pop();file=key[0]
        require(key in self.physical,'native owner original file absent from complete physical inventory')
        pri=self.physical[key];pr=self.root(pri);kind=nr['nodes'][ni]['kind'];matches=[]
        for pn in pr['nodes']:
            if pn['kind']!=kind and not (include_root and ni==0 and kind=='MACRO_ITEMS' and pn['ordinal']==0 and pn['kind']=='SOURCE_FILE'):continue
            if not all(t['original'] is not None and pn['range'][0]<=t['original']['range'][0]<t['original']['range'][1]<=pn['range'][1] for t in native):continue
            try:edges=self.token_edges(nr,ni,pr,pn['ordinal'],file)
            except ValueError:continue
            matches.append((pn['ordinal'],edges))
        require(len(matches)==1,'missing/ambiguous complete physical owner token relation')
        pi,edges=matches[0];mapping={i:e['physical_token'] for e in edges for i in e['native_tokens']}
        signatures=defaultdict(list)
        for i in self.descendants(pr,pi):signatures[(pr['nodes'][i]['kind'],tuple(pr['_members'][i]))].append(i)
        nodes=[];node_map={}
        for i in self.descendants(nr,ni):
            n=nr['nodes'][i];signature=tuple(dict.fromkeys(mapping[t['ordinal']] for t in self.tokens(nr,i)))
            physical_kind='SOURCE_FILE' if include_root and i==0 and n['kind']=='MACRO_ITEMS' else n['kind']
            candidates=signatures[(physical_kind,signature)]
            require(len(candidates)==1,'missing/ambiguous complete original source subnode relation: '+n['kind'])
            target=candidates[0];node_map[i]=target
            nodes.append({'native_node':i,'physical_node':target,'kind':n['kind'],'aggregate_original':n['aggregate_original'],'contextual_original':n['contextual_original']})
        attributes=[]
        for i in self.descendants(nr,ni):
            pn=pr['nodes'][node_map[i]];nn=nr['nodes'][i]
            require(len(nn['direct_attributes'])==len(pn['direct_attributes']),'source/native direct attribute inventory cardinality conflict')
            for na,pa in zip(nn['direct_attributes'],pn['direct_attributes']):
                require(na['ordinal']==pa['ordinal'] and na['style']==pa['style'] and na['kind']==pa['kind'] and node_map[na['node']]==pa['node'],'source/native direct attribute membership/style/order conflict')
                attributes.append({'native_owner_node':i,'physical_owner_node':pn['ordinal'],'native_attribute_node':na['node'],'physical_attribute_node':pa['node'],'ordinal':pa['ordinal'],'style':pa['style'],'kind':pa['kind'],'relationship':'direct' if i==ni else 'nested'})
        physical_nodes=set(self.descendants(pr,pi));native_nodes=set(self.descendants(nr,ni))
        trivia=[t for t in pr['tokens'] if t['trivia'] and t['parent'] in physical_nodes]
        return {'native':reference,'physical':{'root':pri,'node':pi},'file':file,'physical_range':pr['nodes'][pi]['range'],'tokens':edges,'subnodes':nodes,'source_attributes':attributes,'physical_trivia':trivia,'native_trivia':[t for t in nr['tokens'] if t['trivia'] and t['parent'] in native_nodes]}
