import * as T from 'three';
import { RoundedBoxGeometry } from 'three/addons/geometries/RoundedBoxGeometry.js';
import { mergeGeometries } from 'three/addons/utils/BufferGeometryUtils.js';
import type { Resident, Theme } from './state';
export const roleColors:Record<string,string>={chief:'#f7e5a0',mayor:'#f1d65b',scout:'#71e0c4',reader:'#88cbf0',analyst:'#a698fa',writer:'#ed83b2',editor:'#bacf92',critic:'#f1826e'};
export interface AgentModel{root:T.Group;leftLeg:T.Group;rightLeg:T.Group;leftArm:T.Group;rightArm:T.Group;cape?:T.Mesh;prop:T.Group;phase:number;moving:boolean;height:number;meshes:number;triangles:number}
// Small shared construction shapes. Final meshes are baked per joint and material.
const shapes={round:new RoundedBoxGeometry(1,1,1,1,.14),sphere:new T.SphereGeometry(1,12,8),small:new T.SphereGeometry(1,8,6),cylinder:new T.CylinderGeometry(1,1,1,10),cone:new T.ConeGeometry(1,1,10)};
/** Original stylized characters. Shapes and silhouettes carry detail without image textures. */
export function createAgent(r:Resident,theme:Theme):AgentModel{
 const root=new T.Group(),prop=new T.Group();root.userData.resident=r.id;
 const shadow=theme==='shadow',owner=r.id==='chief',bear=shadow&&r.id==='editor',dragon=shadow&&r.id==='scout',ant=shadow&&r.id==='critic',orc=shadow&&r.id==='reader';
 const colors:Record<string,string>={mayor:'#93b6ff',scout:'#71d5ff',reader:'#ba9aff',analyst:'#94b7ed',writer:'#90b9ff',editor:'#7bbcf8',critic:'#ae8bff'};
 const accent=owner?'#f4d88e':shadow?(colors[r.id]||'#98bfff'):(roleColors[r.id]||'#85cbd7');
 const dark=shadow?'#101a31':'#202d38',armor=shadow?'#2d4262':'#415761',edge=shadow?'#5d7ba5':'#7e9ca3';
 const skin=owner?'#deb99b':r.id==='reader'?'#a66e56':r.id==='writer'?'#d7a98b':r.id==='critic'?'#bb8a76':'#cfa180';
 const fabric=owner?'#e4e5d7':shadow?'#263957':r.id==='mayor'?'#586344':r.id==='reader'?'#5b7987':r.id==='writer'?'#835c7b':r.id==='critic'?'#6f454c':'#35575b';
 const materials=new Map<string,T.MeshStandardMaterial>();
 const mat=(color:string,glow=false)=>{const key=color+glow;if(!materials.has(key))materials.set(key,new T.MeshStandardMaterial({color,roughness:.7,metalness:shadow&&!owner?.42:.12,emissive:glow?color:'#000000',emissiveIntensity:glow?.65:0}));return materials.get(key)!;};
 const put=(parent:T.Object3D,geometry:T.BufferGeometry,x:number,y:number,z:number,w:number,h:number,d:number,color:string,glow=false)=>{const mesh=new T.Mesh(geometry,mat(color,glow));mesh.position.set(x,y,z);mesh.scale.set(w,h,d);parent.add(mesh);return mesh;};
 const ball=(p:T.Object3D,x:number,y:number,z:number,w:number,h:number,d:number,c:string,glow=false)=>put(p,shapes.sphere,x,y,z,w,h,d,c,glow);
 const dot=(p:T.Object3D,x:number,y:number,z:number,w:number,h:number,d:number,c:string,glow=false)=>put(p,shapes.small,x,y,z,w,h,d,c,glow);
 const box=(p:T.Object3D,x:number,y:number,z:number,w:number,h:number,d:number,c:string,glow=false)=>put(p,shapes.round,x,y,z,w,h,d,c,glow);
 const cone=(p:T.Object3D,x:number,y:number,z:number,w:number,h:number,d:number,c:string)=>put(p,shapes.cone,x,y,z,w,h,d,c);
 const rod=(p:T.Object3D,a:T.Vector3,b:T.Vector3,radius:number,c:string)=>{const m=put(p,shapes.cylinder,0,0,0,radius,a.distanceTo(b),radius,c);m.position.copy(a).add(b).multiplyScalar(.5);m.quaternion.setFromUnitVectors(new T.Vector3(0,1,0),b.clone().sub(a).normalize());return m;};
 const joint=(x:number,y:number,z:number)=>{const g=new T.Group();g.position.set(x,y,z);root.add(g);return g;};
 const leftLeg=joint(-.115,.6,0),rightLeg=joint(.115,.6,0),leftArm=joint(-.265,1.14,0),rightArm=joint(.265,1.14,0);
 let cape:T.Mesh|undefined;
 const eyes=(p:T.Object3D,y:number,z:number,width=.06,color=accent)=>{for(const side of [-1,1]){const e=dot(p,side*.09,y,z,width,.027,.025,color,true);e.rotation.z=side*-.12;}};
 const cloak=(color:string,w=.31)=>{const vertices=new Float32Array([-w,1.16,-.14,w,1.16,-.14,-w*1.5,.19,-.29,w*1.5,.19,-.29,0,.12,-.37]);const g=new T.BufferGeometry();g.setAttribute('position',new T.BufferAttribute(vertices,3));g.setIndex([0,1,2,1,3,2,2,3,4]);g.computeVertexNormals();cape=new T.Mesh(g,new T.MeshStandardMaterial({color,side:T.DoubleSide,roughness:.95}));cape.userData.animated=true;root.add(cape);};
 const horns=(p:T.Object3D,y:number,z:number,c=accent)=>{for(const side of [-1,1]){const m=cone(p,side*.2,y,z,.065,.36,.065,c);m.rotation.z=side*-.65;}};
 const wing=(side:number)=>{const g=new T.BufferGeometry();g.setAttribute('position',new T.Float32BufferAttribute([side*.18,1.17,-.13,side*.9,1.43,-.25,side*.76,.78,-.27,side*.43,.94,-.18],3));g.setIndex([0,1,3,1,2,3]);g.computeVertexNormals();const m=new T.Mesh(g,new T.MeshStandardMaterial({color:'#243554',side:T.DoubleSide,roughness:.85}));root.add(m);rod(root,new T.Vector3(side*.18,1.17,-.13),new T.Vector3(side*.9,1.43,-.25),.028,accent);rod(root,new T.Vector3(side*.18,1.17,-.13),new T.Vector3(side*.76,.78,-.27),.022,edge);};
 if(bear){
  // Tank has a low, broad bear silhouette, with four paws and round ears.
  leftLeg.position.set(-.23,.4,-.32);rightLeg.position.set(.23,.4,-.32);leftArm.position.set(-.26,.49,.33);rightArm.position.set(.26,.49,.33);
  ball(root,0,.65,0,.4,.39,.57,dark);ball(root,0,.89,.36,.32,.31,.32,armor);
  ball(root,0,.75,.64,.22,.14,.2,edge);dot(root,0,.81,.8,.1,.065,.055,dark);
  for(const side of [-1,1]){dot(root,side*.22,1.1,.33,.11,.13,.08,armor);dot(root,side*.22,1.11,.39,.06,.065,.025,accent);dot(root,side*.15,.94,.63,.065,.035,.028,accent,true);}
  for(const leg of [leftLeg,rightLeg,leftArm,rightArm]){ball(leg,0,-.18,0,.13,.26,.15,armor);ball(leg,0,-.35,.08,.15,.09,.2,dark);for(const x of [-.07,0,.07])cone(leg,x,-.35,.25,.025,.08,.025,'#aacbea').rotation.x=Math.PI/2;}
  ball(root,0,.75,-.55,.12,.12,.13,armor);
 }else{
  const bulk=orc?1.25:shadow&&r.id==='analyst'?1.28:1;
  for(const leg of [leftLeg,rightLeg]){ball(leg,0,-.14,0,.095,.2,.105,dark);ball(leg,0,-.37,.015,.08,.17,.09,fabric);ball(leg,0,-.53,.065,.105,.085,.17,dark);dot(leg,0,-.27,.075,.09,.09,.045,shadow?edge:'#6e858c');}
  ball(root,0,.87,0,.22*bulk,.3,.155,fabric);ball(root,0,.64,0,.18,.1,.14,dark);
  for(const arm of [leftArm,rightArm]){ball(arm,0,-.14,0,.095*bulk,.18,.1,fabric);dot(arm,0,-.3,0,.082,.085,.09,edge);ball(arm,0,-.4,.015,.075,.13,.08,shadow?armor:fabric);dot(arm,0,-.52,.015,.078,.09,.065,shadow&&!owner?dark:skin);if(shadow&&!owner)ball(arm,0,-.02,0,.14*bulk,.13,.15,armor);}
  ball(root,0,1.19,0,.09,.1,.08,shadow&&!owner?dark:skin);
  if(!shadow||owner){
   // Visible nose, ears, eyes, eyebrows and layered hair, with a tapered jaw.
   ball(root,0,1.43,.012,.18,.225,.16,skin);ball(root,0,1.31,.045,.13,.11,.13,skin);
   for(const side of [-1,1])dot(root,side*.178,1.42,.015,.045,.075,.04,skin);
   dot(root,0,1.42,.174,.035,.055,.035,skin);
   for(const side of [-1,1]){dot(root,side*.075,1.47,.159,.047,.027,.017,'#e7e4d8');dot(root,side*.077,1.47,.174,.02,.025,.012,'#1b303c');box(root,side*.075,1.53,.158,.075,.018,.02,'#33282a');}
   box(root,0,1.33,.149,.066,.012,.015,'#925c53');
   const hair=owner?'#2e3039':r.id==='reader'?'#c5c2b0':r.id==='writer'?'#7c3e75':r.id==='scout'?'#1b4349':'#30313c';
   ball(root,0,1.59,-.025,.19,.13,.16,hair);for(let j=0;j<5;j++){const m=ball(root,-.13+j*.06,1.63+Math.sin(j)*.015,.045,.065,.085,.11,hair);m.rotation.z=-.35;}
   for(const side of [-1,1])ball(root,side*.16,1.48,-.06,.04,.12,.09,hair);
   // Lapels, split coat tails and a small colored chest insignia.
   for(const side of [-1,1]){const lapel=box(root,side*.105,1.06,.145,.08,.25,.045,owner?'#f0d995':edge);lapel.rotation.z=side*-.28;ball(root,side*.15,.59,-.04,.11,.23,.15,fabric);}
   box(root,0,.83,.16,.04,.43,.025,owner?accent:'#a2b5b9');dot(root,-.13,1.03,.177,.045,.04,.022,accent,true);
   if(r.id==='reader')for(const side of [-1,1]){put(root,new T.TorusGeometry(.063,.009,4,12),side*.077,1.475,.181,1,1,1,'#293c48');}
   if(r.id==='scout'){ball(root,.19,1.49,-.01,.045,.08,.065,edge);rod(root,new T.Vector3(.2,1.53,-.02),new T.Vector3(.23,1.8,-.02),.012,accent);}
   if(r.id==='critic'){for(const side of [-1,1])ball(root,side*.11,1.32,.08,.045,.065,.07,'#4c3435');}
  }else{
   ball(root,0,1.43,.01,.18*(orc?1.3:1),.23,.17,armor);
   if(dragon){
    ball(root,0,1.44,.19,.13,.11,.23,dark);dot(root,0,1.46,.4,.085,.045,.045,edge);horns(root,1.67,-.06,edge);wing(-1);wing(1);
    rod(root,new T.Vector3(0,.7,-.13),new T.Vector3(0,.45,-.75),.055,dark);cone(root,0,.47,-.76,.07,.22,.07,accent).rotation.x=-1.1;
   }else if(ant){
    ball(root,0,1.47,.02,.15,.26,.16,dark);ball(root,0,.84,.02,.18,.27,.14,armor);ball(root,0,.65,-.06,.1,.14,.11,dark);
    for(const side of [-1,1]){rod(root,new T.Vector3(side*.07,1.67,0),new T.Vector3(side*.18,1.93,.035),.018,accent);const jaw=cone(root,side*.12,1.34,.17,.04,.23,.04,edge);jaw.rotation.z=side*-.55;const blade=cone(leftArm,side*.04,-.56,.12,.04,.42,.055,edge);blade.rotation.x=.45;}
   }else if(orc){
    ball(root,0,1.34,.16,.18,.12,.11,edge);horns(root,1.69,-.06,edge);for(const side of [-1,1])cone(root,side*.13,1.38,.23,.036,.19,.035,'#d8c5ad');
    put(root,new T.CylinderGeometry(.19,.34,.58,12),0,.55,0,1,1,1,'#41365b');
   }else{
    const helm=cone(root,0,1.67,.01,.18,.22,.17,armor);helm.rotation.y=Math.PI/4;
    ball(root,0,1.4,.13,.15,.16,.07,dark);box(root,0,1.43,.181,.04,.24,.035,edge);
    for(const side of [-1,1]){const cheek=box(root,side*.135,1.39,.12,.06,.18,.065,edge);cheek.rotation.z=side*-.18;}
    if(r.id==='mayor'){const plume=ball(root,0,1.83,-.045,.045,.22,.11,'#a64257');plume.rotation.x=-.35;cloak('#252944');}
    else if(r.id==='analyst'){horns(root,1.64,-.01,edge);ball(root,0,1.0,.095,.24,.23,.11,armor);}
    else cloak('#192942',.24);
   }
   eyes(root,1.49,dragon?.23:.178,ant?.065:.07);
   ball(root,0,1.0,.12,.18*bulk,.2,.075,armor);box(root,0,1.0,.192,.025,.27,.02,accent,true);
  }
  if(owner){
   cloak(shadow?'#809bbd':'#a8b0ac',.26);
   const crown=put(root,new T.TorusGeometry(.18,.025,5,16),0,1.67,0,1,1,1,accent);crown.rotation.x=Math.PI/2;
   for(const x of [-.14,0,.14])cone(root,x,1.77,.07,.036,.17,.036,accent);
   dot(root,0,1.72,.17,.035,.042,.018,'#9ae1f4',true);
  }
 }
 root.add(prop);
 // Role tools remain readable even when the named-shadow silhouette is nonhuman.
 if(r.role==='Reader'){
  prop.position.set(.34,.99,.25);box(prop,0,0,0,.28,.32,.055,shadow?'#71578a':'#2e4755');box(prop,0,0,.038,.23,.265,.025,'#d6d5bd');for(let i=0;i<3;i++)box(prop,0,.075-i*.067,.057,.16,.012,.008,'#6d7d85');
  if(orc){rod(root,new T.Vector3(-.43,.08,.1),new T.Vector3(-.43,1.75,.1),.026,'#776588');dot(root,-.43,1.84,.1,.11,.13,.11,'#e07686',true);}
 }else if(r.role==='Analyst'){
  prop.position.set(.35,1.03,.15);dot(prop,0,0,0,.13,.13,.13,accent,true);const ring=put(prop,new T.TorusGeometry(.18,.012,4,18),0,0,0,1,1,1,edge);ring.rotation.x=.7;
  if(shadow){ball(leftArm,-.05,-.3,.13,.21,.29,.07,armor);box(leftArm,-.05,-.3,.2,.025,.39,.02,accent,true);}
 }else if(r.role==='Mayor'&&shadow){
  box(root,-.35,.72,.02,.065,1.12,.035,edge).rotation.z=-.1;box(root,-.34,.5,.02,.29,.05,.055,accent);
 }else if(r.role==='Writer'||r.role==='Editor'){
  if(!bear){prop.position.set(.33,.94,.23);box(prop,0,0,0,.26,.21,.035,shadow?'#697da5':'#4b6268');box(prop,0,0,.025,.2,.15,.012,accent,true);rod(rightArm,new T.Vector3(.03,-.44,.09),new T.Vector3(.03,-.17,.12),.012,edge);}
 }else if(r.role==='Critic'&&!ant){box(prop,-.32,.94,-.1,.055,.95,.04,edge);box(prop,-.32,.61,-.1,.25,.05,.06,accent);}
 else if(r.role==='Scout'&&!dragon){box(root,0,.98,-.18,.27,.34,.12,dark);box(root,0,1.0,-.25,.035,.23,.025,accent);dot(rightArm,0,-.4,.09,.1,.07,.045,accent,true);}
 else if(owner){box(prop,-.32,.99,.15,.16,.23,.035,dark);box(prop,-.32,.99,.18,.115,.16,.015,'#9ce4e8',true);}

 // One draw per material in each articulated joint; facial pieces share the body draw.
 const groups:T.Group[]=[];root.traverse(o=>{if(o instanceof T.Group)groups.push(o);});
 for(const g of groups.reverse()){
  const buckets=new Map<T.Material,T.Mesh[]>();for(const o of g.children)if(o instanceof T.Mesh&&!o.userData.animated&&!Array.isArray(o.material)){
   if(!buckets.has(o.material))buckets.set(o.material,[]);buckets.get(o.material)!.push(o);
  }
  for(const [material,meshes] of buckets){if(meshes.length<2)continue;const baked=meshes.map(m=>{m.updateMatrix();const geometry=m.geometry.index?m.geometry.toNonIndexed():m.geometry.clone();return geometry.applyMatrix4(m.matrix);});const merged=mergeGeometries(baked);baked.forEach(g=>g.dispose());if(!merged)continue;
   const mesh=new T.Mesh(merged,material);g.add(mesh);for(const m of meshes){g.remove(m);if(!Object.values(shapes).includes(m.geometry as typeof shapes.sphere))m.geometry.dispose();}
  }
 }
 // Clone the remaining shared shapes so scene teardown never disposes the construction cache.
 root.traverse(o=>{o.userData.resident=r.id;if(o instanceof T.Mesh){if(Object.values(shapes).includes(o.geometry as typeof shapes.sphere))o.geometry=o.geometry.clone();o.castShadow=false;}});
 const blob=new T.Mesh(new T.CircleGeometry(bear?.5:.3,16),new T.MeshBasicMaterial({color:'#020a19',transparent:true,opacity:.28,depthWrite:false}));blob.rotation.x=-Math.PI/2;blob.position.y=.025;root.add(blob);
 root.updateMatrixWorld(true);const height=new T.Box3().setFromObject(root).max.y;
 let meshes=0,triangles=0;root.traverse(o=>{if(o instanceof T.Mesh){meshes++;triangles+=(o.geometry.index?.count||o.geometry.attributes.position.count)/3;}});
 root.scale.setScalar(owner?1:orc?1.07:1.02);
 return {root,leftLeg,rightLeg,leftArm,rightArm,cape,prop,phase:0,moving:false,height,meshes,triangles};
}
export function poseAgent(a:AgentModel,time:number,moving:boolean,activity:string){
 const swing=moving?Math.sin(time*8)*.45:0;
 a.leftLeg.rotation.x=swing;a.rightLeg.rotation.x=-swing;a.leftArm.rotation.x=-swing*.7;a.rightArm.rotation.x=swing*.7;
 a.root.position.y=moving?Math.abs(Math.sin(time*8))*.025:0;
 if(!moving&&activity==='Writing'){a.rightArm.rotation.x=-.7+Math.sin(time*4)*.04;a.leftArm.rotation.x=-.4;}
 if(!moving&&['Reading','Review'].includes(activity)){a.rightArm.rotation.x=-.48;a.leftArm.rotation.x=-.28;a.prop.rotation.y=Math.sin(time*.65)*.025;}
 if(!moving&&activity==='Coordinating')a.rightArm.rotation.x=-.35+Math.sin(time*2)*.08;
 if(a.cape)a.cape.rotation.x=moving?.08+Math.sin(time*8)*.035:0;
}
