<p align="center">
  <a href="README.ja.md">日本語</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.hi.md">हिन्दी</a> | <a href="README.md">English</a> | <a href="README.pt-BR.md">Português (BR)</a>
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

Portlight Bounty è un gioco di commercio. Si acquistano merci, si naviga, si accettano contratti e si cerca di non finire in bancarotta. La frase che descrive il gioco è: commercio, navigazione, prosperità.

Questo repository contiene la versione in Rust di [Portlight](https://github.com/mcp-tool-shop-org/portlight). La simulazione rappresenta le regole del gioco. La mappa di Godot 4.7 è il gioco vero e proprio. La versione 0.3.2 è l'attuale **versione per sviluppatori, disponibile solo come codice sorgente** (l'esportazione di binari precompilati è prevista per una successiva versione minore).

La mappa del porto, con il mercato aperto ad Al-Manar:

![Mappa del porto ad Al-Manar, mercato aperto](docs/screenshots/chart-1280.png)

Due contratti attivi sulla mappa. I numeri indicano la quantità di merci consegnate, la quantità di merci ancora da consegnare e i giorni rimanenti:

![Mappa dei contratti attivi sopra la mappa di Porto Novo](docs/screenshots/contract-strip-active.png)

## Gioca (a partire dal codice sorgente)

Questa versione viene distribuita solo come codice sorgente (non sono ancora disponibili binari precompilati o impostazioni di esportazione). È necessario Rust 1.98.1 (la versione specificata in `rust-toolchain.toml`) e Godot 4.7.2, la versione ufficiale. L'ambiente CI di Linux utilizza questa combinazione. È stata eseguita una build per Windows dell'estensione. Una build macOS non è stata eseguita. Clona il repository ed esegui l'importazione di asset in modalità "headless" di Godot una volta prima di aprire la mappa:

```
git clone https://github.com/mcp-tool-shop-org/portlight-bounty.git
cd portlight-bounty
cargo build -p portlight-godot
godot --headless --path godot --import --quit
godot --path godot
```

Nuova partita, quindi seleziona "The Merchant" e lascia il nome come "Ada" se desideri che la partita mostri gli screenshot. Naviga, commercia e accetta un contratto dal porto. Il salvataggio creerà uno slot di versione 12 o 13 accanto a "Nuova partita" (la versione 13 verrà creata una volta aperto un conto presso una filiale). Un capitano personalizzato caricato manterrà il nome, l'argento e il giorno. Il modello personalizzato stesso non sarà salvato, quindi i prezzi dopo il caricamento saranno quelli del mercante. Questo corrisponde al gioco Python.

In mare, la fila dei porti si nasconde. Il giorno successivo fa avanzare la partita. Il rapporto giornaliero si apre sopra la mappa quando il giorno ha qualcosa da comunicare.

Il manuale guida attraverso le schermate: [pagina di destinazione](https://mcp-tool-shop-org.github.io/portlight-bounty/), quindi Manuale.

## La simulazione

Le casse non vengono disegnate. `portlight-sim` rappresenta le regole, corrispondenti al gioco Python, ad eccezione degli script ottimizzati per Rust elencati. `portlight-cli` esegue uno script e stampa l'istantanea che il sistema di confronto verifica rispetto alla versione Python. `portlight-chart` rappresenta i calcoli dimetrici, ancora senza l'utilizzo di Godot.

```
cargo run -p portlight-cli -- new --captain merchant --name Ada --seed 42
cargo run -p portlight-cli -- --help
cargo test --locked --workspace --exclude portlight-godot
```

`portlight --help` elenca ogni comando dello script. I codici di uscita sono 0 quando l'esecuzione ha avuto successo, 1 quando la simulazione o un file hanno fallito e 2 quando il comando non è stato riconosciuto. I fallimenti sono rappresentati da frasi. Non è presente telemetria né un'opzione `--debug`.

Il gioco Python alla commit `9b02494` è la fonte delle regole. Cinque archetipi di capitano includono una piccola sovrapposizione di prezzi e consumi ottimizzata per Rust (`crates/portlight-sim/data/tuned_overlay.json`), e "The Bounty Hunter" ottiene un bonus del 25% sulle taglie; la compilazione con `--features python-base` disattiva la sovrapposizione in modo che la simulazione corrisponda alla versione Python non modificata.

## Cos'è la versione 0.3.2

Una mappa giocabile che comprende 20 porti e 43 rotte marittime: mercato, contratti, equipaggio con effetti di ruolo e linee di morale, maestri di diversi stili di combattimento e istruttori di abilità, un negozio di armi con manutenzione dell'equipaggiamento e potenziamenti di qualità, il cantiere navale, l'ufficio del porto (inclusi i conti bancari in sette porti), taglie e cacce, il diario, le schede riepilogative dei combattimenti e dei contratti, affondamenti e recupero di navi trainate e combattimenti in mare. Il canone del mondo (Fasi 1-6) è disponibile all'indirizzo `docs/canon/`. Il pacchetto di lancio è la versione 0.3.0 (`godot/assets/landing/MANIFEST.json`).

## Immagini

Le immagini (ogni file immagine nel repository) sono state create da mcp-tool-shop e sono concesse in licenza CC BY 4.0: chiunque può utilizzarle e adattarle, anche per scopi commerciali, a condizione che venga attribuito il merito a "mcp-tool-shop". Tutto il resto è con licenza MIT. Consulta [LICENSE](LICENSE).

La botte, il bitte, il carrello, la cassa e la torcia sono stati creati da `build_prop()` in [ai-rpg-stage](https://github.com/mcp-tool-shop-org/ai-rpg-stage) (`assets/dimetric/camera/dimetric_60_45.py`) utilizzando elementi primitivi di Blender. Questa funzione è stata aggiunta nella versione `55ac9b2`. Il pacchetto ha definito la copia come un "vendor drop". Ciò significava che era stata copiata dal nostro repository, non ricreata.

Le tre bandiere sul molo sono state dipinte su un'installazione locale ComfyUI (un peso di Qwen Image e la Salt Road LoRA) e quindi ritoccate con una mappa di colori fissa.

La versione 0.3.2 viene distribuita senza asset audio (la mappa è silenziosa) e non ha una voce ElevenLabs.

## Affidabilità

Il gioco è locale. Non apre una connessione di rete, non ha un account e non invia dati di telemetria. Legge il catalogo incorporato nel binario, uno script che si specifica e un salvataggio che si specifica. Scrive un salvataggio di versione 12 o 13 in una cartella che si sceglie. La cartella predefinita della mappa è `saves/` nella directory di lavoro.

Segnala una vulnerabilità in privato in questo repository. Consulta [SECURITY.md](SECURITY.md). Non è disponibile una casella di posta elettronica separata.

## Supporto

La versione 0.3.2 è la versione supportata. Sono benvenuti problemi e richieste di pull. Leggi [CONTRIBUTING.md](CONTRIBUTING.md) prima di inviarne una. Il codice è con licenza MIT e le immagini sono con licenza CC BY 4.0.

Creato da [MCP Tool Shop](https://mcp-tool-shop.github.io/). Codice e grafica: mcp-tool-shop.
