#!/bin/sh
# Hardware Agent USB - Init script
# Questo script viene eseguito automaticamente all'avvio della live

LOGFILE="/var/log/hardware-agent-boot.log"

log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') $1" | tee -a "$LOGFILE"
}

log "========================================"
log "Hardware Agent USB - Avvio automatico"
log "========================================"

# Attendi che i dischi USB siano rilevati
count=0
while [ $count -lt 10 ]; do
    sleep 1
    count=$((count + 1))
done

# Trova la chiavetta USB (la partizione con label o la root stessa)
# Cerca tra i mountpoint
USB_MNT=""
for dev in /dev/sd*1 /dev/sr*; do
    if [ -e "$dev" ]; then
        mkdir -p /mnt/usb_try
        if mount "$dev" /mnt/usb_try 2>/dev/null; then
            # Se c'è spazio writable, è la nostra chiavetta
            if touch /mnt/usb_try/.hwagent_test 2>/dev/null; then
                rm -f /mnt/usb_try/.hwagent_test
                USB_MNT="/mnt/usb_try"
                log "Chiavetta trovata su $dev -> $USB_MNT"
                break
            else
                umount /mnt/usb_try 2>/dev/null
            fi
        fi
    fi
done

# Se non trova chiavetta, monta tmpfs come fallback (debug)
if [ -z "$USB_MNT" ]; then
    log "[WARN] Chiavetta USB non trovata, uso /tmp fallback"
    USB_MNT="/tmp"
fi

# Esegui l'agent
log "Avvio hardware-agent..."
/usr/local/bin/hardware-agent >> "$LOGFILE" 2>&1

EXIT_CODE=$?
log "Agent terminato con codice $EXIT_CODE"

# Notifica visiva su console
if [ -e /dev/console ]; then
    echo -e "\n\n========================================" > /dev/console
    echo -e "   SCAN COMPLETATO" > /dev/console
    echo -e "   Controlla i file sulla chiavetta" > /dev/console
    echo -e "========================================\n" > /dev/console
fi

# Notifica sonora (se beep disponibile)
if command -v beep >/dev/null 2>&1; then
    beep -f 2000 -l 200
    sleep 0.1
    beep -f 2000 -l 200
    sleep 0.1
    beep -f 2000 -l 200
fi

# Notifica con led tastiera (se disponibile)
echo -e "\x1b[?25l" > /dev/console 2>/dev/null || true

# Se configurato per auto-shutdown, spegni dopo 10 secondi
if [ -f "$USB_MNT/auto-shutdown" ]; then
    log "Auto-shutdown abilitato. Spegnimento tra 10 secondi..."
    sleep 10
    poweroff
fi

# Altrimenti resta attivo per debug
log "Sistema in attesa. Puoi spegnere manualmente."

# Mantieni shell aperta su tty2 per debug
setsid sh -c 'exec sh </dev/tty2 >/dev/tty2 2>&1' &
