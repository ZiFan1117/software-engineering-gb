# -*- coding: utf-8 -*-
"""规格计数审计（**仓内可复算**，取代仓外那份 `D:\\Code\\_specmap\\count_13.py`）。

数什么（两类用途）：
  · `fc-2026-003` 的 **`1.3`**：逐能力数 `### Requirement` / `#### Scenario` / `- **证据` 三类行
    ——「改前改后逐能力计数相等」就按这三列判；
  · `fc-2026-003` 的 **`1.4`**：数"长 Requirement"（标题到下一个 `####`／`###` 之间 > 500 **字符**，含空白）。

用法：
    python world-core/tools/spec_length_audit.py          # 打表（并自动与改前快照比对）
    python world-core/tools/spec_length_audit.py --json   # 打 JSON（供账/别的工具引用；**数值现取，不手抄**）

skill §九：「闸在版本控制之外等于没有闸」——故本文具在仓内。
"""
import io
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SPECS = ROOT / 'openspec' / 'specs'
SNAPSHOT = ROOT / 'openspec' / 'changes' / 'fc-2026-003-doc-consolidation' / 'spec-counts-before.json'

COLS = ('Requirement', 'Scenario', '证据行', '长Requirement(>500字)')


def measure() -> dict:
    out = {}
    for p in sorted(SPECS.rglob('spec.md')):
        t = io.open(p, encoding='utf-8').read()
        segs = re.split(r'(?m)^### Requirement: ', t)[1:]
        out[p.parent.name] = {
            'Requirement': len(re.findall(r'(?m)^### Requirement: ', t)),
            'Scenario': len(re.findall(r'(?m)^#### Scenario: ', t)),
            '证据行': len(re.findall(r'(?m)^- \*\*证据', t)),
            '长Requirement(>500字)': sum(
                1 for s in segs if len(re.split(r'(?m)^#### |^### ', s, 1)[0]) > 500),
        }
    return out


def main() -> int:
    out = measure()
    if '--json' in sys.argv:
        print(json.dumps(out, ensure_ascii=False))
        return 0
    print('=== 逐能力计数（`1.3` 的验收面 ＋ `1.4` 的长 Requirement）===')
    print('  %-24s %4s %4s %5s %6s' % ('能力', 'Req', 'Scen', '证据行', '长Req'))
    tot = dict.fromkeys(COLS, 0)
    for cap, d in out.items():
        print('  %-24s %4d %4d %5d %6d' % (cap, d['Requirement'], d['Scenario'], d['证据行'], d['长Requirement(>500字)']))
        for k in COLS:
            tot[k] += d[k]
    print('  %-24s %4d %4d %5d %6d' % ('合计', tot['Requirement'], tot['Scenario'], tot['证据行'], tot['长Requirement(>500字)']))
    if SNAPSHOT.exists():
        try:
            before = json.loads(SNAPSHOT.read_text(encoding='utf-8'))
            three = COLS[:3]  # `1.3` 只管这三列：标题／Scenario／证据行
            bad = [c for c in out if c in before and any(out[c][k] != before[c].get(k) for k in three)]
            drift = [c for c in out if c in before and out[c][COLS[3]] != before[c].get(COLS[3])]
            print('  —— 与改前快照比对：`1.3` 只比前 3 列（标题／Scenario／证据行）——')
            print('     `1.3` 面不符的能力：%s' % ('、'.join(bad) if bad else '无 ✓'))
            print('     `1.4` 面（长 Req 条数）变了的能力：%s（**这是 `1.4` 的成绩，不是 `1.3` 的违规**）'
                  % ('、'.join(drift) if drift else '无'))
        except Exception as e:  # noqa: BLE001
            print('  （快照读取失败：%s）' % e)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
