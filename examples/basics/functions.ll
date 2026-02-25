; ModuleID = 'mdhavers_module'
source_filename = "mdhavers_module"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

@fmt_int = private constant [5 x i8] c"%lld\00"
@fmt_float = private constant [3 x i8] c"%g\00"
@fmt_string = private constant [3 x i8] c"%s\00"
@fmt_true = private constant [4 x i8] c"aye\00"
@fmt_false = private constant [4 x i8] c"nae\00"
@fmt_nil = private constant [9 x i8] c"naething\00"
@fmt_newline = private constant [2 x i8] c"\0A\00"
@str = private unnamed_addr constant [8 x i8] c"Hullo, \00", align 1
@str.1 = private unnamed_addr constant [17 x i8] c"! Hoo's it gaun?\00", align 1
@str.2 = private unnamed_addr constant [5 x i8] c"Jock\00", align 1
@str.3 = private unnamed_addr constant [6 x i8] c"Morag\00", align 1
@result = global { i8, i64 } zeroinitializer
@str.4 = private unnamed_addr constant [9 x i8] c"5 + 3 = \00", align 1
@nil_str = private unnamed_addr constant [9 x i8] c"naething\00", align 1
@true_str = private unnamed_addr constant [4 x i8] c"aye\00", align 1
@false_str = private unnamed_addr constant [4 x i8] c"nae\00", align 1
@int_fmt = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@float_fmt = private unnamed_addr constant [3 x i8] c"%g\00", align 1
@open_bracket = private unnamed_addr constant [2 x i8] c"[\00", align 1
@comma_sep = private unnamed_addr constant [3 x i8] c", \00", align 1
@float_fmt2 = private unnamed_addr constant [3 x i8] c"%g\00", align 1
@int_fmt2 = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@close_bracket = private unnamed_addr constant [2 x i8] c"]\00", align 1
@empty_str = private unnamed_addr constant [1 x i8] zeroinitializer, align 1
@str.5 = private unnamed_addr constant [12 x i8] c"Is 4 even? \00", align 1
@nil_str.6 = private unnamed_addr constant [9 x i8] c"naething\00", align 1
@true_str.7 = private unnamed_addr constant [4 x i8] c"aye\00", align 1
@false_str.8 = private unnamed_addr constant [4 x i8] c"nae\00", align 1
@int_fmt.9 = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@float_fmt.10 = private unnamed_addr constant [3 x i8] c"%g\00", align 1
@open_bracket.11 = private unnamed_addr constant [2 x i8] c"[\00", align 1
@comma_sep.12 = private unnamed_addr constant [3 x i8] c", \00", align 1
@float_fmt2.13 = private unnamed_addr constant [3 x i8] c"%g\00", align 1
@int_fmt2.14 = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@close_bracket.15 = private unnamed_addr constant [2 x i8] c"]\00", align 1
@empty_str.16 = private unnamed_addr constant [1 x i8] zeroinitializer, align 1
@str.17 = private unnamed_addr constant [12 x i8] c"Is 7 even? \00", align 1
@nil_str.18 = private unnamed_addr constant [9 x i8] c"naething\00", align 1
@true_str.19 = private unnamed_addr constant [4 x i8] c"aye\00", align 1
@false_str.20 = private unnamed_addr constant [4 x i8] c"nae\00", align 1
@int_fmt.21 = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@float_fmt.22 = private unnamed_addr constant [3 x i8] c"%g\00", align 1
@open_bracket.23 = private unnamed_addr constant [2 x i8] c"[\00", align 1
@comma_sep.24 = private unnamed_addr constant [3 x i8] c", \00", align 1
@float_fmt2.25 = private unnamed_addr constant [3 x i8] c"%g\00", align 1
@int_fmt2.26 = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@close_bracket.27 = private unnamed_addr constant [2 x i8] c"]\00", align 1
@empty_str.28 = private unnamed_addr constant [1 x i8] zeroinitializer, align 1
@str.29 = private unnamed_addr constant [1 x i8] zeroinitializer, align 1
@str.30 = private unnamed_addr constant [12 x i8] c"Factorials:\00", align 1
@nil_str.31 = private unnamed_addr constant [9 x i8] c"naething\00", align 1
@true_str.32 = private unnamed_addr constant [4 x i8] c"aye\00", align 1
@false_str.33 = private unnamed_addr constant [4 x i8] c"nae\00", align 1
@int_fmt.34 = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@float_fmt.35 = private unnamed_addr constant [3 x i8] c"%g\00", align 1
@open_bracket.36 = private unnamed_addr constant [2 x i8] c"[\00", align 1
@comma_sep.37 = private unnamed_addr constant [3 x i8] c", \00", align 1
@float_fmt2.38 = private unnamed_addr constant [3 x i8] c"%g\00", align 1
@int_fmt2.39 = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@close_bracket.40 = private unnamed_addr constant [2 x i8] c"]\00", align 1
@empty_str.41 = private unnamed_addr constant [1 x i8] zeroinitializer, align 1
@str.42 = private unnamed_addr constant [5 x i8] c"! = \00", align 1
@nil_str.43 = private unnamed_addr constant [9 x i8] c"naething\00", align 1
@true_str.44 = private unnamed_addr constant [4 x i8] c"aye\00", align 1
@false_str.45 = private unnamed_addr constant [4 x i8] c"nae\00", align 1
@int_fmt.46 = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@float_fmt.47 = private unnamed_addr constant [3 x i8] c"%g\00", align 1
@open_bracket.48 = private unnamed_addr constant [2 x i8] c"[\00", align 1
@comma_sep.49 = private unnamed_addr constant [3 x i8] c", \00", align 1
@float_fmt2.50 = private unnamed_addr constant [3 x i8] c"%g\00", align 1
@int_fmt2.51 = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@close_bracket.52 = private unnamed_addr constant [2 x i8] c"]\00", align 1
@empty_str.53 = private unnamed_addr constant [1 x i8] zeroinitializer, align 1
@str.54 = private unnamed_addr constant [1 x i8] zeroinitializer, align 1
@str.55 = private unnamed_addr constant [20 x i8] c"Fibonacci sequence:\00", align 1
@str.56 = private unnamed_addr constant [5 x i8] c"fib(\00", align 1
@nil_str.57 = private unnamed_addr constant [9 x i8] c"naething\00", align 1
@true_str.58 = private unnamed_addr constant [4 x i8] c"aye\00", align 1
@false_str.59 = private unnamed_addr constant [4 x i8] c"nae\00", align 1
@int_fmt.60 = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@float_fmt.61 = private unnamed_addr constant [3 x i8] c"%g\00", align 1
@open_bracket.62 = private unnamed_addr constant [2 x i8] c"[\00", align 1
@comma_sep.63 = private unnamed_addr constant [3 x i8] c", \00", align 1
@float_fmt2.64 = private unnamed_addr constant [3 x i8] c"%g\00", align 1
@int_fmt2.65 = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@close_bracket.66 = private unnamed_addr constant [2 x i8] c"]\00", align 1
@empty_str.67 = private unnamed_addr constant [1 x i8] zeroinitializer, align 1
@str.68 = private unnamed_addr constant [5 x i8] c") = \00", align 1
@nil_str.69 = private unnamed_addr constant [9 x i8] c"naething\00", align 1
@true_str.70 = private unnamed_addr constant [4 x i8] c"aye\00", align 1
@false_str.71 = private unnamed_addr constant [4 x i8] c"nae\00", align 1
@int_fmt.72 = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@float_fmt.73 = private unnamed_addr constant [3 x i8] c"%g\00", align 1
@open_bracket.74 = private unnamed_addr constant [2 x i8] c"[\00", align 1
@comma_sep.75 = private unnamed_addr constant [3 x i8] c", \00", align 1
@float_fmt2.76 = private unnamed_addr constant [3 x i8] c"%g\00", align 1
@int_fmt2.77 = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@close_bracket.78 = private unnamed_addr constant [2 x i8] c"]\00", align 1
@empty_str.79 = private unnamed_addr constant [1 x i8] zeroinitializer, align 1
@str.80 = private unnamed_addr constant [1 x i8] zeroinitializer, align 1
@str.81 = private unnamed_addr constant [17 x i8] c"Double 5 twice: \00", align 1
@nil_str.82 = private unnamed_addr constant [9 x i8] c"naething\00", align 1
@true_str.83 = private unnamed_addr constant [4 x i8] c"aye\00", align 1
@false_str.84 = private unnamed_addr constant [4 x i8] c"nae\00", align 1
@int_fmt.85 = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@float_fmt.86 = private unnamed_addr constant [3 x i8] c"%g\00", align 1
@open_bracket.87 = private unnamed_addr constant [2 x i8] c"[\00", align 1
@comma_sep.88 = private unnamed_addr constant [3 x i8] c", \00", align 1
@float_fmt2.89 = private unnamed_addr constant [3 x i8] c"%g\00", align 1
@int_fmt2.90 = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@close_bracket.91 = private unnamed_addr constant [2 x i8] c"]\00", align 1
@empty_str.92 = private unnamed_addr constant [1 x i8] zeroinitializer, align 1

declare i32 @printf(ptr, ...)

declare ptr @malloc(i64)

declare ptr @realloc(ptr, i64)

declare i64 @strlen(ptr)

declare ptr @strcpy(ptr, ptr)

declare ptr @strcat(ptr, ptr)

declare i32 @snprintf(ptr, i64, ptr, ...)

declare void @exit(i32)

declare ptr @strstr(ptr, ptr)

declare i32 @strcmp(ptr, ptr)

declare ptr @memcpy(ptr, ptr, i64)

declare i32 @toupper(i32)

declare i32 @tolower(i32)

declare i32 @isspace(i32)

declare i32 @clock_gettime(i32, ptr)

declare i32 @nanosleep(ptr, ptr)

declare ptr @fgets(ptr, i32, ptr)

declare ptr @strdup(ptr)

declare i32 @rand()

declare void @srand(i32)

declare i64 @time(ptr)

declare void @qsort(ptr, i64, i64, ptr)

declare { i8, i64 } @__mdh_get_key()

declare { i8, i64 } @__mdh_random(i64, i64)

declare { i8, i64 } @__mdh_term_width()

declare { i8, i64 } @__mdh_term_height()

define { i8, i64 } @greet({ i8, i64 } %0) {
entry:
  %name_shadow = alloca i64, align 8
  %name = alloca { i8, i64 }, align 8
  store { i8, i64 } %0, ptr %name, align 8
  %data = extractvalue { i8, i64 } %0, 1
  store i64 %data, ptr %name_shadow, align 8
  %name1 = load { i8, i64 }, ptr %name, align 8
  %tag = extractvalue { i8, i64 } %name1, 0
  %data2 = extractvalue { i8, i64 } %name1, 1
  %r_int = icmp eq i8 %tag, 2
  %both_int = and i1 false, %r_int
  %r_float = icmp eq i8 %tag, 3
  %either_float = or i1 false, %r_float
  %r_str = icmp eq i8 %tag, 4
  %both_str = and i1 true, %r_str
  br i1 %both_int, label %add_int_int, label %check_float

add_int_int:                                      ; preds = %entry
  %sum = add i64 ptrtoint (ptr @str to i64), %data2
  %v2 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %sum, 1
  br label %add_merge

add_float:                                        ; preds = %check_float
  %rf = bitcast i64 %data2 to double
  %ri2f = sitofp i64 %data2 to double
  %right_as_float = select i1 %r_float, double %rf, double %ri2f
  %fsum = fadd double sitofp (i64 ptrtoint (ptr @str to i64) to double), %right_as_float
  %float_bits = bitcast double %fsum to i64
  %v23 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits, 1
  br label %add_merge

add_string:                                       ; preds = %check_string
  %rstr = inttoptr i64 %data2 to ptr
  %llen = call i64 @strlen(ptr @str)
  %rlen = call i64 @strlen(ptr %rstr)
  %total = add i64 %llen, %rlen
  %alloc_size = add i64 %total, 1
  %new_str = call ptr @malloc(i64 %alloc_size)
  %1 = call ptr @memcpy(ptr %new_str, ptr @str, i64 %llen)
  %dest_offset = getelementptr i8, ptr %new_str, i64 %llen
  %rlen_plus_one = add i64 %rlen, 1
  %2 = call ptr @memcpy(ptr %dest_offset, ptr %rstr, i64 %rlen_plus_one)
  %str_ptr_int = ptrtoint ptr %new_str to i64
  %v24 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int, 1
  br label %add_merge

add_error:                                        ; preds = %check_string
  br label %add_merge

add_merge:                                        ; preds = %add_error, %add_string, %add_float, %add_int_int
  %add_result = phi { i8, i64 } [ %v2, %add_int_int ], [ %v23, %add_float ], [ %v24, %add_string ], [ zeroinitializer, %add_error ]
  %tag5 = extractvalue { i8, i64 } %add_result, 0
  %data6 = extractvalue { i8, i64 } %add_result, 1
  %l_int = icmp eq i8 %tag5, 2
  %both_int12 = and i1 %l_int, false
  %l_float = icmp eq i8 %tag5, 3
  %either_float13 = or i1 %l_float, false
  %l_str = icmp eq i8 %tag5, 4
  %both_str14 = and i1 %l_str, true
  br i1 %both_int12, label %add_int_int7, label %check_float15

check_float:                                      ; preds = %entry
  br i1 %either_float, label %add_float, label %check_string

check_string:                                     ; preds = %check_float
  br i1 %both_str, label %add_string, label %add_error

add_int_int7:                                     ; preds = %add_merge
  %sum17 = add i64 %data6, ptrtoint (ptr @str.1 to i64)
  %v218 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %sum17, 1
  br label %add_merge11

add_float8:                                       ; preds = %check_float15
  %lf = bitcast i64 %data6 to double
  %li2f = sitofp i64 %data6 to double
  %left_as_float = select i1 %l_float, double %lf, double %li2f
  %fsum19 = fadd double %left_as_float, sitofp (i64 ptrtoint (ptr @str.1 to i64) to double)
  %float_bits20 = bitcast double %fsum19 to i64
  %v221 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits20, 1
  br label %add_merge11

add_string9:                                      ; preds = %check_string16
  %lstr = inttoptr i64 %data6 to ptr
  %llen22 = call i64 @strlen(ptr %lstr)
  %rlen23 = call i64 @strlen(ptr @str.1)
  %total24 = add i64 %llen22, %rlen23
  %alloc_size25 = add i64 %total24, 1
  %new_str26 = call ptr @malloc(i64 %alloc_size25)
  %3 = call ptr @memcpy(ptr %new_str26, ptr %lstr, i64 %llen22)
  %dest_offset27 = getelementptr i8, ptr %new_str26, i64 %llen22
  %rlen_plus_one28 = add i64 %rlen23, 1
  %4 = call ptr @memcpy(ptr %dest_offset27, ptr @str.1, i64 %rlen_plus_one28)
  %str_ptr_int29 = ptrtoint ptr %new_str26 to i64
  %v230 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int29, 1
  br label %add_merge11

add_error10:                                      ; preds = %check_string16
  br label %add_merge11

add_merge11:                                      ; preds = %add_error10, %add_string9, %add_float8, %add_int_int7
  %add_result31 = phi { i8, i64 } [ %v218, %add_int_int7 ], [ %v221, %add_float8 ], [ %v230, %add_string9 ], [ zeroinitializer, %add_error10 ]
  %tag32 = extractvalue { i8, i64 } %add_result31, 0
  %data33 = extractvalue { i8, i64 } %add_result31, 1
  switch i8 %tag32, label %print_default [
    i8 0, label %print_nil
    i8 1, label %print_bool
    i8 2, label %print_int
    i8 3, label %print_float
    i8 4, label %print_string
  ]

check_float15:                                    ; preds = %add_merge
  br i1 %either_float13, label %add_float8, label %check_string16

check_string16:                                   ; preds = %check_float15
  br i1 %both_str14, label %add_string9, label %add_error10

print_nil:                                        ; preds = %add_merge11
  %5 = call i32 (ptr, ...) @printf(ptr @fmt_nil)
  br label %print_done

print_bool:                                       ; preds = %add_merge11
  %is_true = icmp ne i64 %data33, 0
  %bool_str = select i1 %is_true, ptr @fmt_true, ptr @fmt_false
  %6 = call i32 (ptr, ...) @printf(ptr %bool_str)
  br label %print_done

print_int:                                        ; preds = %add_merge11
  %7 = call i32 (ptr, ...) @printf(ptr @fmt_int, i64 %data33)
  br label %print_done

print_float:                                      ; preds = %add_merge11
  %f = bitcast i64 %data33 to double
  %8 = call i32 (ptr, ...) @printf(ptr @fmt_float, double %f)
  br label %print_done

print_string:                                     ; preds = %add_merge11
  %str = inttoptr i64 %data33 to ptr
  %9 = call i32 (ptr, ...) @printf(ptr @fmt_string, ptr %str)
  br label %print_done

print_default:                                    ; preds = %add_merge11
  br label %print_done

print_done:                                       ; preds = %print_default, %print_string, %print_float, %print_int, %print_bool, %print_nil
  %10 = call i32 (ptr, ...) @printf(ptr @fmt_newline)
  ret { i8, i64 } zeroinitializer
}

