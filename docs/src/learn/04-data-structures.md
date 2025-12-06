# Data Structures

Work with lists and dictionaries to organize your data.

## Lists

### Creating Lists

```scots
ken empty = []
ken numbers = [1, 2, 3, 4, 5]
ken names = ["Hamish", "Morag", "Angus"]
ken mixed = [1, "two", 3.0, aye, naething]
ken nested = [[1, 2], [3, 4], [5, 6]]
```

### Accessing Elements

Use zero-based indexing:

```scots
ken cities = ["Edinburgh", "Glasgow", "Aberdeen", "Dundee"]

blether cities[0]   # "Edinburgh" (first)
blether cities[2]   # "Aberdeen" (third)
blether cities[-1]  # "Dundee" (last)
blether cities[-2]  # "Aberdeen" (second from last)
```

### Modifying Elements

```scots
ken scores = [85, 90, 78]
scores[1] = 95
blether scores  # [85, 95, 78]
```

### List Operations

```scots
ken fruits = ["apple", "banana"]

# Add to end
shove(fruits, "cherry")
blether fruits  # ["apple", "banana", "cherry"]

# Remove from end
ken last = yank(fruits)
blether last    # "cherry"
blether fruits  # ["apple", "banana"]

# Length
blether len(fruits)  # 2

# Check membership
blether contains(fruits, "apple")   # aye
blether contains(fruits, "orange")  # nae
```

### Slicing with scran

```scots
ken nums = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]

ken slice = scran(nums, 2, 5)  # Start at 2, end before 5
blether slice  # [2, 3, 4]

# Using helper functions
blether heid(nums)   # 0 (first element)
blether bum(nums)    # 9 (last element)
blether tail(nums)   # [1, 2, 3, 4, 5, 6, 7, 8, 9] (all but first)
blether tak(nums, 3) # [0, 1, 2] (first 3)
blether drap(nums, 3) # [3, 4, 5, 6, 7, 8, 9] (drop first 3)
```

### Combining Lists

```scots
ken a = [1, 2, 3]
ken b = [4, 5, 6]

# Concatenate
ken combined = slap(a, b)
blether combined  # [1, 2, 3, 4, 5, 6]

# Using spread operator
ken merged = [...a, ...b]
blether merged  # [1, 2, 3, 4, 5, 6]
```

### Sorting and Reversing

```scots
ken nums = [3, 1, 4, 1, 5, 9, 2, 6]

ken sorted_nums = sort(nums)
blether sorted_nums  # [1, 1, 2, 3, 4, 5, 6, 9]

ken reversed_nums = reverse(nums)
blether reversed_nums  # [6, 2, 9, 5, 1, 4, 1, 3]
```

### Finding Elements

```scots
ken items = ["haggis", "neeps", "tatties", "haggis"]

# Find all indices of a value
ken indices = indices_o(items, "haggis")
blether indices  # [0, 3]

# Count occurrences
ken count = coont(items, "haggis")
blether count  # 2
```

### List Comprehension Style

Use `gaun` (map) and `sieve` (filter):

```scots
ken numbers = [1, 2, 3, 4, 5]

# Square all numbers
ken squares = gaun(numbers, |x| x * x)
blether squares  # [1, 4, 9, 16, 25]

# Get even numbers
ken evens = sieve(numbers, |x| x % 2 == 0)
blether evens  # [2, 4]

# Both: square the even numbers
ken even_squares = gaun(sieve(numbers, |x| x % 2 == 0), |x| x * x)
blether even_squares  # [4, 16]
```

### Useful List Functions

```scots
ken nums = [1, 2, 3, 4, 5]

blether sumaw(nums)    # 15 (sum all)
blether average(nums)  # 3 (mean)
blether median(nums)   # 3
blether product(nums)  # 120 (multiply all)
blether minaw(nums)    # 1 (minimum)
blether maxaw(nums)    # 5 (maximum)
blether range_o(nums)  # 4 (max - min)
```

## Dictionaries

### Creating Dictionaries

```scots
ken empty = {}

ken person = {
    "name": "Rabbie Burns",
    "job": "Poet",
    "born": 1759
}

# Keys can be strings or numbers
ken lookup = {
    1: "one",
    2: "two",
    "three": 3
}
```

### Accessing Values

```scots
ken person = {"name": "Hamish", "age": 30}

blether person["name"]  # "Hamish"
blether person["age"]   # 30

# Safe access with default
blether dict_get(person, "city", "Unknown")  # "Unknown"
```

### Modifying Dictionaries

```scots
ken person = {"name": "Hamish"}

# Add or update
person["age"] = 30
person["name"] = "Angus"

blether person  # {"name": "Angus", "age": 30}

# Remove a key
dict_remove(person, "age")
blether person  # {"name": "Angus"}
```

### Dictionary Operations

```scots
ken person = {"name": "Hamish", "age": 30, "city": "Glasgow"}

# Get keys and values
ken all_keys = keys(person)
blether all_keys  # ["name", "age", "city"]

ken all_values = values(person)
blether all_values  # ["Hamish", 30, "Glasgow"]

# Get key-value pairs
ken pairs = items(person)
blether pairs  # [["name", "Hamish"], ["age", 30], ["city", "Glasgow"]]

# Check if key exists
blether dict_has(person, "name")  # aye
blether dict_has(person, "job")   # nae
```

