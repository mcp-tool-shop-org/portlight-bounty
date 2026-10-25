<p align="center">
  <a href="README.ja.md">日本語</a> | <a href="README.md">English</a> | <a href="README.fr.md">Français</a> | <a href="README.hi.md">हिन्दी</a> | <a href="README.it.md">Italiano</a> | <a href="README.pt-BR.md">Português (BR)</a>
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

Portlight Bounty es un juego de comercio. Compras, navegas, aceptas un contrato e intentas no quedarte sin dinero. La línea debajo del marcador resume el juego: comercio, navegación, prosperidad.

Este repositorio es la versión en Rust de [Portlight](https://github.com/mcp-tool-shop-org/portlight). La simulación son las reglas. El mapa de Godot 4.7 es el juego que juegas. La versión 0.3.2 es la versión actual, que solo incluye el código fuente (se planean versiones binarias precompiladas para una versión menor posterior).

El mapa con el barco atracado, con el mercado abierto en Al-Manar:

![Mapa con el barco atracado en Al-Manar, mercado abierto](docs/screenshots/chart-1280.png)

Dos contratos activos en la lista. Los números indican la carga entregada, la carga pendiente y los días restantes:

![Lista de contratos activos sobre el mapa en Porto Novo](docs/screenshots/contract-strip-active.png)

## Jugar (desde el código fuente)

Esta versión se distribuye únicamente como código fuente (todavía no hay binarios precompilados ni configuraciones de exportación). Necesitas Rust 1.98.1 (la versión especificada en `rust-toolchain.toml`) y Godot 4.7.2, la versión oficial. El entorno de integración continua (CI) de Linux utiliza esta combinación. Se ha ejecutado una versión de Windows de la extensión. No se ha ejecutado una versión de macOS. Clona el repositorio y ejecuta la importación de recursos sin interfaz gráfica de Godot una vez antes de abrir el mapa:

```
git clone https://github.com/mcp-tool-shop-org/portlight-bounty.git
cd portlight-bounty
cargo build -p portlight-godot
godot --headless --path godot --import --quit
godot --path godot
```

Nuevo juego, luego "The Merchant" (El Mercader), y deja el nombre como Ada si quieres que el viaje sea el que se muestra en las capturas de pantalla. Navega, comercia y acepta un contrato del puerto. Guardar crea una ranura de la versión 12 o 13 junto a "Nuevo juego" (la versión 13 se crea una vez que se abre una cuenta en la casa de contabilidad). Un capitán personalizado cargado conserva el nombre, la plata y el día. La plantilla personalizada en sí no se guarda, por lo que los precios después de cargar son los del mercader. Esto coincide con el juego de Python.

En el mar, la fila de puertos se oculta. El día siguiente avanza el viaje. El informe del día se abre sobre el mapa cuando el día tiene algo que decir.

El manual guía a través de las pantallas: [página de inicio](https://mcp-tool-shop-org.github.io/portlight-bounty/), luego Manual.

## La simulación

Las cajas no se dibujan. `portlight-sim` son las reglas, que coinciden con el juego de Python, excepto por los scripts ajustados para Rust que se indican. `portlight-cli` ejecuta un script e imprime la instantánea que el sistema de comparación utiliza para compararla con la versión de Python. `portlight-chart` son los cálculos dimétricos, que todavía no utilizan Godot.

```
cargo run -p portlight-cli -- new --captain merchant --name Ada --seed 42
cargo run -p portlight-cli -- --help
cargo test --locked --workspace --exclude portlight-godot
```

`portlight --help` enumera cada comando de script. Los códigos de salida son 0 cuando la ejecución fue correcta, 1 cuando la simulación o un archivo fallaron y 2 cuando el comando no se entendió. Los fallos se expresan en frases. No hay telemetría ni un interruptor `--debug`.

El juego de Python en el commit `9b02494` es la fuente de las reglas. Cinco arquetipos de capitán tienen una pequeña capa de precios y desgaste ajustada para Rust (`crates/portlight-sim/data/tuned_overlay.json`), y "The Bounty Hunter" (El Cazador de Recompensas) obtiene un +25% en las recompensas; la compilación con `--features python-base` desactiva la capa, por lo que la simulación coincide con la versión de Python sin modificar.

## De qué se trata la versión 0.3.2

Un mapa jugable que abarca 20 puertos y 43 rutas marítimas: mercado, contratos, tripulación con efectos de rol y líneas de moral, maestros de estilos de lucha y entrenadores de habilidades, una tienda de mercader de armería con mantenimiento de equipo y mejoras de calidad de armero, el astillero, el escritorio del puerto (incluidas las cuentas de plata de la casa de contabilidad en siete puertos), caza y recompensas, el diario, tarjetas de resumen de combate y contratos, hundimiento y recuperación de remolque, y combates marítimos. El canon del mundo (Fases 1-6) se encuentra en `docs/canon/`. El paquete de inicio es el manifiesto 0.3.0 (`godot/assets/landing/MANIFEST.json`).

## Imágenes

Las imágenes (cada archivo de imagen en el repositorio) fueron creadas por mcp-tool-shop y se licencian bajo CC BY 4.0: cualquiera puede usarlas y adaptarlas, incluso con fines comerciales, siempre que se dé crédito a "mcp-tool-shop". Todo lo demás está bajo la licencia MIT. Consulta [LICENSE](LICENSE).

El barril, el poste de amarre, el carro, la caja y la antorcha fueron creados por `build_prop()` en [ai-rpg-stage](https://github.com/mcp-tool-shop-org/ai-rpg-stage) (`assets/dimetric/camera/dimetric_60_45.py`) a partir de primitivas de Blender. Esta función se implementó en `55ac9b2`. El manifiesto denominó a la copia como un "vendor drop" (elemento de proveedor). Esto significaba que se copió de nuestro propio repositorio, no que se recreó.

Las tres banderas del muelle fueron pintadas en una configuración local de ComfyUI (un peso de Qwen Image y el Salt Road LoRA) y luego se volvieron a aplicar con un mapa de colores fijo.

La versión 0.3.2 se distribuye sin archivos de audio (el mapa se ejecuta en silencio) y no tiene voz en ElevenLabs.

## Confianza

El juego es local. No abre una conexión de red, no tiene una cuenta y no envía telemetría. Lee el catálogo incluido en el archivo binario, un script que tú nombras y una partida que tú nombras. Escribe una partida de la versión 12 o 13 en una carpeta que tú eliges. La carpeta predeterminada del mapa es `saves/` dentro del directorio de trabajo.

Informa de una vulnerabilidad de forma privada en este repositorio. Consulta [SECURITY.md](SECURITY.md). No hay una bandeja de entrada separada.

## Soporte

0.3.2 es la versión con soporte. Los problemas y las solicitudes de incorporación son bienvenidas. Lee [CONTRIBUTING.md](CONTRIBUTING.md) antes de enviarla. El código está bajo la licencia MIT y las imágenes están bajo la licencia CC BY 4.0.

Creado por [MCP Tool Shop](https://mcp-tool-shop.github.io/). Código y arte: mcp-tool-shop.
