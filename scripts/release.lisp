; scripts/release.my — tag and push a Tauricode release, triggering
; .github/workflows/tauricode-release.yml (four parallel build jobs:
; windows-desktop, windows-console, linux-desktop, linux-console — the
; Tauri shell and the swarm-cli/ecosystem-scheduler CLI crates, on both
; platforms), which assembles one GitHub Release from whatever actually
; built.
;
; Modeled directly on my-lisp's own scripts/release.my (same
; process-run/*argv*/report-and-exit-if-failed shape, read before
; writing this) — one real difference: this project's own tag prefix is
; `t`, not `l` (confirmed live: the t0.0.1 tag already exists on this
; repo, cut by hand the same way this script now automates).
;
; No cargo test --workspace gate here, unlike my-lisp's version: there
; is no single workspace in this repo (crates/swarm-cli,
; crates/ecosystem-scheduler, crates/ecosystem-observer, and
; prototype/swarm_dashboard/src-tauri are all standalone manifests, by
; design — see docs/STAGE2-RUNTIME-ADAPTER-PLAN.md's own note on this
; convention), so there is no one local test command that covers what
; this release actually ships. Each build job in
; tauricode-release.yml compiles its own crate directly; a real build
; failure is caught there, per-target, not hidden behind a single
; local gate that wouldn't have covered the failing target anyway.
;
; No version is bumped in any Cargo.toml: the four release targets
; are four independently-versioned crates already (swarm-cli 0.1.0,
; ecosystem-scheduler 0.1.0, ecosystem-observer 0.1.0, the Tauri shell
; 0.1.0) and the release tag is a product-level release marker, not a
; crate version — confirmed by reading crates/*/Cargo.toml directly:
; none of the four ships a version tied to this tag scheme.
;
;   /home/agents/GitHub/my-lisp/target/release/my-lisp \
;     --allow-process=git \
;     scripts/release.my 0.0.2
;
; Without --allow-process=git, process-run fails named — same disabled-
; by-default posture my-lisp's own docs/language-core.md documents, and
; the same reason this project's own copy of the doctrine applies:
; run this by hand, deliberately, never from something that also
; accepts network input in the same session.
;
; scripts/release.my — тегує й пушить Tauricode-реліз, запускаючи
; .github/workflows/tauricode-release.yml (чотири паралельні джоби
; збірки: windows-desktop, windows-console, linux-desktop,
; linux-console — Tauri-оболонка та CLI-крейти swarm-cli/
; ecosystem-scheduler на обох платформах), який збирає один GitHub
; Release з того, що реально зібралось.
;
; Змодельовано напряму з власного scripts/release.my my-lisp (та сама
; форма process-run/*argv*/report-and-exit-if-failed, прочитано перед
; написанням цього) — одна реальна відмінність: власний префікс тегу
; цього проєкту — `t`, не `l` (перевірено наживо: тег t0.0.1 вже існує
; в цьому репо, зроблений вручну тим самим способом, який цей скрипт
; тепер автоматизує).
;
; Тут немає гейту `cargo test --workspace`, на відміну від версії
; my-lisp: у цьому репо немає єдиного workspace (crates/swarm-cli,
; crates/ecosystem-scheduler, crates/ecosystem-observer та
; prototype/swarm_dashboard/src-tauri — усі окремі маніфести,
; навмисно), тож немає жодної локальної тестової команди, яка
; покривала б усе, що реально постачає цей реліз. Кожна build-джоба в
; tauricode-release.yml компілює свій крейт напряму, справжній провал
; збірки ловиться там, по кожному таргету, а не ховається за одним
; локальним гейтом, який все одно не покрив би провалений таргет.
;
; Жодна версія не піднімається в жодному Cargo.toml: чотири релізні
; таргети — це вже чотири незалежно версійовані крейти (swarm-cli
; 0.1.0, ecosystem-scheduler 0.1.0, ecosystem-observer 0.1.0, Tauri-
; оболонка 0.1.0), а релізний тег — це мітка релізу на рівні продукту,
; не версія крейту — перевірено прямим читанням crates/*/Cargo.toml:
; жоден з чотирьох не має версії, привʼязаної до цієї схеми тегів.

(def *release-remote* "origin")

(def *release-tag*
  (cond
    ((atom *argv*)
     (error-missing-version))
    (t (string-append "t" (car *argv*)))))

; `error-missing-version` deliberately isn't defined, same reasoning as
; my-lisp's own release.my: calling an unbound symbol is a real named
; failure (ErrorKind::UnknownSymbol), not an invented ad-hoc error this
; language doesn't otherwise have.
; `error-missing-version` свідомо не визначено, та сама причина, що й
; у власному release.my my-lisp: виклик незв'язаного символу — це
; реальний названий провал (ErrorKind::UnknownSymbol), не вигаданий
; ad-hoc примітив помилки, якого ця мова інакше не має.

(def report-and-exit-if-failed
  (lambda (label result)
    (let ((exit-code (car result)))
      (cond
        ((eq exit-code 0)
         (print (string-append label ": ok")))
        (t ((lambda ()
              (print (string-append label ": FAILED"))
              (print (second result))
              (print (third result))
              (print "Release aborted.")
              exit-code)))))))

(print (string-append "Tagging " (string-append *release-tag* "...")))

(def tag-result (process-run "git" (list "tag" *release-tag*)))
(report-and-exit-if-failed "git tag" tag-result)

(cond
  ((eq (car tag-result) 0)
   ((lambda ()
      (def push-result
        (process-run "git" (list "push" *release-remote* *release-tag*)))
      (report-and-exit-if-failed "git push" push-result)
      (cond
        ((eq (car push-result) 0)
         ((lambda ()
            (print (string-append "Tagged and pushed " *release-tag*))
            (print "tauricode-release.yml is now building windows-desktop, windows-console, linux-desktop, linux-console.")
            (print (string-append
                     "Check status: gh run list --repo juv4uk/tauricode --workflow=tauricode-release.yml | grep "
                     *release-tag*))
            (print (string-append
                     "Release page once builds finish: https://github.com/juv4uk/tauricode/releases/tag/"
                     *release-tag*)))))
        (t (quote push-failed))))))
  (t (quote tag-failed)))
