#!/usr/bin/env python3
"""Sample an explicitly identified native app process group without UI automation."""
import argparse, datetime, json, pathlib, statistics, subprocess, time
parser=argparse.ArgumentParser()
parser.add_argument('--pids',type=int,nargs='+',required=True)
parser.add_argument('--seconds',type=int,default=600)
parser.add_argument('--label',required=True)
parser.add_argument('--output',type=pathlib.Path,required=True)
args=parser.parse_args()
if args.seconds<1 or args.seconds>28800:parser.error('Duration must be between 1 second and 8 hours.')
if args.output.exists():parser.error('Choose a new output file.')
def cpu_seconds(value):
    days=0
    if '-' in value:day,value=value.split('-',1);days=int(day)
    result=0
    for part in value.split(':'):result=result*60+float(part)
    return days*86400+result
samples=[];started=time.monotonic();previous=None
while True:
    text=subprocess.check_output(['/bin/ps','-p',','.join(map(str,args.pids)),'-o','pid=,rss=,time=,comm='],text=True)
    rows=[line.split(None,3) for line in text.splitlines() if line.strip()]
    found={int(row[0]) for row in rows}
    if found!=set(args.pids):raise SystemExit('A tracked process exited. This sample cannot be a complete group measurement.')
    at=time.monotonic();cpu=sum(cpu_seconds(row[2])for row in rows);rss=sum(int(row[1])for row in rows)/1024
    sample={'elapsedSeconds':at-started,'rssMiB':rss,'processes':[{'pid':int(row[0]),'rssKiB':int(row[1]),'command':row[3]}for row in rows]}
    if previous:sample['cpuPercent']=max(0,(cpu-previous[1])/(at-previous[0])*100)
    samples.append(sample);previous=(at,cpu)
    if at-started>=args.seconds:break
    time.sleep(min(2,args.seconds-(at-started)))
values=[s['cpuPercent'] for s in samples if 'cpuPercent'in s]
report={'label':args.label,'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'convention':'One logical core is 100 percent. RSS sums every supplied process, including shared pages. Supply the native host, web content, GPU, and networking PIDs. Display state must be independently verified.','durationSeconds':samples[-1]['elapsedSeconds'],'meanCpuPercent':statistics.mean(values),'peakCpuPercent':max(values),'meanRssMiB':statistics.mean(s['rssMiB']for s in samples),'peakRssMiB':max(s['rssMiB']for s in samples),'samples':samples}
args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items()if k!='samples'},indent=2))