define { i8, i64 } @add({ i8, i64 } %0, { i8, i64 } %1) {
entry:
  %b_shadow = alloca i64, align 8
  %b = alloca { i8, i64 }, align 8
  %a_shadow = alloca i64, align 8
  %a = alloca { i8, i64 }, align 8
  store { i8, i64 } %0, ptr %a, align 8
  %data = extractvalue { i8, i64 } %0, 1
  store i64 %data, ptr %a_shadow, align 8
  store { i8, i64 } %1, ptr %b, align 8
  %data1 = extractvalue { i8, i64 } %1, 1
  store i64 %data1, ptr %b_shadow, align 8
  %a2 = load { i8, i64 }, ptr %a, align 8
  %b3 = load { i8, i64 }, ptr %b, align 8
  %tag = extractvalue { i8, i64 } %a2, 0
  %tag4 = extractvalue { i8, i64 } %b3, 0
  %data5 = extractvalue { i8, i64 } %a2, 1
  %data6 = extractvalue { i8, i64 } %b3, 1
  %l_int = icmp eq i8 %tag, 2
  %r_int = icmp eq i8 %tag4, 2
  %both_int = and i1 %l_int, %r_int
  %l_float = icmp eq i8 %tag, 3
  %r_float = icmp eq i8 %tag4, 3
  %either_float = or i1 %l_float, %r_float
  %l_str = icmp eq i8 %tag, 4
  %r_str = icmp eq i8 %tag4, 4
  %both_str = and i1 %l_str, %r_str
  br i1 %both_int, label %add_int_int, label %check_float

add_int_int:                                      ; preds = %entry
  %sum = add i64 %data5, %data6
  %v2 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %sum, 1
  br label %add_merge

add_float:                                        ; preds = %check_float
  %lf = bitcast i64 %data5 to double
  %li2f = sitofp i64 %data5 to double
  %left_as_float = select i1 %l_float, double %lf, double %li2f
  %rf = bitcast i64 %data6 to double
  %ri2f = sitofp i64 %data6 to double
  %right_as_float = select i1 %r_float, double %rf, double %ri2f
  %fsum = fadd double %left_as_float, %right_as_float
  %float_bits = bitcast double %fsum to i64
  %v27 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits, 1
  br label %add_merge

add_string:                                       ; preds = %check_string
  %lstr = inttoptr i64 %data5 to ptr
  %rstr = inttoptr i64 %data6 to ptr
  %llen = call i64 @strlen(ptr %lstr)
  %rlen = call i64 @strlen(ptr %rstr)
  %total = add i64 %llen, %rlen
  %alloc_size = add i64 %total, 1
  %new_str = call ptr @malloc(i64 %alloc_size)
  %2 = call ptr @memcpy(ptr %new_str, ptr %lstr, i64 %llen)
  %dest_offset = getelementptr i8, ptr %new_str, i64 %llen
  %rlen_plus_one = add i64 %rlen, 1
  %3 = call ptr @memcpy(ptr %dest_offset, ptr %rstr, i64 %rlen_plus_one)
  %str_ptr_int = ptrtoint ptr %new_str to i64
  %v28 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int, 1
  br label %add_merge

add_error:                                        ; preds = %check_string
  br label %add_merge

add_merge:                                        ; preds = %add_error, %add_string, %add_float, %add_int_int
  %add_result = phi { i8, i64 } [ %v2, %add_int_int ], [ %v27, %add_float ], [ %v28, %add_string ], [ zeroinitializer, %add_error ]
  ret { i8, i64 } %add_result

check_float:                                      ; preds = %entry
  br i1 %either_float, label %add_float, label %check_string

check_string:                                     ; preds = %check_float
  br i1 %both_str, label %add_string, label %add_error
}

define { i8, i64 } @is_even({ i8, i64 } %0) {
entry:
  %n_shadow = alloca i64, align 8
  %n = alloca { i8, i64 }, align 8
  store { i8, i64 } %0, ptr %n, align 8
  %data = extractvalue { i8, i64 } %0, 1
  store i64 %data, ptr %n_shadow, align 8
  %n1 = load { i8, i64 }, ptr %n, align 8
  %data2 = extractvalue { i8, i64 } %n1, 1
  %rem = srem i64 %data2, 2
  %v2 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %rem, 1
  %tag = extractvalue { i8, i64 } %v2, 0
  %data3 = extractvalue { i8, i64 } %v2, 1
  %tags_eq = icmp eq i8 %tag, 2
  %left_is_str = icmp eq i8 %tag, 4
  %both_str = and i1 %left_is_str, false
  br i1 %both_str, label %cmp_string, label %cmp_other

cmp_string:                                       ; preds = %entry
  %left_str = inttoptr i64 %data3 to ptr
  %strcmp_res = call i32 @strcmp(ptr %left_str, ptr null)
  %str_eq = icmp eq i32 %strcmp_res, 0
  br label %cmp_merge

cmp_other:                                        ; preds = %entry
  %data_eq = icmp eq i64 %data3, 0
  %other_eq = and i1 %tags_eq, %data_eq
  br label %cmp_merge

cmp_merge:                                        ; preds = %cmp_other, %cmp_string
  %eq_result = phi i1 [ %str_eq, %cmp_string ], [ %other_eq, %cmp_other ]
  %bool_ext = zext i1 %eq_result to i64
  %v24 = insertvalue { i8, i64 } { i8 1, i64 undef }, i64 %bool_ext, 1
  %tag5 = extractvalue { i8, i64 } %v24, 0
  %data6 = extractvalue { i8, i64 } %v24, 1
  switch i8 %tag5, label %is_other [
    i8 0, label %is_nil
    i8 1, label %is_bool
    i8 2, label %is_int
  ]

is_nil:                                           ; preds = %cmp_merge
  br label %truthy_merge

is_bool:                                          ; preds = %cmp_merge
  %bool_val = trunc i64 %data6 to i1
  br label %truthy_merge

is_int:                                           ; preds = %cmp_merge
  %int_truthy = icmp ne i64 %data6, 0
  br label %truthy_merge

is_other:                                         ; preds = %cmp_merge
  br label %truthy_merge

truthy_merge:                                     ; preds = %is_other, %is_int, %is_bool, %is_nil
  %truthy = phi i1 [ false, %is_nil ], [ %bool_val, %is_bool ], [ %int_truthy, %is_int ], [ true, %is_other ]
  br i1 %truthy, label %then, label %else

then:                                             ; preds = %truthy_merge
  ret { i8, i64 } { i8 1, i64 1 }

else:                                             ; preds = %truthy_merge
  br label %merge

merge:                                            ; preds = %else
  ret { i8, i64 } { i8 1, i64 0 }
}

define { i8, i64 } @factorial({ i8, i64 } %0) {
entry:
  %n_shadow = alloca i64, align 8
  %n = alloca { i8, i64 }, align 8
  store { i8, i64 } %0, ptr %n, align 8
  %data = extractvalue { i8, i64 } %0, 1
  store i64 %data, ptr %n_shadow, align 8
  %n_i64 = load i64, ptr %n_shadow, align 8
  %cmp_direct = icmp sle i64 %n_i64, 1
  br i1 %cmp_direct, label %then, label %else

then:                                             ; preds = %entry
  ret { i8, i64 } { i8 2, i64 1 }

else:                                             ; preds = %entry
  br label %merge

merge:                                            ; preds = %else
  %n1 = load { i8, i64 }, ptr %n, align 8
  %n2 = load { i8, i64 }, ptr %n, align 8
  %tag = extractvalue { i8, i64 } %n2, 0
  %data3 = extractvalue { i8, i64 } %n2, 1
  %l_int = icmp eq i8 %tag, 2
  %both_int = and i1 %l_int, true
  br i1 %both_int, label %sub_int, label %sub_float

sub_int:                                          ; preds = %merge
  %diff = sub i64 %data3, 1
  %v2 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %diff, 1
  br label %sub_merge

sub_float:                                        ; preds = %merge
  %lf = icmp eq i8 %tag, 3
  %lf4 = bitcast i64 %data3 to double
  %li2f = sitofp i64 %data3 to double
  %left_as_float = select i1 %lf, double %lf4, double %li2f
  %fdiff = fsub double %left_as_float, 1.000000e+00
  %float_bits = bitcast double %fdiff to i64
  %v25 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits, 1
  br label %sub_merge

sub_merge:                                        ; preds = %sub_float, %sub_int
  %sub_result = phi { i8, i64 } [ %v2, %sub_int ], [ %v25, %sub_float ]
  %call = tail call { i8, i64 } @factorial({ i8, i64 } %sub_result)
  %tag6 = extractvalue { i8, i64 } %n1, 0
  %tag7 = extractvalue { i8, i64 } %call, 0
  %data8 = extractvalue { i8, i64 } %n1, 1
  %data9 = extractvalue { i8, i64 } %call, 1
  %l_int10 = icmp eq i8 %tag6, 2
  %r_int = icmp eq i8 %tag7, 2
  %both_int11 = and i1 %l_int10, %r_int
  br i1 %both_int11, label %mul_int, label %mul_float

mul_int:                                          ; preds = %sub_merge
  %prod = mul i64 %data8, %data9
  %v212 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %prod, 1
  br label %mul_merge

mul_float:                                        ; preds = %sub_merge
  %lf13 = icmp eq i8 %tag6, 3
  %rf = icmp eq i8 %tag7, 3
  %lf14 = bitcast i64 %data8 to double
  %li2f15 = sitofp i64 %data8 to double
  %left_as_float16 = select i1 %lf13, double %lf14, double %li2f15
  %rf17 = bitcast i64 %data9 to double
  %ri2f = sitofp i64 %data9 to double
  %right_as_float = select i1 %rf, double %rf17, double %ri2f
  %fprod = fmul double %left_as_float16, %right_as_float
  %float_bits18 = bitcast double %fprod to i64
  %v219 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits18, 1
  br label %mul_merge

mul_merge:                                        ; preds = %mul_float, %mul_int
  %mul_result = phi { i8, i64 } [ %v212, %mul_int ], [ %v219, %mul_float ]
  ret { i8, i64 } %mul_result
}

define { i8, i64 } @fibonacci({ i8, i64 } %0) {
entry:
  %n_shadow = alloca i64, align 8
  %n = alloca { i8, i64 }, align 8
  store { i8, i64 } %0, ptr %n, align 8
  %data = extractvalue { i8, i64 } %0, 1
  store i64 %data, ptr %n_shadow, align 8
  %n_i64 = load i64, ptr %n_shadow, align 8
  %cmp_direct = icmp sle i64 %n_i64, 1
  br i1 %cmp_direct, label %then, label %else

then:                                             ; preds = %entry
  %n1 = load { i8, i64 }, ptr %n, align 8
  ret { i8, i64 } %n1

else:                                             ; preds = %entry
  br label %merge

merge:                                            ; preds = %else
  %n2 = load { i8, i64 }, ptr %n, align 8
  %tag = extractvalue { i8, i64 } %n2, 0
  %data3 = extractvalue { i8, i64 } %n2, 1
  %l_int = icmp eq i8 %tag, 2
  %both_int = and i1 %l_int, true
  br i1 %both_int, label %sub_int, label %sub_float

sub_int:                                          ; preds = %merge
  %diff = sub i64 %data3, 1
  %v2 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %diff, 1
  br label %sub_merge

sub_float:                                        ; preds = %merge
  %lf = icmp eq i8 %tag, 3
  %lf4 = bitcast i64 %data3 to double
  %li2f = sitofp i64 %data3 to double
  %left_as_float = select i1 %lf, double %lf4, double %li2f
  %fdiff = fsub double %left_as_float, 1.000000e+00
  %float_bits = bitcast double %fdiff to i64
  %v25 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits, 1
  br label %sub_merge

sub_merge:                                        ; preds = %sub_float, %sub_int
  %sub_result = phi { i8, i64 } [ %v2, %sub_int ], [ %v25, %sub_float ]
  %call = tail call { i8, i64 } @fibonacci({ i8, i64 } %sub_result)
  %n6 = load { i8, i64 }, ptr %n, align 8
  %tag7 = extractvalue { i8, i64 } %n6, 0
  %data8 = extractvalue { i8, i64 } %n6, 1
  %l_int12 = icmp eq i8 %tag7, 2
  %both_int13 = and i1 %l_int12, true
  br i1 %both_int13, label %sub_int9, label %sub_float10

sub_int9:                                         ; preds = %sub_merge
  %diff14 = sub i64 %data8, 2
  %v215 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %diff14, 1
  br label %sub_merge11

sub_float10:                                      ; preds = %sub_merge
  %lf16 = icmp eq i8 %tag7, 3
  %lf17 = bitcast i64 %data8 to double
  %li2f18 = sitofp i64 %data8 to double
  %left_as_float19 = select i1 %lf16, double %lf17, double %li2f18
  %fdiff20 = fsub double %left_as_float19, 2.000000e+00
  %float_bits21 = bitcast double %fdiff20 to i64
  %v222 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits21, 1
  br label %sub_merge11

sub_merge11:                                      ; preds = %sub_float10, %sub_int9
  %sub_result23 = phi { i8, i64 } [ %v215, %sub_int9 ], [ %v222, %sub_float10 ]
  %call24 = tail call { i8, i64 } @fibonacci({ i8, i64 } %sub_result23)
  %tag25 = extractvalue { i8, i64 } %call, 0
  %tag26 = extractvalue { i8, i64 } %call24, 0
  %data27 = extractvalue { i8, i64 } %call, 1
  %data28 = extractvalue { i8, i64 } %call24, 1
  %l_int29 = icmp eq i8 %tag25, 2
  %r_int = icmp eq i8 %tag26, 2
  %both_int30 = and i1 %l_int29, %r_int
  %l_float = icmp eq i8 %tag25, 3
  %r_float = icmp eq i8 %tag26, 3
  %either_float = or i1 %l_float, %r_float
  %l_str = icmp eq i8 %tag25, 4
  %r_str = icmp eq i8 %tag26, 4
  %both_str = and i1 %l_str, %r_str
  br i1 %both_int30, label %add_int_int, label %check_float

add_int_int:                                      ; preds = %sub_merge11
  %sum = add i64 %data27, %data28
  %v231 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %sum, 1
  br label %add_merge

add_float:                                        ; preds = %check_float
  %lf32 = bitcast i64 %data27 to double
  %li2f33 = sitofp i64 %data27 to double
  %left_as_float34 = select i1 %l_float, double %lf32, double %li2f33
  %rf = bitcast i64 %data28 to double
  %ri2f = sitofp i64 %data28 to double
  %right_as_float = select i1 %r_float, double %rf, double %ri2f
  %fsum = fadd double %left_as_float34, %right_as_float
  %float_bits35 = bitcast double %fsum to i64
  %v236 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits35, 1
  br label %add_merge

add_string:                                       ; preds = %check_string
  %lstr = inttoptr i64 %data27 to ptr
  %rstr = inttoptr i64 %data28 to ptr
  %llen = call i64 @strlen(ptr %lstr)
  %rlen = call i64 @strlen(ptr %rstr)
  %total = add i64 %llen, %rlen
  %alloc_size = add i64 %total, 1
  %new_str = call ptr @malloc(i64 %alloc_size)
  %1 = call ptr @memcpy(ptr %new_str, ptr %lstr, i64 %llen)
  %dest_offset = getelementptr i8, ptr %new_str, i64 %llen
  %rlen_plus_one = add i64 %rlen, 1
  %2 = call ptr @memcpy(ptr %dest_offset, ptr %rstr, i64 %rlen_plus_one)
  %str_ptr_int = ptrtoint ptr %new_str to i64
  %v237 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int, 1
  br label %add_merge

add_error:                                        ; preds = %check_string
  br label %add_merge

add_merge:                                        ; preds = %add_error, %add_string, %add_float, %add_int_int
  %add_result = phi { i8, i64 } [ %v231, %add_int_int ], [ %v236, %add_float ], [ %v237, %add_string ], [ zeroinitializer, %add_error ]
  ret { i8, i64 } %add_result

check_float:                                      ; preds = %sub_merge11
  br i1 %either_float, label %add_float, label %check_string

check_string:                                     ; preds = %check_float
  br i1 %both_str, label %add_string, label %add_error
}

