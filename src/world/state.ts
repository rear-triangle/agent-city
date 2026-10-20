import type {World,Road} from '../app/types';
export type Theme='cyber'|'shadow';
export type Activity='Writing'|'Coordinating'|'Researching'|'Reading'|'Waiting'|'Resting'|'Needs approval'|'Pausing'|'Paused'|'Review';
export interface Resident{id:string;domainId:string;name:string;role:string;home:string;shadowHome:string;x:number;z:number;width:number;depth:number;height:number;activity:Activity;task:string;notes:string[];skills:string[];retired?:boolean}
export interface Exchange{id:string;from:string;to:string;purpose:string;summary:string;status:string;time:string}
export const residents:Resident[]=[];
export const exchanges:Exchange[]=[];
export const roadResidents=new Set<string>();
export const matureRoadResidents=new Set<string>();
export function roadContributions(world:World,roadId:string){const seen=new Set<string>();return world.messages.filter(m=>m.route.includes(roadId)&&m.state==='delivered'&&m.acknowledgedAt&&m.purpose==='result ready'&&world.tasks.some(t=>t.id===m.taskId&&t.state==='accepted'&&!t.stale)&&!seen.has(`${m.taskId}:${m.from}:${m.to}`)&&!!seen.add(`${m.taskId}:${m.from}:${m.to}`));}
export function roadIsTransit(world:World,road:Road){return road.presentation==='transit'||road.presentation!=='road'&&road.ruleVersion===1&&roadContributions(world,road.id).length>=12}
export const state={theme:'cyber' as Theme,variant:'A' as 'A'|'B'|'C',townId:'',layoutRevision:0,stage:0,quiet:matchMedia('(prefers-reduced-motion: reduce)').matches,selected:null as string|null,panel:null as string|null,detailsExpanded:false,paused:false,zoom:1,roads:true,cameraMode:'pan' as 'pan'|'orbit',peek:null as string|null,chatter:false,courierSerial:0,replay:0,phase:'idle',clockMode:'local' as 'local'|'accelerated'|'fixed',clockOrigin:Date.now(),lighting:'day' as 'day'|'dusk'|'night'};
const listeners=new Set<()=>void>();
export function subscribe(fn:()=>void){listeners.add(fn);return()=>listeners.delete(fn)}
export function update(fn?:()=>void){fn?.();listeners.forEach(fn=>fn())}
export function select(id:string){update(()=>{state.selected=id;state.panel='resident';state.detailsExpanded=false})}
export function activeResidents(){return residents.filter(r=>!r.retired).sort((a,b)=>Number(b.id==='mayor')-Number(a.id==='mayor')).slice(0,20)}
export function resident(id:string){return residents.find(r=>r.id===id)!}
export function residentName(r:Resident,theme:Theme=state.theme){const names:Record<string,string>={mayor:'Igris',scout:'Kaisel',reader:'Tusk',analyst:'Iron',writer:'Shadow Knight',editor:'Tank',critic:'Beru'};const generated=r.name==='Atlas'||r.name.startsWith(r.role+' ')&&/^\d+$/.test(r.name.slice(r.role.length+1));return theme==='shadow'&&r.id!=='chief'&&generated?(names[r.id]||`Shadow ${r.role}`):r.name}
export const chief:Resident={id:'chief',domainId:'chief',name:'Chief',role:'Chief',home:'Chief sanctuary',shadowHome:'Sovereign sanctuary',x:-5,z:12.7,width:3.6,depth:2.7,height:3.4,activity:'Resting',task:'Set direction and inspect work',notes:[],skills:[]};
export function clockHours(now=Date.now()){if(state.clockMode==='local'){const d=new Date(now);return d.getHours()+d.getMinutes()/60;}if(state.clockMode==='accelerated')return(17+(now-state.clockOrigin)/1200000*24)%24;return{day:12,dusk:18,night:0}[state.lighting]}
export function daylight(now=Date.now()){return Math.max(0,Math.min(1,(Math.sin((clockHours(now)-6)/24*Math.PI*2)+.15)*1.4))}
let layout='';
export function projectWorld(world:World,cityId:string){
 const city=world.cities.find(c=>c.id===cityId);if(!city)return;
 const population=world.residents.filter(r=>r.cityId===cityId);const agents=population.filter(r=>r.lifecycle==='active');
 const used=new Set<string>();const positions=[[-4.4,-3.1],[0,-4.5],[4.3,-2.8],[4.4,2.1],[.2,4.5],[-4.5,2],[-8,-.5],[8,.5],[-3.7,7],[4,7],[-6,-6],[7,-6],[9,-3],[9,3],[-9,-4],[-9,4],[-.2,-8],[4,-7.5],[.2,8.3]];
 const views=agents.map((r,i):Resident=>{let key=r.id===city.mayorId?'mayor':r.role.toLowerCase();if(used.has(key)||!['mayor','scout','reader','analyst','writer','editor','critic'].includes(key))key=`specialist-${r.id}`;used.add(key);const mayor=key==='mayor';const at=population.findIndex(a=>a.id===r.id)===0?[0,-.6]:positions[(population.findIndex(a=>a.id===r.id)-1+positions.length)%positions.length];const task=world.tasks.find(t=>t.residentId===r.id&&['running','starting','pausing','waiting','unknown'].includes(t.state))||[...world.tasks].reverse().find(t=>t.residentId===r.id&&!['accepted','cancelled'].includes(t.state));const activities:Record<string,Activity>={starting:'Waiting',running:mayor?'Coordinating':task?.checkpoint.startsWith('write_file:')?'Writing':task?.checkpoint.startsWith('read_file:')?'Reading':'Researching',waiting:'Needs approval',pausing:'Pausing',paused:'Paused',unknown:'Waiting',blocked:'Waiting',review:'Review',queued:'Waiting'};
 return{id:key,domainId:r.id,name:r.name,role:r.role,home:mayor?'The switchboard':`${r.role} workshop`,shadowHome:mayor?'Command citadel':`${r.role} chamber`,x:at[0],z:at[1],width:mayor?1.8:1.7,depth:1.6,height:mayor?3.5:1.8+(i%3)*.25,activity:r.held?'Paused':task?activities[task.state]||'Resting':'Resting',task:task?.title||'Ready for an assignment',notes:world.memories.filter(m=>m.residentId===r.id&&m.accepted).map(m=>m.content),skills:r.skills.map(id=>world.skills.find(s=>s.id===id)?.name||id)};
 });
 const nextLayout=JSON.stringify([cityId,views.map(v=>[v.id,v.domainId]),world.roads,world.roads.map(r=>roadIsTransit(world,r))]);
 const mapping=new Map(views.map(v=>[v.domainId,v.id]));
 update(()=>{
 residents.splice(0,residents.length,...views);state.townId=cityId;state.theme=world.settings.theme;state.quiet=world.settings.quiet||matchMedia('(prefers-reduced-motion: reduce)').matches;state.clockMode=world.settings.clockMode;state.lighting=world.settings.lighting;state.paused=city.held;
 const accepted=world.tasks.filter(t=>t.projectId===city.projectId&&t.id===t.goalId&&t.acceptedAt).length;state.stage=accepted>=10?2:accepted>=3?1:0;
 if(nextLayout!==layout){state.layoutRevision++;layout=nextLayout;}
 matureRoadResidents.clear();roadResidents.clear();for(const road of world.roads.filter(r=>r.enabled)){if(road.from===city.mayorId&&mapping.has(road.to)){roadResidents.add(mapping.get(road.to)!);if(roadIsTransit(world,road))matureRoadResidents.add(mapping.get(road.to)!);}if(road.to===city.mayorId&&mapping.has(road.from)){roadResidents.add(mapping.get(road.from)!);if(roadIsTransit(world,road))matureRoadResidents.add(mapping.get(road.from)!);}}
 const messages=world.messages.filter(m=>mapping.has(m.from)&&mapping.has(m.to)&&m.state==='delivered');
 const newest=messages.at(-1);if(newest&&!exchanges.some(m=>m.id===newest.id))state.courierSerial++;
 exchanges.splice(0,exchanges.length,...messages.map(m=>({id:m.id,from:mapping.get(m.from)!,to:mapping.get(m.to)!,purpose:m.purpose,summary:m.body,status:m.state,time:m.at})).reverse());
 });
}
