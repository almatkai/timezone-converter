#!/usr/bin/env python3
"""Fetch only public immutable oracle/provenance inputs into owned QA paths."""
import concurrent.futures,hashlib,io,json,pathlib,tarfile,urllib.error,urllib.request
r=pathlib.Path(__file__).resolve().parents[1]
def get(url):return urllib.request.urlopen(url,timeout=30).read()
def checked(url,digest,p):
 b=get(url);assert hashlib.sha256(b).hexdigest()==digest,url;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(b)
assets=json.loads((r/'legal/upstream-assets.json').read_bytes())
def asset(f):
 p=r/'qa/upstream'/f['path']
 if 'sha256' in f:checked(f['url'],f['sha256'],p)
 else:
  try:get(f['url'])
  except urllib.error.HTTPError as e:assert e.code==f['http']
  else:raise AssertionError('Expected absent per-extension license')
with concurrent.futures.ThreadPoolExecutor(max_workers=6) as e:list(e.map(asset,assets['files']))
cldr=json.loads((r/'legal/cldr-pin.json').read_bytes())
for f,h in cldr['inputs'].items():
 if f!='pin.json':checked(f'https://raw.githubusercontent.com/unicode-org/cldr/{cldr["commit"]}/{f}',h,r/'qa/cldr'/f)
pins=json.loads((r/'legal/source-pins.json').read_bytes());iana=pins['iana'];archive=r/'qa/tzdata2026a.tar.gz';checked(iana['url'],iana['archiveSha256'],archive)
with tarfile.open(fileobj=io.BytesIO(archive.read_bytes())) as t:
 for name in ['backward','LICENSE','version']:(r/'qa'/name).write_bytes(t.extractfile(name).read())
assert hashlib.sha256((r/'qa/backward').read_bytes()).hexdigest()==iana['backwardSha256']
print('Exact public upstream artwork / CLDR / IANA inputs verified')
