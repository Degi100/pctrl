# Build stage
FROM node:20-alpine AS builder

WORKDIR /app

# Copy only files that exist in apps/landing
COPY package.json ./
COPY src ./src
COPY astro.config.mjs ./
COPY nginx.conf ./

# Install dependencies
RUN npm install

# Docs API URL for build time (SSG)
ENV PUBLIC_DOCS_API_URL=https://pctrl-api.degidev.de

# Build
RUN ./node_modules/.bin/astro build

# Production stage
FROM nginx:alpine

COPY --from=builder /app/dist /usr/share/nginx/html
COPY --from=builder /app/nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

# Healthcheck disabled - let Coolify/Traefik handle routing
# HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
#   CMD wget -q --spider http://localhost:80/ || exit 1

CMD ["nginx", "-g", "daemon off;"]
