/* QA only. Original TSX/helper + locked Luxon run in a closed VM. No module/OS callbacks. */
const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict'),path=require('node:path');
const base=path.resolve(__dirname,'..'),v1=path.join(base,'qa/v1');
const ts=require(path.join(v1,'node_modules/typescript'));
assert.equal(process.versions.node,'26.7.0','Use the recorded installed Node oracle runtime');assert.equal(process.versions.icu,'78.3');assert.equal(process.versions.tz,'2026a');
let clock=Date.parse('2024-01-15T12:34:56.789Z');
const ctx=vm.createContext({});
vm.runInContext(`globalThis.__clock=${clock};globalThis.Date=class extends Date { constructor(...args){super(...(args.length?args:[globalThis.__clock]))} static now(){return globalThis.__clock} };`,ctx);
vm.runInContext(fs.readFileSync(path.join(v1,'node_modules/luxon/build/global/luxon.js'),'utf8'),ctx,{timeout:2000});
// Lock local zone and locale before executing source. These are metadata, not OS changes.
vm.runInContext(`luxon.Settings.defaultZone='America/Los_Angeles'; luxon.Settings.defaultLocale='en-US';`,ctx);
const L=ctx.luxon;
let states=[],cursor=0,cached=[],prefs={sortTimezones:'manually',hideWorldMap:false},pushed,popped=0,alert,accept=true;
const jsx=(type,props)=>({type,props:props||{}});
const tag=name=>Object.assign(name==='Fragment'?(()=>{}):function(){},{tag:name});
const component=name=>new Proxy(tag(name),{get:(o,k)=>k==='tag'?name:component(name+'.'+String(k))});
const react={Fragment:component('Fragment'),createContext:value=>({value,Provider:component('Provider')}),useContext:c=>c.value,
 useState:initial=>{let i=cursor++;if(!(i in states))states[i]=typeof initial==='function'?initial():initial;return [states[i],v=>states[i]=typeof v==='function'?v(states[i]):v]},
 useMemo:f=>f(),useCallback:f=>f};
const api={...Object.fromEntries(['Action','ActionPanel','Alert','Color','Detail','Form','Icon','List'].map(n=>[n,component(n)])),
 getPreferenceValues:()=>prefs,useNavigation:()=>({push:x=>pushed=x,pop:()=>popped++}),confirmAlert:async a=>{alert=a;if(accept&&a.primaryAction?.onAction)a.primaryAction.onAction();return accept}};
