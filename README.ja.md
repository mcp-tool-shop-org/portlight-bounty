<p align="center">
  <a href="README.md">English</a> | <a href="README.zh.md">中文</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.hi.md">हिन्दी</a> | <a href="README.it.md">Italiano</a> | <a href="README.pt-BR.md">Português (BR)</a>
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

ポートライト・バウンティは、貿易をテーマにしたゲームです。商品を買い、航海し、契約を結び、破産せずに目的地に到着できるようにします。マークの下にある一文が、ゲームの概要です。貿易、航海、そして繁栄。

これは、[Portlight](https://github.com/mcp-tool-shop-org/portlight)のRust版のリポジトリです。シミュレーションはルールに従って行われます。Godot 4.7のチャートは、実際にプレイするゲームです。バージョン0.3.1は、この2つの組み合わせの現在のリリースです。

アル・マナールで市場が開いている、停泊中のチャート：

![アル・マナールで市場が開いている、停泊中のチャート](docs/screenshots/chart-1280.png)

チャート上に表示されている、現在進行中の2つの契約。数字は、輸送済みの貨物、未輸送の貨物、残りの日数を示します。

![ポルト・ノヴォのチャート上に表示されている、現在進行中の契約](docs/screenshots/contract-strip-active.png)

## プレイ

Rust 1.98.1（`rust-toolchain.toml`で指定）と、公式ビルドのGodot 4.7.2が必要です。Linux CIでは、この組み合わせを使用します。拡張機能のWindowsビルドは実行済みです。macOSビルドはまだ実行されていません。新しいクローンを作成した場合、チャートを開く前に、Godotのヘッドレスアセットインポートを一度実行してください。

```
cargo build -p portlight-godot
godot --headless --path godot --import --quit
godot --path godot
```

新しいゲームを開始し、「The Merchant」を選択し、名前を「Ada」のままにすると、スクリーンショットに表示されている航海が始まります。航海し、貿易を行い、港のリストから契約を結びます。保存すると、「新しいゲーム」の横にバージョン12またはバージョン13のセーブスロットが作成されます（会計事務所のアカウントを開くと、バージョン13になります）。ロードされたカスタムの船長は、名前、所持金、および日数を保持します。カスタムテンプレート自体はセーブデータに含まれないため、ロード後の価格は、その商人の価格となります。これはPython版のゲームと同じです。

海上で、港のリストは非表示になります。次の日に進むと、航海が進みます。その日のレポートは、チャート上に表示され、その日に何かが起こると、その内容が表示されます。

ハンドブックには、画面の説明が記載されています。[ランディングページ](https://mcp-tool-shop-org.github.io/portlight-bounty/)から、ハンドブックに進みます。

## シミュレーション

箱は描画されません。`portlight-sim`がルールであり、リストされているRustで調整されたスクリプトを除いて、Python版のゲームと一致します。`portlight-cli`はスクリプトを実行し、パリティ検証ツールがPythonと比較するスナップショットを出力します。`portlight-chart`は、Godotを含まない等角投影の数学です。

```
cargo run -p portlight-cli -- new --captain merchant --name Ada --seed 42
cargo run -p portlight-cli -- --help
cargo test --locked --workspace --exclude portlight-godot
```

`portlight --help`は、すべてのスクリプトコマンドをリストします。終了コードは、実行が成功した場合は0、シミュレーションまたはファイルが失敗した場合は1、コマンドが理解できなかった場合は2です。失敗は、文章で表示されます。テレメトリーはなく、`--debug`スイッチもありません。

Python版のゲーム（コミット`9b02494`）が、ルールのソースです。5つの船長のアーキタイプは、Rustで調整された価格と消耗のオーバーレイ（`crates/portlight-sim/data/tuned_overlay.json`）を持ち、The Bounty Hunterは賞金に+25%のボーナスを与えます。`--features python-base`でビルドすると、オーバーレイが無効になり、シミュレーションは変更されていないPython版と一致します。

## 0.3.1とは

20の港と43の海路を舞台にした、プレイ可能なチャートです。市場、契約、乗組員（コンパニオンの役割と士気ライン付き）、戦闘スタイルの達人やスキルを教えてくれるトレーナー、装備のメンテナンスと鍛冶品質のアップグレードを行う武器庫の商人、造船所、港の事務室（7つの港での会計を含む）、狩りや賞金、日記、戦闘と契約の概要カード、沈没と曳航回収、そして海戦などが含まれます。世界の正史（フェーズ1～6）は、`docs/canon/`にあります。初期のパッケージはマニフェスト0.3.0（`godot/assets/landing/MANIFEST.json`）です。

## 画像

`godot/assets/`と`docs/screenshots/`の画像は、コードと同じMITライセンスで提供されます。

樽、係留柱、カート、箱、そして松明は、Blenderのプリミティブから、`build_prop()`によって[ai-rpg-stage](https://github.com/mcp-tool-shop-org/ai-rpg-stage)（`assets/dimetric/camera/dimetric_60_45.py`）で作成されました。この機能は、`55ac9b2`で実装されました。マニフェストでは、このコピーを「ベンダーからのドロップ」と呼びました。これは、再作成されたものではなく、独自のレポジトリからコピーされたことを意味します。

3つの埠頭の旗は、ローカルのComfyUI環境（Qwen Imageの重みとSalt Road LoRA）で描画され、その後、固定カラーマップで再調整されました。

バージョン0.3.1には、音声アセットが含まれていません（チャートは無音で実行されます）。また、ElevenLabsの音声も含まれていません。

## 信頼

このゲームはローカルで実行されます。ネットワーク接続を開きません。アカウントも持っていません。テレメトリーも送信しません。バイナリに組み込まれたカタログ、指定するスクリプト、および指定するセーブデータを読み込みます。バージョン12またはバージョン13のセーブデータを、選択したフォルダーに書き込みます。チャートのデフォルトフォルダーは、作業ディレクトリ内の`saves/`です。

このリポジトリで、脆弱性を非公開で報告してください。[SECURITY.md](SECURITY.md)を参照してください。別のメールボックスはありません。

## サポート

0.3.1はサポート対象のリリースです。問題やプルリクエストは大歓迎です。送信する前に、[CONTRIBUTING.md](CONTRIBUTING.md)をお読みください。コードと画像はMITライセンスです。

[MCP Tool Shop](https://mcp-tool-shop.github.io/)によって作成されました。コードとアート：mcp-tool-shopの貢献者。
