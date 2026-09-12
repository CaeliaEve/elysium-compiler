# 三仓库数据契约与重构

三个仓库均在 `refactor/core`。新 source → compiler → catalog → NeoNEI 链路已实现并通过共享样本；MCP 任务服务已实现。真实 GTNH 全量采集与高级领域迁移尚未完成。字段定义以 Rust 和生成的 schema 为准。

当前编译器、模组与契约包版本为 0.10.0，source/catalog 修订为 10，包含候选匹配、合成网格、魔法配方、参数化整体构建和原生模型记录。格式与修订作为 schema 字面量由 Rust 常量生成，并通过契约包的 formats 导出；Web 不维护另一份版本判断。旧修订产物直接拒绝，需要使用对应新模组重新采集和编译。业务 API、类型、文件和方法不带演进版本号。

## 约束与所有权

产物 `Output.change` 显式表达输入依赖，action 为 patch 或 merge，samples 与指定输入槽的 choices 一一对应。patch.set 替换指定根标签，patch.limits 按 Minecraft getInteger 语义读取并裁剪数值，保留其他 NBT、metadata 和数量。不存在或非数值标签按 0 读取，long 窄化、float/double 向下取整保持 Java 行为。默认 id/amount 只是首个候选的结果。语义 ID 排除这些观察值，包含输入绑定、标签操作/模板/筛选条件。编译器重算每个样本，拒绝缺失样本、错误引用和虚假的默认结果；反向生产索引包含所有样本。Item.armor 保存实际 ItemArmor 分类，与 tools 一起解释继承限制。

Match.tags 的 keys、present、absent 分别表达根键值相等、仅要求存在和必须缺失，各列表有序且互不重叠；允许其他自定义数据，不能用 boolean 值代替 sceptre 的存在性。MagicRecipe.payment 描述输入法杖自行供能的备选方式，charges 映射要素与 NBT 键，capacity 单位为百分之一 Vis。按原法杖/玩家折扣扣费后再保留并裁剪，或清空；默认结果样本始终是额外法杖供能的结果。creative 为当前创造模式免 Vis 配置；空成本列表只能通过该豁免使用，与六项零成本不同。编译器拒绝充能键、输入绑定和保留策略与产物 patch 相矛盾的数据。

动态界面使用 `clip` 元素引用独立的 `tracks`，20 个 source 集合对应 25 个 catalog 表。轨迹按内容去重，不随每条配方重复保存；每步的持续 tick 和归一化裁剪区域同时控制纹理与目标范围。编译器校验轨迹身份、引用、区域互斥及预算，并拒绝无引用轨迹。空区域是明确隐藏状态，不是采集失败。GT 普通/分段/环形进度与原版熔炉已接入，其他自定义交互和真实游戏验收仍未完成。

- 新格式直接替换旧 SQL、raw/canonical、专用二进制和多套 API。没有迁移器、兼容 reader、别名或旧格式 fallback。
- API、文件、类型、方法使用简短领域词。软件版本和格式 revision 留在元数据中，不进入业务名称。
- NESQL 只支持 GTNH 2.8.4 Java 8 本机单人世界。Agent 通过独立 stdio 桥接调用受限游戏接口，批量文件写盘。
- OC Pattern、样板导入和样板导出已取消。其他已有业务能力需要逐项迁移，删除旧实现不表示功能已完成。
- NESQL 拥有游戏事实、来源、纹理捕获和 source 写入；编译器拥有语义校验、索引、图集和 catalog 发布；NeoNEI 拥有只读查询、展示与用户状态。
- 三个仓库独立发布。共享契约在本仓库生成 tarball，Web 锁定产物，不跨仓库导入 Rust 源码。

## 一套语义，两种存储

source 使用规范 UTF-8 JSON、gzip JSONL 分片和 PNG。catalog 使用 JSON 清单、标准 named-map MessagePack 表和无损 WebP 图集；不再维护手写二进制 offset 或额外 WASM 解码链路。

```text
nesql/datasets/<source-id>/
  manifest.json
  environment.json
  records/<collection>/part-000000.jsonl.gz
  assets/<digest>.png

catalog/
  current.json
  catalogs/<catalog-id>/
    manifest.json
    tables/<collection>/...
    textures/...
```

