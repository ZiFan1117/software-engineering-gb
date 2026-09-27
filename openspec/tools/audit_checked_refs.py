# -*- coding: utf-8 -*-
'''反假勾审计（v3）——`cover-*` 的 10.1 用它核「已勾条目引用的断言名与路径是否真实存在」。

用法：`python openspec/tools/audit_checked_refs.py`（**在仓内任意 cwd 下都能跑**）

反假勾审计（v3，**重写**）：把 `cover-*` 里**已勾**条目引用的断言名与路径抽出来，核它们在仓内真实存在。
设计要点（v1／v2 的教训都写在这里）：
 · v1 误报 12 条：**把缩写当全名**比对（`d02` vs 真名 `d02_…`）⇒ 改**前缀匹配**；
 · v2 的"块级赦免"更坏：一个块里提一句"不存在"，就把该块**所有**引用都放过（连明明存在的 `policy.json` 也赦免）⇒
   那是**遮羞布**。⇒ v3 一律**按"这条引用所在的那一行"**裁定，且**逐条打印排除原因**（不静默放过）。
排除面只有四种，判据写死在代码里、可复核。
'''
import io, re
from pathlib import Path

# ★ 仓根按**脚本自身位置**推（本文件在 `openspec/tools/` 下 ⇒ 仓根＝上两级）；
#   原版写死了 `D:\Code\...`，那是「闸在版本控制之外」的同族问题（skill §九）。
R = Path(__file__).resolve().parent.parent.parent
COV = R / 'openspec/changes/cover-unimplemented-capabilities/tasks.md'
t = io.open(COV, encoding='utf-8', newline='').read()

fn_names = set()
for p in list((R / 'world-core/src').rglob('*.rs')) + list((R / 'world-core/tests').rglob('*.rs')):
    fn_names |= set(re.findall(r'fn\s+([a-z][a-z0-9_]*)',
                               io.open(p, encoding='utf-8', errors='replace').read()))

OUTSIDE = ('audit_checked_refs.py', 'audit_refs_v3.py', 'build_specmap.py', 'build_html.py', 'sync-vm.ps1',
           'push-vm.ps1', 'final_verify.ps1', 'signoff.py', 'check.sh', 'group6_verify.py', 'trace_matrix.py')
NEG = ('不存在', '（无此 delta）', '无此文件', '零命中')
EXPL = ('简称', '内部编号', '真名是', '真名', '映射到')

SHA = re.compile(r'^[0-9a-f]{7,40}$')
blocks = re.split(r'(?m)^(?=- \[[ x]\] \d+\.\d+)', t)
rows = []
for b in blocks:
    m = re.match(r'- \[([ x])\] (\d+\.\d+)', b)
    if m:
        rows.append((m.group(2), m.group(1) == 'x', b))

bad, excl = [], []
for num, checked, blk in rows:
    if not checked:
        continue
    lines = blk.split('\n')
    refs, paths = set(), set()
    for m in re.finditer(r'`([^`]+)`', blk):
        tok = m.group(1).strip()
        if SHA.match(tok):
            continue
        if '::' in tok:
            cand = tok.split('::')[-1].strip()
            if re.fullmatch(r'[a-z][a-z0-9_]*', cand):
                refs.add(cand)
        elif re.fullmatch(r'[a-z][a-z0-9_]*\d{2}[a-z0-9_]*', tok):
            refs.add(tok)
        if re.search(r'\.(rs|py|sh|md|json|csv)$', tok):
            paths.add(tok)

    def line_of(tok):
        return [ln for ln in lines if '`%s`' % tok in ln]

    for n in sorted(refs):
        if n in fn_names or any(x.startswith(n + '_') for x in fn_names):
            continue
        ls = line_of(n)
        why = None
        if any(w in ln for ln in ls for w in EXPL):
            why = '该行是「简称／内部编号 → 真名」的映射说明'
        elif any(w in ln for ln in ls for w in NEG):
            why = '该行在声明"它不存在"'
        if why:
            excl.append('%s ← 名字 `%s`：%s' % (num, n, why))
        else:
            bad.append((num, '名字 `%s`' % n, '；'.join(ls)[:110]))

    for p in sorted(paths):
        cands = [R / p, R / 'world-core' / p, R / 'openspec' / p,
                 R / 'openspec/changes/cover-unimplemented-capabilities' / p]
        if any(c.exists() for c in cands) or any((R / 'world-core').rglob(Path(p).name)) \
                or any((R / 'openspec').rglob(Path(p).name)):
            continue
        ls = line_of(p)
        why = None
        if '{' in p and '}' in p:
            stem, rest = p.split('{', 1)
            body, tail = rest.split('}', 1)
            cs = [stem + x.strip() + tail for x in body.split(',')]
            if all((R / c).exists() or (R / 'world-core' / c).exists()
                   or any((R / 'world-core').rglob(Path(c).name)) for c in cs):
                why = '花括号路径，展开后 %d 个都在' % len(cs)
        if why is None and (Path(p).name in OUTSIDE or ' ' in p):
            why = '仓外工具名或命令串'
        if why is None and any(w in ln for ln in ls for w in NEG):
            why = '该行在声明"它不存在"'
        if why:
            excl.append('%s ← 路径 `%s`：%s' % (num, p, why))
        else:
            bad.append((num, '路径 `%s`' % p, '；'.join(ls)[:110]))

print('=== 已勾条目里「引用对不上」的（逐条）===')
for num, what, ctx in bad:
    print('  [对不上] %s：%s' % (num, what))
    print('            出处行：%s' % ctx)
print('\n=== 被排除的引用（逐条打印原因，不静默放过）===')
for e in excl:
    print('  ' + str(e)[:150])
print('\n===== 汇总 =====')
print('已勾 %d 条；引用对不上 = %d 条；被排除 = %d 条（原因见上）'
      % (sum(1 for _, c, _ in rows if c), len(bad), len(excl)))
