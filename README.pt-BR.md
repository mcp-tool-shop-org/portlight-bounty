<p align="center">
  <a href="README.ja.md">日本語</a> | <a href="README.zh.md">中文</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.hi.md">हिन्दी</a> | <a href="README.it.md">Italiano</a> | <a href="README.md">English</a>
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

Portlight Bounty é um jogo de comércio. Você compra, navega, aceita um contrato e tenta não ficar sem dinheiro. A linha abaixo da marca resume o jogo: comércio, navegação, prosperidade.

Este repositório é a versão em Rust de [Portlight](https://github.com/mcp-tool-shop-org/portlight). A simulação são as regras. O mapa do Godot 4.7 é o jogo que você joga. A versão 0.3.0 é a versão atual deste conjunto.

O mapa do porto, com o mercado aberto em Al-Manar:

![Mapa do porto em Al-Manar, mercado aberto](docs/screenshots/chart-1280.png)

Dois contratos ativos na lista. Os números representam a carga entregue, a carga pendente e os dias restantes:

![Lista de contratos ativos sobre o mapa em Porto Novo](docs/screenshots/contract-strip-active.png)

## Jogar

Você precisa do Rust 1.98.1 (o indicado em `rust-toolchain.toml`) e do Godot 4.7.2, a versão oficial. O Linux CI usa esse conjunto. Uma versão para Windows da extensão foi executada. Uma versão para macOS não foi.

```
cargo build -p portlight-godot
godot --headless --path godot --import --quit
godot --path godot
```

Novo jogo, depois "The Merchant" e deixe o nome como Ada se quiser que a jornada mostre as capturas de tela. Navegue, comercie e aceite um contrato na linha do porto. Salvar cria um arquivo de versão 12 ou versão 13 ao lado de "Novo jogo" (versão 13 quando uma conta da casa de câmbio for aberta). Um capitão personalizado carregado mantém o nome, a prata e o dia. O modelo personalizado em si não está no arquivo salvo, portanto, os preços após o carregamento são os do comerciante. Isso corresponde ao jogo em Python.

No mar, a linha do porto desaparece. O dia seguinte avança a jornada. O relatório do dia é aberto sobre o mapa quando o dia tem algo a dizer.

O manual explica as telas: [página inicial](https://mcp-tool-shop-org.github.io/portlight-bounty/), depois Manual.

## A simulação

As caixas não são desenhadas. `portlight-sim` são as regras, correspondendo ao jogo em Python, exceto pelos scripts ajustados em Rust listados. `portlight-cli` executa um script e imprime o instantâneo que o sistema de comparação usa para comparar com o Python. `portlight-chart` é a matemática dimétrica, ainda sem o Godot.

```
cargo run -p portlight-cli -- new --captain merchant --name Ada --seed 42
cargo run -p portlight-cli -- --help
cargo test --locked --workspace --exclude portlight-godot
```

`portlight --help` lista todos os comandos de script. Os códigos de saída são 0 quando a execução foi bem-sucedida, 1 quando a simulação ou um arquivo falhou e 2 quando o comando não foi compreendido. As falhas são frases. Não há telemetria e não há opção `--debug`.

O jogo em Python no commit `9b02494` é a fonte das regras. Cinco arquétipos de capitão carregam uma pequena sobreposição de preços e consumo ajustada em Rust (`crates/portlight-sim/data/tuned_overlay.json`), e "The Bounty Hunter" oferece +25% nas recompensas; a construção com `--features python-base` desativa a sobreposição para que a simulação corresponda ao Python não modificado.

## O que é a 0.3.0

Um mapa jogável com 20 portos e 43 rotas marítimas: mercado, contratos, tripulação com efeitos de função de companheiro e linhas de moral, mestres de estilo de luta e treinadores de habilidades, uma loja de mercadorias da Armaria com manutenção de equipamentos e melhorias de qualidade de ferreiro, o pátio, o escritório do porto (incluindo contas de prata da casa de câmbio em sete portos), caça e recompensas, o diário, cartões de resumo de combate e contrato, naufrágio e recuperação de reboque e combates marítimos. O cânone do mundo (Fases 1–6) está em `docs/canon/`. O pacote inicial é o manifesto 0.4.3 (`godot/assets/landing/MANIFEST.json`).

## Imagens

As imagens em `godot/assets/` e `docs/screenshots/` estão sob a mesma licença MIT que o código.

O barril, o poste, o carrinho, a caixa e a tocha são construídos por `build_prop()` em [ai-rpg-stage](https://github.com/mcp-tool-shop-org/ai-rpg-stage) (`assets/dimetric/camera/dimetric_60_45.py`) a partir de primitivas do Blender. Essa função foi adicionada em `55ac9b2`. O manifesto chamou a cópia de "item de vendedor". Isso significou copiado de nosso próprio repositório, não recriado.

As três bandeiras do cais foram pintadas em uma configuração local do ComfyUI (um peso de imagem Qwen e o LoRA Salt Road) e, em seguida, retocadas com um mapa de cores fixo.

Não há voz da ElevenLabs nesta versão.

## Confiança

O jogo é local. Ele não abre uma conexão de rede, não tem uma conta e não envia telemetria. Ele lê o catálogo incorporado no binário, um script que você nomeia e um arquivo salvo que você nomeia. Ele grava um arquivo salvo de versão 12 ou versão 13 em uma pasta que você escolhe. A pasta padrão do mapa é `saves/` no diretório de trabalho.

Relate uma vulnerabilidade em particular neste repositório. Consulte [SECURITY.md](SECURITY.md). Não há caixa de correio separada.

## Suporte

A 0.3.0 é a versão com suporte. Problemas e solicitações de pull são bem-vindos. Leia [CONTRIBUTING.md](CONTRIBUTING.md) antes de enviar um. O código e as imagens são MIT.

Criado por [MCP Tool Shop](https://mcp-tool-shop.github.io/). Código e arte: colaboradores da MCP Tool Shop.
