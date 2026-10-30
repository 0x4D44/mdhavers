# quiz

Plain-dictionary quiz and flashcard helpers, plus ready-made Scottish geography, history, culture and language question sets. A quiz is a dictionary, so there are no classes; the module only builds and prints them (there is no answer-checking or scoring function).

Import: `fetch "stdlib/quiz"`

Pure mdhavers, no native built-ins. Loading the module prints `Quiz module loaded! Test yer knowledge!`. It defines a top-level `chr(code)` which is a local helper, not the usual character-code built-in.

## Data shapes

- Question: `{"text", "options", "correct", "explanation", "type"}` where `correct` is a zero-based index and `type` is `"mc"` or `"tf"`.
- Quiz: `{"title", "description", "questions", "score", "answered"}`.
- Flashcard: `{"front", "back"}`. Deck: `{"name", "cards"}`.

None of the functions hurl errors; bad input gives the usual runtime errors (for example a missing dictionary key).

## Functions

### question(text, options, correct_idx, explanation)
Returns a multiple-choice question dictionary (`type` `"mc"`).

### tf_question(text, answer, explanation)
Returns a true/false question with options `["True", "False"]`. A truthy `answer` (`aye`) gives `correct` 0, otherwise 1.

### quiz(title, description)
Returns an empty quiz dictionary.

### add_question(q, qn)
Appends `qn` to the quiz's `questions` (mutating it) and returns the quiz.

### chr(code)
Returns the letter for ASCII codes 65 to 90 (`A` to `Z`), otherwise `"?"`. Used to label options.

### display_question(q, idx)
Prints question number `idx + 1` of the quiz with lettered options, without marking the answer. Returns `naething`.

### demo_quiz(q)
Prints the whole quiz with every option, the correct one marked `[CORRECT]`, and the explanation if there is one. Returns `naething`.

### scots_geography_questions()
Returns a list of 5 geography questions.

### scots_history_questions()
Returns a list of 5 history questions.

### scots_culture_questions()
Returns a list of 5 culture questions.

### scots_language_questions()
Returns a list of 5 Scots-language questions.

### scottish_geography_quiz()
Returns a quiz built from `scots_geography_questions()`.

### scottish_history_quiz()
Returns a quiz built from `scots_history_questions()`.

### scottish_culture_quiz()
Returns a quiz built from `scots_culture_questions()`.

### scots_language_quiz()
Returns a quiz built from `scots_language_questions()`.

### flashcard(front, back)
Returns a flashcard dictionary.

### flashcard_deck(name)
Returns an empty deck.

### add_card(deck, card)
Appends the card to the deck and returns the deck.

### scots_vocab_flashcards()
Returns a deck of 15 Scots vocabulary cards (Braw, Ken, Bairn and so on).

### demo_flashcards(deck)
Prints the deck name, the card count, and one numbered `front -> back` line per card. Returns `naething`.

## Example

```mdhavers
fetch "stdlib/quiz"

ken q = quiz("Wee Quiz", "A wee test")
q = add_question(q, question("Capital o' Scotland?", ["Glasgow", "Edinburgh"], 1, "Since the 15th century."))
q = add_question(q, tf_question("Nessie lives in Loch Ness", aye, ""))
display_question(q, 0)
demo_quiz(q)
blether len(scottish_geography_quiz()["questions"])
```
