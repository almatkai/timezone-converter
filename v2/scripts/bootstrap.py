#!/usr/bin/env python3
"""Create disposable exact V1/tooling installs under qa; never modify V1 or host."""
import io,pathlib,shutil,subprocess,tarfile
r=pathlib.Path(__file__).resolve().parents[1]
base=r/'qa/v1';base.mkdir(parents=True,exist_ok=True)
rev='784f7d1c1743ee6228f26a2384f925475cfb11e9'
b=subprocess.check_output(['git','archive',rev],cwd=r.parent)
with tarfile.open(fileobj=io.BytesIO(b)) as t:
 for m in t:
  if m.isfile():
   p=base/m.name;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(t.extractfile(m).read())
tools=r/'qa/tools';tools.mkdir(parents=True,exist_ok=True)
for n in ['package.json','package-lock.json']:shutil.copyfile(r/'tests/tooling'/n,tools/n)
for name in ['v1','tools']:
 subprocess.run(['python3',str(r/'scripts/owned.py'),'npm','ci','--ignore-scripts','--prefix','qa/'+name],check=True)

subprocess.run(['python3',str(r/'scripts/owned.py'),'python3','scripts/inputs.py'],check=True)
