#!/usr/bin/env python3
"""Deterministic development ZIP. No host checkout, installation or publication."""
import hashlib,json,pathlib,stat,sys,zipfile
r=pathlib.Path(__file__).resolve().parents[1]
binary=pathlib.Path(sys.argv[1]).resolve() if len(sys.argv)>1 else r/'qa/lanes/final-nonce/target/release/timezone-converter'
out=r/'qa/package';out.mkdir(parents=True,exist_ok=True)
files={'bin/timezone-converter':binary.read_bytes(),'LICENSE':(r/'LICENSE').read_bytes(),'NOTICE':(r/'NOTICE').read_bytes()}
for folder in ['resources','legal']:
 for f in (r/folder).rglob('*'):
  if f.is_file():files[str(f.relative_to(r))]=f.read_bytes()
m=json.loads((r/'extension.json').read_bytes());m['files']={p:hashlib.sha256(b).hexdigest() for p,b in sorted(files.items())}
files['extension.json']=(json.dumps(m,indent=2,sort_keys=True)+'\n').encode()
def archive(path):
 with zipfile.ZipFile(path,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=9) as z:
  for n,b in sorted(files.items()):
   i=zipfile.ZipInfo(n,(1980,1,1,0,0,0));i.create_system=3;i.compress_type=zipfile.ZIP_DEFLATED;i.external_attr=((stat.S_IFREG| (0o755 if n.startswith('bin/') else 0o644))<<16);z.writestr(i,b)
zip1=out/'timezone-converter-aarch64-apple-darwin-dev.zip';zip2=out/'repeated.zip';archive(zip1);archive(zip2);assert zip1.read_bytes()==zip2.read_bytes()
extracted=out/('extracted-'+hashlib.sha256(zip1.read_bytes()).hexdigest()[:16]);extracted.mkdir(exist_ok=True)
# Closed inventory extraction, no arbitrary user archives.
with zipfile.ZipFile(zip1) as z:
 assert set(z.namelist())==set(files)
 for info in z.infolist():
  p=extracted/info.filename;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(z.read(info));p.chmod((info.external_attr>>16)&0o777)
actual={str(p.relative_to(extracted)) for p in extracted.rglob('*') if p.is_file()};assert actual==set(files)
for n,b in files.items():assert (extracted/n).read_bytes()==b
for n,digest in m['files'].items():assert hashlib.sha256((extracted/n).read_bytes()).hexdigest()==digest
assert stat.S_IMODE((extracted/'bin/timezone-converter').stat().st_mode)==0o755
report={'extractedDir':str(extracted.relative_to(r)),'developmentOnly':True,'artifacts':m['artifacts'],'nativeAcceptance':'not-run','zipSha256':hashlib.sha256(zip1.read_bytes()).hexdigest(),'executableSha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'files':m['files'],'closedInventoryFilesIncludingManifest':len(files)}
(out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k!='files'}))
