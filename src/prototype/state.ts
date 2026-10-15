/** Throwaway visual prototype. All commands and provider events below are simulated. */
export type Theme = 'cyber' | 'shadow';
export type Variant = 'A' | 'B' | 'C';
export type Activity = 'Coordinating' | 'Researching' | 'Reading' | 'Waiting' | 'Resting' | 'Needs approval' | 'Pausing' | 'Paused' | 'Review';
export interface Resident {
  id: string; name: string; role: string; home: string; shadowHome: string;
  x: number; z: number; width: number; depth: number; height: number;
  activity: Activity; task: string; notes: string[]; skills: string[]; priorActivity?: Activity; retired?: boolean;
}
export const residents: Resident[] = [
  {id:'mayor', name:'Atlas', role:'Mayor', home:'The switchboard', shadowHome:'Command citadel', x:0,z:-.6,width:1.8,depth:1.6,height:3.5,activity:'Coordinating',task:'Coordinate the transport recommendation',notes:['Owner instruction: prefer a small dependency footprint.','Keep evidence linked to its original source.','Only the owner can accept a final outcome.'],skills:['Task planning','Evidence synthesis','Delegation']},
  {id:'scout',name:'Nyx',role:'Scout',home:'Signal station',shadowHome:'Watchtower',x:-4.4,z:-3.1,width:1.55,depth:1.4,height:2.2,activity:'Researching',task:'Find primary sources for three event transports',notes:['Prefer official documentation over summaries.','Research scope: WebSocket, SSE, and local IPC.'],skills:['Source discovery','Relevance checks']},
  {id:'reader',name:'Sage',role:'Reader',home:'Shard archive',shadowHome:'Archive of echoes',x:0,z:-4.5,width:1.7,depth:1.4,height:2.55,activity:'Reading',task:'Extract delivery and recovery guarantees',notes:['Distinguish a delivered message from an acknowledged result.','Record the documentation version with every finding.'],skills:['Source extraction','Citation tracking']},
  {id:'analyst',name:'Vector',role:'Analyst',home:'The observatory',shadowHome:'Astral chamber',x:4.3,z:-2.8,width:1.7,depth:1.55,height:2.0,activity:'Waiting',task:'Compare the transports once evidence is reviewed',notes:['Evaluate resource cost, recovery, and implementation effort.'],skills:['Tradeoff analysis','Decision records']},
  {id:'writer',name:'Quill',role:'Writer',home:'Press house',shadowHome:'Scriptorium',x:4.4,z:2.1,width:1.9,depth:1.4,height:1.65,activity:'Resting',task:'Draft the final recommendation',notes:['Start with the recommendation and explain its limits.'],skills:['Technical writing','Implementation checklists']},
  {id:'editor',name:'Echo',role:'Editor',home:'Revision studio',shadowHome:'Reflection hall',x:.2,z:4.5,width:1.8,depth:1.4,height:1.5,activity:'Resting',task:'Edit the recommendation after the draft is ready',notes:['Use plain words and preserve technical precision.'],skills:['Copy editing','Structure review']},
  {id:'critic',name:'Vex',role:'Critic',home:'Redline workshop',shadowHome:'Hall of judgment',x:-4.5,z:2,width:1.6,depth:1.6,height:1.9,activity:'Needs approval',task:'Verify the recovery claims against a sample trace',notes:['A successful tool call does not establish goal completion.','Flag claims that have no source or repeatable evidence.'],skills:['Evidence review','Failure analysis']},
];
export interface Exchange { id:string; from:string; to:string; purpose:string; summary:string; status:string; time:string }
export const exchanges: Exchange[] = [
 {id:'MSG-014',from:'scout',to:'mayor',purpose:'Research findings',summary:'Three primary sources are ready for review.',status:'Acknowledged',time:'Just now'},
 {id:'MSG-013',from:'mayor',to:'reader',purpose:'Evidence request',summary:'Extract delivery, reconnection, and ordering guarantees.',status:'Delivered',time:'1 min ago'},
 {id:'MSG-012',from:'critic',to:'mayor',purpose:'Decision needed',summary:'A read-only sample trace is outside the current demo grant.',status:'Acknowledged',time:'2 min ago'},
];
const params = new URLSearchParams(location.search);
export const state = {
 theme: (params.get('theme') === 'shadow' ? 'shadow' : 'cyber') as Theme,
 variant: (['A','B','C'].includes(params.get('variant') || '') ? params.get('variant') : 'A') as Variant,
 townId:'afterlight', layoutRevision:1, demand:'normal' as 'normal'|'high'|'low',
 clockMode:'local' as 'local'|'accelerated'|'fixed', clockOrigin:Date.now(),
 lighting:'night' as 'day'|'dusk'|'night',
 quiet:matchMedia('(prefers-reduced-motion: reduce)').matches,
 selected:null as string|null,
 panel:null as 'resident'|'approvals'|'activity'|'goal'|'road'|'settings'|'outcomes'|'towns'|null,
 tab:'Work', paused:false, approval:'pending' as 'pending'|'allowed'|'denied'|'none',
 phase:'working' as 'working'|'review'|'accepted'|'idle',
 accepted:2, stage:0, zoom:1, roads:true,
 revision:1, goal:'Choose the right event transport',
 tasks:[] as {id:string; title:string; resident:string; status:string}[],
 toast:'', replay:0,
};
const listeners = new Set<() => void>();
export function subscribe(fn:()=>void) { listeners.add(fn); return ()=>listeners.delete(fn); }
export function update(fn?:()=>void) { fn?.(); state.revision++; listeners.forEach(fn=>fn()); }
export function resident(id:string) { return residents.find(r=>r.id===id)!; }
let toastTimer:ReturnType<typeof setTimeout>;
export function toast(message:string) { clearTimeout(toastTimer);update(()=>state.toast=message);toastTimer=setTimeout(()=>update(()=>state.toast=''),4200); }
export function select(id:string) { update(()=>{state.selected=id;state.panel='resident';state.tab='Work';}); }
export function pauseResident(r:Resident) {
 if(r.activity==='Paused') { update(()=>{r.activity=r.priorActivity || 'Resting';}); return; }
 if(r.activity==='Pausing') return;
 update(()=>{r.priorActivity=r.activity;r.activity='Pausing';});
 setTimeout(()=>update(()=>{if(r.activity==='Pausing')r.activity='Paused';}),650);
}
export function pauseTown() {
 if(state.paused) { update(()=>{state.paused=false;residents.forEach(r=>{if(r.activity==='Paused'||r.activity==='Pausing')r.activity=r.priorActivity || 'Resting';});}); }
 else { update(()=>state.paused=true);residents.filter(r=>!['Resting','Paused','Pausing'].includes(r.activity)).forEach(pauseResident); }
}
export function setVariant(v:Variant) { update(()=>{state.variant=v;state.panel=null;});syncUrl(); }
export function setTheme(theme:Theme) { update(()=>state.theme=theme);syncUrl(); }
export function syncUrl() { const p=new URLSearchParams(location.search);p.set('variant',state.variant);p.set('theme',state.theme);history.replaceState({},'',`${location.pathname}?${p}`); }
export function decide(allow:boolean) {
 if(state.approval!=='pending')return;
 const originTown=state.townId;
 update(()=>{state.approval=allow?'allowed':'denied';resident('critic').activity=allow?'Review':'Waiting';
 exchanges.unshift({id:'MSG-015',from:'mayor',to:'critic',purpose:allow?'Permission allowed':'Permission denied',summary:allow?'Read the sample trace once. The demo scope remains read-only.':'Use the supplied documentation and disclose the missing trace check.',status:'Delivered',time:'Just now'});});
 toast(allow?'Demo permission allowed once. Review is continuing.':'Demo permission denied. Vex will disclose the unverified trace.');
 setTimeout(()=>{if(state.townId!==originTown){const t=towns.find(t=>t.id===originTown);if(t){t.snapshot.phase='review';t.people.find(r=>r.id==='mayor')!.activity='Review';for(const r of t.people.filter(r=>['scout','reader'].includes(r.id)))r.activity='Resting';}return;}update(()=>{state.phase='review';resident('mayor').activity='Review';resident('scout').activity='Resting';resident('reader').activity='Resting';});},1800);
}
export function accept() {
 if(state.phase!=='review')return;
 update(()=>{state.phase='accepted';state.accepted++;state.stage=state.accepted>=10?2:state.accepted>=3?1:0;residents.forEach(r=>r.activity='Resting');});
 toast(`Sample outcome accepted. ${currentTown().name} has expanded with a new archive annex.`);
}
export function replay() { update(()=>{state.phase='working';state.approval='pending';state.paused=false;state.accepted=2;state.stage=0;state.replay++;
 const statuses:Activity[]=['Coordinating','Researching','Reading','Waiting','Resting','Resting','Needs approval'];residents.forEach((r,i)=>{r.activity=statuses[i]||'Resting';r.retired=false;});state.layoutRevision++;});toast('Demo replay started. No provider calls or project changes.'); }

