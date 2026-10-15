import { residents, state, subscribe, select, update, type Resident } from './state';

type Point = [number,number];
/** Original procedural 2.5D artwork. Static layers are cached; only message couriers animate. */
export class TownScene {
 private canvas=document.createElement('canvas');
 private cache=document.createElement('canvas');
 private ctx=this.canvas.getContext('2d')!;
 private draw=this.cache.getContext('2d')!;
 private w=1; private h=1; private unit=30; private cx=0; private cy=0;
 private dpr=1; private timer=0; private dirty=true; private start=performance.now(); private replay=state.replay;
 private observer:ResizeObserver;
 private labels:HTMLElement;
 private measure={frames:0,total:0,max:0};
 private dragging=false; private moved=false; private pointer:Point=[0,0]; private pan:Point=[0,0];
 constructor(private host:HTMLElement) {
  this.canvas.setAttribute('aria-label','Isometric town. Select homes using the resident buttons.');
  this.canvas.setAttribute('role','img');this.canvas.className='town-canvas';
  this.labels=document.createElement('div');this.labels.className='world-labels';
  host.append(this.canvas,this.labels);
  this.observer=new ResizeObserver(()=>{this.resize();});this.observer.observe(host);
  subscribe(()=>{if(state.replay!==this.replay){this.replay=state.replay;this.start=performance.now();}this.invalidate();});
  document.addEventListener('visibilitychange',()=>{clearTimeout(this.timer);if(!document.hidden)this.invalidate();});
  this.canvas.addEventListener('pointerdown',e=>{this.dragging=true;this.moved=false;this.pointer=[e.clientX,e.clientY];this.canvas.setPointerCapture(e.pointerId);});
  this.canvas.addEventListener('pointermove',e=>{if(!this.dragging)return;const dx=e.clientX-this.pointer[0],dy=e.clientY-this.pointer[1];if(Math.abs(dx)+Math.abs(dy)>2){this.moved=true;this.pan=[this.pan[0]+dx,this.pan[1]+dy];this.pointer=[e.clientX,e.clientY];this.invalidate();}});
  this.canvas.addEventListener('pointerup',e=>{this.dragging=false;if(this.moved)return;const rect=this.canvas.getBoundingClientRect();const x=e.clientX-rect.left,y=e.clientY-rect.top;const hit=[...residents].reverse().find(r=>{const p=this.p(r.x,r.z,.4);return Math.abs(x-p[0])<this.unit*1.6&&y<p[1]+12&&y>p[1]-this.unit*(r.height+1.4);});if(hit)select(hit.id);});
  this.canvas.addEventListener('wheel',e=>{if(!e.ctrlKey&&!e.metaKey)return;e.preventDefault();update(()=>state.zoom=Math.max(.7,Math.min(1.5,state.zoom-e.deltaY*.002)));},{passive:false});
 }
 reset(){this.pan=[0,0];update(()=>state.zoom=1);}
 stats(){return {renderer:'Canvas 2D / cached static layer',...this.measure,meanDrawMs:this.measure.frames?+(this.measure.total/this.measure.frames).toFixed(2):0,canvasPixels:this.canvas.width*this.canvas.height,devicePixelRatio:this.dpr,scheduledFrames:!document.hidden&&!state.quiet&&!state.paused&&performance.now()-this.start<10200,viewport:[this.w,this.h]};}
 private resize(){const rect=this.host.getBoundingClientRect();this.w=rect.width;this.h=rect.height;this.dpr=Math.min(devicePixelRatio,state.variant==='C'?1.5:2);[this.canvas,this.cache].forEach(c=>{c.width=Math.round(this.w*this.dpr);c.height=Math.round(this.h*this.dpr);});this.invalidate();}
 private invalidate(){this.dirty=true;clearTimeout(this.timer);if(!document.hidden)this.frame();}
 private p(x:number,z:number,y=0):Point{return [this.cx+(x-z)*this.unit,this.cy+(x+z)*this.unit*.48-y*this.unit];}
 private poly(points:Point[],fill:string|CanvasGradient,stroke?:string,width=.7){const c=this.draw;c.beginPath();points.forEach(([x,y],i)=>i?c.lineTo(x,y):c.moveTo(x,y));c.closePath();if(fill){c.fillStyle=fill;c.fill();}if(stroke){c.strokeStyle=stroke;c.lineWidth=width;c.stroke();}}
 private line(points:Point[],color:string,width=1,dash:number[]=[]){const c=this.draw;c.beginPath();points.forEach(([x,y],i)=>i?c.lineTo(x,y):c.moveTo(x,y));c.strokeStyle=color;c.lineWidth=width;c.lineJoin='round';c.lineCap='round';c.setLineDash(dash);c.stroke();c.setLineDash([]);}
 private floor(x:number,z:number,w:number,d:number,y:number,color:string,stroke?:string){this.poly([this.p(x-w/2,z-d/2,y),this.p(x+w/2,z-d/2,y),this.p(x+w/2,z+d/2,y),this.p(x-w/2,z+d/2,y)],color,stroke);}
 private box(x:number,z:number,w:number,d:number,y:number,h:number,colors?:string[],outline?:string){const shadow=state.theme==='shadow';const cs=colors|| (shadow?['#22283e','#151b30','#353a53']:['#283947','#172531','#3c4d55']);const a=this.p(x-w/2,z-d/2,y+h),b=this.p(x+w/2,z-d/2,y+h),c=this.p(x+w/2,z+d/2,y+h),d1=this.p(x-w/2,z+d/2,y+h);this.poly([b,c,this.p(x+w/2,z+d/2,y),this.p(x+w/2,z-d/2,y)],cs[1],outline||'#111d2a');const shade=this.draw.createLinearGradient(d1[0],d1[1],d1[0]+8,d1[1]+h*this.unit);shade.addColorStop(0,cs[0]);shade.addColorStop(1,cs[1]);this.poly([d1,c,this.p(x+w/2,z+d/2,y),this.p(x-w/2,z+d/2,y)],shade,outline||'#142430');this.poly([a,b,c,d1],cs[2],outline||'#52606a66');}
 private glow(x:number,z:number,y:number,color:string,r=.6){const p=this.p(x,z,y),c=this.draw;const g=c.createRadialGradient(p[0],p[1],0,p[0],p[1],this.unit*r);g.addColorStop(0,color);g.addColorStop(1,'transparent');c.fillStyle=g;c.beginPath();c.arc(...p,this.unit*r,0,Math.PI*2);c.fill();}
 private ring(x:number,z:number,y:number,r:number,color:string,width=1){const c=this.draw,p=this.p(x,z,y);c.beginPath();c.ellipse(...p,r*this.unit,r*this.unit*.48,0,0,Math.PI*2);c.strokeStyle=color;c.lineWidth=width;c.stroke();}
 private lamp(x:number,z:number,h=1.2){const cyan=state.theme==='shadow'?'#7299ff':'#d4e4b0';this.line([this.p(x,z,.1),this.p(x,z,h)],'#627781',1.4);this.box(x,z,.13,.13,h,.09,['#b9caae','#708795',cyan]);if(state.lighting!=='day')this.glow(x,z,h,'#a4d7dd44',.45);}
 private sign(x:number,z:number,y:number,text:string,color:string,size=7){const c=this.draw,p=this.p(x,z,y);c.save();c.translate(...p);c.transform(1,.48,0,1,0,0);c.font=`600 ${Math.max(5,size*this.unit/31)}px ui-monospace, monospace`;c.fillStyle=color;c.fillText(text,0,0);c.restore();}
 private window(x:number,z:number,y:number,w:number,h:number,side=false,color='#80c8bf'){
 const delta=this.unit<22?.035:0;const points=side?[this.p(x,z-w/2,y+h),this.p(x,z+w/2,y+h),this.p(x,z+w/2,y),this.p(x,z-w/2,y)]:[this.p(x-w/2,z,y+h),this.p(x+w/2,z,y+h),this.p(x+w/2,z,y),this.p(x-w/2,z,y)];this.poly(points,color);if(delta)this.line([points[0],points[1]],'#a4cdd577',.4);
 }
 private windows(r:Resident,h:number,w:number,d:number,base=.25,rows=3){for(let row=0;row<rows;row++){for(let col=0;col<4;col++){const color=(col+row)%3===0?'#b7c97f':(col+row)%2===0?'#5eaaa8':'#344e60';this.window(r.x-w*.35+col*w*.23,r.z+d/2+.01,base+.22+row*(h-.3)/rows,w*.14,.18,false,color);}for(let col=0;col<3;col++)this.window(r.x+w/2+.01,r.z-d*.3+col*d*.28,base+.25+row*(h-.3)/rows,d*.14,.17,true,'#487c91');}}
 private plant(x:number,z:number,y=0){this.box(x,z,.25,.25,y,.2,['#354d51','#223743','#4b6965']);this.line([this.p(x,z,y+.2),this.p(x,z,y+.65)],'#426d68',1.5);for(let j=0;j<4;j++)this.line([this.p(x,z,y+.3+j*.08),this.p(x+(j%2?-.17:.17),z+(j%2?.12:-.12),y+.55+j*.05)],'#578d7b',1.6);}
 private antenna(x:number,z:number,y:number){this.line([this.p(x,z,y),this.p(x,z,y+1.25)],'#7b959e',1);this.line([this.p(x-.18,z,y+.75),this.p(x+.18,z,y+.75)],'#7b959e',1);this.glow(x,z,y+1.25,'#f3947f66',.18);this.box(x,z,.08,.08,y+1.2,.05,['#f4a181','#8b4350','#ffc0a4']);}
 private cyberBuilding(r:Resident){const {x,z,width:w,depth:d}=r;const h=r.height;const sel=state.selected===r.id;
 this.box(x,z,w+.6,d+.6,0,.17,['#26333c','#172630','#3b4850']);
 // Recessed ground markings around each specialist's physical home.
 this.line([this.p(x-w/2-.3,z+d/2+.3,.18),this.p(x+w/2+.3,z+d/2+.3,.18),this.p(x+w/2+.3,z-d/2-.3,.18)],sel?'#d8ef81':'#50626977',sel?1.6:1);
 this.box(x,z,w,d,.17,h);
 this.windows(r,h,w,d,.17,r.id==='mayor'?5:3);
 // Local, role-specific facade treatments keep the small asset kit identifiable.
 if(r.id==='reader'){
  this.window(x,z+d/2+.035,.5,w*.78,h*.58,false,'#385e68');
  for(let j=0;j<5;j++)this.line([this.p(x-w*.39+j*w*.195,z+d/2+.05,.5),this.p(x-w*.39+j*w*.195,z+d/2+.05,h*.58+.5)],'#86bbb180',.8);
  for(let j=0;j<3;j++)this.line([this.p(x-w*.39,z+d/2+.05,.75+j*.37),this.p(x+w*.39,z+d/2+.05,.75+j*.37)],'#8bbdb43b',.6);
 }
 if(r.id==='critic'){
  for(const side of [-1,1])this.box(x+side*(w*.5-.11),z+d/2+.07,.12,.13,.4,h-.35,['#845b4b','#4f3733','#b08a6a']);
  this.box(x-.3,z+d/2+.2,.66,.28,1.1,.15,['#a56f59','#624742','#c39072']);
 }
 if(r.id==='writer')for(let j=0;j<3;j++)this.box(x-.55+j*.5,z-.15,.46,.7,h+.3,.15+j*.08,['#687081','#333f55','#849097']);
 if(r.id==='editor')this.box(x+.05,z+d/2+.23,w*.9,.36,.91,.09,['#6b7e6a','#3b5152','#92a184']);
 // Structural verticals and roof lip.
 for(const t of [-.5,.5])this.line([this.p(x+t*w,z+d/2+.02,.17),this.p(x+t*w,z+d/2+.02,h+.17)],'#75909566',1);
 this.box(x,z,w+.08,d+.08,h+.17,.12,['#40515a','#263a45','#526167']);
 if(r.id==='mayor'){
  this.box(x-.25,z-.2,1.25,.95,h+.29,.65,['#3f4f57','#233947','#5d696b']);
  this.box(x-.25,z-.2,1.32,1.02,h+.94,.08,['#89946b','#456057','#c5d57e']);
  for(let j=0;j<5;j++)this.window(x-.67+j*.22,z+.28,h+.45,.13,.32,false,'#a1d3b2');
  this.antenna(x-.6,z-.45,h+1.02);
  this.sign(x-w*.4,z+d/2+.03,h-.16,'ATLAS // OPS','#d4ec87',8);
  this.box(x-.15,z+d/2+.15,.7,.22,.2,.8,['#151e28','#111b25','#3f5b60']);
  this.window(x-.15,z+d/2+.27,.3,.5,.55,false,'#82bab3');
  this.line([this.p(x-w/2,z-d/2,h+.32),this.p(x+w/2,z-d/2,h+.32)],'#d0df84',2);
 }else if(r.id==='scout'){
  this.box(x-.25,z, .6,.65,h+.3,.5);this.antenna(x+.45,z-.3,h+.3);
  const p=this.p(x-.25,z,h+1);const c=this.draw;c.save();c.translate(...p);c.rotate(-.5);c.beginPath();c.ellipse(0,0,this.unit*.38,this.unit*.16,0,0,Math.PI*2);c.fillStyle='#7b9393';c.fill();c.strokeStyle='#c7d5bc';c.stroke();c.restore();
  this.sign(x-w*.4,z+d/2+.03,h-.16,'SIGNAL 01','#c6d8a7');
 }else if(r.id==='reader'){
  this.box(x+.15,z-.08,w*.7,d*.7,h+.3,.38,['#3a4b50','#283b45','#566a6a']);
  for(let j=0;j<4;j++){this.box(x-.4+j*.27,z-.1,.14,.55,h+.68,.16,['#32545d','#1d3747','#8fb4af']);}
  this.sign(x-w*.4,z+d/2+.03,h-.1,'SHARD ARCHIVE','#83dad1');
  this.line([this.p(x-w/2,z+d/2+.03,.8),this.p(x+w/2,z+d/2+.03,.8)],'#77d2c9',2);
 }else if(r.id==='analyst'){
  this.box(x+.5,z-.35,.6,.65,h+.3,.75,['#2f4556','#1e3246','#536d76']);
  this.ring(x-.3,z+.1,h+.36,.55,'#7cbed0',2);this.ring(x-.3,z+.1,h+.55,.4,'#648b9d');
  this.antenna(x+.5,z-.35,h+1.05);this.sign(x-w*.4,z+d/2+.03,h-.1,'VECTOR LAB','#82b7d9');
 }else if(r.id==='writer'){
  this.box(x-.25,z,1.15,.9,h+.3,.32,['#4c4b59','#2c3546','#717383']);
  this.box(x+.65,z-.4,.22,.22,h+.3,.7,['#3c4650','#26323d','#73808a']);
  this.sign(x-w*.4,z+d/2+.03,h-.12,'PRESS / 04','#dda1ba');
  this.line([this.p(x-w/2,z+d/2,.5),this.p(x+w/2,z+d/2,.5)],'#be789c',2);
 }else if(r.id==='editor'){
  this.box(x-.35,z-.2,.8,.7,h+.3,.45,['#47515a','#283b44','#6b7877']);
  this.plant(x+.5,z+.2,h+.3);this.plant(x+.6,z-.4,h+.3);
  this.sign(x-w*.4,z+d/2+.03,h-.12,'REVISION STUDIO','#d2d9b2');
 }else{
  this.box(x-.3,z-.2,.65,.7,h+.3,.4);this.box(x+.45,z-.3,.24,.28,h+.3,.68);
  this.line([this.p(x-w/2,z+d/2,h-.3),this.p(x+w/2,z+d/2,h-.3)],'#df886b',2);
  this.sign(x-w*.4,z+d/2+.03,h-.08,'REDLINE','#e9a18a');
 }
 // Exposed wall pipes and a rooftop ladder are drawn at the building's actual scale.
 this.line([this.p(x+w/2+.025,z-d*.32,.25),this.p(x+w/2+.025,z-d*.32,h+.23)],'#7b959055',1.4);
 for(let j=0;j<7;j++)this.line([this.p(x+w/2+.06,z-d*.28,.45+j*h/8),this.p(x+w/2+.06,z-d*.12,.45+j*h/8)],'#8ca4a866',.7);
 this.line([this.p(x+w/2+.04,z-d*.12,.25),this.p(x+w/2+.04,z-d*.12,h+.2)],'#8ca4a855',.7);
 // Door, steps, rooftop vents, utility box and street furniture.
 for(let i=0;i<3;i++)this.box(x,z+d/2+.14+i*.13,.5,.14,0,.17-i*.035,['#46545a','#2c3b44','#586569']);
 this.box(x-w/2-.18,z-.35,.22,.5,.18,.5,['#3e535a','#293a44','#688078']);
 for(let i=0;i<4;i++)this.line([this.p(x-w/2-.3,z-.5+i*.1,.48),this.p(x-w/2-.08,z-.5+i*.1,.48)],'#253a43',.8);
 if(r.id==='writer'||r.id==='editor')this.plant(x+w/2+.18,z-.55,.18);
 if(state.lighting!=='day'){this.glow(x,z+d/2,.5,r.id==='critic'?'#dc7b5a22':'#7bbaaf22',1);}
 if(state.stage>0){const dx=r.id==='mayor'?-1.65:w/2+.45;this.box(x+dx,z-.2,.7,.85,.18,.65,['#3b4e58','#243845','#697970']);this.window(x+dx,z+.23,.38,.4,.23,false,'#aac8a0');if(state.stage>1){this.antenna(x+dx,z-.2,.85);this.box(x,z-.2,w*.85,d*.75,h+.3, .3);}}
 }
 private spire(x:number,z:number,y:number,h:number,r=.22){const c=state.theme==='shadow';this.box(x,z,r*2,r*2,y,h,['#30364f','#19213a','#48516c']);this.poly([this.p(x-r,z-r,y+h),this.p(x+r,z-r,y+h),this.p(x+r,z+r,y+h),this.p(x-r,z+r,y+h)],'#515a77');const top=this.p(x,z,y+h+.8);this.poly([this.p(x-r,z+r,y+h),this.p(x+r,z+r,y+h),top],c?'#303d68':'#344455','#6778a3',.5);this.poly([this.p(x+r,z-r,y+h),this.p(x+r,z+r,y+h),top],'#1c2b50','#54669c',.5);this.line([this.p(x+r,z+r,y+.2),this.p(x+r,z+r,y+h)],'#6e93ec',1);}
 private arch(x:number,z:number,y:number,w:number,h:number,fill='#101a31',stroke='#7394c7') {
 const pts=[this.p(x-w/2,z,y),this.p(x-w/2,z,y+h*.66),this.p(x-w*.33,z,y+h*.86),this.p(x,z,y+h),this.p(x+w*.33,z,y+h*.86),this.p(x+w/2,z,y+h*.66),this.p(x+w/2,z,y)];this.poly(pts,fill,stroke,.85);
 this.line([this.p(x,z,y+.07),this.p(x,z,y+h*.91)],'#8ca7db66',.5);
 }
 private shadowBuilding(r:Resident){const {x,z,width:w,depth:d}=r;const h=r.height*.72;const sel=state.selected===r.id;
 this.box(x,z,w+.75,d+.75,0,.23,['#252b43','#151c30','#38425e']);
 this.box(x,z,w+.4,d+.4,.23,.16,['#2a314c','#18213b','#3d4969']);
 this.ring(x,z,.41,Math.max(w,d)*.76,sel?'#a5c6ff':'#405a97',sel?1.5:.7);
 if(r.id==='mayor'){
  this.box(x,z,w,d,.4,h,['#303752','#19223d','#424b69']);
  this.box(x,z-.2,w*.68,d*.7,h+.4,.9,['#3d4260','#212a47','#535b7e']);
  for(const a of [-1,1])for(const b of [-1,1])this.spire(x+a*w*.47,z+b*d*.47,.4,h+.3,.22);
  this.spire(x,z-.25,h+1.3,.8,.3);
  this.window(x,z+d/2+.02,.5,.55,1.5,false,'#162640');
  this.line([this.p(x-.3,z+d/2+.04,.5),this.p(x-.3,z+d/2+.04,1.65),this.p(x,z+d/2+.04,2),this.p(x+.3,z+d/2+.04,1.65),this.p(x+.3,z+d/2+.04,.5)],'#7ca7f5',1.5);
  this.glow(x,z+d/2,1.2,'#5789ed55',1.5);
 }else if(r.id==='scout'){
  this.spire(x,z,.4,h+.2,.5);this.box(x,z,1.4,1.35,h+.6,.25);for(const a of [-1,1])this.spire(x+a*.5,z+.4,h+.85,.18,.1);
 }else if(r.id==='analyst'){
  this.box(x,z,w,d,.4,h*.7);this.ring(x,z,h*.7+.4,.8,'#a8bcf1',1.5);this.ring(x,z,h*.7+.7,.6,'#667fd1',1.5);
  this.spire(x+.7,z-.45,.4,h+.8,.15);this.spire(x-.7,z-.45,.4,h+.8,.15);
  this.glow(x,z,h+1,'#6c7eff66',.8);
 }else if(r.id==='reader'){
  this.box(x,z,w,d,.4,h);this.box(x,z,w+.15,d+.15,h+.4,.2);for(const a of [-1,1])this.spire(x+a*.65,z-.4,.4,h+.4,.17);
  for(let i=0;i<4;i++)this.window(x-w*.32+i*w*.22,z+d/2+.02,.6,.15,h-.4,false,i%2?'#35467c':'#7f9ac9');
 }else{
  this.box(x,z,w,d,.4,h);const top=this.p(x,z-.1,h+1.45);this.poly([this.p(x-w/2-.1,z+d/2+.1,h+.4),this.p(x+w/2+.1,z+d/2+.1,h+.4),top],'#354362','#607399');this.poly([this.p(x+w/2+.1,z-d/2-.1,h+.4),this.p(x+w/2+.1,z+d/2+.1,h+.4),top],'#202b49','#506391');
  for(const a of [-1,1])this.spire(x+a*w*.48,z+d*.48,.4,h*.7,.13);
 }
 // Gothic apertures are recessed into each chamber instead of pasted luminous rectangles.
 this.arch(x,z+d/2+.04,.42,r.id==='mayor'?.54:.42,r.id==='mayor'?1.65:.96,'#0d1931','#759cda');
 for(const side of [-1,1]){
  this.box(x+side*(w/2-.14),z+d/2+.08,.15,.19,.4,h*.68,['#46516b','#25334e','#66738b']);
  if(r.id!=='scout')this.arch(x+side*w*.31,z+d/2+.05,.95,.15,.51,'#647fae','#91b3e0');
 }
 // Stone courses, lintels, and narrow blue cloth banners establish scale.
 for(let row=0;row<4;row++)this.line([this.p(x-w*.39,z+d/2+.03,.5+row*.32),this.p(x-w*.15,z+d/2+.03,.5+row*.32)],'#7280a92b',.5);
 this.line([this.p(x-w/2,z+d/2,h*.7+.4),this.p(x+w/2,z+d/2,h*.7+.4)],'#7898c04d',.7);
 if(r.id==='mayor'||r.id==='reader'){
  for(const side of [-1,1]){const bx=x+side*w*.31;this.poly([this.p(bx-.09,z+d/2+.14,h+.25),this.p(bx+.09,z+d/2+.14,h+.25),this.p(bx+.09,z+d/2+.14,h-.55),this.p(bx,z+d/2+.14,h-.67),this.p(bx-.09,z+d/2+.14,h-.55)],'#334d88','#86a3da66');}
 }
 if(r.id==='writer'||r.id==='editor'||r.id==='critic'){
  for(let j=0;j<5;j++){const rx=x-w*.45+j*w*.22;this.line([this.p(rx,z+d/2+.09,h+.41),this.p(x,z-.1,h+1.44)],'#8fa2c72d',.7);}
 }

 for(let i=0;i<4;i++)this.box(x,z+d/2+.14+i*.13,.65,.14,0,.36-i*.07,['#39445f','#26314d','#4c5c7b']);
 // Rune inscription, specialist's object and a pair of bounded blue braziers.
 this.sign(x-w*.3,z+d/2+.05,h+.18,['mayor','reader'].includes(r.id)?'◇ Ⅰ Ⅲ ◇':'Ⅰ ◇ Ⅱ','#b7cdf7',8);
 for(const a of [-1,1]){const bx=x+a*(w/2+.18),bz=z+d/2+.2;this.box(bx,bz,.14,.14,.12,.55);const p=this.p(bx,bz,.92);this.poly([[p[0]-3,p[1]+4],[p[0]-2,p[1]-4],[p[0],p[1]-9],[p[0]+3,p[1]+1]],'#a6ccff');this.glow(bx,bz,.8,'#658dff55',.55);}
 if(state.stage>0){this.box(x-w/2-.38,z,.6,.9,.4,.7);this.spire(x-w/2-.38,z-.2,1.1,.4,.13);if(state.stage>1){this.box(x,z-.2,w*.7,d*.7,h+.4,.7);this.spire(x,z-.2,h+1.1,.4,.2);}}
 }
 private roadPath(r:Resident):Point[]{return [[0,1],[r.x*.55,1],[r.x*.55,r.z+.9],[r.x,r.z+.9]];}
 private backdrop(){const c=this.draw;const shadow=state.theme==='shadow',day=state.lighting==='day',dusk=state.lighting==='dusk';const g=c.createRadialGradient(this.w*.5,this.h*.5,20,this.w*.5,this.h*.5,this.w*.7);g.addColorStop(0,shadow?(day?'#222b48':'#131e33'):day?'#344248':dusk?'#34343c':'#1c2932');g.addColorStop(1,shadow?'#0b101e':day?'#18222a':'#0d131c');c.fillStyle=g;c.fillRect(0,0,this.w,this.h);
 // Faint neighboring architecture gives the playable block a district context.
 c.save();c.globalAlpha=shadow?.24:.32;
 if(!shadow){for(let j=0;j<16;j++){const x=-13+(j%8)*3.3,z=j<8?-11:10.5;const hh=1.1+((j*13)%7)*.45;this.box(x,z,1.6+(j%3)*.3,1.5,0,hh,['#24323e','#18242f','#33434c']);if(j%2===0)this.line([this.p(x-.7,z+.76,hh*.6),this.p(x+.6,z+.76,hh*.6)],'#6e99a266',1);}}
 else{for(let j=0;j<12;j++){const angle=j/12*Math.PI*2;const x=Math.cos(angle)*11.8,z=Math.sin(angle)*10.2;const hh=1+((j*7)%5)*.4;
   const a=this.p(x-.8,z+.3,-.9),b=this.p(x+.8,z+.3,-.9),top=this.p(x-.15,z,hh);this.poly([a,b,top],'#3b4768','#8197c633');this.poly([b,this.p(x+.6,z-.5,-.8),top],'#263855','#7187c433');
   if(j%3===0)this.spire(x,z,hh,.6,.1);
  }}
 c.restore();
 const fog=c.createLinearGradient(0,0,0,this.h);fog.addColorStop(0,'#0c111a99');fog.addColorStop(.4,'transparent');fog.addColorStop(1,'#0c111a55');c.fillStyle=fog;c.fillRect(0,0,this.w,this.h);
 }
 private terrain(){const shadow=state.theme==='shadow';
 const p=this.p(0,0,-.4);const c=this.draw;c.save();c.translate(p[0]+15,p[1]+35);c.scale(1,.45);const g=c.createRadialGradient(0,0,50,0,0,this.unit*15);g.addColorStop(0,'#00000099');g.addColorStop(1,'transparent');c.fillStyle=g;c.beginPath();c.arc(0,0,this.unit*15,0,Math.PI*2);c.fill();c.restore();
 if(shadow){
  const rim:Point[]=[[-7.8,-4.5],[-6.1,-6.1],[4.5,-6.1],[7.8,-3.6],[7.8,4.3],[5.1,6.5],[-5.8,6.5],[-7.8,4.6]];
  for(let j=0;j<rim.length;j++){const a=rim[j],b=rim[(j+1)%rim.length],mid:Point=[(a[0]+b[0])/2,(a[1]+b[1])/2];
   this.poly([this.p(...a,-.1),this.p(...b,-.1),this.p(b[0]*.94,b[1]*.94,-1.15),this.p(a[0]*.95,a[1]*.95,-1.4)],j%2?'#151d33':'#1c2740','#50648e22');
   this.poly([this.p(...a,-.1),this.p(...mid,-1.9),this.p(a[0]*.95,a[1]*.95,-1.4)],'#10192c','#7086b414');
  }
  this.poly(rim.map(([x,z])=>this.p(x,z,-.1)),'#2c344b','#6e86af77');
  this.poly(rim.map(([x,z])=>this.p(x*.98,z*.98,0)),'#2c3448','#60779d44');
  for(let j=0;j<6;j++)this.box(-1.4,6.2+j*.14,2.2,.18,-.1-j*.09,.1,['#26344d','#182740','#4b5d78']);
 }else{
  this.box(0,.25,15.9,13,-.75,.65,['#1c2b35','#12232d','#303c44']);
  this.box(0,.25,15.6,12.7,-.1,.1,['#405159','#233943','#36434a']);
 }
 for(let i=0;i<100;i++){const x=((i*47)%151)/10-7.5,z=((i*31)%121)/10-5.8;this.floor(x,z,.15+(i%4)*.1,.1+(i%3)*.1,.01,shadow?'#4b587322':'#82938a16');}
 if(shadow){for(let x=-7;x<=7;x+=1.4)for(let z=-5.6;z<=5.6;z+=1.4){if(Math.abs(x)>5.5&&Math.abs(z)>4)continue;this.floor(x,z,1.36,1.36,.01,'transparent','#7287b012');}}
 else{for(let x=-7;x<8;x+=2)this.line([this.p(x,-6,.03),this.p(x,6.5,.03)],'#80949b0e',.7);for(let z=-6;z<7;z+=2)this.line([this.p(-7.6,z,.03),this.p(7.6,z,.03)],'#80949b0e',.7);}
 for(const r of residents.slice(1)){
  const road=this.roadPath(r).map(([x,z])=>this.p(x,z,.04));this.line(road,shadow?'#172037':'#1b2a33',this.unit*.63);
  this.line(road,shadow?'#44577e':'#52636a',this.unit*.5);
  this.line(road,shadow?'#243352':'#263944',this.unit*.43);
  if(state.roads)this.line(road,shadow?'#6d97de88':'#b7ca7a66',1,[3,5]);
  if(state.selected===r.id)this.line(road,shadow?'#9fc2ff':'#d6eb8f',1.5);
 }
 if(shadow){
  this.ring(0,1,.04,2.05,'#93b2e73a',1);this.ring(0,1,.04,2.22,'#7e9fd63a',.7);this.ring(0,1,.04,1.65,'#a4bced30',.5);
  for(let j=0;j<16;j++){const a=j/16*Math.PI*2;this.line([this.p(Math.cos(a)*1.7,1+Math.sin(a)*1.7,.055),this.p(Math.cos(a)*2,1+Math.sin(a)*2,.055)],'#94b7ef55',1);}
 }
 // Central message junction and a mature transit link.
 this.ring(0,1,.055,1.1,shadow?'#7b9edc88':'#b1bf8f77',1.2);this.ring(0,1,.06,.85,shadow?'#516fab':'#74898677',.8);
 this.floor(0,1,.38,.38,.07,shadow?'#8aacf4':'#adbd82');
 if(!shadow){for(let j=0;j<4;j++)this.floor(-1.1+j*.19,2.8,.08,.65,.06,'#c0c6a66b');
 for(let j=0;j<9;j++)this.floor(7.5,-4.6+j*.27,.22,.13,.02,j%2?'#242d35':'#b3b36d');
 this.sign(-5.6,6.51,-.33,'AFTERLIGHT / DISTRICT 01','#8ca4a0',9);
 this.line([this.p(-7.5,6.6,-.3),this.p(-3.5,6.6,-.3)],'#89c3b977',1.4);
 }else{for(let j=0;j<8;j++){const a=j/8*Math.PI*2;this.floor(Math.cos(a)*1.1,1+Math.sin(a)*1.1,.12,.12,.08,'#a0c4ff');}this.sign(-4.1,6.51,-.33,'THE SHADOW DOMAIN','#9fbcf2',9);}
 }
 private details(){const shadow=state.theme==='shadow';
 for(const [x,z] of [[-6.6,-4.4],[-2.7,-5.5],[3,-5.4],[6.7,-3.7],[6.6,1.2],[3.2,5.6],[-2.7,5.6],[-6.4,2.8],[-6.5,-.8],[2.1,.5]] as Point[]){if(shadow){this.spire(x,z,0,.6,.12);this.glow(x,z,.7,'#759fff44',.4);}else this.lamp(x,z,1);}
 if(!shadow){
  for(const [x,z] of [[-6.7,4.5],[5.8,4.7],[-6.4,-3.8],[6,-4.6]] as Point[]){this.box(x,z,.55,.6,0,.17,['#354749','#26393d','#587072']);for(let i=0;i<3;i++)this.plant(x+(i-1)*.15,z+(i%2)*.12,.18);}
  // Service ducts and a compact parked runner vehicle.
  this.box(5.9,3.8,.6,1.15,.1,.35,['#4d6170','#253d51','#788b91']);this.box(5.9,3.8,.48,.5,.45,.25,['#294659','#162c41','#506a7b']);this.window(5.9,4.06,.5,.35,.13,false,'#92b8b0');
  for(let j=0;j<4;j++)this.box(-6.9,1.3+j*.3,.35,.23,.05,.25,['#546058','#303c3e','#86907b']);
 }else{
  for(const [x,z] of [[-6.8,3.7],[6.5,3.9],[-5.7,-5.4],[5.8,-4.6]] as Point[]){this.box(x,z,.65,.65,0,.16);this.spire(x,z,.16,.6,.15);this.ring(x,z,.9,.25,'#829ff4',1);}
 }
 }
 private actor(r:Resident){if(r.activity==='Resting')return;const x=r.x-.55,z=r.z+r.depth/2+.65;const shadow=state.theme==='shadow';const c=this.draw;const p=this.p(x,z,.05);c.fillStyle='#00000066';c.beginPath();c.ellipse(...p,this.unit*.16,this.unit*.07,0,0,Math.PI*2);c.fill();
 this.box(x,z,.13,.11,.12,.32,shadow?['#1a2038','#10172b','#334574']:['#718783','#2a3b4a','#95b4ab']);const head=this.p(x,z,.54);c.fillStyle=shadow?'#7399e9':'#d7c5ac';c.beginPath();c.arc(...head,this.unit*.065,0,Math.PI*2);c.fill();
 if(shadow)this.poly([this.p(x-.12,z,.14),this.p(x+.12,z,.14),this.p(x,z,.5)],'#19213a');
 if(r.activity==='Reading')this.window(x+.16,z+.15,.27,.18,.13,false,'#9dd2cf');
 if(r.activity==='Needs approval'){this.glow(x,z,.85,'#e5ac5d33',.4);const p=this.p(x,z,.9);c.fillStyle='#edb36e';c.font=`bold ${this.unit*.35}px sans-serif`;c.textAlign='center';c.fillText('!',...p);c.textAlign='left';}
 }
 private build(){const t=performance.now();const c=this.draw;c.setTransform(this.dpr,0,0,this.dpr,0,0);
 const compact=state.variant==='C';this.unit=Math.min(this.w/(compact?18:32),this.h/(compact?11:19.3))*state.zoom;
 this.cx=this.w*(state.variant==='A'?.49:.5)+this.pan[0];this.cy=this.h*(compact?.71:.61)+this.pan[1];
 this.backdrop();this.terrain();this.details();
 const shadow=state.theme==='shadow';
 for(const r of [...residents].sort((a,b)=>(a.x+a.z)-(b.x+b.z))){
  const xx=r.x,zz=r.z,hh=r.height;this.poly([this.p(xx-.5,zz+.5,.06),this.p(xx+.8,zz+.5,.06),this.p(xx+hh*.8+.8,zz+hh*.22,.06),this.p(xx+hh*.8-.5,zz+hh*.22,.06)],'#070e1944');
  if(shadow)this.shadowBuilding(r);else this.cyberBuilding(r);this.actor(r);
 }
 if(state.lighting==='dusk'){c.fillStyle=shadow?'#7e69b309':'#ec936e0d';c.fillRect(0,0,this.w,this.h);}else if(state.lighting==='day'){c.fillStyle=shadow?'#b3c9ef0b':'#e7f1ca12';c.fillRect(0,0,this.w,this.h);}
 this.labels.innerHTML=residents.map(r=>{const p=this.p(r.x,r.z+r.depth/2+.9,.02);const active=['Reading','Researching'].includes(r.activity);return `<button class="house-label ${state.selected===r.id?'selected':''} ${r.activity==='Needs approval'?'attention':''} ${compact&&r.id!=='mayor'?'compact-label':''}" style="left:${p[0]}px;top:${p[1]+3}px" data-select="${r.id}" aria-label="Inspect ${r.name}, ${r.role}, ${r.activity}"><span class="status-dot ${active?'active':''}"></span>${r.role}${r.activity==='Needs approval'?'<span class="label-alert">!</span>':''}</button>`;}).join('');
 this.dirty=false;this.measure.max=Math.max(this.measure.max,performance.now()-t);
 }
 private courier(elapsed:number,idx:number){const r=residents[idx===0?1:idx===1?2:6];const duration=3.2;const offset=idx*3.2;const t=Math.max(0,Math.min(1,(elapsed-offset)/duration));if(elapsed<offset||elapsed>offset+duration)return;
 const path=this.roadPath(r);const tt=idx%2? t:1-t;const section=Math.min(2,Math.floor(tt*3));const f=tt*3-section;const a=path[section],b=path[section+1];const x=a[0]+(b[0]-a[0])*f,z=a[1]+(b[1]-a[1])*f;const c=this.ctx;const p=this.p(x,z,state.theme==='shadow'?.15:.48);const floor=this.p(x,z,.06);
 c.save();c.fillStyle='#00000055';c.beginPath();c.ellipse(...floor,this.unit*.23,this.unit*.09,0,0,Math.PI*2);c.fill();
 const color=state.theme==='shadow'?'#94b9ff':'#d7e89b';c.shadowColor=color;c.shadowBlur=8;
 if(state.theme==='shadow'){c.beginPath();c.moveTo(p[0],p[1]-9);c.lineTo(p[0]+4,p[1]+4);c.lineTo(p[0]-4,p[1]+4);c.closePath();c.fillStyle='#4b67a5';c.fill();c.fillStyle=color;c.fillRect(p[0]-1,p[1]-7,2,2);}else{c.strokeStyle=color;c.lineWidth=1.4;c.beginPath();c.moveTo(p[0]-7,p[1]-2);c.lineTo(p[0]+7,p[1]+2);c.moveTo(p[0]-7,p[1]+2);c.lineTo(p[0]+7,p[1]-2);c.stroke();c.fillStyle='#c5d2af';c.fillRect(p[0]-3,p[1]-2,6,5);}
 c.restore();
 }
 private frame(){if(document.hidden)return;const t=performance.now();if(this.dirty)this.build();this.ctx.setTransform(this.dpr,0,0,this.dpr,0,0);this.ctx.clearRect(0,0,this.w,this.h);this.ctx.drawImage(this.cache,0,0,this.w,this.h);const elapsed=(performance.now()-this.start)/1000;
 if(!state.quiet&&!state.paused)for(let j=0;j<3;j++)this.courier(elapsed,j);
 this.measure.frames++;this.measure.total+=performance.now()-t;
 if(!state.quiet&&!state.paused&&elapsed<10.2){clearTimeout(this.timer);this.timer=window.setTimeout(()=>this.frame(),state.variant==='C'?84:42);}
 }
}
