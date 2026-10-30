#!/usr/bin/env python3
"""Synthetic ACP server. Never calls a model or executes tools."""
import json
import os
from pathlib import Path
import sys
assert sys.argv[1:] in (['acp'], ['--acp'], ['--experimental-acp'], [])
assert Path(os.environ['HOME']).is_absolute()
config_options = [dict(id='model', name='Modelo', category='model', type='select', currentValue='fixture/one', options=[dict(value='fixture/one', name='Modelo uno'), dict(value='fixture/two', name='Modelo dos')]), dict(id='mode', name='Modo', category='mode', type='select', currentValue='plan', options=[dict(value='plan', name='plan'), dict(value='build', name='build')])]
def emit(value):
    print(json.dumps(value), flush=True)
def complete(ident):
    emit(dict(id=ident, jsonrpc='2.0', result=dict(stopReason='end_turn')))
turn_id = None
for line in sys.stdin:
    value = json.loads(line)
    with Path('acp-protocol.jsonl').open('a') as log:
        log.write(json.dumps(value) + '\n')
    assert value.get('jsonrpc') == '2.0'
    method, ident, params = value.get('method'), value.get('id'), value.get('params', {})
    result = {}
    if method == 'initialize':
        result = dict(protocolVersion=1, agentCapabilities=(dict(loadSession=True) if Path('support-load').exists() else dict(sessionCapabilities=dict(resume={})) if Path('support-resume').exists() else {}), authMethods=[dict(id='web', name='Web')])
    elif method == 'session/new':
        assert Path(params['cwd']).is_absolute()
        result = dict(sessionId='fixture-session', configOptions=config_options)
    elif method in ('session/load', 'session/resume'):
        assert params['sessionId'] == 'fixture-session'
        assert Path(params['cwd']).is_absolute()
        if Path('load-disconnect').exists():
            sys.exit(0)
        if Path('load-failure').exists():
            emit(dict(id=ident, jsonrpc='2.0', error=dict(code=-32000, message='Fixture failure')))
            continue
        if method == 'session/load':
            emit(dict(jsonrpc='2.0', id='replay-permission', method='session/request_permission', params=dict(sessionId='fixture-session', toolCall=dict(toolCallId='replay-tool', title='Never execute', kind='execute', rawInput=dict(command='false')), options=[dict(kind='allow_once', optionId='yes'),dict(kind='reject_once', optionId='no')])))
            emit(dict(jsonrpc='2.0', method='session/update', params=dict(sessionId='fixture-session', update=dict(sessionUpdate='agent_message_chunk',content=dict(type='text',text='Replayed, not a new message')))))
        result = dict(configOptions=config_options)
    elif method == 'session/set_config_option':
        assert params['value'] == 'fixture/two'
        result = dict(configOptions=config_options)
    elif method == 'session/set_mode':
        assert params['modeId'] in ('plan', 'build')
    elif method == 'session/prompt':
        turn_id = ident
        text = params['prompt'][0]['text'].split('\n\nContexto de trabajo')[0]
        # The first ACP prompt also carries system instructions before the test message.
        trigger = text.rsplit('\n\n', 1)[-1]
        if trigger == 'approval':
            emit(dict(jsonrpc='2.0', id='permission', method='session/request_permission', params=dict(sessionId='fixture-session', toolCall=dict(toolCallId='tool-1', title='Fake command', kind='execute', rawInput=dict(command='printf fixture')), options=[dict(kind='allow_once', optionId='yes', name='Permitir'), dict(kind='reject_once', optionId='no', name='Rechazar')])))
            continue
        if trigger == 'interrupt':
            continue
        answer = Path('markdown-response').read_text() if Path('markdown-response').exists() else 'Respuesta ACP gráfica.'
        middle = len(answer) // 2
        for delta in (answer[:middle], answer[middle:]):
            emit(dict(jsonrpc='2.0', method='session/update', params=dict(sessionId='fixture-session', update=dict(sessionUpdate='agent_message_chunk', content=dict(type='text', text=delta)))))
        complete(ident)
        continue
    elif method == 'session/cancel':
        emit(dict(jsonrpc='2.0', id=turn_id, result=dict(stopReason='cancelled')))
        continue
    elif method is None and ident == 'replay-permission':
        assert value['result']['outcome'] == dict(outcome='cancelled')
        continue
    elif method is None and ident == 'permission':
        assert value['result']['outcome'] in (dict(outcome='selected', optionId='no'), dict(outcome='cancelled'))
        complete(turn_id)
        continue
    if ident is not None:
        emit(dict(id=ident, jsonrpc='2.0', result=result))
