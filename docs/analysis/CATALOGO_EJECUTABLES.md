# Catálogo de nombres EXE y contenido real

Fecha: 21 de octubre de 2026. Fuente: `plugins-inventory.json`. Análisis estático; no se ejecutaron los objetivos.

Este anexo cubre cada ocurrencia `.exe` analizada en los plugins y los ZIP derivados MOBASE/XW. Un nombre EXE puede ser PE, script, enlace o contenido no resuelto. Las descripciones de herramientas conocidas son **roles inferidos** por nombre/delegación y metadatos; no constituyen descompilación exhaustiva ni validación funcional. Se conserva evidencia local seleccionada cuando existe.

## Resumen

- Ocurrencias `.exe`: **485**.
- Tipos reales entre ellas: `{'pe': 190, 'mz-unparsed': 6, 'script': 4, 'cygwin-symlink': 285}`.
- Un enlace hacia un multicall requiere validar su applet; no es un binario nuevo.
- Los bytes LZMA con cabecera adaptada son derivados: el original permanece intacto y ambos hashes se registran.
- `mz-unparsed` significa que el parser local no pudo establecer un PE, no que se haya demostrado un ejecutable DOS.

## Índice por programa

| Programa | Ocurrencias | Función candidata |
|---|---:|---|
| `_rxvt.exe` | 1 | Terminal gráfica de familia rxvt |
| `_ssh.exe` | 2 | Backend SSH seleccionado por wrappers |
| `_ssh1.exe` | 2 | Backend SSH alternativo; versión de protocolo no deducible del nombre |
| `_vim.exe` | 1 | Binario de editor Vim |
| `_xkbcomp.exe` | 1 | Backend compilador de mapas XKB |
| `ascii-xfr.exe` | 1 | Transferencia ASCII por terminal/serie |
| `autossh.exe` | 2 | Supervisión y restablecimiento de conexiones SSH |
| `awk.exe` | 1 | Procesamiento de texto por registros y patrones AWK |
| `balance.exe` | 1 | Distribución de conexiones TCP |
| `bash.exe` | 2 | Intérprete de comandos Bash |
| `bc.exe` | 1 | Calculadora de precisión arbitraria |
| `beep.exe` | 1 | Aviso acústico |
| `busybox.exe` | 1 | Ejecutable multicall de utilidades Unix |
| `c32termd.exe` | 1 | Puente de terminal Windows/Cygwin de 32 bits; IPC pendiente |
| `c64termd.exe` | 1 | Puente de terminal Windows/Cygwin de 64 bits; IPC pendiente |
| `cfourcc.exe` | 1 | Cambio del identificador FourCC en AVI |
| `conin.exe` | 2 | Puente de entrada de consola; contrato exacto pendiente |
| `connect.exe` | 1 | Proxy de conexión TCP para túneles |
| `ctris.exe` | 2 | Juego de bloques en terminal |
| `curl.exe` | 2 | Cliente de transferencias por URL |
| `cygcheck.exe` | 1 | Diagnóstico de instalación/dependencias Cygwin |
| `cygpath.exe` | 1 | Conversión entre rutas Unix y Windows |
| `cygstart.exe` | 1 | Abrir aplicaciones/documentos mediante Windows |
| `cygtermd.exe` | 2 | Adaptación de terminal Cygwin; protocolo IPC pendiente |
| `cygwin-console-helper.exe` | 2 | Ayudante de consola de Cygwin |
| `dwm.exe` | 1 | Gestor de ventanas X de estilo dwm |
| `dwm_w32.exe` | 1 | Gestor de ventanas adaptado a Windows |
| `editrights.exe` | 1 | Administración de derechos Windows |
| `extract_compressed_fs.exe` | 1 | Extracción de sistema de archivos empaquetado; formato exacto pendiente |
| `fdupes.exe` | 1 | Identificación de archivos duplicados |
| `ftp.exe` | 1 | Cliente FTP |
| `fvwm.exe` | 1 | Gestor de ventanas X FVWM |
| `FvwmBacker.exe` | 1 | Módulo FVWM para fondo de escritorio |
| `FvwmTaskBar.exe` | 1 | Módulo de barra de tareas FVWM |
| `getfacl.exe` | 1 | Lectura de ACL |
| `git-add.exe` | 2 | Entrada al subcomando Git `add`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-am.exe` | 2 | Entrada al subcomando Git `am`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-annotate.exe` | 2 | Entrada al subcomando Git `annotate`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-apply.exe` | 2 | Entrada al subcomando Git `apply`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-archive.exe` | 2 | Entrada al subcomando Git `archive`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-bisect--helper.exe` | 2 | Entrada al subcomando Git `bisect--helper`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-blame.exe` | 2 | Entrada al subcomando Git `blame`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-branch.exe` | 2 | Entrada al subcomando Git `branch`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-bugreport.exe` | 2 | Entrada al subcomando Git `bugreport`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-bundle.exe` | 2 | Entrada al subcomando Git `bundle`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-cat-file.exe` | 2 | Entrada al subcomando Git `cat-file`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-check-attr.exe` | 2 | Entrada al subcomando Git `check-attr`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-check-ignore.exe` | 2 | Entrada al subcomando Git `check-ignore`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-check-mailmap.exe` | 2 | Entrada al subcomando Git `check-mailmap`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-check-ref-format.exe` | 2 | Entrada al subcomando Git `check-ref-format`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-checkout--worker.exe` | 2 | Entrada al subcomando Git `checkout--worker`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-checkout-index.exe` | 2 | Entrada al subcomando Git `checkout-index`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-checkout.exe` | 2 | Entrada al subcomando Git `checkout`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-cherry-pick.exe` | 2 | Entrada al subcomando Git `cherry-pick`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-cherry.exe` | 2 | Entrada al subcomando Git `cherry`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-clean.exe` | 2 | Entrada al subcomando Git `clean`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-clone.exe` | 2 | Entrada al subcomando Git `clone`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-column.exe` | 2 | Entrada al subcomando Git `column`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-commit-graph.exe` | 2 | Entrada al subcomando Git `commit-graph`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-commit-tree.exe` | 2 | Entrada al subcomando Git `commit-tree`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-commit.exe` | 2 | Entrada al subcomando Git `commit`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-config.exe` | 2 | Entrada al subcomando Git `config`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-count-objects.exe` | 2 | Entrada al subcomando Git `count-objects`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-credential-cache--daemon.exe` | 2 | Entrada al subcomando Git `credential-cache--daemon`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-credential-cache.exe` | 2 | Entrada al subcomando Git `credential-cache`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-credential-store.exe` | 2 | Entrada al subcomando Git `credential-store`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-credential.exe` | 2 | Entrada al subcomando Git `credential`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-describe.exe` | 2 | Entrada al subcomando Git `describe`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-diff-files.exe` | 2 | Entrada al subcomando Git `diff-files`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-diff-index.exe` | 2 | Entrada al subcomando Git `diff-index`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-diff-tree.exe` | 2 | Entrada al subcomando Git `diff-tree`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-diff.exe` | 2 | Entrada al subcomando Git `diff`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-difftool.exe` | 2 | Entrada al subcomando Git `difftool`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-env--helper.exe` | 2 | Entrada al subcomando Git `env--helper`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-fast-export.exe` | 2 | Entrada al subcomando Git `fast-export`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-fast-import.exe` | 2 | Entrada al subcomando Git `fast-import`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-fetch-pack.exe` | 2 | Entrada al subcomando Git `fetch-pack`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-fetch.exe` | 2 | Entrada al subcomando Git `fetch`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-fmt-merge-msg.exe` | 2 | Entrada al subcomando Git `fmt-merge-msg`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-for-each-ref.exe` | 2 | Entrada al subcomando Git `for-each-ref`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-for-each-repo.exe` | 2 | Entrada al subcomando Git `for-each-repo`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-format-patch.exe` | 2 | Entrada al subcomando Git `format-patch`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-fsck-objects.exe` | 2 | Entrada al subcomando Git `fsck-objects`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-fsck.exe` | 2 | Entrada al subcomando Git `fsck`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-fsmonitor--daemon.exe` | 2 | Entrada al subcomando Git `fsmonitor--daemon`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-gc.exe` | 2 | Entrada al subcomando Git `gc`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-get-tar-commit-id.exe` | 2 | Entrada al subcomando Git `get-tar-commit-id`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-grep.exe` | 2 | Entrada al subcomando Git `grep`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-hash-object.exe` | 2 | Entrada al subcomando Git `hash-object`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-help.exe` | 2 | Entrada al subcomando Git `help`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-hook.exe` | 2 | Entrada al subcomando Git `hook`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-index-pack.exe` | 2 | Entrada al subcomando Git `index-pack`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-init-db.exe` | 2 | Entrada al subcomando Git `init-db`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-init.exe` | 2 | Entrada al subcomando Git `init`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-interpret-trailers.exe` | 2 | Entrada al subcomando Git `interpret-trailers`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-log.exe` | 2 | Entrada al subcomando Git `log`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-ls-files.exe` | 2 | Entrada al subcomando Git `ls-files`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-ls-remote.exe` | 2 | Entrada al subcomando Git `ls-remote`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-ls-tree.exe` | 2 | Entrada al subcomando Git `ls-tree`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-mailinfo.exe` | 2 | Entrada al subcomando Git `mailinfo`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-mailsplit.exe` | 2 | Entrada al subcomando Git `mailsplit`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-maintenance.exe` | 2 | Entrada al subcomando Git `maintenance`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-merge-base.exe` | 2 | Entrada al subcomando Git `merge-base`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-merge-file.exe` | 2 | Entrada al subcomando Git `merge-file`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-merge-index.exe` | 2 | Entrada al subcomando Git `merge-index`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-merge-ours.exe` | 2 | Entrada al subcomando Git `merge-ours`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-merge-recursive.exe` | 2 | Entrada al subcomando Git `merge-recursive`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-merge-subtree.exe` | 2 | Entrada al subcomando Git `merge-subtree`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-merge-tree.exe` | 2 | Entrada al subcomando Git `merge-tree`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-merge.exe` | 2 | Entrada al subcomando Git `merge`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-mktag.exe` | 2 | Entrada al subcomando Git `mktag`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-mktree.exe` | 2 | Entrada al subcomando Git `mktree`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-multi-pack-index.exe` | 2 | Entrada al subcomando Git `multi-pack-index`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-mv.exe` | 2 | Entrada al subcomando Git `mv`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-name-rev.exe` | 2 | Entrada al subcomando Git `name-rev`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-notes.exe` | 2 | Entrada al subcomando Git `notes`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-pack-objects.exe` | 2 | Entrada al subcomando Git `pack-objects`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-pack-redundant.exe` | 2 | Entrada al subcomando Git `pack-redundant`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-pack-refs.exe` | 2 | Entrada al subcomando Git `pack-refs`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-patch-id.exe` | 2 | Entrada al subcomando Git `patch-id`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-prune-packed.exe` | 2 | Entrada al subcomando Git `prune-packed`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-prune.exe` | 2 | Entrada al subcomando Git `prune`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-pull.exe` | 2 | Entrada al subcomando Git `pull`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-push.exe` | 2 | Entrada al subcomando Git `push`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-range-diff.exe` | 2 | Entrada al subcomando Git `range-diff`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-read-tree.exe` | 2 | Entrada al subcomando Git `read-tree`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-rebase.exe` | 2 | Entrada al subcomando Git `rebase`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-receive-pack.exe` | 4 | Entrada al subcomando Git `receive-pack`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-reflog.exe` | 2 | Entrada al subcomando Git `reflog`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-remote-ext.exe` | 2 | Entrada al subcomando Git `remote-ext`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-remote-fd.exe` | 2 | Entrada al subcomando Git `remote-fd`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-remote-ftp.exe` | 2 | Entrada al subcomando Git `remote-ftp`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-remote-ftps.exe` | 2 | Entrada al subcomando Git `remote-ftps`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-remote-https.exe` | 2 | Entrada al subcomando Git `remote-https`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-remote.exe` | 2 | Entrada al subcomando Git `remote`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-repack.exe` | 2 | Entrada al subcomando Git `repack`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-replace.exe` | 2 | Entrada al subcomando Git `replace`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-rerere.exe` | 2 | Entrada al subcomando Git `rerere`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-reset.exe` | 2 | Entrada al subcomando Git `reset`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-restore.exe` | 2 | Entrada al subcomando Git `restore`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-rev-list.exe` | 2 | Entrada al subcomando Git `rev-list`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-rev-parse.exe` | 2 | Entrada al subcomando Git `rev-parse`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-revert.exe` | 2 | Entrada al subcomando Git `revert`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-rm.exe` | 2 | Entrada al subcomando Git `rm`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-send-pack.exe` | 2 | Entrada al subcomando Git `send-pack`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-shortlog.exe` | 2 | Entrada al subcomando Git `shortlog`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-show-branch.exe` | 2 | Entrada al subcomando Git `show-branch`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-show-index.exe` | 2 | Entrada al subcomando Git `show-index`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-show-ref.exe` | 2 | Entrada al subcomando Git `show-ref`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-show.exe` | 2 | Entrada al subcomando Git `show`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-sparse-checkout.exe` | 2 | Entrada al subcomando Git `sparse-checkout`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-stage.exe` | 2 | Entrada al subcomando Git `stage`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-stash.exe` | 2 | Entrada al subcomando Git `stash`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-status.exe` | 2 | Entrada al subcomando Git `status`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-stripspace.exe` | 2 | Entrada al subcomando Git `stripspace`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-submodule--helper.exe` | 2 | Entrada al subcomando Git `submodule--helper`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-switch.exe` | 2 | Entrada al subcomando Git `switch`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-symbolic-ref.exe` | 2 | Entrada al subcomando Git `symbolic-ref`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-tag.exe` | 2 | Entrada al subcomando Git `tag`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-unpack-file.exe` | 2 | Entrada al subcomando Git `unpack-file`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-unpack-objects.exe` | 2 | Entrada al subcomando Git `unpack-objects`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-update-index.exe` | 2 | Entrada al subcomando Git `update-index`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-update-ref.exe` | 2 | Entrada al subcomando Git `update-ref`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-update-server-info.exe` | 2 | Entrada al subcomando Git `update-server-info`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-upload-archive.exe` | 4 | Entrada al subcomando Git `upload-archive`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-upload-pack.exe` | 4 | Entrada al subcomando Git `upload-pack`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-var.exe` | 2 | Entrada al subcomando Git `var`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-verify-commit.exe` | 2 | Entrada al subcomando Git `verify-commit`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-verify-pack.exe` | 2 | Entrada al subcomando Git `verify-pack`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-verify-tag.exe` | 2 | Entrada al subcomando Git `verify-tag`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-whatchanged.exe` | 2 | Entrada al subcomando Git `whatchanged`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-worktree.exe` | 2 | Entrada al subcomando Git `worktree`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git-write-tree.exe` | 2 | Entrada al subcomando Git `write-tree`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando |
| `git.exe` | 4 | Sistema de control de versiones Git y dispatcher de subcomandos |
| `glxgears.exe` | 1 | Demostración/diagnóstico de renderizado GLX |
| `glxinfo.exe` | 1 | Información de GLX/OpenGL |
| `greed.exe` | 2 | Juego de terminal |
| `grep.exe` | 2 | Selección de líneas por patrones y expresiones regulares |
| `httping.exe` | 1 | Sondeo de latencia HTTP |
| `iconv.exe` | 1 | Conversión de codificaciones |
| `iperf.exe` | 2 | Medición de rendimiento de red con protocolo iperf |
| `iperf3.exe` | 1 | Medición de rendimiento de red con protocolo iperf3 |
| `jwm.exe` | 1 | Gestor de ventanas X JWM |
| `kill.exe` | 1 | Envío de señales a procesos |
| `kinit.exe` | 2 | Obtención de tickets Kerberos |
| `klist.exe` | 2 | Consulta de tickets Kerberos |
| `kpasswd.exe` | 1 | Cambio de contraseña Kerberos |
| `ldd.exe` | 1 | Consulta de dependencias dinámicas |
| `ldh.exe` | 1 | Auxiliar de carga; función no establecida |
| `less.exe` | 1 | Paginador de texto |
| `lrz.exe` | 2 | Recepción X/Y/ZMODEM |
| `lsz.exe` | 2 | Envío X/Y/ZMODEM |
| `m32termd.exe` | 1 | Puente de terminal de 32 bits; IPC pendiente |
| `m64termd.exe` | 1 | Puente de terminal de 64 bits; IPC pendiente |
| `mathomatic.exe` | 1 | Álgebra simbólica en terminal |
| `minicom.exe` | 1 | Terminal serie y transferencia |
| `mkgroup.exe` | 1 | Generación de mapeos de grupos para Cygwin |
| `mkpasswd.exe` | 1 | Generación de mapeos de usuarios para Cygwin |
| `mkshortcut.exe` | 1 | Creación de accesos directos Windows |
| `MobaBox.exe` | 2 | Ejecutable multicall de utilidades de la suite |
| `MobAgent.exe` | 1 | Agente SSH integrado en la suite; CRC del miembro incoherente, bytes no confiables |
| `MobaKeyGen.exe` | 1 | Generador/conversor gráfico de claves con indicios PuTTYgen |
| `MobaRTE.exe` | 1 | Auxiliar gráfico multimodo: editor, diff, navegador y RDP; ver informe principal |
| `MobaSCP.exe` | 1 | Auxiliar de transferencias SSH/SCP/SFTP y coordinación con GUI; límites exactos pendientes |
| `MobaVNC.exe` | 2 | Cliente VNC de la suite; variante exacta pendiente |
| `mosh-client.exe` | 2 | Cliente de terminal remota Mosh |
| `MoTTY.exe` | 1 | Cliente terminal/SSH derivado o adaptado de PuTTY; detalles por código pendientes |
| `mount.exe` | 2 | Gestión/consulta de montajes Cygwin |
| `nc.exe` | 1 | Conexiones TCP/UDP tipo netcat |
| `ncdu.exe` | 1 | Explorador de uso de disco |
| `nimesweeper.exe` | 1 | Juego de buscaminas |
| `ninvaders.exe` | 1 | Juego de invasores en terminal |
| `openssl.exe` | 2 | Herramientas criptográficas y TLS |
| `OpenX.exe` | 2 | Auxiliar de lanzamiento/integración X; no confundir por nombre con servidor X completo |
| `passwd.exe` | 1 | Gestión de contraseña del entorno |
| `paste.exe` | 1 | Combinación de columnas de texto |
| `portmap.exe` | 1 | Mapeador de puertos RPC |
| `ps.exe` | 1 | Listado de procesos |
| `rcp.exe` | 2 | Copia remota heredada |
| `regtool.exe` | 1 | Acceso al registro de Windows |
| `rlogin.exe` | 2 | Login remoto heredado |
| `rpc.mountd.exe` | 1 | Servicio auxiliar de montaje NFS |
| `rpc.nfsd.exe` | 1 | Servidor NFS |
| `rsh.exe` | 2 | Ejecución remota heredada |
| `rsync.exe` | 2 | Sincronización incremental de archivos |
| `runscript.exe` | 1 | Ejecución de scripts de conversación de terminal |
| `scp.exe` | 2 | Copia de archivos mediante SSH |
| `setfacl.exe` | 1 | Modificación de ACL |
| `setxkbmap.exe` | 2 | Selección de mapa de teclado XKB |
| `sftp-server.exe` | 2 | Subsistema servidor SFTP |
| `sftp.exe` | 2 | Cliente SFTP |
| `showkey.exe` | 2 | Diagnóstico de teclas |
| `showmount.exe` | 1 | Consulta de exportaciones NFS |
| `shutdown.exe` | 1 | Apagado/reinicio del sistema |
| `socat.exe` | 1 | Puente bidireccional entre endpoints |
| `soliterm.exe` | 2 | Acceso Serial-over-LAN; verificar variante |
| `ssh-add.exe` | 2 | Carga y consulta de identidades en agente SSH |
| `ssh-agent-original.exe` | 1 | Agente SSH original del paquete |
| `ssh-agent.exe` | 2 | Agente SSH o wrapper hacia otro agente |
| `ssh-keygen.exe` | 2 | Generación y conversión de claves SSH |
| `ssh-keyscan.exe` | 2 | Obtención de claves públicas anunciadas por servidores SSH |
| `ssh-pageant.exe` | 1 | Puente de agente SSH hacia Pageant |
| `ssh-pkcs11-helper.exe` | 2 | Auxiliar de acceso SSH a proveedores PKCS#11 |
| `ssh-sk-helper.exe` | 2 | Auxiliar para claves SSH respaldadas por hardware |
| `ssh.exe` | 2 | Cliente SSH o wrapper que selecciona backend |
| `sshd-auth.exe` | 2 | Subproceso de autenticación de sshd |
| `sshd-session.exe` | 2 | Subproceso de sesión de sshd |
| `sshd.exe` | 2 | Servidor SSH |
| `sshpass.exe` | 1 | Automatización de entrada de contraseña SSH; evitar replicar almacenamiento inseguro |
| `stgames.exe` | 1 | Colección/launcher de juegos; alcance pendiente |
| `strace.exe` | 1 | Trazado/diagnóstico de procesos Cygwin |
| `sudoku.exe` | 2 | Juego Sudoku |
| `telnet.exe` | 1 | Cliente Telnet |
| `termwalk.exe` | 2 | Utilidad de terminal; función exacta pendiente |
| `toybox.exe` | 2 | Ejecutable multicall de utilidades Unix |
| `tput.exe` | 1 | Consulta/control de capacidades de terminal |
| `TurboVNC.exe` | 1 | Componente VNC; función cliente/servidor pendiente de confirmar por código |
| `twm_w32.exe` | 1 | Gestor de ventanas twm adaptado a Windows |
| `umount.exe` | 1 | Desmontaje de rutas Cygwin |
| `vim.exe` | 2 | Editor Vim o wrapper hacia su binario |
| `vnp.exe` | 2 | Utilidad no identificada: necesita análisis adicional |
| `wget.exe` | 1 | Descarga por HTTP/FTP |
| `whois.exe` | 1 | Consulta de registros WHOIS |
| `winpty-agent.exe` | 1 | Agente de consola de winpty |
| `winpty.exe` | 1 | Adaptación PTY/console Windows |
| `xauth.exe` | 1 | Gestión de cookies de autenticación X |
| `xdpyinfo.exe` | 1 | Consulta de capacidades de servidor X |
| `xev.exe` | 1 | Visualizador de eventos X |
| `xhost.exe` | 2 | Control de acceso a servidor X |
| `xkbcomp.exe` | 1 | Compilador de mapas XKB |
| `xkbcomp_w32.exe` | 1 | Compilador XKB adaptado a Windows |
| `xkbevd.exe` | 1 | Daemon de eventos XKB |
| `xkill.exe` | 1 | Terminación de clientes X |
| `xlsatoms.exe` | 2 | Listado de átomos X |
| `xlsclients.exe` | 2 | Listado de clientes X |
| `xlsfonts.exe` | 2 | Listado de fuentes X |
| `xmodmap.exe` | 2 | Consulta/modificación de mapas de teclas y botones X |
| `xprop.exe` | 2 | Consulta/modificación de propiedades X |
| `xrdb.exe` | 2 | Base de recursos X |
| `xwinwm.exe` | 1 | Integración de ventanas X con Windows |
| `zsh.exe` | 2 | Intérprete de comandos Zsh |
| `zstd-decompress.exe` | 2 | Descompresión Zstandard |

## Evidencia por ocurrencia

### _rxvt.exe

Terminal gráfica de familia rxvt.

- **Ruta:** `CygUtils.plugin!/bin/_rxvt.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `0eb227bea70a126edf5be712ad02902dc46027f905c695ec20ee193cb818696e`. Hash derivado: `fba36a7a52bc3c5cbee92531fa9e84226d09b21c4b4e1956f8a825b46690e4a8`; tamaño derivado: 190976 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`, `USER32.dll`.
  Indicio en offset derivado `0x25a14`: `Usage: `.

### _ssh.exe

Backend SSH seleccionado por wrappers.

- **Ruta:** `CygUtils.plugin!/bin/_ssh.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `d158458baeda6893557f6a37183f3271dae1bd2bb53858b5be083107dbadebb2`. Hash derivado: `21bc136006ad9b9ab3dfa197bc193a62b51d3f1edf06805f2197d47fb42212fd`; tamaño derivado: 341006 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygopenssh.dll`, `cygcrypto-3.dll`, `cygwin1.dll`, `cyggssapi_krb5-2.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`, `USER32.dll`.
  Indicio en offset derivado `0x3a840`: `usage: ssh [-46AaCfGgKkMNnqsTtVvXxYy] [-B bind_interface] [-b bind_address]`.
  Indicio en offset derivado `0x3b458`: `OpenSSH_10.3p1`.
- **Ruta:** `CygUtils64.plugin!/bin/_ssh.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `a15aa6678971e3ceafc5701ae8c2420c6c0b3e5aa18a5e3b1ec6e2453566290d`. Hash derivado: `3b1747eb3eefebf447a71f71fdcc4e90baac37417df14bee5e3131740462cdb3`; tamaño derivado: 324608 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygopenssh.dll`, `cygcrypto-3.dll`, `cygwin1.dll`, `cyggssapi_krb5-2.dll`, `cyggcc_s-seh-1.dll`, `KERNEL32.dll`, `USER32.dll`.
  Indicio en offset derivado `0x3a800`: `usage: ssh [-46AaCfGgKkMNnqsTtVvXxYy] [-B bind_interface] [-b bind_address]`.
  Indicio en offset derivado `0x3b458`: `OpenSSH_10.3p1`.

### _ssh1.exe

Backend SSH alternativo; versión de protocolo no deducible del nombre.

- **Ruta:** `CygUtils.plugin!/bin/_ssh1.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `7839dd0bfdcf8053b2b74dc510487f3136652dd296e3b9db697b4ea596b199a9`. Hash derivado: `1244121e14b6d58087019df8e10349aeef99fa55170c4d70716d519256b317be`; tamaño derivado: 889358 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygcrypto-3.dll`, `cygwin1.dll`, `cygz.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x91240`: `usage: ssh [-1246AaCfGgKkMNnqsTtVvXxYy] [-b bind_address] [-c cipher_spec]`.
  Indicio en offset derivado `0x918f4`: `OpenSSH_7.5p1`.
- **Ruta:** `CygUtils64.plugin!/bin/_ssh1.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `bf3d9ef2a1c5f4065c34e1e96408247cd32e4c9e9512a7fcc30e5459c808721d`. Hash derivado: `3a948ed85211f631dbccf07399ae0d22fe2deb570feb1849570bb8832b7e4ef0`; tamaño derivado: 688128 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygcrypto-3.dll`, `cygwin1.dll`, `cygz.dll`, `cyggcc_s-seh-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x69018`: `usage: ssh [-1246AaCfGgKkMNnqsTtVvXxYy] [-b bind_address] [-c cipher_spec]`.
  Indicio en offset derivado `0x69a30`: `OpenSSH_7.5p1`.

### _vim.exe

Binario de editor Vim.

- **Ruta:** `CygUtils64.plugin!/bin/_vim.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `c0b15e89e3ad29f0ce4c23fe9ef6339ce4e2b529f4bf46252f2264301a0403f5`. Hash derivado: `b4ecb92edd243819643064b1bb86778184440f4972275560061a566976f6dd1a`; tamaño derivado: 1570816 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `cygncursesw-10.dll`, `KERNEL32.dll`, `USER32.dll`.
  Indicio en offset derivado `0x14c149`: `version 6.0`.
  Indicio en offset derivado `0x152c88`: `Conversion with 'charconvert' failed`.