文件名和排序细节以 manifest 声明为准。读取者只处理声明文件，不扫描候选目录。`source-id` 与 `catalog-id` 均为无前缀的小写 SHA-256；任务 UUID、时间、日志和性能不参与内容身份。

source 清单字段为 `format`、`revision`、`id`、`producer`、`environment`、`scope`、`files`。文件描述含路径、kind、encoding、bytes、decodedBytes、rows、sha256。`scope.mode` 为 `complete` 或 `selection`，集合名排序且唯一。摘要来自去掉 `id` 后的规范 JSON。环境描述固定模组摘要、输入配置/脚本、资源包顺序、语言、采集设置及必要的知识摘要，不含玩家姓名或世界位置。

每个 source 记录最多 1 MiB，分片解压后最多 16 MiB。catalog 表每片最多 4096 行和 16 MiB；图集不超过 4096 × 4096、单页编码不超过 80 MiB。超限报错，不截断。

## 领域

当前 source 有 `aspects`、`assets`、`blocks`、`builds`、`categories`、`circuits`、`fluids`、`groups`、`items`、`materials`、`mutations`、`recipes`、`research`、`shapes`、`species`、`strings`、`structures`、`views` 十八类必需集合。

| 记录 | 含义 |
| --- | --- |
| Item / Fluid | 注册身份、完整 typed NBT、文本与纹理引用；物品还含 NEI 顺序、矿辞、堆叠/工具属性 |
| Recipe | 来源、分类、输入候选、精确数量、消耗/归还、产出概率、tick、EU/t、类型化属性和 View |
| Category | 来源处理器、名称、图标、机器引用及 NEI 分类顺序 |
| Group | NEI guid 来源、显式成员、代表物品、折叠和顺序 |
| Asset | 文件、来源、尺寸、动画裁剪和各帧 tick |
| View | 固定坐标空间的 sprite、slot、text、rectangle、tooltip |
| Text | locale 和 Minecraft 格式文本 |
| Material | 注册材料的颜色、原始化学式、成分相对份数、物品与流体形态；物品含量以约分后的正有理数表示 |
| Circuit | 目标 NEICustomDiagram 定义的系列、电路板、配件及每步电压等级；不是推测的制造配方 |
| Species | Forestry 根及物种 UID、原生温湿度、显隐性和隐藏/黑名单状态、实际成员物品、默认基因与产物 |
| Mutation | 两个亲本、结果物种、基础概率、描述性条件、实际结果基因、实现类和重复登记序号 |

身份由规范 typed NBT 和注册名/meta 计算。数量、显示名、语言与图集位置不改变物品身份。NBT compound 键排序、list 保留类型及顺序；long 使用字符串，float/double 保留位模式。不能删除 NBT 来假装不同物品相同。

数量、持续时间、EU/t 使用范围受限的十进制字符串。概率为精确分子/分母，允许 0 和 1；零概率产出不进入生产索引。每个输入候选各自保存数量、consume/keep/damage、容器归还和匹配规则。规则显式表达 exact、ore、wildcard 或根 NBT 字段的 tags 匹配；ore.exclusive 区分唯一矿辞。相同物品的不同匹配规则是不同候选，重复物品与规则组合则拒绝。

Recipe.grid 保存网格空位、输入槽引用及镜像规则，独立于 View；当前奥术适配器已提供该字段。Recipe.magic 区分奥术、坩埚与注魔，保存基础 Vis/源质成本、研究引用、基础不稳定性和中心材料槽。Element.cost 将原生位置的图标绑定到语义成本。配方 ID 包括网格与魔法语义，排除 research.completed 观察状态；成本、规则改变会改变 ID，玩家完成研究不会改变配方 ID。

属性使用 integer、decimal、quantity、text、flag、reference、list、map、symbol；数量必须有 unit，文本必须是引用。嵌套深度最多 16，小数表示最多 256 字符。未知类型或丢失引用拒绝发布，不按显示名猜测。

