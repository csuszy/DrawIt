# DrawIt – Gyors és Hordozható Képernyőrajzoló

A **DrawIt** egy villámgyors, hordozható képernyőannotáló, magyarázó és képmetsző alkalmazás Windows 10 és 11 rendszerekre. 

Teljesen natív **Rust** nyelven készült, hardveres Direct2D gyorsítással:
* **Nem igényel .NET keretrendszert**, semmilyen külső függőséget vagy futtatókörnyezetet.
* **Nem kér rendszergazdai jogosultságot** (`asInvoker`), pendrive-ról vagy bármilyen mappából azonnal futtatható.
* **Azonnali indulás**: elhanyagolható memóriahasználat (~15-25 MB) és mindössze ~3-4 MB-os fájlméret a korábbi 76 MB helyett.

---

## Letöltés és futtatás

* [**`DrawIt.exe`**](file:///C:/Users/bcsas/Desktop/DrawIt/DrawIt.exe) — Egyetlen önálló, hordozható futtatható fájl. Bárhová átmásolható és közvetlenül indítható.
* [**`Standalone_Files\DrawIt.exe`**](file:///C:/Users/bcsas/Desktop/DrawIt/Standalone_Files/DrawIt.exe) — Külön mappába rendezett változat.
* [**`Standalone_Files.zip`**](file:///C:/Users/bcsas/Desktop/DrawIt/Standalone_Files.zip) — Tömörített archívum egyszerű átküldéshez és megosztáshoz.

Indítás után a program a tálca értesítési területén fut a háttérben. A rajzolást a **`Ctrl + 2`** billentyűkombinációval vagy a tálcaikonra kattintva indíthatja el.

---

## Funkciók és kezelés

### 1. Lebegő vezérlősáv (Dokk)
* **Mozgatás**: A bal szélső 6 pontos fogantyúnál (`⋮⋮`) megfogva a dokk bárhová áthúzható. A menük melletti véletlen kattintások nem mozdítják el a vezérlősávot.
* **Logó menü**: A DrawIt logóra kattintva érhető el a gyorsbillentyű-súgó (`F1`), a számláló nullázása, a névjegy és a kilépés.
* **Aktív eszköz felirat**: Amikor kiválaszt egy rajzeszközt, a gomb kinyílik és megjeleníti a nevét (pl. `— Vonal`, `T Szöveg`, `▭ Téglalap`, `⌒ Ív`, `💧 Homály`). Az inaktív eszközök kis méretű ikonok maradnak, hogy ne takarják a munkaterületet.

### 2. Rajzeszközök
* **Rajz (`✎`)**: Szabadkézi toll simított vonalakkal.
* **Vonal (`—`)**: Egyenes vonalak húzása.
* **Téglalap (`▭`)** és **Kör (`⚪`)**: Területek és képernyőelemek keretezése, kiemelése.
* **Nyíl (`➔`)**: Folyamatok és figyelemfelhívó nyilak húzása.
* **Számozás (`①`)**: Sorszámozott körjelvény (1, 2, 3...). A rajzolófelület megnyitásakor vagy törléskor a számláló mindig automatikusan elölről (1-től) indul.
* **Ív (`⌒`)**: Görbék rajzolása: húzzon egy egyenes vonalat, engedje fel az egeret, majd a kurzor mozgatásával hajlítsa meg a kívánt ívre, és kattintson a rögzítéshez.
* **Szöveg (`T`)**: Kattintson a képernyő bármely pontjára, és gépeljen közvetlenül. Az Enter billentyűvel lezárhatja a szöveget.
* **Homályosítás (`💧`)**: Húzzon egy dobozt a képernyő bármely részére a bizalmas adatok (jelszavak, személyes adatok) azonnali, diszkrét elfedéséhez.

### 3. Színek és ecsetméret
* **Színválasztó (`● 4px ▾`)**: 9 gyorsszín, valamint a **`🎨 Egyedi szín...`** gomb, amellyel tetszőleges szín választható ki a Windows színpalettáján. A szín kiválasztása után a menü nyitva marad, így azonnal állítható a vastagság vagy átlátszatlanság is.
* **Ctrl + Egérgörgő (vagy görgő)**: A vonalvastagság (és a szöveg betűméret) azonnali fokozatmentes növelése vagy csökkentése (1 px – 32 px) a képernyő bármely pontján.
* **Átlátszatlanság**: 25%, 50%, 75% és 100% beállítások kiemelőfilc hatáshoz.
* **Vonalvastagság**: 2px, 4px, 8px, 12px gyorsgombok, valamint `−` és `+` finomállító gombok.

### 4. Kijelző módok (`Kijelző ▾`)
* **Átlátszó**: Rajzolás az élő asztalra és megnyitott ablakokra.
* **Homályosított**: Elegáns, homályosított háttér, amely kiemeli a feliratokat és rajzokat.
* **Fehér tábla**: Tiszta fehér digitális jegyzettábla.
* **Fekete tábla**: Sötét digitális jegyzettábla.

### 5. Többmonitoros intelligencia és Értesítések
* **Fókuszált monitor központosítás**: A lebegő dokk, a képmetsző segédszövegei, a HUD értesítések, valamint az összes felugró ablak (Súgó, Névjegy, Színválasztó, Fájlmentés) mindig automatikusan azon a monitoron és annak közepén jelennek meg, amelyiken éppen az egérrel dolgozik — így két monitor esetén sem csúszik át a kép a monitorok közötti kávára.
* **Tiszta munkalap újranyitáskor**: A felület bezárása (vagy Esc megnyomása) után az újbóli megnyitáskor a korábbi rajzok automatikusan törlődnek, a számozás pedig mindig 1-től indul újra.
* **Lebegő HUD értesítések**: Minden kulcsfontosságú műveletről (másolás vágólapra, mentés, méretváltoztatás, visszavonás) elegáns visszajelzés jelenik meg az aktív képernyő alsó közepén.

### 6. Kivágás és Többmonitoros mentés
* **`✂️ Kivágás`**: Képernyőrészlet kijelölése. A kijelölt terület alatt megjelenő gyorsgombsorral a kép azonnal a vágólapra másolható, vagy elmenthető PNG és PDF formátumban.
* **`💾 Kijelző mentése ▾`**: Többmonitoros támogatás:
  * A lenyíló listában külön sorban szerepel minden csatlakoztatott kijelző (pl. *1. Kijelző [Főképernyő] (1920×1080)*, *2. Kijelző (1920×1080)*), valamint a *Teljes asztal*.
  * Különálló gombokkal (`PNG`, `PDF`, `Másolás`) egyetlen kattintással elmenthető vagy vágólapra tehető a kiválasztott monitor képe.

---

## Gyorsbillentyűk

| Billentyű | Művelet |
| :--- | :--- |
| **Ctrl + 2** *(vagy Ctrl + Alt + D)* | Rajzolófelület be- és kikapcsolása bárhonnan a rendszerből |
| **Ctrl + Egérgörgő** *(vagy görgő)* | Vonalvastagság / Szövegméret azonnali növelése és csökkentése |
| **Esc** *(vagy Jobb egérgomb)* | Kilépés a rajzolásból / Menük bezárása / Kijelölés megszüntetése |
| **Ctrl + Z** | Visszavonás |
| **Ctrl + Y** | Újra |
| **Delete** | Rajzvászon azonnali törlése és számláló újraindítása 1-ről |
| **Ctrl + C** | Kijelölt terület (vagy aktuális képernyő) másolása vágólapra |
| **Ctrl + S** | Képernyőkép mentése PNG fájlba |
| **F1** | Súgó és billentyűk ablak megnyitása a fókusz monitoron |