### _xkbcomp.exe

Backend compilador de mapas XKB.

- **Ruta:** `CygUtils.plugin!/bin/_xkbcomp.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `cf0aa7079f9d0228b566bffed6c8433ce34ea3e84054bb11e17315e2a8a5c711`. Hash derivado: `9212bab9b693001bed6b15c7d0209dbd1e7d88d614ded7e0781fc8e87c26a42c`; tamaño derivado: 180224 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `cygX11-6.dll`, `cygxkbfile-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x27810`: `%s was compiled with XKB version %d.%02d`.
  Indicio en offset derivado `0x2783c`: `X library supports incompatible version %d.%02d`.

### ascii-xfr.exe

Transferencia ASCII por terminal/serie.

- **Ruta:** `CygUtils.plugin!/bin/ascii-xfr.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `c31fd1bdec57baf566d4d4bf2548aaa2980a4fd1efc99f35180ed4724b4f33cd`. Hash derivado: `54fcb6199f5316a45d72620e4f26e31962b49af6ca89383edc6e551d864eaa27`; tamaño derivado: 10254 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x1ac4`: `Usage: ascii-xfr -s\|-r [-dvn] [-l linedelay] [-c character delay] filename`.
  Indicio en offset derivado `0x1d68`: `  Unknown pseudo relocation protocol version %d.`.

### autossh.exe

Supervisión y restablecimiento de conexiones SSH.

- **Ruta:** `CygUtils.plugin!/bin/autossh.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `4b5aeaae64741778fc32199391c5f6045e15199d6bbc8790283b8e71d2927471`. Hash derivado: `060c1f6209e68eabe6eb696b09208fc3977299b1edf6347146746ba4e05c27d0`; tamaño derivado: 21518 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x3710`: `usage: %s [-V] [-M monitor_port[:echo_port]] [-f] [SSH_OPTIONS]`.
  Indicio en offset derivado `0x389c`: `    -V print autossh version and exit.`.
- **Ruta:** `CygUtils64.plugin!/bin/autossh.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `c20ad70ee8da2f3b64a412c5c3d846015575730f200000b4b5dec7f62d87e395`. Hash derivado: `76eaac998af4a9bf9e186f29903a8678279d82e636de4f265f8fae5fd918cf2a`; tamaño derivado: 27648 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x3e80`: `usage: %s [-V] [-M monitor_port[:echo_port]] [-f] [SSH_OPTIONS]`.
  Indicio en offset derivado `0x4010`: `    -V print autossh version and exit.`.

### awk.exe

Procesamiento de texto por registros y patrones AWK.

- **Ruta:** `CygUtils64.plugin!/bin/awk.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `8fda1f3ce3e4ed276055f0216be75197b7586d18bd343c455791392d34c0b485`. Hash derivado: `dcc77ba89d85f4e1add44ca8b45d4e5f9daaa0c57832520c9e2931b351e792ab`; tamaño derivado: 151040 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x1ff98`: `usage: %s [-F fs] [-v var=value] [-f progfile \| 'prog'] [file ...]`.
  Indicio en offset derivado `0x201b3`: `version 20121220`.

### balance.exe

Distribución de conexiones TCP.

- **Ruta:** `CygUtils.plugin!/bin/balance.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `185ea5f3231f922f37891508519b594347e53d702c278f076ac2f8d979088069`. Hash derivado: `bde1d0bbc65c35a38ecbd2837fc16143a2a2137288a312a8a063d1e6b32bd64f`; tamaño derivado: 28686 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x4fcf`: `usage:`.
  Indicio en offset derivado `0x5f10`: `  version                        show version id`.

### bash.exe

Intérprete de comandos Bash.

- **Ruta:** `CygUtils.plugin!/bin/bash.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `f08bdad072cfca6f7eccd65b18b65ba991635bba3fa21534ea197b692045fd8b`. Hash derivado: `ea7f8f53793bc94c5d0c60fd674cf3fe6581fc73de9ad561ff6a9c441375b8e7`; tamaño derivado: 909838 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0xabe5e`: `GNU bash, version %s-(%s)`.
  Indicio en offset derivado `0xabe7c`: `Usage:`.
- **Ruta:** `CygUtils64.plugin!/bin/bash.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `04774300332f1e694cf6e5af8ddffd73005aa07ad2e323250fad8436791a4dfe`. Hash derivado: `ffe3504986c8ebe35e9e2543b38c60ba20073f2b9295efa1a3920d388974d9cc`; tamaño derivado: 1052672 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0xd3417`: `GNU bash, version %s-(%s)`.
  Indicio en offset derivado `0xd3438`: `Usage:`.

### bc.exe

Calculadora de precisión arbitraria.

- **Ruta:** `CygUtils.plugin!/bin/bc.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `2c544bf17d88a3ef8ee36dbf9d5af6900b35afaf0dddfb58afde25c93478260b`. Hash derivado: `bbc21904dd1633c8f1e3542ba9fcdecab444d92e423091338ea3f01226761b04`; tamaño derivado: 68096 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cygreadline7.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0xb400`: `  -v  --version      print version information and exit`.
  Indicio en offset derivado `0xb580`: `usage: %s [options] [file ...]`.

### beep.exe

Aviso acústico.

- **Ruta:** `CygUtils.plugin!/bin/beep.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `31044605b95da0cf95f11f4057077b5e09343717c5f06aa15bfabadf1fbab2e2`. Hash derivado: `e6769e26496d338a2b5574b38c506d22cef6d1efa7bbbfb394389d72c6ff6d3e`; tamaño derivado: 6144 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x1230`: `Usage:  beep [<frequency> [<duration>]]`.
  Indicio en offset derivado `0x12b0`: `  Unknown pseudo relocation protocol version %d.`.

### busybox.exe

Ejecutable multicall de utilidades Unix.

- **Ruta:** `CygUtils.plugin!/bin/busybox.exe`; tipo `mz-unparsed`; arquitectura `no establecida`.
  Hash original: `f5cde65378ddb7fa1d6647bc4a3bfcb09765b1e51ed4695c1c1ebe9d269e13bb`. Hash derivado: `9e48b7440bb7aa5b48118d18a10911a8e097f14e8749b3fcf0629861c4c32df1`; tamaño derivado: 431640 B; método `lzma-derived-header-byte1-zero`.

### c32termd.exe

Puente de terminal Windows/Cygwin de 32 bits; IPC pendiente.

- **Ruta:** `MOBASE!/bin/exttermd/c32termd.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `347a4ca620fc216ed37e7b2430672f12217827e626280481be7a6a8cbf95a750`. Hash derivado: `347a4ca620fc216ed37e7b2430672f12217827e626280481be7a6a8cbf95a750`; tamaño derivado: 23566 B; método `direct`.
  **Integridad:** CRC32 declarado por ZIP no coincide; contenido retenido como observado no confiable. No se usa como prueba de integridad ni de comportamiento autorizado.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.

### c64termd.exe

Puente de terminal Windows/Cygwin de 64 bits; IPC pendiente.

