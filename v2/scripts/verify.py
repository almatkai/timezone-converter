#!/usr/bin/env python3
"""Reproduce checked-in native and immutable host-fixture checks in owned paths."""
import pathlib,subprocess,json
r=pathlib.Path(__file__).resolve().parents[1];q=r/'qa';q.mkdir(exist_ok=True);results=[]
def run(name,args,optional=False):
 with (q/(name+'.log')).open('w') as f:
  p=subprocess.run(['python3',str(r/'scripts/owned.py'),*args],stdout=f,stderr=subprocess.STDOUT)
 results.append({'check':name,'exit':p.returncode,'optional':optional})
 if p.returncode and not optional:raise SystemExit('Failed '+name+'; see qa/'+name+'.log')
for t in ['stable','1.85.0']:
 run('fmt-'+t,['cargo','+'+t,'fmt','--','--check'])
 run('tests-'+t,['cargo','+'+t,'test','--locked'])
 run('clippy-'+t,['cargo','+'+t,'clippy','--locked','--all-targets','--','-D','warnings'])
run('v1-test',['npm','--prefix','qa/v1','test']);run('v1-typecheck',['npm','--prefix','qa/v1','run','typecheck'])
run('oracle',['node','scripts/oracle.cjs'])
# Oracle output is deterministic and reviewed; rerun native tests against that output.
run('oracle-native',['cargo','+1.85.0','test','--locked','--test','time'])
run('production-process',['python3','scripts/process-test.py'])
run('actual-renderer-schema',['node','scripts/host-tests.cjs'])
run('windows-check',['cargo','+1.85.0','check','--locked','--all-targets','--target','x86_64-pc-windows-gnu'],True)
run('windows-clippy',['cargo','+1.85.0','clippy','--locked','--all-targets','--target','x86_64-pc-windows-gnu','--','-D','warnings'],True)
(q/'verification-results.json').write_text(json.dumps(results,indent=2)+'\n');print(json.dumps(results))
