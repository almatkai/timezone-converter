#!/usr/bin/env python3
"""Capture authentic notices from exact locked archives, never synthesize third-party grants."""
import hashlib,json,pathlib,shutil,subprocess,tomllib
r=pathlib.Path(__file__).resolve().parents[1];lock=tomllib.loads((r/'Cargo.lock').read_text());dest=r/'legal/dependencies';dest.mkdir(parents=True,exist_ok=True)
sources=list((r/'qa/cargo/registry/src').glob('*/*'))
records=[]
for p in lock['package']:
 if p.get('source','').startswith('registry+'):
  folder=next(f for f in sources if f.name==p['name']+'-'+p['version']);manifest=tomllib.loads((folder/'Cargo.toml').read_text())['package'];files={}
  for f in folder.rglob('*'):
   if f.is_file() and any(k in f.name.upper() for k in ['LICENSE','LICENCE','COPYING','NOTICE','COPYRIGHT','AUTHORS']):
    rel=f.relative_to(folder);out=dest/(p['name']+'-'+p['version'])/rel;out.parent.mkdir(parents=True,exist_ok=True);out.write_bytes(f.read_bytes());files[str(rel)]=hashlib.sha256(f.read_bytes()).hexdigest()
  records.append({'name':p['name'],'version':p['version'],'checksum':p['checksum'],'licenseDeclaration':manifest.get('license'),'rustVersion':manifest.get('rust-version'),'repository':manifest.get('repository'),'notices':files,'evidenceGap':None if files else 'No grant/notice text in the locked archive; declaration alone is not a reproduced grant.'})
(r/'legal/dependencies.json').write_text(json.dumps(records,indent=2)+'\n')
print('locked crates',len(records),'notice files',sum(len(x['notices']) for x in records),'gaps',[x['name'] for x in records if x['evidenceGap']])