- **Ruta:** `MOBASE!/bin/exttermd/c64termd.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `afb89e71a472928e5fa4626b8733fe33eab6119468463e1aca95fa009d4cb215`. Hash derivado: `afb89e71a472928e5fa4626b8733fe33eab6119468463e1aca95fa009d4cb215`; tamaño derivado: 31336 B; método `direct`.
  **Integridad:** CRC32 declarado por ZIP no coincide; contenido retenido como observado no confiable. No se usa como prueba de integridad ni de comportamiento autorizado.
  DLL importadas: ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `Z�`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `nv`, `�`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `MZ�`, `��`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `�KH�E�@,��~1H�EH�@0�`, ``, ``, `H���`, ``, ``, `�H���`, ``, ``, ``, ``, ``, ``, ``, `�@�U A����H�M����`, ``, ``, ``, ``, ``, ``, `.bss`, ``, `�H�� `, ``, ``, ``, ``, ``, ``, ``, `�=`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `�`, `.rdata`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `PE`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `H��([^�f.�`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `MZ�`, `MZ�`.
  Indicio en offset derivado `0x65c6`: `Mobatek1`.
  Indicio en offset derivado `0x65d8`: `Mobatek0`.

### cfourcc.exe

Cambio del identificador FourCC en AVI.

- **Ruta:** `CygUtils.plugin!/bin/cfourcc.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `61fd081d9ddf51cb686258f4ec4a5bd40c873d86d453a0949f03f0f30f880b67`. Hash derivado: `c193cfacae1fa89618f2509d28ae07ff267aafbfa54c8cc47580ca188353665f`; tamaño derivado: 5632 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.

### conin.exe

Puente de entrada de consola; contrato exacto pendiente.

- **Ruta:** `CygUtils.plugin!/bin/conin.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `76f0f677579933219eb5438fcc1cdd38b1a920378331fb1b210eea5d8151b420`. Hash derivado: `4af650b4cbeaa40057e7fab11a9ceabe3a45a4faf93ba849053744f5a31a1616`; tamaño derivado: 55310 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: ``, `MZ�`, `MZ�`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``.
  Indicio en offset derivado `0x7778`: `Usage: %s [options] [--] program [args]`.
  Indicio en offset derivado `0x782c`: `  --version   Show the winpty version number`.
- **Ruta:** `CygUtils64.plugin!/bin/conin.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `b3ba470b5185466d6723c4a4f1ddb042e236d524cfd847eb9a54466e9b5cf61d`. Hash derivado: `47cdbf0cb73577314f176b30be9fbe4e7c083360b432a4e7445ad55fe4ae6c7f`; tamaño derivado: 209408 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `cyggcc_s-seh-1.dll`, `KERNEL32.dll`, `winpty.dll`.
  Indicio en offset derivado `0x29540`: `Usage: %s [options] [--] program [args]`.
  Indicio en offset derivado `0x29600`: `  --version   Show the winpty version number`.

### connect.exe

Proxy de conexión TCP para túneles.

- **Ruta:** `CygUtils.plugin!/bin/connect.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `a7943597ef8fe946ec75ba7a7b65b232b29978ae17bf8bb022649485b2c2f7b4`. Hash derivado: `6a3de9170b95cc2f40690ddb699d987e8467e587a2d46a75026dede308a306cb`; tamaño derivado: 34816 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x7380`: `usage: %s [-dnhst45] [-p local-port][-R resolve] [-w timeout] `.
  Indicio en offset derivado `0x74c8`: `  Unknown pseudo relocation protocol version %d.`.

### ctris.exe

Juego de bloques en terminal.

- **Ruta:** `CygUtils.plugin!/bin/ctris.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `d8f091c73db5af2b05a7df3fc66d08a2f97c8c543429d0d479f4ecae56ef9e92`. Hash derivado: `ab550fbb73e851d8a423fa08b4e77df5313177b4ca3bed09f432007e9504d365`; tamaño derivado: 19968 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cygwin1.dll`, `cygncurses-8.dll`, `KERNEL32.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`.
  Indicio en offset derivado `0x4166`: `-v             version info`.
- **Ruta:** `CygUtils64.plugin!/bin/ctris.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `9043d20c97c065a1bdddcf50d0a0ea406f80a9e5906a519eda4cd3d6f66783bb`. Hash derivado: `b2179b4b6b89eccdb5c893a667dd854891cc3a2df22bcbb7d6830e3bb26ac059`; tamaño derivado: 50176 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `cygncursesw-10.dll`, `cyggcc_s-seh-1.dll`, `cygstdc++-6.dll`, `KERNEL32.dll`.

### curl.exe

Cliente de transferencias por URL.

- **Ruta:** `CygUtils.plugin!/bin/curl.exe`; tipo `mz-unparsed`; arquitectura `no establecida`.
  Hash original: `14e80e173e28ccedc8d72a0ccfdc57034c4ca20192e993e29fe7a337877afb08`. Hash derivado: `d779c3b6eb58aff61f8834bb727f31353ba30afbdd1bb1a6060a41f55242e83c`; tamaño derivado: 203790 B; método `lzma-derived-header-byte1-zero`.
- **Ruta:** `CygUtils64.plugin!/bin/curl.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `59bd07d62ffd92ac3eb8d9d79f052cff5be2bbc9c4f4798006ebe83cecaf0e6e`. Hash derivado: `e57d4972ba54066b88e6c70e50b832438a169b84b655f70f7a3f355ca9e1d86c`; tamaño derivado: 194048 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygcurl-4.dll`, `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x1d688`: `Overrides previous HTTP version option`.
  Indicio en offset derivado `0x1ff48`: `Usage: curl [options...] <url>`.

### cygcheck.exe

Diagnóstico de instalación/dependencias Cygwin.

- **Ruta:** `CygUtils.plugin!/bin/cygcheck.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `b630774512e0df09488a0049b12c188c8c10bec79eae8980ceb00c1fa9666045`. Hash derivado: `5224d621652d7fde44d45054d8da43ceb62c4c9c7e8dc86c80709ac2277cc684`; tamaño derivado: 133651 B; método `lzma-alone`.
  DLL importadas: `ADVAPI32.dll`, `KERNEL32.dll`, `msvcrt.dll`, `ntdll.dll`, `PSAPI.DLL`, `USER32.dll`, `WININET.DLL`.
  Indicio en offset derivado `0x16c44`: `Usage: cygcheck [-v] [-h] PROGRAM`.
  Indicio en offset derivado `0x16ee0`: `  -c, --check-setup    show installed version of PACKAGE and verify integrity`.

### cygpath.exe

Conversión entre rutas Unix y Windows.

- **Ruta:** `CygUtils.plugin!/bin/cygpath.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `5708c070170ed4ccec0716c59382f034037212859b0820de53bab8e97d10960c`. Hash derivado: `6a6f6970cf2842eceed691bebda74ea9f2bb021a5280ea0c67c6986e2109f357`; tamaño derivado: 25629 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`, `ntdll.dll`, `SHELL32.DLL`, `USERENV.dll`.
  Indicio en offset derivado `0x3660`: `Usage: %1$s (-d\|-m\|-u\|-w\|-t TYPE) [-f FILE] [OPTION]... NAME...`.
  Indicio en offset derivado `0x38fc`: `Path conversion options:`.

### cygstart.exe

Abrir aplicaciones/documentos mediante Windows.

- **Ruta:** `CygUtils.plugin!/bin/cygstart.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `20d3565b4cf40dbad2d0b4d3af8ef0180d80a94b19a7077f2ff02050491f4f6f`. Hash derivado: `a58f1415e0fc8560defca9e485b3c90b4d2f69a754adf40b7e565a9b1ba89d8b`; tamaño derivado: 16896 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cygpopt-0.dll`, `KERNEL32.dll`, `SHELL32.DLL`.
  Indicio en offset derivado `0x1e80`: `%s is part of cygutils version %s`.
  Indicio en offset derivado `0x1f10`: `%s: multibyte to wide string conversion error`.

### cygtermd.exe

Adaptación de terminal Cygwin; protocolo IPC pendiente.

- **Ruta:** `CygUtils.plugin!/bin/cygtermd.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `63f0344632b4938f8e745abe02240cee8de5075a640113b2dd8e11a9d698c4c0`. Hash derivado: `ae7ef4fb23c6600ca6947226188a597083c0e65bece1db2884debb330addc0c3`; tamaño derivado: 23566 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.
- **Ruta:** `CygUtils64.plugin!/bin/cygtermd.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `3e996e7bc5d6211383fc5381313354bf9a9425163438ee7c34b1f8d11aaecd02`. Hash derivado: `a3744867fdb96066cc4725ce45a22ab4a0cdc9f0b6bb9f9941848720263c3015`; tamaño derivado: 31336 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: ``, ``, ``, ``, ``, ``, ``, ``, ``, `�� `, ``, ``, ``, ``, ``, `Z�`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `�H�E�@,;E�~"H�EH�P0�E�H�H��`, ``, ``, ``, ``, ``, ``, ``, `H�@0�`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `nv`, `�`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `MZ�`, `�3`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `�KH�E�@,��~1H�EH�@0�`, ``, ``, `H���`, ``, ``, `�H���`, ``, ``, ``, ``, ``, ``, ``, `�@�U A����H�M�:`, ``, ``, ``, ``, ``, ``, `.bss`, ``, `�H�� `, ``, ``, ``, ``, ``, ``, ``, `�=`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `�H�E�@,;E�~"H�EH�P0�E�H�H��`, `M�� `, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `�`, `.rdata`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `PE`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `H��([^�f.�`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `MZ�`, `MZ�`.
  Indicio en offset derivado `0x65c6`: `Mobatek1`.
  Indicio en offset derivado `0x65d8`: `Mobatek0`.

### cygwin-console-helper.exe

Ayudante de consola de Cygwin.

- **Ruta:** `CygUtils.plugin!/bin/cygwin-console-helper.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `bc4bb512eda051459ed0efcd9d20b529cd29090c56898e5694f7924e962df277`. Hash derivado: `5c2b808ac61090378660551ededb1d7bd8279a4a3abf406c06894f5b86683164`; tamaño derivado: 47133 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `KERNEL32.dll`, `msvcrt.dll`.
  Indicio en offset derivado `0x7e40`: `  Unknown pseudo relocation protocol version %d.`.
- **Ruta:** `CygUtils64.plugin!/bin/cygwin-console-helper.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `adc0b2e60908acf4c08e7933056703c82b15584461df90346008193c3f80e58f`. Hash derivado: `c7c50ce2bf573c7f03de080e0f64d44f78b6d04590eed5bc18565dd5e68feb88`; tamaño derivado: 41491 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `KERNEL32.dll`, `msvcrt.dll`.
  Indicio en offset derivado `0x7460`: `  Unknown pseudo relocation protocol version %d.`.

### dwm.exe

Gestor de ventanas X de estilo dwm.

- **Ruta:** `CygUtils.plugin!/bin/dwm.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `123c6bfb9b10b92296fe560a0b25a9b535ca82b9f0a01bf46299f0a3783aea84`. Hash derivado: `d376299494c80048707cd0eab52c10fb9b40e9b522e3281a7ee4da522ddcdc4b`; tamaño derivado: 28672 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cygX11-6.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x54ee`: `usage: dwm [-v]`.

### dwm_w32.exe

Gestor de ventanas adaptado a Windows.

- **Ruta:** `XW!/bin/dwm_w32.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `d2de9fd7246d885e17643040fe2fbd6f96537791b35a53816c166b96f6560f57`. Hash derivado: `d2de9fd7246d885e17643040fe2fbd6f96537791b35a53816c166b96f6560f57`; tamaño derivado: 38400 B; método `direct`.
  **Integridad:** CRC32 declarado por ZIP no coincide; contenido retenido como observado no confiable. No se usa como prueba de integridad ni de comportamiento autorizado.
  DLL importadas: `KERNEL32.dll`, `msvcrt.dll`, `libX11-6.dll`.
  Indicio en offset derivado `0x6199`: `usage: dwm [-v]`.
  Indicio en offset derivado `0x66d8`: `  Unknown pseudo relocation protocol version %d.`.

### editrights.exe

Administración de derechos Windows.

- **Ruta:** `CygUtils.plugin!/bin/editrights.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `e99ba0327ff99c8e3a307db6aaa102de62b7372c9f9d8e7ab245cf4a3dd8765b`. Hash derivado: `cf876abe991e3e448b4a8a2340460876c21c46ff8b212462bd236339faa15788`; tamaño derivado: 68608 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `ADVAPI32.DLL`, `KERNEL32.dll`.
  Indicio en offset derivado `0x89f0`: `editrights version 1.01: a cygwin application to edit user rights`.
  Indicio en offset derivado `0x8ae4`: `Usage: editrights -u USER {-a\|-l\|-r\|-t} [options]`.

### extract_compressed_fs.exe

Extracción de sistema de archivos empaquetado; formato exacto pendiente.

- **Ruta:** `CygUtils.plugin!/bin/extract_compressed_fs.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `7c94cbf0b0af3a72a5d085487f1637acf25c2be8fb24522f04b09853cda2f6e6`. Hash derivado: `a553c9a9d9954f5294049b04b04f0e12708e3fec143a87ed22b1991c32b67e6d`; tamaño derivado: 10240 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cygz.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x1f58`: `  Unknown pseudo relocation protocol version %d.`.

### fdupes.exe

Identificación de archivos duplicados.

- **Ruta:** `CygUtils.plugin!/bin/fdupes.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `3ea57dafa977e5fcf50d1d000598db4c1f4cba755cdfd04b173a61892245fbd9`. Hash derivado: `735206211ed753777e5216da8b63801230318bf36661b92175b8198caacc9b53`; tamaño derivado: 17920 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x3620`: `Usage: fdupes [options] DIRECTORY...`.
  Indicio en offset derivado `0x3bc4`: ` -v --version     `.

### ftp.exe

Cliente FTP.

- **Ruta:** `CygUtils.plugin!/bin/ftp.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `99fa9f2bfd6d82d3d2053c589d41f4c7a3d785ee028d2f5185da9939d80fe84d`. Hash derivado: `f0e3e2905d64a07c0d7fd8e8c47ba93d88d69d87fb5485cdc034921b176d31d6`; tamaño derivado: 95758 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cygreadline7.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x14472`: `usage: %s [`.
  Indicio en offset derivado `0x14500`: `usage: %s remote-directory local-file`.

### fvwm.exe

Gestor de ventanas X FVWM.

- **Ruta:** `CygUtils.plugin!/lib/fvwm/2.4.7/fvwm.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `dd2cd9448f4a46bd6806ebd5d7f87325e0c4ae14347437354a0f2e14183de659`. Hash derivado: `223a75c3e23c4b89417d9f16de5844e44754374cf1efad7c7affec8e3f0fe8a3`; tamaño derivado: 376832 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygX11-6.dll`, `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x578e4`: `State file version (%s) does not match the current version (%s)`.
  Indicio en offset derivado `0x581ff`: `FVWM version %s Usage:`.

### FvwmBacker.exe

Módulo FVWM para fondo de escritorio.

- **Ruta:** `CygUtils.plugin!/lib/fvwm/2.4.7/FvwmBacker.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `5a0de90dd14c8d353919ba4cf330a9a0e57f2f67e857c127d3fb8d1207a4686b`. Hash derivado: `c26bf9606e5cd89cdff3932acb8d8b14dd85bd5d104d184dd24dd6b6e8a1b69a`; tamaño derivado: 38400 B; método `lzma-alone`.
  DLL importadas: `cygX11-6.dll`, `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x83d4`: `  Unknown pseudo relocation protocol version %d.`.

### FvwmTaskBar.exe

Módulo de barra de tareas FVWM.

- **Ruta:** `CygUtils.plugin!/lib/fvwm/2.4.7/FvwmTaskBar.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `6a8a15ba231c2a296e5173bf49e578bec83304e58cb8a34ab89721db0b3f89dd`. Hash derivado: `c1119cf7a276388d077441b917aef6d73204c369bfd93d3a018ead914fd06766`; tamaño derivado: 78848 B; método `lzma-alone`.
  DLL importadas: `cygX11-6.dll`, `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x11d68`: `  Unknown pseudo relocation protocol version %d.`.

### getfacl.exe

Lectura de ACL.

- **Ruta:** `CygUtils.plugin!/bin/getfacl.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `2f5fdce44718ad7749b10897ecae933a2b845772c116dac916abe0556fa38fed`. Hash derivado: `22fc216a773461c1bd839aa2dfee73250c60c06b359c06269ad182dae626b092`; tamaño derivado: 13341 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x1460`: `Usage: %s [-adn] FILE [FILE2...]`.
  Indicio en offset derivado `0x15a7`: `  -V, --version  output version information and exit`.

### git-add.exe

Entrada al subcomando Git `add`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-add.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-add.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-am.exe

Entrada al subcomando Git `am`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-am.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-am.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-annotate.exe

Entrada al subcomando Git `annotate`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-annotate.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-annotate.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-apply.exe

Entrada al subcomando Git `apply`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-apply.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-apply.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-archive.exe

Entrada al subcomando Git `archive`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-archive.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-archive.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-bisect--helper.exe

Entrada al subcomando Git `bisect--helper`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-bisect--helper.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-bisect--helper.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-blame.exe

Entrada al subcomando Git `blame`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-blame.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-blame.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-branch.exe

Entrada al subcomando Git `branch`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-branch.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-branch.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-bugreport.exe

Entrada al subcomando Git `bugreport`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-bugreport.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-bugreport.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-bundle.exe

Entrada al subcomando Git `bundle`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-bundle.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-bundle.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-cat-file.exe

Entrada al subcomando Git `cat-file`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-cat-file.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-cat-file.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-check-attr.exe

Entrada al subcomando Git `check-attr`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-check-attr.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-check-attr.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-check-ignore.exe

Entrada al subcomando Git `check-ignore`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-check-ignore.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-check-ignore.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-check-mailmap.exe

Entrada al subcomando Git `check-mailmap`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-check-mailmap.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-check-mailmap.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-check-ref-format.exe

Entrada al subcomando Git `check-ref-format`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-check-ref-format.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-check-ref-format.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-checkout--worker.exe

Entrada al subcomando Git `checkout--worker`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-checkout--worker.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-checkout--worker.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-checkout-index.exe

Entrada al subcomando Git `checkout-index`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-checkout-index.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-checkout-index.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-checkout.exe

Entrada al subcomando Git `checkout`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-checkout.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-checkout.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-cherry-pick.exe

Entrada al subcomando Git `cherry-pick`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-cherry-pick.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-cherry-pick.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-cherry.exe

Entrada al subcomando Git `cherry`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-cherry.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-cherry.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-clean.exe

Entrada al subcomando Git `clean`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-clean.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-clean.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-clone.exe

Entrada al subcomando Git `clone`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-clone.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-clone.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-column.exe

Entrada al subcomando Git `column`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-column.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-column.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-commit-graph.exe

Entrada al subcomando Git `commit-graph`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-commit-graph.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-commit-graph.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-commit-tree.exe

Entrada al subcomando Git `commit-tree`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-commit-tree.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-commit-tree.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-commit.exe

Entrada al subcomando Git `commit`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-commit.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-commit.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-config.exe

Entrada al subcomando Git `config`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-config.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-config.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-count-objects.exe

Entrada al subcomando Git `count-objects`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-count-objects.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-count-objects.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-credential-cache--daemon.exe

Entrada al subcomando Git `credential-cache--daemon`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-credential-cache--daemon.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-credential-cache--daemon.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-credential-cache.exe

Entrada al subcomando Git `credential-cache`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-credential-cache.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-credential-cache.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-credential-store.exe

Entrada al subcomando Git `credential-store`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-credential-store.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-credential-store.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-credential.exe

Entrada al subcomando Git `credential`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-credential.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-credential.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-describe.exe

Entrada al subcomando Git `describe`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-describe.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-describe.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-diff-files.exe

Entrada al subcomando Git `diff-files`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-diff-files.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-diff-files.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-diff-index.exe

Entrada al subcomando Git `diff-index`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-diff-index.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-diff-index.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-diff-tree.exe

Entrada al subcomando Git `diff-tree`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-diff-tree.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-diff-tree.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-diff.exe

Entrada al subcomando Git `diff`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-diff.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-diff.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-difftool.exe

Entrada al subcomando Git `difftool`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-difftool.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-difftool.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-env--helper.exe

Entrada al subcomando Git `env--helper`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-env--helper.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-env--helper.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-fast-export.exe

Entrada al subcomando Git `fast-export`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-fast-export.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-fast-export.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-fast-import.exe

Entrada al subcomando Git `fast-import`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-fast-import.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-fast-import.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-fetch-pack.exe

Entrada al subcomando Git `fetch-pack`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-fetch-pack.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-fetch-pack.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-fetch.exe

Entrada al subcomando Git `fetch`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-fetch.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-fetch.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-fmt-merge-msg.exe

Entrada al subcomando Git `fmt-merge-msg`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-fmt-merge-msg.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-fmt-merge-msg.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-for-each-ref.exe

Entrada al subcomando Git `for-each-ref`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-for-each-ref.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-for-each-ref.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-for-each-repo.exe

Entrada al subcomando Git `for-each-repo`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-for-each-repo.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-for-each-repo.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-format-patch.exe

Entrada al subcomando Git `format-patch`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-format-patch.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-format-patch.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-fsck-objects.exe

Entrada al subcomando Git `fsck-objects`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-fsck-objects.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-fsck-objects.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-fsck.exe

Entrada al subcomando Git `fsck`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-fsck.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-fsck.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-fsmonitor--daemon.exe

Entrada al subcomando Git `fsmonitor--daemon`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-fsmonitor--daemon.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-fsmonitor--daemon.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-gc.exe

Entrada al subcomando Git `gc`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-gc.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-gc.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-get-tar-commit-id.exe

Entrada al subcomando Git `get-tar-commit-id`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-get-tar-commit-id.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-get-tar-commit-id.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-grep.exe

Entrada al subcomando Git `grep`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-grep.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-grep.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-hash-object.exe

Entrada al subcomando Git `hash-object`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-hash-object.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-hash-object.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-help.exe

Entrada al subcomando Git `help`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-help.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-help.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-hook.exe

Entrada al subcomando Git `hook`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-hook.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-hook.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-index-pack.exe

Entrada al subcomando Git `index-pack`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-index-pack.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-index-pack.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-init-db.exe

Entrada al subcomando Git `init-db`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-init-db.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-init-db.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-init.exe

Entrada al subcomando Git `init`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-init.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-init.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-interpret-trailers.exe

Entrada al subcomando Git `interpret-trailers`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-interpret-trailers.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-interpret-trailers.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-log.exe

Entrada al subcomando Git `log`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-log.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-log.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-ls-files.exe

Entrada al subcomando Git `ls-files`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-ls-files.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-ls-files.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-ls-remote.exe

Entrada al subcomando Git `ls-remote`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-ls-remote.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-ls-remote.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-ls-tree.exe

Entrada al subcomando Git `ls-tree`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-ls-tree.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-ls-tree.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-mailinfo.exe

Entrada al subcomando Git `mailinfo`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-mailinfo.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-mailinfo.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-mailsplit.exe

Entrada al subcomando Git `mailsplit`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-mailsplit.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-mailsplit.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-maintenance.exe

Entrada al subcomando Git `maintenance`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-maintenance.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-maintenance.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-merge-base.exe

Entrada al subcomando Git `merge-base`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-merge-base.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-merge-base.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-merge-file.exe

Entrada al subcomando Git `merge-file`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-merge-file.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-merge-file.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-merge-index.exe

Entrada al subcomando Git `merge-index`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-merge-index.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-merge-index.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-merge-ours.exe

Entrada al subcomando Git `merge-ours`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-merge-ours.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-merge-ours.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-merge-recursive.exe

Entrada al subcomando Git `merge-recursive`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-merge-recursive.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-merge-recursive.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-merge-subtree.exe

Entrada al subcomando Git `merge-subtree`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-merge-subtree.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-merge-subtree.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-merge-tree.exe

Entrada al subcomando Git `merge-tree`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-merge-tree.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-merge-tree.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-merge.exe

Entrada al subcomando Git `merge`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-merge.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-merge.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-mktag.exe

Entrada al subcomando Git `mktag`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-mktag.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-mktag.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-mktree.exe

Entrada al subcomando Git `mktree`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-mktree.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-mktree.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-multi-pack-index.exe

Entrada al subcomando Git `multi-pack-index`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-multi-pack-index.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-multi-pack-index.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-mv.exe

Entrada al subcomando Git `mv`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-mv.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-mv.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-name-rev.exe

Entrada al subcomando Git `name-rev`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-name-rev.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-name-rev.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-notes.exe

Entrada al subcomando Git `notes`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-notes.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-notes.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-pack-objects.exe

Entrada al subcomando Git `pack-objects`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-pack-objects.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-pack-objects.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-pack-redundant.exe

Entrada al subcomando Git `pack-redundant`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-pack-redundant.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-pack-redundant.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-pack-refs.exe

Entrada al subcomando Git `pack-refs`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-pack-refs.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-pack-refs.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-patch-id.exe

Entrada al subcomando Git `patch-id`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-patch-id.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-patch-id.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-prune-packed.exe

Entrada al subcomando Git `prune-packed`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-prune-packed.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-prune-packed.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-prune.exe

Entrada al subcomando Git `prune`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-prune.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-prune.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-pull.exe

Entrada al subcomando Git `pull`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-pull.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-pull.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-push.exe

Entrada al subcomando Git `push`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-push.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-push.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-range-diff.exe

Entrada al subcomando Git `range-diff`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-range-diff.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-range-diff.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-read-tree.exe

Entrada al subcomando Git `read-tree`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-read-tree.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-read-tree.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-rebase.exe

Entrada al subcomando Git `rebase`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-rebase.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-rebase.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-receive-pack.exe

Entrada al subcomando Git `receive-pack`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/bin/git-receive-pack.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `ebd541f201869576eb66accd249a7e1b99e85e18fa220fb8ad1cf5de412a59d4`. Hash derivado: `f8b0cc4efb6663615eb5fc4a359752df3f01643dec68fd6598d39df5bd7370c2`; tamaño derivado: 28 B; método `lzma-alone`.
  Destino declarado del enlace: `git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-receive-pack.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-receive-pack.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/bin/git-receive-pack.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `ebd541f201869576eb66accd249a7e1b99e85e18fa220fb8ad1cf5de412a59d4`. Hash derivado: `f8b0cc4efb6663615eb5fc4a359752df3f01643dec68fd6598d39df5bd7370c2`; tamaño derivado: 28 B; método `lzma-alone`.
  Destino declarado del enlace: `git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-reflog.exe

Entrada al subcomando Git `reflog`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-reflog.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-reflog.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-remote-ext.exe

Entrada al subcomando Git `remote-ext`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-remote-ext.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-remote-ext.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-remote-fd.exe

Entrada al subcomando Git `remote-fd`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-remote-fd.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-remote-fd.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-remote-ftp.exe

Entrada al subcomando Git `remote-ftp`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-remote-ftp.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `d2e51ecdc04ad8f055ccc7dbff6dd56e5df2942b10153af713cafb5cd601a586`. Hash derivado: `d4001c2bc7f4331951e811070fe359655e8b65a220c829e4779a38299a143936`; tamaño derivado: 52 B; método `lzma-alone`.
  Destino declarado del enlace: `git-remote-http.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-remote-ftp.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `d2e51ecdc04ad8f055ccc7dbff6dd56e5df2942b10153af713cafb5cd601a586`. Hash derivado: `d4001c2bc7f4331951e811070fe359655e8b65a220c829e4779a38299a143936`; tamaño derivado: 52 B; método `lzma-alone`.
  Destino declarado del enlace: `git-remote-http.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-remote-ftps.exe

Entrada al subcomando Git `remote-ftps`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-remote-ftps.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `d2e51ecdc04ad8f055ccc7dbff6dd56e5df2942b10153af713cafb5cd601a586`. Hash derivado: `d4001c2bc7f4331951e811070fe359655e8b65a220c829e4779a38299a143936`; tamaño derivado: 52 B; método `lzma-alone`.
  Destino declarado del enlace: `git-remote-http.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-remote-ftps.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `d2e51ecdc04ad8f055ccc7dbff6dd56e5df2942b10153af713cafb5cd601a586`. Hash derivado: `d4001c2bc7f4331951e811070fe359655e8b65a220c829e4779a38299a143936`; tamaño derivado: 52 B; método `lzma-alone`.
  Destino declarado del enlace: `git-remote-http.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-remote-https.exe

Entrada al subcomando Git `remote-https`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-remote-https.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `d2e51ecdc04ad8f055ccc7dbff6dd56e5df2942b10153af713cafb5cd601a586`. Hash derivado: `d4001c2bc7f4331951e811070fe359655e8b65a220c829e4779a38299a143936`; tamaño derivado: 52 B; método `lzma-alone`.
  Destino declarado del enlace: `git-remote-http.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-remote-https.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `d2e51ecdc04ad8f055ccc7dbff6dd56e5df2942b10153af713cafb5cd601a586`. Hash derivado: `d4001c2bc7f4331951e811070fe359655e8b65a220c829e4779a38299a143936`; tamaño derivado: 52 B; método `lzma-alone`.
  Destino declarado del enlace: `git-remote-http.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-remote.exe

Entrada al subcomando Git `remote`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-remote.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-remote.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-repack.exe

Entrada al subcomando Git `repack`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-repack.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-repack.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-replace.exe

Entrada al subcomando Git `replace`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-replace.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-replace.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-rerere.exe

Entrada al subcomando Git `rerere`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-rerere.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-rerere.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-reset.exe

Entrada al subcomando Git `reset`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-reset.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-reset.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-restore.exe

Entrada al subcomando Git `restore`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-restore.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-restore.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-rev-list.exe

Entrada al subcomando Git `rev-list`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-rev-list.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-rev-list.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-rev-parse.exe

Entrada al subcomando Git `rev-parse`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-rev-parse.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-rev-parse.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-revert.exe

Entrada al subcomando Git `revert`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-revert.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-revert.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-rm.exe

Entrada al subcomando Git `rm`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-rm.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-rm.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-send-pack.exe

Entrada al subcomando Git `send-pack`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-send-pack.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-send-pack.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-shortlog.exe

Entrada al subcomando Git `shortlog`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-shortlog.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-shortlog.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-show-branch.exe

Entrada al subcomando Git `show-branch`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-show-branch.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-show-branch.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-show-index.exe

Entrada al subcomando Git `show-index`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-show-index.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-show-index.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-show-ref.exe

Entrada al subcomando Git `show-ref`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-show-ref.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-show-ref.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-show.exe

Entrada al subcomando Git `show`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-show.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-show.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-sparse-checkout.exe

Entrada al subcomando Git `sparse-checkout`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-sparse-checkout.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-sparse-checkout.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-stage.exe

Entrada al subcomando Git `stage`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-stage.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-stage.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-stash.exe

Entrada al subcomando Git `stash`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-stash.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-stash.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-status.exe

Entrada al subcomando Git `status`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-status.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-status.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-stripspace.exe

Entrada al subcomando Git `stripspace`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-stripspace.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-stripspace.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-submodule--helper.exe

Entrada al subcomando Git `submodule--helper`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-submodule--helper.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-submodule--helper.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-switch.exe

Entrada al subcomando Git `switch`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-switch.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-switch.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-symbolic-ref.exe

Entrada al subcomando Git `symbolic-ref`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-symbolic-ref.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-symbolic-ref.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-tag.exe

Entrada al subcomando Git `tag`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-tag.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-tag.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-unpack-file.exe

Entrada al subcomando Git `unpack-file`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-unpack-file.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-unpack-file.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-unpack-objects.exe

Entrada al subcomando Git `unpack-objects`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-unpack-objects.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-unpack-objects.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-update-index.exe

Entrada al subcomando Git `update-index`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-update-index.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-update-index.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-update-ref.exe

Entrada al subcomando Git `update-ref`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-update-ref.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-update-ref.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-update-server-info.exe

Entrada al subcomando Git `update-server-info`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-update-server-info.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-update-server-info.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-upload-archive.exe

Entrada al subcomando Git `upload-archive`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/bin/git-upload-archive.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `ebd541f201869576eb66accd249a7e1b99e85e18fa220fb8ad1cf5de412a59d4`. Hash derivado: `f8b0cc4efb6663615eb5fc4a359752df3f01643dec68fd6598d39df5bd7370c2`; tamaño derivado: 28 B; método `lzma-alone`.
  Destino declarado del enlace: `git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-upload-archive.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-upload-archive.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/bin/git-upload-archive.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `ebd541f201869576eb66accd249a7e1b99e85e18fa220fb8ad1cf5de412a59d4`. Hash derivado: `f8b0cc4efb6663615eb5fc4a359752df3f01643dec68fd6598d39df5bd7370c2`; tamaño derivado: 28 B; método `lzma-alone`.
  Destino declarado del enlace: `git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-upload-pack.exe

Entrada al subcomando Git `upload-pack`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/bin/git-upload-pack.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `ebd541f201869576eb66accd249a7e1b99e85e18fa220fb8ad1cf5de412a59d4`. Hash derivado: `f8b0cc4efb6663615eb5fc4a359752df3f01643dec68fd6598d39df5bd7370c2`; tamaño derivado: 28 B; método `lzma-alone`.
  Destino declarado del enlace: `git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-upload-pack.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-upload-pack.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/bin/git-upload-pack.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `ebd541f201869576eb66accd249a7e1b99e85e18fa220fb8ad1cf5de412a59d4`. Hash derivado: `f8b0cc4efb6663615eb5fc4a359752df3f01643dec68fd6598d39df5bd7370c2`; tamaño derivado: 28 B; método `lzma-alone`.
  Destino declarado del enlace: `git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-var.exe

Entrada al subcomando Git `var`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-var.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-var.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-verify-commit.exe

Entrada al subcomando Git `verify-commit`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-verify-commit.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-verify-commit.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-verify-pack.exe

Entrada al subcomando Git `verify-pack`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-verify-pack.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-verify-pack.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-verify-tag.exe

Entrada al subcomando Git `verify-tag`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-verify-tag.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-verify-tag.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-whatchanged.exe

Entrada al subcomando Git `whatchanged`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-whatchanged.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-whatchanged.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-worktree.exe

Entrada al subcomando Git `worktree`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-worktree.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-worktree.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git-write-tree.exe

Entrada al subcomando Git `write-tree`; reutilizar Git Linux; semántica concreta requiere contrato del subcomando.

- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git-write-tree.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git-write-tree.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.

### git.exe

Sistema de control de versiones Git y dispatcher de subcomandos.

- **Ruta:** `CygUtils.plugin!/bin/git.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `527ec57599d8683e1254cd82fd58147718f666d8876a7ad8b028d738ab3cd729`. Hash derivado: `c2a0901bfe9bd7793262861a4520fe1b65c585b5478cb978f1ad5156e80e8694`; tamaño derivado: 4108302 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `cygiconv-2.dll`, `cygz.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x2dab58`: `usage: %s`.
  Indicio en offset derivado `0x2e04ca`: `git version:`.
- **Ruta:** `CygUtils.plugin!/usr/git/git-core/git.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/usr/git/git-core/git.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `a838d9909583ec6b81e6a61b1f6eee41fc86ad6d76178c30ad2e9394e438c87d`. Hash derivado: `3e6ecda77250329d1530349fafe7d98e8e8cfbb3f68020fb8be1b730778d03c8`; tamaño derivado: 38 B; método `lzma-alone`.
  Destino declarado del enlace: `/bin/git.exe`; seguir su resolución en el entorno antes de asumir que existe.
- **Ruta:** `CygUtils64.plugin!/bin/git.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `995e4b9bfcb52482b9473519c3e71bf7ac79620bfa3fe444f151eef2b27f5077`. Hash derivado: `8bc19fc48efa8bc55ad910b8de58e9ff2899a2f8cd8c288689404d8c526217d0`; tamaño derivado: 4410880 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `cygiconv-2.dll`, `cygz.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x356534`: `usage: %s`.
  Indicio en offset derivado `0x35c0ce`: `git version:`.

### glxgears.exe

Demostración/diagnóstico de renderizado GLX.

- **Ruta:** `CygUtils.plugin!/bin/glxgears.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `da12e4f3820ec361d9e2e884a249f14152e32901120908d0e39f824e527e95f7`. Hash derivado: `55e705b8647429405a6534fe421c061260962fb9b5c111b2a95c948fbea9161e`; tamaño derivado: 19968 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cygGL-1.dll`, `cygX11-6.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x2f0d`: `Usage:`.

### glxinfo.exe

Información de GLX/OpenGL.

- **Ruta:** `CygUtils.plugin!/bin/glxinfo.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `55b8f396d50ea607b6b26f1163947c64dfa0d42f8739cec5bb3847b646507860`. Hash derivado: `5429ce81180ab00242ec9c3d06624493d2560f745f025b078816920c30bb8bc9`; tamaño derivado: 37405 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cygGL-1.dll`, `cygX11-6.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x434c`: `Usage: glxinfo [-v] [-t] [-h] [-i] [-b] [-s] [-display <dname>]`.
  Indicio en offset derivado `0x4c00`: `server glx version string: %s`.

### greed.exe

Juego de terminal.

- **Ruta:** `CygUtils.plugin!/bin/greed.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `db100bd27a925cfb479517c3256f2a5c38747dfd788ed35b791bf24dffbd86d8`. Hash derivado: `6bd41af503c2acf433d5c878c3679a07f6376440f1d19e32c08162c8b6275d0d`; tamaño derivado: 15872 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cygncurses-8.dll`, `KERNEL32.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`.
  Indicio en offset derivado `0x281b`: `Usage: %s [-p] [-s]`.
- **Ruta:** `CygUtils64.plugin!/bin/greed.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `07db510e23fc136d3dea2911c982c99d9f5a0a7909d14896981963824d7509ca`. Hash derivado: `471c176d92e162ef4df67a441a352d23b3dcc8c7084263526f3ead3b1204ce8f`; tamaño derivado: 84398 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygncursesw-10.dll`, `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x2c30`: `Usage: %s [-p] [-s]`.

### grep.exe

Selección de líneas por patrones y expresiones regulares.

- **Ruta:** `CygUtils.plugin!/bin/grep.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `fc9224966a588652dab94be411c8e78ef3575b2daa2f8d7c80630c3ce07db471`. Hash derivado: `e319cce6b01f6a5bfd17e299882ead75a906b64c9a2dbc7e26f470da61f4a9b6`; tamaño derivado: 164864 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: ``, ``, ``, ``, ``, ``, ``, ``, ``, `��`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `MZ�`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `E�wF�M�������M�1��E�1��U�1�	�������E�9E������9����������`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `B`, ``, ``, ``, ``, ``, ``, ``, ``, ``, `]�f�U������t�$��`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `�\|$�$��`, `�p�����U���1����`, `K `, ``, ``, ``, `�C��`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `�`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `�E��`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `'`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `K `, ``, ``, ``, ``, ``, `MZ�`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `MZ�`, ``, ``, `�`, ``, ``, ``, ``, ``, ``, `���0�`, ``, ``, ``, ``, ``, ``, ``, ``, ``, `���)��	`, `@ut�E�����`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `�`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `��������0�`, ``, `�F�`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `MZ�`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `��)����`�B`, ``, ``, ``, ``, `���`, ``, ``, ``, ``, `B`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `Z�`, ``, ``, ``, ``, ``, ``, ``, `���]����}���t �$�`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `��B`, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, ``, `�����F��<�����G;�H�����`, ``, ``, ``, `�`, `�`, `��v���`, ``, ``, ``, ``, ``, ``, ``, ``, ``.
  Indicio en offset derivado `0x2265c`: `Usage: %s [OPTION]... PATTERN [FILE]...`.
  Indicio en offset derivado `0x22a92`: `  -V, --version             print version information and exit`.
- **Ruta:** `CygUtils64.plugin!/bin/grep.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `ab7bf60eb47d2989d6d8294d275a348f481112dc1faff89471773ccc1062d2b8`. Hash derivado: `aa2cfdcb4476f265e1ee2242ce315e59110d705da495e70c37fe7d4d92f5444f`; tamaño derivado: 57344 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `cygz.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0xb030`: `usage: %s [-abcDEFGHhIiJLlmnOoPqRSsUVvwxZz] [-A num] [-B num] [-C[num]]`.

