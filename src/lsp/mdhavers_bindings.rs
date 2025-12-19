//! Bindings tae the mdhavers parser fer the LSP
//!
//! This module provides the interface between the LSP server
//! and the mdhavers language implementation.

use logos::Logos;

/// Get diagnostics fer a piece o' mdhavers code
/// Returns a list of (line, column, message, severity)
pub fn get_diagnostics(source: &str) -> Vec<(usize, usize, String, String)> {
    let mut diagnostics = Vec::new();

    // Try tae lex the source first
    let mut lexer = TokenKind::lexer(source);
    let mut line: usize = 1;
    let mut col: usize = 1;

    while let Some(token) = lexer.next() {
        let slice = lexer.slice();

        // Update line/column tracking
        for c in slice.chars() {
            if c == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }

        if token.is_err() {
            diagnostics.push((
                line,
                col.saturating_sub(slice.len()),
                format!("Och! Ah dinnae ken whit '{}' is", slice),
                "error".to_string(),
            ));
        }
    }

    // Now try to parse and collect more errors
    if let Err(parse_error) = parse_for_errors(source) {
        diagnostics.push(parse_error);
    }

    diagnostics
}

/// A simplified token enum fer lexing (mirrors the main lexer)
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r]+")]
enum TokenKind {
    // Keywords
    #[token("ken")]
    Ken,
    #[token("gin")]
    Gin,
    #[token("ither")]
    Ither,
    #[token("than")]
    Than,
    #[token("whiles")]
    Whiles,
    #[token("fer")]
    Fer,
    #[token("gie")]
    Gie,
    #[token("blether")]
    Blether,
    #[token("speir")]
    Speir,
    #[token("fae")]
    Fae,
    #[token("tae")]
    Tae,
    #[token("an")]
    An,
    #[token("or")]
    Or,
    #[token("nae")]
    Nae,
    #[token("aye")]
    Aye,
    #[token("naething")]
    Naething,
    #[token("dae")]
    Dae,
    #[token("thing")]
    Thing,
    #[token("fetch")]
    Fetch,
    #[token("kin")]
    Kin,
    #[token("brak")]
    Brak,
    #[token("haud")]
    Haud,
    #[token("in")]
    In,
    #[token("is")]
    Is,
    #[token("masel")]
    Masel,
    #[token("hae_a_bash")]
    HaeABash,
    #[token("gin_it_gangs_wrang")]
    GinItGangsWrang,
    #[token("keek")]
    Keek,
    #[token("whan")]
    Whan,
    #[token("mak_siccar")]
    MakSiccar,

    // Literals
    #[regex(r"[0-9]+")]
    Integer,
    #[regex(r"[0-9]+\.[0-9]+")]
    Float,
    #[regex(r#""([^"\\]|\\.)*""#)]
    String,
    #[regex(r#"'([^'\\]|\\.)*'"#)]
    SingleQuoteString,
    #[regex(r#"f"([^"\\]|\\.)*""#)]
    FString,
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    // Operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("=")]
    Equals,
    #[token("==")]
    EqualsEquals,
    #[token("!=")]
    BangEquals,
    #[token("<")]
    Less,
    #[token("<=")]
    LessEquals,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterEquals,
    #[token("!")]
    Bang,
    #[token("+=")]
    PlusEquals,
    #[token("-=")]
    MinusEquals,
    #[token("*=")]
    StarEquals,
    #[token("/=")]
    SlashEquals,
    #[token("...")]
    DotDotDot,
    #[token("..")]
    DotDot,
    #[token(".")]
    Dot,
    #[token("_", priority = 3)]
    Underscore,

    // Delimiters
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token("[")]
    LeftBracket,
    #[token("]")]
    RightBracket,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token(";")]
    Semicolon,
    #[token("->")]
    Arrow,
    #[token("|>")]
    PipeForward,
    #[token("|")]
    Pipe,
    #[token("\n")]
    Newline,
    #[regex(r"#[^\n]*", logos::skip)]
    Comment,
}

/// Simple parser fer error detection
fn parse_for_errors(source: &str) -> Result<(), (usize, usize, String, String)> {
    let mut brace_stack: Vec<(char, usize, usize)> = Vec::new();
    let mut line = 1;
    let mut col = 1;

    for ch in source.chars() {
        match ch {
            '{' | '(' | '[' => {
                brace_stack.push((ch, line, col));
            }
            '}' => {
                if let Some((open, _, _)) = brace_stack.pop() {
                    if open != '{' {
                        return Err((
                            line,
                            col,
                            format!("Unexpected '}}' - was expectin' a match fer '{}'", open),
                            "error".to_string(),
                        ));
                    }
                } else {
                    return Err((
                        line,
                        col,
                        "Unexpected '}}' - nae matchin' '{{' found".to_string(),
                        "error".to_string(),
                    ));
                }
            }
            ')' => {
                if let Some((open, _, _)) = brace_stack.pop() {
                    if open != '(' {
                        return Err((
                            line,
                            col,
                            format!("Unexpected ')' - was expectin' a match fer '{}'", open),
                            "error".to_string(),
                        ));
                    }
                } else {
                    return Err((
                        line,
                        col,
                        "Unexpected ')' - nae matchin' '(' found".to_string(),
                        "error".to_string(),
                    ));
                }
            }
            ']' => {
                if let Some((open, _, _)) = brace_stack.pop() {
                    if open != '[' {
                        return Err((
                            line,
                            col,
                            format!("Unexpected ']' - was expectin' a match fer '{}'", open),
                            "error".to_string(),
                        ));
                    }
                } else {
                    return Err((
                        line,
                        col,
                        "Unexpected ']' - nae matchin' '[' found".to_string(),
                        "error".to_string(),
                    ));
                }
            }
            '\n' => {
                line += 1;
                col = 0;
            }
            _ => {}
        }
        col += 1;
    }

    // Check for unclosed brackets
    if let Some((ch, l, c)) = brace_stack.pop() {
        let close = match ch {
            '{' => '}',
            '(' => ')',
            '[' => ']',
            _ => '?',
        };
        return Err((
            l,
            c,
            format!("Unclosed '{}' - missin' '{}'", ch, close),
            "error".to_string(),
        ));
    }

    Ok(())
}