const modules={'@raycast/api':api,'@raycast/utils':{useCachedState:()=>[cached,v=>cached=v]},luxon:L,react,'react/jsx-runtime':{jsx,jsxs:jsx,Fragment:react.Fragment}};
ctx.require=n=>{if(!(n in modules))throw Error('Denied module '+n);return modules[n]};
ctx.exports={};
function evaluate(file,extra=''){const s=fs.readFileSync(path.join(v1,file),'utf8')+extra;const code=ts.transpileModule(s,{compilerOptions:{module:ts.ModuleKind.CommonJS,jsx:ts.JsxEmit.ReactJSX,target:ts.ScriptTarget.ES2022}}).outputText;ctx.exports={};vm.runInContext("(function(){"+code+"})()",ctx,{timeout:3000});return ctx.exports}
const source=evaluate('src/index.tsx','\nexports.qa={Timezones,CustomTime,OrderTimezones,TimezoneCotext,ALL_TIMEZONES,markdownFn,getTimezoneOffsetString,formatZoneName};').qa;
const helper=evaluate('src/lib/timezone.ts');
function render(f,p={}){cursor=0;const t=f(p);if(t.type?.tag==='Provider')source.TimezoneCotext.value=t.props.value;return t}
function walk(t,pred,out=[]){if(!t)return out;if(Array.isArray(t)){t.forEach(n=>walk(n,pred,out));return out}if(typeof t==='object'){if(pred(t))out.push(t);walk(t.props?.children,pred,out);if(t.props?.actions)walk(t.props.actions,pred,out)}return out}
const actions=t=>walk(t,n=>n.type?.tag==='Action'||n.type?.tag==='Action.Push');
const action=(t,title)=>{const a=actions(t).find(a=>a.props.title===title);assert.ok(a,'missing '+title);return a.props};
(async()=>{
 const zones=Array.from(source.ALL_TIMEZONES);assert.deepEqual(zones,Array.from(helper.getAllTimezones()));
 assert.equal(zones[0],'UTC');assert.ok(zones.length>400);
 const cases=[];
 for(const iso of ['2024-01-15T12:34:56.789Z','2024-07-15T00:00:00Z','1880-01-01T00:00:00Z','1900-01-01T00:00:00Z','1945-07-01T00:00:00Z','1970-01-01T00:00:00Z','2101-07-01T00:00:00Z'])for(const zone of zones){const d=L.DateTime.fromISO(iso,{zone:'UTC'}).setZone(zone);cases.push({ms:d.toMillis(),zone,ff:d.toFormat('ff'),short:d.toFormat('ZZZZ'),offsetSeconds:Math.round(d.offset*60)});}
 console.log([1880,1900,1945,1970].map(year=>[year,...["Europe/London","America/New_York","Antarctica/Troll"].map(zone=>L.DateTime.fromObject({year,month:1,day:1},{zone}).toFormat("ZZZZ"))]));
 const shorts={};for(const zone of zones){shorts[zone]=['2024-01-15T00:00:00Z','2024-07-15T00:00:00Z'].map(iso=>{const d=L.DateTime.fromISO(iso,{zone:'UTC'}).setZone(zone);return [d.offset*60,d.toFormat('ZZZZ')]})}
 fs.writeFileSync(path.join(base,'resources/catalog.json'),JSON.stringify(zones,null,2)+'\n');
 fs.writeFileSync(path.join(base,'tests/short-names.json'),JSON.stringify(shorts,null,2)+'\n');
 const custom=[];
 for(const local of ['UTC','America/Los_Angeles','America/New_York','Asia/Katmandu','Australia/Lord_Howe','Pacific/Chatham'])for(const zone of ['UTC','America/New_York','Asia/Katmandu','Australia/Lord_Howe','Pacific/Chatham','Pacific/Apia'])for(const iso of ['2024-03-10T09:30:00Z','2024-03-10T10:30:00Z','2024-11-03T08:30:00Z','2024-11-03T09:30:00Z','2024-07-01T12:34:56.789Z','2024-03-10T02:30:00Z','2024-10-06T02:15:00Z','2011-12-30T12:00:00Z']){
  L.Settings.defaultZone=local;
  states=[];source.TimezoneCotext.value={customTime:L.DateTime.fromISO('2024-01-15T12:34:56.789Z')};let selected;let t=render(source.CustomTime,{onSelect:d=>selected=d});
  walk(t,n=>n.type?.tag==='Form.DatePicker')[0].props.onChange(new ctx.Date(iso));t=render(source.CustomTime,{onSelect:d=>selected=d});
  walk(t,n=>n.type?.tag==='Form.Dropdown')[0].props.onChange(zone);t=render(source.CustomTime,{onSelect:d=>selected=d});action(t,'Confirm').onAction();
  custom.push({pickerMs:Date.parse(iso),local,zone,ms:selected.toMillis()});
 }
 L.Settings.defaultZone='America/Los_Angeles';
 states=[];cached=['Asia/Tokyo','UTC','America/New_York'];let t=render(source.Timezones);assert.equal(t.props.children.type.tag,'Detail');
 // Snapshot survives rerenders; reset is a fresh clock read (checked below with Settings.now).
 const snapshot=states[1].toMillis();ctx.__clock=clock+123456;t=render(source.Timezones);assert.equal(states[1].toMillis(),snapshot);
 action(t,'Set Custom Time').onAction();assert.equal(pushed.type,source.CustomTime);
 pushed.props.onSelect(L.DateTime.fromMillis(clock-86400000));t=render(source.Timezones);action(t,'Clear Custom Time').onAction();assert.equal(states[1].toMillis(),clock+123456);assert.equal(states[0],false);ctx.__clock=clock;
 prefs.hideWorldMap=true;t=render(source.Timezones);assert.equal(t.props.children.type.tag,'List');
 prefs.sortTimezones='alphabetically';render(source.Timezones);assert.deepEqual(cached,['America/New_York','Asia/Tokyo','UTC']); // in-place mutation is observable
 prefs.sortTimezones='manually';states=[];let ordered=['Asia/Tokyo','UTC','America/New_York'];let order=render(source.OrderTimezones,{timezones:ordered,onSort:v=>ordered=v});
 const row=walk(order,n=>n.type?.tag==='List.Item')[1];action(row,'Move Up').onAction();assert.deepEqual(Array.from(ordered),['UTC','Asia/Tokyo','America/New_York']);
 order=render(source.OrderTimezones,{timezones:ordered,onSort:v=>ordered=v});const row0=walk(order,n=>n.type?.tag==='List.Item')[0];action(row0,'Move Down').onAction();assert.equal(ordered[1],'UTC');
 order=render(source.OrderTimezones,{timezones:ordered,onSort:v=>ordered=v});accept=false;await action(walk(order,n=>n.type?.tag==='List.Item')[0],'Remove Timezone').onAction();assert.equal(ordered.length,3);accept=true;await action(walk(order,n=>n.type?.tag==='List.Item')[0],'Remove Timezone').onAction();assert.equal(ordered.length,2);
 states=[];cached=['UTC'];t=render(source.Timezones);accept=false;await action(t,'Remove Added Timezones').onAction();assert.equal(cached.length,1);accept=true;await action(t,'Remove Added Timezones').onAction();assert.equal(cached.length,0);
 t=render(source.Timezones);const tokyo=actions(t).find(a=>a.props.title.startsWith('Asia - Tokyo ('));tokyo.props.onAction();assert.deepEqual(Array.from(cached),['Asia/Tokyo']);t=render(source.Timezones);actions(t).find(a=>a.props.title.startsWith('Asia - Tokyo (')).props.onAction();assert.equal(cached.length,0);
 // Navigation captures the original props array. V1's stale move-down bound can create undefined after removal.
 states=[];const captured=['Asia/Tokyo','UTC','America/New_York'];let staleOrder=captured;
 const capturedProps={timezones:captured,onSort:v=>staleOrder=v};let capturedTree=render(source.OrderTimezones,capturedProps);
 await action(walk(capturedTree,n=>n.type?.tag==='List.Item')[0],'Remove Timezone').onAction();capturedTree=render(source.OrderTimezones,capturedProps);
 action(walk(capturedTree,n=>n.type?.tag==='List.Item')[1],'Move Down').onAction();assert.equal(staleOrder[1],undefined);
 fs.writeFileSync(path.join(base,'tests/oracle.json'),JSON.stringify({runtime:process.versions,clock,local:'America/Los_Angeles',cases,custom},null,2)+'\n');
 console.log(JSON.stringify({zones:zones.length,conversionCases:cases.length,customCases:custom.length,componentFlow:'snapshot/reset/add/toggle/sort/up/down/remove-confirm/remove-all-confirm/List/Detail passed',icu:process.versions.icu,tz:process.versions.tz}));
})().catch(e=>{console.error(e);process.exitCode=1});
