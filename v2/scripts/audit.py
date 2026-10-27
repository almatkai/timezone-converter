#!/usr/bin/env python3
"""Independent pinned-object checks and complete unstaged candidate inventory."""
import hashlib,json,pathlib,subprocess,sys
r=pathlib.Path(__file__).resolve().parents[1];repo=r.parent
own='784f7d1c1743ee6228f26a2384f925475cfb11e9';host='ea100cee3f69711de12485ba061950c5fc65258f';sdk='e955911f0bf5b558f9d3eb5183df0adc95c22af2'
def git(*a,cwd=repo):return subprocess.check_output(['git',*a],cwd=cwd)
def digest(b):return hashlib.sha256(b).hexdigest()
assert git('rev-parse','--show-toplevel').decode().strip()==str(repo)
assert git('branch','--show-current').decode().strip()=='feat/native-v2-rust'
assert git('rev-parse','HEAD').decode().strip()==own
assert not git('diff','--cached','--name-only'), 'Index must remain empty'
original=git('ls-tree','-r','--name-only',own).decode().splitlines();v1={}
for name in original:
 b=git('show',own+':'+name);current=(repo/name).read_bytes()
 if name=='README.md':assert current==b+b'\nStandalone native Rust V2: [implementation and verification](v2/README.md).\n'
 else:assert current==b,name
 v1[name]=digest(b)
assets=json.loads((r/'legal/upstream-assets.json').read_bytes())
for f in assets['files']:
 if f['path'].startswith('assets/'):
  p=pathlib.Path(f['path']);assert digest((repo/p).read_bytes())==f['sha256'];assert digest((r/'resources'/p.relative_to('assets')).read_bytes())==f['sha256'];assert digest((r/'qa/upstream'/p).read_bytes())==f['sha256']
pins=json.loads((r/'integration/host-pins.json').read_bytes())
for name,h in pins['files'].items():
 b=(r/'tests/host'/name).read_bytes();assert digest(b)==h
 if '--independent' in sys.argv:assert git('show',host+':'+name,cwd=repo.parent.parent/'Tezbar')==b
if '--independent' in sys.argv:
 for name in ['LICENSE','NOTICE']:
  assert git('show',sdk+':'+name,cwd=repo.parent/'tezbar-extension-sdk-rust')==(r/'legal'/('SDK-'+name)).read_bytes()
lock=(r/'Cargo.lock').read_text();assert 'git+https://github.com/almatkai/tezbar-extension-sdk-rust.git?rev='+sdk+'#'+sdk in lock;assert 'path+' not in lock
allnames=set(git('ls-files','--cached','--others','--exclude-standard','-z').decode().strip('\0').split('\0'))
visible=sorted(p for p in allnames if (p=='README.md' or p.startswith('v2/')) and p!='v2/REVIEW-INVENTORY.json')
assert not any(any(k in pathlib.Path(p).parts for k in ['qa','target','node_modules','dist']) or p.endswith('.zip') for p in visible)
fixture_names=set(str(p.relative_to(repo)) for p in (r/'tests/host').rglob('*') if p.is_file())
assert fixture_names.issubset(set(visible)), 'A pinned host fixture is accidentally ignored'
review={'base':own,'branch':'feat/native-v2-rust','index':'matches HEAD (no staged changes)','files':{p:digest((repo/p).read_bytes()) for p in visible},'pinnedHostFixtureCount':len(fixture_names),'ignoredGeneratedFixtures':'qa/fixtures and qa/patched-host are ignored; immutable test fixtures under tests/host are visible'}
(r/'REVIEW-INVENTORY.json').write_text(json.dumps(review,indent=2)+'\n')
report={'originalV1FilesUnchangedExceptReadmeLink':len(original)-1,'originalAssetsCompared':sum(x['path'].startswith('assets/') for x in assets['files']),'iconSha256':assets['files'][0]['sha256'],'reviewCandidateFilesExcludingInventory':len(visible),'pinnedHostFixturesVisible':len(fixture_names),'head':own,'branch':review['branch'],'index':'matches HEAD (no staged changes)'}
(r/'qa/audit-report.json').write_text(json.dumps(report,indent=2)+'\n');(r/'qa/v1-pin-hashes.json').write_text(json.dumps(v1,indent=2)+'\n');print(json.dumps(report))
