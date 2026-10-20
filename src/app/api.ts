import {invoke,isTauri} from '@tauri-apps/api/core';
import {listen} from '@tauri-apps/api/event';
import type {World,Profile} from './types';
export const native=isTauri();
export async function call<T>(name:string,args:Record<string,unknown>={}):Promise<T>{
 if(native)return invoke<T>(name,args);
 const response=await fetch(`/city-api/${name}`,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(args)});
 if(!response.ok){const error=await response.json().catch(()=>({message:'Start the native app with npm run desktop, or run npm run preview:runtime for the isolated browser demo.'}));throw error;}
 return response.json();
}
export const snapshot=()=>call<World>('snapshot');
export const profiles=()=>call<Profile[]>('profiles');
export function submit(kind:string,args:Record<string,unknown>={},expectedRevision?:number){return call<{targetId:string|null}>('submit',{command:{schemaVersion:1,id:crypto.randomUUID(),expectedRevision,...args,kind}})}
export async function watch(refresh:()=>void){if(native){return listen('city:revision',refresh)}const timer=setInterval(()=>{if(!document.hidden)refresh()},1000);return()=>clearInterval(timer)}
export function message(error:unknown){if(typeof error==='object'&&error&&'message' in error)return String(error.message);return String(error)}
