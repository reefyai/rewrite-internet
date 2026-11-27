import {execFileSync} from 'node:child_process';
const tasks=JSON.parse(execFileSync('gh',['issue','list','--repo','reefyai/rewrite-internet','--state','open','--label','phase:1','--limit','100','--json','number,title,url,labels,assignees'],{encoding:'utf8'}));
for(const t of tasks.filter(t=>t.labels.some(l=>l.name==='status:ready')&&!t.assignees.length).sort((a,b)=>a.title.localeCompare(b.title)))console.log(`#${t.number} ${t.title}\n${t.url}\n`);
if(!tasks.some(t=>t.labels.some(l=>l.name==='status:ready')&&!t.assignees.length))console.log('No ready tasks. Check your existing PR or stop quietly.');
