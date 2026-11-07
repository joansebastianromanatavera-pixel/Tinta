# Knddle — guía del proyecto

App de escritorio que convierte libros (archivos y links web) en ebooks que en el Kindle se ven **como un libro comprado**, y los entrega solos al dispositivo. Autor: Sebastian. Uso personal, solo contenido propio o gratuito y legítimo. **Nunca** se toca DRM.

Lema: **cada detalle, hasta el más mínimo, importa.** Quince funciones impecables valen más que treinta a medias.

## Stack

| Capa | Tecnología |
|---|---|
| App | **Tauri 2** |
| Núcleo | **Rust** (workspace en `crates/`) |
| Interfaz | **HTML + CSS + JS** vanilla (ES modules), con animaciones de **Motion** |
| Concurrencia | `tokio` + `rayon` |
| PDF | `pdfium-render` (binarios por sistema incluidos) |
| Web con JS | ventana oculta del propio webview de Tauri (no se descarga Chromium) + `readability` |
| EPUB | `epub-builder` |
| AZW3/KF8 | generador **nativo en Rust** (sin Calibre ni Python) |
| KFX | fase 2: Kindle Previewer si está instalado (no se puede redistribuir); fase 3: generador nativo sobre `ion-rs` |
| Correo | `lettre` |
| Credenciales | `keyring` (llavero del sistema) |

### Plataformas

- **Linux**: Debian/Ubuntu (`.deb`), Fedora (`.rpm`), Arch (AUR) y AppImage/Flatpak para todas. Interfaz sobre WebKitGTK.
- **Windows**: MSI/NSIS, interfaz sobre WebView2.
- **macOS**: DMG universal, interfaz sobre WKWebView.

Se diseña para **WebKitGTK**, el motor más débil, y se prueba en NVIDIA. Reglas:

- Animar solo `transform` y `opacity`.
- `backdrop-filter` en máximo 2 capas.

## Arquitectura (hexagonal + SOLID)

```
crates/
  knddle-core/     dominio puro: Book, Chapter, Block, Metadata + pipeline y fases (sin IO)
  knddle-kindle/   perfiles de modelos Kindle (resolución, ppi, formato, MTP/USB)
  (próximos) knddle-pdf, knddle-web, knddle-azw3, knddle-epub, knddle-device, knddle-delivery
app/
  src-tauri/       composición de dependencias + comandos Tauri
  ui/              index.html, styles/ (tokens, motion), components/ (un módulo por componente)
```

- Cada fase del pipeline implementa `Phase`.
- Fuentes, empaquetadores y entregas son traits (puertos); las implementaciones son adaptadores.
- La detección de dispositivos usa un trait `DeviceWatcher` con un adaptador por sistema:
  - Linux: `udev` + `libmtp`
  - Windows: eventos de dispositivo + WPD
  - macOS: DiskArbitration + `libmtp`

## Pipeline de conversión

1. **Ingesta**: PDF, EPUB, MOBI/AZW3 sin DRM, DOCX, ODT, RTF, TXT, MD, HTML, FB2, links.
2. **Análisis**: decide si es texto, escaneo (OCR) o maquetación compleja (modo recorte).
3. **Extracción**: texto, imágenes, fuentes y posiciones.
4. **Estructura**: capítulos, índice de dos niveles (partes y capítulos), notas al pie, listas, tablas.
5. **Libro modelo**: el `Book` interno.
6. **Tipografía y limpieza**:
   - repara ligaduras, guiones invisibles, espacios dobles y saltos de línea en mitad de una frase;
   - comillas y rayas correctas;
   - letra capital y versalitas en el primer párrafo de cada capítulo.
7. **Validación**.
8. **Empaquetado según el dispositivo**: KFX si el modelo lo soporta, si no AZW3 (todos desde la 5ª generación), y MOBI para los antiguos.
9. **Entrega**.

## Alcance 1.0

### Conversión con apariencia de libro comprado