/// Get documentation fer a keyword or builtin
pub fn get_keyword_info(keyword: &str) -> Option<String> {
    match keyword {
        // Keywords
        "ken" => Some("**ken** - Variable declaration\n\n```mdhavers\nken x = 42\nken name = \"Angus\"\n```\n\nLike `let` or `var` in other languages. From Scots \"I ken\" meaning \"I know\".".to_string()),
        "gin" => Some("**gin** - If statement\n\n```mdhavers\ngin x > 10 {\n    blether \"Big number!\"\n}\n```\n\nConditional execution. From Scots \"gin\" meaning \"if\".".to_string()),
        "ither" => Some("**ither** - Else clause\n\n```mdhavers\ngin x > 10 {\n    blether \"Big\"\n} ither {\n    blether \"Wee\"\n}\n```\n\nFrom Scots \"ither\" meaning \"other\".".to_string()),
        "than" => Some("**than** - Then (for ternary expressions)\n\n```mdhavers\nken result = gin x > 0 than \"positive\" ither \"negative\"\n```\n\nUsed in ternary/conditional expressions.".to_string()),
        "whiles" => Some("**whiles** - While loop\n\n```mdhavers\nwhiles x < 10 {\n    blether x\n    x = x + 1\n}\n```\n\nFrom Scots \"whiles\" meaning \"while\".".to_string()),
        "fer" => Some("**fer** - For loop\n\n```mdhavers\nfer i in 1..10 {\n    blether i\n}\n\nfer item in my_list {\n    blether item\n}\n```\n\nIterate over ranges or collections.".to_string()),
        "dae" => Some("**dae** - Function definition\n\n```mdhavers\ndae greet(name) {\n    blether f\"Hullo {name}!\"\n}\n\ndae add(a, b = 0) {  # with default\n    gie a + b\n}\n```\n\nFrom Scots \"dae\" meaning \"do\".".to_string()),
        "gie" => Some("**gie** - Return from function\n\n```mdhavers\ndae square(x) {\n    gie x * x\n}\n```\n\nFrom Scots \"gie\" meaning \"give\".".to_string()),
        "blether" => Some("**blether** - Print to output\n\n```mdhavers\nblether \"Hullo warld!\"\nblether f\"The answer is {42}\"\n```\n\nFrom Scots \"blether\" meaning \"chat\" or \"talk\".".to_string()),
        "speir" => Some("**speir** - Get user input\n\n```mdhavers\nken name = speir \"Whit's yer name? \"\n```\n\nFrom Scots \"speir\" meaning \"ask\".".to_string()),
        "aye" => Some("**aye** - Boolean true\n\n```mdhavers\nken happy = aye\n```\n\nFrom Scots \"aye\" meaning \"yes\".".to_string()),
        "nae" => Some("**nae** - Boolean false / logical not\n\n```mdhavers\nken sad = nae\ngin nae is_empty {\n    # do something\n}\n```\n\nFrom Scots \"nae\" meaning \"no\" or \"not\".".to_string()),
        "naething" => Some("**naething** - Null/nil value\n\n```mdhavers\nken nothing = naething\n```\n\nFrom Scots \"naething\" meaning \"nothing\".".to_string()),
        "an" => Some("**an** - Logical AND\n\n```mdhavers\ngin x > 0 an x < 10 {\n    blether \"In range!\"\n}\n```\n\nFrom Scots \"an\" meaning \"and\".".to_string()),
        "or" => Some("**or** - Logical OR\n\n```mdhavers\ngin x < 0 or x > 100 {\n    blether \"Oot o' range!\"\n}\n```".to_string()),
        "brak" => Some("**brak** - Break out of loop\n\n```mdhavers\nfer i in 1..100 {\n    gin i == 50 {\n        brak\n    }\n}\n```\n\nFrom Scots \"brak\" meaning \"break\".".to_string()),
        "haud" => Some("**haud** - Continue to next iteration\n\n```mdhavers\nfer i in 1..10 {\n    gin i % 2 == 0 {\n        haud  # skip even numbers\n    }\n    blether i\n}\n```\n\nFrom Scots \"haud\" meaning \"hold\".".to_string()),
        "kin" => Some("**kin** - Class definition\n\n```mdhavers\nkin Animal {\n    dae mak(name) {\n        masel.name = name\n    }\n    \n    dae speak() {\n        blether f\"{masel.name} says hello!\"\n    }\n}\n```\n\nFrom Scots \"kin\" meaning \"family\" or \"type\".".to_string()),
        "thing" => Some("**thing** - Struct definition\n\n```mdhavers\nthing Point { x, y }\nken p = Point { x: 10, y: 20 }\n```\n\nDefines a simple data structure.".to_string()),
        "masel" => Some("**masel** - Self reference in classes\n\n```mdhavers\nkin Counter {\n    dae increment() {\n        masel.count = masel.count + 1\n    }\n}\n```\n\nFrom Scots \"masel\" meaning \"myself\".".to_string()),
        "fetch" => Some("**fetch** - Import a module\n\n```mdhavers\nfetch \"utils\"\nfetch \"math\" tae maths\n```\n\nImport code from another file.".to_string()),
        "hae_a_bash" => Some("**hae_a_bash** - Try block\n\n```mdhavers\nhae_a_bash {\n    # risky code\n} gin_it_gangs_wrang e {\n    blether f\"Error: {e}\"\n}\n```\n\nFrom Scots \"hae a bash\" meaning \"give it a try\".".to_string()),
        "gin_it_gangs_wrang" => Some("**gin_it_gangs_wrang** - Catch block\n\n```mdhavers\nhae_a_bash {\n    ken x = 1 / 0\n} gin_it_gangs_wrang e {\n    blether \"Oops!\"\n}\n```\n\nFrom Scots \"gin it gangs wrang\" meaning \"if it goes wrong\".".to_string()),
        "keek" => Some("**keek** - Match/switch statement\n\n```mdhavers\nkeek value {\n    whan 1 -> blether \"One\"\n    whan 2 -> blether \"Two\"\n    whan _ -> blether \"Something else\"\n}\n```\n\nFrom Scots \"keek\" meaning \"peek\" or \"look\".".to_string()),
        "whan" => Some("**whan** - Case in match statement\n\n```mdhavers\nkeek x {\n    whan 1 -> blether \"One\"\n    whan 2 -> blether \"Two\"\n}\n```\n\nFrom Scots \"whan\" meaning \"when\".".to_string()),
        "mak_siccar" => Some("**mak_siccar** - Assert\n\n```mdhavers\nmak_siccar x > 0, \"x must be positive!\"\n```\n\nFrom Scots \"mak siccar\" meaning \"make sure\" - famously said by Robert the Bruce!".to_string()),
        "in" => Some("**in** - Used in for loops\n\n```mdhavers\nfer item in list {\n    blether item\n}\n```".to_string()),
        "is" => Some("**is** - Type checking\n\n```mdhavers\ngin x is \"integer\" {\n    blether \"It's a number!\"\n}\n```".to_string()),
        "fae" => Some("**fae** - From (used in imports and inheritance)\n\n```mdhavers\nkin Dog fae Animal {\n    # Dog inherits fae Animal\n}\n```\n\nFrom Scots \"fae\" meaning \"from\".".to_string()),
        "tae" => Some("**tae** - To (used in imports for aliasing)\n\n```mdhavers\nfetch \"math\" tae maths\n```\n\nFrom Scots \"tae\" meaning \"to\".".to_string()),

        // Built-in functions
        "len" => Some("**len(x)** - Get the length\n\n```mdhavers\nken size = len([1, 2, 3])  # 3\nken chars = len(\"hello\")   # 5\n```".to_string()),
        "whit_kind" => Some("**whit_kind(x)** - Get the type of a value\n\n```mdhavers\nblether whit_kind(42)      # \"integer\"\nblether whit_kind(\"hi\")    # \"string\"\nblether whit_kind([1,2])   # \"list\"\n```\n\nFrom Scots \"whit kind\" meaning \"what type\".".to_string()),
        "tae_string" => Some("**tae_string(x)** - Convert to string\n\n```mdhavers\nken s = tae_string(42)  # \"42\"\n```".to_string()),
        "tae_int" => Some("**tae_int(x)** - Convert to integer\n\n```mdhavers\nken n = tae_int(\"42\")   # 42\nken m = tae_int(3.14)   # 3\n```".to_string()),
        "tae_float" => Some("**tae_float(x)** - Convert to float\n\n```mdhavers\nken f = tae_float(\"3.14\")  # 3.14\n```".to_string()),
        "shove" => Some("**shove(list, item)** - Add item to end of list\n\n```mdhavers\nken nums = [1, 2]\nshove(nums, 3)  # [1, 2, 3]\n```\n\nFrom Scots \"shove\" meaning \"push\".".to_string()),
        "yank" => Some("**yank(list)** - Remove and return last item\n\n```mdhavers\nken nums = [1, 2, 3]\nken last = yank(nums)  # 3, nums is now [1, 2]\n```\n\nFrom Scots \"yank\" meaning \"pull\".".to_string()),
        "heid" => Some("**heid(list)** - Get first element\n\n```mdhavers\nken first = heid([1, 2, 3])  # 1\n```\n\nFrom Scots \"heid\" meaning \"head\".".to_string()),
        "tail" => Some("**tail(list)** - Get all but first element\n\n```mdhavers\nken rest = tail([1, 2, 3])  # [2, 3]\n```".to_string()),
        "bum" => Some("**bum(list)** - Get last element\n\n```mdhavers\nken last = bum([1, 2, 3])  # 3\n```\n\nFrom Scots \"bum\" meaning \"bottom\".".to_string()),
        "range" => Some("**range(start, end)** - Create a range\n\n```mdhavers\nken nums = range(1, 5)  # [1, 2, 3, 4]\n```".to_string()),
        "keys" => Some("**keys(dict)** - Get dictionary keys\n\n```mdhavers\nken k = keys({a: 1, b: 2})  # [\"a\", \"b\"]\n```".to_string()),
        "values" => Some("**values(dict)** - Get dictionary values\n\n```mdhavers\nken v = values({a: 1, b: 2})  # [1, 2]\n```".to_string()),
        "abs" => Some("**abs(x)** - Absolute value\n\n```mdhavers\nken n = abs(-42)  # 42\n```".to_string()),
        "min" => Some("**min(a, b)** - Minimum of two values\n\n```mdhavers\nken m = min(3, 7)  # 3\n```".to_string()),
        "max" => Some("**max(a, b)** - Maximum of two values\n\n```mdhavers\nken m = max(3, 7)  # 7\n```".to_string()),
        "floor" => Some("**floor(x)** - Round down\n\n```mdhavers\nken n = floor(3.7)  # 3\n```".to_string()),
        "ceil" => Some("**ceil(x)** - Round up\n\n```mdhavers\nken n = ceil(3.2)  # 4\n```".to_string()),
        "round" => Some("**round(x)** - Round to nearest integer\n\n```mdhavers\nken n = round(3.5)  # 4\n```".to_string()),
        "sqrt" => Some("**sqrt(x)** - Square root\n\n```mdhavers\nken r = sqrt(16)  # 4.0\n```".to_string()),
        "split" => Some("**split(string, delimiter)** - Split string\n\n```mdhavers\nken words = split(\"a,b,c\", \",\")  # [\"a\", \"b\", \"c\"]\n```".to_string()),
        "join" => Some("**join(list, delimiter)** - Join list to string\n\n```mdhavers\nken s = join([\"a\", \"b\"], \"-\")  # \"a-b\"\n```".to_string()),
        "contains" => Some("**contains(haystack, needle)** - Check if contains\n\n```mdhavers\ncontains(\"hello\", \"ell\")  # aye\ncontains([1,2,3], 2)       # aye\n```".to_string()),
        "reverse" => Some("**reverse(list)** - Reverse a list\n\n```mdhavers\nken r = reverse([1, 2, 3])  # [3, 2, 1]\n```".to_string()),
        "sort" => Some("**sort(list)** - Sort a list\n\n```mdhavers\nken s = sort([3, 1, 2])  # [1, 2, 3]\n```".to_string()),
        "upper" => Some("**upper(string)** - Convert to uppercase\n\n```mdhavers\nken u = upper(\"hello\")  # \"HELLO\"\n```".to_string()),
        "lower" => Some("**lower(string)** - Convert to lowercase\n\n```mdhavers\nken l = lower(\"HELLO\")  # \"hello\"\n```".to_string()),
        "shuffle" => Some("**shuffle(list)** - Randomly shuffle a list\n\n```mdhavers\nken s = shuffle([1, 2, 3])  # random order\n```".to_string()),
        "gaun" => Some("**gaun(list, fn)** - Map function over list\n\n```mdhavers\nken doubled = gaun([1, 2, 3], |x| x * 2)  # [2, 4, 6]\n```\n\nFrom Scots \"gaun\" meaning \"going\".".to_string()),
        "sieve" => Some("**sieve(list, fn)** - Filter list by predicate\n\n```mdhavers\nken evens = sieve([1,2,3,4], |x| x % 2 == 0)  # [2, 4]\n```\n\nFrom Scots \"sieve\" meaning \"to filter\".".to_string()),
        "tumble" => Some("**tumble(list, init, fn)** - Reduce/fold list\n\n```mdhavers\nken sum = tumble([1,2,3], 0, |acc, x| acc + x)  # 6\n```\n\nFrom Scots \"tumble\" meaning \"to roll up\".".to_string()),
        "aw" => Some("**aw(list, fn)** - Check if all elements satisfy predicate\n\n```mdhavers\nken all_pos = aw([1,2,3], |x| x > 0)  # aye\n```\n\nFrom Scots \"aw\" meaning \"all\".".to_string()),
        "ony" => Some("**ony(list, fn)** - Check if any element satisfies predicate\n\n```mdhavers\nken has_neg = ony([1,-2,3], |x| x < 0)  # aye\n```\n\nFrom Scots \"ony\" meaning \"any\".".to_string()),
        "hunt" => Some("**hunt(list, fn)** - Find first element satisfying predicate\n\n```mdhavers\nken first_even = hunt([1,2,3,4], |x| x % 2 == 0)  # 2\n```\n\nFrom Scots \"hunt\" meaning \"search\".".to_string()),
        "noo" => Some("**noo()** - Current timestamp in milliseconds\n\n```mdhavers\nken start = noo()\n# ... do stuff ...\nken elapsed = noo() - start\n```\n\nFrom Scots \"noo\" meaning \"now\".".to_string()),
        "bide" => Some("**bide(ms)** - Sleep for milliseconds\n\n```mdhavers\nbide(1000)  # wait 1 second\n```\n\nFrom Scots \"bide\" meaning \"wait\".".to_string()),
        "jammy" => Some("**jammy(min, max)** - Random integer in range\n\n```mdhavers\nken lucky = jammy(1, 100)  # random 1-99\n```\n\nFrom Scots \"jammy\" meaning \"lucky\".".to_string()),
        "clype" => Some("**clype(msg)** - Print debug message to stderr\n\n```mdhavers\nclype(\"Debug info here\")\n```\n\nFrom Scots \"clype\" meaning \"to tell tales\".".to_string()),

        // Set functions
        "creel" => Some("**creel(list)** - Create a set from list\n\n```mdhavers\nken s = creel([1, 2, 2, 3])  # {1, 2, 3}\n```\n\nFrom Scots \"creel\" - a basket.".to_string()),
        "empty_creel" => Some("**empty_creel()** - Create an empty set\n\n```mdhavers\nken s = empty_creel()\n```".to_string()),
        "toss_in" => Some("**toss_in(creel, item)** - Add to set\n\n```mdhavers\ntoss_in(my_set, 42)\n```".to_string()),
        "chuck_oot" => Some("**chuck_oot(creel, item)** - Remove from set\n\n```mdhavers\nchuck_oot(my_set, 42)\n```".to_string()),

        // Audio (soond/muisic/midi)
        "soond_stairt" => Some("**soond_stairt()** - Start the audio device.".to_string()),
        "soond_steek" => Some("**soond_steek()** - Shut the audio device and unload all audio.".to_string()),
        "soond_wheesht" => Some("**soond_wheesht(aye|nae)** - Mute or unmute master audio.".to_string()),
        "soond_luid" => Some("**soond_luid(v)** - Set master volume (0..1).".to_string()),
        "soond_hou_luid" => Some("**soond_hou_luid()** - Get master volume.".to_string()),
        "soond_haud_gang" => Some("**soond_haud_gang()** - Pump streaming audio (call in main loop).".to_string()),
        "soond_lade" => Some("**soond_lade(path)** - Load a sound effect (WAV) and return a handle.".to_string()),
        "soond_ready" => Some("**soond_ready(handle)** - Check if SFX is ready (web backends).".to_string()),
        "soond_spiel" => Some("**soond_spiel(handle)** - Play a sound effect.".to_string()),
        "soond_haud" => Some("**soond_haud(handle)** - Pause a sound effect.".to_string()),
        "soond_gae_on" => Some("**soond_gae_on(handle)** - Resume a sound effect.".to_string()),
        "soond_stap" => Some("**soond_stap(handle)** - Stop a sound effect.".to_string()),
        "soond_unlade" => Some("**soond_unlade(handle)** - Unload a sound effect.".to_string()),
        "soond_is_spielin" => Some("**soond_is_spielin(handle)** - Returns aye if playing.".to_string()),
        "soond_pit_luid" => Some("**soond_pit_luid(handle, v)** - Set SFX volume (0..1).".to_string()),
        "soond_pit_pan" => Some("**soond_pit_pan(handle, pan)** - Set SFX pan (-1..1).".to_string()),
        "soond_pit_tune" => Some("**soond_pit_tune(handle, pitch)** - Set SFX pitch (1.0 normal).".to_string()),
        "soond_pit_rin_roond" => Some("**soond_pit_rin_roond(handle, aye|nae)** - Loop a sound effect.".to_string()),

        "muisic_lade" => Some("**muisic_lade(path)** - Load streaming music (MP3/long WAV).".to_string()),
        "muisic_spiel" => Some("**muisic_spiel(handle)** - Play music.".to_string()),
        "muisic_haud" => Some("**muisic_haud(handle)** - Pause music.".to_string()),
        "muisic_gae_on" => Some("**muisic_gae_on(handle)** - Resume music.".to_string()),
        "muisic_stap" => Some("**muisic_stap(handle)** - Stop music.".to_string()),
        "muisic_unlade" => Some("**muisic_unlade(handle)** - Unload music.".to_string()),
        "muisic_is_spielin" => Some("**muisic_is_spielin(handle)** - Returns aye if playing.".to_string()),
        "muisic_loup" => Some("**muisic_loup(handle, seconds)** - Seek music position.".to_string()),
        "muisic_hou_lang" => Some("**muisic_hou_lang(handle)** - Music length in seconds.".to_string()),
        "muisic_whaur" => Some("**muisic_whaur(handle)** - Current music position in seconds.".to_string()),
        "muisic_pit_luid" => Some("**muisic_pit_luid(handle, v)** - Set music volume (0..1).".to_string()),
        "muisic_pit_pan" => Some("**muisic_pit_pan(handle, pan)** - Set music pan (-1..1).".to_string()),
        "muisic_pit_tune" => Some("**muisic_pit_tune(handle, pitch)** - Set music pitch (1.0 normal).".to_string()),
        "muisic_pit_rin_roond" => Some("**muisic_pit_rin_roond(handle, aye|nae)** - Loop music.".to_string()),

        "midi_lade" => Some("**midi_lade(path, soundfont)** - Load MIDI (soundfont or naething).".to_string()),
        "midi_spiel" => Some("**midi_spiel(handle)** - Play MIDI.".to_string()),
        "midi_haud" => Some("**midi_haud(handle)** - Pause MIDI.".to_string()),
        "midi_gae_on" => Some("**midi_gae_on(handle)** - Resume MIDI.".to_string()),
        "midi_stap" => Some("**midi_stap(handle)** - Stop MIDI.".to_string()),
        "midi_unlade" => Some("**midi_unlade(handle)** - Unload MIDI.".to_string()),
        "midi_is_spielin" => Some("**midi_is_spielin(handle)** - Returns aye if playing.".to_string()),
        "midi_loup" => Some("**midi_loup(handle, seconds)** - Seek MIDI position.".to_string()),
        "midi_hou_lang" => Some("**midi_hou_lang(handle)** - MIDI length in seconds.".to_string()),
        "midi_whaur" => Some("**midi_whaur(handle)** - Current MIDI position in seconds.".to_string()),
        "midi_pit_luid" => Some("**midi_pit_luid(handle, v)** - Set MIDI volume (0..1).".to_string()),
        "midi_pit_pan" => Some("**midi_pit_pan(handle, pan)** - Set MIDI pan (-1..1).".to_string()),
        "midi_pit_rin_roond" => Some("**midi_pit_rin_roond(handle, aye|nae)** - Loop MIDI.".to_string()),

        _ => None,
    }
}

