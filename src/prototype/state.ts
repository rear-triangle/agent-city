/** Throwaway visual prototype. All commands and provider events below are simulated. */
export type Theme = 'cyber' | 'shadow';
export type Variant = 'A' | 'B' | 'C';
export type Activity = 'Coordinating' | 'Researching' | 'Reading' | 'Waiting' | 'Resting' | 'Needs approval' | 'Pausing' | 'Paused' | 'Review';
export interface Resident {
  id: string; name: string; role: string; home: string; shadowHome: string;
  x: number; z: number; width: number; depth: number; height: number;
  activity: Activity; task: string; notes: string[]; skills: string[]; priorActivity?: Activity;
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
 lighting:'night' as 'day'|'dusk'|'night',
 quiet:matchMedia('(prefers-reduced-motion: reduce)').matches,
 selected:null as string|null,
 panel:null as 'resident'|'approvals'|'activity'|'goal'|'road'|'settings'|'outcomes'|null,
 tab:'Work', paused:false, approval:'pending' as 'pending'|'allowed'|'denied',
 phase:'working' as 'working'|'review'|'accepted',
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
 update(()=>{state.approval=allow?'allowed':'denied';resident('critic').activity=allow?'Review':'Waiting';
 exchanges.unshift({id:'MSG-015',from:'mayor',to:'critic',purpose:allow?'Permission allowed':'Permission denied',summary:allow?'Read the sample trace once. The demo scope remains read-only.':'Use the supplied documentation and disclose the missing trace check.',status:'Delivered',time:'Just now'});});
 toast(allow?'Demo permission allowed once. Review is continuing.':'Demo permission denied. Vex will disclose the unverified trace.');
 setTimeout(()=>update(()=>{state.phase='review';resident('mayor').activity='Review';resident('scout').activity='Resting';resident('reader').activity='Resting';}),1800);
}
export function accept() {
 if(state.phase!=='review')return;
 update(()=>{state.phase='accepted';state.accepted++;state.stage=state.accepted>=10?2:state.accepted>=3?1:0;residents.forEach(r=>r.activity='Resting');});
 toast('Sample outcome accepted. Afterlight has expanded with a new archive annex.');
}
export function replay() { update(()=>{state.phase='working';state.approval='pending';state.paused=false;state.accepted=2;state.stage=0;state.replay++;
 const statuses:Activity[]=['Coordinating','Researching','Reading','Waiting','Resting','Resting','Needs approval'];residents.forEach((r,i)=>r.activity=statuses[i]);});toast('Demo replay started. No provider calls or project changes.'); }
