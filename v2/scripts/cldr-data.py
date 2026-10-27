#!/usr/bin/env python3
"""Derive only English CLDR short-name intervals; keep copyright and input hashes."""
import datetime,hashlib,json,pathlib,xml.etree.ElementTree as ET
r=pathlib.Path(__file__).resolve().parents[1];base=r/'qa/cldr'
meta=ET.parse(base/'common/supplemental/metaZones.xml');english=ET.parse(base/'common/main/en.xml')
def ms(s):return int(datetime.datetime.strptime(s,'%Y-%m-%d %H:%M').replace(tzinfo=datetime.timezone.utc).timestamp()*1000)
intervals={}
for z in meta.findall('.//metazoneInfo/timezone'):
 intervals[z.attrib['type']]=[[ms(m.attrib['from']) if 'from' in m.attrib else None,ms(m.attrib['to']) if 'to' in m.attrib else None,m.attrib['mzone']] for m in z.findall('usesMetazone')]
names={}
for z in english.findall('.//timeZoneNames/metazone'):
 short=z.find('short')
 if short is not None:
  names[z.attrib['type']]={m.tag:m.text for m in short if m.text and m.text!='↑↑↑'}
(r/'resources/cldr-zones.json').write_text(json.dumps({'intervals':intervals,'names':names},indent=2,sort_keys=True)+'\n')
(r/'legal/CLDR-LICENSE').write_bytes((base/'LICENSE').read_bytes())
(r/'legal/cldr-pin.json').write_text(json.dumps({'repository':'https://github.com/unicode-org/cldr','commit':'acd6d88ae493633240e19a87a721076a8a75c310','version':'48','inputs':{str(f.relative_to(base)):hashlib.sha256(f.read_bytes()).hexdigest() for f in base.rglob('*') if f.is_file()}},indent=2)+'\n')
print(len(intervals),len(names))
