#!/usr/bin/env python3
"""Run a subprocess with all mutable homes and caches inside this V2 clone."""
import os, pathlib, re, subprocess, sys
root=pathlib.Path(__file__).resolve().parents[1]
args=sys.argv[1:]
qa=root/'qa'
if args[:1]==['--lane']:
 if len(args)<3 or not re.fullmatch('[a-z0-9-]+',args[1]):raise SystemExit('Invalid owned lane')
 qa=qa/'lanes'/args[1];args=args[2:]
env=os.environ.copy()
# Keep access to existing installed toolchains without creating/installing one.
env['RUSTUP_HOME']=os.environ.get('RUSTUP_HOME',str(pathlib.Path.home()/'.rustup'))
for key,sub in {'HOME':'home','TMPDIR':'tmp','CARGO_HOME':'cargo','CARGO_TARGET_DIR':'target','npm_config_cache':'npm','XDG_CACHE_HOME':'cache','XDG_CONFIG_HOME':'config','XDG_DATA_HOME':'data'}.items():
 p=qa/sub;p.mkdir(parents=True,exist_ok=True);env[key]=str(p)
env['CARGO_NET_GIT_FETCH_WITH_CLI']='true'
sys.exit(subprocess.call(args,env=env,cwd=root))
