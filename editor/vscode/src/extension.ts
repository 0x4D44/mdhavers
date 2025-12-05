// mdhavers VS Code Extension - Gie yer editor some Scots smarts!

import * as path from 'path';
import * as vscode from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export function activate(context: vscode.ExtensionContext) {
    const config = vscode.workspace.getConfiguration('mdhavers');

    // Check if LSP is enabled
    if (!config.get<boolean>('lsp.enable', true)) {
        console.log('mdhavers LSP is disabled');
        return;
    }

    // Get the path to the LSP server
    let serverPath = config.get<string>('lsp.path', 'mdhavers-lsp');

    // If it's a relative path, try to find it relative to the extension
    if (!path.isAbsolute(serverPath)) {
        // Try common locations
        const possiblePaths = [
            serverPath,  // PATH lookup
            path.join(context.extensionPath, '..', '..', '..', 'target', 'release', 'mdhavers-lsp'),
            path.join(context.extensionPath, '..', '..', '..', 'target', 'debug', 'mdhavers-lsp'),
        ];

        // Use the first one that exists, or default to PATH lookup
        serverPath = possiblePaths[0];
    }

    // Server options - run the mdhavers-lsp binary
    const serverOptions: ServerOptions = {
        run: {
            command: serverPath,
            transport: TransportKind.stdio
        },
        debug: {
            command: serverPath,
            transport: TransportKind.stdio
        }
    };

    // Client options
    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'mdhavers' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.braw')
        }
    };

    // Create and start the language client
    client = new LanguageClient(
        'mdhavers',
        'mdhavers Language Server',
        serverOptions,
        clientOptions
    );

    // Start the client (this also launches the server)
    client.start();

    console.log('mdhavers language server started! Aw the best tae ye!');
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
