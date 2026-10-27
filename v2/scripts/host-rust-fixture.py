#!/usr/bin/env python3
"""Extract actual pinned pure producer/validator code, with only outer dependency stubs."""
from pathlib import Path
r=Path(__file__).resolve().parents[1];base=r/'tests/host/src-tauri/src/native_core/extensions'
i=(base/'process_install.rs').read_text();l=(base/'launcher.rs').read_text();n=(base/'native.rs').read_text()
def section(s,start,end):return s[s.index(start):s.index(end)]
constants=section(i,'const CAPABILITY_GROUPS:','// Shared by installer')
validation=section(i,'pub(super) fn component','pub(super) fn artifact')
validation=validation.replace('fn digest(s: &str) -> bool {\n    s.len() == 64\n        && s.bytes()\n            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())\n}\n','') # unrelated private helper is not called by the extracted validator.
launcher=section(l,'pub(super) fn descriptors','pub(super) fn selected')
annotate=section(n,'fn annotate','struct LaunchCleanup')
line=next(s for s in n.splitlines() if 'p.call("session.open",json!' in s)
expr=line[line.index('json!('):line.index(',deadline(&entry)')]
s='''// Generated from immutable host ea100cee3f69711de12485ba061950c5fc65258f; run scripts/host-rust-fixture.py.
// Function bodies are original; outer imports/types are test-only dependency stubs.
// Narrow lint allowances preserve the actual pinned host functions unchanged.
#![allow(dead_code, clippy::redundant_closure, clippy::nonminimal_bool)]
use serde_json::{json,Value};
use std::{collections::HashSet,path::PathBuf};
type CoreResult=Result<Value,String>;
pub mod process_install {use super::*;
'''+constants+validation+'''}
pub mod launcher {use super::*;
'''+launcher+'''}
struct Process {generation:&'static str}
struct Entry {canonical:&'static str,command:&'static str,process:Process,policy:Value}
'''+annotate+'''
pub fn validate_fixture(v:&Value)->Result<(),String>{process_install::validate(v)}
pub fn annotate_fixture(value:Value)->CoreResult{annotate(&Entry{canonical:"com.tezbar.native.timezone-converter",command:"index",process:Process{generation:"fixture-generation"},policy:json!({"refreshMs":null})},"fixture-session",value)}
pub fn open_params(body:Value)->Value{let id="fixture-session";let command="index";let preferences=json!({"sortTimezones":"manually"});
'''+expr+'''
}
'''
p=r/'tests/contract/pinned.rs';p.parent.mkdir(parents=True,exist_ok=True);p.write_text(s)