export interface PrototypeTown {id:string;name:string;purpose:string;seed:number;people:Resident[];snapshot:TownSnapshot;messages:Exchange[]}
interface TownSnapshot {demand:typeof state.demand;approval:typeof state.approval;phase:typeof state.phase;goal:string;accepted:number;stage:number;tasks:typeof state.tasks;paused:boolean}
const copy=<T>(value:T):T=>structuredClone(value);
const snapshot=():TownSnapshot=>({demand:state.demand,approval:state.approval,phase:state.phase,goal:state.goal,accepted:state.accepted,stage:state.stage,tasks:copy(state.tasks),paused:state.paused});
const initialPeople=copy(residents);
export const towns:PrototypeTown[]=[{id:'afterlight',name:'Afterlight',purpose:'Personal projects',seed:1,people:copy(residents),snapshot:snapshot(),messages:copy(exchanges)}];
export function currentTown(){return towns.find(t=>t.id===state.townId)!;}
export function activeResidents(){return residents.filter(r=>!r.retired);}
function saveTown(){const t=currentTown();t.people=copy(residents);t.snapshot=snapshot();t.messages=copy(exchanges);}
export function switchTown(id:string){if(id===state.townId)return;saveTown();const t=towns.find(t=>t.id===id);if(!t)return;update(()=>{state.townId=id;Object.assign(state,copy(t.snapshot));residents.splice(0,residents.length,...copy(t.people));exchanges.splice(0,exchanges.length,...copy(t.messages));state.panel='towns';state.selected=null;state.replay++;state.layoutRevision++;});}
export function foundTown(name:string,purpose:string){saveTown();const id=`city-${Date.now().toString(36)}`;towns.push({id,name,purpose,seed:towns.length+1,people:initialPeople.map(r=>({...copy(r),activity:'Resting',retired:false,notes:r.notes.slice(0,1),task:'Ready for an assignment'})),snapshot:{demand:'normal',approval:'none',phase:'idle',goal:'No active goal',accepted:0,stage:0,tasks:[],paused:false},messages:[]});switchTown(id);relayout();toast(`${name} founded in demo memory. Its residents and work are separate.`);}
export function relayout(){const t=currentTown();t.seed++;const slots:PointLike[]=[[-4.7,-3.8],[0,-4.9],[4.6,-3.4],[4.8,2.9],[0,5],[-4.8,2.7],[-8,-.5],[8,.5],[-3.7,6.6],[4,6.6],[-5.8,-6]];update(()=>{activeResidents().filter(r=>r.id!=='mayor').forEach((r,i)=>{const slot=slots[(i+t.seed)%Math.min(slots.length,Math.max(6,activeResidents().length-1))];r.x=slot[0]+Math.sin(t.seed*2+i)*.2;r.z=slot[1]+Math.cos(t.seed*3+i)*.2;});state.layoutRevision++;});}
type PointLike=[number,number];
export function expandCity(){if(state.stage>=2&&residents.length>=12&&!residents.some(r=>r.retired)){toast('The prototype population cap is 12. Reuse or consolidate existing homes.');return;}update(()=>{if(state.stage<2)state.stage++;else{
 const retired=residents.find(r=>r.retired);if(retired){retired.retired=false;}else if(residents.length<12){const i=residents.length,template=copy(initialPeople[3]);residents.push({...template,id:`specialist-${i}`,name:['Forge','Lyra','Rune','Flux','Onyx'][i-7],role:['Builder','Designer','Archivist','Tester','Researcher'][i-7],home:'Specialist annex',shadowHome:'Specialist chamber',x:[-7.8,7.8,-3.7,4,-5.8][i-7],z:[-.6,1,7,7,-6][i-7],activity:'Resting',retired:false,task:'Ready for an assignment',notes:['Retain useful work when the city consolidates.']});}}
 state.layoutRevision++;});toast('Expanded for additional capacity. Cosmetic stages do not grant permissions.');}
