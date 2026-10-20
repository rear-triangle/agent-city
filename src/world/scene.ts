import * as T from 'three';
import { RoundedBoxGeometry } from 'three/addons/geometries/RoundedBoxGeometry.js';
import { activeResidents, state, subscribe, select, update, residents, daylight, clockHours, chief, exchanges, residentName, resident, roadResidents, matureRoadResidents, type Resident } from './state';
import { generateRoads, door, pointOnPath, type GeneratedRoad, type Point } from './layout';
import { createAgent, poseAgent, roleColors, type AgentModel } from './characters';

/** Presentation only. Domain state comes exclusively from runtime snapshots. */
export class TownScene {
 private scene=new T.Scene(); private camera=new T.OrthographicCamera(-14,14,7,-7,.1,120);
 private renderer!:T.WebGLRenderer;private world=new T.Group();private actors=new T.Group();
 private cyanLight=new T.PointLight('#38deef',0,16,2);private redLight=new T.PointLight('#ff594c',0,15,2);
 private sun=new T.DirectionalLight('#ffe3b9',3);private ambient=new T.HemisphereLight('#b3d2f1','#17212f',2);
 private mats=new Map<string,T.MeshStandardMaterial>();private glowMats:T.MeshStandardMaterial[]=[];
 private geometries={box:new RoundedBoxGeometry(1,1,1,1,.035),cylinder:new T.CylinderGeometry(1,1,1,12),sphere:new T.SphereGeometry(1,16,10),cone:new T.ConeGeometry(1,1,4)};
 private teleportPortal:T.Group|null=null;private portalUntil=0;
 private lastReplay=0;private initialTowns=new Set<string>();
 private sceneTown=state.townId; private chiefAt:Point=door(chief); private courier:T.Group|null=null;
 private delivery:{points:Point[];elapsed:number;duration:number;id:string}|null=null;
 private deliveredSerial=-1; private completedInitial=false;
 private bubbles=new Map<string,{text:string;until:number}>(); private nextQuip=9;
 private agents=new Map<string,AgentModel>(); private paths:GeneratedRoad[]=[];
 private chiefMarker=document.createElement('button');private pickers:T.Object3D[]=[];private raycaster=new T.Raycaster();private labels=document.createElement('div');
 private w=1;private h=1;private angle=.67;private elevation=Math.atan2(22,28);private target=new T.Vector3(0,1.5,0);private followId:string|null=null;
 private externallyHidden=false;
 setVisible(visible:boolean){this.externallyHidden=!visible;clearTimeout(this.timer);clearTimeout(this.clockTimer);if(visible){this.invalidate();this.scheduleClock();}}
 private timer=0;private clockTimer=0;private last=performance.now();private lastLight=Number.NEGATIVE_INFINITY;private lastLabels=0;
 private lightState='';private signature='';private elapsed=0;private frames=0;private frameMs=0;private maxFrameMs=0;private ready=true;
 private navigation=document.createElement('details');
 private gesture:{id:number;startX:number;startY:number;lastX:number;lastY:number;mode:'pan'|'orbit';anchor:T.Vector3|null;click:boolean}|null=null;
 private dragged=false;private walking:Record<string,{x:number;z:number;moving:boolean;location:string}>={};
 constructor(private host:HTMLElement){
  try{this.renderer=new T.WebGLRenderer({antialias:true,alpha:false,powerPreference:'low-power'});}catch{this.ready=false;host.innerHTML='<div class="graphics-notice">3D is unavailable in this browser. The resident list and city controls remain available.</div>';return;}
  this.renderer.setPixelRatio(Math.min(devicePixelRatio,1.5));this.renderer.shadowMap.enabled=true;this.renderer.shadowMap.type=T.PCFShadowMap;this.renderer.shadowMap.autoUpdate=false;
  this.renderer.outputColorSpace=T.SRGBColorSpace;this.renderer.toneMapping=T.ACESFilmicToneMapping;this.renderer.toneMappingExposure=1.35;
  const canvas=this.renderer.domElement;canvas.className='town-canvas';canvas.setAttribute('role','img');canvas.setAttribute('aria-label','Interactive 3D Agent City. Click to teleport the Chief. Click a house to visit and open its menu. Drag to move the map. Right-drag or Option/Alt-drag to orbit and tilt. Scroll to zoom. Resident controls also support keyboard access.');
  this.labels.className='world-labels';host.append(canvas,this.labels);
  this.navigation.className='camera-navigation';
  this.navigation.innerHTML='<summary title="Map navigation controls"><span>Move map</span><b aria-hidden="true">⌄</b></summary><div class="camera-navigation-menu"><div class="camera-modes" role="group" aria-label="Left-drag camera mode"><button data-camera-mode="pan" aria-pressed="true">Move</button><button data-camera-mode="orbit" aria-pressed="false">Orbit</button></div><p><strong>Drag</strong><span class="camera-drag-help">Move the point you grabbed</span></p><p><strong>Right / Option-drag</strong> Rotate and tilt</p><p><strong>Scroll</strong> Zoom in or out</p><p><strong>Click</strong> Teleport the Chief</p><button data-action="reset-camera" class="camera-reset">Reset view</button></div>';
  host.append(this.navigation);
  this.chiefMarker.className='chief-presence';this.chiefMarker.dataset.action='locate-chief';this.chiefMarker.setAttribute('aria-label','Find Chief');
  this.chiefMarker.innerHTML='<svg viewBox="0 0 22 29" aria-hidden="true"><path d="m6 8-1-6 6 3 6-3-1 6M7 9h8v7l-4 3-4-3Zm0 9-4 9h16l-4-9M11 19v8"/></svg><span>Chief</span>';
  document.querySelector('.app-shell')!.append(this.chiefMarker);
  this.scene.add(this.world,this.actors,this.sun,this.ambient,this.cyanLight,this.redLight);this.cyanLight.position.set(-3,3,2);this.redLight.position.set(4,3,-2);this.sun.position.set(-7,15,8);this.sun.castShadow=true;this.sun.shadow.mapSize.set(1024,1024);this.sun.shadow.camera.left=-14;this.sun.shadow.camera.right=14;this.sun.shadow.camera.top=12;this.sun.shadow.camera.bottom=-12;this.sun.shadow.normalBias=.04;this.sun.shadow.bias=-.0005;this.sun.shadow.camera.far=55;
  new ResizeObserver(()=>this.resize()).observe(host);
  subscribe(()=>{this.refresh();this.lastLabels=0;this.invalidate();this.scheduleClock();});
  document.addEventListener('visibilitychange',()=>{clearTimeout(this.timer);clearTimeout(this.clockTimer);this.last=performance.now();if(!document.hidden){this.invalidate();this.scheduleClock();}});
  canvas.addEventListener('contextmenu',e=>e.preventDefault());
  canvas.addEventListener('pointerdown',e=>{
   if(this.gesture||![0,1,2].includes(e.button))return;
   const mode=e.button===2||e.altKey?'orbit':e.button===1?'pan':state.cameraMode;
   this.gesture={id:e.pointerId,startX:e.clientX,startY:e.clientY,lastX:e.clientX,lastY:e.clientY,mode,anchor:this.groundPoint(e.clientX,e.clientY),click:e.button===0&&!e.altKey};
   this.dragged=false;canvas.setPointerCapture(e.pointerId);
  });
  canvas.addEventListener('pointermove',e=>{
   const g=this.gesture;if(!g||g.id!==e.pointerId)return;
   if(!this.dragged&&Math.hypot(e.clientX-g.startX,e.clientY-g.startY)<5)return;
   this.dragged=true;this.followId=null;this.host.classList.add('is-navigating');
   if(g.mode==='pan'){
    const point=this.groundPoint(e.clientX,e.clientY);
    if(point&&g.anchor){this.target.x+=g.anchor.x-point.x;this.target.z+=g.anchor.z-point.z;}
   }else{this.angle-=(e.clientX-g.lastX)*.006;this.elevation=T.MathUtils.clamp(this.elevation+(e.clientY-g.lastY)*.006,.35,1.25);}
   g.lastX=e.clientX;g.lastY=e.clientY;this.invalidate();
  });
  canvas.addEventListener('pointerup',e=>{
   const g=this.gesture;if(!g||g.id!==e.pointerId)return;
   this.endGesture();if(this.dragged||!g.click)return;
   const rect=canvas.getBoundingClientRect();
   this.raycaster.setFromCamera(new T.Vector2((e.clientX-rect.left)/rect.width*2-1,-(e.clientY-rect.top)/rect.height*2+1),this.camera);
   const hit=this.raycaster.intersectObjects([...this.pickers,...this.actors.children.filter(o=>o.visible)],true).find(h=>h.object.userData.resident);
   if(hit){if(hit.object.userData.resident==='chief'&&!this.pickers.includes(hit.object))update(()=>{state.panel='chief';state.detailsExpanded=false;});else this.inspectHome(hit.object.userData.resident);return;}
   const point=this.groundPoint(e.clientX,e.clientY);if(point)this.teleportChief([point.x,point.z]);
  });
  canvas.addEventListener('pointercancel',e=>{if(this.gesture?.id===e.pointerId){this.dragged=true;this.endGesture();}});
  canvas.addEventListener('lostpointercapture',e=>{if(this.gesture?.id===e.pointerId){this.dragged=true;this.endGesture();}});
  canvas.addEventListener('wheel',e=>{e.preventDefault();update(()=>state.zoom=Math.max(.7,Math.min(2.6,state.zoom-e.deltaY*.0015)));},{passive:false});
  let contextNotice:HTMLDivElement|null=null;
  canvas.addEventListener('webglcontextlost',e=>{e.preventDefault();this.ready=false;clearTimeout(this.timer);if(!contextNotice){contextNotice=document.createElement('div');contextNotice.className='graphics-notice';contextNotice.setAttribute('role','status');contextNotice.textContent='City graphics are paused. Resident, task, and decision controls remain available.';this.host.append(contextNotice)}});
  canvas.addEventListener('webglcontextrestored',()=>{contextNotice?.remove();contextNotice=null;this.ready=true;this.signature='';this.refresh();this.invalidate();});
  this.refresh();this.resize();this.scheduleClock();
 }
 private groundPoint(x:number,y:number){const rect=this.renderer.domElement.getBoundingClientRect();this.raycaster.setFromCamera(new T.Vector2((x-rect.left)/rect.width*2-1,-(y-rect.top)/rect.height*2+1),this.camera);return this.raycaster.ray.intersectPlane(new T.Plane(new T.Vector3(0,1,0),0),new T.Vector3());}
 private endGesture(){const id=this.gesture?.id;this.gesture=null;this.host.classList.remove('is-navigating');if(id!==undefined&&this.renderer.domElement.hasPointerCapture(id))this.renderer.domElement.releasePointerCapture(id);}
 reset(){this.endGesture();this.elevation=Math.atan2(22,28);this.followId=null;this.target.set(0,1.5,0);this.angle=.67;update(()=>state.zoom=1);}
 rotate(){this.followId=null;this.angle+=Math.PI/4;this.invalidate();}
 follow(id:string){const r=id==='chief'?chief:activeResidents().find(r=>r.id===id);if(!r)return;this.chiefAt=door(r);this.followId=id;this.angle=(this.agents.get(id)?.root.rotation.y||0)+.4;update(()=>{state.zoom=2.2;state.peek=id;});}
 locateChief(){this.followId='chief';update(()=>state.zoom=1.9);}
 focusChief(){this.followId=null;this.target.set(chief.x,2.2,chief.z);this.angle=.67;this.elevation=Math.atan2(22,28);update(()=>{state.peek=null;state.zoom=1.5;});}
 /** House menus and owner travel share one entry point, including keyboard controls. */
 inspectHome(id:string){
  const r=id==='chief'?chief:activeResidents().find(r=>r.id===id);if(!r)return;
  this.teleportChief(door(r),false);
  if(id==='chief')update(()=>{state.panel='chief';state.detailsExpanded=false;});else select(id);
 }
 visitHome(id:string){
  const r=id==='chief'?chief:activeResidents().find(r=>r.id===id);if(!r)return;
  this.teleportChief(door(r),false);this.followId='chief';
  this.speak('chief',id==='chief'?'My own corner of the world.':`Hello, ${residentName(r)}. Just checking in.`);
  update(()=>{state.peek=null;state.zoom=1.9;});
 }
 private teleportChief(point:Point,closeMenu=true){
  this.chiefAt=point;this.portalUntil=performance.now()+1100;this.followId=null;
  // A scene click never queues work or changes communication roads.
  if(closeMenu)update(()=>{state.panel=null;state.selected=null;state.peek=null;});else this.invalidate();
 }
 private speak(id:string,text:string){if(state.chatter&&!state.quiet){if(!this.bubbles.has(id)&&this.bubbles.size>=2)this.bubbles.delete(this.bubbles.keys().next().value!);this.bubbles.set(id,{text,until:this.elapsed+5.5});}}
 private beginDelivery(){
  const exchange=exchanges[0];if(!exchange||state.quiet)return;
  const from=exchange.from==='mayor'?[[0,1.75] as Point]:this.paths.find(p=>p.resident===exchange.from)?.points;
  const to=exchange.to==='mayor'?[[0,1.75] as Point]:this.paths.find(p=>p.resident===exchange.to)?.points.slice().reverse();
  if(!from?.length||!to?.length)return;
  this.delivery={points:[...from,...to],elapsed:0,duration:3,id:exchange.id};
 }