define { i8, i64 } @apply_twice({ i8, i64 } %0, { i8, i64 } %1) {
entry:
  %f_shadow = alloca i64, align 8
  %f = alloca { i8, i64 }, align 8
  %n_shadow = alloca i64, align 8
  %n = alloca { i8, i64 }, align 8
  store { i8, i64 } %0, ptr %n, align 8
  %data = extractvalue { i8, i64 } %0, 1
  store i64 %data, ptr %n_shadow, align 8
  store { i8, i64 } %1, ptr %f, align 8
  %data1 = extractvalue { i8, i64 } %1, 1
  store i64 %data1, ptr %f_shadow, align 8
  %func_val = load { i8, i64 }, ptr %f, align 8
  %func_val2 = load { i8, i64 }, ptr %f, align 8
  %n3 = load { i8, i64 }, ptr %n, align 8
  %func_data = extractvalue { i8, i64 } %func_val2, 1
  %fn_ptr = inttoptr i64 %func_data to ptr
  %call_result = call { i8, i64 } %fn_ptr({ i8, i64 } %n3)
  %func_data4 = extractvalue { i8, i64 } %func_val, 1
  %fn_ptr5 = inttoptr i64 %func_data4 to ptr
  %call_result6 = call { i8, i64 } %fn_ptr5({ i8, i64 } %call_result)
  ret { i8, i64 } %call_result6
}

define { i8, i64 } @double({ i8, i64 } %0) {
entry:
  %x_shadow = alloca i64, align 8
  %x = alloca { i8, i64 }, align 8
  store { i8, i64 } %0, ptr %x, align 8
  %data = extractvalue { i8, i64 } %0, 1
  store i64 %data, ptr %x_shadow, align 8
  %x1 = load { i8, i64 }, ptr %x, align 8
  %tag = extractvalue { i8, i64 } %x1, 0
  %data2 = extractvalue { i8, i64 } %x1, 1
  %l_int = icmp eq i8 %tag, 2
  %both_int = and i1 %l_int, true
  br i1 %both_int, label %mul_int, label %mul_float

mul_int:                                          ; preds = %entry
  %prod = mul i64 %data2, 2
  %v2 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %prod, 1
  br label %mul_merge

mul_float:                                        ; preds = %entry
  %lf = icmp eq i8 %tag, 3
  %lf3 = bitcast i64 %data2 to double
  %li2f = sitofp i64 %data2 to double
  %left_as_float = select i1 %lf, double %lf3, double %li2f
  %fprod = fmul double %left_as_float, 2.000000e+00
  %float_bits = bitcast double %fprod to i64
  %v24 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits, 1
  br label %mul_merge

mul_merge:                                        ; preds = %mul_float, %mul_int
  %mul_result = phi { i8, i64 } [ %v2, %mul_int ], [ %v24, %mul_float ]
  ret { i8, i64 } %mul_result
}

define i32 @main() {
entry:
  %i478 = alloca { i8, i64 }, align 8
  %i = alloca { i8, i64 }, align 8
  %call = tail call { i8, i64 } @greet({ i8, i64 } { i8 4, i64 ptrtoint (ptr @str.2 to i64) })
  %call1 = tail call { i8, i64 } @greet({ i8, i64 } { i8 4, i64 ptrtoint (ptr @str.3 to i64) })
  %call2 = tail call { i8, i64 } @add({ i8, i64 } { i8 2, i64 5 }, { i8, i64 } { i8 2, i64 3 })
  store { i8, i64 } %call2, ptr @result, align 8
  %result = load { i8, i64 }, ptr @result, align 8
  %tag = extractvalue { i8, i64 } %result, 0
  %data = extractvalue { i8, i64 } %result, 1
  switch i8 %tag, label %str_default [
    i8 0, label %str_nil
    i8 1, label %str_bool
    i8 2, label %str_int
    i8 3, label %str_float
    i8 4, label %str_string
    i8 5, label %str_list
  ]

str_nil:                                          ; preds = %entry
  br label %str_merge

str_bool:                                         ; preds = %entry
  %is_true = icmp ne i64 %data, 0
  %bool_ptr = select i1 %is_true, ptr @true_str, ptr @false_str
  %str_ptr_int = ptrtoint ptr %bool_ptr to i64
  %v2 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int, 1
  br label %str_merge

str_int:                                          ; preds = %entry
  %int_buf = call ptr @malloc(i64 32)
  %0 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %int_buf, i64 32, ptr @int_fmt, i64 %data)
  %str_ptr_int3 = ptrtoint ptr %int_buf to i64
  %v24 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int3, 1
  br label %str_merge

str_float:                                        ; preds = %entry
  %float_buf = call ptr @malloc(i64 32)
  %f = bitcast i64 %data to double
  %1 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %float_buf, i64 32, ptr @float_fmt, double %f)
  %str_ptr_int5 = ptrtoint ptr %float_buf to i64
  %v26 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int5, 1
  br label %str_merge

str_string:                                       ; preds = %entry
  br label %str_merge

str_default:                                      ; preds = %entry
  br label %str_merge

str_merge:                                        ; preds = %str_default, %list_loop_end, %str_string, %str_float, %str_int, %str_bool, %str_nil
  %str_result = phi { i8, i64 } [ { i8 4, i64 ptrtoint (ptr @nil_str to i64) }, %str_nil ], [ %v2, %str_bool ], [ %v24, %str_int ], [ %v26, %str_float ], [ %result, %str_string ], [ %v28, %list_loop_end ], [ { i8 4, i64 ptrtoint (ptr @empty_str to i64) }, %str_default ]
  %tag9 = extractvalue { i8, i64 } %str_result, 0
  %data10 = extractvalue { i8, i64 } %str_result, 1
  %r_int = icmp eq i8 %tag9, 2
  %both_int = and i1 false, %r_int
  %r_float = icmp eq i8 %tag9, 3
  %either_float = or i1 false, %r_float
  %r_str = icmp eq i8 %tag9, 4
  %both_str = and i1 true, %r_str
  br i1 %both_int, label %add_int_int, label %check_float

str_list:                                         ; preds = %entry
  %list_ptr = inttoptr i64 %data to ptr
  %len_ptr = getelementptr i64, ptr %list_ptr, i64 1
  %list_len = load i64, ptr %len_ptr, align 8
  %buf_size_mul = mul i64 %list_len, 25
  %list_buf_size = add i64 %buf_size_mul, 3
  %list_buf = call ptr @malloc(i64 %list_buf_size)
  %2 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %list_buf, i64 %list_buf_size, ptr @open_bracket)
  %idx_ptr = alloca i64, align 8
  store i64 0, ptr %idx_ptr, align 8
  br label %list_loop_header

list_loop_header:                                 ; preds = %elem_done, %str_list
  %idx = load i64, ptr %idx_ptr, align 8
  %loop_cond = icmp ult i64 %idx, %list_len
  br i1 %loop_cond, label %list_loop_body, label %list_loop_end

list_loop_body:                                   ; preds = %list_loop_header
  %is_first = icmp eq i64 %idx, 0
  br i1 %is_first, label %elem_block, label %sep_block

list_loop_end:                                    ; preds = %list_loop_header
  %3 = call ptr @strcat(ptr %list_buf, ptr @close_bracket)
  %str_ptr_int7 = ptrtoint ptr %list_buf to i64
  %v28 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int7, 1
  br label %str_merge

sep_block:                                        ; preds = %list_loop_body
  %4 = call ptr @strcat(ptr %list_buf, ptr @comma_sep)
  br label %elem_block

elem_block:                                       ; preds = %sep_block, %list_loop_body
  %idx_in_elem = load i64, ptr %idx_ptr, align 8
  %elements_base = getelementptr i64, ptr %len_ptr, i64 1
  %elem_ptr = getelementptr { i8, i64 }, ptr %elements_base, i64 %idx_in_elem
  %elem_val = load { i8, i64 }, ptr %elem_ptr, align 8
  %elem_tag = extractvalue { i8, i64 } %elem_val, 0
  %elem_data = extractvalue { i8, i64 } %elem_val, 1
  %elem_data_ptr = alloca i64, align 8
  store i64 %elem_data, ptr %elem_data_ptr, align 8
  %elem_is_float = icmp eq i8 %elem_tag, 3
  %elem_is_string = icmp eq i8 %elem_tag, 4
  br i1 %elem_is_float, label %elem_float_block, label %elem_string_check

elem_float_block:                                 ; preds = %elem_block
  %elem_data_float = load i64, ptr %elem_data_ptr, align 8
  %elem_float_buf = call ptr @malloc(i64 25)
  %elem_as_float = bitcast i64 %elem_data_float to double
  %5 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_float_buf, i64 25, ptr @float_fmt2, double %elem_as_float)
  %6 = call ptr @strcat(ptr %list_buf, ptr %elem_float_buf)
  br label %elem_done

elem_string_check:                                ; preds = %elem_block
  br i1 %elem_is_string, label %elem_string_print, label %elem_int_block

elem_string_print:                                ; preds = %elem_string_check
  %elem_data_str = load i64, ptr %elem_data_ptr, align 8
  %elem_str_ptr = inttoptr i64 %elem_data_str to ptr
  %7 = call ptr @strcat(ptr %list_buf, ptr %elem_str_ptr)
  br label %elem_done

elem_int_block:                                   ; preds = %elem_string_check
  %elem_data_int = load i64, ptr %elem_data_ptr, align 8
  %elem_int_buf = call ptr @malloc(i64 25)
  %8 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_int_buf, i64 25, ptr @int_fmt2, i64 %elem_data_int)
  %9 = call ptr @strcat(ptr %list_buf, ptr %elem_int_buf)
  br label %elem_done

elem_done:                                        ; preds = %elem_int_block, %elem_float_block, %elem_string_print
  %idx_for_incr = load i64, ptr %idx_ptr, align 8
  %next_idx = add i64 %idx_for_incr, 1
  store i64 %next_idx, ptr %idx_ptr, align 8
  br label %list_loop_header

add_int_int:                                      ; preds = %str_merge
  %sum = add i64 ptrtoint (ptr @str.4 to i64), %data10
  %v211 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %sum, 1
  br label %add_merge

add_float:                                        ; preds = %check_float
  %rf = bitcast i64 %data10 to double
  %ri2f = sitofp i64 %data10 to double
  %right_as_float = select i1 %r_float, double %rf, double %ri2f
  %fsum = fadd double sitofp (i64 ptrtoint (ptr @str.4 to i64) to double), %right_as_float
  %float_bits = bitcast double %fsum to i64
  %v212 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits, 1
  br label %add_merge

add_string:                                       ; preds = %check_string
  %rstr = inttoptr i64 %data10 to ptr
  %llen = call i64 @strlen(ptr @str.4)
  %rlen = call i64 @strlen(ptr %rstr)
  %total = add i64 %llen, %rlen
  %alloc_size = add i64 %total, 1
  %new_str = call ptr @malloc(i64 %alloc_size)
  %10 = call ptr @memcpy(ptr %new_str, ptr @str.4, i64 %llen)
  %dest_offset = getelementptr i8, ptr %new_str, i64 %llen
  %rlen_plus_one = add i64 %rlen, 1
  %11 = call ptr @memcpy(ptr %dest_offset, ptr %rstr, i64 %rlen_plus_one)
  %str_ptr_int13 = ptrtoint ptr %new_str to i64
  %v214 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int13, 1
  br label %add_merge

add_error:                                        ; preds = %check_string
  br label %add_merge

add_merge:                                        ; preds = %add_error, %add_string, %add_float, %add_int_int
  %add_result = phi { i8, i64 } [ %v211, %add_int_int ], [ %v212, %add_float ], [ %v214, %add_string ], [ zeroinitializer, %add_error ]
  %tag15 = extractvalue { i8, i64 } %add_result, 0
  %data16 = extractvalue { i8, i64 } %add_result, 1
  switch i8 %tag15, label %print_default [
    i8 0, label %print_nil
    i8 1, label %print_bool
    i8 2, label %print_int
    i8 3, label %print_float
    i8 4, label %print_string
  ]

check_float:                                      ; preds = %str_merge
  br i1 %either_float, label %add_float, label %check_string

check_string:                                     ; preds = %check_float
  br i1 %both_str, label %add_string, label %add_error

print_nil:                                        ; preds = %add_merge
  %12 = call i32 (ptr, ...) @printf(ptr @fmt_nil)
  br label %print_done

print_bool:                                       ; preds = %add_merge
  %is_true17 = icmp ne i64 %data16, 0
  %bool_str = select i1 %is_true17, ptr @fmt_true, ptr @fmt_false
  %13 = call i32 (ptr, ...) @printf(ptr %bool_str)
  br label %print_done

print_int:                                        ; preds = %add_merge
  %14 = call i32 (ptr, ...) @printf(ptr @fmt_int, i64 %data16)
  br label %print_done

print_float:                                      ; preds = %add_merge
  %f18 = bitcast i64 %data16 to double
  %15 = call i32 (ptr, ...) @printf(ptr @fmt_float, double %f18)
  br label %print_done

print_string:                                     ; preds = %add_merge
  %str = inttoptr i64 %data16 to ptr
  %16 = call i32 (ptr, ...) @printf(ptr @fmt_string, ptr %str)
  br label %print_done

print_default:                                    ; preds = %add_merge
  br label %print_done

print_done:                                       ; preds = %print_default, %print_string, %print_float, %print_int, %print_bool, %print_nil
  %17 = call i32 (ptr, ...) @printf(ptr @fmt_newline)
  %call19 = tail call { i8, i64 } @is_even({ i8, i64 } { i8 2, i64 4 })
  %tag20 = extractvalue { i8, i64 } %call19, 0
  %data21 = extractvalue { i8, i64 } %call19, 1
  switch i8 %tag20, label %str_default27 [
    i8 0, label %str_nil22
    i8 1, label %str_bool23
    i8 2, label %str_int24
    i8 3, label %str_float25
    i8 4, label %str_string26
    i8 5, label %str_list29
  ]

str_nil22:                                        ; preds = %print_done
  br label %str_merge28

str_bool23:                                       ; preds = %print_done
  %is_true30 = icmp ne i64 %data21, 0
  %bool_ptr31 = select i1 %is_true30, ptr @true_str.7, ptr @false_str.8
  %str_ptr_int32 = ptrtoint ptr %bool_ptr31 to i64
  %v233 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int32, 1
  br label %str_merge28

str_int24:                                        ; preds = %print_done
  %int_buf34 = call ptr @malloc(i64 32)
  %18 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %int_buf34, i64 32, ptr @int_fmt.9, i64 %data21)
  %str_ptr_int35 = ptrtoint ptr %int_buf34 to i64
  %v236 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int35, 1
  br label %str_merge28

str_float25:                                      ; preds = %print_done
  %float_buf37 = call ptr @malloc(i64 32)
  %f38 = bitcast i64 %data21 to double
  %19 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %float_buf37, i64 32, ptr @float_fmt.10, double %f38)
  %str_ptr_int39 = ptrtoint ptr %float_buf37 to i64
  %v240 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int39, 1
  br label %str_merge28

str_string26:                                     ; preds = %print_done
  br label %str_merge28

str_default27:                                    ; preds = %print_done
  br label %str_merge28

str_merge28:                                      ; preds = %str_default27, %list_loop_end49, %str_string26, %str_float25, %str_int24, %str_bool23, %str_nil22
  %str_result81 = phi { i8, i64 } [ { i8 4, i64 ptrtoint (ptr @nil_str.6 to i64) }, %str_nil22 ], [ %v233, %str_bool23 ], [ %v236, %str_int24 ], [ %v240, %str_float25 ], [ %call19, %str_string26 ], [ %v280, %list_loop_end49 ], [ { i8 4, i64 ptrtoint (ptr @empty_str.16 to i64) }, %str_default27 ]
  %tag82 = extractvalue { i8, i64 } %str_result81, 0
  %data83 = extractvalue { i8, i64 } %str_result81, 1
  %r_int89 = icmp eq i8 %tag82, 2
  %both_int90 = and i1 false, %r_int89
  %r_float91 = icmp eq i8 %tag82, 3
  %either_float92 = or i1 false, %r_float91
  %r_str93 = icmp eq i8 %tag82, 4
  %both_str94 = and i1 true, %r_str93
  br i1 %both_int90, label %add_int_int84, label %check_float95

str_list29:                                       ; preds = %print_done
  %list_ptr41 = inttoptr i64 %data21 to ptr
  %len_ptr42 = getelementptr i64, ptr %list_ptr41, i64 1
  %list_len43 = load i64, ptr %len_ptr42, align 8
  %buf_size_mul44 = mul i64 %list_len43, 25
  %list_buf_size45 = add i64 %buf_size_mul44, 3
  %list_buf46 = call ptr @malloc(i64 %list_buf_size45)
  %20 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %list_buf46, i64 %list_buf_size45, ptr @open_bracket.11)
  %idx_ptr50 = alloca i64, align 8
  store i64 0, ptr %idx_ptr50, align 8
  br label %list_loop_header47

