import * as T from 'three';
import { RoundedBoxGeometry } from 'three/addons/geometries/RoundedBoxGeometry.js';
import type { Resident, Theme } from './state';
export const roleColors:Record<string,string>={chief:'#f7e5a0',mayor:'#f1d65b',scout:'#71e0c4',reader:'#88cbf0',analyst:'#a698fa',writer:'#ed83b2',editor:'#bacf92',critic:'#f1826e'};
export interface AgentModel{root:T.Group;leftLeg:T.Group;rightLeg:T.Group;leftArm:T.Group;rightArm:T.Group;cape?:T.Mesh;prop:T.Group;phase:number;moving:boolean}
/** Original stylized models. No extracted game assets. Proportions are shared with the roster portraits. */
export function createAgent(r:Resident,theme:Theme):AgentModel{
 const root=new T.Group();root.userData.resident=r.id;const shadow=theme==='shadow';
 const shadowColors:Record<string,string>={chief:'#e3faff',mayor:'#ae9aff',scout:'#76baff',reader:'#a4d7ff',analyst:'#b6a7ff',writer:'#92b3ff',editor:'#96caff',critic:'#b0a4ff'};const color=shadow?(shadowColors[r.id]||'#9bc7ff'):(roleColors[r.id]||'#f0bc74');const materials=new Map<string,T.MeshStandardMaterial>();
 const mat=(c:string,glow=false)=>{const key=c+glow;if(!materials.has(key))materials.set(key,new T.MeshStandardMaterial({color:c,roughness:.73,metalness:shadow?.3:.13,flatShading:true,emissive:glow?c:'#000000',emissiveIntensity:glow?.9:0}));return materials.get(key)!;};
 const part=(parent:T.Object3D,w:number,h:number,d:number,x:number,y:number,z:number,c:string,glow=false)=>{const m=new T.Mesh(new RoundedBoxGeometry(w,h,d,1,Math.min(.025,w/4,h/4,d/4)),mat(c,glow));m.position.set(x,y,z);m.castShadow=true;parent.add(m);return m;};
 const dark=shadow?'#141c37':'#24333e',fabric=shadow?'#26375c':r.id==='reader'?'#b6c9cc':r.id==='mayor'?'#a69340':r.id==='critic'?'#763e3b':'#36565e';
 const limb=(x:number,y:number,z:number)=>{const group=new T.Group();group.position.set(x,y,z);root.add(group);return group;};
 const leftLeg=limb(-.115,.44,0),rightLeg=limb(.115,.44,0);
 for(const leg of [leftLeg,rightLeg]){part(leg,.145,.37,.17,0,-.17,0,dark);part(leg,.17,.13,.26,0,-.38,.04,'#101b2a');part(leg,.155,.1,.19,0,-.15,.02,shadow?'#526891':'#5c737d');}
 part(root,.34,.43,.23,0,.66,0,fabric);part(root,.36,.09,.26,0,.46,0,'#152433');
 const leftArm=limb(-.24,.81,0),rightArm=limb(.24,.81,0);
 for(const arm of [leftArm,rightArm]){part(arm,.16,.26,.18,0,-.13,0,fabric);part(arm,.13,.18,.14,0,-.34,0,shadow?'#364e7d':'#526970');part(arm,.13,.12,.15,0,-.46,0,shadow?'#131d35':'#c4b7a0');}
 part(root,.28,.3,.25,0,1.02,.015,shadow?'#202e4f':'#cdbca6');
 let cape:T.Mesh|undefined;
 if(shadow){
  const crown=new T.Mesh(new T.ConeGeometry(.21,.28,5),mat('#344772'));crown.position.set(0,1.23,0);crown.castShadow=true;root.add(crown);
  part(root,.36,.2,.28,0,1.06,0,'#18233d');part(root,.26,.035,.025,0,1.08,.151,color,true);
  for(const side of [-1,1]){const shoulder=part(root,.25,.19,.28,side*.25,.83,0,'#344a72');shoulder.rotation.z=side*.22;part(root,.03,.16,.285,side*.34,.83,0,color,true);}
  const shape=new T.Shape();shape.moveTo(-.22,.77);shape.lineTo(.22,.77);shape.lineTo(.33,.08);shape.lineTo(.08,.16);shape.lineTo(0,.03);shape.lineTo(-.12,.14);shape.lineTo(-.33,.06);shape.closePath();
  cape=new T.Mesh(new T.ShapeGeometry(shape),new T.MeshStandardMaterial({color:'#101a34',side:T.DoubleSide,roughness:1}));cape.position.set(0,0,-.16);cape.castShadow=true;root.add(cape);
  part(root,.21,.27,.03,0,.69,.135,'#3e5683');part(root,.04,.22,.036,0,.69,.16,color,true);
 }else{
  // Jacket panels, boots, shoulder hardware, visible hair, and an inset visor.
  part(root,.39,.15,.24,0,1.19,-.015,r.id==='writer'?'#cc90bc':r.id==='reader'?'#adc0ce':'#1b2a35');
  part(root,.31,.08,.035,0,1.065,.155,color,true);part(root,.04,.35,.025,-.06,.68,.135,'#a5bdbb');
  part(root,.25,.33,.14,0,.69,-.19,'#1d2b37');part(root,.08,.2,.03,.055,.7,-.274,color);
  for(const side of [-1,1])part(root,.12,.28,.27,side*.14,.44,0,fabric);
  part(root,.22,.065,.045,0,1.29,0,r.id==='scout'?'#28404a':'#89939a');
  if(r.id==='mayor')part(root,.2,.035,.02,.02,.79,.144,color,true);
 }
 if(r.id==='chief'){
  // The owner has a distinct white coat and crest, separate from the mayor's uniform.
  part(root,.43,.45,.26,0,.64,0,shadow?'#c0d8ef':'#d8cead');
  part(root,.04,.35,.035,0,.66,.15,color,true);
  for(const side of [-1,1])part(root,.1,.38,.27,side*.17,.39,0,shadow?'#96b8dc':'#b9b38e');
  part(root,.4,.065,.31,0,1.23,0,color);part(root,.09,.15,.07,0,1.31,0,color,true);
 }
 const prop=new T.Group();root.add(prop);
 switch(r.role){
  case'Scout':
   for(const side of [-1,1]){const blade=part(root,.035,.65,.07,side*.11,.79,-.28,shadow?'#6379b8':'#8dadaf');blade.rotation.z=side*.45;}part(rightArm,.15,.13,.16,0,-.4,.13,color,true);break;
  case'Reader':
   prop.position.set(.38,.7,.21);part(prop,.29,.36,.055,0,0,0,shadow?'#435487':'#263d4c');part(prop,.24,.29,.03,0,0,.04,'#b7d8e2');for(let i=0;i<4;i++)part(prop,.17,.012,.008,0,.1-i*.065,.06,'#52758d');break;
  case'Analyst':{
   prop.position.set(.34,.71,.18);const orb=new T.Mesh(new T.IcosahedronGeometry(.17,1),mat(color,true));prop.add(orb);const ring=new T.Mesh(new T.TorusGeometry(.23,.015,4,20),mat('#b5c9e3'));ring.rotation.x=.8;prop.add(ring);break;}
  case'Critic':{
   const sword=part(root,.105,1.25,.06,-.29,.8,-.23,shadow?'#9cadd4':'#91a7a7');sword.rotation.z=-.21;part(root,.42,.08,.08,-.22,.64,-.23,color);part(root,.2,.12,.05,0,1.01,.17,dark);break;}
  case'Writer':case'Editor':
   prop.position.set(.3,.65,.25);part(prop,.28,.21,.045,0,0,0,shadow?'#9aa9d1':'#a1c4bf');part(prop,.21,.012,.02,0,.04,.035,color,true);part(rightArm,.025,.4,.025,0,-.28,.13,color);break;
  default:
   prop.position.set(-.37,.78,.04);part(prop,.18,.26,.035,0,0,0,dark);part(prop,.13,.18,.02,0,0,.025,color,true);
 }
 root.scale.setScalar(1.13);root.traverse(o=>{o.userData.resident=r.id;if(o instanceof T.Mesh)o.castShadow=false;});const blob=new T.Mesh(new T.CircleGeometry(.31,16),new T.MeshBasicMaterial({color:'#020a19',transparent:true,opacity:.36,depthWrite:false}));blob.rotation.x=-Math.PI/2;blob.position.y=.025;root.add(blob);
 return {root,leftLeg,rightLeg,leftArm,rightArm,cape,prop,phase:0,moving:false};
}
export function poseAgent(a:AgentModel,time:number,moving:boolean,activity:string){
 const swing=moving?Math.sin(time*8)*.56:0;a.leftLeg.rotation.x=swing;a.rightLeg.rotation.x=-swing;a.leftArm.rotation.x=-swing*.8;a.rightArm.rotation.x=swing*.8;
 a.root.position.y=moving?Math.abs(Math.sin(time*8))*.032:0;
 if(!moving&&['Reading','Review'].includes(activity)){a.rightArm.rotation.x=-.72;a.leftArm.rotation.x=-.5;a.prop.rotation.y=Math.sin(time*.65)*.04;}
 if(!moving&&['Writing','Coordinating'].includes(activity))a.rightArm.rotation.x=-.7+Math.sin(time*2)*.12;
 if(a.cape)a.cape.rotation.x=moving?.12+Math.sin(time*8)*.055:0;
}