/// Get all keywords and builtins fer completion
/// Returns (name, kind, documentation)
pub fn get_keywords_and_builtins() -> Vec<(String, String, String)> {
    vec![
        // Keywords
        (
            "ken".to_string(),
            "keyword".to_string(),
            "Variable declaration".to_string(),
        ),
        (
            "gin".to_string(),
            "keyword".to_string(),
            "If statement".to_string(),
        ),
        (
            "ither".to_string(),
            "keyword".to_string(),
            "Else clause".to_string(),
        ),
        (
            "than".to_string(),
            "keyword".to_string(),
            "Then (ternary)".to_string(),
        ),
        (
            "whiles".to_string(),
            "keyword".to_string(),
            "While loop".to_string(),
        ),
        (
            "fer".to_string(),
            "keyword".to_string(),
            "For loop".to_string(),
        ),
        (
            "dae".to_string(),
            "keyword".to_string(),
            "Function definition".to_string(),
        ),
        (
            "gie".to_string(),
            "keyword".to_string(),
            "Return statement".to_string(),
        ),
        (
            "blether".to_string(),
            "keyword".to_string(),
            "Print output".to_string(),
        ),
        (
            "speir".to_string(),
            "keyword".to_string(),
            "User input".to_string(),
        ),
        (
            "aye".to_string(),
            "constant".to_string(),
            "Boolean true".to_string(),
        ),
        (
            "nae".to_string(),
            "keyword".to_string(),
            "Boolean false / not".to_string(),
        ),
        (
            "naething".to_string(),
            "constant".to_string(),
            "Null value".to_string(),
        ),
        (
            "an".to_string(),
            "keyword".to_string(),
            "Logical AND".to_string(),
        ),
        (
            "or".to_string(),
            "keyword".to_string(),
            "Logical OR".to_string(),
        ),
        (
            "brak".to_string(),
            "keyword".to_string(),
            "Break from loop".to_string(),
        ),
        (
            "haud".to_string(),
            "keyword".to_string(),
            "Continue loop".to_string(),
        ),
        (
            "kin".to_string(),
            "keyword".to_string(),
            "Class definition".to_string(),
        ),
        (
            "thing".to_string(),
            "keyword".to_string(),
            "Struct definition".to_string(),
        ),
        (
            "masel".to_string(),
            "keyword".to_string(),
            "Self reference".to_string(),
        ),
        (
            "fetch".to_string(),
            "keyword".to_string(),
            "Import module".to_string(),
        ),
        (
            "hae_a_bash".to_string(),
            "keyword".to_string(),
            "Try block".to_string(),
        ),
        (
            "gin_it_gangs_wrang".to_string(),
            "keyword".to_string(),
            "Catch block".to_string(),
        ),
        (
            "keek".to_string(),
            "keyword".to_string(),
            "Match statement".to_string(),
        ),
        (
            "whan".to_string(),
            "keyword".to_string(),
            "Match case".to_string(),
        ),
        (
            "mak_siccar".to_string(),
            "keyword".to_string(),
            "Assert".to_string(),
        ),
        (
            "in".to_string(),
            "keyword".to_string(),
            "For-in keyword".to_string(),
        ),
        (
            "is".to_string(),
            "keyword".to_string(),
            "Type check".to_string(),
        ),
        (
            "fae".to_string(),
            "keyword".to_string(),
            "From (inheritance)".to_string(),
        ),
        (
            "tae".to_string(),
            "keyword".to_string(),
            "To (import alias)".to_string(),
        ),
        // Built-in functions
        (
            "len".to_string(),
            "function".to_string(),
            "Get length of list/string".to_string(),
        ),
        (
            "whit_kind".to_string(),
            "function".to_string(),
            "Get type of value".to_string(),
        ),
        (
            "tae_string".to_string(),
            "function".to_string(),
            "Convert to string".to_string(),
        ),
        (
            "tae_int".to_string(),
            "function".to_string(),
            "Convert to integer".to_string(),
        ),
        (
            "tae_float".to_string(),
            "function".to_string(),
            "Convert to float".to_string(),
        ),
        (
            "shove".to_string(),
            "function".to_string(),
            "Add to list (push)".to_string(),
        ),
        (
            "yank".to_string(),
            "function".to_string(),
            "Remove from list (pop)".to_string(),
        ),
        (
            "heid".to_string(),
            "function".to_string(),
            "First element".to_string(),
        ),
        (
            "tail".to_string(),
            "function".to_string(),
            "All but first".to_string(),
        ),
        (
            "bum".to_string(),
            "function".to_string(),
            "Last element".to_string(),
        ),
        (
            "range".to_string(),
            "function".to_string(),
            "Create range".to_string(),
        ),
        (
            "keys".to_string(),
            "function".to_string(),
            "Dictionary keys".to_string(),
        ),
        (
            "values".to_string(),
            "function".to_string(),
            "Dictionary values".to_string(),
        ),
        (
            "abs".to_string(),
            "function".to_string(),
            "Absolute value".to_string(),
        ),
        (
            "min".to_string(),
            "function".to_string(),
            "Minimum".to_string(),
        ),
        (
            "max".to_string(),
            "function".to_string(),
            "Maximum".to_string(),
        ),
        (
            "floor".to_string(),
            "function".to_string(),
            "Round down".to_string(),
        ),
        (
            "ceil".to_string(),
            "function".to_string(),
            "Round up".to_string(),
        ),
        (
            "round".to_string(),
            "function".to_string(),
            "Round".to_string(),
        ),
        (
            "sqrt".to_string(),
            "function".to_string(),
            "Square root".to_string(),
        ),
        (
            "split".to_string(),
            "function".to_string(),
            "Split string".to_string(),
        ),
        (
            "join".to_string(),
            "function".to_string(),
            "Join list".to_string(),
        ),
        (
            "contains".to_string(),
            "function".to_string(),
            "Check containment".to_string(),
        ),
        (
            "reverse".to_string(),
            "function".to_string(),
            "Reverse list".to_string(),
        ),
        (
            "sort".to_string(),
            "function".to_string(),
            "Sort list".to_string(),
        ),
        (
            "upper".to_string(),
            "function".to_string(),
            "Uppercase".to_string(),
        ),
        (
            "lower".to_string(),
            "function".to_string(),
            "Lowercase".to_string(),
        ),
        (
            "shuffle".to_string(),
            "function".to_string(),
            "Shuffle list".to_string(),
        ),
        (
            "gaun".to_string(),
            "function".to_string(),
            "Map function".to_string(),
        ),
        (
            "sieve".to_string(),
            "function".to_string(),
            "Filter list".to_string(),
        ),
        (
            "tumble".to_string(),
            "function".to_string(),
            "Reduce/fold".to_string(),
        ),
        (
            "aw".to_string(),
            "function".to_string(),
            "All satisfy".to_string(),
        ),
        (
            "ony".to_string(),
            "function".to_string(),
            "Any satisfy".to_string(),
        ),
        (
            "hunt".to_string(),
            "function".to_string(),
            "Find first".to_string(),
        ),
        (
            "noo".to_string(),
            "function".to_string(),
            "Current time (ms)".to_string(),
        ),
        (
            "bide".to_string(),
            "function".to_string(),
            "Sleep (ms)".to_string(),
        ),
        (
            "jammy".to_string(),
            "function".to_string(),
            "Random number".to_string(),
        ),
        (
            "clype".to_string(),
            "function".to_string(),
            "Debug print".to_string(),
        ),
        (
            "creel".to_string(),
            "function".to_string(),
            "Create set".to_string(),
        ),
        (
            "empty_creel".to_string(),
            "function".to_string(),
            "Empty set".to_string(),
        ),
        (
            "toss_in".to_string(),
            "function".to_string(),
            "Add to set".to_string(),
        ),
        (
            "chuck_oot".to_string(),
            "function".to_string(),
            "Remove from set".to_string(),
        ),
        (
            "soond_stairt".to_string(),
            "function".to_string(),
            "Start audio device".to_string(),
        ),
        (
            "soond_steek".to_string(),
            "function".to_string(),
            "Stop audio device".to_string(),
        ),
        (
            "soond_wheesht".to_string(),
            "function".to_string(),
            "Mute/unmute audio".to_string(),
        ),
        (
            "soond_luid".to_string(),
            "function".to_string(),
            "Set master volume".to_string(),
        ),
        (
            "soond_hou_luid".to_string(),
            "function".to_string(),
            "Get master volume".to_string(),
        ),
        (
            "soond_haud_gang".to_string(),
            "function".to_string(),
            "Pump streaming audio".to_string(),
        ),
        (
            "soond_lade".to_string(),
            "function".to_string(),
            "Load SFX".to_string(),
        ),
        (
            "soond_ready".to_string(),
            "function".to_string(),
            "Check SFX ready".to_string(),
        ),
        (
            "soond_spiel".to_string(),
            "function".to_string(),
            "Play SFX".to_string(),
        ),
        (
            "soond_haud".to_string(),
            "function".to_string(),
            "Pause SFX".to_string(),
        ),
        (
            "soond_gae_on".to_string(),
            "function".to_string(),
            "Resume SFX".to_string(),
        ),
        (
            "soond_stap".to_string(),
            "function".to_string(),
            "Stop SFX".to_string(),
        ),
        (
            "soond_unlade".to_string(),
            "function".to_string(),
            "Unload SFX".to_string(),
        ),
        (
            "soond_is_spielin".to_string(),
            "function".to_string(),
            "SFX playing?".to_string(),
        ),
        (
            "soond_pit_luid".to_string(),
            "function".to_string(),
            "SFX volume".to_string(),
        ),
        (
            "soond_pit_pan".to_string(),
            "function".to_string(),
            "SFX pan".to_string(),
        ),
        (
            "soond_pit_tune".to_string(),
            "function".to_string(),
            "SFX pitch".to_string(),
        ),
        (
            "soond_pit_rin_roond".to_string(),
            "function".to_string(),
            "Loop SFX".to_string(),
        ),
        (
            "muisic_lade".to_string(),
            "function".to_string(),
            "Load music".to_string(),
        ),
        (
            "muisic_spiel".to_string(),
            "function".to_string(),
            "Play music".to_string(),
        ),
        (
            "muisic_haud".to_string(),
            "function".to_string(),
            "Pause music".to_string(),
        ),
        (
            "muisic_gae_on".to_string(),
            "function".to_string(),
            "Resume music".to_string(),
        ),
        (
            "muisic_stap".to_string(),
            "function".to_string(),
            "Stop music".to_string(),
        ),
        (
            "muisic_unlade".to_string(),
            "function".to_string(),
            "Unload music".to_string(),
        ),
        (
            "muisic_is_spielin".to_string(),
            "function".to_string(),
            "Music playing?".to_string(),
        ),
        (
            "muisic_loup".to_string(),
            "function".to_string(),
            "Seek music".to_string(),
        ),
        (
            "muisic_hou_lang".to_string(),
            "function".to_string(),
            "Music length".to_string(),
        ),
        (
            "muisic_whaur".to_string(),
            "function".to_string(),
            "Music position".to_string(),
        ),
        (
            "muisic_pit_luid".to_string(),
            "function".to_string(),
            "Music volume".to_string(),
        ),
        (
            "muisic_pit_pan".to_string(),
            "function".to_string(),
            "Music pan".to_string(),
        ),
        (
            "muisic_pit_tune".to_string(),
            "function".to_string(),
            "Music pitch".to_string(),
        ),
        (
            "muisic_pit_rin_roond".to_string(),
            "function".to_string(),
            "Loop music".to_string(),
        ),
        (
            "midi_lade".to_string(),
            "function".to_string(),
            "Load MIDI".to_string(),
        ),
        (
            "midi_spiel".to_string(),
            "function".to_string(),
            "Play MIDI".to_string(),
        ),
        (
            "midi_haud".to_string(),
            "function".to_string(),
            "Pause MIDI".to_string(),
        ),
        (
            "midi_gae_on".to_string(),
            "function".to_string(),
            "Resume MIDI".to_string(),
        ),
        (
            "midi_stap".to_string(),
            "function".to_string(),
            "Stop MIDI".to_string(),
        ),
        (
            "midi_unlade".to_string(),
            "function".to_string(),
            "Unload MIDI".to_string(),
        ),
        (
            "midi_is_spielin".to_string(),
            "function".to_string(),
            "MIDI playing?".to_string(),
        ),
        (
            "midi_loup".to_string(),
            "function".to_string(),
            "Seek MIDI".to_string(),
        ),
        (
            "midi_hou_lang".to_string(),
            "function".to_string(),
            "MIDI length".to_string(),
        ),
        (
            "midi_whaur".to_string(),
            "function".to_string(),
            "MIDI position".to_string(),
        ),
        (
            "midi_pit_luid".to_string(),
            "function".to_string(),
            "MIDI volume".to_string(),
        ),
        (
            "midi_pit_pan".to_string(),
            "function".to_string(),
            "MIDI pan".to_string(),
        ),
        (
            "midi_pit_rin_roond".to_string(),
            "function".to_string(),
            "Loop MIDI".to_string(),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_diagnostics_valid() {
        // Valid code should produce no diagnostics
        let source = "ken x = 42\nblether x";
        let diagnostics = get_diagnostics(source);
        assert!(
            diagnostics.is_empty(),
            "Expected no diagnostics for valid code"
        );
    }

    #[test]
    fn test_get_diagnostics_unmatched_braces() {
        // Unclosed brace should produce an error
        let source = "gin x > 0 {\n    blether x\n";
        let diagnostics = get_diagnostics(source);
        assert!(
            !diagnostics.is_empty(),
            "Expected diagnostics for unclosed brace"
        );
        assert!(diagnostics.iter().any(|d| d.2.contains("Unclosed")));
    }

    #[test]
    fn test_get_keyword_info() {
        // Test that we get info for keywords
        let info = get_keyword_info("ken");
        assert!(info.is_some());
        assert!(info.unwrap().contains("Variable declaration"));

        let info = get_keyword_info("gin");
        assert!(info.is_some());
        assert!(info.unwrap().contains("If statement"));

        // Unknown word should return None
        let info = get_keyword_info("foobar");
        assert!(info.is_none());
    }

    #[test]
    fn test_get_keywords_and_builtins() {
        let items = get_keywords_and_builtins();
        assert!(!items.is_empty());

        // Check that common items are present
        let names: Vec<&str> = items.iter().map(|i| i.0.as_str()).collect();
        assert!(names.contains(&"ken"));
        assert!(names.contains(&"gin"));
        assert!(names.contains(&"blether"));
        assert!(names.contains(&"len"));
        assert!(names.contains(&"gaun"));
        assert!(names.contains(&"soond_stairt"));
    }
}
