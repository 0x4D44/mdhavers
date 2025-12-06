" Vim syntax file
" Language: mdhavers (Scots Programming Language)
" Maintainer: mdhavers
" Latest Revision: 2025

if exists("b:current_syntax")
  finish
endif

" Comments
syn match mdhaversComment "#.*$" contains=mdhaversTodo
syn keyword mdhaversTodo contained TODO FIXME XXX NOTE

" Strings
syn region mdhaversString start='"' end='"' skip='\\\"' contains=mdhaversEscape,mdhaversInterpolation
syn region mdhaversString start="'" end="'" skip="\\'" contains=mdhaversEscape
syn region mdhaversFString start='f"' end='"' skip='\\\"' contains=mdhaversEscape,mdhaversInterpolation

" Interpolation in f-strings
syn region mdhaversInterpolation start='{' end='}' contained contains=TOP

" Escape sequences
syn match mdhaversEscape '\\.' contained

" Numbers
syn match mdhaversNumber '\<\d\+\>'
syn match mdhaversFloat '\<\d\+\.\d\+\>'

" Control keywords (Scots style!)
syn keyword mdhaversConditional gin ither
syn keyword mdhaversRepeat whiles fer
syn keyword mdhaversKeyword in brak haud keek whan
syn keyword mdhaversException hae_a_bash gin_it_gangs_wrang

" Declaration keywords
syn keyword mdhaversDeclaration ken dae kin thing fae

" Return
syn keyword mdhaversReturn gie

" Import
syn keyword mdhaversImport fetch tae

" Special keywords
syn keyword mdhaversSpecial blether mak_siccar masel

" Boolean constants
syn keyword mdhaversBoolean aye nae

" Null
syn keyword mdhaversNull naething

" Logical operators
syn keyword mdhaversOperator an or nae

" Built-in functions (the guid stuff!)
syn keyword mdhaversBuiltin len whit_kind tae_string tae_int tae_float
syn keyword mdhaversBuiltin shove yank keys values range
syn keyword mdhaversBuiltin abs min max floor ceil round sqrt
syn keyword mdhaversBuiltin split join contains reverse sort
syn keyword mdhaversBuiltin speir heid tail bum scran slap sumaw
syn keyword mdhaversBuiltin coont wheesht upper lower shuffle
syn keyword mdhaversBuiltin gaun sieve tumble aw ony hunt
syn keyword mdhaversBuiltin noo tick bide clype
syn keyword mdhaversBuiltin ceilidh dram birl stooshie sclaff
syn keyword mdhaversBuiltin blether_format creel empty_creel
syn keyword mdhaversBuiltin is_in_creel toss_in chuck_oot
syn keyword mdhaversBuiltin creels_thegither creels_baith creels_differ
syn keyword mdhaversBuiltin is_subset creel_tae_list
syn keyword mdhaversBuiltin jammy crabbit glaikit numpty_check
syn keyword mdhaversBuiltin och jings crivvens help_ma_boab
syn keyword mdhaversBuiltin roar mutter blooter

" Operators
syn match mdhaversOperator '\.\.\.'
syn match mdhaversOperator '\.\.'
syn match mdhaversOperator '|>'
syn match mdhaversOperator '=='
syn match mdhaversOperator '!='
syn match mdhaversOperator '<='
syn match mdhaversOperator '>='
syn match mdhaversOperator '[+\-*/%<>=]'

" Function calls
syn match mdhaversFunction '\<[a-zA-Z_][a-zA-Z0-9_]*\>\s*(' contains=mdhaversBuiltin

" Highlighting
hi def link mdhaversComment Comment
hi def link mdhaversTodo Todo
hi def link mdhaversString String
hi def link mdhaversFString String
hi def link mdhaversEscape Special
hi def link mdhaversInterpolation Special
hi def link mdhaversNumber Number
hi def link mdhaversFloat Float
hi def link mdhaversConditional Conditional
hi def link mdhaversRepeat Repeat
hi def link mdhaversKeyword Keyword
hi def link mdhaversException Exception
hi def link mdhaversDeclaration Statement
hi def link mdhaversReturn Statement
hi def link mdhaversImport Include
hi def link mdhaversSpecial Special
hi def link mdhaversBoolean Boolean
hi def link mdhaversNull Constant
hi def link mdhaversOperator Operator
hi def link mdhaversBuiltin Function
hi def link mdhaversFunction Function

let b:current_syntax = "mdhavers"
