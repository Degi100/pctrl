# Docs API

Backend-API für die pctrl Dokumentation. Speichert Docs in MongoDB.

## Setup

```bash
cd apps/docs-api
cp .env.example .env
# .env anpassen (API_KEY setzen!)
bun install
bun run dev
```

## Dokumentation verwalten

### Scripts

| Script | Beschreibung |
|--------|--------------|
| `bun run seed` | Docs erstellen/aktualisieren (Upsert) |
| `bun run clean` | Alle Docs aus DB löschen |
| `bun run reseed` | Clean + Seed (komplett neu) |

### Alle Docs aktualisieren (Upsert)

```bash
bun run seed
```

Erstellt neue Docs oder aktualisiert existierende. Die Datei `seed-docs.ts` ist die **Single Source of Truth**.

### Datenbank bereinigen (bei Duplikaten)

```bash
bun run reseed
```

Löscht alle Docs und seedet komplett neu.

### Ein Doc bearbeiten

1. `seed-docs.ts` öffnen
2. Den entsprechenden Eintrag im `docs[]` Array bearbeiten
3. `bun run seed` ausführen

### Neues Doc hinzufügen

1. Neuen Eintrag in `docs[]` Array hinzufügen:

```typescript
{
  slug: 'mein-neues-doc',
  title: 'Mein Neues Doc',
  category: 'guides',  // getting-started, commands, guides
  order: 10,
  content: `# Mein Neues Doc

Markdown content hier...
`
}
```

2. `bun run seed` ausführen

### Ein Doc löschen

```bash
curl -X DELETE http://localhost:3000/docs/SLUG \
  -H "Authorization: Bearer $API_KEY"
```

Oder aus `seed-docs.ts` entfernen (bleibt aber in DB bis manuell gelöscht).

## API Endpoints

| Method | Endpoint | Auth | Beschreibung |
|--------|----------|------|--------------|
| GET | `/docs` | - | Alle Docs (Titel only) |
| GET | `/docs/:slug` | - | Ein Doc |
| GET | `/docs/categories` | - | Alle Kategorien |
| GET | `/docs/category/:cat` | - | Docs einer Kategorie |
| POST | `/docs` | Bearer | Doc erstellen |
| PUT | `/docs/:slug` | Bearer | Doc aktualisieren |
| DELETE | `/docs/:slug` | Bearer | Doc löschen |

## Environment Variables

| Variable | Beschreibung |
|----------|--------------|
| `MONGODB_URI` | MongoDB Connection String |
| `PORT` | Server Port (default: 3000) |
| `API_KEY` | API Key für Write-Operationen |
| `API_URL` | API URL für Seed-Script |

## Troubleshooting

### Duplikate in der Dokumentation

**Problem:** Docs werden doppelt angezeigt (z.B. "Installation" erscheint 2x).

**Ursache:** Das Seed-Script wurde mehrfach ausgeführt bevor Upsert-Logik implementiert war.

**Lösung:**

```bash
bun run reseed
```

Das löscht alle Docs und seedet komplett neu.
