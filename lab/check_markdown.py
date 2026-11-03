#!/usr/bin/env python3
"""Native Markdown widget: visual/source fidelity, undo, recovery and screenshots."""
import os
from pathlib import Path
import subprocess
ROOT = Path(__file__).resolve().parent.parent

def main():
    artifacts = ROOT / 'artifacts'
    artifacts.mkdir(exist_ok=True)
    for theme in ('light', 'dark'):
        env = dict(os.environ, GDK_BACKEND='x11', GSK_RENDERER='cairo',
                   NEXUTERM_MARKDOWN_THEME=theme,
                   NEXUTERM_MARKDOWN_SCREENSHOT=str(artifacts / f'markdown-{theme}.png'))
        result = subprocess.run(['xvfb-run', '-a', 'dbus-run-session', '--', 'cargo',
                                 'test', '--locked', '--bin', 'nexuterm',
                                 'visual_source_undo_and_recovery_preserve_markdown',
                                 '--', '--ignored', '--test-threads=1'], cwd=ROOT,
                                env=env, capture_output=True, text=True, timeout=180)
        log = result.stdout + result.stderr
        (artifacts / f'markdown-{theme}.log').write_text(log)
        assert result.returncode == 0 and '1 passed' in log and 'CRITICAL' not in log, log
        assert (artifacts / f'markdown-{theme}.png').is_file()
        print(f'PASS Markdown {theme}: hidden markup, exact source, unchanged dirty state, formatting, undo and recoverable text')

if __name__ == '__main__':
    main()
