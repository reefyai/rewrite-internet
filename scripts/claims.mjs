import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

export const leaseHours=2;
export function claimDecision({command,actor,assignees=[],labels=[],lease,otherActive=false,now=Date.now()}){
 if(!labels.includes('phase:1'))return 'not-task';
 if(command==='claim'){
  if(lease?.user===actor&&assignees.includes(actor))return 'already-yours';
  if(!labels.includes('status:ready')||assignees.length||lease)return 'unavailable';
  return otherActive?'other-active':'claim';
 }
 if(!lease||lease.user!==actor||!assignees.includes(actor))return 'not-owner';
 if(lease.pr)return 'in-review';
 if(command==='release')return 'release';
 if(command==='renew')return Date.parse(lease.expires)>now?'renew':'expired';
 return command==='submitted'?'submitted':'unknown';
}
export function closesIssue(body,number){return new RegExp(`^Closes #${number}\\s*$`,'im').test(body||'');}

async function run(){
 const repo=process.env.GITHUB_REPOSITORY;
 if(repo!=='reefyai/rewrite-internet')throw Error('Coordination runs only on upstream.');
 const token=process.env.GH_TOKEN;if(!token)throw Error('Missing GitHub workflow token.');
 const api=async(endpoint,method='GET',body)=>{
  const r=await fetch(`https://api.github.com/repos/${repo}/${endpoint}`,{method,headers:{Authorization:`Bearer ${token}`,Accept:'application/vnd.github+json','X-GitHub-Api-Version':'2022-11-28'},...(body?{body:JSON.stringify(body)}:{})});
  if(!r.ok)throw Error(`GitHub ${method} ${endpoint.split('?')[0]} returned ${r.status}`);
  return r.status===204?null:r.json();
 };
 const all=async(endpoint)=>{const out=[];for(let page=1;page<=20;page++){const rows=await api(`${endpoint}${endpoint.includes('?')?'&':'?'}per_page=100&page=${page}`);out.push(...rows);if(rows.length<100)return out;}throw Error('Pagination limit reached; no assignment made.');};
 const marker='<!-- ri-lease:v1 -->';
 const getLease=async(number)=>{
  const rows=await all(`issues/${number}/comments`);
  for(const c of rows.toReversed())if(c.user?.login==='github-actions[bot]'&&c.user?.type==='Bot'&&c.body?.startsWith(marker)){
   try{return {comment:c,lease:JSON.parse(c.body.split('\n')[1])};}catch{throw Error('Malformed bot lease; maintainer intervention required.');}
  }
  return {comment:null,lease:null};
 };
 const say=(n,body)=>api(`issues/${n}/comments`,'POST',{body});
 const save=(n,record,lease,message)=>{
  const body=`${marker}\n${JSON.stringify(lease)}\n\n${message}`;
  return record.comment?api(`issues/comments/${record.comment.id}`,'PATCH',{body}):say(n,body);
 };
 const state=async(issue,status)=>{
  const labels=issue.labels.map(l=>l.name).filter(n=>!n.startsWith('status:'));labels.push(`status:${status}`);
  await api(`issues/${issue.number}`,'PATCH',{labels});
 };
 const linkedPRs=async(issue)=>{
  const prs=await all('pulls?state=open');return prs.filter(p=>closesIssue(p.body,issue.number));
 };
 const release=async(issue,record,message,done=false)=>{
  await state(issue,done?'done':'ready');
  if(record.lease?.user)await api(`issues/${issue.number}/assignees`,'DELETE',{assignees:[record.lease.user]});
  await save(issue.number,record,null,message);
  if(done)await api(`issues/${issue.number}`,'PATCH',{state:'closed',state_reason:'completed'});
 };
 const event=JSON.parse(fs.readFileSync(process.env.GITHUB_EVENT_PATH,'utf8'));
 if(process.env.GITHUB_EVENT_NAME==='issue_comment'){
  if(event.issue.pull_request||event.comment.user.type==='Bot')return;
  const match=/^\/(claim|renew|release|submitted)(?: (https:\/\/github\.com\/reefyai\/rewrite-internet\/pull\/[0-9]+))?\s*$/.exec(event.comment.body||'');if(!match)return;
  const command=match[1],actor=event.comment.user.login;
  const issue=await api(`issues/${event.issue.number}`);if(issue.state!=='open')return;
  const record=await getLease(issue.number);
  const otherActive=(await all(`issues?state=open&assignee=${encodeURIComponent(actor)}&labels=phase%3A1`)).some(i=>i.number!==issue.number);
  const verdict=claimDecision({command,actor,assignees:issue.assignees.map(a=>a.login),labels:issue.labels.map(l=>l.name),lease:record.lease,otherActive});
  if(!['claim','renew','release','submitted'].includes(verdict)){await say(issue.number,`@${actor}: ${verdict}. Read the [claim rules](https://github.com/${repo}/blob/main/docs/agent-loop.md#claims). No new reservation was made.`);return;}
  if(verdict==='claim'){
   await api(`issues/${issue.number}/assignees`,'POST',{assignees:[actor]});
   const assigned=await api(`issues/${issue.number}`);if(!assigned.assignees.some(a=>a.login===actor))throw Error('Assignment was not confirmed.');
   await state(issue,'claimed');
   const lease={user:actor,expires:new Date(Date.now()+leaseHours*3600000).toISOString(),pr:null};
   await save(issue.number,record,lease,`Claim confirmed for @${actor} until ${lease.expires}. Use /renew, /release or /submitted <PR URL>.`);
  }else if(verdict==='renew'){
   const lease={...record.lease,expires:new Date(Date.now()+leaseHours*3600000).toISOString()};await save(issue.number,record,lease,`Renewed for @${actor} until ${lease.expires}.`);
  }else if(verdict==='release'){
   if((await linkedPRs(issue)).length){await say(issue.number,'An open linked PR exists. Submit it to review before releasing the task.');return;}
   await release(issue,record,`Released by @${actor}. Ready for a new claim.`);
  }else{
   if(!match[2]){await say(issue.number,'Use /submitted followed by the full upstream PR URL.');return;}
   const number=Number(match[2].split('/').at(-1)),pr=await api(`pulls/${number}`);
   if(pr.state!=='open'||pr.user.login!==actor||!closesIssue(pr.body,issue.number)){await say(issue.number,'Submission rejected: PR must be open, yours, and include a separate `Closes #'+issue.number+'` line.');return;}
   await state(issue,'review');await save(issue.number,record,{...record.lease,expires:null,pr:number},`Submitted by @${actor}: ${pr.html_url}. Task is in review; the claim no longer expires.`);
  }
 }else{
  for(const issue of (await all('issues?state=open&labels=phase%3A1')).filter(i=>!i.pull_request)){
   const record=await getLease(issue.number),lease=record.lease;if(!lease)continue;
   if(lease.pr){const pr=await api(`pulls/${lease.pr}`);if(pr.state==='closed')await release(issue,record,pr.merged_at?'Contribution merged. Task completed.':'PR closed without merge. Task returned to ready.',!!pr.merged_at);}
   else if(Date.parse(lease.expires)<=Date.now()){
    const prs=await linkedPRs(issue);if(prs.length){await state(issue,'review');await save(issue.number,record,{...lease,pr:prs[0].number,expires:null},`Open linked PR preserved for review: ${prs[0].html_url}`);}
    else await release(issue,record,'Claim expired. Task returned to ready; partial findings are welcome in the issue.');
   }
  }
 }
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url))run().catch(e=>{console.error(e.message);process.exitCode=1;});
