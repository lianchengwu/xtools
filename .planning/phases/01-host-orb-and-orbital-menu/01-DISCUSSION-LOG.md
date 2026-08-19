# Phase 1: Host Orb and Orbital Menu - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-08-19
**Phase:** 1-Host Orb and Orbital Menu
**Areas discussed:** 主球外观, 三球怎么排, 展开收起手感, 拖和点怎么分

---

## 主球外观

| Option | Description | Selected |
|--------|-------------|----------|
| 纯色圆点 | 一块干净的圆，不靠图标认 | |
| 圆里一个简单图标 | 比如工具箱/点阵 | ✓ (then specified as letter x) |
| You decide | 外观由实现定 | |

**User's choice:** ~40px dark disk, light letter x, not translucent, not accent-colored.
**Notes:** Size picked “拇指能点”; color “深色圆 + 浅色图标”; icon “字母 x”.

---

## 三球怎么排

| Option | Description | Selected |
|--------|-------------|----------|
| 正三角 120° | 上、右下、左下 | |
| 主球右侧竖排 | 像展开的菜单条 | |
| 扇形摊在主球上方 | 不挡主球下面 | ✓ |

Marks: clock / `{}` / 文 on-disk, no full labels. Function orbs ~32px.

**User's choice:** Fan above; glyph on disk; slightly smaller than main.
**Notes:** Rejected side labels and color-only identification.

---

## 展开收起手感

| Option | Description | Selected |
|--------|-------------|----------|
| 立刻出现 | 无动画 | |
| 从主球短促弹出到位 | ~100ms 散开 | ✓ |
| 只点主球才收 | 菜单粘住 | |
| 点功能球以外都收 | 普通菜单 | ✓ (and function-orb click also collapses) |
| 悬停展开 | 划过就弹 | |
| 只点击 | 划过不弹 | ✓ |

**User's choice:** Short pop; click-only; any click (outside, function orb, or main toggle) closes.
**Notes:** Outside-click implies expanded overlay must receive those hits; collapsed input region is the main disk only.

---

## 拖和点怎么分

| Option | Description | Selected |
|--------|-------------|----------|
| 6–8 px slop | 手抖仍算点 | ✓ |
| 几乎一动就算拖 | | |
| 菜单开着能拖，球跟着走 | | |
| 一拖先收，再只拖主球 | | ✓ |
| 默认右侧中部 | | ✓ |
| 整颗球留在屏幕里 | | ✓ |

**User's choice:** 6–8px slop; drag collapses menu first; default mid-right; clamp fully on-screen.
**Notes:** Position not persisted this phase.

---

## the agent's Discretion

None.

## Deferred Ideas

- Persist orb position (v2 / HOST-05)
- Full edge-aware constellation beyond stay-on-output (v2 / HOST-04)