### httping.exe

Sondeo de latencia HTTP.

- **Ruta:** `CygUtils.plugin!/bin/httping.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `ecd42b25d0ff078758d360a3c104f141ce2d4fc3216143540517dd525a5e7f50`. Hash derivado: `c63af8c4478f4dd2f623e4a3091dbb239872e42353ee62b50fd262a69dfdf121`; tamaño derivado: 88590 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygcrypto-3.dll`, `cygwin1.dll`, `cygintl-8.dll`, `cygncursesw-10.dll`, `cygssl-3.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0xff28`: `socks5connect: reply with requested authentication method does not say version 5 (%02x)`.
  Indicio en offset derivado `0x10134`: `socks5connect: bind request replies with version other than 0x05 (%02x)`.

### iconv.exe

Conversión de codificaciones.

- **Ruta:** `CygUtils.plugin!/bin/iconv.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `2cc0e268215d8e9574a43a1cab2e6278f841830b6e4bcc36e7cc191bf76cc79c`. Hash derivado: `661e288d208905b2022e826763ccfde9c6cc2310275fcb1a26e88bdbb7ab0095`; tamaño derivado: 25600 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cygiconv-2.dll`, `cygintl-8.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x39bc`: `%s argument: The character '%c' is not a valid conversion specifier.`.
  Indicio en offset derivado `0x3a04`: `%s argument: The character that terminates the format directive is not a valid conversion specifier.`.

### iperf.exe

Medición de rendimiento de red con protocolo iperf.

- **Ruta:** `CygUtils.plugin!/bin/iperf.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `9f6c31296793370d79535c1c1b33bb7cab89fb730c5413822e563ea261ecb714`. Hash derivado: `4b0a812d93dd274aba73761305e4c6b21a90d61a98c8582204e05628d495ba91`; tamaño derivado: 71168 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0xe440`: `iperf version 2.0.4 (7 Apr 2008) pthreads`.
  Indicio en offset derivado `0xe90b`: `  -v, --version            print version information and quit`.
- **Ruta:** `CygUtils64.plugin!/bin/iperf.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `522530ea6b7d27e5eb3e823f5988fea7ad6ead15163a10cad61afa32d6494f78`. Hash derivado: `cc4d8fa62710be2d806468b7cf57327185a874ebaacd649b834c1bf0995049e9`; tamaño derivado: 64019 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `cyggcc_s-seh-1.dll`, `cygstdc++-6.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x9f00`: `iperf version 2.0.5 (08 Jul 2010) pthreads`.
  Indicio en offset derivado `0xa3cb`: `  -v, --version            print version information and quit`.

### iperf3.exe

Medición de rendimiento de red con protocolo iperf3.

- **Ruta:** `CygUtils.plugin!/bin/iperf3.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `bc8b40ffd84ad2e7db5f889e920735943a4761830d8a7bd4af91ca3b09dd88b0`. Hash derivado: `8526f487a0b33f3390329fb3d8bf7ae9038b0c55ffb869397f2838cb832aa617`; tamaño derivado: 104462 B; método `lzma-alone`.
  DLL importadas: `cyggcc_s-1.dll`, `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x13c60`: `Usage: iperf [-s\|-c host] [options]`.
  Indicio en offset derivado `0x13c84`: `       iperf [-h\|--help] [-v\|--version]`.

### jwm.exe

Gestor de ventanas X JWM.

- **Ruta:** `XW!/bin/jwm.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `2431ca1c8924d6d0f791669cda6a8d0d4bc3c7a9778e662a2004f385b1d0c7e3`. Hash derivado: `2431ca1c8924d6d0f791669cda6a8d0d4bc3c7a9778e662a2004f385b1d0c7e3`; tamaño derivado: 220686 B; método `direct`.
  **Integridad:** CRC32 declarado por ZIP no coincide; contenido retenido como observado no confiable. No se usa como prueba de integridad ni de comportamiento autorizado.
  DLL importadas: `KERNEL32.dll`, `msvcrt.dll`, `USER32.dll`, `libX11-6.dll`.
  Indicio en offset derivado `0x26014`: `usage: jwm [ options ]`.
  Indicio en offset derivado `0x26185`: `  -v          Display version information`.

### kill.exe

Envío de señales a procesos.

- **Ruta:** `CygUtils.plugin!/bin/kill.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `975e3dbdf93f65ea3367f9f00a18a2eb328a48475a72c84ec7b79b9528b48def`. Hash derivado: `093e1ad002421259ef2b924ac3884031ae3decc09c8ed9bfb9d3bea017d0fe2b`; tamaño derivado: 13341 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `ADVAPI32.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x1674`: `Usage: %1$s [-f] [-signal] [-s signal] pid1 [pid2 ...]`.
  Indicio en offset derivado `0x17b5`: ` -V, --version   output version information and exit`.

### kinit.exe

Obtención de tickets Kerberos.

- **Ruta:** `CygUtils.plugin!/bin/kinit.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `81c5e99ff72a5623c26a25e03c95d877020fd52561bbcad7822c6b85d8c6cb14`. Hash derivado: `4b9fad61803744e8d773b50c03a07fe93bf66d63f1af6e7ef9d98cdfed58138e`; tamaño derivado: 25629 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygkadm5srv_mit-11.dll`, `cygkdb5-8.dll`, `cygkrb5-3.dll`, `cygcom_err-2.dll`, `cygwin1.dll`, `cygintl-8.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x36b0`: `Usage: %s [-V] [-l lifetime] [-s start_time] `.
- **Ruta:** `CygUtils64.plugin!/bin/kinit.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `9c379f79e9b026bfc8b5bab2143905562f89e3b07627435f597df8a380212615`. Hash derivado: `cf3f8f6c2b939550d687f17763c1afd4a00f3f6d88f1843f9446b2b745cf5c57`; tamaño derivado: 24576 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygcom_err-3.dll`, `cygkadm5srv_mit-11.dll`, `cygkdb5-8.dll`, `cygkrb5-3.dll`, `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x34c8`: `Usage: %s [-V] [-l lifetime] [-s start_time] `.

### klist.exe

Consulta de tickets Kerberos.

- **Ruta:** `CygUtils.plugin!/bin/klist.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `ade337b7489997f0e9e6bbe618e1f0eb05558bea8e8fd49134bdbe3689290053`. Hash derivado: `b9ee2fff97a6fcfd09c085c842f647160529ba154c2978d06c83d4b5959865c8`; tamaño derivado: 24093 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygk5crypto-3.dll`, `cygkrb5-3.dll`, `cygkrb5support-0.dll`, `cygcom_err-2.dll`, `cygwin1.dll`, `cygintl-8.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x3260`: `Usage: %s [-e] [-V] [[-c] [-l] [-A] [-d] [-f] [-s] [-a [-n]]] [-k [-t] [-K]] [name]`.
  Indicio en offset derivado `0x33c9`: `-V shows the Kerberos version and exits`.
- **Ruta:** `CygUtils64.plugin!/bin/klist.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `8b9f0ba838e23eec5dd91c0ed94dd8b2b2efeb2dac4126fa99e930b88b06744a`. Hash derivado: `65c06487707b0538651a94977937fcfc0d8f3075d26ef4227f0b8a2c5f893770`; tamaño derivado: 20992 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygcom_err-3.dll`, `cygk5crypto-3.dll`, `cygkrb5-3.dll`, `cygkrb5support-0.dll`, `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x2e30`: `Usage: %s [-e] [-V] [[-c] [-l] [-A] [-d] [-f] [-s] [-a [-n]]] [-k [-t] [-K]] [name]`.
  Indicio en offset derivado `0x2fa1`: `-V shows the Kerberos version and exits`.

### kpasswd.exe

Cambio de contraseña Kerberos.

- **Ruta:** `CygUtils.plugin!/bin/kpasswd.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `65b18c19b1f3f488e3c95943aa8675215740f73654c33b6205cdbcc7259bb5f6`. Hash derivado: `82466567cd01c0250a140d777c347ffe29555e36d9c7271eae883e4b6d6cb2e2`; tamaño derivado: 12829 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygkrb5-3.dll`, `cygcom_err-2.dll`, `cygwin1.dll`, `cygintl-8.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x1664`: `usage: %s [principal]`.

### ldd.exe

Consulta de dependencias dinámicas.

- **Ruta:** `CygUtils.plugin!/bin/ldd.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `e44c696477564eb22599ca3f2f800d9d02c20d15184d7f468707d3acd25cee6d`. Hash derivado: `8ef1b342ecf5c50e209a98706bccfa18e4bea1b9e9f47bc3814ab7535878194e`; tamaño derivado: 16925 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`, `PSAPI.DLL`.
  Indicio en offset derivado `0x2310`: `Usage: %s [OPTION]... FILE...`.
  Indicio en offset derivado `0x2385`: `  -V, --version           print version information and exit`.

### ldh.exe

Auxiliar de carga; función no establecida.

- **Ruta:** `CygUtils.plugin!/bin/ldh.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `14d5a22fb3e6c7bcb39e46ec0e4ba875c18b6a86baac032c6ca19865156ca988`. Hash derivado: `495253c868072d26d21ef6ec3625afc0ce58c2e4db21b54a39cc60c29fa10041`; tamaño derivado: 3091 B; método `lzma-alone`.
  DLL importadas: `KERNEL32.dll`.

### less.exe

Paginador de texto.

- **Ruta:** `CygUtils64.plugin!/bin/less.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `44cab2c3864738650937a0160189d04bab5343d356abc053c2ae991d072fca70`. Hash derivado: `276f3006d4537ead9858e0b18008a93ce854d5e8320008c1321055e9a89a5ccc`; tamaño derivado: 140288 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `cygncursesw-10.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x1af4c`: `  V                    Print version number of "less".`.
  Indicio en offset derivado `0x1bd36`: `                  Display the version number of "less".`.

### lrz.exe

Recepción X/Y/ZMODEM.

- **Ruta:** `CygUtils.plugin!/bin/lrz.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `6400bcca69018601afad9d1548101c1d15490c64b1e48c454eb4af3f9b30d749`. Hash derivado: `a35c34992fd696ef332fc4f5de36ad581b4fbee238dcb980974bbaa36670c63d`; tamaño derivado: 64014 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0xb0bc`: `%s version %s`.
  Indicio en offset derivado `0xb0cc`: `Usage: %s [options] [filename.if.xmodem]`.
- **Ruta:** `CygUtils64.plugin!/bin/lrz.exe`; tipo `mz-unparsed`; arquitectura `no establecida`.
  Hash original: `3fb71bf14e862397f30d8f242f8ccc27999d99724cdd6f646ff75588bf621c1f`. Hash derivado: `be1f2931392714a4c40c4cc243327c8fa7832f602f87e44cad60373557a079ce`; tamaño derivado: 57344 B; método `lzma-derived-header-byte1-zero`.

### lsz.exe

Envío X/Y/ZMODEM.

- **Ruta:** `CygUtils.plugin!/bin/lsz.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `08f02a7d967c965c977fe8ee720a932b75b8f542af480fe885230f6ea4e790b5`. Hash derivado: `84e39e7d4118041487671975a28e2736f274bc69fd91ddbabf009f1eaa0fd326`; tamaño derivado: 68110 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0xba30`: `%s version %s`.
  Indicio en offset derivado `0xba40`: `Usage: %s [options] file ...`.
- **Ruta:** `CygUtils64.plugin!/bin/lsz.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `77e16acd0f3c19c4c1bb1396bc4e6f708dcf19482aaaacb60a6c90fdb020ff3f`. Hash derivado: `8ad71d4f0ea7e76fcd0b2f988655ca2f35a4270ca1df8c8d7d15280cf2997051`; tamaño derivado: 61440 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0xa200`: `%s version %s`.
  Indicio en offset derivado `0xa210`: `Usage: %s [options] file ...`.

### m32termd.exe

Puente de terminal de 32 bits; IPC pendiente.