list_loop_header47:                               ; preds = %elem_done69, %str_list29
  %idx51 = load i64, ptr %idx_ptr50, align 8
  %loop_cond52 = icmp ult i64 %idx51, %list_len43
  br i1 %loop_cond52, label %list_loop_body48, label %list_loop_end49

list_loop_body48:                                 ; preds = %list_loop_header47
  %is_first53 = icmp eq i64 %idx51, 0
  br i1 %is_first53, label %elem_block55, label %sep_block54

list_loop_end49:                                  ; preds = %list_loop_header47
  %21 = call ptr @strcat(ptr %list_buf46, ptr @close_bracket.15)
  %str_ptr_int79 = ptrtoint ptr %list_buf46 to i64
  %v280 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int79, 1
  br label %str_merge28

sep_block54:                                      ; preds = %list_loop_body48
  %22 = call ptr @strcat(ptr %list_buf46, ptr @comma_sep.12)
  br label %elem_block55

elem_block55:                                     ; preds = %sep_block54, %list_loop_body48
  %idx_in_elem56 = load i64, ptr %idx_ptr50, align 8
  %elements_base57 = getelementptr i64, ptr %len_ptr42, i64 1
  %elem_ptr58 = getelementptr { i8, i64 }, ptr %elements_base57, i64 %idx_in_elem56
  %elem_val59 = load { i8, i64 }, ptr %elem_ptr58, align 8
  %elem_tag60 = extractvalue { i8, i64 } %elem_val59, 0
  %elem_data61 = extractvalue { i8, i64 } %elem_val59, 1
  %elem_data_ptr62 = alloca i64, align 8
  store i64 %elem_data61, ptr %elem_data_ptr62, align 8
  %elem_is_float63 = icmp eq i8 %elem_tag60, 3
  %elem_is_string64 = icmp eq i8 %elem_tag60, 4
  br i1 %elem_is_float63, label %elem_float_block65, label %elem_string_check66

elem_float_block65:                               ; preds = %elem_block55
  %elem_data_float72 = load i64, ptr %elem_data_ptr62, align 8
  %elem_float_buf73 = call ptr @malloc(i64 25)
  %elem_as_float74 = bitcast i64 %elem_data_float72 to double
  %23 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_float_buf73, i64 25, ptr @float_fmt2.13, double %elem_as_float74)
  %24 = call ptr @strcat(ptr %list_buf46, ptr %elem_float_buf73)
  br label %elem_done69

elem_string_check66:                              ; preds = %elem_block55
  br i1 %elem_is_string64, label %elem_string_print67, label %elem_int_block68

elem_string_print67:                              ; preds = %elem_string_check66
  %elem_data_str70 = load i64, ptr %elem_data_ptr62, align 8
  %elem_str_ptr71 = inttoptr i64 %elem_data_str70 to ptr
  %25 = call ptr @strcat(ptr %list_buf46, ptr %elem_str_ptr71)
  br label %elem_done69

elem_int_block68:                                 ; preds = %elem_string_check66
  %elem_data_int75 = load i64, ptr %elem_data_ptr62, align 8
  %elem_int_buf76 = call ptr @malloc(i64 25)
  %26 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_int_buf76, i64 25, ptr @int_fmt2.14, i64 %elem_data_int75)
  %27 = call ptr @strcat(ptr %list_buf46, ptr %elem_int_buf76)
  br label %elem_done69

elem_done69:                                      ; preds = %elem_int_block68, %elem_float_block65, %elem_string_print67
  %idx_for_incr77 = load i64, ptr %idx_ptr50, align 8
  %next_idx78 = add i64 %idx_for_incr77, 1
  store i64 %next_idx78, ptr %idx_ptr50, align 8
  br label %list_loop_header47

add_int_int84:                                    ; preds = %str_merge28
  %sum97 = add i64 ptrtoint (ptr @str.5 to i64), %data83
  %v298 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %sum97, 1
  br label %add_merge88

add_float85:                                      ; preds = %check_float95
  %rf99 = bitcast i64 %data83 to double
  %ri2f100 = sitofp i64 %data83 to double
  %right_as_float101 = select i1 %r_float91, double %rf99, double %ri2f100
  %fsum102 = fadd double sitofp (i64 ptrtoint (ptr @str.5 to i64) to double), %right_as_float101
  %float_bits103 = bitcast double %fsum102 to i64
  %v2104 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits103, 1
  br label %add_merge88

add_string86:                                     ; preds = %check_string96
  %rstr105 = inttoptr i64 %data83 to ptr
  %llen106 = call i64 @strlen(ptr @str.5)
  %rlen107 = call i64 @strlen(ptr %rstr105)
  %total108 = add i64 %llen106, %rlen107
  %alloc_size109 = add i64 %total108, 1
  %new_str110 = call ptr @malloc(i64 %alloc_size109)
  %28 = call ptr @memcpy(ptr %new_str110, ptr @str.5, i64 %llen106)
  %dest_offset111 = getelementptr i8, ptr %new_str110, i64 %llen106
  %rlen_plus_one112 = add i64 %rlen107, 1
  %29 = call ptr @memcpy(ptr %dest_offset111, ptr %rstr105, i64 %rlen_plus_one112)
  %str_ptr_int113 = ptrtoint ptr %new_str110 to i64
  %v2114 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int113, 1
  br label %add_merge88

add_error87:                                      ; preds = %check_string96
  br label %add_merge88

add_merge88:                                      ; preds = %add_error87, %add_string86, %add_float85, %add_int_int84
  %add_result115 = phi { i8, i64 } [ %v298, %add_int_int84 ], [ %v2104, %add_float85 ], [ %v2114, %add_string86 ], [ zeroinitializer, %add_error87 ]
  %tag116 = extractvalue { i8, i64 } %add_result115, 0
  %data117 = extractvalue { i8, i64 } %add_result115, 1
  switch i8 %tag116, label %print_default123 [
    i8 0, label %print_nil118
    i8 1, label %print_bool119
    i8 2, label %print_int120
    i8 3, label %print_float121
    i8 4, label %print_string122
  ]

check_float95:                                    ; preds = %str_merge28
  br i1 %either_float92, label %add_float85, label %check_string96

check_string96:                                   ; preds = %check_float95
  br i1 %both_str94, label %add_string86, label %add_error87

print_nil118:                                     ; preds = %add_merge88
  %30 = call i32 (ptr, ...) @printf(ptr @fmt_nil)
  br label %print_done124

print_bool119:                                    ; preds = %add_merge88
  %is_true125 = icmp ne i64 %data117, 0
  %bool_str126 = select i1 %is_true125, ptr @fmt_true, ptr @fmt_false
  %31 = call i32 (ptr, ...) @printf(ptr %bool_str126)
  br label %print_done124

print_int120:                                     ; preds = %add_merge88
  %32 = call i32 (ptr, ...) @printf(ptr @fmt_int, i64 %data117)
  br label %print_done124

print_float121:                                   ; preds = %add_merge88
  %f127 = bitcast i64 %data117 to double
  %33 = call i32 (ptr, ...) @printf(ptr @fmt_float, double %f127)
  br label %print_done124

print_string122:                                  ; preds = %add_merge88
  %str128 = inttoptr i64 %data117 to ptr
  %34 = call i32 (ptr, ...) @printf(ptr @fmt_string, ptr %str128)
  br label %print_done124

print_default123:                                 ; preds = %add_merge88
  br label %print_done124

print_done124:                                    ; preds = %print_default123, %print_string122, %print_float121, %print_int120, %print_bool119, %print_nil118
  %35 = call i32 (ptr, ...) @printf(ptr @fmt_newline)
  %call129 = tail call { i8, i64 } @is_even({ i8, i64 } { i8 2, i64 7 })
  %tag130 = extractvalue { i8, i64 } %call129, 0
  %data131 = extractvalue { i8, i64 } %call129, 1
  switch i8 %tag130, label %str_default137 [
    i8 0, label %str_nil132
    i8 1, label %str_bool133
    i8 2, label %str_int134
    i8 3, label %str_float135
    i8 4, label %str_string136
    i8 5, label %str_list139
  ]

str_nil132:                                       ; preds = %print_done124
  br label %str_merge138

str_bool133:                                      ; preds = %print_done124
  %is_true140 = icmp ne i64 %data131, 0
  %bool_ptr141 = select i1 %is_true140, ptr @true_str.19, ptr @false_str.20
  %str_ptr_int142 = ptrtoint ptr %bool_ptr141 to i64
  %v2143 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int142, 1
  br label %str_merge138

str_int134:                                       ; preds = %print_done124
  %int_buf144 = call ptr @malloc(i64 32)
  %36 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %int_buf144, i64 32, ptr @int_fmt.21, i64 %data131)
  %str_ptr_int145 = ptrtoint ptr %int_buf144 to i64
  %v2146 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int145, 1
  br label %str_merge138

str_float135:                                     ; preds = %print_done124
  %float_buf147 = call ptr @malloc(i64 32)
  %f148 = bitcast i64 %data131 to double
  %37 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %float_buf147, i64 32, ptr @float_fmt.22, double %f148)
  %str_ptr_int149 = ptrtoint ptr %float_buf147 to i64
  %v2150 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int149, 1
  br label %str_merge138

str_string136:                                    ; preds = %print_done124
  br label %str_merge138

str_default137:                                   ; preds = %print_done124
  br label %str_merge138

str_merge138:                                     ; preds = %str_default137, %list_loop_end159, %str_string136, %str_float135, %str_int134, %str_bool133, %str_nil132
  %str_result191 = phi { i8, i64 } [ { i8 4, i64 ptrtoint (ptr @nil_str.18 to i64) }, %str_nil132 ], [ %v2143, %str_bool133 ], [ %v2146, %str_int134 ], [ %v2150, %str_float135 ], [ %call129, %str_string136 ], [ %v2190, %list_loop_end159 ], [ { i8 4, i64 ptrtoint (ptr @empty_str.28 to i64) }, %str_default137 ]
  %tag192 = extractvalue { i8, i64 } %str_result191, 0
  %data193 = extractvalue { i8, i64 } %str_result191, 1
  %r_int199 = icmp eq i8 %tag192, 2
  %both_int200 = and i1 false, %r_int199
  %r_float201 = icmp eq i8 %tag192, 3
  %either_float202 = or i1 false, %r_float201
  %r_str203 = icmp eq i8 %tag192, 4
  %both_str204 = and i1 true, %r_str203
  br i1 %both_int200, label %add_int_int194, label %check_float205

str_list139:                                      ; preds = %print_done124
  %list_ptr151 = inttoptr i64 %data131 to ptr
  %len_ptr152 = getelementptr i64, ptr %list_ptr151, i64 1
  %list_len153 = load i64, ptr %len_ptr152, align 8
  %buf_size_mul154 = mul i64 %list_len153, 25
  %list_buf_size155 = add i64 %buf_size_mul154, 3
  %list_buf156 = call ptr @malloc(i64 %list_buf_size155)
  %38 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %list_buf156, i64 %list_buf_size155, ptr @open_bracket.23)
  %idx_ptr160 = alloca i64, align 8
  store i64 0, ptr %idx_ptr160, align 8
  br label %list_loop_header157

list_loop_header157:                              ; preds = %elem_done179, %str_list139
  %idx161 = load i64, ptr %idx_ptr160, align 8
  %loop_cond162 = icmp ult i64 %idx161, %list_len153
  br i1 %loop_cond162, label %list_loop_body158, label %list_loop_end159

list_loop_body158:                                ; preds = %list_loop_header157
  %is_first163 = icmp eq i64 %idx161, 0
  br i1 %is_first163, label %elem_block165, label %sep_block164

list_loop_end159:                                 ; preds = %list_loop_header157
  %39 = call ptr @strcat(ptr %list_buf156, ptr @close_bracket.27)
  %str_ptr_int189 = ptrtoint ptr %list_buf156 to i64
  %v2190 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int189, 1
  br label %str_merge138

sep_block164:                                     ; preds = %list_loop_body158
  %40 = call ptr @strcat(ptr %list_buf156, ptr @comma_sep.24)
  br label %elem_block165

elem_block165:                                    ; preds = %sep_block164, %list_loop_body158
  %idx_in_elem166 = load i64, ptr %idx_ptr160, align 8
  %elements_base167 = getelementptr i64, ptr %len_ptr152, i64 1
  %elem_ptr168 = getelementptr { i8, i64 }, ptr %elements_base167, i64 %idx_in_elem166
  %elem_val169 = load { i8, i64 }, ptr %elem_ptr168, align 8
  %elem_tag170 = extractvalue { i8, i64 } %elem_val169, 0
  %elem_data171 = extractvalue { i8, i64 } %elem_val169, 1
  %elem_data_ptr172 = alloca i64, align 8
  store i64 %elem_data171, ptr %elem_data_ptr172, align 8
  %elem_is_float173 = icmp eq i8 %elem_tag170, 3
  %elem_is_string174 = icmp eq i8 %elem_tag170, 4
  br i1 %elem_is_float173, label %elem_float_block175, label %elem_string_check176

elem_float_block175:                              ; preds = %elem_block165
  %elem_data_float182 = load i64, ptr %elem_data_ptr172, align 8
  %elem_float_buf183 = call ptr @malloc(i64 25)
  %elem_as_float184 = bitcast i64 %elem_data_float182 to double
  %41 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_float_buf183, i64 25, ptr @float_fmt2.25, double %elem_as_float184)
  %42 = call ptr @strcat(ptr %list_buf156, ptr %elem_float_buf183)
  br label %elem_done179

elem_string_check176:                             ; preds = %elem_block165
  br i1 %elem_is_string174, label %elem_string_print177, label %elem_int_block178

elem_string_print177:                             ; preds = %elem_string_check176
  %elem_data_str180 = load i64, ptr %elem_data_ptr172, align 8
  %elem_str_ptr181 = inttoptr i64 %elem_data_str180 to ptr
  %43 = call ptr @strcat(ptr %list_buf156, ptr %elem_str_ptr181)
  br label %elem_done179

elem_int_block178:                                ; preds = %elem_string_check176
  %elem_data_int185 = load i64, ptr %elem_data_ptr172, align 8
  %elem_int_buf186 = call ptr @malloc(i64 25)
  %44 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_int_buf186, i64 25, ptr @int_fmt2.26, i64 %elem_data_int185)
  %45 = call ptr @strcat(ptr %list_buf156, ptr %elem_int_buf186)
  br label %elem_done179

elem_done179:                                     ; preds = %elem_int_block178, %elem_float_block175, %elem_string_print177
  %idx_for_incr187 = load i64, ptr %idx_ptr160, align 8
  %next_idx188 = add i64 %idx_for_incr187, 1
  store i64 %next_idx188, ptr %idx_ptr160, align 8
  br label %list_loop_header157

add_int_int194:                                   ; preds = %str_merge138
  %sum207 = add i64 ptrtoint (ptr @str.17 to i64), %data193
  %v2208 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %sum207, 1
  br label %add_merge198

add_float195:                                     ; preds = %check_float205
  %rf209 = bitcast i64 %data193 to double
  %ri2f210 = sitofp i64 %data193 to double
  %right_as_float211 = select i1 %r_float201, double %rf209, double %ri2f210
  %fsum212 = fadd double sitofp (i64 ptrtoint (ptr @str.17 to i64) to double), %right_as_float211
  %float_bits213 = bitcast double %fsum212 to i64
  %v2214 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits213, 1
  br label %add_merge198

add_string196:                                    ; preds = %check_string206
  %rstr215 = inttoptr i64 %data193 to ptr
  %llen216 = call i64 @strlen(ptr @str.17)
  %rlen217 = call i64 @strlen(ptr %rstr215)
  %total218 = add i64 %llen216, %rlen217
  %alloc_size219 = add i64 %total218, 1
  %new_str220 = call ptr @malloc(i64 %alloc_size219)
  %46 = call ptr @memcpy(ptr %new_str220, ptr @str.17, i64 %llen216)
  %dest_offset221 = getelementptr i8, ptr %new_str220, i64 %llen216
  %rlen_plus_one222 = add i64 %rlen217, 1
  %47 = call ptr @memcpy(ptr %dest_offset221, ptr %rstr215, i64 %rlen_plus_one222)
  %str_ptr_int223 = ptrtoint ptr %new_str220 to i64
  %v2224 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int223, 1
  br label %add_merge198

add_error197:                                     ; preds = %check_string206
  br label %add_merge198

