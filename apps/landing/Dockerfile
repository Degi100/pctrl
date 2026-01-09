# Build stage - v2
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

EXPOSE 3000

# No Dockerfile healthcheck - let Coolify handle it
HEALTHCHECK NONE

CMD ["nginx", "-g", "daemon off;"]
