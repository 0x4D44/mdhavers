// mdhavers Playground - Main Application

import init, { run, check, format, compile_to_js, version } from './pkg/mdhavers_playground.js';

// Example code snippets
const EXAMPLES = {
    hello: `// Hello World in mdhavers
blether "Hullo, World!"
blether "Fae Scotland wi' love 🏴󠁧󠁢󠁳󠁣󠁴󠁿"`,

    variables: `// Variables in mdhavers
ken name = "Hamish"
ken age = 25
ken is_scottish = aye

blether f"Ma name is {name}"
blether f"Ah'm {age} years auld"

gin is_scottish {
    blether "An' ah'm proud tae be Scottish!"
}`,

    control: `// Control flow in mdhavers
ken weather = "rainy"

// Conditional statements
gin weather == "sunny" {
    blether "Get yer sunglasses!"
} ither gin weather == "rainy" {
    blether "Dinnae forget yer brolly!"
} ither {
    blether "Just dress fer anything!"
}

// Loops
blether "\\nCoontin' tae 5:"
fer i in 0..5 {
    blether f"  {i + 1}"
}

// While loop
blether "\\nCoontin' doon:"
ken n = 3
whiles n > 0 {
    blether f"  {n}"
    ken n = n - 1
}
blether "Blast aff!"`,

    functions: `// Functions in mdhavers
dae greet(name) {
    gie f"Hullo, {name}! How're ye daein'?"
}

dae factorial(n) {
    gin n <= 1 {
        gie 1
    }
    gie n * factorial(n - 1)
}

dae is_even(n) {
    gie n % 2 == 0
}

blether greet("Angus")
blether f"5! = {factorial(5)}"
blether f"Is 42 even? {is_even(42)}"`,

    lists: `// Lists and Dictionaries in mdhavers

// Lists
ken numbers = [1, 2, 3, 4, 5]
blether f"Numbers: {numbers}"
blether f"First: {numbers[0]}"
blether f"Last: {numbers[-1]}"
blether f"Length: {len(numbers)}"

// Add to list
shove(numbers, 6)
blether f"After shove: {numbers}"

// Dictionaries
ken person = {
    "name": "Morag",
    "age": 30,
    "city": "Edinburgh"
}

blether f"\\nPerson: {person}"
blether f"Name: {person[\"name\"]}"

// Iterate over list
blether "\\nSquares:"
fer n in numbers {
    blether f"  {n}^2 = {n * n}"
}`,

    classes: `// Classes in mdhavers

kin Animal {
    dae init(name) {
        masel.name = name
    }

    dae speak() {
        blether f"{masel.name} makes a sound"
    }
}

kin Dog frae Animal {
    dae init(name, breed) {
        auld.init(name)
        masel.breed = breed
    }

    dae speak() {
        blether f"{masel.name} the {masel.breed} says: Woof!"
    }

    dae fetch() {
        blether f"{masel.name} fetches the ball!"
    }
}

ken rover = Dog("Rover", "Border Collie")
rover.speak()
rover.fetch()`,

    functional: `// Functional programming in mdhavers

ken numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

// Map - transform each element
ken doubled = gaun(numbers, |x| x * 2)
blether f"Doubled: {doubled}"

// Filter - keep matching elements
ken evens = sieve(numbers, |x| x % 2 == 0)
blether f"Evens: {evens}"

// Reduce - combine all elements
ken sum = tumble(numbers, 0, |acc, x| acc + x)
blether f"Sum: {sum}"

// Pipe operations together
ken result = numbers
    |> sieve(|x| x > 3)
    |> gaun(|x| x * x)
    |> tumble(0, |a, b| a + b)

blether f"\\nSum of squares of numbers > 3: {result}"`,

    fibonacci: `// Fibonacci sequence in mdhavers

dae fib(n) {
    gin n <= 1 {
        gie n
    }
    gie fib(n - 1) + fib(n - 2)
}

blether "Fibonacci sequence:"
fer i in 0..12 {
    blether f"  fib({i}) = {fib(i)}"
}

// Iterative version (faster)
dae fib_fast(n) {
    gin n <= 1 {
        gie n
    }
    ken a = 0
    ken b = 1
    fer _ in 2..(n + 1) {
        ken temp = a + b
        ken a = b
        ken b = temp
    }
    gie b
}

blether f"\\nfib_fast(30) = {fib_fast(30)}"`,

    fizzbuzz: `// FizzBuzz in mdhavers

dae fizzbuzz(n) {
    fer i in 1..(n + 1) {
        gin i % 15 == 0 {
            blether "FizzBuzz"
        } ither gin i % 3 == 0 {
            blether "Fizz"
        } ither gin i % 5 == 0 {
            blether "Buzz"
        } ither {
            blether tae_string(i)
        }
    }
}

blether "FizzBuzz 1 tae 20:"
fizzbuzz(20)`
};