add_merge198:                                     ; preds = %add_error197, %add_string196, %add_float195, %add_int_int194
  %add_result225 = phi { i8, i64 } [ %v2208, %add_int_int194 ], [ %v2214, %add_float195 ], [ %v2224, %add_string196 ], [ zeroinitializer, %add_error197 ]
  %tag226 = extractvalue { i8, i64 } %add_result225, 0
  %data227 = extractvalue { i8, i64 } %add_result225, 1
  switch i8 %tag226, label %print_default233 [
    i8 0, label %print_nil228
    i8 1, label %print_bool229
    i8 2, label %print_int230
    i8 3, label %print_float231
    i8 4, label %print_string232
  ]

check_float205:                                   ; preds = %str_merge138
  br i1 %either_float202, label %add_float195, label %check_string206

check_string206:                                  ; preds = %check_float205
  br i1 %both_str204, label %add_string196, label %add_error197

print_nil228:                                     ; preds = %add_merge198
  %48 = call i32 (ptr, ...) @printf(ptr @fmt_nil)
  br label %print_done234

print_bool229:                                    ; preds = %add_merge198
  %is_true235 = icmp ne i64 %data227, 0
  %bool_str236 = select i1 %is_true235, ptr @fmt_true, ptr @fmt_false
  %49 = call i32 (ptr, ...) @printf(ptr %bool_str236)
  br label %print_done234

print_int230:                                     ; preds = %add_merge198
  %50 = call i32 (ptr, ...) @printf(ptr @fmt_int, i64 %data227)
  br label %print_done234

print_float231:                                   ; preds = %add_merge198
  %f237 = bitcast i64 %data227 to double
  %51 = call i32 (ptr, ...) @printf(ptr @fmt_float, double %f237)
  br label %print_done234

print_string232:                                  ; preds = %add_merge198
  %str238 = inttoptr i64 %data227 to ptr
  %52 = call i32 (ptr, ...) @printf(ptr @fmt_string, ptr %str238)
  br label %print_done234

print_default233:                                 ; preds = %add_merge198
  br label %print_done234

print_done234:                                    ; preds = %print_default233, %print_string232, %print_float231, %print_int230, %print_bool229, %print_nil228
  %53 = call i32 (ptr, ...) @printf(ptr @fmt_newline)
  switch i8 4, label %print_default244 [
    i8 0, label %print_nil239
    i8 1, label %print_bool240
    i8 2, label %print_int241
    i8 3, label %print_float242
    i8 4, label %print_string243
  ]

print_nil239:                                     ; preds = %print_done234
  %54 = call i32 (ptr, ...) @printf(ptr @fmt_nil)
  br label %print_done245

print_bool240:                                    ; preds = %print_done234
  %55 = call i32 (ptr, ...) @printf(ptr select (i1 icmp ne (i64 ptrtoint (ptr @str.29 to i64), i64 0), ptr @fmt_true, ptr @fmt_false))
  br label %print_done245

print_int241:                                     ; preds = %print_done234
  %56 = call i32 (ptr, ...) @printf(ptr @fmt_int, i64 ptrtoint (ptr @str.29 to i64))
  br label %print_done245

print_float242:                                   ; preds = %print_done234
  %57 = call i32 (ptr, ...) @printf(ptr @fmt_float, double bitcast (i64 ptrtoint (ptr @str.29 to i64) to double))
  br label %print_done245

print_string243:                                  ; preds = %print_done234
  %58 = call i32 (ptr, ...) @printf(ptr @fmt_string, ptr @str.29)
  br label %print_done245

print_default244:                                 ; preds = %print_done234
  br label %print_done245

print_done245:                                    ; preds = %print_default244, %print_string243, %print_float242, %print_int241, %print_bool240, %print_nil239
  %59 = call i32 (ptr, ...) @printf(ptr @fmt_newline)
  switch i8 4, label %print_default251 [
    i8 0, label %print_nil246
    i8 1, label %print_bool247
    i8 2, label %print_int248
    i8 3, label %print_float249
    i8 4, label %print_string250
  ]

print_nil246:                                     ; preds = %print_done245
  %60 = call i32 (ptr, ...) @printf(ptr @fmt_nil)
  br label %print_done252

print_bool247:                                    ; preds = %print_done245
  %61 = call i32 (ptr, ...) @printf(ptr select (i1 icmp ne (i64 ptrtoint (ptr @str.30 to i64), i64 0), ptr @fmt_true, ptr @fmt_false))
  br label %print_done252

print_int248:                                     ; preds = %print_done245
  %62 = call i32 (ptr, ...) @printf(ptr @fmt_int, i64 ptrtoint (ptr @str.30 to i64))
  br label %print_done252

print_float249:                                   ; preds = %print_done245
  %63 = call i32 (ptr, ...) @printf(ptr @fmt_float, double bitcast (i64 ptrtoint (ptr @str.30 to i64) to double))
  br label %print_done252

print_string250:                                  ; preds = %print_done245
  %64 = call i32 (ptr, ...) @printf(ptr @fmt_string, ptr @str.30)
  br label %print_done252

print_default251:                                 ; preds = %print_done245
  br label %print_done252

print_done252:                                    ; preds = %print_default251, %print_string250, %print_float249, %print_int248, %print_bool247, %print_nil246
  %65 = call i32 (ptr, ...) @printf(ptr @fmt_newline)
  store { i8, i64 } { i8 2, i64 1 }, ptr %i, align 8
  %counter = alloca i64, align 8
  store i64 1, ptr %counter, align 8
  br label %for_loop

for_loop:                                         ; preds = %for_incr, %print_done252
  %current = load i64, ptr %counter, align 8
  %cmp = icmp slt i64 %current, 8
  br i1 %cmp, label %for_body, label %for_after

for_body:                                         ; preds = %for_loop
  %i253 = load { i8, i64 }, ptr %i, align 8
  %tag254 = extractvalue { i8, i64 } %i253, 0
  %data255 = extractvalue { i8, i64 } %i253, 1
  switch i8 %tag254, label %str_default261 [
    i8 0, label %str_nil256
    i8 1, label %str_bool257
    i8 2, label %str_int258
    i8 3, label %str_float259
    i8 4, label %str_string260
    i8 5, label %str_list263
  ]

for_incr:                                         ; preds = %print_done458
  %next = add i64 %current, 1
  store i64 %next, ptr %counter, align 8
  %v2463 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %next, 1
  store { i8, i64 } %v2463, ptr %i, align 8
  br label %for_loop

for_after:                                        ; preds = %for_loop

str_nil256:                                       ; preds = %for_body
  br label %str_merge262

str_bool257:                                      ; preds = %for_body
  %is_true264 = icmp ne i64 %data255, 0
  %bool_ptr265 = select i1 %is_true264, ptr @true_str.32, ptr @false_str.33
  %str_ptr_int266 = ptrtoint ptr %bool_ptr265 to i64
  %v2267 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int266, 1
  br label %str_merge262

str_int258:                                       ; preds = %for_body
  %int_buf268 = call ptr @malloc(i64 32)
  %66 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %int_buf268, i64 32, ptr @int_fmt.34, i64 %data255)
  %str_ptr_int269 = ptrtoint ptr %int_buf268 to i64
  %v2270 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int269, 1
  br label %str_merge262

str_float259:                                     ; preds = %for_body
  %float_buf271 = call ptr @malloc(i64 32)
  %f272 = bitcast i64 %data255 to double
  %67 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %float_buf271, i64 32, ptr @float_fmt.35, double %f272)
  %str_ptr_int273 = ptrtoint ptr %float_buf271 to i64
  %v2274 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int273, 1
  br label %str_merge262

str_string260:                                    ; preds = %for_body
  br label %str_merge262

str_default261:                                   ; preds = %for_body
  br label %str_merge262

str_merge262:                                     ; preds = %str_default261, %list_loop_end283, %str_string260, %str_float259, %str_int258, %str_bool257, %str_nil256
  %str_result315 = phi { i8, i64 } [ { i8 4, i64 ptrtoint (ptr @nil_str.31 to i64) }, %str_nil256 ], [ %v2267, %str_bool257 ], [ %v2270, %str_int258 ], [ %v2274, %str_float259 ], [ %i253, %str_string260 ], [ %v2314, %list_loop_end283 ], [ { i8 4, i64 ptrtoint (ptr @empty_str.41 to i64) }, %str_default261 ]
  %tag316 = extractvalue { i8, i64 } %str_result315, 0
  %data317 = extractvalue { i8, i64 } %str_result315, 1
  %l_int = icmp eq i8 %tag316, 2
  %both_int323 = and i1 %l_int, false
  %l_float = icmp eq i8 %tag316, 3
  %either_float324 = or i1 %l_float, false
  %l_str = icmp eq i8 %tag316, 4
  %both_str325 = and i1 %l_str, true
  br i1 %both_int323, label %add_int_int318, label %check_float326

str_list263:                                      ; preds = %for_body
  %list_ptr275 = inttoptr i64 %data255 to ptr
  %len_ptr276 = getelementptr i64, ptr %list_ptr275, i64 1
  %list_len277 = load i64, ptr %len_ptr276, align 8
  %buf_size_mul278 = mul i64 %list_len277, 25
  %list_buf_size279 = add i64 %buf_size_mul278, 3
  %list_buf280 = call ptr @malloc(i64 %list_buf_size279)
  %68 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %list_buf280, i64 %list_buf_size279, ptr @open_bracket.36)
  %idx_ptr284 = alloca i64, align 8
  store i64 0, ptr %idx_ptr284, align 8
  br label %list_loop_header281

list_loop_header281:                              ; preds = %elem_done303, %str_list263
  %idx285 = load i64, ptr %idx_ptr284, align 8
  %loop_cond286 = icmp ult i64 %idx285, %list_len277
  br i1 %loop_cond286, label %list_loop_body282, label %list_loop_end283

list_loop_body282:                                ; preds = %list_loop_header281
  %is_first287 = icmp eq i64 %idx285, 0
  br i1 %is_first287, label %elem_block289, label %sep_block288

list_loop_end283:                                 ; preds = %list_loop_header281
  %69 = call ptr @strcat(ptr %list_buf280, ptr @close_bracket.40)
  %str_ptr_int313 = ptrtoint ptr %list_buf280 to i64
  %v2314 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int313, 1
  br label %str_merge262

sep_block288:                                     ; preds = %list_loop_body282
  %70 = call ptr @strcat(ptr %list_buf280, ptr @comma_sep.37)
  br label %elem_block289

elem_block289:                                    ; preds = %sep_block288, %list_loop_body282
  %idx_in_elem290 = load i64, ptr %idx_ptr284, align 8
  %elements_base291 = getelementptr i64, ptr %len_ptr276, i64 1
  %elem_ptr292 = getelementptr { i8, i64 }, ptr %elements_base291, i64 %idx_in_elem290
  %elem_val293 = load { i8, i64 }, ptr %elem_ptr292, align 8
  %elem_tag294 = extractvalue { i8, i64 } %elem_val293, 0
  %elem_data295 = extractvalue { i8, i64 } %elem_val293, 1
  %elem_data_ptr296 = alloca i64, align 8
  store i64 %elem_data295, ptr %elem_data_ptr296, align 8
  %elem_is_float297 = icmp eq i8 %elem_tag294, 3
  %elem_is_string298 = icmp eq i8 %elem_tag294, 4
  br i1 %elem_is_float297, label %elem_float_block299, label %elem_string_check300

elem_float_block299:                              ; preds = %elem_block289
  %elem_data_float306 = load i64, ptr %elem_data_ptr296, align 8
  %elem_float_buf307 = call ptr @malloc(i64 25)
  %elem_as_float308 = bitcast i64 %elem_data_float306 to double
  %71 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_float_buf307, i64 25, ptr @float_fmt2.38, double %elem_as_float308)
  %72 = call ptr @strcat(ptr %list_buf280, ptr %elem_float_buf307)
  br label %elem_done303

elem_string_check300:                             ; preds = %elem_block289
  br i1 %elem_is_string298, label %elem_string_print301, label %elem_int_block302

elem_string_print301:                             ; preds = %elem_string_check300
  %elem_data_str304 = load i64, ptr %elem_data_ptr296, align 8
  %elem_str_ptr305 = inttoptr i64 %elem_data_str304 to ptr
  %73 = call ptr @strcat(ptr %list_buf280, ptr %elem_str_ptr305)
  br label %elem_done303

elem_int_block302:                                ; preds = %elem_string_check300
  %elem_data_int309 = load i64, ptr %elem_data_ptr296, align 8
  %elem_int_buf310 = call ptr @malloc(i64 25)
  %74 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_int_buf310, i64 25, ptr @int_fmt2.39, i64 %elem_data_int309)
  %75 = call ptr @strcat(ptr %list_buf280, ptr %elem_int_buf310)
  br label %elem_done303

elem_done303:                                     ; preds = %elem_int_block302, %elem_float_block299, %elem_string_print301
  %idx_for_incr311 = load i64, ptr %idx_ptr284, align 8
  %next_idx312 = add i64 %idx_for_incr311, 1
  store i64 %next_idx312, ptr %idx_ptr284, align 8
  br label %list_loop_header281

add_int_int318:                                   ; preds = %str_merge262
  %sum328 = add i64 %data317, ptrtoint (ptr @str.42 to i64)
  %v2329 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %sum328, 1
  br label %add_merge322

add_float319:                                     ; preds = %check_float326
  %lf = bitcast i64 %data317 to double
  %li2f = sitofp i64 %data317 to double
  %left_as_float = select i1 %l_float, double %lf, double %li2f
  %fsum330 = fadd double %left_as_float, sitofp (i64 ptrtoint (ptr @str.42 to i64) to double)
  %float_bits331 = bitcast double %fsum330 to i64
  %v2332 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits331, 1
  br label %add_merge322

add_string320:                                    ; preds = %check_string327
  %lstr = inttoptr i64 %data317 to ptr
  %llen333 = call i64 @strlen(ptr %lstr)
  %rlen334 = call i64 @strlen(ptr @str.42)
  %total335 = add i64 %llen333, %rlen334
  %alloc_size336 = add i64 %total335, 1
  %new_str337 = call ptr @malloc(i64 %alloc_size336)
  %76 = call ptr @memcpy(ptr %new_str337, ptr %lstr, i64 %llen333)
  %dest_offset338 = getelementptr i8, ptr %new_str337, i64 %llen333
  %rlen_plus_one339 = add i64 %rlen334, 1
  %77 = call ptr @memcpy(ptr %dest_offset338, ptr @str.42, i64 %rlen_plus_one339)
  %str_ptr_int340 = ptrtoint ptr %new_str337 to i64
  %v2341 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int340, 1
  br label %add_merge322

add_error321:                                     ; preds = %check_string327
  br label %add_merge322

add_merge322:                                     ; preds = %add_error321, %add_string320, %add_float319, %add_int_int318
  %add_result342 = phi { i8, i64 } [ %v2329, %add_int_int318 ], [ %v2332, %add_float319 ], [ %v2341, %add_string320 ], [ zeroinitializer, %add_error321 ]
  %i343 = load { i8, i64 }, ptr %i, align 8
  %call344 = tail call { i8, i64 } @factorial({ i8, i64 } %i343)
  %tag345 = extractvalue { i8, i64 } %call344, 0
  %data346 = extractvalue { i8, i64 } %call344, 1
  switch i8 %tag345, label %str_default352 [
    i8 0, label %str_nil347
    i8 1, label %str_bool348
    i8 2, label %str_int349
    i8 3, label %str_float350
    i8 4, label %str_string351
    i8 5, label %str_list354
  ]

check_float326:                                   ; preds = %str_merge262
  br i1 %either_float324, label %add_float319, label %check_string327

check_string327:                                  ; preds = %check_float326
  br i1 %both_str325, label %add_string320, label %add_error321

str_nil347:                                       ; preds = %add_merge322
  br label %str_merge353

str_bool348:                                      ; preds = %add_merge322
  %is_true355 = icmp ne i64 %data346, 0
  %bool_ptr356 = select i1 %is_true355, ptr @true_str.44, ptr @false_str.45
  %str_ptr_int357 = ptrtoint ptr %bool_ptr356 to i64
  %v2358 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int357, 1
  br label %str_merge353

str_int349:                                       ; preds = %add_merge322
  %int_buf359 = call ptr @malloc(i64 32)
  %78 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %int_buf359, i64 32, ptr @int_fmt.46, i64 %data346)
  %str_ptr_int360 = ptrtoint ptr %int_buf359 to i64
  %v2361 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int360, 1
  br label %str_merge353

str_float350:                                     ; preds = %add_merge322
  %float_buf362 = call ptr @malloc(i64 32)
  %f363 = bitcast i64 %data346 to double
  %79 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %float_buf362, i64 32, ptr @float_fmt.47, double %f363)
  %str_ptr_int364 = ptrtoint ptr %float_buf362 to i64
  %v2365 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int364, 1
  br label %str_merge353

str_string351:                                    ; preds = %add_merge322
  br label %str_merge353

