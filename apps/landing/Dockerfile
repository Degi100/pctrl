# Build stage
FROM node:20-alpine AS builder

WORKDIR /app

# Copy package files and install
COPY package.json ./
RUN npm install --legacy-peer-deps

# Copy source code (must include src/, public/, astro.config.mjs etc.)
COPY . .

# Verify the files are there
RUN ls -la && ls -la src/ || echo "src not found"

# Build argument for docs API URL (needed at build time for SSG)
ARG PUBLIC_DOCS_API_URL=http://localhost:3000
ENV PUBLIC_DOCS_API_URL=$PUBLIC_DOCS_API_URL

# Build with the installed astro from node_modules
RUN ./node_modules/.bin/astro build

# Production stage - serve static files with nginx
FROM nginx:alpine

# Copy built static files
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy nginx config for SPA routing
COPY --from=builder /app/nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget -q --spider http://localhost:80/ || exit 1

CMD ["nginx", "-g", "daemon off;"]
