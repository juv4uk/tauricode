# Аудит-нотатка: TAURICODE-WSM-LANGUAGE-MAP позначена відкритою, хоча зроблена

**Статус:** ЗВІТ (виявлено, не виправлено).
**Джерело:** agent-team аудит суперечностей/застарілих пунктів, повний
звіт — `/home/agents/ecosystem/docs/agent-team-contradiction-audit-2026-08-27.md`
(§5.1).

## Знахідка

`tasks.my:22-27`, задача `TAURICODE-WSM-LANGUAGE-MAP`, досі
`(done . ())`.

Реально закрита двома комітами того ж дня:

- `2afc6e7ee5` (06:50) — мапінг мови в `packages/opencode/src/lsp/language.ts:121-123`
- `e50822c2a7` (14:35) — "feat(lsp): register WsmLS server so
  .wsm/.my/.lisp get real diagnostics/completion" — це відповідь на
  задачу `WSM-OPENCODE-LSP-EXTENSIONS`, виконану координатором agent-team
  аудиту в цій же сесії, з реальними тестами
  (`packages/opencode/test/lsp/language.test.ts`, `wsm-root.test.ts`,
  `index.test.ts`).

`tasks.my` не оновлено після другого коміту.

## Виправлення (не зроблено)

Позначити `TAURICODE-WSM-LANGUAGE-MAP` як `(done . t)` з посиланням на
обидва коміти як доказ.
