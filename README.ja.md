<p align="center">
  <a href="README.md">English</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.hi.md">हिन्दी</a> | <a href="README.it.md">Italiano</a> | <a href="README.pt-BR.md">Português (BR)</a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/mcp-tool-shop-org/brand/main/logos/portlight-bounty/readme.png" alt="Portlight Bounty" width="400">
</p>

<p align="center">
  <a href="https://github.com/mcp-tool-shop-org/portlight-bounty/actions/workflows/ci.yml"><img src="https://github.com/mcp-tool-shop-org/portlight-bounty/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://codecov.io/gh/mcp-tool-shop-org/portlight-bounty"><img src="https://codecov.io/gh/mcp-tool-shop-org/portlight-bounty/branch/main/graph/badge.svg" alt="Coverage"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT%20%2B%20CC%20BY%204.0-blue" alt="MIT code, CC BY 4.0 pictures"></a>
  <a href="https://mcp-tool-shop-org.github.io/portlight-bounty/"><img src="https://img.shields.io/badge/docs-landing%20page-b08900" alt="Landing page"></a>
</p>

ポートライト・バウンティは、貿易をテーマにしたゲームです。商品を買い、船で航海し、契約を結び、破産せずに目的地に到着できるようにします。マークの下にある線は、ゲームの全体的なコンセプトを表しています。貿易、航海、そして繁栄。

