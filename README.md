# Hardware Agent USB

Agente diagnostico hardware standalone su chiavetta USB. **Zero installazione, zero interazione.**

Inserisci la chiavetta in un server senza OS, accendi, aspetta 30-60 secondi: trovi il report PDF e TXT sulla stessa chiavetta.

## Cosa raccoglie (100% read-only)

- **System Info**: vendor, modello, serial number, UUID, asset tag
- **BIOS**: vendor, versione, data
- **CPU**: modello, socket, core, thread, architettura
- **Memoria**: totale, dettaglio DIMM (slot, speed, SN, rank)
- **Storage**: dischi rilevati con SMART (health, ore accensione, temperatura, serial)
- **Rete**: interfacce, MAC address, driver, speed, duplex
- **PCI**: dispositivi PCIe con driver/kernel
- **Sensori**: temperature, voltaggi, RPM ventole (lm-sensors / hwmon)
- **IPMI**: sensori BMC, log SEL eventi (se disponibile)

## Output generati automaticamente

Nella root della chiavetta USB:

- `Report_Server_<SERIAL>_<timestamp>.txt` — report in testo semplice
- `Report_Server_<SERIAL>_<timestamp>.html` — report formattato (apribile in browser)
- `Report_Server_<SERIAL>_<timestamp>.pdf` — report PDF professionale (se `wkhtmltopdf` disponibile)
- `COMPLETATO.txt` — conferma con timestamp e riepilogo

## Build rapida (da WSL2 / Linux)

**Prerequisiti**: Docker oppure WSL2 con Ubuntu/Debian, Rust, `cargo`

```bash
# Clona o entra nella directory del progetto
cd hardware-agent-usb

# Build completo: compila agent + crea ISO live
bash live-build/build.sh
```

Al termine trovi l'ISO in `live-build/build/hardware-agent-live.iso`.

### Flash chiavetta USB

Da Windows (PowerShell con admin):

```powershell
# Sostituisci X: con la lettera della tua chiavetta
$usb = "X:"
# Usa Rufus o Ventoy, o in alternativa con balenaEtcher
```

Consigliato **Rufus** (https://rufus.ie) o **Ventoy** (https://ventoy.net).

## Uso operativo

1. **Flash** l'ISO su chiavetta USB (min 512MB)
2. **Inserisci** la chiavetta nel server senza OS
3. **Accendi** il server → premi F11/F12 per Boot Menu → seleziona USB
4. **Attendi** 30-60 secondi (lo schermo mostra "Hardware Agent USB v0.1.0")
5. **Senti 3 beep** (o vedi "SCAN COMPLETATO")
6. **Spegni** il server, togli la chiavetta
7. **Inseriscila nel tuo PC Windows**: trovi TXT, HTML, PDF e `COMPLETATO.txt`

## Sicurezza

- **Solo lettura**: nessuna scrittura sui dischi del server
- **Nessuna modifica**: non tocca BIOS, BMC, RAID, o configurazioni
- **Nessun test invasivo**: non esegue stress test, memtest, o scrittura su storage
- **Open source**: il codice è leggibile e verificabile

## Struttura progetto

```
hardware-agent-usb/
  Cargo.toml              # Progetto Rust
  src/
    main.rs               # Entry point, rilevamento chiavetta, orchestrazione
    models.rs             # Strutture dati (serde)
    report.rs             # Generazione TXT e HTML
    collectors/
      mod.rs
      dmi.rs              # /sys/class/dmi/id, /proc/cpuinfo, dmidecode
      ipmi.rs             # ipmitool (mc info, sensor list, sel list)
      smart.rs            # smartctl --scan + --all
      pci.rs              # lspci, fallback /sys/bus/pci/devices
      sensors.rs          # sensors -u, fallback /sys/class/hwmon
      network.rs          # /sys/class/net
  live-build/
    build.sh              # Script build ISO completa
    init.sh               # Init automatico dentro la live
```

## Tech Stack

- **Agent**: Rust (serde, chrono), binario statico `musl`
- **Live OS**: Alpine Linux v3.19 (~130MB)
- **Report**: HTML/CSS inline + wkhtmltopdf (opzionale)
- **ISO**: syslinux/isolinux, boot BIOS/UEFI

## Note

- Se il server non ha IPMI (BMC non alimentato), l'agent salta quella sezione senza errori.
- Se `wkhtmltopdf` non è presente nella live, viene generato comunque HTML apribile nel browser.
- L'agent rileva automaticamente la chiavetta USB come destinazione output.

## Licenza

MIT