- [ ] AZW3 nativo con `cdeType=EBOK`, ASIN, miniatura de portada en `system/thumbnails` y números de página reales (APNX).
- [ ] Imágenes a la resolución exacta de cada modelo, en escala de grises optimizada para e-ink.
- [ ] Notas al pie emergentes (`epub:type="footnote"`).
- [ ] "Ir al inicio" apuntando al capítulo 1.
- [ ] Sin fuentes incrustadas, para que funcionen Bookerly y los ajustes del Kindle.
- [ ] Letra capital y versalitas al inicio de cada capítulo.
- [ ] Series agrupadas y portada correcta en la biblioteca.
- [ ] Índice de dos niveles.
- [ ] Limpieza tipográfica profunda.
- [ ] Detección automática del idioma (separación silábica y diccionario).
- [ ] Metadatos y portada automáticos desde Open Library; si no aparece, portada minimalista en monocromo.

### Experiencia

- [ ] Solo dos botones: **Insertar** (archivo o arrastrar y soltar) y **Enlace**.
- [ ] Vista previa por modelo: la resolución, el tamaño y el aspecto e-ink del Kindle conectado; fuente Literata.
- [ ] Lista de capítulos editable (renombrar, unir, quitar) con deshacer (Ctrl+Z).
- [ ] Actualizaciones automáticas (Tauri updater).

### Kindle y entrega

- [ ] Detección al conectar, en menos de 500 ms: modelo, número de serie, firmware, resolución.
- [ ] Perfil por dispositivo, con el correo @kindle.com guardado una sola vez en el llavero del sistema.
- [ ] Entrega automática:
  - si el Kindle está conectado, por USB (y expulsión segura al terminar);
  - si no, por correo;
  - sin conexión, a una cola que se entrega al conectar.
- [ ] Aviso de duplicados: si el libro ya está en el Kindle, ofrecer reemplazarlo.
- [ ] Varios Kindle, cada uno con su perfil.

### Después de la 1.0

- KFX nativo (fase 3).
- Gestión de la biblioteca del Kindle.
- Cómics y manga.
- Interfaz en inglés.
- Colecciones del Kindle.

**No se puede imitar**: X-Ray ni funciones que dependen de los servidores de Amazon.

## Diseño

- Inspirado en Apple. **Monocromático, negro primero**: grises y blanco, sin color de acento.
- **Escena**: fondo negro, luz blanca suave desde arriba a la derecha y grano fino tipo arena (`grain.js`, pintado una vez en canvas). Casi estática: nada sigue al mouse.
- **Liquid glass** (`styles/glass.css` + `glass.js`): **el mouse es la luz y el vidrio solo la refleja**. Desenfoque, brillo, grosor, borde que brilla del lado de la luz (más cuanto más cerca) y reflejo que se desliza hacia ella. Resortes con amortiguación crítica.
- **Prohibido**: inclinar, rebotar o desplazar elementos con el mouse (se probó y se descartó por recargado), y colores de acento.
- No usar `clip-path` ni `mask` sobre elementos con `backdrop-filter`: WebKitGTK no recorta bien sus hijos compuestos.
- Esquinas con curvatura continua, como en Apple.
- Tipografía Inter en la interfaz y Literata en la vista previa del libro.
- Todos los valores visuales viven en `app/ui/styles/tokens.css`; ningún componente usa valores sueltos.
- Movimiento:
  - solo resortes, interrumpibles;
  - respeta `prefers-reduced-motion`.
- Cada estado está diseñado: carga, vacío, error y éxito.

## Metas medibles (verificadas en CI)

| Métrica | Meta |
|---|---|
| Arranque en frío | < 400 ms |
| Animaciones | 60 fps estables (120 donde la pantalla lo soporte) |
| PDF de 300 páginas → AZW3 | < 3 s |
| Instalador | < 15 MB |
| Detección del Kindle | < 500 ms |

## Definición de terminado

Una función entra en la 1.0 solo cuando cumple todo esto:

- tiene tests, escritos primero (TDD);
- tiene diseñados sus estados de carga, error y vacío;
- su animación va a 60 fps en WebKitGTK;
- funciona en Linux, Windows y macOS;
- pasa `cargo fmt`, `cargo clippy -D warnings` y `cargo test`.

## Convenciones

- Código en inglés; interfaz en español.
- Principios SOLID, Clean Code y XP:
  - funciones cortas;
  - sin números mágicos;
  - errores con `thiserror`.
- Commits con Conventional Commits.
- Preferir cambios puntuales a reescrituras completas.

## Comandos

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
cd app && npm install && npm run tauri dev   # o `npm run dev` para ver solo la interfaz en http://localhost:1420
```