export function consolidateCity(){if(state.stage>0){update(()=>{state.stage--;state.layoutRevision++;});toast('Unused annexes consolidated. Identity, memory, and history remain.');return;}
 const candidate=[...residents].reverse().find(r=>r.id!=='mayor'&&!r.retired&&r.activity==='Resting'&&!state.tasks.some(t=>t.resident===r.id&&t.status!=='Accepted'));if(!candidate){toast('No idle home can be consolidated. Pause and finish its work first.');return;}update(()=>{candidate.retired=true;state.layoutRevision++;});toast(`${candidate.name}'s home is retired. Notes and history are retained for recall.`);}
export function recallResident(id:string){update(()=>{resident(id).retired=false;state.layoutRevision++;});toast(`${resident(id).name} recalled with the same identity and memory.`);}
export function clockHours(now=Date.now()){if(state.clockMode==='local'){const d=new Date(now);return d.getHours()+d.getMinutes()/60+d.getSeconds()/3600;}if(state.clockMode==='accelerated')return (17+(now-state.clockOrigin)/90000*24)%24;return {day:12,dusk:18,night:0}[state.lighting];}
export function daylight(now=Date.now()){return Math.max(0,Math.min(1,(Math.sin((clockHours(now)-6)/24*Math.PI*2)+.15)*1.4));}
export function clockLabel(){const h=clockHours();return `${String(Math.floor(h)).padStart(2,'0')}:${String(Math.floor(h%1*60)).padStart(2,'0')}`;}
export function setClockMode(mode:typeof state.clockMode){update(()=>{state.clockMode=mode;state.clockOrigin=Date.now();});}

