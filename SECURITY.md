# Security Policy

## Scope

This policy covers the following repositories:

- `techtown-payroll-contracts` — Soroban smart contracts for DAO onboarding, treasury management, and confidential payroll
- `techtown-payroll-backend` — Rust/Axum relayer, proof generation service, REST API, and background workers
- `techtown-payroll-web` — Next.js frontend for DAO and employee dashboards

**In scope:**
- Stellar/smart-contract wallet addresses, contract IDs, and on-chain funds held by deployed contracts
- Backend REST endpoints, database credentials, and Redis job queues
- Frontend secrets, injection risks, and wallet-connect flows
- CI/CD pipelines, deploy scripts, and infrastructure configuration

**Out of scope:**
- Third-party dependencies unless directly exploitable in our context
- Test networks and local development environments running stale data
- Issues that require physical access to a maintainer's machine
- Social engineering against individual contributors outside official channels

## Reporting a Vulnerability

Please do **not** open a public GitHub issue for security vulnerabilities.

### Preferred: Private GitHub Advisory

Use GitHub's private vulnerability reporting feature on the relevant repository:

1. Navigate to the repository's **Security** tab
2. Click **Report a vulnerability**
3. Fill out the advisory form with:
   - Affected component (`techtown-payroll-contracts`, `techtown-payroll-backend`, or `techtown-payroll-web`)
   - Vulnerability type (e.g., reentrancy, access control bypass, credential leak)
   - Step-by-step reproduction
   - Suggested remediation
   - Your preferred handle for credit

### Alternative: Email

Send a PGP-encrypted email to `security@techtown.example`. Attach the same details listed above.

## Expected Response Time

| Phase | Timeframe |
|---|---|
| Initial acknowledgment | 48 hours |
| Triage and severity assessment | 5 business days |
| Patch development | 30 days for critical/high, 90 days for medium/low |
| Coordinated disclosure | After patch ships or 90 days, whichever comes first |

We will keep you informed of progress even if the fix is not ready.

## Production Readiness Notice

Parts of this codebase are **not yet production-ready**.

In particular, `zk_verifier::verify_payroll_proof` is a **structural placeholder** that only validates byte lengths and public-input consistency. A real Groth16/PLONK pairing check has not been implemented. We welcome research input on integrating `arkworks-rs` or Soroban host crypto, but the current stub must be treated as **non-auditable ZK code**.

When reporting issues, please distinguish between:

1. **Smart contract / backend / frontend vulnerabilities** — we treat these with full severity.
2. **ZK circuit / proof verifier research gaps** — these are open engineering tasks; reports should include mathematical or implementation specifics rather than generic "ZK not secure" flags.

Deployments on mainnet or testnet should use the contracts at your own risk until the pairing check and a formal audit are complete.
