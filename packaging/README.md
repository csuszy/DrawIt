# DrawIt – Microsoft Store & Telepítő Csomagolás / Packaging Guide

Ez a mappa tartalmazza a Microsoft Store-hoz és a Windows telepítőkhöz szükséges konfigurációkat és eszközöket.

---

## 📦 Kétféle Microsoft Store terjesztési mód (Two Distribution Methods)

A Microsoft Partner Center kétféle módon teszi lehetővé a Win32 alkalmazások publikálását a **Microsoft Store**-ban:

### 1. Módszer: MSIX Csomag közvetlen feltöltése (Ajánlott / Recommended)
Az MSIX a modern Windows csomagformátum. Előnyei:
* Egykattintásos telepítés közvetlenül a Microsoft Store áruházból.
* **Automatikus Microsoft aláírás**: Nem szükséges drága egyedi kódaláíró tanúsítványt vásárolni, a Store közzétételkor a Microsoft automatikusan aláírja a saját megbízható tanúsítványával.
* Nincs SmartScreen figyelmeztetés a végfelhasználóknál.
* Tiszta háttérbeli frissítések és maradéktalan eltávolítás.

#### Lépések:
1. Nyisd meg a [Microsoft Partner Center](https://partner.microsoft.com/dashboard) fiókodat.
2. Foglald le az alkalmazás nevét (pl. `DrawIt`).
3. A **Product management > Package identity** (Termékkezelés > Csomagidentitás) menüpontban másold ki az azonosítókat:
   - **Package/Identity/Name** (Csomagnév, pl. `YourName.DrawIt`)
   - **Package/Identity/Publisher** (Kiadó azonosító, pl. `CN=01234567-89AB-...`)
   - **Package/Properties/PublisherDisplayName** (Kiadó megjelenített neve)
4. Írd be ezeket a [packaging/msix/AppxManifest.xml](file:///c:/Users/bcsas/Desktop/DrawIt/packaging/msix/AppxManifest.xml) fájlba.
5. Futtasd a csomagoló szkriptet:
   ```powershell
   powershell -ExecutionPolicy Bypass -File packaging\build-packages.ps1
   ```
6. A létrejött `DrawIt.msix` fájlt töltsd fel a Partner Center "Packages" menüpontjában.

---

### 2. Módszer: Win32 Telepítő URL (Inno Setup)
A Microsoft Store lehetővé teszi hagyományos `.exe` telepítő beküldését is közvetlen letöltési hivatkozással (pl. a GitHub Release-ből):
* **Telepítő fájl**: `DrawIt-Setup.exe`
* **Csendes telepítés kapcsolói (Silent install)**: `/VERYSILENT /SUPPRESSMSGBOXES /NORESTART`
* **Csendes eltávolítás kapcsolói (Silent uninstall)**: `/VERYSILENT /SUPPRESSMSGBOXES /NORESTART`
* **Telepítési útvonal**: `%LOCALAPPDATA%\Programs\DrawIt`
* **Jogosultság**: Nem igényel rendszergazdai jogot (`PrivilegesRequired=lowest`), bármely felhasználói fiók alatt azonnal lefut.

---

## 🛠 Helyi tesztelés (Sideloading)

Ha helyben szeretnéd tesztelni az elkészült csomagokat:
1. **MSIX telepítés tesztelése**:
   A szkript generál egy fejlesztői tanúsítványt (`target\DrawIt_Dev.pfx`), amellyel aláírja a `DrawIt.msix` csomagot.
   A sikeres helyi telepítéshez nyisd meg a `DrawIt.msix` fájl Tulajdonságok > Digitális aláírások menüjét, és telepítsd a tanúsítványt a **Helyi gép > Megbízható személyek (Local Machine > Trusted People)** tárolóba.
2. **Inno Setup telepítő**:
   Közvetlenül indítható a `DrawIt-Setup.exe` futtatásával (angol és magyar nyelvi támogatással, asztali és indítópulti ikon opciókkal).