/** Preview capacity responding to workload events. This fixture creates no provider work. */
export function previewDemand(high:boolean){
 if(high){
  while(residents.length<9){const i=residents.length;const template=copy(initialPeople[3]);residents.push({...template,id:`specialist-${i}`,name:i===7?'Forge':'Lyra',role:i===7?'Builder':'Designer',home:i===7?'Fabrication lab':'Design studio',shadowHome:i===7?'Forge chamber':'Hall of visions',x:i===7?-7.8:7.8,z:i===7?-.6:1,activity:'Waiting',task:'Additional demo workload queued',notes:['Capacity can consolidate without erasing my identity or knowledge.']});}
  update(()=>{state.stage=2;state.demand='high';for(const r of residents.filter(r=>r.id.startsWith('specialist-'))){r.retired=false;r.activity='Waiting';if(!state.tasks.some(t=>t.resident===r.id&&t.status==='Queued'))state.tasks.push({id:`DEMAND-${r.id}-${state.revision}`,title:`Prepare the ${r.role.toLowerCase()} deliverable`,resident:r.id,status:'Queued'});}state.layoutRevision++;});
  toast('Demo workload increased. Two specialist homes and annexes added. Active turns stay capped.');
 }else{
  update(()=>{state.stage=0;state.demand='low';for(const task of state.tasks.filter(t=>t.id.startsWith('DEMAND-')))task.status='Review';for(const r of residents.filter(r=>r.id.startsWith('specialist-'))){r.activity='Resting';r.retired=true;}state.layoutRevision++;});
  toast('Demo workload settled. Extra homes retired, notes and task history retained.');
 }
}
