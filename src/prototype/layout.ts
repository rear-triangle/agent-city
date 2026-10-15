import type { Resident } from './state';
export type Point=[number,number];
export interface GeneratedRoad { resident:string; points:Point[]; cells:number }
const step=.4, extent=11, n=Math.round(extent*2/step)+1;
const cell=(p:Point):Point=>[Math.round((p[0]+extent)/step),Math.round((p[1]+extent)/step)];
const world=(p:Point):Point=>[p[0]*step-extent,p[1]*step-extent];
const key=(p:Point)=>`${p[0]},${p[1]}`;
export function door(r:Resident):Point{return [r.x,r.z+r.depth/2+.65];}
/** Geometry is generated from footprints. This does not create communication permission. */
export function generateRoads(people:Resident[],stage=0):GeneratedRoad[]{
 const blocked=new Set<string>(),used=new Set<string>();
 for(const r of people)for(let ix=0;ix<n;ix++)for(let iz=0;iz<n;iz++){const [x,z]=world([ix,iz]);if((Math.abs(x-r.x)<r.width/2+.3&&Math.abs(z-r.z)<r.depth/2+.28)||(stage>0&&Math.abs(x-(r.x-r.width/2-.5))<.7&&Math.abs(z-(r.z-.2))<.9))blocked.add(key([ix,iz]));}
 const hub=cell([0,1.75]);
 return people.filter(r=>r.id!=='mayor').map(r=>{
  const start=cell(door(r)),end=hub,open:Point[]=[start],cost=new Map([[key(start),0]]),prev=new Map<string,Point>(),closed=new Set<string>();
  let found=false;
  while(open.length){open.sort((a,b)=>(cost.get(key(a))!+Math.abs(a[0]-end[0])+Math.abs(a[1]-end[1]))-(cost.get(key(b))!+Math.abs(b[0]-end[0])+Math.abs(b[1]-end[1])));const cur=open.shift()!,ck=key(cur);if(closed.has(ck))continue;if(ck===key(end)){found=true;break;}closed.add(ck);
   for(const [dx,dz] of [[1,0],[0,1],[-1,0],[0,-1]]){const next:Point=[cur[0]+dx,cur[1]+dz],nk=key(next);if(next.some(c=>c<0||c>=n)||(blocked.has(nk)&&nk!==key(start))||closed.has(nk))continue;const nextCost=cost.get(ck)!+(used.has(nk)?.68:1);if(nextCost<(cost.get(nk)??Infinity)){cost.set(nk,nextCost);prev.set(nk,cur);open.push(next);}}
  }
  if(!found)return {resident:r.id,points:[],cells:0};
  const cells:Point[]=[end];let current=end;while(key(current)!==key(start)){current=prev.get(key(current))!;cells.push(current);}cells.reverse();cells.forEach(p=>used.add(key(p)));
  const points=cells.filter((p,i)=>i===0||i===cells.length-1||(p[0]-cells[i-1][0])!==(cells[i+1][0]-p[0])||(p[1]-cells[i-1][1])!==(cells[i+1][1]-p[1])).map(world);
  return {resident:r.id,points,cells:cells.length};
 });
}
export function pointOnPath(path:Point[],fraction:number):{x:number;z:number;angle:number}{
 if(path.length<2)return {x:path[0]?.[0]||0,z:path[0]?.[1]||0,angle:0};
 const lengths=path.slice(1).map((p,i)=>Math.hypot(p[0]-path[i][0],p[1]-path[i][1]));let left=Math.max(0,Math.min(1,fraction))*lengths.reduce((a,b)=>a+b,0);
 for(let i=0;i<lengths.length;i++){if(left<=lengths[i]||i===lengths.length-1){const t=lengths[i]?left/lengths[i]:0,a=path[i],b=path[i+1];return {x:a[0]+(b[0]-a[0])*t,z:a[1]+(b[1]-a[1])*t,angle:Math.atan2(b[0]-a[0],b[1]-a[1])};}left-=lengths[i];}return {x:0,z:0,angle:0};
}
