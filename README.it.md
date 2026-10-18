<p align="center">
  <a href="README.ja.md">日本語</a> | <a href="README.zh.md">中文</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.hi.md">हिन्दी</a> | <a href="README.md">English</a> | <a href="README.pt-BR.md">Português (BR)</a>
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

Portlight Bounty è un gioco di commercio. Si acquistano merci, si naviga, si accettano contratti e si cerca di non finire in bancarotta. La frase che riassume il gioco è: commercio, navigazione, prosperità.

Questo repository contiene la versione in Rust di [Portlight](https://github.com/mcp-tool-shop-org/portlight). La simulazione si basa sulle regole. La mappa di Godot 4.7 rappresenta il gioco a cui si gioca. La versione 0.3.2 è l'ultima versione disponibile (solo codice sorgente per sviluppatori) di questo pacchetto.

La mappa del porto, con il mercato aperto ad Al-Manar:

![Mappa del porto ad Al-Manar, mercato aperto](docs/screenshots/chart-1280.png)

Due contratti attivi sulla mappa. I numeri indicano la quantità di merci consegnate, la quantità di merci ancora da consegnare e i giorni rimanenti:

![Mappa dei contratti attivi sopra la mappa di Porto Novo](docs/screenshots/contract-strip-active.png)

## Gioca (dal codice sorgente)

È necessario Rust 1.98.1 (la versione specificata in `rust-toolchain.toml`) e Godot 4.7.2, la versione ufficiale. Il sistema di integrazione continua (CI) per Linux utilizza questa combinazione. È stata eseguita una build per Windows dell'estensione. Non è stata eseguita una build per macOS. Dopo aver clonato il repository, eseguire una sola volta l'importazione delle risorse in modalità "headless" di Godot prima di aprire la mappa:

```
git clone https://github.com/mcp-tool-shop-org/portlight-bounty.git
cd portlight-bounty
cargo build -p portlight-godot
godot --headless --path godot --import --quit
godot --path godot
```

Nuova partita, quindi seleziona "The Merchant" e lascia il nome come "Ada" se desideri seguire il percorso mostrato negli screenshot. Naviga, commercia e accetta un contratto da una delle città portuali. Il salvataggio creerà uno slot di versione 12 o 13 accanto a "Nuova partita" (la versione 13 verrà creata una volta aperto un conto presso una banca). Un capitano personalizzato caricato manterrà il nome, l'argento e il giorno. Il modello personalizzato stesso non sarà salvato, quindi i prezzi dopo il caricamento saranno quelli del mercante. Questo corrisponde al gioco Python.

In mare, la fila dei porti si nasconde. Il giorno successivo fa avanzare il viaggio. Il rapporto giornaliero si apre sopra la mappa quando il giorno ha qualcosa da comunicare.

Il manuale guida attraverso le schermate: [pagina iniziale](https://mcp-tool-shop-org.github.io/portlight-bounty/), quindi Manuale.

## La simulazione

Le casse non vengono disegnate. `portlight-sim` rappresenta le regole, corrispondenti al gioco Python, ad eccezione degli script ottimizzati per Rust elencati. `portlight-cli` esegue uno script e stampa l'istantanea che il sistema di confronto verifica rispetto alla versione Python. `portlight-chart` rappresenta i calcoli dimetrici, ancora senza Godot.

```
cargo run -p portlight-cli -- new --captain merchant --name Ada --seed 42
cargo run -p portlight-cli -- --help
cargo test --locked --workspace --exclude portlight-godot
```

`portlight --help` elenca ogni comando dello script. I codici di uscita sono 0 quando l'esecuzione ha avuto successo, 1 quando la simulazione o un file hanno fallito e 2 quando il comando non è stato riconosciuto. I fallimenti sono rappresentati da frasi. Non c'è telemetria e non c'è un'opzione `--debug`.

Il gioco Python alla commit `9b02494` è la fonte delle regole. Cinque archetipi di capitano hanno un piccolo overlay di prezzi e consumi ottimizzato per Rust (`crates/portlight-sim/data/tuned_overlay.json`), e "The Bounty Hunter" ottiene un bonus del 25% sulle taglie; la compilazione con `--features python-base` disattiva l'overlay in modo che la simulazione corrisponda alla versione Python non modificata.

## Cos'è la versione 0.3.2

Una mappa giocabile che comprende 20 porti e 43 rotte marittime: mercato, contratti, equipaggio con effetti di ruolo e indicatori di morale, maestri di diversi stili di combattimento e istruttori di abilità, un negozio di armi con servizi di manutenzione dell'equipaggiamento e potenziamenti di qualità, il cantiere navale, l'ufficio del porto (compresi i conti in argento di sette porti), caccia e taglie, il diario, le schede riepilogative dei combattimenti e dei contratti, il recupero di navi affondate e rimorchiate, e i combattimenti in mare. Il canone del mondo (Fasi 1-6) è disponibile all'indirizzo `docs/canon/`. Il pacchetto iniziale è la versione 0.3.0 (`godot/assets/landing/MANIFEST.json`).

## Immagini

Le immagini in `godot/assets/` e `docs/screenshots/` sono soggette alla stessa licenza MIT del codice.

La botte, il bitte, il carrello, la cassa e la torcia sono create da `build_prop()` in [ai-rpg-stage](https://github.com/mcp-tool-shop-org/ai-rpg-stage) (`assets/dimetric/camera/dimetric_60_45.py`) utilizzando elementi primitivi di Blender. Questa funzione è stata aggiunta in `55ac9b2`. Il pacchetto ha definito la copia come un "vendor drop". Ciò significa che è stata copiata dal nostro repository, non ricreata.

Le tre bandiere sul molo sono state disegnate utilizzando un'installazione locale di ComfyUI (un peso di Qwen Image e il LoRA di Salt Road) e quindi ritoccate con una mappa di colori fissa.

La versione 0.3.2 viene distribuita senza risorse audio (la mappa viene eseguita senza audio) e non include la voce di ElevenLabs.

## Affidabilità

Il gioco è locale. Non apre una connessione di rete, non ha un account e non invia dati di telemetria. Legge il catalogo incorporato nel file eseguibile, uno script che si specifica e un salvataggio che si crea. Scrive un salvataggio di versione 12 o 13 in una cartella che si sceglie. La cartella predefinita della mappa è `saves/` nella directory di lavoro.

Segnala una vulnerabilità in privato in questo repository. Consulta [SECURITY.md](SECURITY.md). Non è disponibile una casella di posta elettronica separata.

## Supporto

La versione 0.3.2 è la versione supportata. Sono benvenuti segnalazioni di problemi e richieste di modifica. Prima di inviare una richiesta, leggere il file [CONTRIBUTING.md](CONTRIBUTING.md). Il codice e le immagini sono distribuiti con licenza MIT.

Creato da [MCP Tool Shop](https://mcp-tool-shop.github.io/). Codice e grafica: collaboratori di MCP Tool Shop.
