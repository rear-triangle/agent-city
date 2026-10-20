import re,sys
from pathlib import Path
lines=[line for line in Path(sys.argv[1]).read_text().splitlines() if 'after-show' in line]
if not lines:
    raise SystemExit('FAIL: no native transition observation')
line=lines[-1]
fields=dict(re.findall(r'(active|key|can_key|ignores_mouse|visible)=(true|false)',line))
expected={'active':'false','key':'false','can_key':'false','ignores_mouse':'true','visible':'true'}
failures={k:{'expected':v,'actual':fields.get(k)} for k,v in expected.items() if fields.get(k)!=v}
print('FAIL' if failures else 'PASS',failures or fields)
raise SystemExit(bool(failures))
