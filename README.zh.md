<p align="center">
  <a href="README.ja.md">日本語</a> | <a href="README.md">English</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.hi.md">हिन्दी</a> | <a href="README.it.md">Italiano</a> | <a href="README.pt-BR.md">Português (BR)</a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/mcp-tool-shop-org/brand/main/logos/portlight-bounty/readme.png" alt="Portlight Bounty" width="400">
</p>

<p align="center">
  <a href="https://github.com/mcp-tool-shop-org/portlight-bounty/actions/workflows/ci.yml"><img src="https://github.com/mcp-tool-shop-org/portlight-bounty/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://codecov.io/gh/mcp-tool-shop-org/portlight-bounty"><img src="https://codecov.io/gh/mcp-tool-shop-org/portlight-bounty/branch/main/graph/badge.svg" alt="Coverage"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="MIT"></a>
  <a href="https://mcp-tool-shop-org.github.io/portlight-bounty/"><img src="https://img.shields.io/badge/docs-landing%20page-b08900" alt="Landing page"></a>
</p>

《港口财富》是一款贸易游戏。你购买商品，航行，接受合同，并努力避免破产。下方线条所描述的就是游戏的核心：贸易、航行、致富。

这个仓库是《港口财富》（[https://github.com/mcp-tool-shop-org/portlight](https://github.com/mcp-tool-shop-org/portlight)）的 Rust 移植版本。模拟的是游戏规则。Godot 4.7 版本是你可以玩的游戏。0.3.0 版本是这两个版本的当前发布版本。

停靠的港口，在阿尔-马纳尔港口开放市场：

![在阿尔-马纳尔港口停靠，市场开放](docs/screenshots/chart-1280.png)

在地图上显示了两个可用的合同。数字分别代表已交付的货物、应交付的货物和剩余天数：

![在波尔图诺沃港口的地图上显示可用的合同](docs/screenshots/contract-strip-active.png)

## 开始游戏

你需要 Rust 1.98.1（在 `rust-toolchain.toml` 中指定）和 Godot 4.7.2，这是官方版本。Linux CI 使用这两个版本。已经运行了扩展程序的 Windows 版本。尚未运行 macOS 版本。

```
cargo build -p portlight-godot
godot --headless --path godot --import --quit
godot --path godot
```

新游戏，然后选择“商人”，如果你想体验截图中显示的航行，可以将角色名设置为 Ada。航行、贸易，并从港口列表中选择一个合同。保存游戏会将版本 12 或版本 13 的存档保存在“新游戏”旁边（当打开一个会计账户时，会保存版本 13）。加载自定义角色会保留角色名、银币和当前日期。自定义模板本身不会保存在存档中，因此加载后商品的价格将是商人的价格。这与 Python 游戏相同。

在海上，港口列表会隐藏。点击“下一天”会推进航行。当一天发生了一些事情时，当天的报告会在地图上显示。

手册会引导你浏览各个界面：[主页](https://mcp-tool-shop-org.github.io/portlight-bounty/)，然后是手册。

## 模拟

箱子不会被绘制出来。`portlight-sim` 是游戏规则，与 Python 游戏相同，除了列出的经过 Rust 优化的脚本。`portlight-cli` 运行一个脚本，并打印快照，以便与 Python 进行比较。`portlight-chart` 是等距数学，仍然不包含 Godot。

```
cargo run -p portlight-cli -- new --captain merchant --name Ada --seed 42
cargo run -p portlight-cli -- --help
cargo test --locked --workspace --exclude portlight-godot
```

`portlight --help` 列出了每个脚本命令。退出代码为 0 表示运行成功，1 表示模拟或文件失败，2 表示命令无法理解。失败会以句子形式显示。没有遥测数据，也没有 `--debug` 开关。

Python 游戏在提交 `9b02494` 处是规则的来源。五个船长原型具有少量经过 Rust 优化的定价和消耗叠加层（`crates/portlight-sim/data/tuned_overlay.json`），并且“赏金猎人”可以获得 25% 的赏金；使用 `--features python-base` 构建会关闭叠加层，以便模拟与未修改的 Python 游戏匹配。

## 0.3.0 版本的内容

一个可以在 20 个港口和 43 条航线上运行的地图：市场、合同、船员及其同伴角色效果和士气值、战斗风格大师和技能训练师、军械商铺，提供装备维护和铁匠质量升级、船坞、港口控制台（包括七个港口的会计账户）、狩猎和赏金、日志、战斗和合同总结卡、沉船和拖船回收，以及海上战斗。世界背景（阶段 1-6）位于 `docs/canon/` 下。初始包的版本为 0.4.3（`godot/assets/landing/MANIFEST.json`）。

## 图片

在 `godot/assets/` 和 `docs/screenshots/` 中的图片使用与代码相同的 MIT 许可证。

桶、栓桩、手推车、箱子和火把由 `build_prop()` 在 [ai-rpg-stage](https://github.com/mcp-tool-shop-org/ai-rpg-stage)（`assets/dimetric/camera/dimetric_60_45.py`）中，使用 Blender 的基本形状构建。该功能在 `55ac9b2` 中实现。清单将副本称为“供应商物品”。这意味着是从我们自己的仓库复制的，而不是重新制作的。

三个码头旗帜是在本地 ComfyUI 设置（Qwen Image 权重和 Salt Road LoRA）上绘制的，然后使用固定的颜色映射重新调整。

此版本中没有 ElevenLabs 语音。

## 信任

游戏是本地的。它不会打开网络连接，它没有账户，也不会发送遥测数据。它会读取二进制文件中嵌入的目录、你命名的脚本和存档。它会将版本 12 或版本 13 的存档写入你选择的文件夹。地图的默认文件夹是工作目录下的 `saves/`。

在此仓库中私下报告漏洞。请参阅 [SECURITY.md](SECURITY.md)。没有单独的邮箱。

## 支持

0.3.0 是受支持的版本。欢迎提交问题和拉取请求。在提交之前，请阅读 [CONTRIBUTING.md](CONTRIBUTING.md)。代码和图片均采用 MIT 许可证。

由 [MCP Tool Shop](https://mcp-tool-shop.github.io/) 构建。代码和艺术作品：mcp-tool-shop 贡献者。