- **Ruta:** `MOBASE!/bin/exttermd/m32termd.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `1881298d5c104a2c0fdbf95dcd3ffd61b936da19c9b18a8814c5f7e820d3030d`. Hash derivado: `1881298d5c104a2c0fdbf95dcd3ffd61b936da19c9b18a8814c5f7e820d3030d`; tamaño derivado: 31944 B; método `direct`.
  **Integridad:** CRC32 declarado por ZIP no coincide; contenido retenido como observado no confiable. No se usa como prueba de integridad ni de comportamiento autorizado.
  DLL importadas: `msys-2.0.dll`, `msys-gcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x5c35`: `Mobatek1`.
  Indicio en offset derivado `0x5c47`: `Mobatek0`.

### m64termd.exe

Puente de terminal de 64 bits; IPC pendiente.

- **Ruta:** `MOBASE!/bin/exttermd/m64termd.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `45ae891a3b0eb17dfd55816da98e155773ac9d34fdf9d646b44cfe3cd8ce334c`. Hash derivado: `45ae891a3b0eb17dfd55816da98e155773ac9d34fdf9d646b44cfe3cd8ce334c`; tamaño derivado: 34488 B; método `direct`.
  **Integridad:** CRC32 declarado por ZIP no coincide; contenido retenido como observado no confiable. No se usa como prueba de integridad ni de comportamiento autorizado.
  DLL importadas: `msys-2.0.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x6625`: `Mobatek1`.
  Indicio en offset derivado `0x6637`: `Mobatek0`.

### mathomatic.exe

Álgebra simbólica en terminal.

- **Ruta:** `CygUtils.plugin!/bin/mathomatic.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `fc75e329fb465aa06f160d3e310bd72fdba43903aa4d9074be3a645d9b648900`. Hash derivado: `94886b683079c10d6912f00feaa46748cc187718c4dba3a3489446d35cfd0139`; tamaño derivado: 198158 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x28fe1`: `Mathomatic computer algebra system version %s`.
  Indicio en offset derivado `0x29014`: `Usage: %s [ options ] [ input_files ]`.

### minicom.exe

Terminal serie y transferencia.

- **Ruta:** `CygUtils.plugin!/bin/minicom.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `39f0362a7640742451e4ba575fd0ed2aeb7ec4da5409e8a8e75215cbe301a383`. Hash derivado: `db0594bed05a06eac769bd1c44afbff21587f274ee975db2a5aa2690b012c3fa`; tamaño derivado: 158720 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `cygiconv-2.dll`, `cygncurses-9.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x20194`: `%s version %s`.
  Indicio en offset derivado `0x202dc`: `Usage: %s [OPTION]... [configuration]`.

### mkgroup.exe

Generación de mapeos de grupos para Cygwin.

- **Ruta:** `CygUtils.plugin!/bin/mkgroup.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `b77d0820dbc213372809dac64215727ffb9b667ada44a0c2da5f0e66f7cb30ea`. Hash derivado: `039e078e4bc4c7726273daf97864a30b5fa6950785fcca8fc04c8a7044f55b78`; tamaño derivado: 21533 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `ADVAPI32.dll`, `KERNEL32.dll`, `NETAPI32.dll`.
  Indicio en offset derivado `0x2df4`: `Usage: %s [OPTION]...`.
  Indicio en offset derivado `0x3426`: `   -v,--version            Print version information and exit.`.

### mkpasswd.exe

Generación de mapeos de usuarios para Cygwin.

- **Ruta:** `CygUtils.plugin!/bin/mkpasswd.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `7fc82852a485f9167a594a644f8acfc23c72b6b3dd551c40e56f6f7c134663f4`. Hash derivado: `96bfa829aa69c37bc52419d85504b5c8ba352d85655c3b2b678bcfa6b3839239`; tamaño derivado: 21533 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `ADVAPI32.dll`, `KERNEL32.dll`, `NETAPI32.dll`.
  Indicio en offset derivado `0x2c2c`: `Usage: %s [OPTIONS]...`.
  Indicio en offset derivado `0x32da`: `   -V,--version            Version information and exit.`.

### mkshortcut.exe

Creación de accesos directos Windows.

- **Ruta:** `CygUtils.plugin!/bin/mkshortcut.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `f84549be6d5bc48d7ab84f56304eacce6485d0942122b8ea05eaaf62ad1ccd5e`. Hash derivado: `46a6db420e05a837630aae61528dfb78add550423e6b238df8674782d7665506`; tamaño derivado: 16398 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cygpopt-0.dll`, `KERNEL32.dll`, `OLE32.dll`, `SHELL32.DLL`.
  Indicio en offset derivado `0x2480`: `%s is part of cygutils version %s`.
  Indicio en offset derivado `0x282a`: `the Free Software Foundation, either version 3 of the License, or`.

### MobaBox.exe

Ejecutable multicall de utilidades de la suite.

- **Ruta:** `CygUtils.plugin!/bin/MobaBox.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `89da76f00c0d8264f65520483552e3279ba6e502cff6b4ce59e719b2c48ae679`. Hash derivado: `44483bf60fe11539505db5113f2dfcf7531559d7c46f141b5665248ffe586121`; tamaño derivado: 652080 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `kernel32.dll`, `user32.dll`, `advapi32.dll`, `oleaut32.dll`, `kernel32.dll`, `advapi32.dll`, `kernel32.dll`, `version.dll`, `gdi32.dll`, `user32.dll`, `kernel32.dll`, `oleaut32.dll`, `ole32.dll`, `oleaut32.dll`, `wsock32.dll`, `comctl32.dll`, `shell32.dll`, `netapi32.dll`, `wininet.dll`, `IPHLPAPI.DLL`.
  Indicio en offset derivado `0x589c4`: `SetReqVerLow: WinSock version can't be changed now`.
  Indicio en offset derivado `0x58a24`: `SetReqVerHigh: WinSock version can't be changed now`.
- **Ruta:** `CygUtils64.plugin!/bin/MobaBox.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `89da76f00c0d8264f65520483552e3279ba6e502cff6b4ce59e719b2c48ae679`. Hash derivado: `44483bf60fe11539505db5113f2dfcf7531559d7c46f141b5665248ffe586121`; tamaño derivado: 652080 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `kernel32.dll`, `user32.dll`, `advapi32.dll`, `oleaut32.dll`, `kernel32.dll`, `advapi32.dll`, `kernel32.dll`, `version.dll`, `gdi32.dll`, `user32.dll`, `kernel32.dll`, `oleaut32.dll`, `ole32.dll`, `oleaut32.dll`, `wsock32.dll`, `comctl32.dll`, `shell32.dll`, `netapi32.dll`, `wininet.dll`, `IPHLPAPI.DLL`.
  Indicio en offset derivado `0x589c4`: `SetReqVerLow: WinSock version can't be changed now`.
  Indicio en offset derivado `0x58a24`: `SetReqVerHigh: WinSock version can't be changed now`.

### MobAgent.exe

Agente SSH integrado en la suite; CRC del miembro incoherente, bytes no confiables.

- **Ruta:** `MOBASE!/bin/MobAgent.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `25faf1fb98489fe557d1c2a9c131e7d6a157f3cf0cf9a572697491411bf0b72e`. Hash derivado: `25faf1fb98489fe557d1c2a9c131e7d6a157f3cf0cf9a572697491411bf0b72e`; tamaño derivado: 526128 B; método `direct`.
  **Integridad:** CRC32 declarado por ZIP no coincide; contenido retenido como observado no confiable. No se usa como prueba de integridad ni de comportamiento autorizado.
  DLL importadas: `GDI32.dll`, `KERNEL32.dll`, `msvcrt.dll`, `SHELL32.dll`, `ADVAPI32.dll`, `COMDLG32.DLL`, `USER32.dll`.
  Indicio en offset derivado `0x66ab7`: `TMobaXtermForm`.
  Indicio en offset derivado `0x66fe4`: `Unable to write OpenSSH config file to %s`.

### MobaKeyGen.exe

Generador/conversor gráfico de claves con indicios PuTTYgen.

- **Ruta:** `MOBASE!/bin/MobaKeyGen.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `0334d8b86d29ea695ab8c4a7187fc1f23afb1a56ad692db5b58a57f5f67c0641`. Hash derivado: `0334d8b86d29ea695ab8c4a7187fc1f23afb1a56ad692db5b58a57f5f67c0641`; tamaño derivado: 574768 B; método `direct`.
  **Integridad:** CRC32 declarado por ZIP no coincide; contenido retenido como observado no confiable. No se usa como prueba de integridad ni de comportamiento autorizado.
  DLL importadas: `GDI32.dll`, `KERNEL32.dll`, `msvcrt.dll`, `SHELL32.dll`, `ADVAPI32.dll`, `COMDLG32.DLL`, `USER32.dll`.
  Indicio en offset derivado `0x71a00`: `PuTTYgen command line error`.
  Indicio en offset derivado `0x71a80`: `This public key contains an OpenSSH certificate.`.

### MobaRTE.exe

Auxiliar gráfico multimodo: editor, diff, navegador y RDP; ver informe principal.

- **Ruta:** `MOBASE!/bin/MobaRTE.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `02a72ace3736c436b003f92e7f966b887273d920b8f61a9606992d61d3f05db1`. Hash derivado: `02a72ace3736c436b003f92e7f966b887273d920b8f61a9606992d61d3f05db1`; tamaño derivado: 6426928 B; método `direct`.
  **Integridad:** CRC32 declarado por ZIP no coincide; contenido retenido como observado no confiable. No se usa como prueba de integridad ni de comportamiento autorizado.
  DLL importadas: `kernel32.dll`, `user32.dll`, `advapi32.dll`, `oleaut32.dll`, `kernel32.dll`, `advapi32.dll`, `kernel32.dll`, `gdi32.dll`, `user32.dll`, `kernel32.dll`, `ole32.dll`, `oleaut32.dll`, `ole32.dll`, `oleaut32.dll`, `comctl32.dll`, `imm32.dll`, `shell32.dll`, `shell32.dll`, `comdlg32.dll`, `kernel32.dll`, `kernel32.dll`, `user32.dll`.
  Indicio en offset derivado `0x11fdc`: `Starting MobaXterm`.
  Indicio en offset derivado `0x6fcc0`: `FormMobaRDP3`.

### MobaSCP.exe

Auxiliar de transferencias SSH/SCP/SFTP y coordinación con GUI; límites exactos pendientes.

- **Ruta:** `MOBASE!/bin/MobaSCP.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `1d42bb1ddc825013d65f39020e074d4a493eaf7255e3e3c2de32fc3d3ef61894`. Hash derivado: `1d42bb1ddc825013d65f39020e074d4a493eaf7255e3e3c2de32fc3d3ef61894`; tamaño derivado: 896816 B; método `direct`.
  **Integridad:** CRC32 declarado por ZIP no coincide; contenido retenido como observado no confiable. No se usa como prueba de integridad ni de comportamiento autorizado.
  DLL importadas: `KERNEL32.dll`, `msvcrt.dll`, `ADVAPI32.dll`, `USER32.dll`.
  Indicio en offset derivado `0xaf030`: `        [MobaSCP%d]  %s`.
  Indicio en offset derivado `0xaf574`: `Software\MobaXterm\MoTTY\ScpOpenFiles`.

### MobaVNC.exe

Cliente VNC de la suite; variante exacta pendiente.

- **Ruta:** `CygUtils.plugin!/bin/MobaVNC.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `2b777075c3ae797604e6064853ac536278919b549942492741ad918ea3e4cbeb`. Hash derivado: `2cd7aadab4196c3eff16f3872e7e56b92ea1bfe84aa9978fd9119c870fe488c4`; tamaño derivado: 547328 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `WSOCK32.dll`, `COMCTL32.dll`, `VNCHooks.dll`, `KERNEL32.dll`, `USER32.dll`, `GDI32.dll`, `ADVAPI32.dll`, `SHELL32.dll`, `ole32.dll`.
  Indicio en offset derivado `0x5d1c0`: `Driver version is: display = %u.%u.%u.%u (build %u), miniport = %u.%u.%u.%u (build %u), appMinReq = %u.%u.%u.%u`.
  Indicio en offset derivado `0x5d24b`: `vncVideoDriver::CheckVersion: ESC_QVI not supported by this version of Mirage`.
- **Ruta:** `CygUtils64.plugin!/bin/MobaVNC.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `2b777075c3ae797604e6064853ac536278919b549942492741ad918ea3e4cbeb`. Hash derivado: `2cd7aadab4196c3eff16f3872e7e56b92ea1bfe84aa9978fd9119c870fe488c4`; tamaño derivado: 547328 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `WSOCK32.dll`, `COMCTL32.dll`, `VNCHooks.dll`, `KERNEL32.dll`, `USER32.dll`, `GDI32.dll`, `ADVAPI32.dll`, `SHELL32.dll`, `ole32.dll`.
  Indicio en offset derivado `0x5d1c0`: `Driver version is: display = %u.%u.%u.%u (build %u), miniport = %u.%u.%u.%u (build %u), appMinReq = %u.%u.%u.%u`.
  Indicio en offset derivado `0x5d24b`: `vncVideoDriver::CheckVersion: ESC_QVI not supported by this version of Mirage`.

### mosh-client.exe

Cliente de terminal remota Mosh.

- **Ruta:** `CygUtils.plugin!/bin/mosh-client.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `6ea20282a2e746f99662beefb76cf8b306204d9023214c75f03402af7a7298cb`. Hash derivado: `d07cbc87fbf786d600a8af7b1053f815bad937223ce32cd871c1a3220aaf8c3e`; tamaño derivado: 701966 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygcrypto-3.dll`, `cygwin1.dll`, `cygncursesw-10.dll`, `cygz.dll`, `cyggcc_s-1.dll`, `cygstdc++-6.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x7469c`: `License GPLv3+: GNU GPL version 3 or later <http://gnu.org/licenses/gpl.html>.`.
  Indicio en offset derivado `0x74768`: `Usage: %s [-# 'ARGS'] IP PORT`.
- **Ruta:** `CygUtils64.plugin!/bin/mosh-client.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `3c44174dad3b3cedd1c1dc3e0ce8cf5c4281cb89a7de2f7895b22c507400a2e8`. Hash derivado: `5c2019e9954c6ac4faa320118f346bc21a3fa0cc8b87e94c5f1c719890d6a8b3`; tamaño derivado: 569856 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygcrypto-3.dll`, `cygwin1.dll`, `cygncursesw-10.dll`, `cygz.dll`, `cyggcc_s-seh-1.dll`, `cygstdc++-6.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x6ae5f`: `License GPLv3+: GNU GPL version 3 or later <http://gnu.org/licenses/gpl.html>.`.
  Indicio en offset derivado `0x6af29`: `Usage: %s [-# 'ARGS'] IP PORT`.

### MoTTY.exe

Cliente terminal/SSH derivado o adaptado de PuTTY; detalles por código pendientes.

- **Ruta:** `MOBASE!/bin/MoTTY.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `76e27f80a310cb8a40ebc0247fb071cbc965d331efa83de55873b652c4302157`. Hash derivado: `76e27f80a310cb8a40ebc0247fb071cbc965d331efa83de55873b652c4302157`; tamaño derivado: 3374376 B; método `direct`.
  **Integridad:** CRC32 declarado por ZIP no coincide; contenido retenido como observado no confiable. No se usa como prueba de integridad ni de comportamiento autorizado.
  DLL importadas: `GDI32.dll`, `KERNEL32.dll`, `msvcrt.dll`, `SHELL32.dll`, `ADVAPI32.dll`, `COMDLG32.DLL`, `IMM32.DLL`, `ole32.dll`, `USER32.dll`.
  Indicio en offset derivado `0xf34c8`: `HiddenMobaPasswdPrompt , %i , %s`.
  Indicio en offset derivado `0xf3510`: `MobaFont`.

### mount.exe

Gestión/consulta de montajes Cygwin.

- **Ruta:** `CygUtils.plugin!/bin/mount.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `c70d685041e2e0c61c80bd4742f38576d59544f8e23eb5b2ad1d3d4eb09c4526`. Hash derivado: `7397c89beb97a0410432b2d4edb7e1a16f137eb1fd66c24239de3c73682e9105`; tamaño derivado: 17437 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x2260`: `Usage: %1$s [OPTION] [<win32path> <posixpath>]`.
  Indicio en offset derivado `0x251d`: `  -V, --version                 output version information and exit`.
- **Ruta:** `CygUtils64.plugin!/bin/mount.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `f8362d6262426a857ad5ebcb10fd12d0e37bbb4471f16bd0a666243377a5552e`. Hash derivado: `80effe08493d2ba01b9d35209534a143baede1b01662daab770a01f53021b9e1`; tamaño derivado: 18451 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x2200`: `Usage: %1$s [OPTION] [<win32path> <posixpath>]`.
  Indicio en offset derivado `0x24bd`: `  -V, --version                 output version information and exit`.

### nc.exe

Conexiones TCP/UDP tipo netcat.

- **Ruta:** `CygUtils.plugin!/bin/nc.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `a23b77058abaf740a3464b20ed38857a2b92a84e79235e204dd17afaecfd409c`. Hash derivado: `727b7d61585bee2a8f1c07989745008bc6278081217d1e8c72fe6fa5fe85032a`; tamaño derivado: 24064 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x3fb4`: `usage: nc [-46CDdhklnrtUuvz] [-I length] [-i interval] [-O length]`.

### ncdu.exe

Explorador de uso de disco.

- **Ruta:** `CygUtils.plugin!/bin/ncdu.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `b39e0bd2acc2b7614327c6074d0ea1148717f624057f7534d7e4e20c43200ee1`. Hash derivado: `a9b1186f795d40b175111b908ab86f48118ce8fb187d72949a5e89e9a6d157ee`; tamaño derivado: 36366 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cygncursesw-10.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x75d6`: `   Disk usage:`.
  Indicio en offset derivado `0x7694`: ` Total disk usage: %s  Apparent size: %s  Items: %d`.

### nimesweeper.exe

Juego de buscaminas.

- **Ruta:** `CygUtils.plugin!/bin/nimesweeper.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `34d76ae498d8a08f80290b0a1b4a48edeb3790976e54865471fefbc3ed0f842d`. Hash derivado: `7bf9da43fa963fd32ac6202f879dbf4f1b3229b8936563b72d0f43a825726b60`; tamaño derivado: 31768 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cygncursesw-10.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x5789`: `Usage: %s [OPTION]`.
  Indicio en offset derivado `0x5982`: `Show version information`.

### ninvaders.exe

Juego de invasores en terminal.

- **Ruta:** `CygUtils.plugin!/bin/ninvaders.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `011e86467a4cc6a8e1285d3aca48144890f17fd7d83c0e3515e07e8695c4b4e7`. Hash derivado: `0afc333055ef30d40923705d264077a1359ea76480d531d6b385cc9fbb33a9d4`; tamaño derivado: 20992 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cygncurses-8.dll`, `KERNEL32.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`.
  Indicio en offset derivado `0x430a`: `Usage: nInvaders [-l skill] [-gpl]`.
  Indicio en offset derivado `0x4464`: `the Free Software Foundation; either version 2 of the License, or`.

### openssl.exe

Herramientas criptográficas y TLS.

- **Ruta:** `CygUtils.plugin!/bin/openssl.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `5e90ce566667988cc8a4e22c3c6b25c291597200aa2ccee28652266a8cb81b61`. Hash derivado: `e28ccb4cd8693a9fe71f961e3ea027e6220d4a062141cba054408f73047efc79`; tamaño derivado: 950286 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygcrypto-3.dll`, `cygssl-3.dll`, `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x82d2c`: `Usage: %s [options] [certreq...]`.
  Indicio en offset derivado `0x840c4`: `Usage: %s [options] [cipher]`.
- **Ruta:** `CygUtils64.plugin!/bin/openssl.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `118ead2e7d7042dda1fe2031fcb451c7cbfeebb9616c92855f8e0476da7b9051`. Hash derivado: `a2028a501c49e605e652eea0e682d600ffdaa3812eedb1463dbce6e83769d6c7`; tamaño derivado: 932371 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygcrypto-3.dll`, `cygssl-3.dll`, `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x82cf8`: `Usage: %s [options] [certreq...]`.
  Indicio en offset derivado `0x84348`: `Usage: %s [options] [cipher]`.

### OpenX.exe

Auxiliar de lanzamiento/integración X; no confundir por nombre con servidor X completo.

- **Ruta:** `CygUtils.plugin!/bin/OpenX.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `a6943a40ecbc95e4bfa2e20396e5d4f047ce3ca5827603934d1341fc3c4f1254`. Hash derivado: `c714be255e55884d5c440812c90dc5a4a9be7bdfe59a35dc309ea632f6f043ea`; tamaño derivado: 7680 B; método `lzma-alone`.
  DLL importadas: `cygX11-6.dll`, `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x1264`: `  Unknown pseudo relocation protocol version %d.`.
- **Ruta:** `XW!/bin/OpenX.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `8500f2a13c3194136e87081a54370cd664479f2e568872861f585fb140e891fe`. Hash derivado: `8500f2a13c3194136e87081a54370cd664479f2e568872861f585fb140e891fe`; tamaño derivado: 17422 B; método `direct`.
  **Integridad:** CRC32 declarado por ZIP no coincide; contenido retenido como observado no confiable. No se usa como prueba de integridad ni de comportamiento autorizado.
  DLL importadas: `KERNEL32.dll`, `msvcrt.dll`, `libX11-6.dll`.
  Indicio en offset derivado `0x2244`: `  Unknown pseudo relocation protocol version %d.`.

### passwd.exe

Gestión de contraseña del entorno.

- **Ruta:** `CygUtils.plugin!/bin/passwd.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `48e218507788f2df518411a1eb65203c282fbe610dcc20b6e4ac74cc2af6fe50`. Hash derivado: `b1e5cfabdc46c31bd9d76c8adb7ec51f5ea4c7ec14435b6c1a807d17dbd73d6e`; tamaño derivado: 23069 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `ADVAPI32.dll`, `KERNEL32.dll`, `NETAPI32.dll`.
  Indicio en offset derivado `0x2a60`: `Usage: %s [OPTION] [USER]`.
  Indicio en offset derivado `0x307c`: `  -V, --version            output version information and exit.`.

