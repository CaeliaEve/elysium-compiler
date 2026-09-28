# 本地实例资源与来源审计实现计划

> 按用户批准的 E:/codex/NEI/LOCAL-DATA-PLAN.md 内联执行，使用 executing-plans、test-driven-development、verification-before-completion；真实游戏导出由用户安排的 agent 执行。沿用 refactor/core，不改变 UI，不清理历史数据。

**目标：** 提供可重现的处理器来源审计和无游戏依赖的静态资源导入，为后续事实/渲染分片集成建立可验证输入。

**架构：** NESQL Bridge 从已有实机检查点与工作清单生成证据明确的来源报告；Compiler 读取显式文件清单，校验已固定摘要的 ZIP/JAR，保存资源候选及按内容寻址的原始字节。没有原生资源解析证据时不选择覆盖赢家，不推断物品图标。

**技术栈：** 现有 Node.js Bridge、Rust Compiler、serde、sha2、ZIP/DEFLATE；不引入服务端设施。

## 任务 1：真实来源与覆盖审计

文件：NESQL++/bridge/src/audit.mjs、bridge/test/audit.test.mjs、docs/local-inputs.md。

- [x] 测试先行：构造 passed/failed/pending/excluded 检查点，断言工作清单的 implemented 不会把失败项变为通过；拒绝重复、缺失及多余身份；对未知类保留 unverified。
- [x] 运行 `node --test bridge/test/audit.test.mjs` 看见失败，随后实现纯函数 `audit(worklist, checkpoint)`，按精确类表声明已核对来源，其他类保留独立待审查组。
- [x] CLI 读取两份输入原件，记录 SHA256 与字节数，新输出使用 wx；不修改历史清单或运行游戏。
- [x] 对 2026-09-28 原件生成独立报告，检查汇总 330/153/149/27/1。说明覆盖与来源确认是两个维度。

## 任务 2：静态资源导入最小闭环

文件：Compiler core/src/resources.rs、cli/tests/resources.rs、core/src/catalog/cli.rs、Cargo.toml、README.md。

- [x] 测试通过真实 CLI 导入两个小 ZIP：同路径不同字节保持两个候选，同字节只存一份，PNG/mcmeta/lang 保留原字节，Java 类与根目录凭据不导出。
- [x] 先运行 `cargo test --test resources` 验证 CLI 尚无 resources 子命令而失败。
- [x] 实现 `resources --input <manifest> --output <new-directory>`。输入 format=elysium.resources、revision=1、archives=[{key,path,bytes,sha256}]；路径相对清单或绝对，定位路径不进入结果身份。输出携带归档 key/摘要、候选资源逻辑路径/来源/字节/摘要及 blob 路径，不承诺这些归档就是当前游戏全部已加载资源。
- [x] 显式限定资源类型，严格拒绝相关路径穿越、重复资源条目、摘要不符、非普通文件、超预算数据。新目录中临时构建，成功后原子发布；失败无完整结果，已有目标不覆盖。
- [x] 增加腐损、摘要变化、重复名字、目录逃逸、重复目标与重定位确定性测试；资源选择由将来的运行时证据负责，不能按 ZIP 顺序猜测。
- [x] 使用实际实例中若干固定摘要 jar 做只读导入，结果保存至新的 .refactor-state 路径；不称为完整 Source/Catalog，不启用游戏导出。

## 任务 3：交接与后续边界

- [ ] 执行 Bridge 测试、Compiler fmt/test/clippy/build；核对 diff 与精确提交，推送 refactor/core 并检查对应 CI。
- [x] 记录本轮产物、命令、摘要与局限。下一阶段连接 NESQL 原生资源解析记录，再以样本贯通 Source/Catalog；随后实现分片恢复及语义家族适配。这些工作未交付前保持未完成状态。
- [x] 不让导出 agent 对本轮资源工具提前重跑 330 项或全量 Source；达到可验收候选包时再提供独立交接单。

## 执行记录

2026-09-28：行为测试先失败再实现；Compiler 新增 3 项资源 CLI 测试（正常路径、5 种非法输入、5 种归档错误），workspace 共 10 项通过，fmt/clippy/release/check-release 通过。Bridge 共 19 项通过，含来源审计 CLI 和两项协调器实机问题的离线重现。实际五归档样本导入 21,375 项资源候选、10,845 个去重 blob，独立逐份核对字节长度与 SHA256 通过；资源输出仍为 unverified。

来源审计保留全部 330 项，但仅对已核对的精确原生类填入来源，其余 unverified；完整的 330 项权威来源审查仍未完成。资源解析绑定、Source 组装、恢复机制和剩余配方适配均不在本次完成范围内。已顺带修复协调器 domain:structure 路由和加载器原始错误归档，未改 Java 模组或 UI。
