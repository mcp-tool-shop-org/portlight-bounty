<p align="center">
  <a href="README.ja.md">日本語</a> | <a href="README.zh.md">中文</a> | <a href="README.es.md">Español</a> | <a href="README.md">English</a> | <a href="README.hi.md">हिन्दी</a> | <a href="README.it.md">Italiano</a> | <a href="README.pt-BR.md">Português (BR)</a>
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

Portlight Bounty est un jeu de commerce. Vous achetez, vous naviguez, vous acceptez un contrat et vous essayez de ne pas finir à découvert. La ligne sous le repère résume le jeu : commerce, navigation, prospérité.

Ce dépôt est la version Rust de [Portlight](https://github.com/mcp-tool-shop-org/portlight). La simulation est basée sur les règles. Le diagramme Godot 4.7 représente le jeu auquel vous jouez. La version 0.3.2 est la version actuelle (code source uniquement pour les développeurs) de cet ensemble.

La carte du port, avec le marché ouvert à Al-Manar :

![Carte du port à Al-Manar, marché ouvert](docs/screenshots/chart-1280.png)

Deux contrats actifs affichés sur la carte. Les chiffres représentent les marchandises livrées, les marchandises dues et le nombre de jours restants :

![Bandeau des contrats actifs sur la carte de Porto Novo](docs/screenshots/contract-strip-active.png)

## Jouer (depuis le code source)

Vous avez besoin de Rust 1.98.1 (la version spécifiée dans `rust-toolchain.toml`) et de Godot 4.7.2, la version officielle. L’intégration continue (CI) pour Linux utilise cet ensemble. Une version Windows de l’extension a été exécutée. Une version macOS n’a pas été exécutée. Sur un clone récent, exécutez une fois l’importation d’actifs en mode « headless » de Godot avant d’ouvrir le diagramme :

```
git clone https://github.com/mcp-tool-shop-org/portlight-bounty.git
cd portlight-bounty
cargo build -p portlight-godot
godot --headless --path godot --import --quit
godot --path godot
```

Nouveau jeu, puis sélectionnez « The Merchant », et laissez le nom tel quel (Ada) si vous souhaitez que le voyage corresponde aux captures d’écran. Naviguez, commercez et acceptez un contrat dans la rangée des ports. La sauvegarde crée un fichier de version 12 ou 13 à côté de « Nouveau jeu » (version 13 une fois qu’un compte de comptoir est ouvert). Un capitaine personnalisé chargé conserve le nom, l’argent et le jour. Le modèle personnalisé lui-même n’est pas inclus dans la sauvegarde, de sorte que les prix après le chargement correspondent à ceux du marchand. Cela correspond au jeu Python.

En mer, la rangée des ports disparaît. Le jour suivant fait progresser le voyage. Le rapport du jour s’affiche sur la carte lorsque le jour a quelque chose à dire.

Le manuel explique les écrans : [page d’accueil](https://mcp-tool-shop-org.github.io/portlight-bounty/), puis Manuel.

## La simulation

Les caisses ne sont pas affichées. `portlight-sim` contient les règles, qui correspondent au jeu Python, à l’exception des scripts Rust optimisés mentionnés. `portlight-cli` exécute un script et affiche l’instantané que le système de comparaison vérifie par rapport à Python. `portlight-chart` contient les calculs dimétriques, sans Godot.

```
cargo run -p portlight-cli -- new --captain merchant --name Ada --seed 42
cargo run -p portlight-cli -- --help
cargo test --locked --workspace --exclude portlight-godot
```

`portlight --help` liste toutes les commandes de script. Les codes de sortie sont 0 lorsque l’exécution a réussi, 1 lorsque la simulation ou un fichier a échoué, et 2 lorsque la commande n’a pas été comprise. Les erreurs sont des phrases. Il n’y a pas de télémétrie et pas d’option `--debug`.

Le jeu Python à la version `9b02494` est la source des règles. Cinq archétypes de capitaine possèdent une petite superposition Rust optimisée pour les prix et la consommation (`crates/portlight-sim/data/tuned_overlay.json`), et The Bounty Hunter obtient +25 % sur les primes ; la compilation avec `--features python-base` désactive la superposition afin que la simulation corresponde à la version Python non modifiée.

## Ce qu’est la version 0.3.2

Un diagramme jouable couvrant 20 ports et 43 routes maritimes : marché, contrats, équipage avec des effets de rôle et des lignes de moral, maîtres de différents styles de combat et entraîneurs de compétences, un magasin d’armurerie proposant l’entretien de l’équipement et des améliorations de qualité de forgeron, le chantier, le bureau du port (y compris les comptes d’argent des comptoirs dans sept ports), la chasse et les primes, le journal, les cartes récapitulatives des combats et des contrats, la récupération des navires coulés et des navires remorqués, et les combats maritimes. Le canon de l’univers (phases 1 à 6) est disponible sous `docs/canon/`. Le paquet d’installation initial est la version 0.3.0 (`godot/assets/landing/MANIFEST.json`).

## Images

Les images dans `godot/assets/` et `docs/screenshots/` sont soumises à la même licence MIT que le code.

Le tonneau, le bollard, le chariot, la caisse et la torche sont créés par `build_prop()` dans [ai-rpg-stage](https://github.com/mcp-tool-shop-org/ai-rpg-stage) (`assets/dimetric/camera/dimetric_60_45.py`) à partir de primitives Blender. Cette fonction a été ajoutée dans `55ac9b2`. Le paquet a qualifié la copie de « vendor drop ». Cela signifie qu’elle a été copiée à partir de notre propre dépôt, et non recréée.

Les trois drapeaux du quai ont été peints sur une configuration locale de ComfyUI (un poids Qwen Image et le LoRA Salt Road), puis retouchés avec une carte de couleurs fixe.

La version 0.3.2 est livrée sans fichiers audio (le diagramme s’exécute sans son) et ne contient pas de voix ElevenLabs.

## Confiance

Le jeu est local. Il n’ouvre pas de connexion réseau, il n’a pas de compte et il n’envoie pas de télémétrie. Il lit le catalogue intégré au fichier binaire, un script que vous nommez et une sauvegarde que vous nommez. Il écrit une sauvegarde de version 12 ou 13 dans un dossier que vous choisissez. Le dossier par défaut de la carte est `saves/` dans le répertoire de travail.

Signalez une vulnérabilité en privé dans ce dépôt. Consultez [SECURITY.md](SECURITY.md). Il n’y a pas de boîte de réception séparée.

## Assistance

La version 0.3.2 est la version prise en charge. Les problèmes et les demandes de contribution sont les bienvenus. Lisez le fichier [CONTRIBUTING.md](CONTRIBUTING.md) avant de les soumettre. Le code et les images sont sous licence MIT.

Créé par [MCP Tool Shop](https://mcp-tool-shop.github.io/). Code et illustrations : contributeurs de MCP Tool Shop.