### paste.exe

Combinación de columnas de texto.

- **Ruta:** `CygUtils.plugin!/bin/paste.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `912e988bdc6c623cb50f5f69883f1ec8ac5b01f12ffb10c1e41177576465f543`. Hash derivado: `d515555e7a7505f7a2d4a2ff1ee62f32f91ef5db25ec4e9ba316ee725c053e27`; tamaño derivado: 19470 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cygintl-8.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x36d4`: `Usage: %s [OPTION]... [FILE]...`.
  Indicio en offset derivado `0x38b4`: `      --version  output version information and exit`.

### portmap.exe

Mapeador de puertos RPC.

- **Ruta:** `CygUtils.plugin!/bin/portmap.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `e93acf2f05bb9fd19d3c7df81792570f46e9b4b9d79b28d78a56300d0c64f1b1`. Hash derivado: `7f8db649a06f92a19cfd0993aa793efb68269436d023e472598019cb21a271fa`; tamaño derivado: 41998 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x6b10`: `Usage: %s [-D]`.
  Indicio en offset derivado `0x70ac`: `; low version = %lu, high version = %lu`.

### ps.exe

Listado de procesos.

- **Ruta:** `CygUtils.plugin!/bin/ps.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `f759613be2545a48376aa0a6cb2d60be2a81925f5a7b327488f0bea5979ed96a`. Hash derivado: `a9e9c3ebd2b775543120b7e4db91a526ab32fa97354cf310d15a01305ff11c0f`; tamaño derivado: 16925 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `ADVAPI32.dll`, `KERNEL32.dll`, `ntdll.dll`, `PSAPI.DLL`.
  Indicio en offset derivado `0x20cc`: `Usage: %1$s [-aefls] [-u UID] [-p PID]`.
  Indicio en offset derivado `0x2283`: ` -V, --version   output version information and exit`.

### rcp.exe

Copia remota heredada.

- **Ruta:** `CygUtils.plugin!/bin/rcp.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `b3c544be40aad6f2e294e20dd29b87a212c94561a5a2f7068711db4c152758a7`. Hash derivado: `6a5fb68ad5870207cc2cccc986218dcc6d94acceb1f7c79e7a02892d19cb82f3`; tamaño derivado: 19968 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x37dc`: `usage: rcp [-p] f1 f2; or: rcp [-rp] f1 ... fn directory`.
  Indicio en offset derivado `0x3c28`: `  Unknown pseudo relocation protocol version %d.`.
- **Ruta:** `CygUtils64.plugin!/bin/rcp.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `f2a297155e260fcdb482fd3ff3eba021c54b41d8468b549b217d24acd732e88c`. Hash derivado: `9e71c3ebf087fb70c37c947eed8ca337ed8f6b6f2f3be15ac66c54c549c716b8`; tamaño derivado: 18451 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x3048`: `usage: rcp [-p] f1 f2; or: rcp [-rp] f1 ... fn directory`.

### regtool.exe

Acceso al registro de Windows.

- **Ruta:** `CygUtils.plugin!/bin/regtool.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `3f25a1f3b645dbc427f6581290eaed131423db15963d4787fb91f57f05236cd3`. Hash derivado: `e398415bf0dc68d81ffbad5d0205dec8a943ddb8182edcf76d8e5fb5c6da6959`; tamaño derivado: 25629 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `ADVAPI32.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x3080`: `Usage: %s [OPTION] ACTION KEY [data...]`.
  Indicio en offset derivado `0x38f1`: ` -V, --version  output version information and exit`.

### rlogin.exe

Login remoto heredado.

- **Ruta:** `CygUtils.plugin!/bin/rlogin.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `3a2259854c09aee259e038ab146fd9515c6ff3f04e851970ed5478abe4a93efa`. Hash derivado: `1a26798b6c117533b3267ec677c2161cecf38132d8b9323853e730b703c8762f`; tamaño derivado: 15872 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x2994`: `usage: rlogin [ -%s]%s[-e char] [ -l username ] host`.
  Indicio en offset derivado `0x2cb8`: `  Unknown pseudo relocation protocol version %d.`.
- **Ruta:** `CygUtils64.plugin!/bin/rlogin.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `58abe03ff34e7c740c7895759fdc3365fd523f8d8e5dcb95c0faf1fdf378ac24`. Hash derivado: `3a2957532bc41ccf64e081c086aa19de7aeaec705e442de07a22c88dc43aa08d`; tamaño derivado: 14867 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x2448`: `usage: rlogin [ -%s]%s[-e char] [ -l username ] host`.

### rpc.mountd.exe

Servicio auxiliar de montaje NFS.

- **Ruta:** `CygUtils.plugin!/bin/rpc.mountd.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `87f7508ca949407cd94d297c15a7757e12c70bbcf5567a3308a3e78ff24b2167`. Hash derivado: `e8cd6148077e6373d135d2fdb6d79f636fde180c2ecaebf8271f06568eccbf20`; tamaño derivado: 82958 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0xdfb8`: `Usage: %s [-Fhnpv] [-d kind] [-f exports-file] [-P port]`.
  Indicio en offset derivado `0xf72c`: `; low version = %lu, high version = %lu`.

### rpc.nfsd.exe

Servidor NFS.

- **Ruta:** `CygUtils.plugin!/bin/rpc.nfsd.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `12044a6dea996a3700c44ab1e97f54cd1525cf354698d53ad141ebc5d1f3be9c`. Hash derivado: `bbdd556c56f040d3a51c48dcb59656deddd40b0396aefccad201fb782fcb4868`; tamaño derivado: 96270 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x103cc`: `Usage: %s [-Fhnpv] [-d kind] [-f exports-file] [-P port] [--version]`.
  Indicio en offset derivado `0x11e8c`: `; low version = %lu, high version = %lu`.

### rsh.exe

Ejecución remota heredada.

- **Ruta:** `CygUtils.plugin!/bin/rsh.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `4c79278c8de19e48d0804a85164588fbcc85908272dd0fd94e7644106c75521e`. Hash derivado: `484992c00b73f81eba61907ddac7c83a6f68ef3696c229d24994647e6c393d9d`; tamaño derivado: 13312 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x1f84`: `usage: rsh [-nd%s]%s[-l login] host [command]`.
  Indicio en offset derivado `0x21e0`: `  Unknown pseudo relocation protocol version %d.`.
- **Ruta:** `CygUtils64.plugin!/bin/rsh.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `e7651d7ff86e29acc3975c61835d14ce969dbcf0c948636ea390b7562a478fac`. Hash derivado: `bcb89e196bebf5b0520776dc35370a721af815094ddb25dcf9bf8205d0e8137f`; tamaño derivado: 11795 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x1a38`: `usage: rsh [-nd%s]%s[-l login] host [command]`.

### rsync.exe

Sincronización incremental de archivos.

- **Ruta:** `CygUtils.plugin!/bin/rsync.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `e44bd912e11d5e7dbfd1e0691b7454c2c09c31f508f8736141a0a0e76eac8793`. Hash derivado: `4baa7ce7c05187c8e5404bd08e186239facbbcf7eb81b2d0d3a78bf0d9db3ce9`; tamaño derivado: 576526 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygcrypto-3.dll`, `cygwin1.dll`, `cygiconv-2.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
- **Ruta:** `CygUtils64.plugin!/bin/rsync.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `8e735ab17d75c72f3479f364bcd3b82f53ec91ca723d18c1d20e2b910404467f`. Hash derivado: `bce105974907e1aa34f7fdba97c771cba07af901dd0a38705a1f1fef6e7b079b`; tamaño derivado: 538112 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygcrypto-3.dll`, `cygwin1.dll`, `cygiconv-2.dll`, `KERNEL32.dll`.

### runscript.exe

Ejecución de scripts de conversación de terminal.

- **Ruta:** `CygUtils.plugin!/bin/runscript.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `21cc8f1fd3603ac351f111903e8e147870efc67416dae6105bfc910124a3749d`. Hash derivado: `116d535c11a9abc43aa5380144501d2339813a74337e104e0d6fabbdabe6db07`; tamaño derivado: 17422 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x3380`: `runscript, part of minicom version %s`.
  Indicio en offset derivado `0x33ac`: `Usage: runscript <scriptfile> [logfile [homedir]]%s`.

### scp.exe

Copia de archivos mediante SSH.

- **Ruta:** `CygUtils.plugin!/bin/scp.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `f6b8dc55d2fef0b8a0b3d5f86749065e5802c0cd47a00398fb312c4f2e97ea50`. Hash derivado: `0bd5c466b7b7a78208ea272b27919bb33cd5571a2d590087d04ee5749b6518b8`; tamaño derivado: 219662 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x26b1c`: `usage: scp [-346ABCOpqRrsTv] [-c cipher] [-D sftp_server_path] [-F ssh_config]`.
  Indicio en offset derivado `0x28170`: `Server version does not support lstat operation`.
- **Ruta:** `CygUtils64.plugin!/bin/scp.exe`; tipo `mz-unparsed`; arquitectura `no establecida`.
  Hash original: `4930da526a66b0f29b00774f0d22ac1489ca96ecc2c046be0f0200f640eb241c`. Hash derivado: `fd9b4d6cdf6b3027455bf2d8585c60d793ab37ecfcb9953ac56755a1aaa48b30`; tamaño derivado: 199680 B; método `lzma-derived-header-byte1-zero`.

### setfacl.exe

Modificación de ACL.

- **Ruta:** `CygUtils.plugin!/bin/setfacl.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `14bf7e944a774d2a0aea76f9d5584fd0305ffc9bd6690f58d290fb0cd291bd89`. Hash derivado: `86bf81214cabb1060e80e6f946bb5feaac482105ec10697cef77d3a18f5019fe`; tamaño derivado: 20509 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x2060`: `Usage: %s [-r] {-f ACL_FILE \| -s acl_entries} FILE...`.
  Indicio en offset derivado `0x2365`: `  -V, --version    output version information and exit`.

### setxkbmap.exe

Selección de mapa de teclado XKB.

- **Ruta:** `CygUtils.plugin!/bin/setxkbmap.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `46facecf826c6814b7f2985f9637e762994a077166f2b81adfacddb7d194f275`. Hash derivado: `fd23e3a2525fc0a766a1033b6ca16cc554195cf8575c13ed5ca4c394ba75a929`; tamaño derivado: 19968 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cygX11-6.dll`, `cygxkbfile-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x2f78`: `Usage: %s [options] [<layout> [<variant> [<option> ... ]]]`.
  Indicio en offset derivado `0x37b4`: `%s was compiled with XKB version %d.%02d`.
- **Ruta:** `XW!/bin/setxkbmap.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `7d0cff9b2df115cfffb94b970d86b0602a94debb569829219166d784e7f0b627`. Hash derivado: `7d0cff9b2df115cfffb94b970d86b0602a94debb569829219166d784e7f0b627`; tamaño derivado: 57870 B; método `direct`.
  **Integridad:** CRC32 declarado por ZIP no coincide; contenido retenido como observado no confiable. No se usa como prueba de integridad ni de comportamiento autorizado.
  DLL importadas: `KERNEL32.dll`, `msvcrt.dll`, `libX11-6.dll`.
  Indicio en offset derivado `0x929c`: `Usage: %s [options] [<layout> [<variant> [<option> ... ]]]`.
  Indicio en offset derivado `0x974c`: `  -version            Print the program's version number`.

### sftp-server.exe

Subsistema servidor SFTP.

- **Ruta:** `CygUtils.plugin!/bin/sftp-server.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `c5c6902de1ab970a2195c60d405f3458ba81864655f6391833d7500903e152f0`. Hash derivado: `18732ceb4eb0b04b74963df64d46b348e2e1308e505e33b01f4bdc4cfc6e6cdd`; tamaño derivado: 57870 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygopenssh.dll`, `cygwin1.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x99a8`: `usage: %s [-ehR] [-d start_directory] [-f log_facility] [-l log_level]`.
  Indicio en offset derivado `0x9b49`: `received client version %u`.
- **Ruta:** `CygUtils64.plugin!/bin/sftp-server.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `c68347805e1f5704f93358d946b21f34374d71e4e2a5dd9c96637e526c499dfe`. Hash derivado: `5dfff9b2ec8f5bddbd93f19f19466b4b930e5939fcaf1c4e7694bf8873be4908`; tamaño derivado: 54272 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygopenssh.dll`, `cygwin1.dll`, `cyggcc_s-seh-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x99a0`: `usage: %s [-ehR] [-d start_directory] [-f log_facility] [-l log_level]`.
  Indicio en offset derivado `0x9b45`: `received client version %u`.

### sftp.exe

Cliente SFTP.

- **Ruta:** `CygUtils.plugin!/bin/sftp.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `c740fa3180085015fc01382ed8227828925b291cfba85cc9366f82d131e32fb3`. Hash derivado: `e0251d5bc3a83a4da5e61aa1b61720146c276f50ce044e6fef993f9971304d9e`; tamaño derivado: 130062 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygopenssh.dll`, `cygwin1.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x1665c`: `usage: %s [-46AaCfNpqrv] [-B buffer_size] [-b batchfile] [-c cipher]`.
  Indicio en offset derivado `0x1757c`: `version                            Show SFTP version`.
- **Ruta:** `CygUtils64.plugin!/bin/sftp.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `57d6d90f80deda4a35d1a135a4f6fc13d86ce9603d824fbe54c8ac21c7ec4e23`. Hash derivado: `d6855090407b7040bd1432ed9276bcd5258139bb6611db152409b89f281d1693`; tamaño derivado: 120320 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygopenssh.dll`, `cygwin1.dll`, `cyggcc_s-seh-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x15c20`: `usage: %s [-46AaCfNpqrv] [-B buffer_size] [-b batchfile] [-c cipher]`.
  Indicio en offset derivado `0x16b70`: `version                            Show SFTP version`.

### showkey.exe

Diagnóstico de teclas.

- **Ruta:** `CygUtils.plugin!/bin/showkey.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `bfbdecee12134593acb49e63a007abad7e64690b00498233db751db130c43c8f`. Hash derivado: `874ea0c9c7b62f1e598694d66c2e576d61fcb6306d402e24d91adf4ae88a13fd`; tamaño derivado: 6670 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.
- **Ruta:** `CygUtils64.plugin!/bin/showkey.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `89e71028e019b852137aaad0e30be07038789df342f5d34bd0399277406d61d2`. Hash derivado: `4a79951453a3b55a20bf687ac5991323be7fdd83e188e51b8c034639204081af`; tamaño derivado: 9728 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.

### showmount.exe

Consulta de exportaciones NFS.

- **Ruta:** `CygUtils.plugin!/bin/showmount.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `a96c2f401b5ec0c95ee75220306e178ff678753f74f51c7a9b34c95bf547e973`. Hash derivado: `7c83b2d2a4bd2664bc0d61229ca0cc62aadcd8a2e93971055ea174c107d3b373`; tamaño derivado: 28672 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x5e30`: `Usage: %s [-adehv]`.
  Indicio en offset derivado `0x61a0`: `; low version = %lu, high version = %lu`.

### shutdown.exe

Apagado/reinicio del sistema.

- **Ruta:** `CygUtils.plugin!/bin/shutdown.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `394a4a203cf2c7b2b0a8366c66d01e630c0f2d0c80f46d9af508dbf4d510318c`. Hash derivado: `8bf95ffecfcf0344ebde1c5e2f9e644211ad7de048fc72676591e785df5020a5`; tamaño derivado: 11264 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `ADVAPI32.DLL`, `KERNEL32.dll`, `USER32.dll`.
  Indicio en offset derivado `0x1b9d`: `Usage: %s [OPTION]... time`.
  Indicio en offset derivado `0x1dbc`: `      --version    Output version information and exit.`.

### socat.exe

Puente bidireccional entre endpoints.

- **Ruta:** `CygUtils.plugin!/bin/socat.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `34db058ea411107a0d5012994f18d0df739cbaa62a4522021baed7fc3d3322bd`. Hash derivado: `b516665250b5e2e8344ce1631e94fbd86e686e2bfe3c8e8dd7c9bc531748084a`; tamaño derivado: 375822 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygcrypto-3.dll`, `cygwin1.dll`, `cygssl-3.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x3a6fb`: `socat version %s on %s`.
  Indicio en offset derivado `0x3a71c`: `running on %s version %s, release %s, machine %s`.

### soliterm.exe

Acceso Serial-over-LAN; verificar variante.

- **Ruta:** `CygUtils.plugin!/bin/soliterm.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `a40f259a74bf8cad0a9bbc693f7217647fc899fc3b2c508b9882f107f3d5ca87`. Hash derivado: `61135275b72804d4eac811c7f71e53c9c65c3029acba7e2912a62f98fa1ab610`; tamaño derivado: 37390 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `cygncursesw-10.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x6688`: `Solitaire games for your terminal, by Mobatek`.
  Indicio en offset derivado `0x69b9`: `usage: %s [-v\|--version] [-h\|--help] [-f\|--freecell] [-k\|--klondike [-p\|--passes=NUMBER]]`.
- **Ruta:** `CygUtils64.plugin!/bin/soliterm.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `716f572415a1ab255772a9eb4a03b1357d6e7d7acdf59f1c0362781b6d38e54c`. Hash derivado: `b66550de1aea2b0dc15ebca3d76a8787c6c0bc8f338c3641718a50fbb144826a`; tamaño derivado: 41472 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `cygncursesw-10.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x7648`: `Solitaire games for your terminal, by Mobatek`.
  Indicio en offset derivado `0x7989`: `usage: %s [-v\|--version] [-h\|--help] [-f\|--freecell] [-k\|--klondike [-p\|--passes=NUMBER]]`.

### ssh-add.exe

Carga y consulta de identidades en agente SSH.

- **Ruta:** `CygUtils.plugin!/bin/ssh-add.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `afd51e31eb560d5961b0e9886b21eca9d84175d9635ea2b0cd2f2e0cd6f1e480`. Hash derivado: `e88ad12545d7d2245d61a82a13b46b9ca36569d4b3065d1cfaa5d959cf21cc1a`; tamaño derivado: 48654 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygopenssh.dll`, `cygwin1.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`, `USER32.dll`.
  Indicio en offset derivado `0x79f3`: `TMobaXtermForm`.
  Indicio en offset derivado `0x7d28`: `usage: ssh-add [-CcDdKkLlqvXx] [-E fingerprint_hash] [-H hostkey_file]`.
- **Ruta:** `CygUtils64.plugin!/bin/ssh-add.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `53a1b3bf2ec2599df9d80b7d444ddaa833a3fcac96fa47f550c10d04239ba780`. Hash derivado: `ab603f2f4cc4eeb5e1420802dfa686287fc09f679b7f361f07137d531839443f`; tamaño derivado: 47616 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygopenssh.dll`, `cygwin1.dll`, `cyggcc_s-seh-1.dll`, `KERNEL32.dll`, `USER32.dll`.
  Indicio en offset derivado `0x77bf`: `TMobaXtermForm`.
  Indicio en offset derivado `0x7b08`: `usage: ssh-add [-CcDdKkLlqvXx] [-E fingerprint_hash] [-H hostkey_file]`.

### ssh-agent-original.exe

Agente SSH original del paquete.

- **Ruta:** `CygUtils.plugin!/bin/ssh-agent-original.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `8d8224889be30dab744185e819297489740c126e23a60d2c1213491a3c7a494d`. Hash derivado: `ac3c4ea37fbc55dc4eaf0c43a5b460b0c8cf3b7257aee43dd77027f317e20c3b`; tamaño derivado: 79886 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygopenssh.dll`, `cygwin1.dll`, `cyggcc_s-1.dll`, `ADVAPI32.dll`, `KERNEL32.dll`, `USER32.dll`.
  Indicio en offset derivado `0xd054`: `usage: ssh-agent [-c \| -s] [-DdTU] [-a bind_address] [-E fingerprint_hash]`.

### ssh-agent.exe

Agente SSH o wrapper hacia otro agente.

- **Ruta:** `CygUtils.plugin!/bin/ssh-agent.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `7bea1533efe48d6acce7d9ada49f32383f7c9072a89df69029a6e98559f3d81f`. Hash derivado: `b5ca98b87b3ac27aa61034fc23b228f50df4c13d029b56a94a81b2d6f2aae0a7`; tamaño derivado: 18446 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `ADVAPI32.dll`, `KERNEL32.dll`, `USER32.dll`.
  Indicio en offset derivado `0x2470`: `Usage: %s [options] [command [arg ...]]`.
  Indicio en offset derivado `0x24d4`: `  -v, --version  Display version information`.
- **Ruta:** `CygUtils64.plugin!/bin/ssh-agent.exe`; tipo `mz-unparsed`; arquitectura `no establecida`.
  Hash original: `e8d221785ccac4be3fafa703ea7ed8a5108918a7fa912bc32210f877c7cf255f`. Hash derivado: `b118e6c62fd924ba45944318d568636c11f7250b7a0a4fbdd1050577fa7f307d`; tamaño derivado: 74752 B; método `lzma-derived-header-byte1-zero`.

### ssh-keygen.exe

Generación y conversión de claves SSH.

- **Ruta:** `CygUtils.plugin!/bin/ssh-keygen.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `80f281f33b322ae587c45085fcf8a0b96e9f364a1de474545173125c0a50632e`. Hash derivado: `00c208ddabba7051222ef34515cff0941ad8e8e15aa2ab32759100fab469f680`; tamaño derivado: 153102 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygopenssh.dll`, `cygcrypto-3.dll`, `cygwin1.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x19a04`: `usage: ssh-keygen [-q] [-a rounds] [-b bits] [-C comment] [-f output_keyfile]`.
  Indicio en offset derivado `0x1b000`: `%u-bit %s, converted by %s@%s from OpenSSH`.