 stats(){if(!this.renderer)return {renderer:'WebGL2 unavailable'};return {renderer:'Three.js / orthographic 3D',frames:this.frames,meanDrawMs:this.frames?+(this.frameMs/this.frames).toFixed(2):0,maxDrawMs:+this.maxFrameMs.toFixed(2),drawCalls:this.renderer.info.render.calls,triangles:this.renderer.info.render.triangles,geometries:this.renderer.info.memory.geometries,textures:this.renderer.info.memory.textures,characterPositions:structuredClone(this.walking),camera:{mode:state.cameraMode,target:this.target.toArray(),yaw:this.angle,elevation:this.elevation,zoom:state.zoom,dragging:!!this.gesture,following:this.followId},characterDetail:[...this.agents].map(([id,a])=>({id,meshes:a.meshes,triangles:a.triangles})),chiefTravel:'teleport',chiefVisible:this.agents.get('chief')?.root.visible,chiefScreen:this.project(this.chiefAt[0],1,this.chiefAt[1]),houseTargets:activeResidents().map(r=>({id:r.id,name:residentName(r),...this.project(r.x,.6,r.z)})),portalActive:performance.now()<this.portalUntil,insideView:state.peek,courier:this.delivery?{id:this.delivery.id,progress:this.delivery.elapsed/this.delivery.duration}:null,speech:[...this.bubbles].map(([id,b])=>({id,text:b.text})),generatedRoads:this.paths.map(p=>({resident:p.resident,points:p.points,cells:p.cells})),scheduledFrames:this.ready&&!this.externallyHidden&&!document.hidden&&!state.quiet&&!state.paused&&(this.hasMotion()||state.chatter&&this.hasWork()),motionFrames:this.hasMotion(),clockMode:state.clockMode,hour:clockHours(),daylight:daylight(),viewport:[this.w,this.h]};}
 private material(color:string,glow=false){const key=color+glow;if(!this.mats.has(key)){const mat=new T.MeshStandardMaterial({color,roughness:.83,metalness:glow?.15:.2,flatShading:true,emissive:glow?color:'#000000',emissiveIntensity:glow?.8:0});this.mats.set(key,mat);if(glow)this.glowMats.push(mat);}return this.mats.get(key)!;}
 private box(parent:T.Object3D,x:number,y:number,z:number,w:number,h:number,d:number,color:string,glow=false){const m=new T.Mesh(this.geometries.box,this.material(color,glow));m.position.set(x,y,z);m.scale.set(w,h,d);m.castShadow=!glow;m.receiveShadow=!glow;parent.add(m);return m;}
 private cylinder(parent:T.Object3D,x:number,y:number,z:number,r:number,h:number,color:string){const m=new T.Mesh(this.geometries.cylinder,this.material(color));m.position.set(x,y,z);m.scale.set(r,h,r);m.castShadow=true;m.receiveShadow=true;parent.add(m);return m;}
 private sphere(parent:T.Object3D,x:number,y:number,z:number,r:number,color:string,glow=false){const m=new T.Mesh(this.geometries.sphere,this.material(color,glow));m.position.set(x,y,z);m.scale.setScalar(r);m.castShadow=true;parent.add(m);return m;}
 private cone(parent:T.Object3D,x:number,y:number,z:number,r:number,h:number,color:string){const m=new T.Mesh(this.geometries.cone,this.material(color));m.position.set(x,y,z);m.rotation.y=Math.PI/4;m.scale.set(r,h,r);m.castShadow=true;parent.add(m);return m;}
 private ring(parent:T.Object3D,x:number,y:number,z:number,r:number,color:string,thickness=.025){const m=new T.Mesh(new T.TorusGeometry(r,thickness,4,40),this.material(color,true));m.rotation.x=-Math.PI/2;m.position.set(x,y,z);parent.add(m);return m;}
 private segment(a:[number,number],b:[number,number],width:number,y:number,color:string,glow=false){const len=Math.hypot(b[0]-a[0],b[1]-a[1]);const m=this.box(this.world,(a[0]+b[0])/2,y,(a[1]+b[1])/2,width,.035,len+.03,color,glow);m.rotation.y=Math.atan2(b[0]-a[0],b[1]-a[1]);}
 private sign(parent:T.Object3D,text:string,x:number,y:number,z:number,w:number,color:string){
  const shadow=state.theme==='shadow',canvas=document.createElement('canvas');canvas.width=512;canvas.height=112;const c=canvas.getContext('2d')!;
  c.fillStyle=shadow?'#102449':'#08151e';c.fillRect(0,0,512,112);c.strokeStyle=color;c.lineWidth=2;c.strokeRect(5,5,502,102);c.globalAlpha=.35;c.strokeRect(12,12,488,88);c.globalAlpha=1;
  c.fillStyle=color;c.shadowColor=color;c.shadowBlur=shadow?0:13;c.font=shadow?'38px "Friend or Foe Tall BB", Abel, sans-serif':'700 40px Rajdhani, sans-serif';c.textAlign='center';c.fillText(text,256,70,475);
  const texture=new T.CanvasTexture(canvas);texture.colorSpace=T.SRGBColorSpace;const m=new T.Mesh(new T.PlaneGeometry(w,w*112/512),new T.MeshBasicMaterial({map:texture,toneMapped:false}));m.position.set(x,y,z);parent.add(m);return m;
 }
 private beam(parent:T.Object3D,a:T.Vector3,b:T.Vector3,r:number,color:string){const m=new T.Mesh(this.geometries.cylinder,this.material(color));m.position.copy(a).add(b).multiplyScalar(.5);m.scale.set(r,a.distanceTo(b),r);m.quaternion.setFromUnitVectors(new T.Vector3(0,1,0),b.clone().sub(a).normalize());parent.add(m);return m;}
 private cable(parent:T.Object3D,a:T.Vector3,b:T.Vector3,sag=.45){for(let j=0;j<10;j++){const point=(t:number)=>a.clone().lerp(b,t).add(new T.Vector3(0,-Math.sin(t*Math.PI)*sag,0));this.beam(parent,point(j/10),point((j+1)/10),.017,'#18272d');}}
 private lantern(parent:T.Object3D,x:number,y:number,z:number){
  this.beam(parent,new T.Vector3(x,y+.4,z),new T.Vector3(x,y,z),.013,'#283138');const body=this.sphere(parent,x,y-.12,z,.16,'#db3928',true);body.scale.y*=.9;
  for(const dy of [-.23,0])this.cylinder(parent,x,y+dy,z,.13,.025,'#783d37');for(const dx of [-.07,.07])this.box(parent,x+dx,y-.12,z+.145,.012,.21,.015,'#a63e32');
 }
 private plant(parent:T.Object3D,x:number,y:number,z:number){this.box(parent,x,y+.12,z,.35,.24,.35,'#455a5b');this.cylinder(parent,x,y+.48,z,.035,.52,'#637c70');for(const side of [-1,1]){const leaf=this.sphere(parent,x+side*.14,y+.61,z,.21,'#547e6e');leaf.scale.y=.7;}}
 private cyber(r:Resident){const g=new T.Group();g.position.set(r.x,0,r.z);this.world.add(g);const {width:w,depth:d}=r,h=r.height;
  const base=r.id==='mayor'?'#33474b':r.id==='reader'?'#416361':r.id==='critic'?'#563a3b':r.id==='writer'?'#4d444f':'#31464d';
  this.box(g,0,.14,0,w+.5,.28,d+.5,'#344751');this.box(g,0,h/2+.27,0,w,h,d,base);this.box(g,0,h+.3,0,w+.13,.16,d+.13,'#516567');
  this.box(g,0,h*.55,d/2+.025,w*.82,h*.7,.04,'#162c3a');
  for(let floor=0;floor<Math.floor(h/ .6);floor++)for(let j=0;j<4;j++){this.box(g,-w*.34+j*w*.225,.65+floor*.52,d/2+.055,w*.13,.24,.025,(j+floor)%3===0?'#efbc78':'#4eb9c1',true);this.box(g,w/2+.03,.65+floor*.52,-d*.28+j*d*.2,.025,.22,d*.13,'#7ba4bd',true);}
  for(const side of [-1,1]){this.box(g,side*w*.47,h*.5+.3,d/2+.09,.085,h,.1,'#2c4453');this.box(g,side*w*.4,h+.46,0,.04,.13,d,'#6f8986');}
  const color=roleColors[r.id]||'#60dce2';
  if(r.id==='mayor'){
   this.box(g,-.4,h+.81,-.2,1.05,.92,1.05,'#405f6c');this.box(g,-.4,h+.92,.34,.85,.38,.025,'#85bcba',true);this.box(g,-.4,h+1.31,-.2,1.2,.12,1.15,'#ba6750');
   this.box(g,.48,h+.47,.2,.62,.18,1.1,'#cbd1a1');this.box(g,-w*.6,1.8,.2,.55,.25,1.5,'#586f7c');
   this.cylinder(g,-.75,h+2,-.48,.035,1.5,'#b3c6cb');this.sphere(g,-.75,h+2.78,-.48,.07,'#fa8864',true);
   this.sign(g,'ATLAS / OPERATIONS',0,h-.18,d/2+.12,w*.88,'#55e5eb');
  }else if(r.id==='scout'){
   this.cylinder(g,.4,h+1,-.3,.035,1.5,'#abbfc2');for(let j=0;j<4;j++)this.box(g,.4,h+.5+j*.25,-.3,.6-j*.1,.025,.035,'#7795a0');
   const dish=this.sphere(g,-.35,h+.7,.1,.45,'#788b8b');dish.scale.y=.3;dish.rotation.z=.5;this.box(g,-.1,.68,d/2+.42,w*.85,.09,.7,'#8e5d4a');
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
  const neon=r.id==='critic'?'#ff665a':r.id==='writer'?'#fb7cb9':'#52dfe4';
  // Layered street fronts, service pipes, roof equipment and lit shop awnings.
  this.box(g,0,1.52,d/2+.37,w*.85,.09,.62,r.id==='critic'?'#a2463e':'#284449');
  this.box(g,0,1.48,d/2+.68,w*.8,.035,.035,neon,true);
  for(const side of [-1,1])this.box(g,side*w*.43,.82,d/2+.6,.035,1.35,.035,'#52605d');
  this.box(g,w/2+.08,h*.55,-.18,.12,h*.85,.11,'#92614d');this.box(g,w/2+.14,.55,-.18,.22,.12,.24,'#526766');
  this.box(g,-.3,h+.53,.25,.64,.38,.52,'#293d42');for(let j=0;j<5;j++)this.box(g,-.3,h+.73,.07+j*.075,.48,.018,.025,'#6b817d');
  if(h>1.8){this.box(g,0,1.75,d/2+.2,w*.9,.09,.38,'#3f5458');for(const side of [-1,1])this.box(g,side*w*.43,1.98,d/2+.36,.025,.45,.025,'#708781');this.box(g,0,2.18,d/2+.36,w*.88,.025,.025,'#7a8a84');}
  this.box(g,w/2+.21,h*.74,d/2+.1,.45,.08,.16,'#263d45');
  const board=this.sign(g,r.role.toUpperCase(),w/2+.29,h*.74-.35,d/2+.14,.9,neon);board.rotation.z=Math.PI/2;
  this.box(g,-w/2-.02,h*.6,d/2+.08,.045,h*.54,.04,neon,true);
  this.lantern(g,-w*.29,1.36,d/2+.38);this.lantern(g,w*.29,1.36,d/2+.38);
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
  this.sign(g,r.id==='mayor'?'COMMAND CITADEL':r.role.toUpperCase()+' / '+residentName(r).toUpperCase(),0,h+.38,d/2+.09,w*.91,'#b3d3ff');
  if(state.stage>0){this.box(g,-w/2-.45,.94,-.1,.7,1.3,1.05,'#495b7c');this.cone(g,-w/2-.45,1.9,-.1,.7,.7,'#3a5486');}if(state.stage>1){for(const x of [-.45,.45])this.spire(g,x,-.3,h+.6,1,.13);}
 }
 private terrain(){const shadow=state.theme==='shadow';this.scene.background=new T.Color(shadow?'#090f20':'#0e1a26');this.scene.fog=new T.Fog(shadow?'#090f20':'#0e1a26',38,64);
  this.box(this.world,0,-1.3,0,90,.2,90,shadow?'#0c1425':'#142331');
  const size=activeResidents().length>12?26:activeResidents().length>8?21:18;
  if(shadow){const shape=new T.Shape();const pts=[[-9,-5],[-7,-7],[5.5,-7],[9,-4.5],[9,4.5],[6,7],[-6,7],[-9,4.7]];const expansion=activeResidents().length>12?1.5:activeResidents().length>8?1.16:1;pts.forEach(([x,z],i)=>i?shape.lineTo(x*expansion,-z*expansion):shape.moveTo(x*expansion,-z*expansion));shape.closePath();const geo=new T.ExtrudeGeometry(shape,{depth:.85,bevelEnabled:true,bevelThickness:.13,bevelSize:.13,bevelSegments:1,steps:1});geo.rotateX(-Math.PI/2);const m=new T.Mesh(geo,this.material('#344059'));m.position.y=-.9;m.receiveShadow=true;m.castShadow=true;this.world.add(m);this.ring(this.world,0,.03,1.75,2.1,'#6c94d7',.02);this.ring(this.world,0,.04,1.75,2.35,'#4c6f9f',.015);
   for(let j=0;j<16;j++){const a=j/16*Math.PI*2;this.box(this.world,Math.cos(a)*2.22,.06,1.75+Math.sin(a)*2.22,.09,.02,.16,'#8aa9df',true).rotation.y=-a;}
   for(let j=0;j<14;j++){const angle=j/14*Math.PI*2,x=Math.cos(angle)*13,z=Math.sin(angle)*12;if(Math.hypot(x-chief.x,z-chief.z)<5.5)continue;this.cone(this.world,x,-.8,z,1+(j%3)*.4,2+(j%5)*.6,'#1d2943');}
  }else{
   this.box(this.world,0,-.42,0,size,.82,activeResidents().length>12?20:15.6,'#263e4b');this.box(this.world,0,.015,0,size-.18,.09,activeResidents().length>12?19.8:15.4,'#243c42');
   for(let x=-8;x<9;x+=2)for(let z=-7;z<8;z+=2)this.box(this.world,x,.067,z,1.97,.008,1.97,(x+z)%4===0?'#2b4347':'#253a40');
   for(let j=0;j<16;j++){const angle=j/16*Math.PI*2,x=Math.cos(angle)*17,z=Math.sin(angle)*15;const h=1.5+(j%5)*1.1;if(z>-6||Math.hypot(x-chief.x,z-chief.z)<5.5)continue;this.box(this.world,x,h/2-1.2,z,1.8,h,2.2,'#213747');if(j%3===0)this.box(this.world,x,h*.7-1,z+1.11,1.2,.06,.02,'#426276',true);}
  }
  for(const road of this.paths)for(let i=0;i<road.points.length-1;i++){
   const a=road.points[i],b=road.points[i+1];this.segment(a,b,.65,.075,shadow?'#52627e':'#566963');this.segment(a,b,matureRoadResidents.has(road.resident)?.32:.52,.1,shadow?'#253c62':'#162b34');
   if(state.roads){const len=Math.hypot(b[0]-a[0],b[1]-a[1]);for(let j=.16;j<len;j+=.5){const f=j/len,x=a[0]+(b[0]-a[0])*f,z=a[1]+(b[1]-a[1])*f;this.box(this.world,x,.13,z,.045,.015,.13,shadow?'#91b9ff':'#d7ce84',shadow);}}
  }
  this.cylinder(this.world,0,.13,1.75,.45,.15,shadow?'#5273a0':'#3a5a63');this.ring(this.world,0,.22,1.75,.49,shadow?'#a4cfff':'#50e4df');
  for(const [x,z] of [[-7,-5],[3,-6.5],[7,-4],[7,3],[-2,6.5],[-7,3],[-6,0]]){
   if(shadow){this.spire(this.world,x,z,.05,.8,.12);this.ring(this.world,x,1.05,z,.15,'#9ebaff');}else{this.cylinder(this.world,x,.72,z,.035,1.4,'#8ca3a5');this.box(this.world,x,1.45,z,.24,.09,.15,'#e2dbb3',true);}
  }
  if(!shadow){for(const [x,z] of [[-7,5],[6,5],[-6,-5.8],[6,-5.8]])this.plant(this.world,x,.1,z);this.box(this.world,6.6,.35,2,.65,.5,1.2,'#537f92');this.box(this.world,6.6,.66,2,.55,.22,.6,'#213e55');}
 }
 private cyberDistrict(){
  // Background blocks give scale without adding simulated residents or altering roads.
  for(let i=0;i<12;i++){
   const x=-15+i*2.7,z=-10.5-(i%3)*1.5,h=4+(i*7%6),w=1.8+(i%2)*.5;
   this.box(this.world,x,h/2-.5,z,w,h,2.2,i%3?'#1b3541':'#3d4247');this.box(this.world,x,h-.35,z,w+.18,.2,2.3,'#42575b');
   for(let floor=0;floor<h/.8;floor++)for(let col=0;col<3;col++)if((floor+col+i)%3!==0)this.box(this.world,x-w*.31+col*w*.3,.2+floor*.78,z+1.12,w*.18,.23,.025,i%3?'#467f88':'#b57b54',true);
   this.box(this.world,x+w*.48,h*.7,z+1.16,.055,h*.4,.035,i%2?'#38cbd6':'#e06853',true);
  }
  const sorted=activeResidents().slice().sort((a,b)=>Math.atan2(a.z,a.x)-Math.atan2(b.z,b.x));
  for(let i=0;i<sorted.length;i++){
   const a=sorted[i],b=sorted[(i+1)%sorted.length];if(Math.hypot(a.x-b.x,a.z-b.z)>9)continue;
   this.cable(this.world,new T.Vector3(a.x,a.height+1.2,a.z),new T.Vector3(b.x,b.height+1.2,b.z),.7);
  }
  // Lantern-lined market at the rear edge; the active street network stays clear.
  this.cable(this.world,new T.Vector3(-7,2.6,-6.6),new T.Vector3(-2.5,2.6,-6.6),.3);
  for(let i=0;i<6;i++)this.lantern(this.world,-6.8+i*.8,2.3,-6.6);
  for(const x of [-6,-3.8]){this.box(this.world,x,.45,-7,.95,.85,.65,'#314e51');this.box(this.world,x,1.35,-7,1.3,.1,.95,'#845b49');this.box(this.world,x,1.31,-6.5,1.2,.035,.04,'#4bdde4',true);for(const side of [-1,1])this.box(this.world,x+side*.53,.8,-6.7,.03,1.1,.03,'#73837b');}
  // Static wet-surface glints. No reflections, blur pass, particles or extra lights per building.
  for(const [x,z] of [[-2,2],[3,4],[-6,-1],[2,-5]]){const patch=this.box(this.world,x,.085,z,.72,.007,.42,'#284f57');(patch.material as T.MeshStandardMaterial).roughness=.2;(patch.material as T.MeshStandardMaterial).metalness=.7;}
 }
 private chiefHome(){
  const blue=state.theme==='shadow',stone=blue?'#8c9eb9':'#7f9691',dark=blue?'#2c4267':'#263e4a',gold=blue?'#badfff':'#e3cb83',glow=blue?'#9de9ff':'#5be7ec';
  // An isolated estate on its own terrace. No pathfinding node or bridge is created.
  const base=new T.Group();base.position.set(chief.x,0,chief.z);base.userData.keepForCutaway=true;this.world.add(base);
  this.cylinder(base,0,-.42,0,3.1,.85,dark);this.cylinder(base,0,.04,0,3.04,.14,stone);this.ring(base,0,.13,0,2.96,gold,.025);
  for(let i=0;i<16;i++){const a=i/16*Math.PI*2;if(Math.sin(a)>.65)continue;const x=Math.cos(a)*2.85,z=Math.sin(a)*2.85;this.cylinder(base,x,.36,z,.07,.52,stone);this.sphere(base,x,.66,z,.09,gold,true);}
  // Arrival waystone and letterbox belong to the Chief's grounds, not the road graph.
  this.ring(base,1.7,.17,1.65,.53,glow);this.ring(base,1.7,.18,1.65,.68,gold,.017);this.cylinder(base,1.7,.1,1.65,.74,.12,dark);
  this.box(base,-1.7,.58,1.7,.11,1,.11,stone);this.box(base,-1.7,1.1,1.7,.5,.4,.4,gold);this.box(base,-1.7,1.12,1.91,.32,.04,.02,dark);
  const g=new T.Group();g.position.set(chief.x,0,chief.z);this.world.add(g);const w=chief.width,d=chief.depth;
  this.box(g,0,.28,0,w+.45,.32,d+.45,stone);this.box(g,0,1.65,0,w,2.5,d,dark);this.box(g,0,3.04,0,w+.35,.2,d+.3,stone);
  for(const x of [-1.4,-.95,.95,1.4]){this.cylinder(g,x,1.72,d/2+.14,.095,2.6,stone);this.box(g,x,3.08,d/2+.14,.3,.16,.3,gold);}
  this.box(g,0,1.15,d/2+.05,.82,1.65,.08,blue?'#3f6ca2':'#153843');this.box(g,0,2.08,d/2+.11,.8,.06,.04,glow,true);
  for(const x of [-1.15,1.15]){this.box(g,x,1.9,d/2+.05,.62,.95,.04,glow,true);this.box(g,x,1.89,d/2+.1,.035,.95,.04,gold);}
  for(let i=0;i<6;i++)this.box(g,0,.46-i*.06,d/2+.22+i*.15,1.65+i*.1,.12,.2,stone);
  if(blue){
   const dome=this.sphere(g,0,3.65,-.15,1.25,'#527eb6');dome.scale.y*=.8;this.ring(g,0,3.45,-.15,1.3,gold);
   for(const x of [-1.65,1.65])for(const z of [-1.15,1.15]){this.cylinder(g,x,2.35,z,.22,3.8,stone);this.cone(g,x,4.75,z,.4,1.15,'#4269a0');this.sphere(g,x,5.38,z,.065,glow,true);}
   this.cone(g,0,5.03,-.15,.3,.85,stone);this.sphere(g,0,5.56,-.15,.16,gold,true);this.ring(g,0,5.56,-.15,.34,glow,.018);
  }else{
   this.box(g,0,3.75,-.1,2.5,1.3,1.7,'#4c6c76');this.box(g,0,3.8,.77,2.25,.67,.035,glow,true);this.box(g,0,4.46,-.1,2.9,.16,2,gold);this.box(g,0,4.32,.93,2.7,.05,.04,'#ff6b5b',true);
   this.cylinder(g,-1,4.95,-.6,.04,1.1,stone);this.sphere(g,-1,5.58,-.6,.12,gold,true);this.box(g,-.7,5.25,-.6,.6,.3,.025,'#d9cfac');
   this.lantern(g,-1.2,2.5,d/2+.5);this.lantern(g,1.2,2.5,d/2+.5);this.box(g,0,3.2,1.1,3.6,.06,.04,glow,true);
  }
  this.sign(g,blue?'CHIEF / SOVEREIGN SANCTUARY':'CHIEF / PRIVATE SANCTUARY',0,2.68,d/2+.22,2.9,gold);
  g.traverse(o=>o.userData.home='chief');
 }
 private room(r:Resident){
  const g=new T.Group();g.userData.interior=r.id;g.position.set(r.x,0,r.z);this.world.add(g);
  const shadow=state.theme==='shadow';this.box(g,0,.12,0,r.width+.25,.24,r.depth+.25,shadow?'#6a7e9a':'#9caa9a');
  this.box(g,.2,.58,-.22,.82,.07,.43,shadow?'#334e7b':'#304c5b');for(const x of [-.12,.52])this.box(g,x,.33,-.22,.06,.5,.3,'#233849');
  this.box(g,.2,.87,-.4,.62,.39,.04,shadow?'#a4dcff':'#a7d8cb',true);this.box(g,.2,.64,-.12,.4,.02,.16,'#708e9b');
  this.box(g,-r.width*.38,.63,-r.depth*.25,.22,1,.5,shadow?'#3c5683':'#516c70');
  for(let j=0;j<5;j++)this.box(g,-r.width*.38,.3+j*.16,-r.depth*.25,.24,.06,.39,j%2?'#94b8cc':'#b1b7a0');
  if(!shadow){
   this.box(g,0,1,-r.depth/2,r.width,.1+1.65,.06,'#1f3d45');this.box(g,0,1.7,-r.depth/2+.045,r.width*.9,.04,.035,'#65e0e4',true);
   this.box(g,r.width*.35,1.1,-r.depth/2+.08,.19,.95,.05,'#8f6f49');this.box(g,r.width*.35,1.28,-r.depth/2+.115,.1,.22,.015,'#ffc87f',true);
   this.cylinder(g,-.1,.34,.35,.18,.25,'#31505a');this.box(g,-.1,.48,.35,.4,.08,.36,'#5c6b63');
   this.cable(g,new T.Vector3(.4,.6,-.25),new T.Vector3(.5,.1,.3),.07);
   this.box(g,.2,.15,-.15,.65,.015,.8,'#223c48');
  }
  g.traverse(o=>o.userData.interior=r.id);
 }
 /** Instance repeated static primitives by geometry and material. Picking uses separate house volumes. */
 private batchStatic(){this.world.updateMatrixWorld(true);const buckets=new Map<string,T.Mesh[]>();this.world.traverse(o=>{if(o instanceof T.Mesh&&o.geometry&&!(Array.isArray(o.material))){const k=(o.userData.home||'')+':'+(o.userData.interior||'')+o.geometry.uuid+o.material.uuid;if(!buckets.has(k))buckets.set(k,[]);buckets.get(k)!.push(o);}});const batched=new T.Group();
  for(const objects of buckets.values()){const first=objects[0];const mesh=new T.InstancedMesh(first.geometry,first.material,objects.length);mesh.userData={...first.userData};objects.forEach((o,i)=>mesh.setMatrixAt(i,o.matrixWorld));mesh.castShadow=first.castShadow;mesh.receiveShadow=first.receiveShadow;mesh.instanceMatrix.needsUpdate=true;mesh.computeBoundingSphere();batched.add(mesh);}this.world.clear();this.world.add(batched);
 }
 private disposeGroup(g:T.Group){const shared=new Set(Object.values(this.geometries));g.traverse(o=>{if(o instanceof T.Mesh){if(!shared.has(o.geometry))o.geometry.dispose();const materials=Array.isArray(o.material)?o.material:[o.material];for(const m of materials){if(m instanceof T.MeshBasicMaterial&&m.map)m.map.dispose();if(![...this.mats.values()].includes(m as T.MeshStandardMaterial))m.dispose();}}});g.clear();}
 private refresh(){if(!this.renderer)return;const signature=JSON.stringify([state.theme,state.stage,state.roads,state.layoutRevision,state.townId,activeResidents().map(r=>[r.id,r.name,r.x,r.z])]);if(signature===this.signature)return;this.signature=signature;if(this.lastReplay!==state.replay){this.initialTowns.delete(state.townId);this.lastReplay=state.replay;}this.elapsed=0;this.portalUntil=0;this.delivery=null;this.bubbles.clear();this.nextQuip=9;this.completedInitial=false;this.deliveredSerial=state.courierSerial;if(this.sceneTown!==state.townId){this.chiefAt=door(chief);this.sceneTown=state.townId;this.target.set(0,1.5,0);this.angle=.67;this.elevation=Math.atan2(22,28);}this.lastLight=Number.NEGATIVE_INFINITY;this.followId=null;this.endGesture();this.disposeGroup(this.world);this.disposeGroup(this.actors);this.mats.forEach(m=>m.dispose());this.mats.clear();this.glowMats=[];this.walking={};this.agents.clear();for(const p of this.pickers){if(p instanceof T.Mesh){p.geometry.dispose();(p.material as T.Material).dispose();}}this.pickers=[];this.paths=generateRoads(activeResidents(),state.stage).filter(p=>roadResidents.has(p.resident));this.terrain();if(state.theme==='cyber')this.cyberDistrict();
  for(const r of [...activeResidents(),chief]){
   const before=this.world.children.length;
   if(r.id==='chief')this.chiefHome();else if(state.theme==='cyber')this.cyber(r);else this.shadow(r);
   this.world.children.slice(before).filter(g=>!g.userData.keepForCutaway).forEach(g=>g.traverse(o=>o.userData.home=r.id));this.room(r);
   const picker=new T.Mesh(new T.BoxGeometry(r.width,r.height+1,r.depth),new T.MeshBasicMaterial({visible:false}));picker.position.set(r.x,(r.height+1)/2,r.z);picker.userData.resident=r.id;picker.updateMatrixWorld();this.pickers.push(picker);
   const actor=createAgent(r,state.theme);
   if(r.id==='chief'){
    actor.root.scale.setScalar(1.1);
    const ring=new T.Mesh(new T.TorusGeometry(.4,.025,4,32),new T.MeshBasicMaterial({color:'#f8e7aa',depthTest:false,depthWrite:false}));ring.rotation.x=-Math.PI/2;ring.position.y=.04;actor.root.add(ring);
    // Draw the owner after the world, but retain depth between its own facial/body parts.
    let order=30,first=true;
    actor.root.traverse(o=>{if(o instanceof T.Mesh){o.renderOrder=order++;const ms=Array.isArray(o.material)?o.material:[o.material];for(const m of ms){m.depthTest=true;m.depthWrite=!m.transparent;}if(first){o.onBeforeRender=renderer=>renderer.clearDepth();first=false;}}});
   }
   this.agents.set(r.id,actor);this.actors.add(actor.root);
  }
  this.courier=new T.Group();
  this.cylinder(this.courier,0,.32,0,.17,.4,state.theme==='shadow'?'#8ebcec':'#c3b377');this.sphere(this.courier,0,.65,0,.18,'#466d8c');this.box(this.courier,0,.66,.16,.22,.08,.04,'#c3edff',true);this.box(this.courier,0,.37,-.2,.25,.22,.13,'#efe0af');
  for(const x of [-.13,.13])this.sphere(this.courier,x,.12,0,.1,'#193449');this.courier.visible=false;this.actors.add(this.courier);
  this.teleportPortal=new T.Group();this.ring(this.teleportPortal,0,.17,0,.55,'#c2f3ff',.03);this.ring(this.teleportPortal,0,.2,0,.74,'#89b5ff',.02);this.teleportPortal.visible=false;this.actors.add(this.teleportPortal);
  this.batchStatic();this.renderer.shadowMap.needsUpdate=true;this.labelNodes();
 }
 private labelNodes(){const escape=(s:string)=>s.replace(/[&<>\"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','\"':'&quot;',"'":'&#39;'}[c]!));this.labels.innerHTML=[...activeResidents(),chief].map(r=>`<button class="house-label ${state.selected===r.id?'selected':''}" ${r.id==='chief'?'data-inspect-home="chief"':`data-select="${r.id}"`} aria-label="Inspect ${escape(residentName(r))}, ${escape(r.role)}, ${r.activity}"><span class="status-dot ${['Reading','Researching'].includes(r.activity)?'active':''}"></span>${r.id==='chief'?'Sanctuary':`<strong>${escape(residentName(r))}</strong><small>${escape(r.role)}</small>`}</button><button class="agent-label" data-follow="${r.id}" aria-label="Inspect agent ${escape(residentName(r))}">${escape(residentName(r))}</button><div class="speech-bubble" data-bubble="${r.id}" hidden></div>`).join('')+'<div class="speech-bubble courier-bubble" data-bubble="courier" hidden></div>';}
 private resize(){if(!this.renderer)return;const rect=this.host.getBoundingClientRect();this.w=rect.width;this.h=rect.height;if(this.w<1||this.h<1)return;this.renderer.setPixelRatio(Math.min(devicePixelRatio,state.variant==='C'?1.25:1.5));this.renderer.setSize(this.w,this.h,false);this.invalidate();}
 private positionCamera(){const aspect=this.w/this.h;let height=state.variant==='C'?10.8:19.4;if(aspect<1.1)height=30;this.camera.left=-height*aspect/2;this.camera.right=height*aspect/2;this.camera.top=height/2;this.camera.bottom=-height/2;this.camera.zoom=state.zoom;this.camera.updateProjectionMatrix();
  if(this.followId){const a=this.agents.get(this.followId);if(a){
   this.target.set(a.root.position.x,.9,a.root.position.z);
   const obstruction=(angle:number)=>activeResidents().reduce((score,r)=>{const dx=r.x-a.root.position.x,dz=r.z-a.root.position.z,forward=dx*Math.sin(angle)+dz*Math.cos(angle),side=Math.abs(dx*Math.cos(angle)-dz*Math.sin(angle));return score+(forward>.15&&forward<5&&side<(r.width+r.depth)*.33&&r.height+.8>1+forward*.78?1:0);},0);
   if(obstruction(this.angle)>0){const options=Array.from({length:8},(_,i)=>this.angle+(i<=4?i:i-8)*Math.PI/4);options.sort((x,y)=>obstruction(x)-obstruction(y)||Math.abs(x-this.angle)-Math.abs(y-this.angle));this.angle=options[0];}
  }}const distance=Math.hypot(28,22),horizontal=Math.cos(this.elevation)*distance;this.camera.position.set(this.target.x+Math.sin(this.angle)*horizontal,this.target.y+Math.sin(this.elevation)*distance,this.target.z+Math.cos(this.angle)*horizontal);this.camera.lookAt(this.target);this.camera.updateMatrixWorld();
 }
 private updateLighting(){
  const sun=daylight(),shadow=state.theme==='shadow';
  this.renderer.toneMappingExposure=shadow?1.35:1.15;
  this.ambient.intensity=shadow?.65+sun*.75:.65+sun*.8;this.sun.intensity=shadow?.65+sun*1.8:.45+sun*2;
  this.sun.color.set(shadow?'#afc7ff':sun>.4?'#ffd6a3':'#78accc');this.sun.position.set(-10+Math.sin(clockHours()/24*Math.PI*2)*7,10+sun*9,9);
  this.cyanLight.visible=!shadow;this.redLight.visible=!shadow;this.cyanLight.intensity=4+(1-sun)*22;this.redLight.intensity=3+(1-sun)*18;
  this.glowMats.forEach(m=>m.emissiveIntensity=shadow?.35+(1-sun)*.9:.75+(1-sun)*1.35);
  const color=new T.Color(shadow?'#090f20':'#08141c').lerp(new T.Color(shadow?'#273b59':'#506973'),sun*(shadow?.62:.42));this.scene.background=color;this.scene.fog!.color.copy(color);this.renderer.shadowMap.needsUpdate=true;
 }
 private hasMotion(){return performance.now()<this.portalUntil||!!this.delivery||this.bubbles.size>0||!!(state.peek&&residents.some(r=>r.id===state.peek&&!['Resting','Paused','Pausing','Waiting','Needs approval'].includes(r.activity)));}
 private hasWork(){return performance.now()<this.portalUntil||!!this.delivery||residents.some(r=>!r.retired&&!['Resting','Paused','Pausing','Waiting','Needs approval'].includes(r.activity));}
 private animate(dt:number){
  const running=!state.paused&&!state.quiet;if(running)this.elapsed+=dt;
  this.world.traverse(o=>{if(o.userData.home)o.visible=state.peek!==o.userData.home;if(o.userData.interior)o.visible=state.peek===o.userData.interior;});
  for(const r of activeResidents()){
   const a=this.agents.get(r.id)!,road=this.paths.find(p=>p.resident===r.id),working=['Researching','Reading','Writing','Review','Coordinating'].includes(r.activity);
   if(working&&running)a.phase+=dt;
   // One purposeful sample errand. Reading and coordination normally happen indoors.
   let x=r.x,z=r.z+.35,angle=Math.PI,moving=false,location='Inside home';
   const previous=this.walking[r.id];if((r.activity==='Paused'||r.activity==='Pausing'||state.paused)&&previous){x=previous.x;z=previous.z;location=previous.location;angle=a.root.rotation.y;moving=false;}
   a.root.visible=location!=='Inside home'||state.peek===r.id;a.root.position.set(x,0,z);a.root.rotation.y=angle;
   if(r.activity!=='Paused'&&r.activity!=='Pausing'&&!state.paused)poseAgent(a,this.elapsed,moving&&running,r.activity);
   this.walking[r.id]={x,z,moving:moving&&running,location};
  }
  const owner=this.agents.get('chief')!,homeDoor=door(chief),atHome=Math.hypot(this.chiefAt[0]-homeDoor[0],this.chiefAt[1]-homeDoor[1])<.5;
  owner.root.position.set(this.chiefAt[0],0,this.chiefAt[1]);owner.root.visible=true;
  if(running)poseAgent(owner,this.elapsed,false,'Coordinating');
  this.walking.chief={x:this.chiefAt[0],z:this.chiefAt[1],moving:false,location:atHome?'Sanctuary terrace':'Visiting'};
  if(this.teleportPortal){const left=(this.portalUntil-performance.now())/1100;this.teleportPortal.visible=left>0&&!state.quiet&&!state.paused;this.teleportPortal.position.set(this.chiefAt[0],.04,this.chiefAt[1]);this.teleportPortal.scale.setScalar(1+(1-Math.max(0,left))*.3);}
  if(state.courierSerial!==this.deliveredSerial){this.deliveredSerial=state.courierSerial;this.beginDelivery();}
  if(this.courier){this.courier.visible=!!this.delivery;if(this.delivery){const d=this.delivery;if(running)d.elapsed+=dt;const p=pointOnPath(d.points,d.elapsed/d.duration);this.courier.position.set(p.x,.04,p.z);this.courier.rotation.y=p.angle;
   if(d.elapsed>=d.duration){this.delivery=null;this.courier.visible=false;}
  }}
  if(running&&state.chatter&&this.elapsed>=this.nextQuip){this.nextQuip=this.elapsed+14;const lines:Record<string,string[]>={mayor:['Keeping the big picture small.','One useful step at a time.'],scout:['Found a lead. Checking the source.','No rabbit holes today. Probably.'],reader:['Reading the fine print.','Footnotes are a love language.'],writer:['The first draft is allowed to be messy.'],editor:['That adjective has had a good run.'],critic:['Chief, one small decision.']};const candidates=activeResidents().filter(r=>!['Resting','Paused','Pausing'].includes(r.activity));const r=candidates[Math.floor(this.elapsed/14)%candidates.length];if(r){const options=lines[r.id]||['Making room for good work.'];this.speak(r.id,options[Math.floor(this.elapsed/28)%options.length]);}}
  for(const [id,b] of this.bubbles)if(b.until<this.elapsed||!state.chatter||state.quiet)this.bubbles.delete(id);
 }
 private project(x:number,y:number,z:number){const p=new T.Vector3(x,y,z).project(this.camera);return {x:(p.x+1)*this.w/2,y:(1-p.y)*this.h/2,visible:p.z<1&&Math.abs(p.x)<1.1&&Math.abs(p.y)<1.1};}
 syncOverlay(){if(this.ready&&this.agents.size)this.updateLabels();}
 private updateLabels(){for(const r of [...activeResidents(),chief]){
  const label=this.labels.querySelector<HTMLElement>(r.id==='chief'?'[data-inspect-home="chief"]':`[data-select="${r.id}"]`),name=this.labels.querySelector<HTMLElement>(`[data-follow="${r.id}"]`),a=this.agents.get(r.id)!;
  if(label){label.setAttribute('aria-label',r.id==='chief'?'Chief headquarters':`Inspect ${residentName(r)}, ${r.role}, ${r.activity}`);const p=this.project(r.x,.1,r.z+r.depth/2+.65);label.style.left=p.x+'px';label.style.top=(p.y+6)+'px';label.hidden=!p.visible||(state.variant==='C'&&r.id!=='mayor');label.classList.toggle('selected',state.selected===r.id||r.id==='chief'&&state.panel==='chief');label.classList.toggle('attention',r.activity==='Needs approval');label.querySelector('.status-dot')?.classList.toggle('active',['Reading','Researching','Writing','Coordinating','Review'].includes(r.activity));}
  if(name){const p=this.project(a.root.position.x,a.height*a.root.scale.y+.15,a.root.position.z);name.style.left=p.x+'px';name.style.top=p.y+'px';name.hidden=r.id==='chief'||!p.visible||!a.root.visible||state.variant==='C';}
  const bubble=this.labels.querySelector<HTMLElement>(`[data-bubble="${r.id}"]`),b=this.bubbles.get(r.id);
  if(bubble){const outdoors=a.root.visible,p=this.project(outdoors?a.root.position.x:r.x,outdoors?a.height*a.root.scale.y+.4:r.height+1.1,outdoors?a.root.position.z:r.z);bubble.hidden=!b||!p.visible||state.variant==='C';bubble.textContent=b?.text||'';bubble.style.left=p.x+'px';bubble.style.top=p.y+'px';}
 }
  const marker=this.chiefMarker,p=this.project(this.chiefAt[0],2.3,this.chiefAt[1]),bounds=this.host.getBoundingClientRect();
  const edge=p.x<45||p.x>this.w-45||p.y<40||p.y>this.h-50;
  let x=bounds.x+Math.max(45,Math.min(this.w-45,p.x)),y=bounds.y+Math.max(40,Math.min(this.h-50,p.y));
  const panel=document.querySelector('.inspector')?.getBoundingClientRect();
  if(panel&&x+30>panel.left&&x-30<panel.right&&y>panel.top&&y-34<panel.bottom){x=panel.left+40;y=panel.top+48;}
  marker.style.left=x+'px';marker.style.top=y+'px';marker.classList.toggle('at-edge',edge);marker.title=edge?'Chief is outside this view. Click to find.':'Chief. Click to focus.';

 }
 private scheduleClock(){clearTimeout(this.clockTimer);if(this.externallyHidden||document.hidden||state.clockMode==='fixed')return;this.clockTimer=window.setTimeout(()=>{this.lastLight=Number.NEGATIVE_INFINITY;this.invalidate();this.scheduleClock();},state.clockMode==='accelerated'?(state.quiet?10000:1000):30000);}
 private invalidate(){this.host.dataset.cameraMode=state.cameraMode;if(this.navigation.firstElementChild){this.navigation.querySelector('.camera-drag-help')!.textContent=state.cameraMode==='pan'?'Move the point you grabbed':'Rotate and tilt the view';this.navigation.querySelector('summary span')!.textContent=state.cameraMode==='pan'?'Move map':'Orbit map';this.navigation.querySelectorAll<HTMLButtonElement>('[data-camera-mode]').forEach(b=>b.setAttribute('aria-pressed',String(b.dataset.cameraMode===state.cameraMode)));}this.lastLabels=0;clearTimeout(this.timer);if(!this.externallyHidden&&!document.hidden&&this.ready&&this.w>0&&this.h>0)this.frame();}
 private frame(){if(!this.ready||this.externallyHidden||document.hidden)return;const start=performance.now(),dt=Math.min(2,(start-this.last)/1000);this.last=start;const lightState=JSON.stringify([state.clockMode,state.lighting,state.clockOrigin]);if(lightState!==this.lightState||start-this.lastLight>(state.clockMode==='accelerated'?900:30000)){this.lightState=lightState;this.updateLighting();this.lastLight=start;}this.animate(dt);this.positionCamera();this.renderer.render(this.scene,this.camera);if(start-this.lastLabels>90){this.updateLabels();this.lastLabels=start;}this.frames++;const ms=performance.now()-start;this.frameMs+=ms;this.maxFrameMs=Math.max(this.maxFrameMs,ms);
  if(!state.quiet&&!state.paused){if(this.hasMotion())this.timer=window.setTimeout(()=>this.frame(),state.variant==='C'?84:34);else if(state.chatter&&this.hasWork())this.timer=window.setTimeout(()=>this.frame(),2000);}
 }
}