### Merging Dictionaries

```scots
ken defaults = {"color": "blue", "size": "medium"}
ken overrides = {"size": "large", "quantity": 5}

ken merged = dict_merge(defaults, overrides)
blether merged  # {"color": "blue", "size": "large", "quantity": 5}
```

### Iterating Over Dictionaries

```scots
ken scores = {"Hamish": 85, "Morag": 92, "Angus": 78}

# Iterate over keys
fer name in keys(scores) {
    blether f"{name}: {scores[name]}"
}

# Iterate over key-value pairs
fer pair in items(scores) {
    ken name = pair[0]
    ken score = pair[1]
    blether f"{name} scored {score}"
}
```

### Creating from Pairs

```scots
ken pairs = [["a", 1], ["b", 2], ["c", 3]]
ken dict = fae_pairs(pairs)
blether dict  # {"a": 1, "b": 2, "c": 3}
```

### Inverting Dictionaries

```scots
ken grades = {"A": 90, "B": 80, "C": 70}
ken inverted = dict_invert(grades)
blether inverted  # {90: "A", 80: "B", 70: "C"}
```

## Working with Nested Structures

### Nested Lists

```scots
ken matrix = [
    [1, 2, 3],
    [4, 5, 6],
    [7, 8, 9]
]

blether matrix[0][0]  # 1
blether matrix[1][2]  # 6

# Flatten nested lists
blether sclaff([[1, 2], [3, [4, 5]]])  # [1, 2, 3, 4, 5]
```

### Nested Dictionaries

```scots
ken company = {
    "name": "Highland Tech",
    "address": {
        "street": "123 Royal Mile",
        "city": "Edinburgh",
        "postcode": "EH1 1AA"
    },
    "employees": [
        {"name": "Hamish", "role": "Developer"},
        {"name": "Morag", "role": "Designer"}
    ]
}

blether company["address"]["city"]  # "Edinburgh"
blether company["employees"][0]["name"]  # "Hamish"
```

## Practical Examples

### Frequency Counter

```scots
ken text = "banana"
ken freq = {}

fer char in text {
    gin dict_has(freq, char) {
        freq[char] = freq[char] + 1
    } ither {
        freq[char] = 1
    }
}

blether freq  # {"b": 1, "a": 3, "n": 2}
```

### Grouping Data

```scots
ken people = [
    {"name": "Hamish", "city": "Glasgow"},
    {"name": "Morag", "city": "Edinburgh"},
    {"name": "Angus", "city": "Glasgow"},
    {"name": "Flora", "city": "Edinburgh"}
]

ken by_city = {}

fer person in people {
    ken city = person["city"]
    gin nae dict_has(by_city, city) {
        by_city[city] = []
    }
    shove(by_city[city], person["name"])
}

blether by_city
# {"Glasgow": ["Hamish", "Angus"], "Edinburgh": ["Morag", "Flora"]}
```

### Stack Operations

```scots
ken stack = []

# Push
shove(stack, 1)
shove(stack, 2)
shove(stack, 3)

# Pop
ken top = yank(stack)
blether top    # 3
blether stack  # [1, 2]

# Peek
blether bum(stack)  # 2 (without removing)
```

### Queue Operations

```scots
ken queue = []

# Enqueue
shove(queue, "first")
shove(queue, "second")
shove(queue, "third")

# Dequeue
ken front = heid(queue)
queue = tail(queue)

blether front  # "first"
blether queue  # ["second", "third"]
```

## Exercises

1. **Remove Duplicates**: Write a function that removes duplicates from a list

2. **Word Count**: Count words in a sentence and return a dictionary

3. **Two Sum**: Find two numbers in a list that add up to a target

<details>
<summary>Solutions</summary>

```scots
# 1. Remove Duplicates
dae remove_duplicates(list) {
    ken seen = {}
    ken result = []
    fer item in list {
        ken key = tae_string(item)
        gin nae dict_has(seen, key) {
            seen[key] = aye
            shove(result, item)
        }
    }
    gie result
}

blether remove_duplicates([1, 2, 2, 3, 3, 3])  # [1, 2, 3]

# 2. Word Count
dae word_count(sentence) {
    ken words = split(sentence, " ")
    ken counts = {}
    fer word in words {
        ken w = lower(word)
        gin dict_has(counts, w) {
            counts[w] = counts[w] + 1
        } ither {
            counts[w] = 1
        }
    }
    gie counts
}

blether word_count("the cat and the dog")
# {"the": 2, "cat": 1, "and": 1, "dog": 1}

# 3. Two Sum
dae two_sum(nums, target) {
    ken seen = {}
    fer i in 0..len(nums) {
        ken complement = target - nums[i]
        ken comp_key = tae_string(complement)
        gin dict_has(seen, comp_key) {
            gie [seen[comp_key], i]
        }
        seen[tae_string(nums[i])] = i
    }
    gie naething
}

blether two_sum([2, 7, 11, 15], 9)  # [0, 1]
```

</details>

## Next Steps

Learn about [strings](./05-strings.md) and the many ways to manipulate text in mdhavers.