- **Ruta:** `CygUtils64.plugin!/bin/ssh-keygen.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `ef4e883d49c98ed4da1b4bd1cd9060a9184484dd2bcd75120d0d81d369f250bf`. Hash derivado: `47897d3672ca2c4386d5fbc7a606daea0dec8a2df36a24113f66ad3192b49d23`; tamaño derivado: 145920 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygopenssh.dll`, `cygcrypto-3.dll`, `cygwin1.dll`, `cyggcc_s-seh-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x197c8`: `usage: ssh-keygen [-q] [-a rounds] [-b bits] [-C comment] [-f output_keyfile]`.
  Indicio en offset derivado `0x1ae78`: `%u-bit %s, converted by %s@%s from OpenSSH`.

### ssh-keyscan.exe

Obtención de claves públicas anunciadas por servidores SSH.

- **Ruta:** `CygUtils.plugin!/bin/ssh-keyscan.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `ed0aa3854d6992880e1f69b0d4369cceeeda9dd439017ebb756d0e1eb1d38251`. Hash derivado: `5cc0907a6952423efee086f86467aad6882eb0ec0dd764cab25e786c97cf6cff`; tamaño derivado: 44046 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygopenssh.dll`, `cygwin1.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x6440`: `usage: ssh-keyscan [-46cDHqv] [-f file] [-O option] [-p port] [-T timeout]`.
  Indicio en offset derivado `0x67a3`: `SSH-%d.%d-OpenSSH-keyscan`.
- **Ruta:** `CygUtils64.plugin!/bin/ssh-keyscan.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `1fad60eed914baf13e73fef142dbca3ba875d7fe1f29a8f358e45454999009c3`. Hash derivado: `3823b430b0492ee2e0c652fb182789aa73a8ddae12e16bce81f2e1dc8b506d72`; tamaño derivado: 43008 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygopenssh.dll`, `cygwin1.dll`, `cyggcc_s-seh-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x6600`: `usage: ssh-keyscan [-46cDHqv] [-f file] [-O option] [-p port] [-T timeout]`.
  Indicio en offset derivado `0x6987`: `SSH-%d.%d-OpenSSH-keyscan`.

### ssh-pageant.exe

Puente de agente SSH hacia Pageant.

- **Ruta:** `CygUtils.plugin!/bin/ssh-pageant.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `45d938407b52b90dc8a850fba0cf82bf400fbc484bd3c5166172ce27eafab994`. Hash derivado: `15da26abeac31bc2350c807acd06d827d7f4242b94d4bba99694676a26042493`; tamaño derivado: 18446 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `ADVAPI32.dll`, `KERNEL32.dll`, `USER32.dll`.
  Indicio en offset derivado `0x2470`: `Usage: %s [options] [command [arg ...]]`.
  Indicio en offset derivado `0x24d4`: `  -v, --version  Display version information`.

### ssh-pkcs11-helper.exe

Auxiliar de acceso SSH a proveedores PKCS#11.

- **Ruta:** `CygUtils.plugin!/bin/ssh-pkcs11-helper.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `1cd8607d0caa185a2d348e1bf1862874c6599f5dd8bbbadf223cb022db14a7c4`. Hash derivado: `d77b09a04479ec8e7af142980c5c14875d850b256dd300da642de28075459ee1`; tamaño derivado: 55310 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygopenssh.dll`, `cygcrypto-3.dll`, `cygwin1.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x905c`: `usage: %s [-v]`.
- **Ruta:** `CygUtils64.plugin!/bin/ssh-pkcs11-helper.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `c859406816bdfa9f83d05556be11b29d0dd0a8a1db55d972190f365114a25fa9`. Hash derivado: `121a707da3c0e18876338d2d20556d41d6615ef20d25653a4f464a58684be435`; tamaño derivado: 55296 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygopenssh.dll`, `cygcrypto-3.dll`, `cygwin1.dll`, `cyggcc_s-seh-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x941c`: `usage: %s [-v]`.

### ssh-sk-helper.exe

Auxiliar para claves SSH respaldadas por hardware.

- **Ruta:** `CygUtils.plugin!/bin/ssh-sk-helper.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `d7a86feafb0fcca3c0b282d32abe16713446b68550b9679bbee7e3934bff3730`. Hash derivado: `f7022b6607ebb47be87bba4cd9051356386d81986ae0e4e326e2452460a57448`; tamaño derivado: 53262 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygopenssh.dll`, `cygcrypto-3.dll`, `cygwin1.dll`, `cygfido2-1.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x8094`: `usage: %s [-v]`.
  Indicio en offset derivado `0x833c`: `provider %s is not an OpenSSH FIDO library`.
- **Ruta:** `CygUtils64.plugin!/bin/ssh-sk-helper.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `d460b0fb842d9bc1c3b9b0006fd28edc209b9d2e3e2f8c3df050c0d95f859112`. Hash derivado: `fa1eddae9b34b1f3525e3996dcaec62bd55b51fe6bf8becc3323a45c5eb2f1a7`; tamaño derivado: 51200 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygopenssh.dll`, `cygcrypto-3.dll`, `cygwin1.dll`, `cygfido2-1.dll`, `cyggcc_s-seh-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x7e54`: `usage: %s [-v]`.
  Indicio en offset derivado `0x8110`: `provider %s is not an OpenSSH FIDO library`.

### ssh.exe

Cliente SSH o wrapper que selecciona backend.

- **Ruta:** `CygUtils.plugin!/bin/ssh.exe`; tipo `script`; arquitectura `no establecida`.
  Hash original: `4f6a13d542e3816177432aca8398b70e69fdb162642c8ca6195fb9f77a0dbddd`. Hash derivado: `144f79fd75db4fb5a400900ecfb426a2667305a55173246a05b25d693a0d46fa`; tamaño derivado: 5077 B; método `lzma-alone`.
  Script observado; intérprete `#!/bin/bash.exe`. Referencias a programas: `/bin/MobaBox.exe`, `/bin/XWin.exe`, `/bin/_ssh.exe`, `/bin/_ssh1.exe`, `/bin/bash.exe`, `/bin/busybox.exe`, `/bin/cygcom_err-2.dll`, `/bin/cygcom_err-3.dll`, `/bin/cygcrypto-1.1.dll`, `/bin/cyggcc_s-1.dll`, `/bin/cyggcc_s-seh-1.dll`, `/bin/cyggssapi_krb5-2.dll`.
- **Ruta:** `CygUtils64.plugin!/bin/ssh.exe`; tipo `script`; arquitectura `no establecida`.
  Hash original: `58f42e3700c46ea476ee88ebe9dc9e5cddb176a88a8ece1b5bca75c69734d897`. Hash derivado: `4b5fb3d2186c51daac38442227e0006826b32ebc71cdb86a781be7ce4a867c22`; tamaño derivado: 3831 B; método `lzma-alone`.
  Script observado; intérprete `#!/bin/zsh.exe`. Referencias a programas: `/bin/MobaBox.exe`, `/bin/_ssh.exe`, `/bin/_ssh1.exe`, `/bin/busybox.exe`, `/bin/cygcom_err-2.dll`, `/bin/cygcom_err-3.dll`, `/bin/cygcrypto-1.1.dll`, `/bin/cyggcc_s-1.dll`, `/bin/cyggcc_s-seh-1.dll`, `/bin/cyggssapi_krb5-2.dll`, `/bin/cygintl-8.dll`, `/bin/cygk5crypto-3.dll`.

### sshd-auth.exe

Subproceso de autenticación de sshd.

- **Ruta:** `CygUtils.plugin!/bin/sshd-auth.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `7a628e785021cae2c2516c00676e2d7c220a29b2b41d53394a42698eb55a2efa`. Hash derivado: `86187f6ead847470084d660461ddd1bb64f91d021a76710de32fb109613b09e3`; tamaño derivado: 354830 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygopenssh.dll`, `cygcrypto-3.dll`, `cygwin1.dll`, `cyggssapi_krb5-2.dll`, `cygkrb5-3.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x3bb46`: `OpenSSH_10.3`.
  Indicio en offset derivado `0x3bb5c`: `usage: sshd [-46DdeGiqTtV] [-C connection_spec] [-c host_cert_file]`.
- **Ruta:** `CygUtils64.plugin!/bin/sshd-auth.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `b015780922922a12a14e2be0aa8d19374b06d0c8ca78c65d4ac633536fe4bb85`. Hash derivado: `44aba2f879c83c4f34f238958685eef937c3b70a300c20fca11b9e6df1e80efe`; tamaño derivado: 337408 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygopenssh.dll`, `cygcrypto-3.dll`, `cygwin1.dll`, `cyggssapi_krb5-2.dll`, `cygkrb5-3.dll`, `cyggcc_s-seh-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x3c306`: `OpenSSH_10.3`.
  Indicio en offset derivado `0x3c320`: `usage: sshd [-46DdeGiqTtV] [-C connection_spec] [-c host_cert_file]`.

### sshd-session.exe

Subproceso de sesión de sshd.

- **Ruta:** `CygUtils.plugin!/bin/sshd-session.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `e47c8f147029ea8f92536044fe31777fa0f704a67d6059f0324a94f7f4c4374b`. Hash derivado: `6c1c3128dc7adadaca30cd355edfceb8bed3db839ab352974de6655bd591c0c1`; tamaño derivado: 397326 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygopenssh.dll`, `cygcrypto-3.dll`, `cygwin1.dll`, `cyggssapi_krb5-2.dll`, `cygkrb5-3.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x43a01`: `OpenSSH_10.3p1`.
  Indicio en offset derivado `0x43a18`: `usage: sshd [-46DdeGiqTtV] [-C connection_spec] [-c host_cert_file]`.
- **Ruta:** `CygUtils64.plugin!/bin/sshd-session.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `ed5d1daf1c1d3684ae87ad247956cb5eb644fce39c79fa34fb82e64b5c549901`. Hash derivado: `7b72d09ac8462ca9f39812e2774054194a6d846e3d2c278b68de16c0dcbacc03`; tamaño derivado: 375808 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygopenssh.dll`, `cygcrypto-3.dll`, `cygwin1.dll`, `cyggssapi_krb5-2.dll`, `cygkrb5-3.dll`, `cyggcc_s-seh-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x43f85`: `OpenSSH_10.3p1`.
  Indicio en offset derivado `0x43fa0`: `usage: sshd [-46DdeGiqTtV] [-C connection_spec] [-c host_cert_file]`.

### sshd.exe

Servidor SSH.

- **Ruta:** `CygUtils.plugin!/bin/sshd.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `34ec2ac7359ba47097b126e753d12bd7f20a6bce463b2c5108f55b1108a25630`. Hash derivado: `357bef57ba798aff1ccf34f2fe31cf787d82fb26f25200b7b94f55140792d1e5`; tamaño derivado: 177166 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygopenssh.dll`, `cygcrypto-3.dll`, `cygwin1.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x1dd3b`: `OpenSSH_10.3p1`.
  Indicio en offset derivado `0x1dd54`: `usage: sshd [-46DdeGiqTtV] [-C connection_spec] [-c host_cert_file]`.
- **Ruta:** `CygUtils64.plugin!/bin/sshd.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `37b9ac9a6cff13bfd991fb60d630fc224dc3272352ebebd4cef379a586ed890f`. Hash derivado: `2f802b41dd4df99e4480d5a8f7d79f4e4cab3329656fa2e6d0d7e215df873960`; tamaño derivado: 171520 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygopenssh.dll`, `cygcrypto-3.dll`, `cygwin1.dll`, `cyggcc_s-seh-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x1e703`: `OpenSSH_10.3p1`.
  Indicio en offset derivado `0x1e720`: `usage: sshd [-46DdeGiqTtV] [-C connection_spec] [-c host_cert_file]`.

### sshpass.exe

Automatización de entrada de contraseña SSH; evitar replicar almacenamiento inseguro.

- **Ruta:** `CygUtils.plugin!/bin/sshpass.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `b51edc3053147fc283fef99b0109cb411c7ea7caf54e1be37f7e38dbd48dca00`. Hash derivado: `d7301a78c247c7d47527f4a11715e3cf9c572ce9f46fa4ef106438ec0a99ccfb`; tamaño derivado: 12814 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x1a5c`: `Usage: sshpass [-f\|-d\|-p\|-e] [-hV] command parameters`.
  Indicio en offset derivado `0x1be3`: `   -V            Print version information`.

### stgames.exe

Colección/launcher de juegos; alcance pendiente.

- **Ruta:** `MOBASE!/bin/stgames.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `f06350d2a7981bf3aa40cd846d0e1f873c9b4008d8da06e80a21a15e10656deb`. Hash derivado: `f06350d2a7981bf3aa40cd846d0e1f873c9b4008d8da06e80a21a15e10656deb`; tamaño derivado: 292864 B; método `direct`.
  **Integridad:** CRC32 declarado por ZIP no coincide; contenido retenido como observado no confiable. No se usa como prueba de integridad ni de comportamiento autorizado.
  DLL importadas: `COMCTL32.DLL`, `COMDLG32.DLL`, `GDI32.dll`, `KERNEL32.dll`, `msvcrt.dll`, `USER32.dll`.
  Indicio en offset derivado `0x4105c`: `Cannot handle this version of the saved game file format`.
  Indicio en offset derivado `0x44450`: `  Unknown pseudo relocation protocol version %d.`.

### strace.exe

Trazado/diagnóstico de procesos Cygwin.

- **Ruta:** `CygUtils.plugin!/bin/strace.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `4894da9e1082f171174d7b53ff3b84fd26387137720fffff60c15bd79b5c4354`. Hash derivado: `4c687706154ae07dcb0dff7d2a4c10b0f5e8ee2f178da59b2455fe58043836b6`; tamaño derivado: 39955 B; método `lzma-alone`.
  DLL importadas: `ADVAPI32.dll`, `KERNEL32.dll`, `msvcrt.dll`, `ntdll.dll`.
  Indicio en offset derivado `0x6968`: `Usage: %s [OPTIONS] <command-line>`.
  Indicio en offset derivado `0x698b`: `Usage: %s [OPTIONS] -p <pid>`.

### sudoku.exe

Juego Sudoku.

- **Ruta:** `CygUtils.plugin!/bin/sudoku.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `816a2393ba5dc8cfa1b9f74d3dc4f047c5c295a73067ba963b21403f81e3d27c`. Hash derivado: `2bed2565a72468fc5c5b1b706913df4c46e4f2add886cf50f71303025cea909d`; tamaño derivado: 32768 B; método `lzma-alone`.
  DLL importadas: `cygncurses-8.dll`, `cygwin1.dll`, `KERNEL32.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`, `cygncurses-8.dll`.
  Indicio en offset derivado `0x6734`: `Usage: %s [options] [<filename>]`.
- **Ruta:** `CygUtils64.plugin!/bin/sudoku.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `7df5c5cfe99672ad86aa820d1d73c7b09b23c797a9c434259f32fa628ffc08bc`. Hash derivado: `f233686ef80543da3271042b9fbcd89e53bc086f811d43ebf283217dfc5d279b`; tamaño derivado: 37888 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygncursesw-10.dll`, `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x7738`: `Usage: %s [options] [<filename>]`.

### telnet.exe

Cliente Telnet.

- **Ruta:** `CygUtils.plugin!/bin/telnet.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `e4bc122521a9cd07006b567fcf71f3f004ea02e28dc0a4778787657a573cd76a`. Hash derivado: `73464849e95aa70ddd92402766d5628c862aecffc3503bbdf9705cc17470bb18`; tamaño derivado: 96768 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cygncursesw-10.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x13654`: `Usage: send %s <value\|option>`.
  Indicio en offset derivado `0x137f0`: `usage: %s [-4] [-6] [-l user] [-a] host-name [port]`.

### termwalk.exe

Utilidad de terminal; función exacta pendiente.

- **Ruta:** `CygUtils.plugin!/bin/termwalk.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `fd30ab47e731f8b427c720d551f81542b070ef492d66ed204b0dbe76457b2a1a`. Hash derivado: `209ab4832c9e8627b561655e665bee2620d80ae1d219ac3cc89897201c4377c4`; tamaño derivado: 23054 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `cygncursesw-10.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x3f5c`: `Copyright 2020 Mobatek (https://mobaxterm.mobatek.net)`.
  Indicio en offset derivado `0x3f9d`: `usage: %s [-v\|--version] [-h\|--help] [-s ODD_NUMBER\|--grid-size=ODD_NUMBER]`.
