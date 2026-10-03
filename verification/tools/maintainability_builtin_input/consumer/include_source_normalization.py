"""Lossless official SourceFile maps and independently authenticated raw bytes."""
import hashlib
from pathlib import Path
from include_source_authority import require


def path(root,source):
    p=Path(source)
    return str((p if p.is_absolute() else Path(root)/p).resolve())


def original_position(record,offset,api):
    require(0<=offset<=record['normalized_source_len'],'normalized source position outside original SourceFile',api)
    difference=0
    for point in record['official_normalization']:
        if point['position']>offset:break
        difference=point['difference']
    return offset+difference


def authenticate_source(record,inputs,api):
    filename=path(inputs['root'],record['file']);p=Path(filename)
    require(filename in inputs['files'] and p.is_file(),'required original SourceFile lacks raw input authority: '+filename,api)
    raw=p.read_bytes()
    require(api.digest(raw)==inputs['files'][filename],'original physical SourceFile raw input drift',api)
    algorithm,expected=record['original_source_hash'].split('=',1)
    require(algorithm in {'md5','sha1','sha256'},'unsupported actual compiler original source hash algorithm',api)
    require(hashlib.new(algorithm,raw).hexdigest()==expected,'compiler original SourceFile/raw byte hash conflict',api)
    require(record['original_source_len']==len(raw),'compiler original SourceFile byte length conflict',api)
    previous=-1
    for point in record['official_normalization']:
        require(set(point)=={'position','difference','original_position'} and previous<point['position']<=record['normalized_source_len'],'official normalization inventory order/field conflict',api)
        require(point['original_position']==point['position']+point['difference'],'official original-relative byte position witness conflict',api)
        previous=point['position']
    require(record['official_endpoints']==[original_position(record,0,api),original_position(record,record['normalized_source_len'],api)] and record['official_endpoints'][1]==len(raw),'official complete normalization length/boundary conflict',api)
    # The official API maps boundaries. A normalized LF's complete interval
    # maps to both raw CRLF bytes; its starting boundary points before the CR.
    # Authenticate each full interval instead of treating a boundary as a
    # one-byte origin and accidentally discarding the LF.
    normalized=bytearray();omitted=[]
    first=original_position(record,0,api)
    require(first==0 or (first==3 and raw[:first]==b'\xef\xbb\xbf'),'unsupported initial official source normalization',api)
    if first:omitted.append((0,first,0))
    for offset in range(record['normalized_source_len']):
        start=original_position(record,offset,api);end=original_position(record,offset+1,api)
        require(0<=start<end<=len(raw),'non-monotonic/out-of-bounds official byte interval',api)
        interval=raw[start:end]
        if len(interval)==1:normalized.extend(interval)
        else:
            require(interval==b'\r\n','unsupported official source normalization disposition',api)
            normalized.append(10);omitted.append((start,start+1,offset))
    text=bytes(normalized).decode('utf-8')
    if record['normalized_text'] is not None:require(text==record['normalized_text'],'original compiler normalized source text conflict',api)
    return {'file':filename,'raw_sha256':api.digest(raw),'normalization_authority':record,'omitted_bytes':[{'range':[s,e],'normalized_position':n} for s,e,n in omitted]}


def source_for(span,records,inputs,api):
    require(span['kind']=='original' and span['hygiene']=='#0','required declaration lacks exact original compiler source authority',api)
    matches=[s for s in records if s['start_pos']==span['source_file_start_pos']]
    require(len(matches)==1,'missing/ambiguous actual owning compiler SourceFile',api)
    record=matches[0]
    require(record['source_hash']==span['source_hash'] and record['original_source_hash']==span['original_source_hash'] and path(inputs['root'],record['file'])==path(inputs['root'],span['file']),'span/actual SourceFile identity conflict',api)
    for normalized,original in zip(span['normalized_interval'],(span['start'],span['end'])):
        require(original_position(record,normalized,api)==original,'actual compiler span original-relative byte bridge conflict',api)
    filename=path(inputs['root'],span['file']);raw=Path(filename).read_bytes()
    require(0<=span['start']<=span['end']<=len(raw),'original compiler declaration/use range conflict',api)
    if span['snippet'] is not None:
        normalized=raw[span['start']:span['end']].decode('utf-8').replace('\r\n','\n')
        require(normalized==span['snippet'],'actual compiler source snippet/byte bridge conflict',api)
    return filename
