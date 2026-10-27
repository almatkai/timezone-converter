/* Execute exact pinned renderer functions/components with deterministic React hooks and inert OS APIs. */
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict'),cp=require('node:child_process');
const root=path.resolve(__dirname,'..'),ts=require(root+'/qa/v1/node_modules/typescript');
// Apply only to owned fixture copies; never edit or apply against the host checkout.
const fixture=root+'/tests/host',patched=root+'/qa/patched-host';fs.mkdirSync(patched,{recursive:true});
for(const f of ['src/renderer/ExtensionRuntimeView.tsx','src/renderer/src/raycast-api/list-runtime.tsx']){fs.mkdirSync(path.dirname(patched+'/'+f),{recursive:true});fs.copyFileSync(fixture+'/'+f,patched+'/'+f)}
cp.execFileSync('git',['apply','--unsafe-paths',root+'/integration/host.patch'],{cwd:patched});
let active,seq=0;const timers=new Map();
const equal=(a,b)=>a&&b&&a.length===b.length&&a.every((x,i)=>Object.is(x,b[i]));
const react={Fragment:'Fragment',createElement:(type,props,...children)=>({type,props:{...props,children}}),
 useState:initial=>{const a=active,i=a.i++;if(!a.slots[i])a.slots[i]={v:typeof initial==='function'?initial():initial};return [a.slots[i].v,v=>{a.slots[i].v=typeof v==='function'?v(a.slots[i].v):v}]},
 useRef:v=>{const a=active,i=a.i++;return (a.slots[i]??={current:v})},
 useMemo:(fn,deps)=>{const a=active,i=a.i++;if(!a.slots[i]||!equal(a.slots[i].deps,deps))a.slots[i]={deps,v:fn()};return a.slots[i].v},
 useEffect:(fn,deps)=>{const a=active,i=a.i++;if(!a.slots[i]||!equal(a.slots[i].deps,deps)){const old=a.slots[i];a.effects.push(()=>{old?.cleanup?.();a.slots[i]={deps,cleanup:fn()}})}},
 useCallback:(fn,deps)=>react.useMemo(()=>fn,deps),forwardRef:f=>f,
 useImperativeHandle:(ref,fn)=>{active.i++;if(ref)ref.current=fn()}};
