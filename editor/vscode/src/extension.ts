// mdhavers VS Code Extension - Gie yer editor some Scots smarts!

import * as fs from 'fs';
import * as path from 'path';
import * as vscode from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

function firstExistingPath(paths: string[]): string | undefined {
    for (const candidate of paths) {
        if (fs.existsSync(candidate)) {
            return candidate;
        }
    }
    return undefined;
}

export function activate(context: vscode.ExtensionContext) {
    const config = vscode.workspace.getConfiguration('mdhavers');

    // Check if LSP is enabled
    if (!config.get<boolean>('lsp.enable', true)) {
        console.log('mdhavers LSP is disabled');
        return;
    }

    // Get the path to the LSP server
    const lspBinaryName = process.platform === 'win32' ? 'mdhavers-lsp.exe' : 'mdhavers-lsp';
    let serverPath = config.get<string>('lsp.path', lspBinaryName);

    // If it's a relative path, try to find it relative to the extension
    if (!path.isAbsolute(serverPath)) {
        const workspaceRoots = (vscode.workspace.workspaceFolders ?? []).map((folder) => folder.uri.fsPath);
        const devRepoRootGuess = path.resolve(context.extensionPath, '..', '..', '..');
        const searchRoots = Array.from(new Set([...workspaceRoots, devRepoRootGuess]));

        const possiblePaths: string[] = [];

        // User-provided path (if it looks like a path, not a bare command)
        const looksLikePath = serverPath.includes('/') || serverPath.includes('\\') || serverPath.startsWith('.');
        if (looksLikePath) {
            for (const root of searchRoots) {
                possiblePaths.push(path.resolve(root, serverPath));
            }
        }

        // Common local-dev build outputs (prefer these over PATH)
        for (const root of searchRoots) {
            possiblePaths.push(path.join(root, 'target', 'release', lspBinaryName));
            possiblePaths.push(path.join(root, 'target', 'debug', lspBinaryName));
        }

        const foundPath = firstExistingPath(possiblePaths);
        if (foundPath) {
            serverPath = foundPath;
        }
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
