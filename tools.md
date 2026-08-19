# 1. netwatch

## Propósito

Monitor de actividad de red orientado a entender qué procesos están comunicándose, con quién y cuánto tráfico generan.

Debe ser una herramienta interactiva y también usable en scripts.

## Comandos principales

```text
netwatch
netwatch top
netwatch connections
netwatch process <pid>
netwatch host <host>
netwatch port <port>
netwatch watch
```

## Funcionalidades

### Monitor general

```text
netwatch
```

Mostrar en tiempo real:

* conexiones activas
* conexiones nuevas/cerradas
* procesos
* PID
* usuario
* protocolo
* dirección local
* dirección remota
* hostname cuando pueda resolverse
* puerto remoto
* bytes enviados
* bytes recibidos
* velocidad de transferencia
* duración de conexión

Ejemplo conceptual:

```text
PID    PROCESS       PROTO  LOCAL             REMOTE              RX       TX
8123   chrome        TCP    192.168.1.10:5321 142.250.x.x:443      2.1MB    84KB
9211   my-server     TCP    0.0.0.0:8080      192.168.1.20:4312   18MB     3MB
4421   ssh           TCP    192.168.1.10:22   10.0.0.5:55122      1.2MB    2MB
```

### Vista por proceso

```text
netwatch process 8123
```

Mostrar todas las conexiones del proceso.

### Vista por host

```text
netwatch host github.com
```

Mostrar conexiones hacia/desde ese host.

### Vista por puerto

```text
netwatch port 443
```

### Estadísticas

Mostrar:

* conexiones totales
* conexiones activas
* bytes RX/TX
* top processes
* top remote hosts
* top ports
* top protocols

### Filtros

```text
--process <name|pid>
--user <user>
--host <host>
--port <port>
--protocol <tcp|udp>
--local
--remote
--established
--listening
```

### Refresh

```text
--interval 1s
```

### Resolución DNS

```text
--resolve
--no-resolve
```

La resolución debe ser opcional para evitar ralentizar el monitor.

### Output

```text
--json
--csv
--quiet
```

### Persistencia

Opcionalmente:

```text
netwatch record
netwatch replay <file>
```

Guardar eventos de red para analizarlos posteriormente.

## Configuración

Archivo:

```text
~/.config/netwatch/config.yaml
```

Ejemplo:

```yaml
interval: 1s
resolve_hosts: false
show_listening: false

filters:
  protocols:
    - tcp

ui:
  color: true
  max_connections: 100
```

Los parámetros CLI tienen prioridad.

## Requerimientos

* Linux inicialmente.
* Utilizar APIs del sistema directamente.
* No depender de netstat, ss, lsof, etc.
* Identificar procesos asociados cuando el kernel lo permita.
* Manejar correctamente permisos insuficientes.
* No enviar paquetes ni modificar tráfico.
* La herramienta debe ser read-only.
* Bajo overhead.
* Actualización incremental en lugar de reconstruir todo innecesariamente.
* No bloquear la UI esperando DNS.
* Soportar IPv4 e IPv6.
* TCP y UDP.
* Manejar procesos que desaparecen durante el scan.
* TUI responsiva.
* `--json` debe ser estable y apto para scripts.

---

# 2. tunnel

## Propósito

Herramienta unificada para crear y administrar túneles de red.

Inicialmente debe enfocarse en:

* SSH local forwarding.
* SSH remote forwarding.
* TCP forwarding.
* Unix sockets.

No intentaría convertirla inicialmente en un servicio tipo ngrok.

## Funcionalidades

### Local forwarding

```text
tunnel forward 5432 localhost:5432 --via server
```

Conceptualmente:

```text
localhost:5432
      ↓
 SSH tunnel
      ↓
server:5432
```

### Remote forwarding

```text
tunnel reverse 8080 localhost:8080 --via server
```

### SOCKS proxy

```text
tunnel socks 1080 --via server
```

### Unix socket forwarding

```text
tunnel unix /tmp/postgres.sock localhost:5432
```

### Named tunnels

Permitir:

```text
tunnel start database
tunnel stop database
tunnel restart database
tunnel status
```

