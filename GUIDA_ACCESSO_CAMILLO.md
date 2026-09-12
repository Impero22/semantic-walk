# Guida alla Connessione e Configurazione Ambiente di Sviluppo (Camillo)

Benvenuto a bordo! Questo documento contiene tutte le istruzioni dettagliate, **partendo da zero**, per configurare il tuo computer (sia esso **Windows**, **Linux** o **macOS**), connetterti alla tua shell sul server Lyra e iniziare a lavorare sul progetto **`semantic-geo`** in coppia con Iris.

---

## 1. Parametri di Connessione

* **Host / Indirizzo:** `uncino.eu`
* **Porta SSH:** `2222`
* **Nome Utente:** `camillo`
* **Metodo di Autenticazione:** **Esclusivamente chiave crittografica SSH (ED25519)**
  *(Nota: l'accesso con password è disabilitato per motivi di sicurezza).*
* **Cartella Progetto Condivisa:** `/home/camillo/Sviluppo/Progetti/semantic-geo`
* **Sistema Operativo:** CachyOS (Arch Linux con kernel ottimizzato ad alte prestazioni)
* **Accelerazione Hardware:** GPU AMD Radeon 780M dedicata/condivisa (`/dev/dri`, `/dev/kfd`)

---

## 2. Passo 1: Generare la Coppia di Chiavi SSH sul tuo PC

Prima di poterti connettere, devi possedere una coppia di chiavi SSH (una **chiave privata** che rimane sempre e solo sul tuo PC, e una **chiave pubblica** da fornire all'amministratore).

### Se usi Windows (Windows 10 / Windows 11)

1. Apri **Terminale Windows** oppure **PowerShell** (premi il tasto `Windows`, digita `powershell` e premi `Invio`).
2. Esegui il comando:
   ```powershell
   ssh-keygen -t ed25519 -C "camillo"
   ```
3. Alla domanda *“Enter file in which to save the key”*, premi semplicemente **Invio** (salverà la chiave nella posizione predefinita `C:\Users\<tuo_nome>\.ssh\id_ed25519`).
4. Se lo desideri, inserisci una **passphrase** per proteggere la chiave (o premi due volte **Invio** per lasciarla senza passphrase).
5. Ora visualizza e copia la tua **chiave pubblica**:
   ```powershell
   Get-Content ~/.ssh/id_ed25519.pub
   ```
   *(In alternativa puoi aprirla con il blocco note: `notepad ~/.ssh/id_ed25519.pub`)*

---

### Se usi Linux o macOS

1. Apri il tuo **Terminale**.
2. Esegui il comando:
   ```bash
   ssh-keygen -t ed25519 -C "camillo"
   ```
3. Premi **Invio** per accettare il percorso predefinito (`~/.ssh/id_ed25519`).
4. Scegli se impostare una passphrase o premi due volte **Invio** per continuare senza passphrase.
5. Visualizza e copia la tua **chiave pubblica**:
   ```bash
   cat ~/.ssh/id_ed25519.pub
   ```

---

## 3. Passo 2: Inviare la Chiave Pubblica all'Amministratore

Copia l'intero testo visualizzato nel passaggio precedente:
* Sarà una singola riga di testo che inizia per `ssh-ed25519 AAAA...` e termina con `camillo`.
* Invia questa riga a Federico (tramite chat sicura, email o messaggio).
* **IMPORTANTE:** Non inviare **mai** il file senza estensione `id_ed25519` (quella è la tua chiave segreta/privata); invia solo il file `.pub`!

> **Nota per l'amministratore:** per registrare la chiave di Camillo su Lyra, basta aggiungere la riga in `/mnt/nvme/cachyos/home/camillo/.ssh/authorized_keys` e verificare che i permessi siano `600` e il proprietario `1001:1001`.

---

## 4. Passo 3: Connessione alla Shell via Terminale

Una volta che Federico ha inserito la tua chiave sul server, puoi connetterti immediatamente.

### Connessione con comando diretto
Apri il tuo terminale (PowerShell su Windows, Bash/Zsh su Linux/macOS) ed esegui:
```bash
ssh -p 2222 camillo@uncino.eu
```

*(Alla primissima connessione, SSH ti chiederà conferma dell'autenticità dell'host: digita `yes` e premi Invio).*

---

### Connessione rapida (Consigliato: configurare `~/.ssh/config`)

Per non dover digitare ogni volta porta, utente e indirizzo, puoi configurare un alias rapido.

1. Apri (o crea se non esiste) il file di configurazione SSH sul tuo PC:
   * Su **Windows** (PowerShell):
     ```powershell
     notepad ~/.ssh/config
     ```
   * Su **Linux / macOS**:
     ```bash
     nano ~/.ssh/config
     ```
2. Aggiungi questo blocco e salva il file:
   ```text
   Host cachy-lyra
       HostName uncino.eu
       Port 2222
       User camillo
       IdentityFile ~/.ssh/id_ed25519
   ```
3. Ora per connetterti ti basterà dare semplicemente:
   ```bash
   ssh cachy-lyra
   ```

---

## 5. Passo 4: Sviluppare con Visual Studio Code (Opzione Consigliata)

Se preferisci lavorare in un ambiente grafico con editor di codice, terminale integrato, git e debugger, puoi utilizzare **Visual Studio Code**:

1. Scarica e installa **Visual Studio Code** sul tuo PC (se non lo hai già).
2. Apri la sezione **Estensioni** (`Ctrl+Shift+X` su Windows/Linux o `Cmd+Shift+X` su Mac).
3. Cerca e installa l'estensione ufficiale **Remote - SSH** (di Microsoft).
4. Premi `F1` (oppure `Ctrl+Shift+P` / `Cmd+Shift+P`), digita e seleziona:
   ```text
   Remote-SSH: Connect to Host...
   ```
5. Inserisci:
   ```text
   ssh -p 2222 camillo@uncino.eu
   ```
   *(oppure seleziona `cachy-lyra` se hai configurato il file `~/.ssh/config` al passo precedente).*
6. Si aprirà una nuova finestra di VS Code connessa direttamente all'interno del container CachyOS.
7. Da VS Code fai `File` $\rightarrow$ `Open Folder...` e seleziona:
   ```text
   /home/camillo/Sviluppo/Progetti/semantic-geo
   ```
8. Ora hai l'intero progetto aperto nell'editor con tutte le funzionalità native (terminale, estensioni Python, Git diff, ecc.).

---

## 6. Informazioni sull'Ambiente di Lavoro

Una volta effettuato il login:

* **Cartella di lavoro:**
  ```bash
  cd ~/Sviluppo/Progetti/semantic-geo
  ```
  Tutto ciò che crei, modifichi o salvi qui dentro viene istantaneamente condiviso con Iris e sincronizzato con i permessi corretti (`iris:iris` sull'host).

* **Permessi di Amministrazione (sudo):**
  L'utente `camillo` è abilitato all'uso di `sudo` per installare librerie o strumenti di sviluppo aggiuntivi.
  * Per aggiornare i repository di CachyOS/Arch:
    ```bash
    sudo pacman -Sy
    ```
  * Per installare un pacchetto (ad esempio `htop`, `git`, `python`, ecc.):
    ```bash
    sudo pacman -S <nome_pacchetto>
    ```

* **GPU AMD Radeon:**
  Il container ha accesso diretto ai dispositivi hardware `/dev/dri` e `/dev/kfd` con i gruppi `video` e `render`, utile per l'accelerazione grafica o carichi di calcolo ROCm / PyTorch.

* **Ispezione Root:**
  La directory `/mnt` all'interno del container monta direttamente la root persistente del sistema per controlli e manutenzioni rapide.

---

## 7. Risoluzione dei Problemi Comuni

* **Errore `Permission denied (publickey)`:**
  * Assicurati che Federico abbia aggiunto la tua chiave pubblica in `authorized_keys`.
  * Se hai salvato la chiave in un percorso personalizzato, specifica la chiave con `-i`:
    ```bash
    ssh -i /percorso/della/tua/chiave -p 2222 camillo@uncino.eu
    ```
* **Errore `Host key verification failed`:**
  * Si verifica se in precedenza ti eri collegato a un altro server sulla stessa porta.
  * Su Linux/macOS/Windows PowerShell, puoi ripulire la vecchia chiave con:
    ```bash
    ssh-keygen -R "[uncino.eu]:2222"
    ```
