# Landing Page Deployment

## Coolify Setup

### 1. Deploy docs-api zuerst

Die Landing Page braucht die docs-api zur Build-Zeit (SSG).

**docs-api Service:**
- Source: `apps/docs-api`
- Build Pack: Dockerfile
- Port: 3000

**Environment Variables (docs-api):**
```
MONGODB_URI=mongodb://user:pass@host:27017/pctrl?authSource=pctrl
PORT=3000
API_KEY=<sicherer-random-key>
```

### 2. Deploy landing

**landing Service:**
- Source: `apps/landing`
- Build Pack: Dockerfile
- Port: 80

**Build Arguments:**
```
PUBLIC_DOCS_API_URL=https://docs-api.pctrl.dev
```

**Wichtig:** `PUBLIC_DOCS_API_URL` muss als Build-Argument gesetzt werden, da Astro SSG die Docs beim Build fetcht!

### 3. Domain Setup

| Service | Domain |
|---------|--------|
| docs-api | `docs-api.pctrl.dev` (intern oder public) |
| landing | `pctrl.dev`, `www.pctrl.dev` |

### Deployment-Reihenfolge

1. docs-api deployen und warten bis healthy
2. `bun run reseed` ausführen (Docs in DB laden)
3. landing deployen (mit korrekter `PUBLIC_DOCS_API_URL`)

### Rebuild Landing

Bei Docs-Änderungen muss die Landing Page neu gebaut werden:
1. Docs in DB aktualisieren (`bun run seed`)
2. Landing Page in Coolify re-deployen

### Lokales Testen

```bash
# Docker bauen
cd apps/docs-api
docker build -t pctrl-docs-api .

cd apps/landing
docker build --build-arg PUBLIC_DOCS_API_URL=http://host.docker.internal:3000 -t pctrl-landing .

# Starten
docker run -d -p 3000:3000 --env-file .env pctrl-docs-api
docker run -d -p 8080:80 pctrl-landing
```
