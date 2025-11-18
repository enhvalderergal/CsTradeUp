# CS Trade Up – README

Dette projekt er en CS:GO trade‑up applikation skrevet i Rust. Formålet er at håndtere skins, trade‑ups og inventar via en simpel SQL lite database, samtidig med at UI‑delen fungerer som en ren frontend adskilt fra backend‑logikken.

## Projektstruktur

Projektet er opdelt i følgende hoveddele:

- **Backend / Core Logic**
  
  - Databasehåndtering (SQLite via `cs_trade_up.db`)
    
  - Koden til databasen kan findes i`db.rs`hvor hver funktion der benyttes er lavet
    
  - Databasen includere også login og registereing af user Databasens form er beskrevet i `modules.rs`
    
  - **Frontend / UI (Egui)**
    
- - UI‑skærme og visuelle komponenter ligger i deres egne moduler
    
  - Alle UI modulerne (screens) ligger i SRC/UI/(buy,sell,....)
    

Denne adskillelse gør koden lettere at teste, lettere at udbygge og mere vedligeholdelsesvenlig.

---

# Seedning af Skins-kataloget

Her er en detaljeret gennemgang af hvordan seedning af skins fungerer samt hvordan du kører seederen manuelt. (Altså hvordan en dev skal ligge nye skins til)

## Hvor seed‑data ligger

- **Udviklerens seed‑fil**:
  
  - `data/skins.json` – Indeholder et JSON array af skin‑objekter.
    
  - Hvert objekt bør minimum have et `name`‑felt.
    
  - Valgfri felter:
    
    - `rarity`
      
    - `price`
      
    - `collection`
      
    - `weapon_type`
      
    - `image_base64`
      
- **Database fil**:
  
  - `cs_trade_up.db` – oprettes automatisk i projektets rodmappe hvis den ikke findes.

## Adfærd ved opstart

Når applikationen startes, kalder `CsApp::default()` følgende funktion:

```
db::init_db(&db_path)
```

Denne funktion gør:

1. Opretter tabellerne `users`, `inventory` og `skins` hvis de ikke allerede findes.
  
2. Hvis `data/skins.json` findes:
  
  - Forsøger at parse filen
    
  - Kalder `add_skin` for hvert skin
    
  - Ignorerer fejl på individuelle skins, så startup er robust
    

### Idempotens

`add_skin` bruger SQL‑kommandoen `INSERT OR IGNORE`, hvilket betyder:

- Samme skin‑navn bliver **ikke** tilføjet flere gange
  
- Seedning kan køres flere gange uden at skabe duplikater
  

---

# Kør seederen manuelt

Hvis du hellere vil køre seed‑processen selv (i stedet for auto‑seed), medfølger et lille CLI‑tool.

Fra projektets rodmappe, kør i PowerShell:

```powershell
cargo run --bin seed_skins
```

Dette vil:

- Læse `data/skins.json`
  
- Forsøge at indsætte alle skins i `skins`‑tabellen
  

---

# Eksempel på minimal JSON‑entry

```json
{
  "name": "AK-47 | Redline",
  "rarity": "Classified",
  "price": 12.5,
  "collection": "The Hydra Collection",
  "weapon_type": "AK-47",
  "image_base64": null
}
```

Her kan der altså ligges nye skins til databasen uden manuelt at have fat i sql lite

Billederne transformeres til BASE64 inden det ligges til databasen

---