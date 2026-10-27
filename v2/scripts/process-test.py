#!/usr/bin/env python3
"""Real production executable / pinned SDK JSONL pipes. Synthetic data only."""
import base64,hashlib,json,os,pathlib,queue,subprocess,sys,tempfile,threading,uuid
root=pathlib.Path(__file__).resolve().parents[1]
binary=(root/json.loads((root/'qa/package/report.json').read_text())['extractedDir']/'bin/timezone-converter') if sys.argv[1:]==['--packaged'] else (pathlib.Path(sys.argv[1]).resolve() if len(sys.argv)>1 else root/'qa/target/debug/timezone-converter')
count=0
class Process:
 def __init__(self):
  self.tmp=tempfile.TemporaryDirectory(dir=root/'qa/tmp');self.dir=pathlib.Path(self.tmp.name);self.i=0;self.q=queue.Queue()
  self.p=subprocess.Popen([str(binary)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=os.environ.copy())
  def reader():
   for line in self.p.stdout:self.q.put(json.loads(line))
  self.thread=threading.Thread(target=reader,daemon=True);self.thread.start()
  assert self.q.get(timeout=10)['type']=='hello'
  assert self.req('initialize',{'protocol':1,'extensionId':'com.tezbar.native.timezone-converter','platform':'aarch64-apple-darwin','packageDir':str(binary.parent.parent),'dataDir':str(self.dir/'data'),'tempDir':str(self.dir/'temp'),'grantedCapabilities':[]})['ready']
 def send(self,x):self.p.stdin.write(json.dumps(x).encode()+b'\n');self.p.stdin.flush()
 def req(self,method,params):
  global count
  self.i+=1;ident='h:'+str(uuid.uuid4());self.send({'type':'request','id':ident,'method':method,'params':params})
  r=self.q.get(timeout=20);assert r.get('id')==ident,r;assert r['type']=='response',r;count+=1
  return r.get('result',{'error':r.get('error')})
 def open(self,s,hide=True,sort='manually'):return self.req('session.open',{'sessionId':s,'commandName':'index','preferences':{'sortTimezones':sort,'hideWorldMap':hide},'searchText':None})
 def call(self,s,op,p={}):return self.req('session.call',{'sessionId':s,'operation':op,'payload':p})
 def act(self,s,v,title,values=None):
  ident=next(a['id'] for a in v['actions'] if a['title']==title);return self.call(s,'extension:invoke-action',{'actionId':ident,'formValues':values})
 def close(self,eof=False):
  if not eof:assert self.req('shutdown',{}).get('ok',True)
  self.p.stdin.close();assert self.p.wait(timeout=10)==0,self.p.stderr.read();self.thread.join(1);self.tmp.cleanup()
for arg in ['--protocol-info','--self-test']:
 info=json.loads(subprocess.check_output([str(binary),arg]));assert info['commands']==['index'];assert info['sdkVersion']=='0.1.1'
p=Process();assert 'error' in p.req('session.open',{'sessionId':'missing-prefs','commandName':'index','preferences':{},'searchText':None});a=p.open('a');b=p.open('b');assert a['root']['type']=='List'
form=p.act('a',a,'Set Custom Time');old=form['actions'][0]['id']
invalid=p.act('a',form,'Confirm',{'customTime':'B invalid','timezone':'UTC'});assert 'Failed' in invalid['message'];assert invalid['root']['children'][0]['props'].get('value') is None
assert p.call('a','extension:invoke-action',{'actionId':old})['mode']=='unchanged'
assert p.call('b','extension:invoke-action',{'actionId':invalid['actions'][0]['id']})['mode']=='unchanged'
valid=p.act('a',invalid,'Confirm',{'customTime':'2024-01-15T12:34:56.789','timezone':'UTC'});assert valid['root']['type']=='List'
add=p.act('a',valid,'Add Timezones');assert sum(len(s['children']) for s in add['root']['children'])==419
utc=next(x['title'] for x in add['actions'] if x['title']=='Add UTC (GMT+0)');added=p.act('a',add,utc)
assert p.call('a','extension:invoke-action',{'actionId':next(x['id'] for x in add['actions'] if x['title']==utc)})['mode']=='unchanged'
search=p.call('a','extension:search-text-changed',{'searchText':'Tokyo'});assert search['root']['props'].get('searchText') is None
second=next(x['title'] for x in search['actions'] if x['title'].startswith('Add Asia - Tokyo ('));added=p.act('a',search,second)
# Stale session cannot overwrite the successful write.
badd=p.act('b',b,'Add Timezones');conflict=p.act('b',badd,'Add UTC (GMT+0)');assert 'another session' in conflict['message'];bhome=p.act('b',conflict,'Done');breload=p.act('b',bhome,'Reload Saved Timezones');assert len(breload['root']['children'])==2
home=p.act('a',added,'Done');order=p.act('a',home,'Reorder Timezones');order=p.act('a',order,'Move Down');assert order['root']['children'][0]['props']['id']=='Asia/Tokyo'
confirm=p.act('a',order,'Remove Timezone');order=p.act('a',confirm,'Cancel');assert len(order['root']['children'])==2
confirm=p.act('a',order,'Remove Timezone');order=p.act('a',confirm,'Delete');assert len(order['root']['children'])==1
home=p.act('a',order,'Done');confirm=p.act('a',home,'Remove Added Timezones');home=p.act('a',confirm,'Cancel');assert home['root']['type']=='List'
confirm=p.act('a',home,'Remove Added Timezones');home=p.act('a',confirm,'Remove All');assert home['root']['children'][0]['type']=='List.EmptyView'
assert p.call('a','extension:refresh-session')['mode']=='unchanged';assert 'error' in p.call('a','extension:invented')
# Per-process scoped saved zones survive a new session and do not migrate V1.
add=p.act('a',home,'Add Timezones');add=p.act('a',add,'Add UTC (GMT+0)');p.req('session.dispose',{'sessionId':'a'})
v=p.open('restored',False,'alphabetically');assert v['root']['type']=='Detail';assert v['root']['metadata']['children'][0]['props']['title']=='UTC (UTC)'
markdown=v['root']['props']['markdown'];marker='data:image/png;base64,'
if marker in markdown:
 raw=base64.b64decode(markdown.split(marker)[1].split(')')[0]);assert raw in [f.read_bytes() for f in (root/'resources/timezones').glob('*.png')]
# Save renderer input fixtures from production; they contain only synthetic state.
(root/'qa/renderer-input.json').write_text(json.dumps({'home':v,'form':form,'invalid':invalid,'add':add,'order':order}))
# Corruption is explicit and does not mutate bytes, including retry.
storage=p.dir/'data/selected-timezones.json';storage.write_bytes(b'{broken')
corrupt=p.open('corrupt');assert 'Corrupt' in corrupt['message'];corrupt=p.act('corrupt',corrupt,'Retry Loading Saved Timezones');assert storage.read_bytes()==b'{broken'
# Real write failure after loading: change a formerly missing data directory into a file.
p.close()
p=Process();v=p.open('writefail');add=p.act('writefail',v,'Add Timezones');data=p.dir/'data';data.write_bytes(b'preserve previous bytes')
failed=p.act('writefail',add,'Add UTC (GMT+0)');assert 'Failed' in failed['message'];assert data.read_bytes()==b'preserve previous bytes';assert sum(len(s['children']) for s in failed['root']['children'])==419
p.close(eof=True)
# Capacity, disposal and live opens use the production SDK, without a handwritten dispatcher.
p=Process()
for i in range(128):assert p.open(f'live-{i}')['root']['type']=='List'
assert 'error' in p.open('over-capacity')
for i in range(128):p.req('session.dispose',{'sessionId':f'live-{i}'})
assert p.open('after-dispose')['root']['type']=='List';p.close()
# Framing failures must stop promptly and must never write storage.
for frame in [b'{invalid}\n',b'\xff\n',b'x'*(16*1024*1024+1)+b'\n',b'{"type":"bogus"}\n']:
 x=subprocess.Popen([str(binary)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE);assert json.loads(x.stdout.readline())['type']=='hello'
 try:x.stdin.write(frame);x.stdin.close()
 except BrokenPipeError:pass
 assert x.wait(timeout=15)!=0
print(json.dumps({'productionResponsesChecked':count,'processSmoke':'passed','frames':4,'nativeAcceptance':'not-run'}))