str_default352:                                   ; preds = %add_merge322
  br label %str_merge353

str_merge353:                                     ; preds = %str_default352, %list_loop_end374, %str_string351, %str_float350, %str_int349, %str_bool348, %str_nil347
  %str_result406 = phi { i8, i64 } [ { i8 4, i64 ptrtoint (ptr @nil_str.43 to i64) }, %str_nil347 ], [ %v2358, %str_bool348 ], [ %v2361, %str_int349 ], [ %v2365, %str_float350 ], [ %call344, %str_string351 ], [ %v2405, %list_loop_end374 ], [ { i8 4, i64 ptrtoint (ptr @empty_str.53 to i64) }, %str_default352 ]
  %tag407 = extractvalue { i8, i64 } %add_result342, 0
  %tag408 = extractvalue { i8, i64 } %str_result406, 0
  %data409 = extractvalue { i8, i64 } %add_result342, 1
  %data410 = extractvalue { i8, i64 } %str_result406, 1
  %l_int416 = icmp eq i8 %tag407, 2
  %r_int417 = icmp eq i8 %tag408, 2
  %both_int418 = and i1 %l_int416, %r_int417
  %l_float419 = icmp eq i8 %tag407, 3
  %r_float420 = icmp eq i8 %tag408, 3
  %either_float421 = or i1 %l_float419, %r_float420
  %l_str422 = icmp eq i8 %tag407, 4
  %r_str423 = icmp eq i8 %tag408, 4
  %both_str424 = and i1 %l_str422, %r_str423
  br i1 %both_int418, label %add_int_int411, label %check_float425

str_list354:                                      ; preds = %add_merge322
  %list_ptr366 = inttoptr i64 %data346 to ptr
  %len_ptr367 = getelementptr i64, ptr %list_ptr366, i64 1
  %list_len368 = load i64, ptr %len_ptr367, align 8
  %buf_size_mul369 = mul i64 %list_len368, 25
  %list_buf_size370 = add i64 %buf_size_mul369, 3
  %list_buf371 = call ptr @malloc(i64 %list_buf_size370)
  %80 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %list_buf371, i64 %list_buf_size370, ptr @open_bracket.48)
  %idx_ptr375 = alloca i64, align 8
  store i64 0, ptr %idx_ptr375, align 8
  br label %list_loop_header372

list_loop_header372:                              ; preds = %elem_done394, %str_list354
  %idx376 = load i64, ptr %idx_ptr375, align 8
  %loop_cond377 = icmp ult i64 %idx376, %list_len368
  br i1 %loop_cond377, label %list_loop_body373, label %list_loop_end374

list_loop_body373:                                ; preds = %list_loop_header372
  %is_first378 = icmp eq i64 %idx376, 0
  br i1 %is_first378, label %elem_block380, label %sep_block379

list_loop_end374:                                 ; preds = %list_loop_header372
  %81 = call ptr @strcat(ptr %list_buf371, ptr @close_bracket.52)
  %str_ptr_int404 = ptrtoint ptr %list_buf371 to i64
  %v2405 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int404, 1
  br label %str_merge353

sep_block379:                                     ; preds = %list_loop_body373
  %82 = call ptr @strcat(ptr %list_buf371, ptr @comma_sep.49)
  br label %elem_block380

elem_block380:                                    ; preds = %sep_block379, %list_loop_body373
  %idx_in_elem381 = load i64, ptr %idx_ptr375, align 8
  %elements_base382 = getelementptr i64, ptr %len_ptr367, i64 1
  %elem_ptr383 = getelementptr { i8, i64 }, ptr %elements_base382, i64 %idx_in_elem381
  %elem_val384 = load { i8, i64 }, ptr %elem_ptr383, align 8
  %elem_tag385 = extractvalue { i8, i64 } %elem_val384, 0
  %elem_data386 = extractvalue { i8, i64 } %elem_val384, 1
  %elem_data_ptr387 = alloca i64, align 8
  store i64 %elem_data386, ptr %elem_data_ptr387, align 8
  %elem_is_float388 = icmp eq i8 %elem_tag385, 3
  %elem_is_string389 = icmp eq i8 %elem_tag385, 4
  br i1 %elem_is_float388, label %elem_float_block390, label %elem_string_check391

elem_float_block390:                              ; preds = %elem_block380
  %elem_data_float397 = load i64, ptr %elem_data_ptr387, align 8
  %elem_float_buf398 = call ptr @malloc(i64 25)
  %elem_as_float399 = bitcast i64 %elem_data_float397 to double
  %83 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_float_buf398, i64 25, ptr @float_fmt2.50, double %elem_as_float399)
  %84 = call ptr @strcat(ptr %list_buf371, ptr %elem_float_buf398)
  br label %elem_done394

elem_string_check391:                             ; preds = %elem_block380
  br i1 %elem_is_string389, label %elem_string_print392, label %elem_int_block393

elem_string_print392:                             ; preds = %elem_string_check391
  %elem_data_str395 = load i64, ptr %elem_data_ptr387, align 8
  %elem_str_ptr396 = inttoptr i64 %elem_data_str395 to ptr
  %85 = call ptr @strcat(ptr %list_buf371, ptr %elem_str_ptr396)
  br label %elem_done394

elem_int_block393:                                ; preds = %elem_string_check391
  %elem_data_int400 = load i64, ptr %elem_data_ptr387, align 8
  %elem_int_buf401 = call ptr @malloc(i64 25)
  %86 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_int_buf401, i64 25, ptr @int_fmt2.51, i64 %elem_data_int400)
  %87 = call ptr @strcat(ptr %list_buf371, ptr %elem_int_buf401)
  br label %elem_done394

elem_done394:                                     ; preds = %elem_int_block393, %elem_float_block390, %elem_string_print392
  %idx_for_incr402 = load i64, ptr %idx_ptr375, align 8
  %next_idx403 = add i64 %idx_for_incr402, 1
  store i64 %next_idx403, ptr %idx_ptr375, align 8
  br label %list_loop_header372

add_int_int411:                                   ; preds = %str_merge353
  %sum427 = add i64 %data409, %data410
  %v2428 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %sum427, 1
  br label %add_merge415

add_float412:                                     ; preds = %check_float425
  %lf429 = bitcast i64 %data409 to double
  %li2f430 = sitofp i64 %data409 to double
  %left_as_float431 = select i1 %l_float419, double %lf429, double %li2f430
  %rf432 = bitcast i64 %data410 to double
  %ri2f433 = sitofp i64 %data410 to double
  %right_as_float434 = select i1 %r_float420, double %rf432, double %ri2f433
  %fsum435 = fadd double %left_as_float431, %right_as_float434
  %float_bits436 = bitcast double %fsum435 to i64
  %v2437 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits436, 1
  br label %add_merge415

add_string413:                                    ; preds = %check_string426
  %lstr438 = inttoptr i64 %data409 to ptr
  %rstr439 = inttoptr i64 %data410 to ptr
  %llen440 = call i64 @strlen(ptr %lstr438)
  %rlen441 = call i64 @strlen(ptr %rstr439)
  %total442 = add i64 %llen440, %rlen441
  %alloc_size443 = add i64 %total442, 1
  %new_str444 = call ptr @malloc(i64 %alloc_size443)
  %88 = call ptr @memcpy(ptr %new_str444, ptr %lstr438, i64 %llen440)
  %dest_offset445 = getelementptr i8, ptr %new_str444, i64 %llen440
  %rlen_plus_one446 = add i64 %rlen441, 1
  %89 = call ptr @memcpy(ptr %dest_offset445, ptr %rstr439, i64 %rlen_plus_one446)
  %str_ptr_int447 = ptrtoint ptr %new_str444 to i64
  %v2448 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int447, 1
  br label %add_merge415

add_error414:                                     ; preds = %check_string426
  br label %add_merge415

add_merge415:                                     ; preds = %add_error414, %add_string413, %add_float412, %add_int_int411
  %add_result449 = phi { i8, i64 } [ %v2428, %add_int_int411 ], [ %v2437, %add_float412 ], [ %v2448, %add_string413 ], [ zeroinitializer, %add_error414 ]
  %tag450 = extractvalue { i8, i64 } %add_result449, 0
  %data451 = extractvalue { i8, i64 } %add_result449, 1
  switch i8 %tag450, label %print_default457 [
    i8 0, label %print_nil452
    i8 1, label %print_bool453
    i8 2, label %print_int454
    i8 3, label %print_float455
    i8 4, label %print_string456
  ]

check_float425:                                   ; preds = %str_merge353
  br i1 %either_float421, label %add_float412, label %check_string426

check_string426:                                  ; preds = %check_float425
  br i1 %both_str424, label %add_string413, label %add_error414

print_nil452:                                     ; preds = %add_merge415
  %90 = call i32 (ptr, ...) @printf(ptr @fmt_nil)
  br label %print_done458

print_bool453:                                    ; preds = %add_merge415
  %is_true459 = icmp ne i64 %data451, 0
  %bool_str460 = select i1 %is_true459, ptr @fmt_true, ptr @fmt_false
  %91 = call i32 (ptr, ...) @printf(ptr %bool_str460)
  br label %print_done458

print_int454:                                     ; preds = %add_merge415
  %92 = call i32 (ptr, ...) @printf(ptr @fmt_int, i64 %data451)
  br label %print_done458

print_float455:                                   ; preds = %add_merge415
  %f461 = bitcast i64 %data451 to double
  %93 = call i32 (ptr, ...) @printf(ptr @fmt_float, double %f461)
  br label %print_done458

print_string456:                                  ; preds = %add_merge415
  %str462 = inttoptr i64 %data451 to ptr
  %94 = call i32 (ptr, ...) @printf(ptr @fmt_string, ptr %str462)
  br label %print_done458

print_default457:                                 ; preds = %add_merge415
  br label %print_done458

print_done458:                                    ; preds = %print_default457, %print_string456, %print_float455, %print_int454, %print_bool453, %print_nil452
  %95 = call i32 (ptr, ...) @printf(ptr @fmt_newline)
  br label %for_incr
  switch i8 4, label %print_default469 [
    i8 0, label %print_nil464
    i8 1, label %print_bool465
    i8 2, label %print_int466
    i8 3, label %print_float467
    i8 4, label %print_string468
  ]

print_nil464:                                     ; preds = %print_done458
  %96 = call i32 (ptr, ...) @printf(ptr @fmt_nil)
  br label %print_done470

print_bool465:                                    ; preds = %print_done458
  %97 = call i32 (ptr, ...) @printf(ptr select (i1 icmp ne (i64 ptrtoint (ptr @str.54 to i64), i64 0), ptr @fmt_true, ptr @fmt_false))
  br label %print_done470

print_int466:                                     ; preds = %print_done458
  %98 = call i32 (ptr, ...) @printf(ptr @fmt_int, i64 ptrtoint (ptr @str.54 to i64))
  br label %print_done470

print_float467:                                   ; preds = %print_done458
  %99 = call i32 (ptr, ...) @printf(ptr @fmt_float, double bitcast (i64 ptrtoint (ptr @str.54 to i64) to double))
  br label %print_done470

print_string468:                                  ; preds = %print_done458
  %100 = call i32 (ptr, ...) @printf(ptr @fmt_string, ptr @str.54)
  br label %print_done470

print_default469:                                 ; preds = %print_done458
  br label %print_done470

print_done470:                                    ; preds = %print_default469, %print_string468, %print_float467, %print_int466, %print_bool465, %print_nil464
  %101 = call i32 (ptr, ...) @printf(ptr @fmt_newline)
  switch i8 4, label %print_default476 [
    i8 0, label %print_nil471
    i8 1, label %print_bool472
    i8 2, label %print_int473
    i8 3, label %print_float474
    i8 4, label %print_string475
  ]

print_nil471:                                     ; preds = %print_done470
  %102 = call i32 (ptr, ...) @printf(ptr @fmt_nil)
  br label %print_done477

print_bool472:                                    ; preds = %print_done470
  %103 = call i32 (ptr, ...) @printf(ptr select (i1 icmp ne (i64 ptrtoint (ptr @str.55 to i64), i64 0), ptr @fmt_true, ptr @fmt_false))
  br label %print_done477

print_int473:                                     ; preds = %print_done470
  %104 = call i32 (ptr, ...) @printf(ptr @fmt_int, i64 ptrtoint (ptr @str.55 to i64))
  br label %print_done477

print_float474:                                   ; preds = %print_done470
  %105 = call i32 (ptr, ...) @printf(ptr @fmt_float, double bitcast (i64 ptrtoint (ptr @str.55 to i64) to double))
  br label %print_done477

print_string475:                                  ; preds = %print_done470
  %106 = call i32 (ptr, ...) @printf(ptr @fmt_string, ptr @str.55)
  br label %print_done477

print_default476:                                 ; preds = %print_done470
  br label %print_done477

print_done477:                                    ; preds = %print_default476, %print_string475, %print_float474, %print_int473, %print_bool472, %print_nil471
  %107 = call i32 (ptr, ...) @printf(ptr @fmt_newline)
  store { i8, i64 } { i8 2, i64 0 }, ptr %i478, align 8
  %counter479 = alloca i64, align 8
  store i64 0, ptr %counter479, align 8
  br label %for_loop480

for_loop480:                                      ; preds = %for_incr482, %print_done477
  %current484 = load i64, ptr %counter479, align 8
  %cmp485 = icmp slt i64 %current484, 10
  br i1 %cmp485, label %for_body481, label %for_after483

for_body481:                                      ; preds = %for_loop480
  %i486 = load { i8, i64 }, ptr %i478, align 8
  %tag487 = extractvalue { i8, i64 } %i486, 0
  %data488 = extractvalue { i8, i64 } %i486, 1
  switch i8 %tag487, label %str_default494 [
    i8 0, label %str_nil489
    i8 1, label %str_bool490
    i8 2, label %str_int491
    i8 3, label %str_float492
    i8 4, label %str_string493
    i8 5, label %str_list496
  ]

for_incr482:                                      ; preds = %print_done732
  %next737 = add i64 %current484, 1
  store i64 %next737, ptr %counter479, align 8
  %v2738 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %next737, 1
  store { i8, i64 } %v2738, ptr %i478, align 8
  br label %for_loop480

for_after483:                                     ; preds = %for_loop480

str_nil489:                                       ; preds = %for_body481
  br label %str_merge495

str_bool490:                                      ; preds = %for_body481
  %is_true497 = icmp ne i64 %data488, 0
  %bool_ptr498 = select i1 %is_true497, ptr @true_str.58, ptr @false_str.59
  %str_ptr_int499 = ptrtoint ptr %bool_ptr498 to i64
  %v2500 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int499, 1
  br label %str_merge495

str_int491:                                       ; preds = %for_body481
  %int_buf501 = call ptr @malloc(i64 32)
  %108 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %int_buf501, i64 32, ptr @int_fmt.60, i64 %data488)
  %str_ptr_int502 = ptrtoint ptr %int_buf501 to i64
  %v2503 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int502, 1
  br label %str_merge495

str_float492:                                     ; preds = %for_body481
  %float_buf504 = call ptr @malloc(i64 32)
  %f505 = bitcast i64 %data488 to double
  %109 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %float_buf504, i64 32, ptr @float_fmt.61, double %f505)
  %str_ptr_int506 = ptrtoint ptr %float_buf504 to i64
  %v2507 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int506, 1
  br label %str_merge495

str_string493:                                    ; preds = %for_body481
  br label %str_merge495

str_default494:                                   ; preds = %for_body481
  br label %str_merge495

str_merge495:                                     ; preds = %str_default494, %list_loop_end516, %str_string493, %str_float492, %str_int491, %str_bool490, %str_nil489
  %str_result548 = phi { i8, i64 } [ { i8 4, i64 ptrtoint (ptr @nil_str.57 to i64) }, %str_nil489 ], [ %v2500, %str_bool490 ], [ %v2503, %str_int491 ], [ %v2507, %str_float492 ], [ %i486, %str_string493 ], [ %v2547, %list_loop_end516 ], [ { i8 4, i64 ptrtoint (ptr @empty_str.67 to i64) }, %str_default494 ]
  %tag549 = extractvalue { i8, i64 } %str_result548, 0
  %data550 = extractvalue { i8, i64 } %str_result548, 1
  %r_int556 = icmp eq i8 %tag549, 2
  %both_int557 = and i1 false, %r_int556
  %r_float558 = icmp eq i8 %tag549, 3
  %either_float559 = or i1 false, %r_float558
  %r_str560 = icmp eq i8 %tag549, 4
  %both_str561 = and i1 true, %r_str560
  br i1 %both_int557, label %add_int_int551, label %check_float562