// DOM elements
let editor, output, status, lineNumbers;
let runBtn, formatBtn, compileBtn, clearBtn, shareBtn, examplesSelect;
let compileModal, compiledCode, closeModal, copyCompiledBtn;
let toast;

// App state
let wasmReady = false;

// Initialize the application
async function initApp() {
    // Get DOM elements
    editor = document.getElementById('editor');
    output = document.getElementById('output');
    status = document.getElementById('status');
    lineNumbers = document.getElementById('lineNumbers');
    runBtn = document.getElementById('runBtn');
    formatBtn = document.getElementById('formatBtn');
    compileBtn = document.getElementById('compileBtn');
    clearBtn = document.getElementById('clearBtn');
    shareBtn = document.getElementById('shareBtn');
    examplesSelect = document.getElementById('examples');
    compileModal = document.getElementById('compileModal');
    compiledCode = document.getElementById('compiledCode');
    closeModal = document.getElementById('closeModal');
    copyCompiledBtn = document.getElementById('copyCompiledBtn');
    toast = document.getElementById('toast');

    // Set up event listeners
    setupEventListeners();

    // Initialize WASM
    try {
        await init();
        wasmReady = true;
        setStatus('Ready', 'ready');
        console.log(`mdhavers Playground v${version()} loaded`);
    } catch (error) {
        setStatus('WASM Error', 'error');
        showError('Failed to load mdhavers WASM module: ' + error.message);
        console.error('WASM init error:', error);
    }

    // Check for shared code in URL
    loadFromUrl();

    // Set initial code if editor is empty
    if (!editor.value.trim()) {
        editor.value = EXAMPLES.hello;
    }

    // Update line numbers
    updateLineNumbers();
}

function setupEventListeners() {
    // Button clicks
    runBtn.addEventListener('click', runCode);
    formatBtn.addEventListener('click', formatCode);
    compileBtn.addEventListener('click', compileCode);
    clearBtn.addEventListener('click', clearOutput);
    shareBtn.addEventListener('click', shareCode);

    // Example selection
    examplesSelect.addEventListener('change', loadExample);

    // Editor events
    editor.addEventListener('input', updateLineNumbers);
    editor.addEventListener('scroll', syncScroll);
    editor.addEventListener('keydown', handleEditorKeydown);

    // Modal events
    closeModal.addEventListener('click', () => compileModal.classList.remove('show'));
    compileModal.addEventListener('click', (e) => {
        if (e.target === compileModal) compileModal.classList.remove('show');
    });
    copyCompiledBtn.addEventListener('click', copyCompiledCode);

    // Keyboard shortcuts
    document.addEventListener('keydown', handleGlobalKeydown);
}

function handleEditorKeydown(e) {
    // Tab key - insert spaces instead of changing focus
    if (e.key === 'Tab') {
        e.preventDefault();
        const start = editor.selectionStart;
        const end = editor.selectionEnd;
        editor.value = editor.value.substring(0, start) + '    ' + editor.value.substring(end);
        editor.selectionStart = editor.selectionEnd = start + 4;
        updateLineNumbers();
    }
}

function handleGlobalKeydown(e) {
    // Ctrl+Enter - Run
    if (e.ctrlKey && e.key === 'Enter') {
        e.preventDefault();
        runCode();
    }
    // Ctrl+Shift+F - Format
    if (e.ctrlKey && e.shiftKey && e.key === 'F') {
        e.preventDefault();
        formatCode();
    }
    // Escape - Close modal
    if (e.key === 'Escape') {
        compileModal.classList.remove('show');
    }
}

function updateLineNumbers() {
    const lines = editor.value.split('\n');
    lineNumbers.innerHTML = lines.map((_, i) => i + 1).join('\n');
}

function syncScroll() {
    lineNumbers.scrollTop = editor.scrollTop;
}

function setStatus(text, type = 'ready') {
    status.textContent = text;
    status.className = 'status ' + type;
}

function showToast(message, type = 'success') {
    toast.textContent = message;
    toast.className = 'toast show ' + type;
    setTimeout(() => {
        toast.className = 'toast';
    }, 3000);
}

