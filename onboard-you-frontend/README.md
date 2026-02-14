# OnboardYou Frontend

A React/TypeScript/Vite monorepo powering the OnboardYou platform UI.

## Monorepo Structure

```
onboard-you-frontend/
├── packages/
│   ├── platform/    — Main SaaS platform app (dashboard, auth, org management)
│   └── config/      — Self-service configuration portal (pipeline builder, connectors)
├── package.json          — Workspace root scripts
├── pnpm-workspace.yaml   — pnpm workspace config
└── tsconfig.base.json    — Shared TypeScript configuration
```

### Packages

| Package | Description |
|---------|-------------|
| `@onboard-you/platform` | Primary platform application — authentication, dashboard, user & org management |
| `@onboard-you/config` | Configuration portal — pipeline builder, connector setup, field mapping |

## Getting Started

### Prerequisites

- **Node.js** ≥ 18
- **pnpm** ≥ 9

### Install Dependencies

```bash
pnpm install
```

### Development

```bash
# Run all packages in parallel
pnpm dev

# Run a single package
pnpm dev:platform
pnpm dev:config
```

### Build

```bash
# Build all packages
pnpm build

# Build a single package
pnpm build:platform
pnpm build:config
```

### Lint

```bash
pnpm lint
```

## Design System

The platform package owns the SCSS design system. Other packages consume it via:

```scss
@use '@onboard-you/platform/src/styles' as platform;

// Access variables
color: platform.$color-primary;

// Use mixins
@include platform.flex-center;
```

Variables, mixins, typography, animations, and reset styles are all defined in `packages/platform/src/styles/`.

## Backend

This frontend connects to the OnboardYou Rust API (see workspace root `lambdas/` and `infra/` directories). Authentication is handled via AWS Cognito through `@aws-amplify/auth`.
