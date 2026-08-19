# xtools

## What This Is

Linux 桌面上给自己用的工具箱。一颗始终置顶的主悬浮球，点击后时间戳 / JSON / 翻译三颗功能球围绕主球弹出；再点某颗功能球，打开对应的独立 Rust 窗口。三个功能是独立窗口进程，主程序只负责球和唤起。

## Core Value

点击主球，功能球围绕它弹出；再点功能球，打开或聚焦对应独立窗口。这一条必须成立。

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] 主球始终置顶，可拖动
- [ ] 点击主球后，三颗功能球围绕主球弹出
- [ ] 再点主球收起功能球
- [ ] 点功能球打开对应独立窗口；已打开则聚焦，不新开
- [ ] 时间戳、JSON、翻译是三个独立 Rust 窗口进程
- [ ] 三个窗口视觉风格一致（同一套主题、控件、布局节奏）
- [ ] 时间戳：Unix 秒/毫秒 ↔ 日期时间，常用格式一键复制（10 位、13 位、RFC3339、自定义）
- [ ] JSON：格式化、压缩、校验并标出错误位置
- [ ] 翻译：统一输入/输出/语种界面，翻译引擎可替换
- [ ] v1 菜单入口写死三个功能；架构按独立进程预留，不做插件目录扫描

### Out of Scope

- 插件目录丢文件自动出现在菜单 — 明确以后再加，v1 先跑通三个窗口
- uTools 式搜索框 / 命令面板 — 用户选择环绕功能球，不要点球出搜索
- 点主球直接弹出三个功能窗口 — 用户否定；必须先出环绕球
- 桌面上始终三颗球、没有主球 — 用户要的是一点主球再展开
- 同一功能多开窗口 — 用户选择聚焦已有窗口
- 离线词典 / 本地翻译模型 — v1 只要可插引擎的统一界面
- 安装包、开机自启、多桌面环境分发 — 先给自己用
- Windows / macOS — 当前环境是 Linux 自用
- 全局热键、剪贴板监听、超级面板 — 不是这次要的入口

## Context

对标 utools 的「随手唤起小工具」，但入口不是搜索框：一颗置顶主球 + 环绕三颗功能球。功能本身是独立窗口程序，主程序不内嵌业务 UI。

已经拍板的使用路径：

1. 主球一直浮在最上层，可拖到顺手的位置
2. 点主球 → 时间戳 / JSON / 翻译三颗球绕着主球出现
3. 点其中一颗 → 打开该功能的独立窗口；若已开则提到前面
4. 再点主球 → 功能球收起
5. 三个窗口长得像一套软件，不是三个完全不同的程序皮

翻译窗口先做壳：输入、输出、语种切换。引擎可换，v1 接一个能用的实现即可。JSON 要能看懂哪里写坏了，不只是成不成功。时间戳要能把结果按常用格式复制走。

## Constraints

- **Tech stack**: Rust — 用户指定；主程序和三个功能都是 Rust 窗口程序
- **Process model**: 功能 = 独立进程窗口；主程序只负责悬浮球和启动/聚焦
- **Platform**: Linux 桌面，自用
- **v1 surface**: 三个写死入口，不做动态插件发现
- **UI**: 窗口风格必须一致，需要共享主题/控件，而不是三个窗口各画一套

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| 环绕三球菜单，而不是文字列表或点球出三个窗口 | 用户明确：菜单贴在球上；最好点主球后功能球围绕主球弹出 | — Pending |
| 功能是独立 Rust 窗口进程 | 用户：菜单只需唤起各种独立窗口，功能独立 | — Pending |
| 同一功能单实例，再点聚焦 | 用户选择聚焦已有窗口、不新开 | — Pending |
| v1 入口写死，插件扫描以后再加 | 用户选择先做三个窗口 | — Pending |
| 翻译做可插引擎的统一界面 | 用户不要绑死某一个离线方案；引擎可换 | — Pending |
| JSON 格式化 + 校验报错 | 用户不要一上来做 jq 路径查询 | — Pending |
| 时间戳换算 + 常用格式一键复制 | 日常对日志/接口字段，结果要能立刻拷走 | — Pending |
| 只做 Linux 自用，不做分发 | 用户：就我自己日常用 | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-08-19 after initialization*
