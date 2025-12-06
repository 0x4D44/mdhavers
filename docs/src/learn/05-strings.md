# Strings

Master text manipulation in mdhavers.

## Creating Strings

```scots
ken double = "Hello, World!"
ken single = 'Also works'
ken empty = ""

# Escape characters
ken with_newline = "Line one\nLine two"
ken with_tab = "Name:\tHamish"
ken with_quote = "She said \"Braw!\""
```

## F-Strings (Interpolation)

Embed expressions directly in strings:

```scots
ken name = "Hamish"
ken age = 30

blether f"I'm {name}, {age} years auld"
# "I'm Hamish, 30 years auld"

# Expressions work too
blether f"Next year I'll be {age + 1}"
# "Next year I'll be 31"

blether f"2 + 2 = {2 + 2}"
# "2 + 2 = 4"

# Lists and other types
ken cities = ["Edinburgh", "Glasgow"]
blether f"Cities: {cities}"
# "Cities: [Edinburgh, Glasgow]"
```

## String Operations

### Length

```scots
ken text = "Scotland"
blether len(text)  # 8
```

### Concatenation

```scots
ken first = "Hullo"
ken second = "World"

ken combined = first + ", " + second + "!"
blether combined  # "Hullo, World!"

# Or use slap
ken joined = slap(first, second)
blether joined  # "HulloWorld"
```

### Repetition

```scots
ken stars = "*" * 5
blether stars  # "*****"

ken pattern = "ab" * 3
blether pattern  # "ababab"
```

### Accessing Characters

```scots
ken word = "Scotland"

blether word[0]   # "S"
blether word[4]   # "l"
blether word[-1]  # "d"
blether word[-2]  # "n"
```

### Iteration

```scots
ken text = "Braw"

fer char in text {
    blether char
}
# B
# r
# a
# w
```

## Case Conversion

```scots
ken text = "Hullo Scotland"

blether upper(text)     # "HULLO SCOTLAND"
blether lower(text)     # "hullo scotland"
blether swapcase(text)  # "hULLO sCOTLAND"

# Check case
blether is_upper("ABC")  # aye
blether is_lower("abc")  # aye
```

## Trimming and Padding

### Trimming

```scots
ken messy = "   hello world   "

blether wheesht(messy)       # "hello world" (trim both sides)
blether strip_left(messy)    # "hello world   "
blether strip_right(messy)   # "   hello world"
```

### Padding

```scots
ken text = "42"

blether pad_left(text, 5, "0")   # "00042"
blether pad_right(text, 5, "0")  # "42000"
blether center(text, 6, "-")     # "--42--"
```

## Searching

### Contains

```scots
ken sentence = "The quick brown fox"

blether contains(sentence, "quick")  # aye
blether contains(sentence, "slow")   # nae
```

### Finding Substrings

```scots
ken text = "banana"

# Find all occurrences
ken positions = haggis_hunt(text, "an")
blether positions  # [1, 3]

# Count occurrences
blether coont(text, "a")  # 3
```

### Between Markers

```scots
ken html = "<title>Hello World</title>"
ken title = substr_between(html, "<title>", "</title>")
blether title  # "Hello World"
```

## Splitting and Joining

### Split

```scots
ken sentence = "one,two,three,four"
ken parts = split(sentence, ",")
blether parts  # ["one", "two", "three", "four"]

# Split on whitespace
ken words = split("hello world", " ")
blether words  # ["hello", "world"]
```

### Join

```scots
ken words = ["hello", "world"]
ken sentence = join(words, " ")
blether sentence  # "hello world"

ken path = join(["home", "user", "docs"], "/")
blether path  # "home/user/docs"
```

## Replacing

### Replace All

```scots
ken text = "hello hello hello"

# Replace all occurrences
ken result = text  # Note: use gaun for replace
blether result
```

### Replace First

```scots
ken text = "hello hello hello"
ken result = replace_first(text, "hello", "hi")
blether result  # "hi hello hello"
```

## Scottish String Functions

mdhavers has some uniquely Scottish string functions:

```scots
# Scottify English text
blether scottify("Hello, how are you today?")
# "Hullo, hoo are ye the day?"

# Various expressions
blether roar("hello")    # "HELLO!" (shout)
blether mutter("HELLO")  # "...hello..." (whisper)
blether bonnie("wow")    # "~~~ wow ~~~" (decorate)

# Chaos functions
blether stooshie("hello")  # Random shuffle of characters
blether blooter("hello")   # Another way to scramble

# Repeat with separator
blether tattie_scone("la", 3)  # "la|la|la"
```

## String Validation

```scots
# Check if string is empty or problematic
blether haverin("")        # aye (talking nonsense = empty)
blether haverin("hello")   # nae

blether glaikit("")        # aye (silly = empty)
blether dreich("aaaa")     # aye (boring = monotonous)

# Format with dictionary
ken template = "Hello {name}, you have {count} messages"
ken data = {"name": "Hamish", "count": 5}
blether blether_format(template, data)
# "Hello Hamish, you have 5 messages"
```

## Conversion

### String to List

```scots
ken text = "hello"
ken chars = [..."hello"]  # Spread into list
blether chars  # ["h", "e", "l", "l", "o"]
```

### List to String

```scots
ken chars = ["h", "e", "l", "l", "o"]
ken text = join(chars, "")
blether text  # "hello"
```

### To/From Numbers

```scots
# String to number
ken num = tae_int("42")
ken decimal = tae_float("3.14")

# Number to string
ken text = tae_string(42)
blether f"The answer is {text}"
```

## Practical Examples

### Palindrome Check

```scots
dae is_palindrome(text) {
    ken clean = lower(wheesht(text))
    gie clean == reverse(clean)
}

blether is_palindrome("radar")    # aye
blether is_palindrome("A man a plan a canal Panama")  # nae (has spaces)
```

### Word Capitalizer

```scots
dae capitalize_words(sentence) {
    ken words = split(sentence, " ")
    ken capitalized = gaun(words, |word| {
        gin len(word) == 0 {
            gie ""
        }
        ken first = upper(word[0])
        ken rest = lower(scran(word, 1, len(word)))
        gie first + rest
    })
    gie join(capitalized, " ")
}

blether capitalize_words("hello world")  # "Hello World"
```

### Simple Template

```scots
dae template(text, replacements) {
    ken result = text
    fer pair in items(replacements) {
        ken placeholder = "{" + pair[0] + "}"
        # Manual replace using split/join
        ken parts = split(result, placeholder)
        result = join(parts, tae_string(pair[1]))
    }
    gie result
}

ken msg = template("Hello {name}, your order #{id} is ready!", {
    "name": "Hamish",
    "id": 12345
})
blether msg  # "Hello Hamish, your order #12345 is ready!"
```

### CSV Parser (Simple)

```scots
dae parse_csv_line(line) {
    gie split(line, ",")
}

ken header = parse_csv_line("name,age,city")
ken row1 = parse_csv_line("Hamish,30,Glasgow")
ken row2 = parse_csv_line("Morag,28,Edinburgh")

blether header  # ["name", "age", "city"]
blether row1    # ["Hamish", "30", "Glasgow"]
```

## Exercises

1. **Reverse Words**: Reverse the order of words in a sentence

2. **Count Vowels**: Count all vowels in a string

3. **Slug Generator**: Convert a title to a URL slug

<details>
<summary>Solutions</summary>

```scots
# 1. Reverse Words
dae reverse_words(sentence) {
    ken words = split(sentence, " ")
    ken reversed_list = reverse(words)
    gie join(reversed_list, " ")
}

blether reverse_words("hello world foo bar")
# "bar foo world hello"

# 2. Count Vowels
dae count_vowels(text) {
    ken vowels = "aeiouAEIOU"
    ken count = 0
    fer char in text {
        gin contains(vowels, char) {
            count = count + 1
        }
    }
    gie count
}

blether count_vowels("Hello World")  # 3

# 3. Slug Generator
dae to_slug(title) {
    ken lowered = lower(title)
    ken words = split(lowered, " ")
    ken clean_words = sieve(words, |w| len(w) > 0)
    gie join(clean_words, "-")
}

blether to_slug("Hello World This Is A Test")
# "hello-world-this-is-a-test"
```

</details>

## Next Steps

Learn about [classes](./06-classes.md) to create your own custom data types.