- **Ruta:** `CygUtils64.plugin!/bin/termwalk.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `fbcd0cc6754be6c4775757347b903437022957a28efd94bc4c5ecbfe3fa4cd18`. Hash derivado: `3735eec9b74ee349ae40c10d1e8d0953f73b78e7ef52403ddbde1485ed2ac10d`; tamaño derivado: 24576 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `cygncursesw-10.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x4520`: `Copyright 2020 Mobatek (https://mobaxterm.mobatek.net)`.
  Indicio en offset derivado `0x4561`: `usage: %s [-v\|--version] [-h\|--help] [-s ODD_NUMBER\|--grid-size=ODD_NUMBER]`.

### toybox.exe

Ejecutable multicall de utilidades Unix.

- **Ruta:** `CygUtils.plugin!/bin/toybox.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `498eb9ec575ab841a694f587f61d378f4f261e2953c36be37c24c5d2dd99b3f0`. Hash derivado: `8a06c5f1f140a8441a350001d2eec93f32c354603d2879f95be7f1958f950052`; tamaño derivado: 387598 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `cygz.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x40c60`: `usage: toybox [--long \| --help \| --version \| [command] [arguments...]]`.
  Indicio en offset derivado `0x40e1c`: `Show toybox version (only)`.
- **Ruta:** `CygUtils64.plugin!/bin/toybox.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `840f7f4f8a349670a83530d8d944ea9ab290951d4f640307d5e9ccb4daeb5099`. Hash derivado: `e6b72054638df2963d1ed6ae9e6f01660ced68d4f3ecaa247302b39016338a89`; tamaño derivado: 464384 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygcrypto-3.dll`, `cygwin1.dll`, `cygiconv-2.dll`, `cygssl-3.dll`, `cygz.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x5169a`: `usage: toybox [--long \| --help \| --version \| [COMMAND] [ARGUMENTS...]]`.
  Indicio en offset derivado `0x51843`: `Show toybox version (only)`.

### tput.exe

Consulta/control de capacidades de terminal.

- **Ruta:** `CygUtils.plugin!/bin/tput.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `a5dc5d25cfad7786a32fe773ba15ec8d01f15f5473e8461353b00945ff31a01a`. Hash derivado: `2322fe7bca3f03c65932927c701d7e5b5c8566f0872aca268c12a429ffc719de`; tamaño derivado: 18461 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`, `cygncursesw-10.dll`.
  Indicio en offset derivado `0x2860`: `usage: %s [-V] [-S] [-T term] capname`.

### TurboVNC.exe

Componente VNC; función cliente/servidor pendiente de confirmar por código.

- **Ruta:** `CygUtils.plugin!/bin/TurboVNC.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `9ada893fcd300352c30949c8aa2ed361f1d039445f3803ae5019d63fb96b3df5`. Hash derivado: `ca03ed03d00ca1b8885e206af60890aaab5350782766001123a77bafd3660333`; tamaño derivado: 476672 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `WSOCK32.dll`, `GDI32.dll`, `COMDLG32.dll`, `WINMM.dll`, `COMCTL32.dll`, `USER32.dll`, `ADVAPI32.dll`, `SHELL32.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x590e8`: `Connected to RFB server, using protocol version 3.%d`.
  Indicio en offset derivado `0x59120`: `Protocol version negotiated`.

### twm_w32.exe

Gestor de ventanas twm adaptado a Windows.

- **Ruta:** `XW!/bin/twm_w32.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `c16e175aa78b78437b5765c0bf6b733fbbfc1846fb258d64f1bb622d598443b2`. Hash derivado: `c16e175aa78b78437b5765c0bf6b733fbbfc1846fb258d64f1bb622d598443b2`; tamaño derivado: 168462 B; método `direct`.
  **Integridad:** CRC32 declarado por ZIP no coincide; contenido retenido como observado no confiable. No se usa como prueba de integridad ni de comportamiento autorizado.
  DLL importadas: `KERNEL32.dll`, `msvcrt.dll`, `libICE-6.dll`, `libSM-6.dll`, `libX11-6.dll`, `libXext-6.dll`, `libXmu-6.dll`, `libXt-6.dll`.
  Indicio en offset derivado `0x209b0`: `usage:  %s [-display dpy] [-f file] [-s] [-q] [-v] [-clientId id] [-restore file]`.
  Indicio en offset derivado `0x22e98`: `  Unknown pseudo relocation protocol version %d.`.

### umount.exe

Desmontaje de rutas Cygwin.

- **Ruta:** `CygUtils.plugin!/bin/umount.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `16bfbbee87365bbc34dedec412978d6abbed0e8a20eb93b64dbd4918ea71c027`. Hash derivado: `9bb84d0a49635233e8857ba608962502a6640ff45ed74211566da900070b3fd2`; tamaño derivado: 10781 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x1260`: `Usage: %s [OPTION] [<posixpath>]`.
  Indicio en offset derivado `0x1310`: `  -V, --version                 output version information and exit`.

### vim.exe

Editor Vim o wrapper hacia su binario.

- **Ruta:** `CygUtils.plugin!/bin/vim.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `e7a9f50a6cff9ea0b400f88aad6e871ad8670f391c44798073bf84d13cb60efe`. Hash derivado: `8b967b9328911cb846a96e11d241563d94419364add6acda9dbaf55df70c41c8`; tamaño derivado: 1579520 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `cygiconv-2.dll`, `cygintl-8.dll`, `cygncursesw-10.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x164605`: `version 6.0`.
  Indicio en offset derivado `0x168118`: `Conversion with 'charconvert' failed`.
- **Ruta:** `CygUtils64.plugin!/bin/vim.exe`; tipo `script`; arquitectura `no establecida`.
  Hash original: `a8793319aa35d63023b6cc5c43a379a2c4d0a48859c1ccf50495881072b962a3`. Hash derivado: `9cbe0dde20314d43388f772fc0389c5b65294ade09860275e9a4f501b9785cdf`; tamaño derivado: 177 B; método `lzma-alone`.
  Script observado; intérprete `#!/bin/zsh.exe`. Referencias a programas: `/bin/_vim.exe`, `/bin/toybox.exe`, `/bin/zsh.exe`.

### vnp.exe

Utilidad no identificada: necesita análisis adicional.

- **Ruta:** `CygUtils.plugin!/bin/vnp.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `25fbaca992d67b0177eea8b6eea11d897adbf4cd746f6fe47e093eeda7e836d3`. Hash derivado: `5596724d47f3a1eb2b8cc8474845201ff149e8b0c3c17d2077dd365385e87dcc`; tamaño derivado: 13312 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x2644`: `  Unknown pseudo relocation protocol version %d.`.
- **Ruta:** `CygUtils64.plugin!/bin/vnp.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `3d354f96e64415f546cdb08d3e4c9e3ac1f73f7b2e342f9a4008c2a7b87337ed`. Hash derivado: `b0668336026072385c7075cd56fa134ab471a020c97b5d5cd44b6c3fb5f0b953`; tamaño derivado: 16384 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.

### wget.exe

Descarga por HTTP/FTP.

- **Ruta:** `CygUtils.plugin!/bin/wget.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `587253b9b929ceee66e627cca82b22602b99e16b3bf89b9fe42a38012daf69c8`. Hash derivado: `a099e6da73ab370a1bda21f6563f919e7e888284b40937ab879304ef62dd603b`; tamaño derivado: 512014 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygcrypto-3.dll`, `cygwin1.dll`, `cygiconv-2.dll`, `cygssl-3.dll`, `cygz.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x5d6e0`: `Usage: %s [OPTION]... [URL]...`.
  Indicio en offset derivado `0x5d844`: `License GPLv3+: GNU GPL version 3 or later`.

### whois.exe

Consulta de registros WHOIS.

- **Ruta:** `CygUtils.plugin!/bin/whois.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `032ba534109d157b69cd981dc505a54b953f3d12021f90f1cf920261d0d124ef`. Hash derivado: `46cbde18c0366894507248c24bdc13292f5f28e1b2d6c700bb66da9690db967c`; tamaño derivado: 40989 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cygiconv-2.dll`, `cygidn-11.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x4c08`: `Usage: whois [OPTION]... OBJECT...`.
  Indicio en offset derivado `0x4d13`: `      --version        output version information and exit`.

### winpty-agent.exe

Agente de consola de winpty.

- **Ruta:** `CygUtils64.plugin!/bin/winpty-agent.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `2ed0e36a581a5965874c4e21acdef8c8ea5c8ec8fb374e81cd4182640736f365`. Hash derivado: `9e12ecdb8c4d8a2ffd60655148129fb2693c22ce21e7d3a417a58308b30116b8`; tamaño derivado: 343040 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `ADVAPI32.dll`, `KERNEL32.dll`, `msvcrt.dll`, `SHELL32.dll`, `USER32.dll`.
  Indicio en offset derivado `0x41960`: `Usage: %ls controlPipeName flags mouseMode cols rows`.
  Indicio en offset derivado `0x41995`: `Usage: %ls controlPipeName --create-desktop`.

### winpty.exe

Adaptación PTY/console Windows.

- **Ruta:** `CygUtils.plugin!/bin/winpty.exe`; tipo `cygwin-symlink`; arquitectura `no establecida`.
  Hash original: `3427c43830d7839b69cab4d4145f5dc05f76dbc3dafbdfe283a917cc2c14edd4`. Hash derivado: `4df00a6f743a0a1e7e0d0405babaa921c7124068f6daaf4f4ff03b162815b81d`; tamaño derivado: 32 B; método `lzma-alone`.
  Destino declarado del enlace: `conin.exe`; seguir su resolución en el entorno antes de asumir que existe.

### xauth.exe

Gestión de cookies de autenticación X.

- **Ruta:** `CygUtils.plugin!/bin/xauth.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `3c97dbc86c309414b3286a914ca6c122d403d8d2a4c3b29ff8f0e14951fd8b85`. Hash derivado: `069e3be25b3ecbb40438090a5c9eb909d9cf384ddc4c036b9925523bb5cc90d8`; tamaño derivado: 33821 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cygX11-6.dll`, `cygXau-6.dll`, `cygXext-6.dll`, `cygXmuu-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x6068`: `version                        show version number of xauth`.
  Indicio en offset derivado `0x62a0`: `usage:  %s [-options ...] [command arg ...]`.

### xdpyinfo.exe

Consulta de capacidades de servidor X.

- **Ruta:** `CygUtils.plugin!/bin/xdpyinfo.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `af762592a375d3274ece156f27dee4b89a6b9ed6cc90874ac8294d47e51144bb`. Hash derivado: `17c031ff948c8b63b92fe239f285e923392e3d6ea15ad85dc17ebea08d91c3af`; tamaño derivado: 67086 B; método `lzma-alone`.
  DLL importadas: `cygX11-6.dll`, `cygXext-6.dll`, `cygwin1.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`, `cygX11-6.dll`.
  Indicio en offset derivado `0xd8c0`: `%s version %d.%d `.
  Indicio en offset derivado `0xdee0`: `usage:  %s [options]`.

### xev.exe

Visualizador de eventos X.

- **Ruta:** `CygUtils.plugin!/bin/xev.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `d3736acc67b1a1d14ef839bc024037b582cc612bdc70a7055a7915ce608e7555`. Hash derivado: `b89765856470f507c82808f0eb21d5dc7083d85cfe7c30adbbcd6e1b6d43ec19`; tamaño derivado: 20494 B; método `lzma-alone`.
  DLL importadas: `cygX11-6.dll`, `cygwin1.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x3500`: `usage:  %s [-options ...]`.

### xhost.exe

Control de acceso a servidor X.

- **Ruta:** `CygUtils.plugin!/bin/xhost.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `b3b8b735ca41cc217786568798195714f11ce1f60bf0adc6ae16d5817e42177a`. Hash derivado: `ebc4263dea5a956e25c40b586d13fead65e55f209af0ff484faf909f4cdc621d`; tamaño derivado: 12288 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cygX11-6.dll`, `cygXmuu-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x1ed6`: `usage: %s [[+-]hostname ...]`.
- **Ruta:** `XW!/bin/xhost.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `0086d3d6334b0a1a741cd043be6e3498c1ce2f77a0ed5636e19b430ee3310b9c`. Hash derivado: `0086d3d6334b0a1a741cd043be6e3498c1ce2f77a0ed5636e19b430ee3310b9c`; tamaño derivado: 21518 B; método `direct`.
  **Integridad:** CRC32 declarado por ZIP no coincide; contenido retenido como observado no confiable. No se usa como prueba de integridad ni de comportamiento autorizado.
  DLL importadas: `KERNEL32.dll`, `msvcrt.dll`, `WS2_32.dll`, `libX11-6.dll`, `libXmuu-1.dll`.
  Indicio en offset derivado `0x2c9e`: `usage: %s [[+-]hostname ...]`.
  Indicio en offset derivado `0x2fdc`: `  Unknown pseudo relocation protocol version %d.`.

### xkbcomp.exe

Compilador de mapas XKB.

- **Ruta:** `CygUtils.plugin!/bin/xkbcomp.exe`; tipo `script`; arquitectura `no establecida`.
  Hash original: `b7f25a13e043f9831b46c41817635b21bac874050c61d65780ac1ca4ec0d0b22`. Hash derivado: `0b1855f4c458df6379bb56a9cf71116e73ba9554fb9e3b38613d5b1b079188ff`; tamaño derivado: 399 B; método `lzma-alone`.
  Script observado; intérprete `#!/bin/bash.exe`. Referencias a programas: `/bin/_xkbcomp.exe`, `/bin/bash.exe`.

### xkbcomp_w32.exe

Compilador XKB adaptado a Windows.

- **Ruta:** `XW!/bin/xkbcomp_w32.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `6de5719f7663b883d247ca44a7ce9f93e252da1e53998ba7a194f52d5cfe1a45`. Hash derivado: `6de5719f7663b883d247ca44a7ce9f93e252da1e53998ba7a194f52d5cfe1a45`; tamaño derivado: 297998 B; método `direct`.
  **Integridad:** CRC32 declarado por ZIP no coincide; contenido retenido como observado no confiable. No se usa como prueba de integridad ni de comportamiento autorizado.
  DLL importadas: `KERNEL32.dll`, `msvcrt.dll`, `libX11-6.dll`.
  Indicio en offset derivado `0x3a244`: `%s was compiled with XKB version %d.%02d`.
  Indicio en offset derivado `0x3a270`: `X library supports incompatible version %d.%02d`.

### xkbevd.exe

Daemon de eventos XKB.

- **Ruta:** `CygUtils.plugin!/bin/xkbevd.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `4e3810ec398cbf42a33f4646878899a249c0f4c57a5ae16ba6d8ec56c7ed88e0`. Hash derivado: `0a7d71da3af2db9dc8dc0bae051ed9be53b2233d90feb8f4f83711d5115c25e4`; tamaño derivado: 27150 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cygX11-6.dll`, `cygxkbfile-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x582f`: `Usage: %s [options]...`.
  Indicio en offset derivado `0x5c0c`: `%s was compiled with XKB version %d.%02d`.

### xkill.exe

Terminación de clientes X.

- **Ruta:** `CygUtils.plugin!/bin/xkill.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `e19a6774e3ccd8ae992e94076b36cfca372f23b7927284769d4e482c7c8813da`. Hash derivado: `1392a7d81ca79be56384a930ace5acae89ad37d09fbd1cca401409b850f9b41c`; tamaño derivado: 9230 B; método `lzma-alone`.
  DLL importadas: `cygX11-6.dll`, `cygXmuu-1.dll`, `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x19d9`: `usage:  %s [-option ...]`.

### xlsatoms.exe

Listado de átomos X.

- **Ruta:** `CygUtils.plugin!/bin/xlsatoms.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `31859de3fea8bd7a93c51b65b3a2af7e8bb20097b336669ade0978702e311069`. Hash derivado: `e34679d6e32840552291e27a8d6dbbf1e75de26931383c5a3c461e5bad058ecf`; tamaño derivado: 7694 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cygxcb-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x1480`: `usage:  %s [-options...]`.
- **Ruta:** `XW!/bin/xlsatoms.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `ce700f1a3f19f4cf48f2a8a4e072d16cdc9c6e38f5ca9848d76bfcb50617cc94`. Hash derivado: `ce700f1a3f19f4cf48f2a8a4e072d16cdc9c6e38f5ca9848d76bfcb50617cc94`; tamaño derivado: 19982 B; método `direct`.
  **Integridad:** CRC32 declarado por ZIP no coincide; contenido retenido como observado no confiable. No se usa como prueba de integridad ni de comportamiento autorizado.
  DLL importadas: `KERNEL32.dll`, `msvcrt.dll`, `libxcb-1.dll`.
  Indicio en offset derivado `0x2760`: `    -version                print program version`.
  Indicio en offset derivado `0x2793`: `usage:  %s [-options...]`.

### xlsclients.exe

Listado de clientes X.

- **Ruta:** `CygUtils.plugin!/bin/xlsclients.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `c57758ed8380c7b3b26dcf974c6c11eea83880fc65533c4a9bf61f7460b6999f`. Hash derivado: `e559e69f4da357f0d8acf1612e7c24a00275d1e6b703d41c6ffb0f517d1dcc1f`; tamaño derivado: 10254 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cygxcb-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x1ea0`: `usage:  %s  [-display dpy] [-m len] [-[a][l]]`.
- **Ruta:** `XW!/bin/xlsclients.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `959e77d6befb1a21d0bbb4e25c287ae82abd6e62da398b86f29dd6b2ec89b840`. Hash derivado: `959e77d6befb1a21d0bbb4e25c287ae82abd6e62da398b86f29dd6b2ec89b840`; tamaño derivado: 23054 B; método `direct`.
  **Integridad:** CRC32 declarado por ZIP no coincide; contenido retenido como observado no confiable. No se usa como prueba de integridad ni de comportamiento autorizado.
  DLL importadas: `KERNEL32.dll`, `msvcrt.dll`, `libxcb-1.dll`.
  Indicio en offset derivado `0x3064`: `usage:  %s  [-display dpy] [-m len] [-[a][l]] [-version]`.
  Indicio en offset derivado `0x3354`: `  Unknown pseudo relocation protocol version %d.`.

### xlsfonts.exe

Listado de fuentes X.

- **Ruta:** `CygUtils.plugin!/bin/xlsfonts.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `5a221bc7ccea99dc154f8dd49bf4658d97a8bfc2e634fb8474eb67fc62e984e9`. Hash derivado: `3d4fa2424acb9803c893b68da9691a85200305f0683abd8c4c702781ec6f5161`; tamaño derivado: 13838 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cygX11-6.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x29d8`: `usage:  %s [-options] [-fn pattern]`.
- **Ruta:** `XW!/bin/xlsfonts.exe`; tipo `mz-unparsed`; arquitectura `no establecida`.
  Hash original: `72a9833d77a8eb18a5bd6aaa4d63c299609d9f09e9630adb825fa9d9d4171ae3`. Hash derivado: `72a9833d77a8eb18a5bd6aaa4d63c299609d9f09e9630adb825fa9d9d4171ae3`; tamaño derivado: 27150 B; método `direct`.
  **Integridad:** CRC32 declarado por ZIP no coincide; contenido retenido como observado no confiable. No se usa como prueba de integridad ni de comportamiento autorizado.

### xmodmap.exe

Consulta/modificación de mapas de teclas y botones X.

- **Ruta:** `CygUtils.plugin!/bin/xmodmap.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `71b1ef7e474043525721000120c29124100b6c036a5171cc51a562c3c20c55a7`. Hash derivado: `08e78676ff411d59bef85308bf0c0184534e497cce364cad23cf2fb186ca423c`; tamaño derivado: 26624 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cygX11-6.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x4d08`: `usage:  %s [-options ...] [filename]`.
- **Ruta:** `XW!/bin/xmodmap.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `9bc317087d47ce8665f429b8e2696e4f46793afc8a6546116082ff2f846ecbf4`. Hash derivado: `9bc317087d47ce8665f429b8e2696e4f46793afc8a6546116082ff2f846ecbf4`; tamaño derivado: 39438 B; método `direct`.
  **Integridad:** CRC32 declarado por ZIP no coincide; contenido retenido como observado no confiable. No se usa como prueba de integridad ni de comportamiento autorizado.
  DLL importadas: `KERNEL32.dll`, `msvcrt.dll`, `libX11-6.dll`.
  Indicio en offset derivado `0x5e20`: `usage:  %s [-options ...] [filename]`.
  Indicio en offset derivado `0x665e`: `    -version                     print program version`.

### xprop.exe

Consulta/modificación de propiedades X.

- **Ruta:** `CygUtils.plugin!/bin/xprop.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `7e95837cd39f3743417eebd5ac23612cbbc116ee8f0267a06d926afe237c3c5a`. Hash derivado: `3e4fedefa7c9efc7278f57f337a9b80f54108171bf611334e494f1a5e62e6b9a`; tamaño derivado: 27662 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cygX11-6.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x4bf0`: `usage:  %s [-options ...] [[format [dformat]] atom] ...`.
  Indicio en offset derivado `0x503a`: `unsupported conversion for %s`.
- **Ruta:** `XW!/bin/xprop.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `d850a35dad09e720be3eeff72a653ab1bbe4f7e954c552a2951e1fbb9e771ea9`. Hash derivado: `d850a35dad09e720be3eeff72a653ab1bbe4f7e954c552a2951e1fbb9e771ea9`; tamaño derivado: 40974 B; método `direct`.
  **Integridad:** CRC32 declarado por ZIP no coincide; contenido retenido como observado no confiable. No se usa como prueba de integridad ni de comportamiento autorizado.
  DLL importadas: `KERNEL32.dll`, `msvcrt.dll`, `libX11-6.dll`.
  Indicio en offset derivado `0x5d50`: `usage:  %s [-options ...] [[format [dformat]] atom] ...`.
  Indicio en offset derivado `0x619a`: `unsupported conversion for %s`.

### xrdb.exe

Base de recursos X.

- **Ruta:** `CygUtils.plugin!/bin/xrdb.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `c3ae7e5dd41437113d8b9575eeb18a81accc44256de3d9823716e4b051e1fcd7`. Hash derivado: `5eb292f593efafe880441108a0998fc15912a7e5212af6822dda0423c77bbf32`; tamaño derivado: 22557 B; método `lzma-alone`.
  DLL importadas: `cygwin1.dll`, `cygX11-6.dll`, `cygXmuu-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x36a8`: `usage:  %s [-options ...] [filename]`.
  Indicio en offset derivado `0x3712`: ` -version            print the program version`.
- **Ruta:** `XW!/bin/xrdb.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `613b48aa5451c62b2d814a1ca074163e15ad7df0766829ac0830fd8aae96baa0`. Hash derivado: `613b48aa5451c62b2d814a1ca074163e15ad7df0766829ac0830fd8aae96baa0`; tamaño derivado: 33294 B; método `direct`.
  **Integridad:** CRC32 declarado por ZIP no coincide; contenido retenido como observado no confiable. No se usa como prueba de integridad ni de comportamiento autorizado.
  DLL importadas: `KERNEL32.dll`, `msvcrt.dll`, `libX11-6.dll`.
  Indicio en offset derivado `0x4a6c`: `usage:  %s [-options ...] [filename]`.
  Indicio en offset derivado `0x4ad6`: ` -version            print the program version`.

### xwinwm.exe

Integración de ventanas X con Windows.

- **Ruta:** `CygUtils.plugin!/bin/xwinwm.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `e298d92042b72585f1b709fb971498f0f7f72498a92a6c0bc90daccf489d730f`. Hash derivado: `a80bcc084d5e076f6c5a2b35f9784366b1dd9c2d60728f2adec287168474d2f8`; tamaño derivado: 172544 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygWindowsWM-7.dll`, `cygX11-6.dll`, `cygXext-6.dll`, `cygwin1.dll`, `KERNEL32.dll`, `USER32.dll`.
  Indicio en offset derivado `0x282de`: `display version and exit.`.

### zsh.exe

Intérprete de comandos Zsh.

- **Ruta:** `CygUtils.plugin!/bin/zsh.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `191d3f5961a626662f2787a0635fa521859dae57e2c3459ab8c9e992da86b0b0`. Hash derivado: `f7a63a17a316fad3b3fa677a12eee06271356814abaab5fe1556b61896b77c66`; tamaño derivado: 1250304 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `cygncursesw-10.dll`, `cyggcc_s-1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0xf1bfc`: `Usage: %s [<options>] [<argument> ...]`.
  Indicio en offset derivado `0xf1c64`: `  --version  show zsh version number, then exit`.
- **Ruta:** `CygUtils64.plugin!/bin/zsh.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `bd16387fa513deeb05d66271f17e8b91f1384f655dbe631cc87ef37e1a5a708c`. Hash derivado: `beb171617d1aa12e5cab4661c9fb02c9e51851f16acc2404a20f6cb7aa885144`; tamaño derivado: 1186816 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `cygncursesw-10.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0xfb940`: `Usage: %s [<options>] [<argument> ...]`.
  Indicio en offset derivado `0xfb9b0`: `  --version  show zsh version number, then exit`.

### zstd-decompress.exe

Descompresión Zstandard.

- **Ruta:** `CygUtils.plugin!/bin/zstd-decompress.exe`; tipo `pe`; arquitectura `x86`.
  Hash original: `e39e1621a4e21acfdf17bb79ee9669b4e0f5d4d239c1710939c50839b57a5160`. Hash derivado: `0c7353d0734580c1a1a0c95d609840ffc37fab853da8a1bfe47e9f21f73ea6c8`; tamaño derivado: 195086 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x27218`: `Error : incorrect library version (expecting : %s ; actual : %s ) `.
  Indicio en offset derivado `0x2725c`: `Please update library to version %s, or use stand-alone zstd binary `.
- **Ruta:** `CygUtils64.plugin!/bin/zstd-decompress.exe`; tipo `pe`; arquitectura `x86_64`.
  Hash original: `ff88bcdfb48dc78ba975563cadbc86ff6510d0086137b03b946229fb5993ec74`. Hash derivado: `d9dc7d69ed908f253564c120aed08a596df92613da61d128853b22bfd41e9c19`; tamaño derivado: 239616 B; método `lzma-derived-header-byte1-zero`.
  DLL importadas: `cygwin1.dll`, `KERNEL32.dll`.
  Indicio en offset derivado `0x34810`: `Error : incorrect library version (expecting : %s ; actual : %s ) `.
  Indicio en offset derivado `0x34858`: `Please update library to version %s, or use stand-alone zstd binary `.

## DLL, scripts y otros archivos

El inventario JSON conserva todas las entradas regulares de los archivos ZIP examinados y analiza una selección explícita de EXE, DLL, scripts en bin y mobapack. No todos los recursos, documentación o archivos de datos tienen análisis semántico. Las DLL de Cygwin y X proporcionan ABI/plataforma; en Linux se sustituyen por sus bibliotecas nativas, sin deducir compatibilidad por nombre.

## Comprobación de cobertura

Para reproducir el recuento, seleccionar `analyzed_entries` cuyo `path.lower().endswith(".exe")` en `plugins-inventory.json`. El hash del contenedor original y `zip_header_offset` anclan cada entrada; los offsets de cadenas son del contenido derivado. Los miembros de `MobaXterm_installer.dat` son copias de contenedores y no se cuentan otra vez como implementaciones.
