#!/usr/bin/env python3
"""Exercise real local Whisper and GTK with a WAV fixture; never opens the microphone."""
import argparse
import os
from pathlib import Path
import shutil
import tempfile
import wave
from check_ai import ROOT, ARTIFACTS, run

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('audio', type=Path, help='Synthetic or approved test audio, mono PCM16 at 16 kHz')
args = parser.parse_args()
audio = args.audio.resolve(strict=True)
with wave.open(str(audio)) as sample:
    assert (sample.getnchannels(), sample.getsampwidth(), sample.getframerate()) == (1, 2, 16000)
    assert sample.getnframes() <= 16000 * 60
assert (ROOT / '.local/whisper/bin/whisper-cli').is_file()
assert (ROOT / '.local/whisper/models/ggml-tiny.bin').is_file()
ARTIFACTS.mkdir(exist_ok=True)
with tempfile.TemporaryDirectory(prefix='nexuterm-dictation-ui-') as temporary:
    directory = Path(temporary)
    binary = directory / 'bin'
    binary.mkdir()
    python = shutil.which('python3')
    recorder = binary / 'pw-record'
    recorder.write_text(f'#!{python}\nimport pathlib,shutil,signal,sys,time\n'
                        f'shutil.copyfile({str(audio)!r},sys.argv[-1])\n'
                        'signal.signal(signal.SIGINT,lambda *_:sys.exit(0))\n'
                        'while True:time.sleep(.02)\n')
    cli = binary / 'codex'
    cli.write_text(f'#!{python}\n' + (ROOT / 'lab/codex_fixture.py').read_text())
    for executable in (recorder, cli):
        executable.chmod(0o700)
    os.environ['PATH'] = str(binary) + os.pathsep + os.environ['PATH']
    assert Path(shutil.which('pw-record')) == recorder, 'Refusing to use a real microphone'
    config, logs = run('dictation', dict(id='dictation', name='Codex', kind='Codex', program=str(cli), directory=str(directory)), directory)
    assert 'NEXUTERM_SMOKE_DICTATION_LOCAL_OK' in logs, logs
    assert not config['preferences'].get('dictation_model'), 'Test must exercise automatic discovery'
    print('PASS real local Whisper: automatic discovery, synthetic WAV, stop, transcription inserted in GTK draft; no microphone or external API')
