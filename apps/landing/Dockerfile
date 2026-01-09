# Build stage
FROM node:20-alpine AS builder

WORKDIR /app

# Copy landing page files from apps/landing (Base Directory: /)
COPY apps/landing/package.json ./
COPY apps/landing/src ./src
COPY apps/landing/astro.config.mjs ./
COPY apps/landing/nginx.conf ./

# Copy ROADMAP.md from root for live stats
COPY ROADMAP.md ./ROADMAP.md

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

CMD ["nginx", "-g", "daemon off;"]