react.default=react;react.__esModule=true;
const jsx=(type,props)=>({type,props:props||{}});
const stubModules={'react':react,'react/jsx-runtime':{jsx,jsxs:jsx,Fragment:'Fragment'},'../../ui/primitives':{Hint:'Hint',HintBar:'HintBar',Kbd:'Kbd',ViewHeader:'ViewHeader',cx:(...x)=>x.filter(Boolean).join(' ')},'./ui/primitives':{Button:'Button',Message:'Message'},'../../ui/Markdown':{Markdown:'Markdown'},'../../colorConverter':{buildColorFormatRows:()=>[],buildColorConversionResults:()=>[]},'./extensionSessionRefresh':{startSessionRefresh:()=>()=>{}},'./src/raycast-api':{ExtensionRuntimeSurface:'Surface'}};
let response={ok:true,mode:'unchanged'},searchCalls=0;
const window={addEventListener(){},removeEventListener(){},clearTimeout:id=>timers.delete(id),setTimeout:fn=>{timers.set(++seq,fn);return seq},tezbar:new Proxy({extensionInvokeAction:async()=>response,extensionSearchTextChanged:async()=>{searchCalls++;return response},extensionDisposeSession:async()=>({})},{get:(o,k)=>o[k]||(()=>{throw Error('Denied host call '+String(k))})})};
const ctx=vm.createContext({window,console:{log(){},error(){}},setTimeout:window.setTimeout,clearTimeout:window.clearTimeout});
const loaded={};
function load(name,usePatch=false){const key=name+usePatch;if(loaded[key])return loaded[key];let f=(usePatch?patched:fixture)+'/'+name;if(!fs.existsSync(f))f=fixture+'/'+name;
 const mod={exports:{}};ctx.module=mod;ctx.exports=mod.exports;ctx.require=n=>{if(n==='./detail-runtime')return load('src/renderer/src/raycast-api/detail-runtime.tsx');if(n in stubModules)return stubModules[n];throw Error('Denied renderer import '+n)};
 const code=ts.transpileModule(fs.readFileSync(f,'utf8'),{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS,jsx:ts.JsxEmit.ReactJSX}}).outputText;
 vm.runInContext('(function(){'+code+';exports.__test={'+(name.endsWith('list-runtime.tsx')?'parseListRows,clientSideFilter':name.endsWith('form-runtime.tsx')?'collectFormItems':name.endsWith('detail-runtime.tsx')?'resolveMarkdownImageSrc,markdownFromNode':'fromRunResult')+'};})()',ctx,{timeout:2000});return loaded[key]=mod.exports;
}
function instance(fn){const a={slots:[],effects:[],i:0};return props=>{active=a;a.i=0;a.effects=[];const tree=fn(props);for(const e of a.effects)e();return tree}}
function walk(t,p,out=[]){if(Array.isArray(t)){for(const c of t)walk(c,p,out)}else if(t&&typeof t==='object'){if(p(t))out.push(t);walk(t.props?.children,p,out)}return out}
function flush(){const list=Array.from(timers.values());timers.clear();for(const f of list)f()}
const input=JSON.parse(fs.readFileSync(root+'/qa/renderer-input.json'));
const detail=load('src/renderer/src/raycast-api/detail-runtime.tsx');
let d=instance(detail.DetailRuntime)({root:input.home.root,title:'Timezone Converter',onBack(){},onRunPrimaryAction(){},onOpenActions(){}});
const md=walk(d,n=>n.type==='Markdown')[0];assert.ok(md);const uri=md.props.text.match(/data:image\/png;base64,[^)]+/)?.[0];if(uri){assert.equal(md.props.imageSrcResolver(uri),uri);assert.ok(Buffer.from(uri.split(',')[1],'base64').subarray(0,8).equals(Buffer.from([137,80,78,71,13,10,26,10])))}
const metadata=instance(detail.MetadataSidebar)({root:input.home.root.metadata});assert.ok(walk(metadata,n=>n.props?.children==='UTC (UTC)').length||JSON.stringify(metadata).includes('UTC (UTC)'));
const form=load('src/renderer/src/raycast-api/form-runtime.tsx');let submit;
const formRender=instance(form.FormRuntime);const props={root:input.form.root,title:'Timezone Converter',onBack(){},onSubmitForm:v=>submit=v,onChangeField(){throw Error('Submit-only form emitted a field action')},onInvokeAction(){},onOpenActions(){}};
let f=formRender(props);walk(f,n=>n.type==='input')[0].props.onChange({target:{value:'B invalid'}});f=formRender(props);walk(f,n=>n.type==='form')[0].props.onSubmit({preventDefault(){}});assert.equal(submit.customTime,'B invalid');
f=formRender({...props,root:input.invalid.root});f=formRender({...props,root:input.invalid.root});assert.equal(walk(f,n=>n.type==='input')[0].props.value,'B invalid');
walk(f,n=>n.type==='input')[0].props.onChange({target:{value:'A'}});f=formRender({...props,root:input.invalid.root});assert.equal(walk(f,n=>n.type==='input')[0].props.value,'A');
// Execute both original and patched ListRuntime. Native client-owned search must not round-trip.
for(const patch of [false,true]){
 const list=load('src/renderer/src/raycast-api/list-runtime.tsx',patch),render=instance(list.ListRuntime);let calls=0;
 const p={root:input.add.root,title:'Timezone Converter',commandName:'index',actions:input.add.actions,onBack(){},onRunPrimaryAction(){},onOpenActions(){},onSearchTextChanged(){calls++},onLoadMore(){}};
 let t=render(p);const search=walk(t,n=>n.type==='input')[0];assert.ok(search);search.props.onChange({target:{value:'Tokyo'}});t=render(p);flush();assert.equal(calls,patch?0:1);
 // An older server view with no controlled searchText cannot replace the input draft.
 t=render({...p,root:JSON.parse(JSON.stringify(input.add.root))});assert.equal(walk(t,n=>n.type==='input')[0].props.value,'Tokyo');
 assert.equal(list.__test.parseListRows(input.add.root).length,419);
 assert.equal(list.__test.clientSideFilter(list.__test.parseListRows(input.add.root),'Tokyo').length,1);
}
(async()=>{
 for(const patch of [false,true]){
  const View=load('src/renderer/ExtensionRuntimeView.tsx',patch).default,render=instance(View);
  const initial={...input.home,sessionId:'renderer-session',extensionId:'com.tezbar.native.timezone-converter',commandName:'index',generation:'test'};
  let t=render({initial,onBack(){}});let surface=walk(t,n=>n.type==='Surface')[0];assert.ok(surface);
  await surface.props.onInvokeAction('stale-id');let failed=false;try{t=render({initial,onBack(){}});surface=walk(t,n=>n.type==='Surface')[0];assert.ok(surface.props.root)}catch(e){failed=true}assert.equal(failed,!patch); // original loses root on unchanged; patch preserves it.
 }
 const Ajv=require(root+'/qa/tools/node_modules/ajv/dist/2020'),formats=require(root+'/qa/tools/node_modules/ajv-formats');const ajv=new Ajv({strict:false});formats(ajv);
 const validate=ajv.compile(JSON.parse(fs.readFileSync(fixture+'/schemas/extension-v3.schema.json')));const manifest=JSON.parse(fs.readFileSync(root+'/extension.json'));assert.ok(validate(manifest),JSON.stringify(validate.errors));
 let negative=0;for(const [key,value] of [['schemaVersion',2],['runtime','javascript'],['execution','embedded'],['protocolVersion',2],['id','timezone-converter'],['replaces',[]],['artifacts',{'bad-target':{}}],['capabilities',['invented']],['unexpected',true]]){assert.equal(validate({...manifest,[key]:value}),false,key);negative++}
 console.log(JSON.stringify({actualPinnedComponents:['FormRuntime','ListRuntime','DetailRuntime','MetadataSidebar','ExtensionRuntimeView'],clientDraft:'A/B/A retained',hostPatch:'baseline reproduced and patched fixtures passed',schemaPositive:1,schemaNegative:negative}));
})().catch(e=>{console.error(e);process.exitCode=1});