### Background mode

```text
tunnel start database --background
```

### Persistent tunnels

Reconectar automáticamente cuando:

* SSH se desconecta.
* network cambia.
* server deja de responder.

### Keepalive

Configurable:

```text
--keepalive 30s
```

### Multiplexing

Cuando sea posible, permitir múltiples forwards sobre una conexión SSH.

## Información

```text
tunnel list
tunnel status database
```

Mostrar:

```text
NAME       STATUS      LOCAL       REMOTE
database   connected   :5432       db:5432
web        connected   :8080       web:8080
```

## Logs

```text
tunnel logs database
```

## Test

```text
tunnel test database
```

## Configuración

```text
~/.config/tunnel/config.yaml
```

Ejemplo:

```yaml
tunnels:
  database:
    type: local
    local: 127.0.0.1:5432
    remote: 127.0.0.1:5432

    ssh:
      host: dev.example.com
      user: jp
      port: 22
      identity_file: ~/.ssh/id_ed25519

    reconnect:
      enabled: true
      delay: 5s
      max_delay: 60s

    keepalive: 30s
```

CLI:

```text
tunnel start database
```

Puede sobrescribir:

```text
tunnel start database --local 15432
```

## Requerimientos

* Linux inicialmente.
* SSH mediante implementación nativa o librería apropiada.
* No depender obligatoriamente de ejecutar ssh.
* Soportar autenticación mediante:

  * SSH agent
  * identity files
  * configuración estándar de SSH.
* Nunca almacenar passwords en config.yaml.
* Permisos restrictivos para archivos sensibles.
* Reconexión automática.
* Backoff exponencial.
* Detección de network failures.
* Graceful shutdown.
* Manejar SIGINT/SIGTERM.
* No perder conexiones existentes innecesariamente.
* Logs estructurados.
* Soporte para foreground y background.
* Exit codes adecuados.

---

# 3. dupe

## Propósito

Encontrar y administrar archivos duplicados de forma eficiente y segura.

```text
dupe ~/Downloads
```

## Funcionalidades

### Buscar duplicados

```text
dupe ~/Downloads
```

Agrupar:

```text
Duplicate group #1 — 2.4 GB

  1.2 GB  ./movie-copy.mkv
  1.2 GB  ./backup/movie.mkv
```

### Algoritmo progresivo

No calcular hashes completos de todo inicialmente.

Pipeline:

```text
comparar tamaño
agrupar por tamaño
comparar primeros bytes
calcular hash parcial
calcular hash completo solamente cuando sea necesario.
```

### Hashes

Soportar:

* SHA-256 como mínimo.
* BLAKE3 preferiblemente.

Configurable:

```text
--hash blake3
```

### Estadísticas

Mostrar:

* cantidad de archivos analizados
* duplicados
* espacio recuperable
* tiempo empleado

### Filtros

```text
--min-size 10MB
--max-size 10GB
--include '*.jpg'
--exclude '*.tmp'
--exclude-dir node_modules
```

### Recursión

```text
--depth 3
```

### Symlinks

```text
--follow-symlinks
```

Desactivado por defecto.

### Interactividad

```text
dupe clean
```

Permitir seleccionar qué copia conservar.

### Reemplazo por hard links

Opcional:

```text
dupe dedupe
```

Convertir duplicados apropiados en hard links.

Esto debe ser una funcionalidad avanzada y muy explícita.

### Exportación

```text
--json
--csv
```

## Configuración

```text
~/.config/dupe/config.yaml
```

Ejemplo:

```yaml
hash: blake3
min_size: 1MB
follow_symlinks: false

exclude:
  - node_modules
  - .git
  - target
  - .cache

scan:
  workers: 8
```

## Requerimientos

* Linux inicialmente.
* Procesamiento concurrente.
* Uso eficiente de memoria.
* No cargar archivos completos en RAM.
* Leer archivos en chunks.
* Cancelación limpia.
* Manejar archivos modificados durante el scan.
* No seguir symlinks por defecto.
* Detectar filesystem boundaries.
* Manejar permisos insuficientes.
* Nunca eliminar archivos durante un scan normal.
* `clean` requiere acción explícita.
* Antes de borrar, verificar que el archivo siga siendo el mismo.
* Evitar race conditions entre scan y delete.
* Preservar nombres Unicode.
* Soportar archivos muy grandes.
* Mostrar progreso para scans largos.

