import fs from 'node:fs';
import {execFileSync} from 'node:child_process';
const repo='reefyai/rewrite-internet',gh=(args,input)=>execFileSync('gh',args,{encoding:'utf8',input});
for(const [name,color] of [['phase:1','0e8a16'],['priority:P0','b60205'],['priority:P1','fbca04'],['status:ready','0e8a16'],['status:claimed','1d76db'],['status:review','5319e7'],['status:done','6f7c85'],['status:triage','ededed']])gh(['label','create',name,'--repo',repo,'--color',color,'--force']);
const existing=JSON.parse(gh(['issue','list','--repo',repo,'--state','all','--limit','200','--json','title,url']));
for(const t of JSON.parse(fs.readFileSync('tasks/phase-1.json','utf8'))){
 const title=`[${t.id}] ${t.title}`;if(existing.some(i=>i.title===title))continue;
 const body=`## Task\n\n${t.id} · Phase one · ${t.priority}\n\n## Expected artifact\n\n\`${t.artifact}\` (or a justified equivalent)\n\n## Acceptance criteria\n\n${t.acceptance.map(x=>`- [ ] ${x}`).join('\n')}\n\n## Non-goals\n\n${t.non_goals.map(x=>`- ${x}`).join('\n')}\n\n## Start\n\nRead [CONTRIBUTING.md](https://github.com/${repo}/blob/main/CONTRIBUTING.md), check existing PRs, and comment \`/claim\`. Wait for bot confirmation. One active task per contributor.\n\nAfter opening your linked PR, comment \`/submitted <PR URL>\` here. Include exact validation commands and results; draft or unmeasured work must be identified honestly.\n`;
 console.log(gh(['issue','create','--repo',repo,'--title',title,'--body-file','-','--label','phase:1','--label',`priority:${t.priority}`,'--label','status:ready'],body).trim());
}
