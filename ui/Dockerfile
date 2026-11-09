# Build the UI with Vite, then serve the static files with nginx.
FROM node:22-bookworm-slim AS builder
WORKDIR /app

# Dependencies first, so this layer is cached until package*.json changes.
COPY package.json package-lock.json ./
RUN --mount=type=cache,target=/root/.npm \
    npm ci

COPY . .
# Type-checks (tsc -b), then bundles into dist/. Uses the committed src/api/schema.d.ts.
RUN npm run build

FROM nginx:1.27-alpine
# The nginx image runs envsubst on /etc/nginx/templates/*.template at startup,
# so FLARE_API_URL can be changed per deployment without rebuilding.
ENV FLARE_API_URL=http://server:8080
COPY nginx.conf.template /etc/nginx/templates/default.conf.template
COPY --from=builder /app/dist /usr/share/nginx/html

EXPOSE 80