---

# 4. pack

## Propósito

Herramienta moderna y unificada para crear, inspeccionar, verificar y extraer archivos comprimidos.

Soportaría inicialmente:

* tar
* tar.gz
* tar.zst
* tar.xz
* zip

Prioridad: zstd como formato moderno y rápido.

## Funcionalidades

### Crear

```text
pack create backup.tar.zst ./project
```

Auto-detectar formato.

También:

```text
--format zstd
```

### Listar

```text
pack list backup.tar.zst
```

Mostrar:

* path
* tamaño
* compressed size cuando esté disponible
* fecha
* permisos

### Extraer

```text
pack extract backup.tar.zst
```

Extraer a destino:

```text
pack extract backup.tar.zst --output ./restore
```

Extraer elementos específicos:

```text
pack extract backup.tar.zst src/main.go
```

### Verificar

```text
pack verify backup.tar.zst
```

Test de integridad.

Debe leer el archive completo cuando sea necesario y detectar corrupción.

### Compresión

```text
--compression 1
--compression 10
```

Dependiendo del backend.

### Parallel compression

Cuando el formato lo permita.

### Progress

```text
--progress
```

### Overwrite behavior

```text
--overwrite
--skip-existing
--interactive
```

### Información

```text
pack info backup.tar.zst
```

Mostrar:

```text
Format          tar.zst
Files           12,482
Original size   4.2 GB
Archive size    812 MB
Compression     80.7%
```

### Streaming

Soportar stdin/stdout:

```text
tar-like-command | pack create - output.tar.zst
```

y:

```text
pack extract archive.tar.zst --stdout path/to/file
```

### Comparación

Opcionalmente:

```text
pack diff archive.tar.zst ./directory
```

## Configuración

```text
~/.config/arc/config.yaml
```

Ejemplo:

```yaml
format: tar.zst

compression:
  level: 3
  threads: 0

output:
  progress: true

extract:
  overwrite: false
  preserve_permissions: true
```

## Requerimientos

* Linux inicialmente.
* No depender de tar, gzip, xz, zstd, zip.
* Utilizar librerías nativas.
* Streaming siempre que sea posible.
* Bajo consumo de memoria.
* Preservar:

  * permisos
  * timestamps
  * ownership cuando tenga permisos
  * symlinks.
* Protección contra path traversal:

  * `../../etc/passwd`
  * nunca debe poder escapar del directorio destino.
* Manejar archivos corruptos correctamente.
* Atomicidad durante creación.
* No sobrescribir archivos por defecto.
* Progreso para operaciones largas.
* Soportar Ctrl-C limpiamente.
* Validar archivos antes de extraer.
* Manejar archivos especiales de forma segura.

---

# 5. sync

## Propósito

Sincronización eficiente y segura entre dos directorios.

```text
sync ./photos /backup/photos
```

Debe ser primero local → local.

Posteriormente podría soportar:

* SSH
* SFTP
* almacenamiento remoto

## Funcionalidades

### Dry run

```text
sync --dry-run ./src /backup/src
```

Mostrar:

```text
+ 42 files
~ 18 files
- 3 files
= 812 files unchanged
```

### Sincronización incremental

Detectar:

* archivos nuevos
* modificados
* eliminados
* renombrados cuando pueda determinarse

### Comparación

Modos:

```text
--compare size
--compare mtime
--compare hash
```

Por defecto:

```text
tamaño
timestamp
hash únicamente cuando sea necesario.
```

### Hash

```text
--hash blake3
```

### Delete

Por defecto no eliminar archivos del destino.

Para hacer espejo:

```text
sync --delete source destination
```

Debe mostrar previamente qué será eliminado.

### Conflictos

Detectar:

```text
CONFLICT
file.txt

Source:
  modified 10:42

Destination:
  modified 10:45
```

Opciones:

```text
--conflict ask
--conflict source
--conflict destination
--conflict newest
```

