#!/usr/bin/env python3
"""Disposable Codex app-server fixture: no model, browser, network or credentials."""
import json
from pathlib import Path
import sys
assert sys.argv[1:] == ['app-server', '--listen', 'stdio://']
account = not Path('unauthenticated').exists()
turn = 0
approval_index = 0

def emit(value):
    print(json.dumps(value), flush=True)

def notify(method, params):
    emit(dict(method=method, params=params))

for line in sys.stdin:
    message = json.loads(line)
    with Path('protocol.jsonl').open('a') as log:
        log.write(json.dumps(message) + '\n')
    method = message.get('method')
    ident = message.get('id')
    params = message.get('params', {})
    result = {}
    if method == 'account/read':
        result = dict(account=dict(type='chatgpt', email='fixture@example.invalid', planType='free') if account else None, requiresOpenaiAuth=True)
    elif method == 'account/logout':
        account = False
        emit(dict(id=ident, result={}))
        notify('account/updated', dict(authMode=None, planType=None))
        continue
    elif method == 'account/login/start':
        assert params == dict(type='chatgpt', useHostedLoginSuccessPage=True)
        emit(dict(id=ident, result=dict(type='chatgpt', loginId='fixture-login', authUrl='https://example.invalid/login')))
        account = True
        notify('account/login/completed', dict(loginId='fixture-login', success=True))
        continue
    elif method == 'account/rateLimits/read':
        if Path('no-rate-limits').exists():
            emit(dict(id=ident,error=dict(code=-32601,message='not supported')));continue
        result = dict(rateLimits=dict(primary=dict(usedPercent=25+turn,windowDurationMins=300,resetsAt=1800000000),secondary=dict(usedPercent=60+turn,windowDurationMins=10080,resetsAt=1800500000)))
    elif method == 'model/list':
        result = dict(data=[dict(id='fixture', model='fixture-model', displayName='Modelo de prueba', supportedReasoningEfforts=[dict(reasoningEffort='low', description='Ligero'),dict(reasoningEffort='high', description='Alto')])])
    elif method == 'thread/list':
        result = dict(data=[dict(id='old-thread', name='Chat anterior de prueba', preview='Consulta anterior')])
    elif method == 'thread/resume':
        result = dict(thread=dict(id='fixture-thread', turns=[dict(items=[dict(type='userMessage', content=[dict(type='text', text='Consulta anterior')]),dict(type='agentMessage', text='Respuesta anterior')])]))
    elif method == 'thread/start':
        assert params['sandbox'] in ('read-only', 'workspace-write')
        assert params['approvalPolicy'] == 'untrusted'
        result = dict(thread=dict(id='fixture-thread'))
    elif method == 'turn/start':
        assert params['threadId'] == 'fixture-thread'
        turn += 1
        emit(dict(id=ident, result=dict(turn=dict(id=f'turn-{turn}'))))
        text = params['input'][0]['text']
        if text == 'approval':
            approval_index = 1
            item = dict(type='commandExecution', id='tool', command='printf fixture', cwd=str(Path.cwd()), status='inProgress')
            notify('item/started', dict(item=item))
            emit(dict(id='approval-1', method='item/commandExecution/requestApproval', params=dict(itemId='tool', command='printf fixture', cwd=str(Path.cwd()))))
            continue
        if text == 'interrupt':
            continue
        for delta in ('Respuesta ', 'gráfica de Codex.'):
            notify('item/agentMessage/delta', dict(itemId=f'message-{turn}', delta=delta))
        notify('item/completed', dict(item=dict(type='agentMessage', id=f'message-{turn}', text='Respuesta gráfica de Codex.')))
        notify('thread/tokenUsage/updated',dict(tokenUsage=dict(last=dict(totalTokens=1024),modelContextWindow=4096)))
        notify('account/rateLimits/updated',dict(rateLimits=dict(primary=dict(usedPercent=25+turn,windowDurationMins=300),secondary=dict(usedPercent=60+turn,windowDurationMins=10080))))
        notify('turn/completed', dict(turn=dict(id=f'turn-{turn}', status='completed')))
        continue
    elif method == 'turn/interrupt':
        notify('turn/completed', dict(turn=dict(id=f'turn-{turn}', status='interrupted')))
    elif method is None and isinstance(ident, str) and ident.startswith('approval-'):
        assert message['result']['decision'] in ('decline', 'accept')
        notify('serverRequest/resolved', dict(requestId=ident))
        if Path('repeat-approval').exists() and approval_index < 2:
            approval_index += 1
            emit(dict(id=f'approval-{approval_index}', method='item/commandExecution/requestApproval', params=dict(itemId='tool', command='printf fixture', cwd=str(Path.cwd()))))
        else:
            notify('turn/completed', dict(turn=dict(id=f'turn-{turn}', status='completed')))
        continue
    if ident is not None:
        emit(dict(id=ident, result=result))
