# crypto

Simple encoding, ciphers and hashing for learning and puzzles: base64, hex, Caesar and ROT13, XOR, Vigenere and Atbash ciphers, a DJB2 string hash and a password-strength checker. The source says it plainly: this is educational only. None of it is secure; use a real cryptography library for real security work. It is pure mdhavers and works on every backend.

Import: `fetch "stdlib/crypto"`

Importing prints `Crypto module loaded! Keep yer secrets safe!`. No function hurls a custom error. Text is processed one character at a time using `ord` and `chr`, so base64 and hex are only correct for ASCII text. Helpers starting with `_` are internal and not documented.

## Constants

`BASE64_CHARS` (the base64 alphabet), `HEX_CHARS` (`0123456789abcdef`), `ALPHABET_UPPER` and `ALPHABET_LOWER`.

## Functions

### base64_encode(text)
Standard base64 with `=` padding.

### base64_decode(encoded)
Decodes base64. Characters outside the alphabet are skipped silently.

### index_of_char(s, c)
Position of character `c` in `s`, or -1. (Public helper used by the ciphers.)

### hex_encode(text)
Two lowercase hex digits per character.

### hex_decode(hex)
Decodes hex (case-insensitive). Pairs containing a non-hex character are skipped, and a trailing odd digit is ignored.

### caesar_cipher(text, shift)
Shifts letters by `shift` (negative allowed), keeping case; other characters pass through.

### rot13(text)
`caesar_cipher(text, 13)`; applying it twice gives the original.

### caesar_decrypt(text, shift)
Undoes `caesar_cipher(text, shift)`.

### xor_cipher(text, key)
XORs each character with the repeating `key`. Symmetric: running it again with the same key restores the text. Fails on an empty key (division by zero).

### bitwise_xor(a, b)
XOR of two non-negative integers, computed arithmetically for the low 9 bits only (values from 0 to 511).

### simple_hash(text)
DJB2 hash, an integer below 2^32. Not collision resistant.

### hash_hex(text)
`simple_hash` as eight lowercase hex digits.

### check_password_strength(password)
Scores a password out of 7 (one point each for length 8, 12 and 16, upper case, lower case, digit, special character). Returns `{"score", "max_score", "strength", "feedback"}` where `strength` is `"Weak"`, `"Medium"` (3 or more), `"Strong"` (5 or more) or `"Very Strong"` (7), and `feedback` lists suggestions such as `Add numbers`.

### vigenere_encrypt(text, key)
Vigenere cipher; the key letters advance only on letters in the text. The key must contain letters.

### vigenere_decrypt(text, key)
Reverses `vigenere_encrypt`.

### generate_simple_key(length = 16)
Random alphanumeric string. Not secure. (It never uses the final character, `9`.)

### reverse_str(s)
Reversed string.

### atbash(text)
Maps A to Z, B to Y, and so on, keeping case.

## Example

```mdhavers
fetch "stdlib/crypto"

blether base64_encode("Hello")
blether base64_decode("SGVsbG8=")
blether hex_encode("Hi")
blether rot13("Hello")
blether vigenere_encrypt("Attack at Dawn", "LEMON")
blether hash_hex("a")
blether check_password_strength("Abcdef1!xyz")["strength"]
```