### Resume

Si una transferencia se interrumpe:

```text
sync --resume
```

### Parallelism

```text
--workers 8
```

### Progress

Mostrar:

* archivos
* bytes
* velocidad
* ETA

### Exclusions

```text
--exclude '*.tmp'
--exclude node_modules
```

### Verification

```text
sync --verify
```

Después de copiar, verificar integridad.

### Snapshots

Opcionalmente:

```text
sync snapshot
sync history
```

## Configuración

```text
~/.config/sync/config.yaml
```

Ejemplo:

```yaml
workers: 8

compare:
  mode: size_mtime

hash:
  algorithm: blake3

exclude:
  - .git
  - node_modules
  - target

conflict:
  strategy: ask

delete:
  enabled: false

verify: false
```

## Requerimientos

* Linux inicialmente.
* Concurrencia controlada.
* No consumir memoria proporcional al tamaño total del árbol.
* Soportar millones de archivos.
* Manejar archivos que cambian durante el sync.
* No eliminar archivos accidentalmente.
* `--delete` debe ser explícito.
* Detectar conflictos.
* Transferencias atómicas cuando sea posible.
* Archivos parcialmente transferidos no deben aparecer como completos.
* Poder reanudar operaciones.
* Manejar interrupciones.
* Preservar metadata configurable.
* Soportar Unicode.
* No seguir symlinks por defecto.
* Prevenir ciclos.
* Mostrar progreso.
* JSON para automatización.

## Futuro

La arquitectura debería permitir:

```text
sync ./data server:/backup/data
```

sin hacer que el primer release dependa de implementar networking.

---

# 6. timeline

Este es el más ambicioso de los seis.

## Propósito

Crear una línea de tiempo de actividad de la máquina, combinando diferentes fuentes de eventos.

La pregunta que debe responder es:

> "¿Qué pasó en mi máquina durante este período?"

## Funcionalidades

### Timeline básica

```text
timeline today
```

Ejemplo:

```text
09:12  process   vscode started
09:14  file      modified ~/project/main.go
09:15  process   build started
09:16  network   connection → github.com
09:18  git       commit 82a91f
09:19  docker    container started
09:21  file      modified ~/project/config.yaml
```

### Fuentes

Inicialmente:

#### Processes

* start
* exit
* PID
* process name
* command line cuando sea posible

#### Filesystem

* create
* modify
* delete
* rename

#### Git

Detectar:

* commits
* branch changes
* repository changes

#### Network

Integración con información de netwatch.

#### Docker

Opcionalmente:

* container started
* stopped
* restarted

#### Shell

Opcionalmente integrar shell history.

## Consultas temporales

```text
timeline today
timeline yesterday
timeline week
timeline "2026-08-18"
```

También:

```text
timeline --from 10:00 --to 12:00
```

## Filtros

```text
timeline --process vscode
timeline --path ~/projects
timeline --repo ~/projects/foo
timeline --type git
timeline --type filesystem
timeline --type network
```

## Buscar

```text
timeline search "docker"
```

## Por proceso

```text
timeline process 1234
```

## Por archivo

```text
timeline file ~/project/main.go
```

## Por proyecto

```text
timeline project ~/project
```

## Agrupación inteligente

En lugar de mostrar miles de eventos:

```text
10:00  main.go modified
10:00  main.go modified
10:01  main.go modified
10:01  main.go modified
...
```

Agrupar:

```text
10:00–10:12
main.go
14 modifications
```

## Sesiones

Detectar actividad relacionada:

```text
09:12–10:03
Project: my-server

  42 file changes
  3 builds
  2 git commits
  1 Docker restart
  18 network connections
```

Esto sería una de las funcionalidades más interesantes del proyecto.

## TUI

```text
timeline
```

TUI navegable:

```text
TIME      TYPE       EVENT
────────────────────────────────────────
09:12     PROCESS    vscode started
09:14     FILE       main.go modified
09:15     BUILD      build started
09:18     GIT        commit 82a91f
09:19     DOCKER     postgres started
```

Teclas:

```text
↑/↓       navigate
Enter     inspect
f         filter
/         search
p         process
q         quit
```

