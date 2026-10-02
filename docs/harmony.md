# 鸿蒙之眼共享过程契约

实现范围：原版 GTNH 2.8.4 / GregTech 5.09.51.482 的 EyeOfHarmonyRecipe 与 MTEEyeOfHarmony。修复 NEI 的零流体占位和固定产量投影；不模拟运行世界，不执行随机数，不读取玩家实时存量。保留界面布局。代码完成后仍待统一实机验收。

Recipe.process 使用 kind=harmony，mode=single/parallel，保存 hydrogen、helium、ticks、startEu、outputEu、chance、rocketTier、compressionTier。整数为十进制字符串；chance 是原生 double 的往返十进制字符串。版本升级至 Source 18 / Catalog 17。

单次与并行分别输出配方分支，共享原生视图。物品槽 0 保留触发星球方块，忽略 metadata/NBT，数量至少 1。单次流体槽 0/1 对应氢/氦；并行槽 2 对应 RawStarMatter。Consumption.buffer 表示启动时清空该流体的内部储量；amount 是最小门槛（并行为首档 8 并行的门槛，实际随阵列数变化）。槽位可不在原生画面显示，但必须保留在配方数据与现有详情列表。

每个普通产出使用 Quantity.harmony(outcome=success, nominal=原生 long 基数)。额外一个流体失败产出 outcome=failure，nominal=14400*2^(rocketTier+1)。所有输出 amount=null、chance=1/1，无独立概率或 item change；输出量相关性由同一个 process 确定。Compiler 校验模式、身份、槽位、流体门槛、参数、唯一失败产出及所有普通产出。区间仅作保守显示范围，不宣称每个整数可达到。

过程规则（原生数值顺序必须保留）：

1. 三种场方块 metadata 0..8；compression >= compressionTier。第一集成电路模式 clamp(0..24)。星界阵列是安装设备，数量 0..8637；不是每配方消耗物。
2. 阵列数 A=0 时 n=1；否则 e=floor(log(8*min(A,8637))/log(1.7))，n=(long)GTUtility.powInt(2,e)，n=8..1048576。单次氢/氦门槛分别是参数；并行门槛为 double(helium)*1.24E-5*n，比较 double(内部 RawStarMatter 存量)。
3. outputEu 起点=(long)(double(baseOutputEu)*(1-(8-stabilisation)*.05))。输入为 -startEu * (long)powInt(4,circuit)，BigInteger 乘法。并行分别乘 (long)(max(1,powInt(2.3,e))*1000000) 与 (long)(max(1,powInt(2.3,e))*1.63*1000000)，再 BigInteger /20700000（向零截断）。开始时无线能量扣款失败不进入后续清空；完成后的能量产出不依赖成功次数。
4. 时长=(int)max(ticks*powInt(2,-dilation)*powInt(.97,compression-compressionTier)*min(1,powInt(2,-circuit)),1)。powInt 是 GT 的平方乘算法，不能替换为 Math.pow 而声称位级相同。
5. 每个所选流体惩罚 f(x,r)=1-exp(-powInt(30*(double(x)/r-1),2))；单次相加氢/氦惩罚，并行使用 RawStarMatter 的 double 门槛。
6. rawChance=baseChance-dilation*.0925+stabilisation*.05。pity=Double.MIN_VALUE 时先置 rawChance；previousChance=上次 successChance。单次若 rawChance==previousChance 且 pity>=1，将 chance 置1；减所选惩罚后 clamp(0..1)。并行不使用该提升。
7. 清空本次使用的内部流体；yield=clamp(1-stabilisation*.05-所选惩罚,0,1)。k=ParallelHelper.calculateIntegralChancedOutputMultiplier((int)(10000*chance),(int)n)，所有产出共享 k。原生 helper 在 mean±3σ 在 [0,n] 内时使用截断正态近似及随机尾数舍入，否则执行 nextInt(10000) 阈值试验；不能标记为精确二项分布。
8. 普通产出=(long)(double(nominal)*(yield*double(k)))。失败时空流体=(long)(chance*14400*powInt(2,rocketTier+1)*double(n-k))，不乘 yield；零产出不发出。Java double→long 截断/饱和规则保留。
9. 单次失败后 pity=previousChance==chance ? pity+(1-chance)*chance : chance；单次成功后 pity=Double.MIN_VALUE。并行不改 pity。此状态跨配方运行，不能把每条配方当作无历史独立 Bernoulli 过程。

验证：在现有轻量契约测试加入跨输出相关性/非法混搭拒绝/大整数边界；在可选原生机器测试内检查只读配方 getter、原生纯计算方法及副本所有权；共享 Java fixture→Compiler→Web 同步。实际世界、GL 和 RNG 分布不以这些离线测试替代。这里描述规则而不新增运行预测器，避免维护第二套机器模拟器。