str_list496:                                      ; preds = %for_body481
  %list_ptr508 = inttoptr i64 %data488 to ptr
  %len_ptr509 = getelementptr i64, ptr %list_ptr508, i64 1
  %list_len510 = load i64, ptr %len_ptr509, align 8
  %buf_size_mul511 = mul i64 %list_len510, 25
  %list_buf_size512 = add i64 %buf_size_mul511, 3
  %list_buf513 = call ptr @malloc(i64 %list_buf_size512)
  %110 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %list_buf513, i64 %list_buf_size512, ptr @open_bracket.62)
  %idx_ptr517 = alloca i64, align 8
  store i64 0, ptr %idx_ptr517, align 8
  br label %list_loop_header514

list_loop_header514:                              ; preds = %elem_done536, %str_list496
  %idx518 = load i64, ptr %idx_ptr517, align 8
  %loop_cond519 = icmp ult i64 %idx518, %list_len510
  br i1 %loop_cond519, label %list_loop_body515, label %list_loop_end516

list_loop_body515:                                ; preds = %list_loop_header514
  %is_first520 = icmp eq i64 %idx518, 0
  br i1 %is_first520, label %elem_block522, label %sep_block521

list_loop_end516:                                 ; preds = %list_loop_header514
  %111 = call ptr @strcat(ptr %list_buf513, ptr @close_bracket.66)
  %str_ptr_int546 = ptrtoint ptr %list_buf513 to i64
  %v2547 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int546, 1
  br label %str_merge495

sep_block521:                                     ; preds = %list_loop_body515
  %112 = call ptr @strcat(ptr %list_buf513, ptr @comma_sep.63)
  br label %elem_block522

elem_block522:                                    ; preds = %sep_block521, %list_loop_body515
  %idx_in_elem523 = load i64, ptr %idx_ptr517, align 8
  %elements_base524 = getelementptr i64, ptr %len_ptr509, i64 1
  %elem_ptr525 = getelementptr { i8, i64 }, ptr %elements_base524, i64 %idx_in_elem523
  %elem_val526 = load { i8, i64 }, ptr %elem_ptr525, align 8
  %elem_tag527 = extractvalue { i8, i64 } %elem_val526, 0
  %elem_data528 = extractvalue { i8, i64 } %elem_val526, 1
  %elem_data_ptr529 = alloca i64, align 8
  store i64 %elem_data528, ptr %elem_data_ptr529, align 8
  %elem_is_float530 = icmp eq i8 %elem_tag527, 3
  %elem_is_string531 = icmp eq i8 %elem_tag527, 4
  br i1 %elem_is_float530, label %elem_float_block532, label %elem_string_check533

elem_float_block532:                              ; preds = %elem_block522
  %elem_data_float539 = load i64, ptr %elem_data_ptr529, align 8
  %elem_float_buf540 = call ptr @malloc(i64 25)
  %elem_as_float541 = bitcast i64 %elem_data_float539 to double
  %113 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_float_buf540, i64 25, ptr @float_fmt2.64, double %elem_as_float541)
  %114 = call ptr @strcat(ptr %list_buf513, ptr %elem_float_buf540)
  br label %elem_done536

elem_string_check533:                             ; preds = %elem_block522
  br i1 %elem_is_string531, label %elem_string_print534, label %elem_int_block535

elem_string_print534:                             ; preds = %elem_string_check533
  %elem_data_str537 = load i64, ptr %elem_data_ptr529, align 8
  %elem_str_ptr538 = inttoptr i64 %elem_data_str537 to ptr
  %115 = call ptr @strcat(ptr %list_buf513, ptr %elem_str_ptr538)
  br label %elem_done536

elem_int_block535:                                ; preds = %elem_string_check533
  %elem_data_int542 = load i64, ptr %elem_data_ptr529, align 8
  %elem_int_buf543 = call ptr @malloc(i64 25)
  %116 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_int_buf543, i64 25, ptr @int_fmt2.65, i64 %elem_data_int542)
  %117 = call ptr @strcat(ptr %list_buf513, ptr %elem_int_buf543)
  br label %elem_done536

elem_done536:                                     ; preds = %elem_int_block535, %elem_float_block532, %elem_string_print534
  %idx_for_incr544 = load i64, ptr %idx_ptr517, align 8
  %next_idx545 = add i64 %idx_for_incr544, 1
  store i64 %next_idx545, ptr %idx_ptr517, align 8
  br label %list_loop_header514

add_int_int551:                                   ; preds = %str_merge495
  %sum564 = add i64 ptrtoint (ptr @str.56 to i64), %data550
  %v2565 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %sum564, 1
  br label %add_merge555

add_float552:                                     ; preds = %check_float562
  %rf566 = bitcast i64 %data550 to double
  %ri2f567 = sitofp i64 %data550 to double
  %right_as_float568 = select i1 %r_float558, double %rf566, double %ri2f567
  %fsum569 = fadd double sitofp (i64 ptrtoint (ptr @str.56 to i64) to double), %right_as_float568
  %float_bits570 = bitcast double %fsum569 to i64
  %v2571 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits570, 1
  br label %add_merge555

add_string553:                                    ; preds = %check_string563
  %rstr572 = inttoptr i64 %data550 to ptr
  %llen573 = call i64 @strlen(ptr @str.56)
  %rlen574 = call i64 @strlen(ptr %rstr572)
  %total575 = add i64 %llen573, %rlen574
  %alloc_size576 = add i64 %total575, 1
  %new_str577 = call ptr @malloc(i64 %alloc_size576)
  %118 = call ptr @memcpy(ptr %new_str577, ptr @str.56, i64 %llen573)
  %dest_offset578 = getelementptr i8, ptr %new_str577, i64 %llen573
  %rlen_plus_one579 = add i64 %rlen574, 1
  %119 = call ptr @memcpy(ptr %dest_offset578, ptr %rstr572, i64 %rlen_plus_one579)
  %str_ptr_int580 = ptrtoint ptr %new_str577 to i64
  %v2581 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int580, 1
  br label %add_merge555

add_error554:                                     ; preds = %check_string563
  br label %add_merge555

add_merge555:                                     ; preds = %add_error554, %add_string553, %add_float552, %add_int_int551
  %add_result582 = phi { i8, i64 } [ %v2565, %add_int_int551 ], [ %v2571, %add_float552 ], [ %v2581, %add_string553 ], [ zeroinitializer, %add_error554 ]
  %tag583 = extractvalue { i8, i64 } %add_result582, 0
  %data584 = extractvalue { i8, i64 } %add_result582, 1
  %l_int590 = icmp eq i8 %tag583, 2
  %both_int591 = and i1 %l_int590, false
  %l_float592 = icmp eq i8 %tag583, 3
  %either_float593 = or i1 %l_float592, false
  %l_str594 = icmp eq i8 %tag583, 4
  %both_str595 = and i1 %l_str594, true
  br i1 %both_int591, label %add_int_int585, label %check_float596

check_float562:                                   ; preds = %str_merge495
  br i1 %either_float559, label %add_float552, label %check_string563

check_string563:                                  ; preds = %check_float562
  br i1 %both_str561, label %add_string553, label %add_error554

add_int_int585:                                   ; preds = %add_merge555
  %sum598 = add i64 %data584, ptrtoint (ptr @str.68 to i64)
  %v2599 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %sum598, 1
  br label %add_merge589

add_float586:                                     ; preds = %check_float596
  %lf600 = bitcast i64 %data584 to double
  %li2f601 = sitofp i64 %data584 to double
  %left_as_float602 = select i1 %l_float592, double %lf600, double %li2f601
  %fsum603 = fadd double %left_as_float602, sitofp (i64 ptrtoint (ptr @str.68 to i64) to double)
  %float_bits604 = bitcast double %fsum603 to i64
  %v2605 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits604, 1
  br label %add_merge589

add_string587:                                    ; preds = %check_string597
  %lstr606 = inttoptr i64 %data584 to ptr
  %llen607 = call i64 @strlen(ptr %lstr606)
  %rlen608 = call i64 @strlen(ptr @str.68)
  %total609 = add i64 %llen607, %rlen608
  %alloc_size610 = add i64 %total609, 1
  %new_str611 = call ptr @malloc(i64 %alloc_size610)
  %120 = call ptr @memcpy(ptr %new_str611, ptr %lstr606, i64 %llen607)
  %dest_offset612 = getelementptr i8, ptr %new_str611, i64 %llen607
  %rlen_plus_one613 = add i64 %rlen608, 1
  %121 = call ptr @memcpy(ptr %dest_offset612, ptr @str.68, i64 %rlen_plus_one613)
  %str_ptr_int614 = ptrtoint ptr %new_str611 to i64
  %v2615 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int614, 1
  br label %add_merge589

add_error588:                                     ; preds = %check_string597
  br label %add_merge589

add_merge589:                                     ; preds = %add_error588, %add_string587, %add_float586, %add_int_int585
  %add_result616 = phi { i8, i64 } [ %v2599, %add_int_int585 ], [ %v2605, %add_float586 ], [ %v2615, %add_string587 ], [ zeroinitializer, %add_error588 ]
  %i617 = load { i8, i64 }, ptr %i478, align 8
  %call618 = tail call { i8, i64 } @fibonacci({ i8, i64 } %i617)
  %tag619 = extractvalue { i8, i64 } %call618, 0
  %data620 = extractvalue { i8, i64 } %call618, 1
  switch i8 %tag619, label %str_default626 [
    i8 0, label %str_nil621
    i8 1, label %str_bool622
    i8 2, label %str_int623
    i8 3, label %str_float624
    i8 4, label %str_string625
    i8 5, label %str_list628
  ]

check_float596:                                   ; preds = %add_merge555
  br i1 %either_float593, label %add_float586, label %check_string597

check_string597:                                  ; preds = %check_float596
  br i1 %both_str595, label %add_string587, label %add_error588

str_nil621:                                       ; preds = %add_merge589
  br label %str_merge627

str_bool622:                                      ; preds = %add_merge589
  %is_true629 = icmp ne i64 %data620, 0
  %bool_ptr630 = select i1 %is_true629, ptr @true_str.70, ptr @false_str.71
  %str_ptr_int631 = ptrtoint ptr %bool_ptr630 to i64
  %v2632 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int631, 1
  br label %str_merge627

str_int623:                                       ; preds = %add_merge589
  %int_buf633 = call ptr @malloc(i64 32)
  %122 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %int_buf633, i64 32, ptr @int_fmt.72, i64 %data620)
  %str_ptr_int634 = ptrtoint ptr %int_buf633 to i64
  %v2635 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int634, 1
  br label %str_merge627

str_float624:                                     ; preds = %add_merge589
  %float_buf636 = call ptr @malloc(i64 32)
  %f637 = bitcast i64 %data620 to double
  %123 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %float_buf636, i64 32, ptr @float_fmt.73, double %f637)
  %str_ptr_int638 = ptrtoint ptr %float_buf636 to i64
  %v2639 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int638, 1
  br label %str_merge627

str_string625:                                    ; preds = %add_merge589
  br label %str_merge627

str_default626:                                   ; preds = %add_merge589
  br label %str_merge627

str_merge627:                                     ; preds = %str_default626, %list_loop_end648, %str_string625, %str_float624, %str_int623, %str_bool622, %str_nil621
  %str_result680 = phi { i8, i64 } [ { i8 4, i64 ptrtoint (ptr @nil_str.69 to i64) }, %str_nil621 ], [ %v2632, %str_bool622 ], [ %v2635, %str_int623 ], [ %v2639, %str_float624 ], [ %call618, %str_string625 ], [ %v2679, %list_loop_end648 ], [ { i8 4, i64 ptrtoint (ptr @empty_str.79 to i64) }, %str_default626 ]
  %tag681 = extractvalue { i8, i64 } %add_result616, 0
  %tag682 = extractvalue { i8, i64 } %str_result680, 0
  %data683 = extractvalue { i8, i64 } %add_result616, 1
  %data684 = extractvalue { i8, i64 } %str_result680, 1
  %l_int690 = icmp eq i8 %tag681, 2
  %r_int691 = icmp eq i8 %tag682, 2
  %both_int692 = and i1 %l_int690, %r_int691
  %l_float693 = icmp eq i8 %tag681, 3
  %r_float694 = icmp eq i8 %tag682, 3
  %either_float695 = or i1 %l_float693, %r_float694
  %l_str696 = icmp eq i8 %tag681, 4
  %r_str697 = icmp eq i8 %tag682, 4
  %both_str698 = and i1 %l_str696, %r_str697
  br i1 %both_int692, label %add_int_int685, label %check_float699

str_list628:                                      ; preds = %add_merge589
  %list_ptr640 = inttoptr i64 %data620 to ptr
  %len_ptr641 = getelementptr i64, ptr %list_ptr640, i64 1
  %list_len642 = load i64, ptr %len_ptr641, align 8
  %buf_size_mul643 = mul i64 %list_len642, 25
  %list_buf_size644 = add i64 %buf_size_mul643, 3
  %list_buf645 = call ptr @malloc(i64 %list_buf_size644)
  %124 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %list_buf645, i64 %list_buf_size644, ptr @open_bracket.74)
  %idx_ptr649 = alloca i64, align 8
  store i64 0, ptr %idx_ptr649, align 8
  br label %list_loop_header646

list_loop_header646:                              ; preds = %elem_done668, %str_list628
  %idx650 = load i64, ptr %idx_ptr649, align 8
  %loop_cond651 = icmp ult i64 %idx650, %list_len642
  br i1 %loop_cond651, label %list_loop_body647, label %list_loop_end648

list_loop_body647:                                ; preds = %list_loop_header646
  %is_first652 = icmp eq i64 %idx650, 0
  br i1 %is_first652, label %elem_block654, label %sep_block653

list_loop_end648:                                 ; preds = %list_loop_header646
  %125 = call ptr @strcat(ptr %list_buf645, ptr @close_bracket.78)
  %str_ptr_int678 = ptrtoint ptr %list_buf645 to i64
  %v2679 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int678, 1
  br label %str_merge627

sep_block653:                                     ; preds = %list_loop_body647
  %126 = call ptr @strcat(ptr %list_buf645, ptr @comma_sep.75)
  br label %elem_block654

elem_block654:                                    ; preds = %sep_block653, %list_loop_body647
  %idx_in_elem655 = load i64, ptr %idx_ptr649, align 8
  %elements_base656 = getelementptr i64, ptr %len_ptr641, i64 1
  %elem_ptr657 = getelementptr { i8, i64 }, ptr %elements_base656, i64 %idx_in_elem655
  %elem_val658 = load { i8, i64 }, ptr %elem_ptr657, align 8
  %elem_tag659 = extractvalue { i8, i64 } %elem_val658, 0
  %elem_data660 = extractvalue { i8, i64 } %elem_val658, 1
  %elem_data_ptr661 = alloca i64, align 8
  store i64 %elem_data660, ptr %elem_data_ptr661, align 8
  %elem_is_float662 = icmp eq i8 %elem_tag659, 3
  %elem_is_string663 = icmp eq i8 %elem_tag659, 4
  br i1 %elem_is_float662, label %elem_float_block664, label %elem_string_check665

elem_float_block664:                              ; preds = %elem_block654
  %elem_data_float671 = load i64, ptr %elem_data_ptr661, align 8
  %elem_float_buf672 = call ptr @malloc(i64 25)
  %elem_as_float673 = bitcast i64 %elem_data_float671 to double
  %127 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_float_buf672, i64 25, ptr @float_fmt2.76, double %elem_as_float673)
  %128 = call ptr @strcat(ptr %list_buf645, ptr %elem_float_buf672)
  br label %elem_done668

elem_string_check665:                             ; preds = %elem_block654
  br i1 %elem_is_string663, label %elem_string_print666, label %elem_int_block667

elem_string_print666:                             ; preds = %elem_string_check665
  %elem_data_str669 = load i64, ptr %elem_data_ptr661, align 8
  %elem_str_ptr670 = inttoptr i64 %elem_data_str669 to ptr
  %129 = call ptr @strcat(ptr %list_buf645, ptr %elem_str_ptr670)
  br label %elem_done668

elem_int_block667:                                ; preds = %elem_string_check665
  %elem_data_int674 = load i64, ptr %elem_data_ptr661, align 8
  %elem_int_buf675 = call ptr @malloc(i64 25)
  %130 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_int_buf675, i64 25, ptr @int_fmt2.77, i64 %elem_data_int674)
  %131 = call ptr @strcat(ptr %list_buf645, ptr %elem_int_buf675)
  br label %elem_done668

elem_done668:                                     ; preds = %elem_int_block667, %elem_float_block664, %elem_string_print666
  %idx_for_incr676 = load i64, ptr %idx_ptr649, align 8
  %next_idx677 = add i64 %idx_for_incr676, 1
  store i64 %next_idx677, ptr %idx_ptr649, align 8
  br label %list_loop_header646

