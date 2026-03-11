# Release Process

Version: 1.0.0 | Date: 10 novembre 2025

## Release Types

- **Major** (x.0.0): Breaking changes, architecture changes
- **Minor** (0.x.0): New features, backward compatible
- **Patch** (0.0.x): Bug fixes only

## Pre-Release Checklist

### 1. Code Quality
- [ ] All tests pass (`make test`)
- [ ] Linting passes (`make lint`)
- [ ] No security vulnerabilities (`cargo audit`)
- [ ] Coverage ≥ 80% for new code

### 2. Documentation
- [ ] CHANGELOG.md updated with all changes
- [ ] API documentation updated if endpoints changed
- [ ] User guides updated if UI/UX changed
- [ ] Migration guide if breaking changes

### 3. Testing
- [ ] Unit tests pass (`cargo test --lib`)
- [ ] Integration tests pass (`cargo test --test integration`)
- [ ] BDD tests pass (`cargo test --test bdd`)
- [ ] E2E tests pass (`cargo test --test e2e`)
- [ ] Load tests executed (`cd load-tests && k6 run`)
- [ ] Manual QA in staging environment

### 4. Database
- [ ] Migrations tested (up + down)
- [ ] Backup taken before migration
- [ ] Migration rollback plan documented

### 5. Dependencies
- [ ] Dependencies updated (`cargo update`)
- [ ] Security audit passed (`cargo audit`)
- [ ] License compatibility verified

## Release Steps

### 1. Prepare Release Branch

```bash
git checkout -b release/v1.2.0
```

### 2. Update Version

```bash
# backend/Cargo.toml
version = "1.2.0"

# frontend/package.json
"version": "1.2.0"
```

### 3. Update CHANGELOG.md

```markdown
## [1.2.0] - 2025-11-10

### Added
- New feature X (#123)
- New endpoint `/api/v1/feature-x` (#124)

### Changed
- Improved performance of Y by 30% (#125)

### Fixed
- Bug in Z causing incorrect calculation (#126)

### Security
- Updated dependency A to patch CVE-2025-XXXX
```

### 4. Build & Test

```bash
# Backend
cd backend
cargo build --release
cargo test --all

# Frontend
cd frontend
npm run build
npm test
```

### 5. Create Git Tag

```bash
git add .
git commit -m "chore: prepare release v1.2.0"
git tag -a v1.2.0 -m "Release v1.2.0"
git push origin release/v1.2.0 --tags
```

### 6. Create GitHub Release

```bash
gh release create v1.2.0 \
  --title "v1.2.0 - Feature X Release" \
  --notes-file CHANGELOG.md \
  --target main
```

### 7. Deploy to Staging

```bash
cd infrastructure/ansible
ansible-playbook -i inventory-staging.ini deploy.yml \
  -e "version=1.2.0"
```

### 8. Smoke Test Staging

```bash
curl https://staging.koprogo.com/health
curl https://staging.koprogo.com/api/v1/buildings
# Manual testing: Login, create building, etc.
```

### 9. Deploy to Production

```bash
# Backup first!
ansible-playbook -i inventory-prod.ini backup.yml

# Deploy
ansible-playbook -i inventory-prod.ini deploy.yml \
  -e "version=1.2.0"

# Run migrations
ansible-playbook -i inventory-prod.ini migrate.yml
```

### 10. Verify Production

```bash
curl https://api.koprogo.com/health
# Check Grafana dashboards
# Monitor logs for errors
```

### 11. Announce Release

- [ ] Update status page (https://status.koprogo.com)
- [ ] Send email to users if major changes
- [ ] Post in #announcements (Slack)
- [ ] Update documentation site (https://docs.koprogo.com)

## Rollback Procedure

If issues detected post-release:

```bash
# 1. Deploy previous version
ansible-playbook -i inventory-prod.ini deploy.yml \
  -e "version=1.1.0"

# 2. Rollback database migrations
cd backend
sqlx migrate revert

# 3. Notify users
# Update status page + send incident email

# 4. Post-mortem
# Document what went wrong and prevent recurrence
```

## Hotfix Process

For critical bugs in production:

1. **Branch from main** (not develop):
   ```bash
   git checkout main
   git checkout -b hotfix/v1.2.1
   ```

2. **Fix bug + tests**:
   ```bash
   # Fix code
   cargo test
   ```

3. **Fast-track deployment**:
   - Skip staging (only for critical fixes)
   - Deploy directly to production
   - Monitor closely

4. **Merge back**:
   ```bash
   git checkout main
   git merge hotfix/v1.2.1
   git push origin main --tags
   ```

## Versioning Strategy

- **Backend** (`backend/Cargo.toml`): Source of truth
- **Frontend** (`frontend/package.json`): Must match backend
- **Docker images**: Tagged with same version (`koprogo/api:1.2.0`)
- **Git tags**: `v1.2.0` (with 'v' prefix)

## Release Calendar

- **Patch releases**: As needed (bug fixes)
- **Minor releases**: Monthly (feature releases)
- **Major releases**: Quarterly (architecture changes)

## Communication Templates

### Release Email (Major Features)

```
Subject: KoproGo v1.2.0 Released - New Feature X

Bonjour,

Nous sommes heureux d'annoncer la sortie de KoproGo v1.2.0 avec :

✨ Nouvelle fonctionnalité X
⚡ Amélioration performance Y (+30%)
🐛 Correction bug Z

Documentation : https://docs.koprogo.com/changelog/v1.2.0

Cordialement,
L'équipe KoproGo
```

### Status Page Update

```
✅ KoproGo v1.2.0 deployed successfully
Posted: 2025-11-10 10:00 UTC

All systems operational. New features available.
See release notes: https://github.com/gilmry/koprogo/releases/tag/v1.2.0
```

---

**Version**: 1.0.0 | **Last Update**: 10 novembre 2025
