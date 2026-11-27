import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import Ajv from 'ajv';

const trustedRoot=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
export function check(root){
 root=path.resolve(root);const errors=[],files=[];
 const fail=m=>errors.push(m);
 const walk=dir=>{for(const e of fs.readdirSync(dir,{withFileTypes:true})){
  if(['.git','node_modules','.participation'].includes(e.name))continue;
  const p=path.join(dir,e.name);
  if(e.isSymbolicLink()){fail(`Symlink is not allowed: ${path.relative(root,p)}`);continue;}
  if(e.isDirectory())walk(p);else if(e.isFile()){if(fs.statSync(p).size>1024*1024)fail(`File exceeds bootstrap size limit: ${p}`);else files.push(path.relative(root,p).split(path.sep).join('/'));}
 }};walk(root);
 const read=p=>fs.readFileSync(path.join(root,p),'utf8');
 const json=p=>{try{return JSON.parse(read(p));}catch(e){fail(`${p}: ${e.message}`);return null;}};
 const ajv=new Ajv({allErrors:true,strict:true});
 const validate=(kind,p,data)=>{const v=ajv.compile(JSON.parse(fs.readFileSync(path.join(trustedRoot,'schemas',`${kind}.schema.json`),'utf8')));if(!v(data))fail(`${p}: ${ajv.errorsText(v.errors)}`);};
 const tasks=json('tasks/phase-1.json')||[],taskIds=new Set();
 for(const t of tasks){if(!/^F\d{3}$/.test(t.id)||taskIds.has(t.id)||!['P0','P1'].includes(t.priority)||!t.title||!t.artifact||!Array.isArray(t.acceptance)||t.acceptance.length<2||t.acceptance.some(x=>typeof x!=='string'||!x.trim()))fail(`Invalid or duplicate task: ${t.id}`);taskIds.add(t.id);}
 const specs=new Map(),ids=new Set();
 for(const p of files.filter(p=>/^specifications\/[^/]+\.json$/.test(p))){
  const s=json(p);if(!s)continue;validate('specification',p,s);specs.set(p,s);
  if(ids.has(s.id))fail(`${p}: duplicate specification ID`);ids.add(s.id);
  if(!taskIds.has(s.task))fail(`${p}: unknown task`);
  const md=p.replace(/\.json$/,'.md');if(!files.includes(md)){fail(`${p}: matching Markdown is missing`);continue;}
  const prose=read(md);
  for(const heading of ['Intent and users','Standards and existing behavior','Scope and non-goals','Requirements','Security boundaries','Data lifecycle','Performance','Acceptance and evidence','Open decisions'])if(!prose.includes(`## ${heading}`))fail(`${md}: missing section ${heading}`);
  for(const id of s.requirements||[])if(!new RegExp(`^### ${id}(?:\\s|$)`,'m').test(prose))fail(`${md}: missing requirement ${id}`);
 }
 const covered=new Set(),evidenceIds=new Set();
 for(const p of files.filter(p=>/^evidence\/[^/]+\.json$/.test(p))){
  const e=json(p);if(!e)continue;validate('evidence',p,e);
  if(evidenceIds.has(e.id))fail(`${p}: duplicate evidence ID`);evidenceIds.add(e.id);
  const s=specs.get(e.spec);if(!s){fail(`${p}: unknown spec`);continue;}covered.add(e.spec);
  if(e.spec_version!==s.version)fail(`${p}: stale spec version`);
  if(e.task!==s.task)fail(`${p}: evidence and spec task differ`);
  const seen=new Set();
  for(const c of e.checks||[]){
   if(!s.requirements?.includes(c.requirement))fail(`${p}: unknown requirement ${c.requirement}`);seen.add(c.requirement);
   if(e.status==='proposed'&&(c.result!=='not-run'||c.artifact!==null))fail(`${p}: proposed checks must be not-run with no result artifact`);
   if(e.status!=='proposed'){
    if(!e.source_commit||!['pass','fail'].includes(c.result)||typeof c.artifact!=='string')fail(`${p}: measured check requires source, result and artifact`);
    else if(c.artifact.includes('\\')||path.posix.normalize(c.artifact)!==c.artifact||!c.artifact.startsWith('evidence/artifacts/')||!files.includes(c.artifact)||!read(c.artifact).trim())fail(`${p}: unsafe, empty or missing artifact`);
   }
  }
  for(const id of s.requirements||[])if(!seen.has(id))fail(`${p}: uncovered requirement ${id}`);
  if(e.status==='independently-reproduced'&&!e.independent_review)fail(`${p}: independent review reference required`);
 }
 for(const p of specs.keys())if(!covered.has(p))fail(`${p}: evidence plan required`);
 for(const p of files.filter(p=>p.endsWith('.md'))){
  for(const m of read(p).matchAll(/\[[^\]]*\]\(([^\s)]+)(?:\s+"[^"]*")?\)/g)){
   const link=m[1].replace(/^<|>$/g,'');if(/^(https?:|mailto:|#)/.test(link))continue;
   let target;try{target=decodeURIComponent(link.split('#')[0]);}catch{fail(`${p}: invalid link encoding`);continue;}
   const dest=path.resolve(root,path.dirname(p),target);
   if(!dest.startsWith(root+path.sep)||!fs.existsSync(dest))fail(`${p}: broken or unsafe local link ${link}`);
  }
 }
 return {ok:errors.length===0,errors,specifications:specs.size,evidence:covered.size,files:files.length};
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)){
 try{const result=check(process.argv[2]||process.cwd());console.log(JSON.stringify(result,null,2));if(!result.ok)process.exitCode=1;}catch(e){console.error(`Check failed: ${e.message}`);process.exitCode=1;}
}