このリポジトリは、[Portlight](https://github.com/mcp-tool-shop-org/portlight)のRust版です。シミュレーションはルールを表し、Godot 4.7のチャートは実際にプレイするゲームです。バージョン0.3.2は、この2つの組み合わせの現在の**ソースコードのみのデベロッパー向けリリース**です（事前にビルドされたバイナリ版の配布は、今後のマイナーリリースで予定されています）。

アル・マナールで市場が開いている状態で停泊しているチャート：

![アル・マナールで市場が開いている状態で停泊しているチャート](docs/screenshots/chart-1280.png)

チャート上に表示されている2つの有効な契約。数字は、輸送された貨物、未輸送の貨物、残りの日数を示します。

![ポルト・ノヴォのチャート上に表示されている有効な契約](docs/screenshots/contract-strip-active.png)

## （ソースコードから）プレイ

このリリースは、ソースコードのみで提供されます（事前にビルドされたバイナリやエクスポートプリセットはまだありません）。Rust 1.98.1（`rust-toolchain.toml`に記載）とGodot 4.7.2（公式ビルド）が必要です。Linux CIは、この組み合わせを使用します。拡張機能のWindowsビルドは実行済みです。macOSビルドはまだ実行されていません。リポジトリをクローンし、チャートを開く前に、Godotのヘッドレスアセットインポートを一度実行してください。

```
git clone https://github.com/mcp-tool-shop-org/portlight-bounty.git
cd portlight-bounty
cargo build -p portlight-godot
godot --headless --path godot --import --quit
godot --path godot
```

新しいゲームを開始し、「The Merchant」を選択し、名前を「Ada」のままにすると、スクリーンショットに表示されている航海が始まります。航海し、貿易を行い、港のリストから契約を結びます。保存すると、「新しいゲーム」の横にバージョン12またはバージョン13のセーブスロットが作成されます（会計事務所のアカウントを開くと、バージョン13になります）。ロードされたカスタムの船長は、名前、所持金、および日数を保持します。カスタムテンプレート自体はセーブに含まれていないため、ロード後の価格は、その商人の価格になります。これはPythonゲームと同じです。

海上で、港のリストは非表示になります。次の日に航海が進みます。その日のレポートは、チャート上に表示され、その日に何かが起こった場合に表示されます。

ハンドブックは、画面を順に説明します。[ランディングページ](https://mcp-tool-shop-org.github.io/portlight-bounty/)、次にハンドブック。

## シミュレーション

箱は描画されません。`portlight-sim`はルールであり、リストされているRustで調整されたスクリプトを除いて、Pythonゲームと一致します。`portlight-cli`はスクリプトを実行し、パリティハーネスがPythonと比較するスナップショットを出力します。`portlight-chart`は、次元的な数学であり、まだGodotは含まれていません。

```
cargo run -p portlight-cli -- new --captain merchant --name Ada --seed 42
cargo run -p portlight-cli -- --help
cargo test --locked --workspace --exclude portlight-godot
```

`portlight --help`は、すべてのスクリプトコマンドをリストします。終了コードは、実行が成功した場合は0、シミュレーションまたはファイルが失敗した場合は1、コマンドが理解できなかった場合は2です。失敗は文章で表示されます。テレメトリはなく、`--debug`スイッチもありません。

Pythonゲームのコミット`9b02494`は、ルールのソースです。5つの船長のアーキタイプは、小さなRustで調整された価格と消耗のオーバーレイ（`crates/portlight-sim/data/tuned_overlay.json`）を持ち、「The Bounty Hunter」は賞金に+25%のボーナスを与えます。`--features python-base`でビルドすると、オーバーレイが無効になり、シミュレーションは変更されていないPythonと一致します。

## 0.3.2とは

20の港と43の海路を航行できるゲームチャート：市場、契約、仲間との関係による効果と士気を示す乗組員、戦闘スタイルをマスターし、スキルを訓練するトレーナー、ギアのメンテナンスと鍛冶屋による品質のアップグレードが可能な武器庫、造船所、港の事務室（7つの港で会計事務所のアカウントを含む）、賞金稼ぎと賞金、ジャーナル、戦闘と契約の概要カード、沈没と曳航による回収、そして海戦。世界の正史（フェーズ1〜6）は、`docs/canon/`にあります。ランディングバンドルは、マニフェスト0.3.0（`godot/assets/landing/MANIFEST.json`）です。

## 画像

画像（リポジトリ内のすべての画像ファイル）は、mcp-tool-shopによって作成され、CC BY 4.0ライセンスで提供されます。誰でも、クレジットを「mcp-tool-shop」に表示することで、商業目的を含め、それらを使用および変更できます。その他のすべてのものは、MITライセンスです。[LICENSE](LICENSE)を参照してください。

樽、係留柱、カート、箱、松明は、Blenderのプリミティブを使用して、`build_prop()`によって[ai-rpg-stage](https://github.com/mcp-tool-shop-org/ai-rpg-stage)（`assets/dimetric/camera/dimetric_60_45.py`）で作成されました。この機能は、`55ac9b2`で実装されました。マニフェストでは、このコピーは「ベンダーからのドロップ」と呼ばれていました。これは、リメイクされたのではなく、独自のレポジトリからコピーされたことを意味します。

3つの埠頭の旗は、ローカルのComfyUI環境（Qwen Imageの重みとSalt RoadのLoRA）で描画され、その後、固定カラーマップで再調整されました。

バージョン0.3.2には音声アセットが含まれていません（チャートは無音で実行されます）。また、ElevenLabsの音声機能も搭載されていません。

## 信頼

このゲームはローカルで実行されます。ネットワーク接続を開き、アカウントを作成したり、テレメトリを送信したりすることはありません。バイナリに組み込まれたカタログ、指定するスクリプト、および指定するセーブファイルを読み取ります。バージョン12またはバージョン13のセーブファイルを、選択したフォルダーに書き込みます。チャートのデフォルトフォルダーは、作業ディレクトリの下の`saves/`です。

このリポジトリで、脆弱性を非公開で報告してください。[SECURITY.md](SECURITY.md)を参照してください。別のメールボックスはありません。

## サポート

0.3.2は、サポートされているリリースです。問題やプルリクエストは大歓迎です。[CONTRIBUTING.md](CONTRIBUTING.md)を読んでから送信してください。コードはMITライセンスで、画像はCC BY 4.0ライセンスです。

[MCP Tool Shop](https://mcp-tool-shop.github.io/)によって作成されました。コードとアート：mcp-tool-shop。
