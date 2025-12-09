"use strict";
// mdhavers VS Code Extension - Gie yer editor some Scots smarts!
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const path = __importStar(require("path"));
const vscode = __importStar(require("vscode"));
const node_1 = require("vscode-languageclient/node");
let client;
function activate(context) {
    const config = vscode.workspace.getConfiguration('mdhavers');
    // Check if LSP is enabled
    if (!config.get('lsp.enable', true)) {
        console.log('mdhavers LSP is disabled');
        return;
    }
    // Get the path to the LSP server
    let serverPath = config.get('lsp.path', 'mdhavers-lsp');
    // If it's a relative path, try to find it relative to the extension
    if (!path.isAbsolute(serverPath)) {
        // Try common locations
        const possiblePaths = [
            serverPath, // PATH lookup
            path.join(context.extensionPath, '..', '..', '..', 'target', 'release', 'mdhavers-lsp'),
            path.join(context.extensionPath, '..', '..', '..', 'target', 'debug', 'mdhavers-lsp'),
        ];
        // Use the first one that exists, or default to PATH lookup
        serverPath = possiblePaths[0];
    }
    // Server options - run the mdhavers-lsp binary
    const serverOptions = {
        run: {
            command: serverPath,
            transport: node_1.TransportKind.stdio
        },
        debug: {
            command: serverPath,
            transport: node_1.TransportKind.stdio
        }
    };
    // Client options
    const clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'mdhavers' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.braw')
        }
    };
    // Create and start the language client
    client = new node_1.LanguageClient('mdhavers', 'mdhavers Language Server', serverOptions, clientOptions);
    // Start the client (this also launches the server)
    client.start();
    console.log('mdhavers language server started! Aw the best tae ye!');
}
function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
//# sourceMappingURL=extension.js.map