add_int_int685:                                   ; preds = %str_merge627
  %sum701 = add i64 %data683, %data684
  %v2702 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %sum701, 1
  br label %add_merge689

add_float686:                                     ; preds = %check_float699
  %lf703 = bitcast i64 %data683 to double
  %li2f704 = sitofp i64 %data683 to double
  %left_as_float705 = select i1 %l_float693, double %lf703, double %li2f704
  %rf706 = bitcast i64 %data684 to double
  %ri2f707 = sitofp i64 %data684 to double
  %right_as_float708 = select i1 %r_float694, double %rf706, double %ri2f707
  %fsum709 = fadd double %left_as_float705, %right_as_float708
  %float_bits710 = bitcast double %fsum709 to i64
  %v2711 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits710, 1
  br label %add_merge689

add_string687:                                    ; preds = %check_string700
  %lstr712 = inttoptr i64 %data683 to ptr
  %rstr713 = inttoptr i64 %data684 to ptr
  %llen714 = call i64 @strlen(ptr %lstr712)
  %rlen715 = call i64 @strlen(ptr %rstr713)
  %total716 = add i64 %llen714, %rlen715
  %alloc_size717 = add i64 %total716, 1
  %new_str718 = call ptr @malloc(i64 %alloc_size717)
  %132 = call ptr @memcpy(ptr %new_str718, ptr %lstr712, i64 %llen714)
  %dest_offset719 = getelementptr i8, ptr %new_str718, i64 %llen714
  %rlen_plus_one720 = add i64 %rlen715, 1
  %133 = call ptr @memcpy(ptr %dest_offset719, ptr %rstr713, i64 %rlen_plus_one720)
  %str_ptr_int721 = ptrtoint ptr %new_str718 to i64
  %v2722 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int721, 1
  br label %add_merge689

add_error688:                                     ; preds = %check_string700
  br label %add_merge689

add_merge689:                                     ; preds = %add_error688, %add_string687, %add_float686, %add_int_int685
  %add_result723 = phi { i8, i64 } [ %v2702, %add_int_int685 ], [ %v2711, %add_float686 ], [ %v2722, %add_string687 ], [ zeroinitializer, %add_error688 ]
  %tag724 = extractvalue { i8, i64 } %add_result723, 0
  %data725 = extractvalue { i8, i64 } %add_result723, 1
  switch i8 %tag724, label %print_default731 [
    i8 0, label %print_nil726
    i8 1, label %print_bool727
    i8 2, label %print_int728
    i8 3, label %print_float729
    i8 4, label %print_string730
  ]

check_float699:                                   ; preds = %str_merge627
  br i1 %either_float695, label %add_float686, label %check_string700

check_string700:                                  ; preds = %check_float699
  br i1 %both_str698, label %add_string687, label %add_error688

print_nil726:                                     ; preds = %add_merge689
  %134 = call i32 (ptr, ...) @printf(ptr @fmt_nil)
  br label %print_done732

print_bool727:                                    ; preds = %add_merge689
  %is_true733 = icmp ne i64 %data725, 0
  %bool_str734 = select i1 %is_true733, ptr @fmt_true, ptr @fmt_false
  %135 = call i32 (ptr, ...) @printf(ptr %bool_str734)
  br label %print_done732

print_int728:                                     ; preds = %add_merge689
  %136 = call i32 (ptr, ...) @printf(ptr @fmt_int, i64 %data725)
  br label %print_done732

print_float729:                                   ; preds = %add_merge689
  %f735 = bitcast i64 %data725 to double
  %137 = call i32 (ptr, ...) @printf(ptr @fmt_float, double %f735)
  br label %print_done732

print_string730:                                  ; preds = %add_merge689
  %str736 = inttoptr i64 %data725 to ptr
  %138 = call i32 (ptr, ...) @printf(ptr @fmt_string, ptr %str736)
  br label %print_done732

print_default731:                                 ; preds = %add_merge689
  br label %print_done732

print_done732:                                    ; preds = %print_default731, %print_string730, %print_float729, %print_int728, %print_bool727, %print_nil726
  %139 = call i32 (ptr, ...) @printf(ptr @fmt_newline)
  br label %for_incr482
  switch i8 4, label %print_default744 [
    i8 0, label %print_nil739
    i8 1, label %print_bool740
    i8 2, label %print_int741
    i8 3, label %print_float742
    i8 4, label %print_string743
  ]

print_nil739:                                     ; preds = %print_done732
  %140 = call i32 (ptr, ...) @printf(ptr @fmt_nil)
  br label %print_done745

print_bool740:                                    ; preds = %print_done732
  %141 = call i32 (ptr, ...) @printf(ptr select (i1 icmp ne (i64 ptrtoint (ptr @str.80 to i64), i64 0), ptr @fmt_true, ptr @fmt_false))
  br label %print_done745

print_int741:                                     ; preds = %print_done732
  %142 = call i32 (ptr, ...) @printf(ptr @fmt_int, i64 ptrtoint (ptr @str.80 to i64))
  br label %print_done745

print_float742:                                   ; preds = %print_done732
  %143 = call i32 (ptr, ...) @printf(ptr @fmt_float, double bitcast (i64 ptrtoint (ptr @str.80 to i64) to double))
  br label %print_done745

print_string743:                                  ; preds = %print_done732
  %144 = call i32 (ptr, ...) @printf(ptr @fmt_string, ptr @str.80)
  br label %print_done745

print_default744:                                 ; preds = %print_done732
  br label %print_done745

print_done745:                                    ; preds = %print_default744, %print_string743, %print_float742, %print_int741, %print_bool740, %print_nil739
  %145 = call i32 (ptr, ...) @printf(ptr @fmt_newline)
  %call746 = tail call { i8, i64 } @apply_twice({ i8, i64 } { i8 2, i64 5 }, { i8, i64 } { i8 7, i64 ptrtoint (ptr @double to i64) })
  %tag747 = extractvalue { i8, i64 } %call746, 0
  %data748 = extractvalue { i8, i64 } %call746, 1
  switch i8 %tag747, label %str_default754 [
    i8 0, label %str_nil749
    i8 1, label %str_bool750
    i8 2, label %str_int751
    i8 3, label %str_float752
    i8 4, label %str_string753
    i8 5, label %str_list756
  ]

str_nil749:                                       ; preds = %print_done745
  br label %str_merge755

str_bool750:                                      ; preds = %print_done745
  %is_true757 = icmp ne i64 %data748, 0
  %bool_ptr758 = select i1 %is_true757, ptr @true_str.83, ptr @false_str.84
  %str_ptr_int759 = ptrtoint ptr %bool_ptr758 to i64
  %v2760 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int759, 1
  br label %str_merge755

str_int751:                                       ; preds = %print_done745
  %int_buf761 = call ptr @malloc(i64 32)
  %146 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %int_buf761, i64 32, ptr @int_fmt.85, i64 %data748)
  %str_ptr_int762 = ptrtoint ptr %int_buf761 to i64
  %v2763 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int762, 1
  br label %str_merge755

str_float752:                                     ; preds = %print_done745
  %float_buf764 = call ptr @malloc(i64 32)
  %f765 = bitcast i64 %data748 to double
  %147 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %float_buf764, i64 32, ptr @float_fmt.86, double %f765)
  %str_ptr_int766 = ptrtoint ptr %float_buf764 to i64
  %v2767 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int766, 1
  br label %str_merge755

str_string753:                                    ; preds = %print_done745
  br label %str_merge755

str_default754:                                   ; preds = %print_done745
  br label %str_merge755

str_merge755:                                     ; preds = %str_default754, %list_loop_end776, %str_string753, %str_float752, %str_int751, %str_bool750, %str_nil749
  %str_result808 = phi { i8, i64 } [ { i8 4, i64 ptrtoint (ptr @nil_str.82 to i64) }, %str_nil749 ], [ %v2760, %str_bool750 ], [ %v2763, %str_int751 ], [ %v2767, %str_float752 ], [ %call746, %str_string753 ], [ %v2807, %list_loop_end776 ], [ { i8 4, i64 ptrtoint (ptr @empty_str.92 to i64) }, %str_default754 ]
  %tag809 = extractvalue { i8, i64 } %str_result808, 0
  %data810 = extractvalue { i8, i64 } %str_result808, 1
  %r_int816 = icmp eq i8 %tag809, 2
  %both_int817 = and i1 false, %r_int816
  %r_float818 = icmp eq i8 %tag809, 3
  %either_float819 = or i1 false, %r_float818
  %r_str820 = icmp eq i8 %tag809, 4
  %both_str821 = and i1 true, %r_str820
  br i1 %both_int817, label %add_int_int811, label %check_float822

str_list756:                                      ; preds = %print_done745
  %list_ptr768 = inttoptr i64 %data748 to ptr
  %len_ptr769 = getelementptr i64, ptr %list_ptr768, i64 1
  %list_len770 = load i64, ptr %len_ptr769, align 8
  %buf_size_mul771 = mul i64 %list_len770, 25
  %list_buf_size772 = add i64 %buf_size_mul771, 3
  %list_buf773 = call ptr @malloc(i64 %list_buf_size772)
  %148 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %list_buf773, i64 %list_buf_size772, ptr @open_bracket.87)
  %idx_ptr777 = alloca i64, align 8
  store i64 0, ptr %idx_ptr777, align 8
  br label %list_loop_header774

list_loop_header774:                              ; preds = %elem_done796, %str_list756
  %idx778 = load i64, ptr %idx_ptr777, align 8
  %loop_cond779 = icmp ult i64 %idx778, %list_len770
  br i1 %loop_cond779, label %list_loop_body775, label %list_loop_end776

list_loop_body775:                                ; preds = %list_loop_header774
  %is_first780 = icmp eq i64 %idx778, 0
  br i1 %is_first780, label %elem_block782, label %sep_block781

list_loop_end776:                                 ; preds = %list_loop_header774
  %149 = call ptr @strcat(ptr %list_buf773, ptr @close_bracket.91)
  %str_ptr_int806 = ptrtoint ptr %list_buf773 to i64
  %v2807 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int806, 1
  br label %str_merge755

sep_block781:                                     ; preds = %list_loop_body775
  %150 = call ptr @strcat(ptr %list_buf773, ptr @comma_sep.88)
  br label %elem_block782

elem_block782:                                    ; preds = %sep_block781, %list_loop_body775
  %idx_in_elem783 = load i64, ptr %idx_ptr777, align 8
  %elements_base784 = getelementptr i64, ptr %len_ptr769, i64 1
  %elem_ptr785 = getelementptr { i8, i64 }, ptr %elements_base784, i64 %idx_in_elem783
  %elem_val786 = load { i8, i64 }, ptr %elem_ptr785, align 8
  %elem_tag787 = extractvalue { i8, i64 } %elem_val786, 0
  %elem_data788 = extractvalue { i8, i64 } %elem_val786, 1
  %elem_data_ptr789 = alloca i64, align 8
  store i64 %elem_data788, ptr %elem_data_ptr789, align 8
  %elem_is_float790 = icmp eq i8 %elem_tag787, 3
  %elem_is_string791 = icmp eq i8 %elem_tag787, 4
  br i1 %elem_is_float790, label %elem_float_block792, label %elem_string_check793

elem_float_block792:                              ; preds = %elem_block782
  %elem_data_float799 = load i64, ptr %elem_data_ptr789, align 8
  %elem_float_buf800 = call ptr @malloc(i64 25)
  %elem_as_float801 = bitcast i64 %elem_data_float799 to double
  %151 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_float_buf800, i64 25, ptr @float_fmt2.89, double %elem_as_float801)
  %152 = call ptr @strcat(ptr %list_buf773, ptr %elem_float_buf800)
  br label %elem_done796

elem_string_check793:                             ; preds = %elem_block782
  br i1 %elem_is_string791, label %elem_string_print794, label %elem_int_block795

elem_string_print794:                             ; preds = %elem_string_check793
  %elem_data_str797 = load i64, ptr %elem_data_ptr789, align 8
  %elem_str_ptr798 = inttoptr i64 %elem_data_str797 to ptr
  %153 = call ptr @strcat(ptr %list_buf773, ptr %elem_str_ptr798)
  br label %elem_done796

elem_int_block795:                                ; preds = %elem_string_check793
  %elem_data_int802 = load i64, ptr %elem_data_ptr789, align 8
  %elem_int_buf803 = call ptr @malloc(i64 25)
  %154 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_int_buf803, i64 25, ptr @int_fmt2.90, i64 %elem_data_int802)
  %155 = call ptr @strcat(ptr %list_buf773, ptr %elem_int_buf803)
  br label %elem_done796

elem_done796:                                     ; preds = %elem_int_block795, %elem_float_block792, %elem_string_print794
  %idx_for_incr804 = load i64, ptr %idx_ptr777, align 8
  %next_idx805 = add i64 %idx_for_incr804, 1
  store i64 %next_idx805, ptr %idx_ptr777, align 8
  br label %list_loop_header774

add_int_int811:                                   ; preds = %str_merge755
  %sum824 = add i64 ptrtoint (ptr @str.81 to i64), %data810
  %v2825 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %sum824, 1
  br label %add_merge815

add_float812:                                     ; preds = %check_float822
  %rf826 = bitcast i64 %data810 to double
  %ri2f827 = sitofp i64 %data810 to double
  %right_as_float828 = select i1 %r_float818, double %rf826, double %ri2f827
  %fsum829 = fadd double sitofp (i64 ptrtoint (ptr @str.81 to i64) to double), %right_as_float828
  %float_bits830 = bitcast double %fsum829 to i64
  %v2831 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits830, 1
  br label %add_merge815

add_string813:                                    ; preds = %check_string823
  %rstr832 = inttoptr i64 %data810 to ptr
  %llen833 = call i64 @strlen(ptr @str.81)
  %rlen834 = call i64 @strlen(ptr %rstr832)
  %total835 = add i64 %llen833, %rlen834
  %alloc_size836 = add i64 %total835, 1
  %new_str837 = call ptr @malloc(i64 %alloc_size836)
  %156 = call ptr @memcpy(ptr %new_str837, ptr @str.81, i64 %llen833)
  %dest_offset838 = getelementptr i8, ptr %new_str837, i64 %llen833
  %rlen_plus_one839 = add i64 %rlen834, 1
  %157 = call ptr @memcpy(ptr %dest_offset838, ptr %rstr832, i64 %rlen_plus_one839)
  %str_ptr_int840 = ptrtoint ptr %new_str837 to i64
  %v2841 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int840, 1
  br label %add_merge815

add_error814:                                     ; preds = %check_string823
  br label %add_merge815

add_merge815:                                     ; preds = %add_error814, %add_string813, %add_float812, %add_int_int811
  %add_result842 = phi { i8, i64 } [ %v2825, %add_int_int811 ], [ %v2831, %add_float812 ], [ %v2841, %add_string813 ], [ zeroinitializer, %add_error814 ]
  %tag843 = extractvalue { i8, i64 } %add_result842, 0
  %data844 = extractvalue { i8, i64 } %add_result842, 1
  switch i8 %tag843, label %print_default850 [
    i8 0, label %print_nil845
    i8 1, label %print_bool846
    i8 2, label %print_int847
    i8 3, label %print_float848
    i8 4, label %print_string849
  ]

check_float822:                                   ; preds = %str_merge755
  br i1 %either_float819, label %add_float812, label %check_string823

check_string823:                                  ; preds = %check_float822
  br i1 %both_str821, label %add_string813, label %add_error814

print_nil845:                                     ; preds = %add_merge815
  %158 = call i32 (ptr, ...) @printf(ptr @fmt_nil)
  br label %print_done851

print_bool846:                                    ; preds = %add_merge815
  %is_true852 = icmp ne i64 %data844, 0
  %bool_str853 = select i1 %is_true852, ptr @fmt_true, ptr @fmt_false
  %159 = call i32 (ptr, ...) @printf(ptr %bool_str853)
  br label %print_done851

print_int847:                                     ; preds = %add_merge815
  %160 = call i32 (ptr, ...) @printf(ptr @fmt_int, i64 %data844)
  br label %print_done851

print_float848:                                   ; preds = %add_merge815
  %f854 = bitcast i64 %data844 to double
  %161 = call i32 (ptr, ...) @printf(ptr @fmt_float, double %f854)
  br label %print_done851

print_string849:                                  ; preds = %add_merge815
  %str855 = inttoptr i64 %data844 to ptr
  %162 = call i32 (ptr, ...) @printf(ptr @fmt_string, ptr %str855)
  br label %print_done851

print_default850:                                 ; preds = %add_merge815
  br label %print_done851

print_done851:                                    ; preds = %print_default850, %print_string849, %print_float848, %print_int847, %print_bool846, %print_nil845
  %163 = call i32 (ptr, ...) @printf(ptr @fmt_newline)
  ret i32 0
}