catalog 保留语义表，并生成 `browse`、`links`、`index`、`topics`、`lineage`、`textures`。`browse` 提供名称/注册名/矿辞/拼音搜索，`links` 按 category.order 和 recipe.order 排序并关联资料，`index` 只含配方筛选需要的身份和引用。`topics` 提供材料、电路、蜜蜂与树木的轻量搜索和分页；Web 只读取选中的完整记录。`lineage` 保存每个物种的来源突变与参与突变 ID，自交只在参与索引中计入一次，重复登记仍逐项保留。表类型、必需集合和记录检查共用一份 Rust 声明。

材料成分和部件数量不是配方用量；材料含量的分母必须非零且有理数约分。流体形态只保存真实引用，没有按名称猜测的体积/质量换算。电路等级必须连续且包含实际标称电压，配件系列不强加电压等级。材料之间允许图关系，页面按需跳转，不递归展开或推导不存在的反应。

遗传适配依据目标 Forestry 4.10.17：蜜蜂产物率为 `[0,1]`，突变基础率为百分数，导出时统一约分；树木 API 只提供可能产物，`chance` 必须为空，并记录默认果实家族匹配状态。特产与普通产物分开。基因数值只来自明确 API，行为 provider 保留 UID/名称而不编造值；默认模板不等同于玩家个体。突变的结果基因独立保存，不用物种默认基因覆盖。

突变身份覆盖完整事实，包括本地化条件和 `occurrence`；它不是跨语言的注册 ID。相同事实的重复登记按 `0..n-1` 编号并校验连续性，保留独立尝试。亲本规范排序但保留自交的两个引用；条件为可读说明，不是可执行规则，基础概率不代表当前世界的有效概率。隐藏和黑名单是来源状态，不能直接推导为物种不可获得。未提供模板或引用时明确失败，没有旧数据兜底。

结构使用 `Structure`、`Build`、`Block` 和 `Shape`。Structure 保存来源、控制器、说明、首组 probe、片段和 variants；Build 保存原生整体构建、参数、方法/返回值、调色板与坐标框架；Block 保存实际方块状态、完整 typed tile NBT 及可选物品。Shape 是内容寻址的几何块，Cell.index 引用片段规则或构建调色板。每块最多 2048 格，每个几何体最多 1048576 格和 512 块；构建调色板最多8192项。

Environment.probes 为 1–16 组不同参数，按 count、通道键值对排序，count 和通道值按数字比较。count 范围 1–64；channels 最多 32 项，键为 1–64 个小写字母、数字、下划线或连字符，值为 1–65535。参数参与环境摘要。Structure.variants 必须与该列表逐项一致，每项恰有 build 或 problem；Build.probe 必须与引用它的变体相同。完整 scope 不接受失败变体，selection 可以保留明确错误。Structure.probe 必须等于第一组参数，明确片段提示的采集条件。旧 builds/buildProblem 字段已移除。

整体预览坐标为东/下/北，控制器朝南，origin 将局部坐标对应到独立预览世界；控制器在该世界固定为 [0,64,0]。NBT 中存在标准 x/y/z 坐标时必须与该格位一致。验证内容身份、归属、边界、计数、控制器、引用完整性及无孤立记录。片段与构建分别表达定义和观察结果，均不冒充完整运行时成型判定。

新增models集合，source共19个领域集合、catalog共24个表。Build.palette使用Appearance（block/model/problem），允许同一Block在不同邻接和位置下引用不同模型；Build.rendered显式区分未请求外观和采集失败。未请求时模型与问题均为空；已请求时至少有模型或问题。complete要求已请求且无缺失，selection可保存部分外观及原因。没有旧字符串调色板别名。

Model为内容寻址的四边形列表，上限1024面，保持单条source记录小于1MiB。Face保存纹理asset引用、solid/blend通道和4个Vertex；三角形重复末顶点。Vertex.at为局部东/下/北坐标（每轴±256），uv为[0,1]、v=0在图像顶端；坐标均用float32十进制字符串保留Java/Rust/JS身份一致性，颜色是RRGGBBAA u32。验证有限值、范围、预算、纹理引用、模型身份及无孤立模型；同一Block可出现在不同Appearance中，完全重复的Appearance拒绝。现有asset帧与图集承载模型纹理，没有第二种纹理存储格式。

