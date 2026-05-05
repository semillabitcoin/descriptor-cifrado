---
phase: 01-crypto-core-http-api
plan: 02
type: execute
wave: 1
depends_on: []
files_modified:
  - .github/workflows/ci.yml
autonomous: true
requirements: [CI-01]
must_haves:
  truths:
    - "GitHub Actions workflow tiene 5 jobs: fmt, clippy, test, audit, deny"
    - "Workflow se gatilla en pull_request y push a main"
    - "Cada job tiene timeout-minutes ≤ 15"
    - "Job audit usa rustsec/audit-check@v2"
    - "Job deny usa EmbarkStudios/cargo-deny-action@v2"
  artifacts:
    - path: ".github/workflows/ci.yml"
      provides: "CI pipeline completo"
      contains: "EmbarkStudios/cargo-deny-action@v2"
  key_links:
    - from: ".github/workflows/ci.yml job clippy"
      to: "Cargo.toml workspace.lints.clippy"
      via: "cargo clippy --all-targets --all-features --workspace -- -D warnings"
      pattern: "-D warnings"
    - from: ".github/workflows/ci.yml job test"
      to: "crates/*/tests/*"
      via: "cargo test --all-features --workspace --locked"
      pattern: "cargo test"
---

<objective>
Crear el pipeline de CI en GitHub Actions con 5 jobs paralelos: `fmt`, `clippy`, `test`, `audit`, `deny`. Trigger en `pull_request` y `push` a `main`. Runner `ubuntu-latest`. Timeout máximo 15 min por job. Cumple CI-01 (audit + deny en CI).

Purpose: CI debe fallar inmediatamente si alguien añade `unwrap()` (clippy + lints), introduce vulnerabilidad (audit), añade dep con `openssl-sys`/`native-tls` (deny), o un test falla (test).
Output: `.github/workflows/ci.yml` válido; primera ejecución en PR posterior pasa todos los jobs (asume Plan 01 ya commiteado).
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/01-crypto-core-http-api/01-CONTEXT.md
@.planning/phases/01-crypto-core-http-api/01-RESEARCH.md
@CLAUDE.md
</context>

<tasks>

<task type="auto">
  <name>Task 1: Crear .github/workflows/ci.yml con 5 jobs paralelos</name>
  <files>.github/workflows/ci.yml</files>
  <read_first>
    - .planning/phases/01-crypto-core-http-api/01-RESEARCH.md (sección "GitHub Actions CI (D-28, D-29)" — copia literal del YAML)
    - .planning/phases/01-crypto-core-http-api/01-CONTEXT.md (D-28, D-29)
  </read_first>
  <action>
    Crear directorio `.github/workflows/` si no existe, después escribir `.github/workflows/ci.yml` con el contenido EXACTO del bloque "GitHub Actions CI (D-28, D-29)" de `01-RESEARCH.md`:

    ```yaml
    name: CI

    on:
      pull_request:
      push:
        branches: [main]

    jobs:
      fmt:
        runs-on: ubuntu-latest
        timeout-minutes: 5
        steps:
          - uses: actions/checkout@v4
          - uses: dtolnay/rust-toolchain@stable
            with:
              components: rustfmt
          - run: cargo fmt --all -- --check

      clippy:
        runs-on: ubuntu-latest
        timeout-minutes: 15
        steps:
          - uses: actions/checkout@v4
          - uses: dtolnay/rust-toolchain@stable
            with:
              components: clippy
          - uses: Swatinem/rust-cache@v2
          - run: cargo clippy --all-targets --all-features --workspace -- -D warnings

      test:
        runs-on: ubuntu-latest
        timeout-minutes: 15
        steps:
          - uses: actions/checkout@v4
          - uses: dtolnay/rust-toolchain@stable
          - uses: Swatinem/rust-cache@v2
          - run: cargo test --all-features --workspace --locked

      audit:
        runs-on: ubuntu-latest
        timeout-minutes: 10
        steps:
          - uses: actions/checkout@v4
          - uses: rustsec/audit-check@v2
            with:
              token: ${{ secrets.GITHUB_TOKEN }}

      deny:
        runs-on: ubuntu-latest
        timeout-minutes: 10
        steps:
          - uses: actions/checkout@v4
          - uses: EmbarkStudios/cargo-deny-action@v2
            with:
              command: check
    ```

    NOTA al executor: NO modificar versiones de actions; son las canonicales actuales. NO añadir matrix de Rust versions (stable solo). NO añadir job de release/publish (eso es Phase 3).
  </action>
  <verify>
    <automated>test -f .github/workflows/ci.yml && grep -c 'runs-on: ubuntu-latest' .github/workflows/ci.yml</automated>
  </verify>
  <acceptance_criteria>
    - `.github/workflows/ci.yml` existe
    - `grep -c '^  fmt:\|^  clippy:\|^  test:\|^  audit:\|^  deny:' .github/workflows/ci.yml` == 5
    - `grep -c 'runs-on: ubuntu-latest' .github/workflows/ci.yml` == 5
    - `grep -c 'timeout-minutes:' .github/workflows/ci.yml` == 5
    - `grep -q 'EmbarkStudios/cargo-deny-action@v2' .github/workflows/ci.yml`
    - `grep -q 'rustsec/audit-check@v2' .github/workflows/ci.yml`
    - `grep -q '\-D warnings' .github/workflows/ci.yml`
    - `grep -q 'cargo test --all-features --workspace --locked' .github/workflows/ci.yml`
    - YAML es válido: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"` exits 0
    - NO contiene `0.0.0.0` ni referencia a publicación a GHCR (Phase 3)
  </acceptance_criteria>
  <done>Workflow listo; primera ejecución en CI tras commit pasará `fmt`, `clippy`, `test` (con tests vacíos compila), `audit`, `deny`.</done>
</task>

</tasks>

<verification>
- `.github/workflows/ci.yml` válido (parsea con yaml.safe_load)
- 5 jobs declarados, todos con timeout
- Trigger en `pull_request` + `push` a `main`
</verification>

<success_criteria>
- Workflow file commiteable
- Cubre CI-01 (`cargo audit` + `cargo deny` corren y fallan en violación)
- Lints `unwrap_used`/`expect_used` se enforzan vía clippy job
</success_criteria>

<output>
Tras completar, crear `.planning/phases/01-crypto-core-http-api/01-02-github-actions-ci-SUMMARY.md` documentando:
- Path del archivo creado
- Lista de los 5 jobs y su timeout
- Acciones externas usadas y sus versiones
</output>
