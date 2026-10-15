import * as T from 'three';
import { RoundedBoxGeometry } from 'three/addons/geometries/RoundedBoxGeometry.js';
import { activeResidents, state, subscribe, select, update, residents, daylight, clockHours, type Resident } from './state';
import { generateRoads, door, pointOnPath, type GeneratedRoad } from './layout';
import { createAgent, poseAgent, roleColors, type AgentModel } from './characters';

/** Throwaway 3D candidate. Geometry and movement remain independent of work authority. */
export class TownScene {
 private scene=new T.Scene(); private camera=new T.OrthographicCamera(-14,14,7,-7,.1,120);
 private renderer!:T.WebGLRenderer;private world=new T.Group();private actors=new T.Group();
 private sun=new T.DirectionalLight('#ffe3b9',3);private ambient=new T.HemisphereLight('#b3d2f1','#17212f',2);
 private mats=new Map<string,T.MeshStandardMaterial>();private glowMats:T.MeshStandardMaterial[]=[];
 private geometries={box:new RoundedBoxGeometry(1,1,1,1,.035),cylinder:new T.CylinderGeometry(1,1,1,12),sphere:new T.SphereGeometry(1,16,10),cone:new T.ConeGeometry(1,1,4)};
 private agents=new Map<string,AgentModel>(); private paths:GeneratedRoad[]=[];
 private pickers:T.Object3D[]=[];private raycaster=new T.Raycaster();private labels=document.createElement('div');
 private w=1;private h=1;private angle=.67;private target=new T.Vector3(0,1.5,0);private followId:string|null=null;
 private timer=0;private clockTimer=0;private last=performance.now();private lastLight=Number.NEGATIVE_INFINITY;private lastLabels=0;
 private signature='';private elapsed=0;private frames=0;private frameMs=0;private maxFrameMs=0;private ready=true;
 private pointer:[number,number]=[0,0];private dragged=false;private down=false;private walking:Record<string,{x:number;z:number;moving:boolean}>={};
 constructor(private host:HTMLElement){
  try{this.renderer=new T.WebGLRenderer({antialias:true,alpha:false,powerPreference:'low-power'});}catch{this.ready=false;host.innerHTML='<div class="graphics-notice">3D is unavailable in this browser. The resident list and city controls remain available.</div>';return;}
  this.renderer.setPixelRatio(Math.min(devicePixelRatio,1.5));this.renderer.shadowMap.enabled=true;this.renderer.shadowMap.type=T.PCFSoftShadowMap;this.renderer.shadowMap.autoUpdate=false;
  this.renderer.outputColorSpace=T.SRGBColorSpace;this.renderer.toneMapping=T.ACESFilmicToneMapping;this.renderer.toneMappingExposure=1.35;
  const canvas=this.renderer.domElement;canvas.className='town-canvas';canvas.setAttribute('role','img');canvas.setAttribute('aria-label','Interactive 3D Agent City. Drag to rotate. Use the resident list for keyboard access.');
  this.labels.className='world-labels';host.append(canvas,this.labels);
  this.scene.add(this.world,this.actors,this.sun,this.ambient);this.sun.position.set(-7,15,8);this.sun.castShadow=true;this.sun.shadow.mapSize.set(1024,1024);this.sun.shadow.camera.left=-14;this.sun.shadow.camera.right=14;this.sun.shadow.camera.top=12;this.sun.shadow.camera.bottom=-12;this.sun.shadow.normalBias=.04;this.sun.shadow.bias=-.0005;this.sun.shadow.camera.far=55;
  new ResizeObserver(()=>this.resize()).observe(host);
  subscribe(()=>{this.refresh();this.invalidate();this.scheduleClock();});
  document.addEventListener('visibilitychange',()=>{clearTimeout(this.timer);clearTimeout(this.clockTimer);this.last=performance.now();if(!document.hidden){this.invalidate();this.scheduleClock();}});
  canvas.addEventListener('pointerdown',e=>{this.down=true;this.dragged=false;this.pointer=[e.clientX,e.clientY];canvas.setPointerCapture(e.pointerId);});
  canvas.addEventListener('pointermove',e=>{if(!this.down)return;const dx=e.clientX-this.pointer[0],dy=e.clientY-this.pointer[1];if(Math.abs(dx)+Math.abs(dy)>2){this.dragged=true;this.followId=null;this.angle-=dx*.006;if(e.shiftKey)this.target.y+=dy*.008;this.pointer=[e.clientX,e.clientY];this.invalidate();}});
  canvas.addEventListener('pointerup',e=>{this.down=false;if(this.dragged)return;const rect=canvas.getBoundingClientRect();this.raycaster.setFromCamera(new T.Vector2((e.clientX-rect.left)/rect.width*2-1,-(e.clientY-rect.top)/rect.height*2+1),this.camera);const hit=this.raycaster.intersectObjects([...this.pickers,...this.actors.children],true).find(h=>h.object.userData.resident);if(hit)select(hit.object.userData.resident);});
  canvas.addEventListener('wheel',e=>{e.preventDefault();update(()=>state.zoom=Math.max(.7,Math.min(2.6,state.zoom-e.deltaY*.0015)));},{passive:false});
  canvas.addEventListener('webglcontextlost',e=>{e.preventDefault();this.ready=false;clearTimeout(this.timer);});canvas.addEventListener('webglcontextrestored',()=>{this.ready=true;this.signature='';this.refresh();this.invalidate();});
  this.refresh();this.resize();this.scheduleClock();
 }
 reset(){this.followId=null;this.target.set(0,1.5,0);this.angle=.67;update(()=>state.zoom=1);}
 rotate(){this.followId=null;this.angle+=Math.PI/4;this.invalidate();}
 follow(id:string){this.followId=id;update(()=>state.zoom=2.2);}
 stats(){if(!this.renderer)return {renderer:'WebGL2 unavailable'};return {renderer:'Three.js / orthographic 3D',frames:this.frames,meanDrawMs:this.frames?+(this.frameMs/this.frames).toFixed(2):0,maxDrawMs:+this.maxFrameMs.toFixed(2),drawCalls:this.renderer.info.render.calls,triangles:this.renderer.info.render.triangles,geometries:this.renderer.info.memory.geometries,textures:this.renderer.info.memory.textures,characterPositions:structuredClone(this.walking),generatedRoads:this.paths.map(p=>({resident:p.resident,points:p.points,cells:p.cells})),scheduledFrames:!document.hidden&&!state.quiet&&!state.paused&&this.hasWork(),clockMode:state.clockMode,hour:clockHours(),daylight:daylight(),viewport:[this.w,this.h]};}
 private material(color:string,glow=false){const key=color+glow;if(!this.mats.has(key)){const mat=new T.MeshStandardMaterial({color,roughness:.83,metalness:glow?.15:.2,flatShading:true,emissive:glow?color:'#000000',emissiveIntensity:glow?.8:0});this.mats.set(key,mat);if(glow)this.glowMats.push(mat);}return this.mats.get(key)!;}
 private box(parent:T.Object3D,x:number,y:number,z:number,w:number,h:number,d:number,color:string,glow=false){const m=new T.Mesh(this.geometries.box,this.material(color,glow));m.position.set(x,y,z);m.scale.set(w,h,d);m.castShadow=!glow;m.receiveShadow=!glow;parent.add(m);return m;}
 private cylinder(parent:T.Object3D,x:number,y:number,z:number,r:number,h:number,color:string){const m=new T.Mesh(this.geometries.cylinder,this.material(color));m.position.set(x,y,z);m.scale.set(r,h,r);m.castShadow=true;m.receiveShadow=true;parent.add(m);return m;}
 private sphere(parent:T.Object3D,x:number,y:number,z:number,r:number,color:string,glow=false){const m=new T.Mesh(this.geometries.sphere,this.material(color,glow));m.position.set(x,y,z);m.scale.setScalar(r);m.castShadow=true;parent.add(m);return m;}
 private cone(parent:T.Object3D,x:number,y:number,z:number,r:number,h:number,color:string){const m=new T.Mesh(this.geometries.cone,this.material(color));m.position.set(x,y,z);m.rotation.y=Math.PI/4;m.scale.set(r,h,r);m.castShadow=true;parent.add(m);return m;}
 private ring(parent:T.Object3D,x:number,y:number,z:number,r:number,color:string,thickness=.025){const m=new T.Mesh(new T.TorusGeometry(r,thickness,4,40),this.material(color,true));m.rotation.x=-Math.PI/2;m.position.set(x,y,z);parent.add(m);return m;}
 private segment(a:[number,number],b:[number,number],width:number,y:number,color:string,glow=false){const len=Math.hypot(b[0]-a[0],b[1]-a[1]);const m=this.box(this.world,(a[0]+b[0])/2,y,(a[1]+b[1])/2,width,.035,len+.03,color,glow);m.rotation.y=Math.atan2(b[0]-a[0],b[1]-a[1]);}
 private sign(parent:T.Object3D,text:string,x:number,y:number,z:number,w:number,color:string){const canvas=document.createElement('canvas');canvas.width=512;canvas.height=96;const c=canvas.getContext('2d')!;c.fillStyle=state.theme==='shadow'?'#102449':'#0e242c';c.fillRect(0,0,512,96);c.fillStyle=color;c.font=state.theme==='shadow'?'42px Georgia':'bold 42px monospace';c.textAlign='center';c.fillText(text,256,64);const texture=new T.CanvasTexture(canvas);texture.colorSpace=T.SRGBColorSpace;const m=new T.Mesh(new T.PlaneGeometry(w,w*96/512),new T.MeshBasicMaterial({map:texture}));m.position.set(x,y,z);parent.add(m);}
 private plant(parent:T.Object3D,x:number,y:number,z:number){this.box(parent,x,y+.12,z,.35,.24,.35,'#455a5b');this.cylinder(parent,x,y+.48,z,.035,.52,'#637c70');for(const side of [-1,1]){const leaf=this.sphere(parent,x+side*.14,y+.61,z,.21,'#547e6e');leaf.scale.y=.7;}}
 private cyber(r:Resident){const g=new T.Group();g.position.set(r.x,0,r.z);this.world.add(g);const {width:w,depth:d}=r,h=r.height;
  const base=r.id==='mayor'?'#465964':r.id==='reader'?'#71898c':r.id==='critic'?'#3e4c57':'#526977';
  this.box(g,0,.14,0,w+.5,.28,d+.5,'#344751');this.box(g,0,h/2+.27,0,w,h,d,base);this.box(g,0,h+.3,0,w+.13,.16,d+.13,'#8b9f9c');
  this.box(g,0,h*.55,d/2+.025,w*.82,h*.7,.04,'#162c3a');
  for(let floor=0;floor<Math.floor(h/ .6);floor++)for(let j=0;j<4;j++){this.box(g,-w*.34+j*w*.225,.65+floor*.52,d/2+.055,w*.13,.24,.025,(j+floor)%3===0?'#e8cb7b':'#76bdbf',true);this.box(g,w/2+.03,.65+floor*.52,-d*.28+j*d*.2,.025,.22,d*.13,'#7ba4bd',true);}
  for(const side of [-1,1]){this.box(g,side*w*.47,h*.5+.3,d/2+.09,.085,h,.1,'#2c4453');this.box(g,side*w*.4,h+.46,0,.04,.13,d,'#c3cdba');}
  const color=roleColors[r.id]||'#d8c88b';
  if(r.id==='mayor'){
   this.box(g,-.4,h+.81,-.2,1.05,.92,1.05,'#405f6c');this.box(g,-.4,h+.92,.34,.85,.38,.025,'#85bcba',true);this.box(g,-.4,h+1.31,-.2,1.2,.12,1.15,'#e7d05e');
   this.box(g,.48,h+.47,.2,.62,.18,1.1,'#cbd1a1');this.box(g,-w*.6,1.8,.2,.55,.25,1.5,'#586f7c');
   this.cylinder(g,-.75,h+2,-.48,.035,1.5,'#b3c6cb');this.sphere(g,-.75,h+2.78,-.48,.07,'#fa8864',true);
   this.sign(g,'ATLAS / OPERATIONS',0,h-.18,d/2+.12,w*.88,'#f4da63');
  }else if(r.id==='scout'){
   this.cylinder(g,.4,h+1,-.3,.035,1.5,'#abbfc2');for(let j=0;j<4;j++)this.box(g,.4,h+.5+j*.25,-.3,.6-j*.1,.025,.035,'#7795a0');
   const dish=this.sphere(g,-.35,h+.7,.1,.45,'#adbcba');dish.scale.y=.3;dish.rotation.z=.5;this.box(g,-.1,.68,d/2+.42,w*.85,.09,.7,'#ddbb67');
  }else if(r.id==='reader'){
   this.box(g,0,h*.6,d/2+.07,w*.86,h*.66,.055,'#427580');for(let j=0;j<5;j++)this.box(g,-w*.4+j*w*.2,h*.6,d/2+.11,.035,h*.66,.03,'#aaccc6');
   for(let j=0;j<5;j++)this.box(g,-.55+j*.27,h+.62,-.1,.13,.6,.85,'#7d9b9e');this.sign(g,'ARCHIVE / SAGE',0,h+.11,d/2+.13,w*.95,'#b5e4e6');
  }else if(r.id==='analyst'){
   const dome=this.sphere(g,0,h+.27,0,.65,'#829cae');dome.scale.y=.65;this.ring(g,0,h+.36,0,.76,'#91ccdd');this.box(g,-.5,h+.3,-.4,.45,.45,.5,'#4c6979');
  }else if(r.id==='writer'){
   for(let j=0;j<3;j++){const roof=this.box(g,-.6+j*.6,h+.45,0,.55,.25,1.15,'#8191a1');roof.rotation.z=.12;}this.sign(g,'QUILL / PRESS',0,h-.22,d/2+.13,w*.83,'#f3a1c9');this.box(g,.6,h+.8,-.35,.19,1,.19,'#566572');
  }else if(r.id==='editor'){
   this.box(g,-.3,h+.56,-.1,.85,.6,.85,'#6b8289');for(const x of [.35,.7])this.plant(g,x,h+.4,0);this.box(g,0,.98,d/2+.3,w,.09,.55,'#a7ba9a');
  }else{
   for(const x of [-.6,.6])this.cylinder(g,x,h*.5+.3,.68,.1,h,'#9c6458');this.box(g,0,1.1,d/2+.22,1.1,.15,.4,'#b98369');this.sign(g,'VEX / REDLINE',0,h-.05,d/2+.15,w*.9,'#ffab83');
  }
  for(let i=0;i<4;i++)this.box(g,0,.18-i*.04,d/2+.21+i*.12,.57,.12,.18,'#809292');
  this.box(g,0,.67,d/2+.08,.38,.8,.05,'#11303c');this.box(g,-.16,.7,d/2+.12,.025,.6,.035,color,true);
  this.box(g,-w/2-.13,.62,-.25,.22,.7,.43,'#8d9f9e');for(let i=0;i<4;i++)this.box(g,-w/2-.25,.5+i*.1,-.25,.025,.025,.34,'#344b56');
  if(state.stage>0){this.box(g,-w/2-.5,.68,-.25,.85,1.2,1.15,'#577987');this.box(g,-w/2-.5,1.31,-.25,.95,.1,1.22,'#a1b19d');this.box(g,-w/2-.5,.85,.34,.6,.23,.03,'#9bc8c7',true);}if(state.stage>1){this.box(g,0,h+.55,-.2,w*.68,.45,d*.7,'#607e88');this.cylinder(g,.6,h+1.1,-.4,.03,1,'#c2d1ca');}
 }
 private pointedWindow(g:T.Group,x:number,y:number,z:number,w:number,h:number){const shape=new T.Shape();shape.moveTo(-w/2,0);shape.lineTo(-w/2,h*.65);shape.quadraticCurveTo(-w/2,h*.84,0,h);shape.quadraticCurveTo(w/2,h*.84,w/2,h*.65);shape.lineTo(w/2,0);shape.closePath();const m=new T.Mesh(new T.ShapeGeometry(shape),this.material('#6387bc',true));m.position.set(x,y,z);g.add(m);this.box(g,x,y+h*.43,z+.012,.025,h*.85,.025,'#283855');}
 private spire(g:T.Group,x:number,z:number,y:number,h:number,r=.2){this.cylinder(g,x,y+h/2,z,r,h,'#34405b');this.cone(g,x,y+h+.42,z,r*1.5,.85,'#526b9a');this.box(g,x,y+h*.6,z+r+.01,.035,h*.65,.025,'#7d9ae2',true);}
 private shadow(r:Resident){const g=new T.Group();g.position.set(r.x,0,r.z);this.world.add(g);const w=r.width,d=r.depth,h=r.height*.8;
  this.box(g,0,.15,0,w+.7,.3,d+.7,'#38435c');this.box(g,0,.38,0,w+.36,.16,d+.36,'#626e85');this.box(g,0,h/2+.48,0,w,h,d,'#48536c');
  this.box(g,0,h+.52,0,w+.15,.16,d+.15,'#79849a');
  for(let j=0;j<4;j++)this.box(g,0,.65+j*.39,d/2+.015,w,.017,.02,'#293853');
  if(r.id==='mayor'){
   this.box(g,0,h+.95,-.15,w*.62,.9,d*.65,'#5c6983');for(const x of [-1,1])for(const z of [-1,1])this.spire(g,x*w*.48,z*d*.48,.5,h+.1,.19);this.spire(g,0,-.2,h+1.4,1,.3);
   this.pointedWindow(g,0,.48,d/2+.035,.7,1.95);for(const x of [-.5,.5]){this.box(g,x,1.75,d/2+.13,.19,1.1,.04,'#28488b');this.box(g,x,2.15,d/2+.16,.025,.35,.015,'#a2cbff',true);}
  }else if(r.id==='scout'){
   this.cylinder(g,0,h+.7,0,.78,.35,'#65738f');for(const x of [-.5,.5])this.spire(g,x,.4,h+.85,.45,.12);this.cone(g,0,h+1.3,0,.5,.6,'#344b79');
  }else if(r.id==='analyst'){
   this.sphere(g,0,h+1,0,.38,'#a0a4f0',true);const orbit=this.ring(g,0,h+1,0,.65,'#b5bff1');orbit.rotation.x=.5;const orbit2=this.ring(g,0,h+1,0,.65,'#8b9de7');orbit2.rotation.z=.8;for(const x of [-.65,.65])this.spire(g,x,-.5,.45,h+.35,.17);
  }else if(r.id==='reader'){
   this.box(g,0,h+.85,-.2,w*.72,.55,d*.75,'#516783');for(const x of [-.63,0,.63])this.spire(g,x,-.3,.4,h+.5,.13);for(const x of [-.48,0,.48])this.pointedWindow(g,x,1.05,d/2+.04,.22,1.2);
  }else{
   this.cone(g,0,h+1.05,0,Math.max(w,d)*.83,1.1,'#3b527f');for(const x of [-w*.45,w*.45])this.spire(g,x,d*.45,.45,h*.7,.15);
   if(r.id==='critic')this.spire(g,0,-.5,h+.6,.85,.18);
  }
  if(r.id!=='mayor')this.pointedWindow(g,0,.48,d/2+.05,.42,1.05);
  for(const side of [-1,1]){
   this.box(g,side*(w/2-.13),h*.42+.45,d/2+.13,.17,h*.82,.25,'#77839d');this.box(g,side*(w/2+.26),.62,d/2+.26,.13,.8,.13,'#32415f');this.cone(g,side*(w/2+.26),1.12,d/2+.26,.11,.28,'#9ac9ff');
  }
  for(let i=0;i<5;i++)this.box(g,0,.37-i*.055,d/2+.19+i*.12,.7,.12,.16,'#78849c');
  this.sign(g,r.id==='mayor'?'COMMAND CITADEL':r.role.toUpperCase()+' / '+r.name.toUpperCase(),0,h+.38,d/2+.09,w*.91,'#b3d3ff');
  if(state.stage>0){this.box(g,-w/2-.45,.94,-.1,.7,1.3,1.05,'#495b7c');this.cone(g,-w/2-.45,1.9,-.1,.7,.7,'#3a5486');}if(state.stage>1){for(const x of [-.45,.45])this.spire(g,x,-.3,h+.6,1,.13);}
 }
 private terrain(){const shadow=state.theme==='shadow';this.scene.background=new T.Color(shadow?'#090f20':'#0e1a26');this.scene.fog=new T.Fog(shadow?'#090f20':'#0e1a26',38,64);
  this.box(this.world,0,-1.3,0,90,.2,90,shadow?'#0c1425':'#142331');
  const size=activeResidents().length>8?21:18;
  if(shadow){const shape=new T.Shape();const pts=[[-9,-5],[-7,-7],[5.5,-7],[9,-4.5],[9,4.5],[6,7],[-6,7],[-9,4.7]];const expansion=activeResidents().length>8?1.16:1;pts.forEach(([x,z],i)=>i?shape.lineTo(x*expansion,-z*expansion):shape.moveTo(x*expansion,-z*expansion));shape.closePath();const geo=new T.ExtrudeGeometry(shape,{depth:.85,bevelEnabled:true,bevelThickness:.13,bevelSize:.13,bevelSegments:1,steps:1});geo.rotateX(-Math.PI/2);const m=new T.Mesh(geo,this.material('#344059'));m.position.y=-.9;m.receiveShadow=true;m.castShadow=true;this.world.add(m);this.ring(this.world,0,.03,1.75,2.1,'#6c94d7',.02);this.ring(this.world,0,.04,1.75,2.35,'#4c6f9f',.015);
   for(let j=0;j<16;j++){const a=j/16*Math.PI*2;this.box(this.world,Math.cos(a)*2.22,.06,1.75+Math.sin(a)*2.22,.09,.02,.16,'#8aa9df',true).rotation.y=-a;}
   for(let j=0;j<14;j++){const angle=j/14*Math.PI*2,x=Math.cos(angle)*13,z=Math.sin(angle)*12;this.cone(this.world,x,-.8,z,1+(j%3)*.4,2+(j%5)*.6,'#1d2943');}
  }else{
   this.box(this.world,0,-.42,0,size,.82,15.6,'#263e4b');this.box(this.world,0,.015,0,size-.18,.09,15.4,'#4f636b');
   for(let x=-8;x<9;x+=2)for(let z=-7;z<8;z+=2)this.box(this.world,x,.067,z,1.97,.008,1.97,(x+z)%4===0?'#50656b':'#4a6068');
   for(let j=0;j<16;j++){const angle=j/16*Math.PI*2,x=Math.cos(angle)*17,z=Math.sin(angle)*15;const h=1.5+(j%5)*1.1;this.box(this.world,x,h/2-1.2,z,1.8,h,2.2,'#213747');if(j%3===0)this.box(this.world,x,h*.7-1,z+1.11,1.2,.06,.02,'#426276',true);}
  }
  for(const road of this.paths)for(let i=0;i<road.points.length-1;i++){
   const a=road.points[i],b=road.points[i+1];this.segment(a,b,.65,.075,shadow?'#52627e':'#8e9d94');this.segment(a,b,.52,.1,shadow?'#253c62':'#233b49');
   if(state.roads){const len=Math.hypot(b[0]-a[0],b[1]-a[1]);for(let j=.16;j<len;j+=.5){const f=j/len,x=a[0]+(b[0]-a[0])*f,z=a[1]+(b[1]-a[1])*f;this.box(this.world,x,.13,z,.045,.015,.13,shadow?'#91b9ff':'#d7ce84',shadow);}}
  }
  this.cylinder(this.world,0,.13,1.75,.45,.15,shadow?'#5273a0':'#bec08c');this.ring(this.world,0,.22,1.75,.49,shadow?'#a4cfff':'#e3d46b');
  for(const [x,z] of [[-7,-5],[3,-6.5],[7,-4],[7,3],[-2,6.5],[-7,3],[-6,0]]){
   if(shadow){this.spire(this.world,x,z,.05,.8,.12);this.ring(this.world,x,1.05,z,.15,'#9ebaff');}else{this.cylinder(this.world,x,.72,z,.035,1.4,'#8ca3a5');this.box(this.world,x,1.45,z,.24,.09,.15,'#e2dbb3',true);}
  }
  if(!shadow){for(const [x,z] of [[-7,5],[6,5],[-6,-5.8],[6,-5.8]])this.plant(this.world,x,.1,z);this.box(this.world,6.6,.35,2,.65,.5,1.2,'#537f92');this.box(this.world,6.6,.66,2,.55,.22,.6,'#213e55');}
 }
 /** Instance repeated static primitives by geometry and material. Picking uses separate house volumes. */
 private batchStatic(){this.world.updateMatrixWorld(true);const buckets=new Map<string,T.Mesh[]>();this.world.traverse(o=>{if(o instanceof T.Mesh&&o.geometry&&!(Array.isArray(o.material))){const k=o.geometry.uuid+o.material.uuid;if(!buckets.has(k))buckets.set(k,[]);buckets.get(k)!.push(o);}});const batched=new T.Group();
  for(const objects of buckets.values()){const first=objects[0];const mesh=new T.InstancedMesh(first.geometry,first.material,objects.length);objects.forEach((o,i)=>mesh.setMatrixAt(i,o.matrixWorld));mesh.castShadow=first.castShadow;mesh.receiveShadow=first.receiveShadow;mesh.instanceMatrix.needsUpdate=true;mesh.computeBoundingSphere();batched.add(mesh);}this.world.clear();this.world.add(batched);
 }
 private disposeGroup(g:T.Group){const shared=new Set(Object.values(this.geometries));g.traverse(o=>{if(o instanceof T.Mesh){if(!shared.has(o.geometry))o.geometry.dispose();const materials=Array.isArray(o.material)?o.material:[o.material];for(const m of materials){if(m instanceof T.MeshBasicMaterial&&m.map)m.map.dispose();if(![...this.mats.values()].includes(m as T.MeshStandardMaterial))m.dispose();}}});g.clear();}
 private refresh(){if(!this.renderer)return;const signature=JSON.stringify([state.theme,state.stage,state.roads,state.layoutRevision,state.townId,activeResidents().map(r=>[r.id,r.x,r.z])]);if(signature===this.signature)return;this.signature=signature;this.elapsed=0;this.lastLight=Number.NEGATIVE_INFINITY;this.followId=null;this.target.set(0,1.5,0);this.disposeGroup(this.world);this.disposeGroup(this.actors);this.mats.forEach(m=>m.dispose());this.mats.clear();this.glowMats=[];this.walking={};this.agents.clear();for(const p of this.pickers){if(p instanceof T.Mesh){p.geometry.dispose();(p.material as T.Material).dispose();}}this.pickers=[];this.paths=generateRoads(activeResidents(),state.stage);this.terrain();
  for(const r of activeResidents()){
   if(state.theme==='cyber')this.cyber(r);else this.shadow(r);
   const picker=new T.Mesh(new T.BoxGeometry(r.width,r.height+1,r.depth),new T.MeshBasicMaterial({visible:false}));picker.position.set(r.x,(r.height+1)/2,r.z);picker.userData.resident=r.id;picker.updateMatrixWorld();this.pickers.push(picker);
   const actor=createAgent(r,state.theme);this.agents.set(r.id,actor);this.actors.add(actor.root);
  }
  this.batchStatic();this.renderer.shadowMap.needsUpdate=true;this.labelNodes();
 }
 private labelNodes(){this.labels.innerHTML=activeResidents().map(r=>`<button class="house-label ${state.selected===r.id?'selected':''}" data-select="${r.id}" aria-label="Inspect ${r.name}, ${r.role}, ${r.activity}"><span class="status-dot ${['Reading','Researching'].includes(r.activity)?'active':''}"></span>${r.role}</button><button class="agent-label" data-follow="${r.id}" aria-label="Follow ${r.name}">${r.name}</button>`).join('');}
 private resize(){if(!this.renderer)return;const rect=this.host.getBoundingClientRect();this.w=rect.width;this.h=rect.height;if(this.w<1||this.h<1)return;this.renderer.setPixelRatio(Math.min(devicePixelRatio,state.variant==='C'?1.25:1.5));this.renderer.setSize(this.w,this.h,false);this.invalidate();}
 private positionCamera(){const aspect=this.w/this.h;let height=state.variant==='C'?10.8:13.6;if(aspect<1.1)height=25;this.camera.left=-height*aspect/2;this.camera.right=height*aspect/2;this.camera.top=height/2;this.camera.bottom=-height/2;this.camera.zoom=state.zoom;this.camera.updateProjectionMatrix();
  if(this.followId){const a=this.agents.get(this.followId);if(a){
   this.target.set(a.root.position.x,.9,a.root.position.z);
   const obstruction=(angle:number)=>activeResidents().reduce((score,r)=>{const dx=r.x-a.root.position.x,dz=r.z-a.root.position.z,forward=dx*Math.sin(angle)+dz*Math.cos(angle),side=Math.abs(dx*Math.cos(angle)-dz*Math.sin(angle));return score+(forward>.15&&forward<5&&side<(r.width+r.depth)*.33&&r.height+.8>1+forward*.78?1:0);},0);
   if(obstruction(this.angle)>0){const options=Array.from({length:8},(_,i)=>this.angle+(i<=4?i:i-8)*Math.PI/4);options.sort((x,y)=>obstruction(x)-obstruction(y)||Math.abs(x-this.angle)-Math.abs(y-this.angle));this.angle=options[0];}
  }}this.camera.position.set(this.target.x+Math.sin(this.angle)*28,this.target.y+22,this.target.z+Math.cos(this.angle)*28);this.camera.lookAt(this.target);this.camera.updateMatrixWorld();
 }
 private updateLighting(){const sun=daylight(),shadow=state.theme==='shadow';this.ambient.intensity=shadow?.65+sun*.75:.7+sun*1.25;this.sun.intensity=shadow?.65+sun*1.8:.6+sun*3.2;this.sun.color.set(shadow?'#afc7ff':sun>.4?'#ffe8ba':'#97bbe4');this.sun.position.set(-10+Math.sin(clockHours()/24*Math.PI*2)*7,10+sun*9,9);this.glowMats.forEach(m=>m.emissiveIntensity=.35+(1-sun)*.9);const color=new T.Color(shadow?'#090f20':'#101e2d').lerp(new T.Color(shadow?'#273b59':'#6a8696'),sun*.62);this.scene.background=color;this.scene.fog!.color.copy(color);this.renderer.shadowMap.needsUpdate=true;}
 private hasWork(){return residents.some(r=>!r.retired&&!['Resting','Paused','Pausing','Waiting','Needs approval'].includes(r.activity));}
 private animate(dt:number){if(!state.paused&&!state.quiet)this.elapsed+=dt;for(const r of activeResidents()){const a=this.agents.get(r.id)!;const road=this.paths.find(p=>p.resident===r.id);const work=['Researching','Reading','Writing','Analyzing'].includes(r.activity);if(work&&!state.paused&&!state.quiet&&!['Paused','Pausing'].includes(r.activity))a.phase+=dt;let x=door(r)[0]-.35,z=door(r)[1],angle=0,moving=false;
   if(work&&road?.points.length){const cycle=(a.phase+(r.id==='reader'?5:0))%30;let fraction=0;if(cycle<10){fraction=cycle/10;moving=true;}else if(cycle<17)fraction=1;else if(cycle<27){fraction=1-(cycle-17)/10;moving=true;}const p=pointOnPath(road.points,fraction);x=p.x;z=p.z;angle=p.angle+(cycle>=17&&cycle<27?Math.PI:0);}
   // Pause preserves the last visible position and pose rather than resetting an agent home.
   const previous=this.walking[r.id];if((r.activity==='Paused'||r.activity==='Pausing'||state.paused)&&previous){x=previous.x;z=previous.z;moving=false;angle=a.root.rotation.y;}
   a.root.position.x=x;a.root.position.z=z;a.root.rotation.y=angle;if(!state.paused&&r.activity!=='Paused'&&r.activity!=='Pausing')poseAgent(a,this.elapsed,moving&&!state.quiet,r.activity);this.walking[r.id]={x,z,moving:moving&&!state.quiet&&!state.paused};
  }
 }
 private project(x:number,y:number,z:number){const p=new T.Vector3(x,y,z).project(this.camera);return {x:(p.x+1)*this.w/2,y:(1-p.y)*this.h/2,visible:p.z<1&&Math.abs(p.x)<1.1&&Math.abs(p.y)<1.1};}
 private updateLabels(){for(const r of activeResidents()){const label=this.labels.querySelector<HTMLElement>(`[data-select="${r.id}"]`),name=this.labels.querySelector<HTMLElement>(`[data-follow="${r.id}"]`);if(label){const p=this.project(r.x,.1,r.z+r.depth/2+.65);label.style.left=p.x+'px';label.style.top=(p.y+6)+'px';label.hidden=!p.visible||(state.variant==='C'&&r.id!=='mayor');label.classList.toggle('selected',state.selected===r.id);label.classList.toggle('attention',r.activity==='Needs approval');}if(name){const a=this.agents.get(r.id)!,p=this.project(a.root.position.x,1.75,a.root.position.z);name.style.left=p.x+'px';name.style.top=p.y+'px';name.hidden=!p.visible||state.variant==='C'||(!['Researching','Reading'].includes(r.activity)&&this.followId!==r.id);}}
 }
 private scheduleClock(){clearTimeout(this.clockTimer);if(document.hidden||state.clockMode==='fixed')return;this.clockTimer=window.setTimeout(()=>{this.lastLight=Number.NEGATIVE_INFINITY;this.invalidate();this.scheduleClock();},state.clockMode==='accelerated'?(state.quiet?10000:1000):30000);}
 private invalidate(){clearTimeout(this.timer);if(!document.hidden&&this.ready&&this.w>0&&this.h>0)this.frame();}
 private frame(){if(!this.ready||document.hidden)return;const start=performance.now(),dt=Math.min(.1,(start-this.last)/1000);this.last=start;if(start-this.lastLight>(state.clockMode==='accelerated'?900:30000)){this.updateLighting();this.lastLight=start;}this.animate(dt);this.positionCamera();this.renderer.render(this.scene,this.camera);if(start-this.lastLabels>90){this.updateLabels();this.lastLabels=start;}this.frames++;const ms=performance.now()-start;this.frameMs+=ms;this.maxFrameMs=Math.max(this.maxFrameMs,ms);
  if(!state.quiet&&!state.paused&&this.hasWork())this.timer=window.setTimeout(()=>this.frame(),state.variant==='C'?84:42);
 }
}
