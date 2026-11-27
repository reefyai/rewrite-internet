import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {check} from '../scripts/check.mjs';

function fixture(t){
 const root=fs.mkdtempSync(path.join(os.tmpdir(),'ri-check-'));t.after(()=>fs.rmSync(root,{recursive:true,force:true}));
 const put=(p,v)=>{fs.mkdirSync(path.dirname(path.join(root,p)),{recursive:true});fs.writeFileSync(path.join(root,p),typeof v==='string'?v:JSON.stringify(v));};
 const spec={id:'SPEC-SYNTHETIC',version:'0.1.0',status:'draft',risk:'normal',task:'F001',requirements:['SYN-001','SYN-002'],sources:['https://www.rfc-editor.org/'],decisions:[]};
 const evidence={id:'EVIDENCE-SYNTHETIC',task:'F001',spec:'specifications/synthetic.json',spec_version:'0.1.0',status:'proposed',source_commit:null,checks:spec.requirements.map(requirement=>({requirement,method:'property-test',procedure:'Run synthetic cases',result:'not-run',artifact:null})),independent_review:null,limitations:['Plan only']};
 put('tasks/phase-1.json',[{id:'F001',priority:'P0',title:'Synthetic task',artifact:'specifications/synthetic.md',acceptance:['Positive case','Negative case']}]);
 put('specifications/synthetic.json',spec);put('evidence/synthetic.json',evidence);
 put('specifications/synthetic.md',['# Synthetic',...['Intent and users','Standards and existing behavior','Scope and non-goals','Requirements','Security boundaries','Data lifecycle','Performance','Acceptance and evidence','Open decisions'].map(s=>'## '+s),'### SYN-001','MUST reject invalid input.','### SYN-002','MUST preserve bytes.'].join('\n\n'));
 return {root,put,spec,evidence};
}
test('accepts a complete proposed spec and evidence plan',t=>{const f=fixture(t);assert.deepEqual(check(f.root).errors,[]);});
test('rejects missing requirement coverage',t=>{const f=fixture(t);f.evidence.checks.pop();f.put('evidence/synthetic.json',f.evidence);assert.match(check(f.root).errors.join('\n'),/uncovered requirement SYN-002/);});
test('rejects stale evidence version',t=>{const f=fixture(t);f.evidence.spec_version='0.0.1';f.put('evidence/synthetic.json',f.evidence);assert.match(check(f.root).errors.join('\n'),/stale spec version/);});
test('rejects measured claims without provenance or artifacts',t=>{const f=fixture(t);f.evidence.status='measured';f.put('evidence/synthetic.json',f.evidence);assert.match(check(f.root).errors.join('\n'),/measured check requires/);});
test('accepts measured failure with provenance and artifacts',t=>{const f=fixture(t);f.evidence.status='measured';f.evidence.source_commit='a'.repeat(40);f.put('evidence/artifacts/synthetic.txt','Synthetic failure log\n');for(const c of f.evidence.checks){c.result='fail';c.artifact='evidence/artifacts/synthetic.txt';}f.put('evidence/synthetic.json',f.evidence);assert.deepEqual(check(f.root).errors,[]);});
test('rejects artifact traversal and symlinks',t=>{const f=fixture(t);f.evidence.status='measured';f.evidence.source_commit='a'.repeat(40);for(const c of f.evidence.checks){c.result='pass';c.artifact='evidence/artifacts/../../../outside.txt';}f.put('evidence/synthetic.json',f.evidence);fs.symlinkSync('/tmp',path.join(f.root,'escape'));assert.match(check(f.root).errors.join('\n'),/Symlink/);assert.match(check(f.root).errors.join('\n'),/unsafe, empty or missing artifact/);});
test('rejects unreviewed independent reproduction claims',t=>{const f=fixture(t);f.evidence.status='independently-reproduced';f.put('evidence/synthetic.json',f.evidence);assert.match(check(f.root).errors.join('\n'),/independent review reference/);});
test('rejects reviewed spec without review reference',t=>{const f=fixture(t);f.spec.status='reviewed';f.put('specifications/synthetic.json',f.spec);assert.match(check(f.root).errors.join('\n'),/review/);});
test('checks cannot be bypassed by replacing candidate schema or package script',t=>{const f=fixture(t);f.put('schemas/specification.schema.json',{});f.put('package.json',{scripts:{test:'exit 0'}});delete f.spec.requirements;f.put('specifications/synthetic.json',f.spec);assert.match(check(f.root).errors.join('\n'),/requirements/);});
test('rejects broken local links',t=>{const f=fixture(t);f.put('README.md','[missing](docs/missing.md)');assert.match(check(f.root).errors.join('\n'),/broken or unsafe local link/);});