## Persistencia

Los eventos deben almacenarse localmente.

Una base de datos SQLite sería una buena opción.

Ubicación:

```text
~/.local/share/timeline/timeline.db
```

La configuración:

```text
~/.config/timeline/config.yaml
```

## Configuración

Ejemplo:

```yaml
retention:
  days: 90

sources:
  processes: true
  filesystem: true
  git: true
  network: false
  docker: true
  shell: false

filesystem:
  paths:
    - ~/projects
    - ~/Documents

  exclude:
    - node_modules
    - .git
    - target
    - .cache

privacy:
  store_command_lines: false
  store_environment: false
  store_network_hosts: true
```

## Daemon

El collector debería poder ejecutarse como daemon:

```text
timeline daemon
```

Y consultar:

```text
timeline today
```

La UI no debería necesitar mantener el collector abierto.

## Systemd

Opcionalmente instalar:

```text
timeline.service
```

para comenzar automáticamente al iniciar sesión.

## Retención

```text
timeline cleanup
```

Eliminar eventos antiguos según configuración.

También:

```text
timeline --since 30d
```

## Exportación

```text
timeline export --format json
timeline export --format csv
```

## Requerimientos

* Linux inicialmente.
* Daemon de bajo consumo.
* SQLite como almacenamiento.
* Índices apropiados para consultas temporales.
* Event ingestion concurrente.
* Tolerancia a eventos duplicados.
* Manejar eventos fuera de orden.
* Timestamp con suficiente precisión.
* No bloquear la máquina por filesystem monitoring.
* Utilizar APIs del sistema como inotify donde correspondan.
* No registrar contenido de archivos.
* No almacenar secrets.
* No almacenar environment variables por defecto.
* Network data debe ser opcional.
* Shell history debe ser opcional.
* Permisos restrictivos para la database.
* Configuración explícita de privacidad.
* Retención configurable.
* Database migration mechanism desde el primer diseño.
* Collector debe sobrevivir a reinicios.
* Evitar perder todos los eventos si SQLite queda temporalmente indisponible.
* TUI independiente del collector.
* CLI debe poder funcionar aunque el daemon esté detenido para consultas históricas.

## Principio importante

`timeline` no debería intentar capturar absolutamente todo. El objetivo es construir una representación útil de actividad, no un keylogger ni un auditor forense.

---

# Convenciones comunes

Para que las seis herramientas se sientan parte del mismo ecosistema, establecería estos requisitos desde el principio.

## Configuración

Cada programa que tenga configuración persistente:

```text
~/.config/<name>/config.yaml
```

Por ejemplo:

```text
~/.config/netwatch/config.yaml
~/.config/tunnel/config.yaml
~/.config/dupe/config.yaml
~/.config/pack/config.yaml
~/.config/sync/config.yaml
~/.config/timeline/config.yaml
```

## Prioridad de configuración

```text
CLI arguments
    ↓
environment variables (si aplica)
    ↓
config.yaml
    ↓
built-in defaults
```

## Flags comunes

Cuando tengan sentido:

```text
--config <path>
--json
--quiet
--verbose
--no-color
--help
--version
```

## UX

* Colores solamente en TTY.
* Respetar `NO_COLOR`.
* stdout para resultados.
* stderr para errores/progreso.
* Exit codes consistentes.
* Ctrl-C debe detener operaciones limpiamente.
* Unicode soportado.
* Paths con espacios y caracteres especiales deben funcionar.
* Nunca pedir privilegios elevados automáticamente.

## Tecnología

* Utilizar implementaciones nativas o librerías apropiadas para cada herramienta.
* Priorizar bajo consumo de memoria y CPU.
* Preferir APIs del sistema directamente cuando estén disponibles.
* Evitar dependencias externas innecesarias.
* Mantener las herramientas independientes y composables.
* Priorizar portabilidad futura sin sacrificar la integración con Linux.
* Diseñar las interfaces internas para permitir reemplazar componentes sin afectar la CLI.
* Se podría usar bash y python para los scripts. Aunque si es necesario por temas de performance o seguridad puede hacerse en golang o rust
