const vscode = require('vscode');
const { LanguageClient, TransportKind } = require('vscode-languageclient/node');

let client;

function activate(context) {
    const serverOptions = {
        command: 'parvan',
        args: ['--lsp'],
        transport: TransportKind.stdio
    };

    const clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'parvan' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.prv')
        }
    };

    client = new LanguageClient(
        'parvanLsp',
        'Parvan Language Server',
        serverOptions,
        clientOptions
    );

    client.start();
    console.log('Parvan Language Server Client activated successfully.');
}

function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}

module.exports = {
    activate,
    deactivate
};