验证几何摘要、唯一坐标、边界、规则和物品引用、块顺序及计数。未引用的几何拒绝发布。不可枚举定义只能在 selection 中以 problem 明确记录。建议放置物品对应首组规范化参数，不是可执行成型规则或完整用料。topics 与物品关联包含 structure，列表无需解码几何。

`Aspect` 保留注册要素的身份、组成对、颜色、图标和已观察到的发现状态。物品 `aspects` 为带精确字符串数量的列表；null 表示提供方没有返回值，空列表表示返回了零项。`Research` 保留分类、位置、标记、研究要素、前置／隐藏前置／关联解锁、触发条目和完成状态。研究正文页面及魔法配方仍需独立迁移，不能把这些元数据称为完整魔导手册。

研究引用的普通 key 必须指向已注册记录；未注册的 `@` key 是目标版本认可的知识标记，显式保存且不编造记录。知识缓存尚未同步时保存 null，已同步的 false 不与之合并。验证要素组成循环、数量、资源与记录引用，以及同一研究在快照中的观察一致性。知识指纹包含缓存是否存在，采集不调用扫描或解锁接口。

`Topic.image` 引用独立图集资源，支持没有物品图标的要素和研究。`topicKinds` 与 `collections` 从共享 schema 导出，HTTP 不再手工同步另一份类型清单。CLI inspect 的字段为 id/environment/scope/counts，counts 覆盖所有已验证的 source 集合。

## 发布与错误

Model.hidden明确表示原生渲染器不单独绘制该位置，例如双箱的第二个方块；hidden=true恰对应空faces，其他模型必须非空。它与Appearance.problem的失败状态分离，也不同于Build.rendered=false的未请求外观。构建仍保留全部物理格位及Block引用。普通/陷阱箱、双箱和末影箱通过独立原版模型盒与纹理资源适配；未知专用渲染器保持明确失败。

NESQL 通过有界外部排序合并重复记录，冲突身份报错；图片编码、压缩和文件摘要在后台处理。目录先封存校验，再原子发布；结果日志与不可变内容目录分开。

编译器验证清单、压缩字节、解压字节、记录数、身份、引用、视图槽位、图片尺寸及资源预算，之后生成 catalog。失败不更新 `current.json`。已固定快照继续有效属于事务一致性，不是旧格式 fallback。

写入目标必须在 source 之外。report 必须在 source 和 catalog 之外。创建目录前检查已有祖先，拒绝路径中的 `..`、symlink 和 Windows junction。不能用 copy+覆盖冒充原子目录发布。

CLI 为 `inspect`、`compile`、`check`、`schema`，stdout 返回一个 JSON 结果，stderr 返回错误。schema 由 Rust 导出，再生成 TypeScript 类型及 JS 解码器。未知修订、字段或集合均明确拒绝。

## 验证与后续

当前保留两组 Java、两组 MCP、五个 Rust 集成测试、四组 HTTP、三组前端逻辑和四组浏览器行为检查。重点保护跨语言身份与数量、图片像素、引用、确定性发布、损坏输入、路径边界、任务/缓存生命周期；不验证源码字符串和内部函数位置。结构样本包含两组参数、两种整体几何及共享块；错误场景验证缺失、重复、乱序、参数错配和部分范围的显式失败。

Java 与 Rust 构建、clippy、共享 fixture 编译、Web 构建及浏览器行为已通过。依赖和产物检查使用真实生产 jar/可执行文件，避免历史 stub 测试给出假通过。

GT 材料、电路、遗传、结构片段、参数化整体构建与世界四边形模型已有记录、采集适配、编译校验和页面，并扩展现有测试；实际目标游戏采集仍待验收。Windows junction 的输出和报告路径拒绝已在本机实测。完整离线资料库已在真实浏览器验证；专用实体/其他模组渲染器、其余魔法语义、完整处理器覆盖和动态 UI 仍需继续。

游戏验收还应覆盖 Agent 断线、取消、离开世界、资源重载、GL 恢复、磁盘失败与全量导出。性能必须注明数据集摘要、机器、构建、冷/热状态、总耗时、帧占用、峰值内存和输出体积。共享小样本不代表全量 GTNH，未实测前不承诺提升倍数。
