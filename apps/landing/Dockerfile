# Build stage
FROM node:20-alpine AS builder

WORKDIR /app

# Copy package files
COPY package.json package-lock.json* ./

# Install dependencies
RUN npm ci || npm install

# Copy source code
COPY . .

# Build argument for docs API URL (needed at build time for SSG)
ARG PUBLIC_DOCS_API_URL=http://localhost:3000

# Set environment variable for build
ENV PUBLIC_DOCS_API_URL=$PUBLIC_DOCS_API_URL

# Build the static site
RUN npm run build

# Production stage - serve static files with nginx
FROM nginx:alpine

# Copy built static files
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy nginx config for SPA routing (from builder to avoid cache issues)
COPY --from=builder /app/nginx.conf /etc/nginx/conf.d/default.conf

# Expose port
EXPOSE 80

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget -q --spider http://localhost:80/ || exit 1

CMD ["nginx", "-g", "daemon off;"]