function showOutput(lines, result = null, timing = null) {
    output.innerHTML = '';

    // Show output lines
    lines.forEach(line => {
        const div = document.createElement('div');
        div.className = 'output-line output-text';
        div.textContent = line;
        output.appendChild(div);
    });

    // Show result
    if (result !== null && result !== 'naething' && result !== '') {
        const div = document.createElement('div');
        div.className = 'output-line result';
        div.textContent = `=> ${result}`;
        output.appendChild(div);
    }

    // Show timing
    if (timing !== null) {
        const div = document.createElement('div');
        div.className = 'output-line timing';
        div.textContent = `Executed in ${timing.toFixed(2)}ms`;
        output.appendChild(div);
    }
}

function showError(message, scotsHeader = null) {
    output.innerHTML = '';

    if (scotsHeader) {
        const header = document.createElement('div');
        header.className = 'output-line error-header';
        header.textContent = scotsHeader;
        output.appendChild(header);
    }

    const div = document.createElement('div');
    div.className = 'output-line error';
    div.textContent = message;
    output.appendChild(div);
}

function runCode() {
    if (!wasmReady) {
        showError('WASM module not ready yet');
        return;
    }

    const code = editor.value;
    if (!code.trim()) {
        showError("Och, there's naething tae run! Write some code first.");
        return;
    }

    setStatus('Running...', 'running');

    try {
        const startTime = performance.now();
        const resultJson = run(code);
        const endTime = performance.now();
        const result = JSON.parse(resultJson);

        if (result.success) {
            setStatus('Success', 'success');
            showOutput(result.output, result.result, endTime - startTime);
        } else {
            setStatus('Error', 'error');
            showError(result.error, "Och naw! Something's gone wrang:");
        }
    } catch (error) {
        setStatus('Error', 'error');
        showError('Unexpected error: ' + error.message);
        console.error('Run error:', error);
    }
}

function formatCode() {
    if (!wasmReady) {
        showError('WASM module not ready yet');
        return;
    }

    const code = editor.value;
    if (!code.trim()) {
        return;
    }

    try {
        const formatted = format(code);
        editor.value = formatted;
        updateLineNumbers();
        showToast('Code formatted!');
    } catch (error) {
        showError('Format error: ' + error.message);
    }
}

function compileCode() {
    if (!wasmReady) {
        showError('WASM module not ready yet');
        return;
    }

    const code = editor.value;
    if (!code.trim()) {
        showError("Och, there's naething tae compile!");
        return;
    }

    try {
        const resultJson = compile_to_js(code);
        const result = JSON.parse(resultJson);

        if (result.success) {
            compiledCode.textContent = result.code;
            compileModal.classList.add('show');
        } else {
            showError(result.error, "Compilation failed:");
        }
    } catch (error) {
        showError('Compile error: ' + error.message);
    }
}

function copyCompiledCode() {
    navigator.clipboard.writeText(compiledCode.textContent).then(() => {
        showToast('Copied to clipboard!');
    }).catch(() => {
        showToast('Failed to copy', 'error');
    });
}

function clearOutput() {
    output.innerHTML = '';
    setStatus('Ready', 'ready');
}

function loadExample() {
    const example = examplesSelect.value;
    if (example && EXAMPLES[example]) {
        editor.value = EXAMPLES[example];
        updateLineNumbers();
        examplesSelect.value = '';
        clearOutput();
    }
}

function shareCode() {
    const code = editor.value;
    if (!code.trim()) {
        showToast('Nothing to share!', 'error');
        return;
    }

    try {
        const encoded = btoa(encodeURIComponent(code));
        const url = new URL(window.location.href);
        url.searchParams.set('code', encoded);

        navigator.clipboard.writeText(url.toString()).then(() => {
            showToast('Share URL copied to clipboard!');
        }).catch(() => {
            // Fallback
            prompt('Copy this URL to share:', url.toString());
        });
    } catch (error) {
        showToast('Failed to create share URL', 'error');
    }
}

function loadFromUrl() {
    const params = new URLSearchParams(window.location.search);
    const encoded = params.get('code');

    if (encoded) {
        try {
            const code = decodeURIComponent(atob(encoded));
            editor.value = code;
            updateLineNumbers();
        } catch (error) {
            console.error('Failed to decode URL code:', error);
        }
    }
}

// Initialize on DOM ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initApp);
} else {
    initApp();
}
