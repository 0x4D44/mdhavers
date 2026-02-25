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
@str = private unnamed_addr constant [40 x i8] c"=== Large-Scale Fibonacci Benchmark ===\00", align 1
@str.1 = private unnamed_addr constant [19 x i8] c"Correctness check:\00", align 1
@str.2 = private unnamed_addr constant [13 x i8] c"  fib(10) = \00", align 1
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
@str.3 = private unnamed_addr constant [13 x i8] c"  fib(20) = \00", align 1
@nil_str.4 = private unnamed_addr constant [9 x i8] c"naething\00", align 1
@true_str.5 = private unnamed_addr constant [4 x i8] c"aye\00", align 1
@false_str.6 = private unnamed_addr constant [4 x i8] c"nae\00", align 1
@int_fmt.7 = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@float_fmt.8 = private unnamed_addr constant [3 x i8] c"%g\00", align 1
@open_bracket.9 = private unnamed_addr constant [2 x i8] c"[\00", align 1
@comma_sep.10 = private unnamed_addr constant [3 x i8] c", \00", align 1
@float_fmt2.11 = private unnamed_addr constant [3 x i8] c"%g\00", align 1
@int_fmt2.12 = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@close_bracket.13 = private unnamed_addr constant [2 x i8] c"]\00", align 1
@empty_str.14 = private unnamed_addr constant [1 x i8] zeroinitializer, align 1
@str.15 = private unnamed_addr constant [13 x i8] c"  fib(50) = \00", align 1
@nil_str.16 = private unnamed_addr constant [9 x i8] c"naething\00", align 1
@true_str.17 = private unnamed_addr constant [4 x i8] c"aye\00", align 1
@false_str.18 = private unnamed_addr constant [4 x i8] c"nae\00", align 1
@int_fmt.19 = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@float_fmt.20 = private unnamed_addr constant [3 x i8] c"%g\00", align 1
@open_bracket.21 = private unnamed_addr constant [2 x i8] c"[\00", align 1
@comma_sep.22 = private unnamed_addr constant [3 x i8] c", \00", align 1
@float_fmt2.23 = private unnamed_addr constant [3 x i8] c"%g\00", align 1
@int_fmt2.24 = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@close_bracket.25 = private unnamed_addr constant [2 x i8] c"]\00", align 1
@empty_str.26 = private unnamed_addr constant [1 x i8] zeroinitializer, align 1
@str.27 = private unnamed_addr constant [1 x i8] zeroinitializer, align 1
@str.28 = private unnamed_addr constant [29 x i8] c"Large-scale iterative tests:\00", align 1
@str.29 = private unnamed_addr constant [19 x i8] c"  fib_iter(1000): \00", align 1
@nil_str.30 = private unnamed_addr constant [9 x i8] c"naething\00", align 1
@true_str.31 = private unnamed_addr constant [4 x i8] c"aye\00", align 1
@false_str.32 = private unnamed_addr constant [4 x i8] c"nae\00", align 1
@int_fmt.33 = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@float_fmt.34 = private unnamed_addr constant [3 x i8] c"%g\00", align 1
@open_bracket.35 = private unnamed_addr constant [2 x i8] c"[\00", align 1
@comma_sep.36 = private unnamed_addr constant [3 x i8] c", \00", align 1
@float_fmt2.37 = private unnamed_addr constant [3 x i8] c"%g\00", align 1
@int_fmt2.38 = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@close_bracket.39 = private unnamed_addr constant [2 x i8] c"]\00", align 1
@empty_str.40 = private unnamed_addr constant [1 x i8] zeroinitializer, align 1
@str.41 = private unnamed_addr constant [4 x i8] c" us\00", align 1
@str.42 = private unnamed_addr constant [20 x i8] c"  fib_iter(10000): \00", align 1
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
@str.54 = private unnamed_addr constant [4 x i8] c" us\00", align 1
@str.55 = private unnamed_addr constant [20 x i8] c"  fib_iter(50000): \00", align 1
@nil_str.56 = private unnamed_addr constant [9 x i8] c"naething\00", align 1
@true_str.57 = private unnamed_addr constant [4 x i8] c"aye\00", align 1
@false_str.58 = private unnamed_addr constant [4 x i8] c"nae\00", align 1
@int_fmt.59 = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@float_fmt.60 = private unnamed_addr constant [3 x i8] c"%g\00", align 1
@open_bracket.61 = private unnamed_addr constant [2 x i8] c"[\00", align 1
@comma_sep.62 = private unnamed_addr constant [3 x i8] c", \00", align 1
@float_fmt2.63 = private unnamed_addr constant [3 x i8] c"%g\00", align 1
@int_fmt2.64 = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@close_bracket.65 = private unnamed_addr constant [2 x i8] c"]\00", align 1
@empty_str.66 = private unnamed_addr constant [1 x i8] zeroinitializer, align 1
@str.67 = private unnamed_addr constant [4 x i8] c" us\00", align 1
@str.68 = private unnamed_addr constant [21 x i8] c"  fib_iter(100000): \00", align 1
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
@str.80 = private unnamed_addr constant [4 x i8] c" us\00", align 1
@str.81 = private unnamed_addr constant [1 x i8] zeroinitializer, align 1
@str.82 = private unnamed_addr constant [44 x i8] c"Stress test (10000 calls to fib_iter(100)):\00", align 1
@str.83 = private unnamed_addr constant [25 x i8] c"  10000x fib_iter(100): \00", align 1
@nil_str.84 = private unnamed_addr constant [9 x i8] c"naething\00", align 1
@true_str.85 = private unnamed_addr constant [4 x i8] c"aye\00", align 1
@false_str.86 = private unnamed_addr constant [4 x i8] c"nae\00", align 1
@int_fmt.87 = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@float_fmt.88 = private unnamed_addr constant [3 x i8] c"%g\00", align 1
@open_bracket.89 = private unnamed_addr constant [2 x i8] c"[\00", align 1
@comma_sep.90 = private unnamed_addr constant [3 x i8] c", \00", align 1
@float_fmt2.91 = private unnamed_addr constant [3 x i8] c"%g\00", align 1
@int_fmt2.92 = private unnamed_addr constant [5 x i8] c"%lld\00", align 1
@close_bracket.93 = private unnamed_addr constant [2 x i8] c"]\00", align 1
@empty_str.94 = private unnamed_addr constant [1 x i8] zeroinitializer, align 1
@str.95 = private unnamed_addr constant [4 x i8] c" ms\00", align 1
@str.96 = private unnamed_addr constant [1 x i8] zeroinitializer, align 1
@str.97 = private unnamed_addr constant [39 x i8] c"=== Fibonacci large-scale complete ===\00", align 1

declare i32 @printf(ptr, ...)

declare ptr @malloc(i64)

declare ptr @realloc(ptr, i64)

declare i64 @strlen(ptr)

declare ptr @strcpy(ptr, ptr)

declare ptr @strcat(ptr, ptr)

declare i32 @snprintf(ptr, i64, ptr, ...)

declare void @exit(i32)

declare ptr @strstr(ptr, ptr)

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

define { i8, i64 } @fib_iter({ i8, i64 } %0) {
entry:
  %temp = alloca { i8, i64 }, align 8
  %temp_shadow = alloca i64, align 8
  %i = alloca { i8, i64 }, align 8
  %i_shadow = alloca i64, align 8
  %b = alloca { i8, i64 }, align 8
  %b_shadow = alloca i64, align 8
  %a = alloca { i8, i64 }, align 8
  %a_shadow = alloca i64, align 8
  %n_shadow = alloca i64, align 8
  %n = alloca { i8, i64 }, align 8
  store { i8, i64 } %0, ptr %n, align 8
  %data = extractvalue { i8, i64 } %0, 1
  store i64 %data, ptr %n_shadow, align 8
  %n_i64 = load i64, ptr %n_shadow, align 8
  %cmp_direct = icmp slt i64 %n_i64, 2
  br i1 %cmp_direct, label %then, label %else

then:                                             ; preds = %entry
  %n1 = load { i8, i64 }, ptr %n, align 8
  ret { i8, i64 } %n1

else:                                             ; preds = %entry
  br label %merge

merge:                                            ; preds = %else
  store i64 0, ptr %a_shadow, align 8
  store { i8, i64 } { i8 2, i64 0 }, ptr %a, align 8
  store i64 1, ptr %b_shadow, align 8
  store { i8, i64 } { i8 2, i64 1 }, ptr %b, align 8
  store i64 2, ptr %i_shadow, align 8
  store { i8, i64 } { i8 2, i64 2 }, ptr %i, align 8
  br label %loop

loop:                                             ; preds = %body, %merge
  %i_i64 = load i64, ptr %i_shadow, align 8
  %n_i642 = load i64, ptr %n_shadow, align 8
  %cmp_direct3 = icmp sle i64 %i_i64, %n_i642
  br i1 %cmp_direct3, label %body, label %after

body:                                             ; preds = %loop
  %a_i64 = load i64, ptr %a_shadow, align 8
  %b_i64 = load i64, ptr %b_shadow, align 8
  %add_i64 = add i64 %a_i64, %b_i64
  store i64 %add_i64, ptr %temp_shadow, align 8
  %b_i644 = load i64, ptr %b_shadow, align 8
  store i64 %b_i644, ptr %a_shadow, align 8
  %v2 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %b_i644, 1
  %temp_i64 = load i64, ptr %temp_shadow, align 8
  store i64 %temp_i64, ptr %b_shadow, align 8
  %v25 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %temp_i64, 1
  %i_i646 = load i64, ptr %i_shadow, align 8
  %add_i647 = add i64 %i_i646, 1
  store i64 %add_i647, ptr %i_shadow, align 8
  %v28 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %add_i647, 1
  br label %loop

after:                                            ; preds = %loop
  %a_sync = load i64, ptr %a_shadow, align 8
  %v29 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %a_sync, 1
  store { i8, i64 } %v29, ptr %a, align 8
  %b_sync = load i64, ptr %b_shadow, align 8
  %v210 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %b_sync, 1
  store { i8, i64 } %v210, ptr %b, align 8
  %i_sync = load i64, ptr %i_shadow, align 8
  %v211 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %i_sync, 1
  store { i8, i64 } %v211, ptr %i, align 8
  %n_sync = load i64, ptr %n_shadow, align 8
  %v212 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %n_sync, 1
  store { i8, i64 } %v212, ptr %n, align 8
  %temp_sync = load i64, ptr %temp_shadow, align 8
  %v213 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %temp_sync, 1
  store { i8, i64 } %v213, ptr %temp, align 8
  %b14 = load { i8, i64 }, ptr %b, align 8
  ret { i8, i64 } %b14
}

define i32 @main() {
entry:
  %i = alloca { i8, i64 }, align 8
  %i_shadow = alloca i64, align 8
  %elapsed = alloca { i8, i64 }, align 8
  %result = alloca { i8, i64 }, align 8
  %start = alloca { i8, i64 }, align 8
  switch i8 4, label %print_default [
    i8 0, label %print_nil
    i8 1, label %print_bool
    i8 2, label %print_int
    i8 3, label %print_float
    i8 4, label %print_string
  ]

print_nil:                                        ; preds = %entry
  %0 = call i32 (ptr, ...) @printf(ptr @fmt_nil)
  br label %print_done

print_bool:                                       ; preds = %entry
  %1 = call i32 (ptr, ...) @printf(ptr select (i1 icmp ne (i64 ptrtoint (ptr @str to i64), i64 0), ptr @fmt_true, ptr @fmt_false))
  br label %print_done

print_int:                                        ; preds = %entry
  %2 = call i32 (ptr, ...) @printf(ptr @fmt_int, i64 ptrtoint (ptr @str to i64))
  br label %print_done

print_float:                                      ; preds = %entry
  %3 = call i32 (ptr, ...) @printf(ptr @fmt_float, double bitcast (i64 ptrtoint (ptr @str to i64) to double))
  br label %print_done

print_string:                                     ; preds = %entry
  %4 = call i32 (ptr, ...) @printf(ptr @fmt_string, ptr @str)
  br label %print_done

print_default:                                    ; preds = %entry
  br label %print_done

print_done:                                       ; preds = %print_default, %print_string, %print_float, %print_int, %print_bool, %print_nil
  %5 = call i32 (ptr, ...) @printf(ptr @fmt_newline)
  switch i8 4, label %print_default6 [
    i8 0, label %print_nil1
    i8 1, label %print_bool2
    i8 2, label %print_int3
    i8 3, label %print_float4
    i8 4, label %print_string5
  ]

print_nil1:                                       ; preds = %print_done
  %6 = call i32 (ptr, ...) @printf(ptr @fmt_nil)
  br label %print_done7

print_bool2:                                      ; preds = %print_done
  %7 = call i32 (ptr, ...) @printf(ptr select (i1 icmp ne (i64 ptrtoint (ptr @str.1 to i64), i64 0), ptr @fmt_true, ptr @fmt_false))
  br label %print_done7

print_int3:                                       ; preds = %print_done
  %8 = call i32 (ptr, ...) @printf(ptr @fmt_int, i64 ptrtoint (ptr @str.1 to i64))
  br label %print_done7

print_float4:                                     ; preds = %print_done
  %9 = call i32 (ptr, ...) @printf(ptr @fmt_float, double bitcast (i64 ptrtoint (ptr @str.1 to i64) to double))
  br label %print_done7

print_string5:                                    ; preds = %print_done
  %10 = call i32 (ptr, ...) @printf(ptr @fmt_string, ptr @str.1)
  br label %print_done7

print_default6:                                   ; preds = %print_done
  br label %print_done7

print_done7:                                      ; preds = %print_default6, %print_string5, %print_float4, %print_int3, %print_bool2, %print_nil1
  %11 = call i32 (ptr, ...) @printf(ptr @fmt_newline)
  %call = tail call { i8, i64 } @fib_iter({ i8, i64 } { i8 2, i64 10 })
  %tag = extractvalue { i8, i64 } %call, 0
  %data = extractvalue { i8, i64 } %call, 1
  switch i8 %tag, label %str_default [
    i8 0, label %str_nil
    i8 1, label %str_bool
    i8 2, label %str_int
    i8 3, label %str_float
    i8 4, label %str_string
    i8 5, label %str_list
  ]

str_nil:                                          ; preds = %print_done7
  br label %str_merge

str_bool:                                         ; preds = %print_done7
  %is_true = icmp ne i64 %data, 0
  %bool_ptr = select i1 %is_true, ptr @true_str, ptr @false_str
  %str_ptr_int = ptrtoint ptr %bool_ptr to i64
  %v2 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int, 1
  br label %str_merge

str_int:                                          ; preds = %print_done7
  %int_buf = call ptr @malloc(i64 32)
  %12 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %int_buf, i64 32, ptr @int_fmt, i64 %data)
  %str_ptr_int8 = ptrtoint ptr %int_buf to i64
  %v29 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int8, 1
  br label %str_merge

str_float:                                        ; preds = %print_done7
  %float_buf = call ptr @malloc(i64 32)
  %f = bitcast i64 %data to double
  %13 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %float_buf, i64 32, ptr @float_fmt, double %f)
  %str_ptr_int10 = ptrtoint ptr %float_buf to i64
  %v211 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int10, 1
  br label %str_merge

str_string:                                       ; preds = %print_done7
  br label %str_merge

str_default:                                      ; preds = %print_done7
  br label %str_merge

str_merge:                                        ; preds = %str_default, %list_loop_end, %str_string, %str_float, %str_int, %str_bool, %str_nil
  %str_result = phi { i8, i64 } [ { i8 4, i64 ptrtoint (ptr @nil_str to i64) }, %str_nil ], [ %v2, %str_bool ], [ %v29, %str_int ], [ %v211, %str_float ], [ %call, %str_string ], [ %v213, %list_loop_end ], [ { i8 4, i64 ptrtoint (ptr @empty_str to i64) }, %str_default ]
  %tag14 = extractvalue { i8, i64 } %str_result, 0
  %data15 = extractvalue { i8, i64 } %str_result, 1
  %r_int = icmp eq i8 %tag14, 2
  %both_int = and i1 false, %r_int
  %r_float = icmp eq i8 %tag14, 3
  %either_float = or i1 false, %r_float
  %r_str = icmp eq i8 %tag14, 4
  %both_str = and i1 true, %r_str
  br i1 %both_int, label %add_int_int, label %check_float

str_list:                                         ; preds = %print_done7
  %list_ptr = inttoptr i64 %data to ptr
  %list_len = load i64, ptr %list_ptr, align 8
  %buf_size_mul = mul i64 %list_len, 25
  %list_buf_size = add i64 %buf_size_mul, 3
  %list_buf = call ptr @malloc(i64 %list_buf_size)
  %14 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %list_buf, i64 %list_buf_size, ptr @open_bracket)
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
  %15 = call ptr @strcat(ptr %list_buf, ptr @close_bracket)
  %str_ptr_int12 = ptrtoint ptr %list_buf to i64
  %v213 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int12, 1
  br label %str_merge

sep_block:                                        ; preds = %list_loop_body
  %16 = call ptr @strcat(ptr %list_buf, ptr @comma_sep)
  br label %elem_block

elem_block:                                       ; preds = %sep_block, %list_loop_body
  %idx_in_elem = load i64, ptr %idx_ptr, align 8
  %elements_base = getelementptr i64, ptr %list_ptr, i64 1
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
  %17 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_float_buf, i64 25, ptr @float_fmt2, double %elem_as_float)
  %18 = call ptr @strcat(ptr %list_buf, ptr %elem_float_buf)
  br label %elem_done

elem_string_check:                                ; preds = %elem_block
  br i1 %elem_is_string, label %elem_string_print, label %elem_int_block

elem_string_print:                                ; preds = %elem_string_check
  %elem_data_str = load i64, ptr %elem_data_ptr, align 8
  %elem_str_ptr = inttoptr i64 %elem_data_str to ptr
  %19 = call ptr @strcat(ptr %list_buf, ptr %elem_str_ptr)
  br label %elem_done

elem_int_block:                                   ; preds = %elem_string_check
  %elem_data_int = load i64, ptr %elem_data_ptr, align 8
  %elem_int_buf = call ptr @malloc(i64 25)
  %20 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_int_buf, i64 25, ptr @int_fmt2, i64 %elem_data_int)
  %21 = call ptr @strcat(ptr %list_buf, ptr %elem_int_buf)
  br label %elem_done

elem_done:                                        ; preds = %elem_int_block, %elem_float_block, %elem_string_print
  %idx_for_incr = load i64, ptr %idx_ptr, align 8
  %next_idx = add i64 %idx_for_incr, 1
  store i64 %next_idx, ptr %idx_ptr, align 8
  br label %list_loop_header

add_int_int:                                      ; preds = %str_merge
  %sum = add i64 ptrtoint (ptr @str.2 to i64), %data15
  %v216 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %sum, 1
  br label %add_merge

add_float:                                        ; preds = %check_float
  %rf = bitcast i64 %data15 to double
  %ri2f = sitofp i64 %data15 to double
  %right_as_float = select i1 %r_float, double %rf, double %ri2f
  %fsum = fadd double sitofp (i64 ptrtoint (ptr @str.2 to i64) to double), %right_as_float
  %float_bits = bitcast double %fsum to i64
  %v217 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits, 1
  br label %add_merge

add_string:                                       ; preds = %check_string
  %rstr = inttoptr i64 %data15 to ptr
  %llen = call i64 @strlen(ptr @str.2)
  %rlen = call i64 @strlen(ptr %rstr)
  %total = add i64 %llen, %rlen
  %alloc_size = add i64 %total, 1
  %new_str = call ptr @malloc(i64 %alloc_size)
  %22 = call ptr @strcpy(ptr %new_str, ptr @str.2)
  %23 = call ptr @strcat(ptr %new_str, ptr %rstr)
  %str_ptr_int18 = ptrtoint ptr %new_str to i64
  %v219 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int18, 1
  br label %add_merge

add_error:                                        ; preds = %check_string
  br label %add_merge

add_merge:                                        ; preds = %add_error, %add_string, %add_float, %add_int_int
  %add_result = phi { i8, i64 } [ %v216, %add_int_int ], [ %v217, %add_float ], [ %v219, %add_string ], [ zeroinitializer, %add_error ]
  %tag20 = extractvalue { i8, i64 } %add_result, 0
  %data21 = extractvalue { i8, i64 } %add_result, 1
  switch i8 %tag20, label %print_default27 [
    i8 0, label %print_nil22
    i8 1, label %print_bool23
    i8 2, label %print_int24
    i8 3, label %print_float25
    i8 4, label %print_string26
  ]

check_float:                                      ; preds = %str_merge
  br i1 %either_float, label %add_float, label %check_string

check_string:                                     ; preds = %check_float
  br i1 %both_str, label %add_string, label %add_error

print_nil22:                                      ; preds = %add_merge
  %24 = call i32 (ptr, ...) @printf(ptr @fmt_nil)
  br label %print_done28

print_bool23:                                     ; preds = %add_merge
  %is_true29 = icmp ne i64 %data21, 0
  %bool_str = select i1 %is_true29, ptr @fmt_true, ptr @fmt_false
  %25 = call i32 (ptr, ...) @printf(ptr %bool_str)
  br label %print_done28

print_int24:                                      ; preds = %add_merge
  %26 = call i32 (ptr, ...) @printf(ptr @fmt_int, i64 %data21)
  br label %print_done28

print_float25:                                    ; preds = %add_merge
  %f30 = bitcast i64 %data21 to double
  %27 = call i32 (ptr, ...) @printf(ptr @fmt_float, double %f30)
  br label %print_done28

print_string26:                                   ; preds = %add_merge
  %str = inttoptr i64 %data21 to ptr
  %28 = call i32 (ptr, ...) @printf(ptr @fmt_string, ptr %str)
  br label %print_done28

print_default27:                                  ; preds = %add_merge
  br label %print_done28

print_done28:                                     ; preds = %print_default27, %print_string26, %print_float25, %print_int24, %print_bool23, %print_nil22
  %29 = call i32 (ptr, ...) @printf(ptr @fmt_newline)
  %call31 = tail call { i8, i64 } @fib_iter({ i8, i64 } { i8 2, i64 20 })
  %tag32 = extractvalue { i8, i64 } %call31, 0
  %data33 = extractvalue { i8, i64 } %call31, 1
  switch i8 %tag32, label %str_default39 [
    i8 0, label %str_nil34
    i8 1, label %str_bool35
    i8 2, label %str_int36
    i8 3, label %str_float37
    i8 4, label %str_string38
    i8 5, label %str_list41
  ]

str_nil34:                                        ; preds = %print_done28
  br label %str_merge40

str_bool35:                                       ; preds = %print_done28
  %is_true42 = icmp ne i64 %data33, 0
  %bool_ptr43 = select i1 %is_true42, ptr @true_str.5, ptr @false_str.6
  %str_ptr_int44 = ptrtoint ptr %bool_ptr43 to i64
  %v245 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int44, 1
  br label %str_merge40

str_int36:                                        ; preds = %print_done28
  %int_buf46 = call ptr @malloc(i64 32)
  %30 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %int_buf46, i64 32, ptr @int_fmt.7, i64 %data33)
  %str_ptr_int47 = ptrtoint ptr %int_buf46 to i64
  %v248 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int47, 1
  br label %str_merge40

str_float37:                                      ; preds = %print_done28
  %float_buf49 = call ptr @malloc(i64 32)
  %f50 = bitcast i64 %data33 to double
  %31 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %float_buf49, i64 32, ptr @float_fmt.8, double %f50)
  %str_ptr_int51 = ptrtoint ptr %float_buf49 to i64
  %v252 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int51, 1
  br label %str_merge40

str_string38:                                     ; preds = %print_done28
  br label %str_merge40

str_default39:                                    ; preds = %print_done28
  br label %str_merge40

str_merge40:                                      ; preds = %str_default39, %list_loop_end60, %str_string38, %str_float37, %str_int36, %str_bool35, %str_nil34
  %str_result92 = phi { i8, i64 } [ { i8 4, i64 ptrtoint (ptr @nil_str.4 to i64) }, %str_nil34 ], [ %v245, %str_bool35 ], [ %v248, %str_int36 ], [ %v252, %str_float37 ], [ %call31, %str_string38 ], [ %v291, %list_loop_end60 ], [ { i8 4, i64 ptrtoint (ptr @empty_str.14 to i64) }, %str_default39 ]
  %tag93 = extractvalue { i8, i64 } %str_result92, 0
  %data94 = extractvalue { i8, i64 } %str_result92, 1
  %r_int100 = icmp eq i8 %tag93, 2
  %both_int101 = and i1 false, %r_int100
  %r_float102 = icmp eq i8 %tag93, 3
  %either_float103 = or i1 false, %r_float102
  %r_str104 = icmp eq i8 %tag93, 4
  %both_str105 = and i1 true, %r_str104
  br i1 %both_int101, label %add_int_int95, label %check_float106

str_list41:                                       ; preds = %print_done28
  %list_ptr53 = inttoptr i64 %data33 to ptr
  %list_len54 = load i64, ptr %list_ptr53, align 8
  %buf_size_mul55 = mul i64 %list_len54, 25
  %list_buf_size56 = add i64 %buf_size_mul55, 3
  %list_buf57 = call ptr @malloc(i64 %list_buf_size56)
  %32 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %list_buf57, i64 %list_buf_size56, ptr @open_bracket.9)
  %idx_ptr61 = alloca i64, align 8
  store i64 0, ptr %idx_ptr61, align 8
  br label %list_loop_header58

list_loop_header58:                               ; preds = %elem_done80, %str_list41
  %idx62 = load i64, ptr %idx_ptr61, align 8
  %loop_cond63 = icmp ult i64 %idx62, %list_len54
  br i1 %loop_cond63, label %list_loop_body59, label %list_loop_end60

list_loop_body59:                                 ; preds = %list_loop_header58
  %is_first64 = icmp eq i64 %idx62, 0
  br i1 %is_first64, label %elem_block66, label %sep_block65

list_loop_end60:                                  ; preds = %list_loop_header58
  %33 = call ptr @strcat(ptr %list_buf57, ptr @close_bracket.13)
  %str_ptr_int90 = ptrtoint ptr %list_buf57 to i64
  %v291 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int90, 1
  br label %str_merge40

sep_block65:                                      ; preds = %list_loop_body59
  %34 = call ptr @strcat(ptr %list_buf57, ptr @comma_sep.10)
  br label %elem_block66

elem_block66:                                     ; preds = %sep_block65, %list_loop_body59
  %idx_in_elem67 = load i64, ptr %idx_ptr61, align 8
  %elements_base68 = getelementptr i64, ptr %list_ptr53, i64 1
  %elem_ptr69 = getelementptr { i8, i64 }, ptr %elements_base68, i64 %idx_in_elem67
  %elem_val70 = load { i8, i64 }, ptr %elem_ptr69, align 8
  %elem_tag71 = extractvalue { i8, i64 } %elem_val70, 0
  %elem_data72 = extractvalue { i8, i64 } %elem_val70, 1
  %elem_data_ptr73 = alloca i64, align 8
  store i64 %elem_data72, ptr %elem_data_ptr73, align 8
  %elem_is_float74 = icmp eq i8 %elem_tag71, 3
  %elem_is_string75 = icmp eq i8 %elem_tag71, 4
  br i1 %elem_is_float74, label %elem_float_block76, label %elem_string_check77

elem_float_block76:                               ; preds = %elem_block66
  %elem_data_float83 = load i64, ptr %elem_data_ptr73, align 8
  %elem_float_buf84 = call ptr @malloc(i64 25)
  %elem_as_float85 = bitcast i64 %elem_data_float83 to double
  %35 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_float_buf84, i64 25, ptr @float_fmt2.11, double %elem_as_float85)
  %36 = call ptr @strcat(ptr %list_buf57, ptr %elem_float_buf84)
  br label %elem_done80

elem_string_check77:                              ; preds = %elem_block66
  br i1 %elem_is_string75, label %elem_string_print78, label %elem_int_block79

elem_string_print78:                              ; preds = %elem_string_check77
  %elem_data_str81 = load i64, ptr %elem_data_ptr73, align 8
  %elem_str_ptr82 = inttoptr i64 %elem_data_str81 to ptr
  %37 = call ptr @strcat(ptr %list_buf57, ptr %elem_str_ptr82)
  br label %elem_done80

elem_int_block79:                                 ; preds = %elem_string_check77
  %elem_data_int86 = load i64, ptr %elem_data_ptr73, align 8
  %elem_int_buf87 = call ptr @malloc(i64 25)
  %38 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_int_buf87, i64 25, ptr @int_fmt2.12, i64 %elem_data_int86)
  %39 = call ptr @strcat(ptr %list_buf57, ptr %elem_int_buf87)
  br label %elem_done80

elem_done80:                                      ; preds = %elem_int_block79, %elem_float_block76, %elem_string_print78
  %idx_for_incr88 = load i64, ptr %idx_ptr61, align 8
  %next_idx89 = add i64 %idx_for_incr88, 1
  store i64 %next_idx89, ptr %idx_ptr61, align 8
  br label %list_loop_header58

add_int_int95:                                    ; preds = %str_merge40
  %sum108 = add i64 ptrtoint (ptr @str.3 to i64), %data94
  %v2109 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %sum108, 1
  br label %add_merge99

add_float96:                                      ; preds = %check_float106
  %rf110 = bitcast i64 %data94 to double
  %ri2f111 = sitofp i64 %data94 to double
  %right_as_float112 = select i1 %r_float102, double %rf110, double %ri2f111
  %fsum113 = fadd double sitofp (i64 ptrtoint (ptr @str.3 to i64) to double), %right_as_float112
  %float_bits114 = bitcast double %fsum113 to i64
  %v2115 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits114, 1
  br label %add_merge99

add_string97:                                     ; preds = %check_string107
  %rstr116 = inttoptr i64 %data94 to ptr
  %llen117 = call i64 @strlen(ptr @str.3)
  %rlen118 = call i64 @strlen(ptr %rstr116)
  %total119 = add i64 %llen117, %rlen118
  %alloc_size120 = add i64 %total119, 1
  %new_str121 = call ptr @malloc(i64 %alloc_size120)
  %40 = call ptr @strcpy(ptr %new_str121, ptr @str.3)
  %41 = call ptr @strcat(ptr %new_str121, ptr %rstr116)
  %str_ptr_int122 = ptrtoint ptr %new_str121 to i64
  %v2123 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int122, 1
  br label %add_merge99

add_error98:                                      ; preds = %check_string107
  br label %add_merge99

add_merge99:                                      ; preds = %add_error98, %add_string97, %add_float96, %add_int_int95
  %add_result124 = phi { i8, i64 } [ %v2109, %add_int_int95 ], [ %v2115, %add_float96 ], [ %v2123, %add_string97 ], [ zeroinitializer, %add_error98 ]
  %tag125 = extractvalue { i8, i64 } %add_result124, 0
  %data126 = extractvalue { i8, i64 } %add_result124, 1
  switch i8 %tag125, label %print_default132 [
    i8 0, label %print_nil127
    i8 1, label %print_bool128
    i8 2, label %print_int129
    i8 3, label %print_float130
    i8 4, label %print_string131
  ]

check_float106:                                   ; preds = %str_merge40
  br i1 %either_float103, label %add_float96, label %check_string107

check_string107:                                  ; preds = %check_float106
  br i1 %both_str105, label %add_string97, label %add_error98

print_nil127:                                     ; preds = %add_merge99
  %42 = call i32 (ptr, ...) @printf(ptr @fmt_nil)
  br label %print_done133

print_bool128:                                    ; preds = %add_merge99
  %is_true134 = icmp ne i64 %data126, 0
  %bool_str135 = select i1 %is_true134, ptr @fmt_true, ptr @fmt_false
  %43 = call i32 (ptr, ...) @printf(ptr %bool_str135)
  br label %print_done133

print_int129:                                     ; preds = %add_merge99
  %44 = call i32 (ptr, ...) @printf(ptr @fmt_int, i64 %data126)
  br label %print_done133

print_float130:                                   ; preds = %add_merge99
  %f136 = bitcast i64 %data126 to double
  %45 = call i32 (ptr, ...) @printf(ptr @fmt_float, double %f136)
  br label %print_done133

print_string131:                                  ; preds = %add_merge99
  %str137 = inttoptr i64 %data126 to ptr
  %46 = call i32 (ptr, ...) @printf(ptr @fmt_string, ptr %str137)
  br label %print_done133

print_default132:                                 ; preds = %add_merge99
  br label %print_done133

print_done133:                                    ; preds = %print_default132, %print_string131, %print_float130, %print_int129, %print_bool128, %print_nil127
  %47 = call i32 (ptr, ...) @printf(ptr @fmt_newline)
  %call138 = tail call { i8, i64 } @fib_iter({ i8, i64 } { i8 2, i64 50 })
  %tag139 = extractvalue { i8, i64 } %call138, 0
  %data140 = extractvalue { i8, i64 } %call138, 1
  switch i8 %tag139, label %str_default146 [
    i8 0, label %str_nil141
    i8 1, label %str_bool142
    i8 2, label %str_int143
    i8 3, label %str_float144
    i8 4, label %str_string145
    i8 5, label %str_list148
  ]

str_nil141:                                       ; preds = %print_done133
  br label %str_merge147

str_bool142:                                      ; preds = %print_done133
  %is_true149 = icmp ne i64 %data140, 0
  %bool_ptr150 = select i1 %is_true149, ptr @true_str.17, ptr @false_str.18
  %str_ptr_int151 = ptrtoint ptr %bool_ptr150 to i64
  %v2152 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int151, 1
  br label %str_merge147

str_int143:                                       ; preds = %print_done133
  %int_buf153 = call ptr @malloc(i64 32)
  %48 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %int_buf153, i64 32, ptr @int_fmt.19, i64 %data140)
  %str_ptr_int154 = ptrtoint ptr %int_buf153 to i64
  %v2155 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int154, 1
  br label %str_merge147

str_float144:                                     ; preds = %print_done133
  %float_buf156 = call ptr @malloc(i64 32)
  %f157 = bitcast i64 %data140 to double
  %49 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %float_buf156, i64 32, ptr @float_fmt.20, double %f157)
  %str_ptr_int158 = ptrtoint ptr %float_buf156 to i64
  %v2159 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int158, 1
  br label %str_merge147

str_string145:                                    ; preds = %print_done133
  br label %str_merge147

str_default146:                                   ; preds = %print_done133
  br label %str_merge147

str_merge147:                                     ; preds = %str_default146, %list_loop_end167, %str_string145, %str_float144, %str_int143, %str_bool142, %str_nil141
  %str_result199 = phi { i8, i64 } [ { i8 4, i64 ptrtoint (ptr @nil_str.16 to i64) }, %str_nil141 ], [ %v2152, %str_bool142 ], [ %v2155, %str_int143 ], [ %v2159, %str_float144 ], [ %call138, %str_string145 ], [ %v2198, %list_loop_end167 ], [ { i8 4, i64 ptrtoint (ptr @empty_str.26 to i64) }, %str_default146 ]
  %tag200 = extractvalue { i8, i64 } %str_result199, 0
  %data201 = extractvalue { i8, i64 } %str_result199, 1
  %r_int207 = icmp eq i8 %tag200, 2
  %both_int208 = and i1 false, %r_int207
  %r_float209 = icmp eq i8 %tag200, 3
  %either_float210 = or i1 false, %r_float209
  %r_str211 = icmp eq i8 %tag200, 4
  %both_str212 = and i1 true, %r_str211
  br i1 %both_int208, label %add_int_int202, label %check_float213

str_list148:                                      ; preds = %print_done133
  %list_ptr160 = inttoptr i64 %data140 to ptr
  %list_len161 = load i64, ptr %list_ptr160, align 8
  %buf_size_mul162 = mul i64 %list_len161, 25
  %list_buf_size163 = add i64 %buf_size_mul162, 3
  %list_buf164 = call ptr @malloc(i64 %list_buf_size163)
  %50 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %list_buf164, i64 %list_buf_size163, ptr @open_bracket.21)
  %idx_ptr168 = alloca i64, align 8
  store i64 0, ptr %idx_ptr168, align 8
  br label %list_loop_header165

list_loop_header165:                              ; preds = %elem_done187, %str_list148
  %idx169 = load i64, ptr %idx_ptr168, align 8
  %loop_cond170 = icmp ult i64 %idx169, %list_len161
  br i1 %loop_cond170, label %list_loop_body166, label %list_loop_end167

list_loop_body166:                                ; preds = %list_loop_header165
  %is_first171 = icmp eq i64 %idx169, 0
  br i1 %is_first171, label %elem_block173, label %sep_block172

list_loop_end167:                                 ; preds = %list_loop_header165
  %51 = call ptr @strcat(ptr %list_buf164, ptr @close_bracket.25)
  %str_ptr_int197 = ptrtoint ptr %list_buf164 to i64
  %v2198 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int197, 1
  br label %str_merge147

sep_block172:                                     ; preds = %list_loop_body166
  %52 = call ptr @strcat(ptr %list_buf164, ptr @comma_sep.22)
  br label %elem_block173

elem_block173:                                    ; preds = %sep_block172, %list_loop_body166
  %idx_in_elem174 = load i64, ptr %idx_ptr168, align 8
  %elements_base175 = getelementptr i64, ptr %list_ptr160, i64 1
  %elem_ptr176 = getelementptr { i8, i64 }, ptr %elements_base175, i64 %idx_in_elem174
  %elem_val177 = load { i8, i64 }, ptr %elem_ptr176, align 8
  %elem_tag178 = extractvalue { i8, i64 } %elem_val177, 0
  %elem_data179 = extractvalue { i8, i64 } %elem_val177, 1
  %elem_data_ptr180 = alloca i64, align 8
  store i64 %elem_data179, ptr %elem_data_ptr180, align 8
  %elem_is_float181 = icmp eq i8 %elem_tag178, 3
  %elem_is_string182 = icmp eq i8 %elem_tag178, 4
  br i1 %elem_is_float181, label %elem_float_block183, label %elem_string_check184

elem_float_block183:                              ; preds = %elem_block173
  %elem_data_float190 = load i64, ptr %elem_data_ptr180, align 8
  %elem_float_buf191 = call ptr @malloc(i64 25)
  %elem_as_float192 = bitcast i64 %elem_data_float190 to double
  %53 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_float_buf191, i64 25, ptr @float_fmt2.23, double %elem_as_float192)
  %54 = call ptr @strcat(ptr %list_buf164, ptr %elem_float_buf191)
  br label %elem_done187

elem_string_check184:                             ; preds = %elem_block173
  br i1 %elem_is_string182, label %elem_string_print185, label %elem_int_block186

elem_string_print185:                             ; preds = %elem_string_check184
  %elem_data_str188 = load i64, ptr %elem_data_ptr180, align 8
  %elem_str_ptr189 = inttoptr i64 %elem_data_str188 to ptr
  %55 = call ptr @strcat(ptr %list_buf164, ptr %elem_str_ptr189)
  br label %elem_done187

elem_int_block186:                                ; preds = %elem_string_check184
  %elem_data_int193 = load i64, ptr %elem_data_ptr180, align 8
  %elem_int_buf194 = call ptr @malloc(i64 25)
  %56 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_int_buf194, i64 25, ptr @int_fmt2.24, i64 %elem_data_int193)
  %57 = call ptr @strcat(ptr %list_buf164, ptr %elem_int_buf194)
  br label %elem_done187

elem_done187:                                     ; preds = %elem_int_block186, %elem_float_block183, %elem_string_print185
  %idx_for_incr195 = load i64, ptr %idx_ptr168, align 8
  %next_idx196 = add i64 %idx_for_incr195, 1
  store i64 %next_idx196, ptr %idx_ptr168, align 8
  br label %list_loop_header165

add_int_int202:                                   ; preds = %str_merge147
  %sum215 = add i64 ptrtoint (ptr @str.15 to i64), %data201
  %v2216 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %sum215, 1
  br label %add_merge206

add_float203:                                     ; preds = %check_float213
  %rf217 = bitcast i64 %data201 to double
  %ri2f218 = sitofp i64 %data201 to double
  %right_as_float219 = select i1 %r_float209, double %rf217, double %ri2f218
  %fsum220 = fadd double sitofp (i64 ptrtoint (ptr @str.15 to i64) to double), %right_as_float219
  %float_bits221 = bitcast double %fsum220 to i64
  %v2222 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits221, 1
  br label %add_merge206

add_string204:                                    ; preds = %check_string214
  %rstr223 = inttoptr i64 %data201 to ptr
  %llen224 = call i64 @strlen(ptr @str.15)
  %rlen225 = call i64 @strlen(ptr %rstr223)
  %total226 = add i64 %llen224, %rlen225
  %alloc_size227 = add i64 %total226, 1
  %new_str228 = call ptr @malloc(i64 %alloc_size227)
  %58 = call ptr @strcpy(ptr %new_str228, ptr @str.15)
  %59 = call ptr @strcat(ptr %new_str228, ptr %rstr223)
  %str_ptr_int229 = ptrtoint ptr %new_str228 to i64
  %v2230 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int229, 1
  br label %add_merge206

add_error205:                                     ; preds = %check_string214
  br label %add_merge206

add_merge206:                                     ; preds = %add_error205, %add_string204, %add_float203, %add_int_int202
  %add_result231 = phi { i8, i64 } [ %v2216, %add_int_int202 ], [ %v2222, %add_float203 ], [ %v2230, %add_string204 ], [ zeroinitializer, %add_error205 ]
  %tag232 = extractvalue { i8, i64 } %add_result231, 0
  %data233 = extractvalue { i8, i64 } %add_result231, 1
  switch i8 %tag232, label %print_default239 [
    i8 0, label %print_nil234
    i8 1, label %print_bool235
    i8 2, label %print_int236
    i8 3, label %print_float237
    i8 4, label %print_string238
  ]

check_float213:                                   ; preds = %str_merge147
  br i1 %either_float210, label %add_float203, label %check_string214

check_string214:                                  ; preds = %check_float213
  br i1 %both_str212, label %add_string204, label %add_error205

print_nil234:                                     ; preds = %add_merge206
  %60 = call i32 (ptr, ...) @printf(ptr @fmt_nil)
  br label %print_done240

print_bool235:                                    ; preds = %add_merge206
  %is_true241 = icmp ne i64 %data233, 0
  %bool_str242 = select i1 %is_true241, ptr @fmt_true, ptr @fmt_false
  %61 = call i32 (ptr, ...) @printf(ptr %bool_str242)
  br label %print_done240

print_int236:                                     ; preds = %add_merge206
  %62 = call i32 (ptr, ...) @printf(ptr @fmt_int, i64 %data233)
  br label %print_done240

print_float237:                                   ; preds = %add_merge206
  %f243 = bitcast i64 %data233 to double
  %63 = call i32 (ptr, ...) @printf(ptr @fmt_float, double %f243)
  br label %print_done240

print_string238:                                  ; preds = %add_merge206
  %str244 = inttoptr i64 %data233 to ptr
  %64 = call i32 (ptr, ...) @printf(ptr @fmt_string, ptr %str244)
  br label %print_done240

print_default239:                                 ; preds = %add_merge206
  br label %print_done240

print_done240:                                    ; preds = %print_default239, %print_string238, %print_float237, %print_int236, %print_bool235, %print_nil234
  %65 = call i32 (ptr, ...) @printf(ptr @fmt_newline)
  switch i8 4, label %print_default250 [
    i8 0, label %print_nil245
    i8 1, label %print_bool246
    i8 2, label %print_int247
    i8 3, label %print_float248
    i8 4, label %print_string249
  ]

print_nil245:                                     ; preds = %print_done240
  %66 = call i32 (ptr, ...) @printf(ptr @fmt_nil)
  br label %print_done251

print_bool246:                                    ; preds = %print_done240
  %67 = call i32 (ptr, ...) @printf(ptr select (i1 icmp ne (i64 ptrtoint (ptr @str.27 to i64), i64 0), ptr @fmt_true, ptr @fmt_false))
  br label %print_done251

print_int247:                                     ; preds = %print_done240
  %68 = call i32 (ptr, ...) @printf(ptr @fmt_int, i64 ptrtoint (ptr @str.27 to i64))
  br label %print_done251

print_float248:                                   ; preds = %print_done240
  %69 = call i32 (ptr, ...) @printf(ptr @fmt_float, double bitcast (i64 ptrtoint (ptr @str.27 to i64) to double))
  br label %print_done251

print_string249:                                  ; preds = %print_done240
  %70 = call i32 (ptr, ...) @printf(ptr @fmt_string, ptr @str.27)
  br label %print_done251

print_default250:                                 ; preds = %print_done240
  br label %print_done251

print_done251:                                    ; preds = %print_default250, %print_string249, %print_float248, %print_int247, %print_bool246, %print_nil245
  %71 = call i32 (ptr, ...) @printf(ptr @fmt_newline)
  switch i8 4, label %print_default257 [
    i8 0, label %print_nil252
    i8 1, label %print_bool253
    i8 2, label %print_int254
    i8 3, label %print_float255
    i8 4, label %print_string256
  ]

print_nil252:                                     ; preds = %print_done251
  %72 = call i32 (ptr, ...) @printf(ptr @fmt_nil)
  br label %print_done258

print_bool253:                                    ; preds = %print_done251
  %73 = call i32 (ptr, ...) @printf(ptr select (i1 icmp ne (i64 ptrtoint (ptr @str.28 to i64), i64 0), ptr @fmt_true, ptr @fmt_false))
  br label %print_done258

print_int254:                                     ; preds = %print_done251
  %74 = call i32 (ptr, ...) @printf(ptr @fmt_int, i64 ptrtoint (ptr @str.28 to i64))
  br label %print_done258

print_float255:                                   ; preds = %print_done251
  %75 = call i32 (ptr, ...) @printf(ptr @fmt_float, double bitcast (i64 ptrtoint (ptr @str.28 to i64) to double))
  br label %print_done258

print_string256:                                  ; preds = %print_done251
  %76 = call i32 (ptr, ...) @printf(ptr @fmt_string, ptr @str.28)
  br label %print_done258

print_default257:                                 ; preds = %print_done251
  br label %print_done258

print_done258:                                    ; preds = %print_default257, %print_string256, %print_float255, %print_int254, %print_bool253, %print_nil252
  %77 = call i32 (ptr, ...) @printf(ptr @fmt_newline)
  %timespec = alloca { i64, i64 }, align 8
  %clock_result = call i32 @clock_gettime(i32 1, ptr %timespec)
  %sec_ptr = getelementptr inbounds { i64, i64 }, ptr %timespec, i32 0, i32 0
  %tv_sec = load i64, ptr %sec_ptr, align 8
  %nsec_ptr = getelementptr inbounds { i64, i64 }, ptr %timespec, i32 0, i32 1
  %tv_nsec = load i64, ptr %nsec_ptr, align 8
  %sec_ns = mul i64 %tv_sec, 1000000000
  %total_ns = add i64 %sec_ns, %tv_nsec
  %v2259 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %total_ns, 1
  store { i8, i64 } %v2259, ptr %start, align 8
  %call260 = tail call { i8, i64 } @fib_iter({ i8, i64 } { i8 2, i64 1000 })
  store { i8, i64 } %call260, ptr %result, align 8
  %timespec261 = alloca { i64, i64 }, align 8
  %clock_result262 = call i32 @clock_gettime(i32 1, ptr %timespec261)
  %sec_ptr263 = getelementptr inbounds { i64, i64 }, ptr %timespec261, i32 0, i32 0
  %tv_sec264 = load i64, ptr %sec_ptr263, align 8
  %nsec_ptr265 = getelementptr inbounds { i64, i64 }, ptr %timespec261, i32 0, i32 1
  %tv_nsec266 = load i64, ptr %nsec_ptr265, align 8
  %sec_ns267 = mul i64 %tv_sec264, 1000000000
  %total_ns268 = add i64 %sec_ns267, %tv_nsec266
  %v2269 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %total_ns268, 1
  %start270 = load { i8, i64 }, ptr %start, align 8
  %tag271 = extractvalue { i8, i64 } %v2269, 0
  %tag272 = extractvalue { i8, i64 } %start270, 0
  %data273 = extractvalue { i8, i64 } %v2269, 1
  %data274 = extractvalue { i8, i64 } %start270, 1
  %l_int = icmp eq i8 %tag271, 2
  %r_int275 = icmp eq i8 %tag272, 2
  %both_int276 = and i1 %l_int, %r_int275
  br i1 %both_int276, label %sub_int, label %sub_float

sub_int:                                          ; preds = %print_done258
  %diff = sub i64 %data273, %data274
  %v2277 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %diff, 1
  br label %sub_merge

sub_float:                                        ; preds = %print_done258
  %lf = icmp eq i8 %tag271, 3
  %rf278 = icmp eq i8 %tag272, 3
  %lf279 = bitcast i64 %data273 to double
  %li2f = sitofp i64 %data273 to double
  %left_as_float = select i1 %lf, double %lf279, double %li2f
  %rf280 = bitcast i64 %data274 to double
  %ri2f281 = sitofp i64 %data274 to double
  %right_as_float282 = select i1 %rf278, double %rf280, double %ri2f281
  %fdiff = fsub double %left_as_float, %right_as_float282
  %float_bits283 = bitcast double %fdiff to i64
  %v2284 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits283, 1
  br label %sub_merge

sub_merge:                                        ; preds = %sub_float, %sub_int
  %sub_result = phi { i8, i64 } [ %v2277, %sub_int ], [ %v2284, %sub_float ]
  store { i8, i64 } %sub_result, ptr %elapsed, align 8
  %elapsed285 = load { i8, i64 }, ptr %elapsed, align 8
  %tag286 = extractvalue { i8, i64 } %elapsed285, 0
  %data287 = extractvalue { i8, i64 } %elapsed285, 1
  switch i8 %tag286, label %str_default293 [
    i8 0, label %str_nil288
    i8 1, label %str_bool289
    i8 2, label %str_int290
    i8 3, label %str_float291
    i8 4, label %str_string292
    i8 5, label %str_list295
  ]

str_nil288:                                       ; preds = %sub_merge
  br label %str_merge294

str_bool289:                                      ; preds = %sub_merge
  %is_true296 = icmp ne i64 %data287, 0
  %bool_ptr297 = select i1 %is_true296, ptr @true_str.31, ptr @false_str.32
  %str_ptr_int298 = ptrtoint ptr %bool_ptr297 to i64
  %v2299 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int298, 1
  br label %str_merge294

str_int290:                                       ; preds = %sub_merge
  %int_buf300 = call ptr @malloc(i64 32)
  %78 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %int_buf300, i64 32, ptr @int_fmt.33, i64 %data287)
  %str_ptr_int301 = ptrtoint ptr %int_buf300 to i64
  %v2302 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int301, 1
  br label %str_merge294

str_float291:                                     ; preds = %sub_merge
  %float_buf303 = call ptr @malloc(i64 32)
  %f304 = bitcast i64 %data287 to double
  %79 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %float_buf303, i64 32, ptr @float_fmt.34, double %f304)
  %str_ptr_int305 = ptrtoint ptr %float_buf303 to i64
  %v2306 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int305, 1
  br label %str_merge294

str_string292:                                    ; preds = %sub_merge
  br label %str_merge294

str_default293:                                   ; preds = %sub_merge
  br label %str_merge294

str_merge294:                                     ; preds = %str_default293, %list_loop_end314, %str_string292, %str_float291, %str_int290, %str_bool289, %str_nil288
  %str_result346 = phi { i8, i64 } [ { i8 4, i64 ptrtoint (ptr @nil_str.30 to i64) }, %str_nil288 ], [ %v2299, %str_bool289 ], [ %v2302, %str_int290 ], [ %v2306, %str_float291 ], [ %elapsed285, %str_string292 ], [ %v2345, %list_loop_end314 ], [ { i8 4, i64 ptrtoint (ptr @empty_str.40 to i64) }, %str_default293 ]
  %tag347 = extractvalue { i8, i64 } %str_result346, 0
  %data348 = extractvalue { i8, i64 } %str_result346, 1
  %r_int354 = icmp eq i8 %tag347, 2
  %both_int355 = and i1 false, %r_int354
  %r_float356 = icmp eq i8 %tag347, 3
  %either_float357 = or i1 false, %r_float356
  %r_str358 = icmp eq i8 %tag347, 4
  %both_str359 = and i1 true, %r_str358
  br i1 %both_int355, label %add_int_int349, label %check_float360

str_list295:                                      ; preds = %sub_merge
  %list_ptr307 = inttoptr i64 %data287 to ptr
  %list_len308 = load i64, ptr %list_ptr307, align 8
  %buf_size_mul309 = mul i64 %list_len308, 25
  %list_buf_size310 = add i64 %buf_size_mul309, 3
  %list_buf311 = call ptr @malloc(i64 %list_buf_size310)
  %80 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %list_buf311, i64 %list_buf_size310, ptr @open_bracket.35)
  %idx_ptr315 = alloca i64, align 8
  store i64 0, ptr %idx_ptr315, align 8
  br label %list_loop_header312

list_loop_header312:                              ; preds = %elem_done334, %str_list295
  %idx316 = load i64, ptr %idx_ptr315, align 8
  %loop_cond317 = icmp ult i64 %idx316, %list_len308
  br i1 %loop_cond317, label %list_loop_body313, label %list_loop_end314

list_loop_body313:                                ; preds = %list_loop_header312
  %is_first318 = icmp eq i64 %idx316, 0
  br i1 %is_first318, label %elem_block320, label %sep_block319

list_loop_end314:                                 ; preds = %list_loop_header312
  %81 = call ptr @strcat(ptr %list_buf311, ptr @close_bracket.39)
  %str_ptr_int344 = ptrtoint ptr %list_buf311 to i64
  %v2345 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int344, 1
  br label %str_merge294

sep_block319:                                     ; preds = %list_loop_body313
  %82 = call ptr @strcat(ptr %list_buf311, ptr @comma_sep.36)
  br label %elem_block320

elem_block320:                                    ; preds = %sep_block319, %list_loop_body313
  %idx_in_elem321 = load i64, ptr %idx_ptr315, align 8
  %elements_base322 = getelementptr i64, ptr %list_ptr307, i64 1
  %elem_ptr323 = getelementptr { i8, i64 }, ptr %elements_base322, i64 %idx_in_elem321
  %elem_val324 = load { i8, i64 }, ptr %elem_ptr323, align 8
  %elem_tag325 = extractvalue { i8, i64 } %elem_val324, 0
  %elem_data326 = extractvalue { i8, i64 } %elem_val324, 1
  %elem_data_ptr327 = alloca i64, align 8
  store i64 %elem_data326, ptr %elem_data_ptr327, align 8
  %elem_is_float328 = icmp eq i8 %elem_tag325, 3
  %elem_is_string329 = icmp eq i8 %elem_tag325, 4
  br i1 %elem_is_float328, label %elem_float_block330, label %elem_string_check331

elem_float_block330:                              ; preds = %elem_block320
  %elem_data_float337 = load i64, ptr %elem_data_ptr327, align 8
  %elem_float_buf338 = call ptr @malloc(i64 25)
  %elem_as_float339 = bitcast i64 %elem_data_float337 to double
  %83 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_float_buf338, i64 25, ptr @float_fmt2.37, double %elem_as_float339)
  %84 = call ptr @strcat(ptr %list_buf311, ptr %elem_float_buf338)
  br label %elem_done334

elem_string_check331:                             ; preds = %elem_block320
  br i1 %elem_is_string329, label %elem_string_print332, label %elem_int_block333

elem_string_print332:                             ; preds = %elem_string_check331
  %elem_data_str335 = load i64, ptr %elem_data_ptr327, align 8
  %elem_str_ptr336 = inttoptr i64 %elem_data_str335 to ptr
  %85 = call ptr @strcat(ptr %list_buf311, ptr %elem_str_ptr336)
  br label %elem_done334

elem_int_block333:                                ; preds = %elem_string_check331
  %elem_data_int340 = load i64, ptr %elem_data_ptr327, align 8
  %elem_int_buf341 = call ptr @malloc(i64 25)
  %86 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_int_buf341, i64 25, ptr @int_fmt2.38, i64 %elem_data_int340)
  %87 = call ptr @strcat(ptr %list_buf311, ptr %elem_int_buf341)
  br label %elem_done334

elem_done334:                                     ; preds = %elem_int_block333, %elem_float_block330, %elem_string_print332
  %idx_for_incr342 = load i64, ptr %idx_ptr315, align 8
  %next_idx343 = add i64 %idx_for_incr342, 1
  store i64 %next_idx343, ptr %idx_ptr315, align 8
  br label %list_loop_header312

add_int_int349:                                   ; preds = %str_merge294
  %sum362 = add i64 ptrtoint (ptr @str.29 to i64), %data348
  %v2363 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %sum362, 1
  br label %add_merge353

add_float350:                                     ; preds = %check_float360
  %rf364 = bitcast i64 %data348 to double
  %ri2f365 = sitofp i64 %data348 to double
  %right_as_float366 = select i1 %r_float356, double %rf364, double %ri2f365
  %fsum367 = fadd double sitofp (i64 ptrtoint (ptr @str.29 to i64) to double), %right_as_float366
  %float_bits368 = bitcast double %fsum367 to i64
  %v2369 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits368, 1
  br label %add_merge353

add_string351:                                    ; preds = %check_string361
  %rstr370 = inttoptr i64 %data348 to ptr
  %llen371 = call i64 @strlen(ptr @str.29)
  %rlen372 = call i64 @strlen(ptr %rstr370)
  %total373 = add i64 %llen371, %rlen372
  %alloc_size374 = add i64 %total373, 1
  %new_str375 = call ptr @malloc(i64 %alloc_size374)
  %88 = call ptr @strcpy(ptr %new_str375, ptr @str.29)
  %89 = call ptr @strcat(ptr %new_str375, ptr %rstr370)
  %str_ptr_int376 = ptrtoint ptr %new_str375 to i64
  %v2377 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int376, 1
  br label %add_merge353

add_error352:                                     ; preds = %check_string361
  br label %add_merge353

add_merge353:                                     ; preds = %add_error352, %add_string351, %add_float350, %add_int_int349
  %add_result378 = phi { i8, i64 } [ %v2363, %add_int_int349 ], [ %v2369, %add_float350 ], [ %v2377, %add_string351 ], [ zeroinitializer, %add_error352 ]
  %tag379 = extractvalue { i8, i64 } %add_result378, 0
  %data380 = extractvalue { i8, i64 } %add_result378, 1
  %l_int386 = icmp eq i8 %tag379, 2
  %both_int387 = and i1 %l_int386, false
  %l_float = icmp eq i8 %tag379, 3
  %either_float388 = or i1 %l_float, false
  %l_str = icmp eq i8 %tag379, 4
  %both_str389 = and i1 %l_str, true
  br i1 %both_int387, label %add_int_int381, label %check_float390

check_float360:                                   ; preds = %str_merge294
  br i1 %either_float357, label %add_float350, label %check_string361

check_string361:                                  ; preds = %check_float360
  br i1 %both_str359, label %add_string351, label %add_error352

add_int_int381:                                   ; preds = %add_merge353
  %sum392 = add i64 %data380, ptrtoint (ptr @str.41 to i64)
  %v2393 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %sum392, 1
  br label %add_merge385

add_float382:                                     ; preds = %check_float390
  %lf394 = bitcast i64 %data380 to double
  %li2f395 = sitofp i64 %data380 to double
  %left_as_float396 = select i1 %l_float, double %lf394, double %li2f395
  %fsum397 = fadd double %left_as_float396, sitofp (i64 ptrtoint (ptr @str.41 to i64) to double)
  %float_bits398 = bitcast double %fsum397 to i64
  %v2399 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits398, 1
  br label %add_merge385

add_string383:                                    ; preds = %check_string391
  %lstr = inttoptr i64 %data380 to ptr
  %llen400 = call i64 @strlen(ptr %lstr)
  %rlen401 = call i64 @strlen(ptr @str.41)
  %total402 = add i64 %llen400, %rlen401
  %alloc_size403 = add i64 %total402, 1
  %new_str404 = call ptr @malloc(i64 %alloc_size403)
  %90 = call ptr @strcpy(ptr %new_str404, ptr %lstr)
  %91 = call ptr @strcat(ptr %new_str404, ptr @str.41)
  %str_ptr_int405 = ptrtoint ptr %new_str404 to i64
  %v2406 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int405, 1
  br label %add_merge385

add_error384:                                     ; preds = %check_string391
  br label %add_merge385

add_merge385:                                     ; preds = %add_error384, %add_string383, %add_float382, %add_int_int381
  %add_result407 = phi { i8, i64 } [ %v2393, %add_int_int381 ], [ %v2399, %add_float382 ], [ %v2406, %add_string383 ], [ zeroinitializer, %add_error384 ]
  %tag408 = extractvalue { i8, i64 } %add_result407, 0
  %data409 = extractvalue { i8, i64 } %add_result407, 1
  switch i8 %tag408, label %print_default415 [
    i8 0, label %print_nil410
    i8 1, label %print_bool411
    i8 2, label %print_int412
    i8 3, label %print_float413
    i8 4, label %print_string414
  ]

check_float390:                                   ; preds = %add_merge353
  br i1 %either_float388, label %add_float382, label %check_string391

check_string391:                                  ; preds = %check_float390
  br i1 %both_str389, label %add_string383, label %add_error384

print_nil410:                                     ; preds = %add_merge385
  %92 = call i32 (ptr, ...) @printf(ptr @fmt_nil)
  br label %print_done416

print_bool411:                                    ; preds = %add_merge385
  %is_true417 = icmp ne i64 %data409, 0
  %bool_str418 = select i1 %is_true417, ptr @fmt_true, ptr @fmt_false
  %93 = call i32 (ptr, ...) @printf(ptr %bool_str418)
  br label %print_done416

print_int412:                                     ; preds = %add_merge385
  %94 = call i32 (ptr, ...) @printf(ptr @fmt_int, i64 %data409)
  br label %print_done416

print_float413:                                   ; preds = %add_merge385
  %f419 = bitcast i64 %data409 to double
  %95 = call i32 (ptr, ...) @printf(ptr @fmt_float, double %f419)
  br label %print_done416

print_string414:                                  ; preds = %add_merge385
  %str420 = inttoptr i64 %data409 to ptr
  %96 = call i32 (ptr, ...) @printf(ptr @fmt_string, ptr %str420)
  br label %print_done416

print_default415:                                 ; preds = %add_merge385
  br label %print_done416

print_done416:                                    ; preds = %print_default415, %print_string414, %print_float413, %print_int412, %print_bool411, %print_nil410
  %97 = call i32 (ptr, ...) @printf(ptr @fmt_newline)
  %timespec421 = alloca { i64, i64 }, align 8
  %clock_result422 = call i32 @clock_gettime(i32 1, ptr %timespec421)
  %sec_ptr423 = getelementptr inbounds { i64, i64 }, ptr %timespec421, i32 0, i32 0
  %tv_sec424 = load i64, ptr %sec_ptr423, align 8
  %nsec_ptr425 = getelementptr inbounds { i64, i64 }, ptr %timespec421, i32 0, i32 1
  %tv_nsec426 = load i64, ptr %nsec_ptr425, align 8
  %sec_ns427 = mul i64 %tv_sec424, 1000000000
  %total_ns428 = add i64 %sec_ns427, %tv_nsec426
  %v2429 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %total_ns428, 1
  store { i8, i64 } %v2429, ptr %start, align 8
  %call430 = tail call { i8, i64 } @fib_iter({ i8, i64 } { i8 2, i64 10000 })
  store { i8, i64 } %call430, ptr %result, align 8
  %timespec431 = alloca { i64, i64 }, align 8
  %clock_result432 = call i32 @clock_gettime(i32 1, ptr %timespec431)
  %sec_ptr433 = getelementptr inbounds { i64, i64 }, ptr %timespec431, i32 0, i32 0
  %tv_sec434 = load i64, ptr %sec_ptr433, align 8
  %nsec_ptr435 = getelementptr inbounds { i64, i64 }, ptr %timespec431, i32 0, i32 1
  %tv_nsec436 = load i64, ptr %nsec_ptr435, align 8
  %sec_ns437 = mul i64 %tv_sec434, 1000000000
  %total_ns438 = add i64 %sec_ns437, %tv_nsec436
  %v2439 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %total_ns438, 1
  %start440 = load { i8, i64 }, ptr %start, align 8
  %tag441 = extractvalue { i8, i64 } %v2439, 0
  %tag442 = extractvalue { i8, i64 } %start440, 0
  %data443 = extractvalue { i8, i64 } %v2439, 1
  %data444 = extractvalue { i8, i64 } %start440, 1
  %l_int448 = icmp eq i8 %tag441, 2
  %r_int449 = icmp eq i8 %tag442, 2
  %both_int450 = and i1 %l_int448, %r_int449
  br i1 %both_int450, label %sub_int445, label %sub_float446

sub_int445:                                       ; preds = %print_done416
  %diff451 = sub i64 %data443, %data444
  %v2452 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %diff451, 1
  br label %sub_merge447

sub_float446:                                     ; preds = %print_done416
  %lf453 = icmp eq i8 %tag441, 3
  %rf454 = icmp eq i8 %tag442, 3
  %lf455 = bitcast i64 %data443 to double
  %li2f456 = sitofp i64 %data443 to double
  %left_as_float457 = select i1 %lf453, double %lf455, double %li2f456
  %rf458 = bitcast i64 %data444 to double
  %ri2f459 = sitofp i64 %data444 to double
  %right_as_float460 = select i1 %rf454, double %rf458, double %ri2f459
  %fdiff461 = fsub double %left_as_float457, %right_as_float460
  %float_bits462 = bitcast double %fdiff461 to i64
  %v2463 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits462, 1
  br label %sub_merge447

sub_merge447:                                     ; preds = %sub_float446, %sub_int445
  %sub_result464 = phi { i8, i64 } [ %v2452, %sub_int445 ], [ %v2463, %sub_float446 ]
  store { i8, i64 } %sub_result464, ptr %elapsed, align 8
  %elapsed465 = load { i8, i64 }, ptr %elapsed, align 8
  %tag466 = extractvalue { i8, i64 } %elapsed465, 0
  %data467 = extractvalue { i8, i64 } %elapsed465, 1
  switch i8 %tag466, label %str_default473 [
    i8 0, label %str_nil468
    i8 1, label %str_bool469
    i8 2, label %str_int470
    i8 3, label %str_float471
    i8 4, label %str_string472
    i8 5, label %str_list475
  ]

str_nil468:                                       ; preds = %sub_merge447
  br label %str_merge474

str_bool469:                                      ; preds = %sub_merge447
  %is_true476 = icmp ne i64 %data467, 0
  %bool_ptr477 = select i1 %is_true476, ptr @true_str.44, ptr @false_str.45
  %str_ptr_int478 = ptrtoint ptr %bool_ptr477 to i64
  %v2479 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int478, 1
  br label %str_merge474

str_int470:                                       ; preds = %sub_merge447
  %int_buf480 = call ptr @malloc(i64 32)
  %98 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %int_buf480, i64 32, ptr @int_fmt.46, i64 %data467)
  %str_ptr_int481 = ptrtoint ptr %int_buf480 to i64
  %v2482 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int481, 1
  br label %str_merge474

str_float471:                                     ; preds = %sub_merge447
  %float_buf483 = call ptr @malloc(i64 32)
  %f484 = bitcast i64 %data467 to double
  %99 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %float_buf483, i64 32, ptr @float_fmt.47, double %f484)
  %str_ptr_int485 = ptrtoint ptr %float_buf483 to i64
  %v2486 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int485, 1
  br label %str_merge474

str_string472:                                    ; preds = %sub_merge447
  br label %str_merge474

str_default473:                                   ; preds = %sub_merge447
  br label %str_merge474

str_merge474:                                     ; preds = %str_default473, %list_loop_end494, %str_string472, %str_float471, %str_int470, %str_bool469, %str_nil468
  %str_result526 = phi { i8, i64 } [ { i8 4, i64 ptrtoint (ptr @nil_str.43 to i64) }, %str_nil468 ], [ %v2479, %str_bool469 ], [ %v2482, %str_int470 ], [ %v2486, %str_float471 ], [ %elapsed465, %str_string472 ], [ %v2525, %list_loop_end494 ], [ { i8 4, i64 ptrtoint (ptr @empty_str.53 to i64) }, %str_default473 ]
  %tag527 = extractvalue { i8, i64 } %str_result526, 0
  %data528 = extractvalue { i8, i64 } %str_result526, 1
  %r_int534 = icmp eq i8 %tag527, 2
  %both_int535 = and i1 false, %r_int534
  %r_float536 = icmp eq i8 %tag527, 3
  %either_float537 = or i1 false, %r_float536
  %r_str538 = icmp eq i8 %tag527, 4
  %both_str539 = and i1 true, %r_str538
  br i1 %both_int535, label %add_int_int529, label %check_float540

str_list475:                                      ; preds = %sub_merge447
  %list_ptr487 = inttoptr i64 %data467 to ptr
  %list_len488 = load i64, ptr %list_ptr487, align 8
  %buf_size_mul489 = mul i64 %list_len488, 25
  %list_buf_size490 = add i64 %buf_size_mul489, 3
  %list_buf491 = call ptr @malloc(i64 %list_buf_size490)
  %100 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %list_buf491, i64 %list_buf_size490, ptr @open_bracket.48)
  %idx_ptr495 = alloca i64, align 8
  store i64 0, ptr %idx_ptr495, align 8
  br label %list_loop_header492

list_loop_header492:                              ; preds = %elem_done514, %str_list475
  %idx496 = load i64, ptr %idx_ptr495, align 8
  %loop_cond497 = icmp ult i64 %idx496, %list_len488
  br i1 %loop_cond497, label %list_loop_body493, label %list_loop_end494

list_loop_body493:                                ; preds = %list_loop_header492
  %is_first498 = icmp eq i64 %idx496, 0
  br i1 %is_first498, label %elem_block500, label %sep_block499

list_loop_end494:                                 ; preds = %list_loop_header492
  %101 = call ptr @strcat(ptr %list_buf491, ptr @close_bracket.52)
  %str_ptr_int524 = ptrtoint ptr %list_buf491 to i64
  %v2525 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int524, 1
  br label %str_merge474

sep_block499:                                     ; preds = %list_loop_body493
  %102 = call ptr @strcat(ptr %list_buf491, ptr @comma_sep.49)
  br label %elem_block500

elem_block500:                                    ; preds = %sep_block499, %list_loop_body493
  %idx_in_elem501 = load i64, ptr %idx_ptr495, align 8
  %elements_base502 = getelementptr i64, ptr %list_ptr487, i64 1
  %elem_ptr503 = getelementptr { i8, i64 }, ptr %elements_base502, i64 %idx_in_elem501
  %elem_val504 = load { i8, i64 }, ptr %elem_ptr503, align 8
  %elem_tag505 = extractvalue { i8, i64 } %elem_val504, 0
  %elem_data506 = extractvalue { i8, i64 } %elem_val504, 1
  %elem_data_ptr507 = alloca i64, align 8
  store i64 %elem_data506, ptr %elem_data_ptr507, align 8
  %elem_is_float508 = icmp eq i8 %elem_tag505, 3
  %elem_is_string509 = icmp eq i8 %elem_tag505, 4
  br i1 %elem_is_float508, label %elem_float_block510, label %elem_string_check511

elem_float_block510:                              ; preds = %elem_block500
  %elem_data_float517 = load i64, ptr %elem_data_ptr507, align 8
  %elem_float_buf518 = call ptr @malloc(i64 25)
  %elem_as_float519 = bitcast i64 %elem_data_float517 to double
  %103 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_float_buf518, i64 25, ptr @float_fmt2.50, double %elem_as_float519)
  %104 = call ptr @strcat(ptr %list_buf491, ptr %elem_float_buf518)
  br label %elem_done514

elem_string_check511:                             ; preds = %elem_block500
  br i1 %elem_is_string509, label %elem_string_print512, label %elem_int_block513

elem_string_print512:                             ; preds = %elem_string_check511
  %elem_data_str515 = load i64, ptr %elem_data_ptr507, align 8
  %elem_str_ptr516 = inttoptr i64 %elem_data_str515 to ptr
  %105 = call ptr @strcat(ptr %list_buf491, ptr %elem_str_ptr516)
  br label %elem_done514

elem_int_block513:                                ; preds = %elem_string_check511
  %elem_data_int520 = load i64, ptr %elem_data_ptr507, align 8
  %elem_int_buf521 = call ptr @malloc(i64 25)
  %106 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_int_buf521, i64 25, ptr @int_fmt2.51, i64 %elem_data_int520)
  %107 = call ptr @strcat(ptr %list_buf491, ptr %elem_int_buf521)
  br label %elem_done514

elem_done514:                                     ; preds = %elem_int_block513, %elem_float_block510, %elem_string_print512
  %idx_for_incr522 = load i64, ptr %idx_ptr495, align 8
  %next_idx523 = add i64 %idx_for_incr522, 1
  store i64 %next_idx523, ptr %idx_ptr495, align 8
  br label %list_loop_header492

add_int_int529:                                   ; preds = %str_merge474
  %sum542 = add i64 ptrtoint (ptr @str.42 to i64), %data528
  %v2543 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %sum542, 1
  br label %add_merge533

add_float530:                                     ; preds = %check_float540
  %rf544 = bitcast i64 %data528 to double
  %ri2f545 = sitofp i64 %data528 to double
  %right_as_float546 = select i1 %r_float536, double %rf544, double %ri2f545
  %fsum547 = fadd double sitofp (i64 ptrtoint (ptr @str.42 to i64) to double), %right_as_float546
  %float_bits548 = bitcast double %fsum547 to i64
  %v2549 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits548, 1
  br label %add_merge533

add_string531:                                    ; preds = %check_string541
  %rstr550 = inttoptr i64 %data528 to ptr
  %llen551 = call i64 @strlen(ptr @str.42)
  %rlen552 = call i64 @strlen(ptr %rstr550)
  %total553 = add i64 %llen551, %rlen552
  %alloc_size554 = add i64 %total553, 1
  %new_str555 = call ptr @malloc(i64 %alloc_size554)
  %108 = call ptr @strcpy(ptr %new_str555, ptr @str.42)
  %109 = call ptr @strcat(ptr %new_str555, ptr %rstr550)
  %str_ptr_int556 = ptrtoint ptr %new_str555 to i64
  %v2557 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int556, 1
  br label %add_merge533

add_error532:                                     ; preds = %check_string541
  br label %add_merge533

add_merge533:                                     ; preds = %add_error532, %add_string531, %add_float530, %add_int_int529
  %add_result558 = phi { i8, i64 } [ %v2543, %add_int_int529 ], [ %v2549, %add_float530 ], [ %v2557, %add_string531 ], [ zeroinitializer, %add_error532 ]
  %tag559 = extractvalue { i8, i64 } %add_result558, 0
  %data560 = extractvalue { i8, i64 } %add_result558, 1
  %l_int566 = icmp eq i8 %tag559, 2
  %both_int567 = and i1 %l_int566, false
  %l_float568 = icmp eq i8 %tag559, 3
  %either_float569 = or i1 %l_float568, false
  %l_str570 = icmp eq i8 %tag559, 4
  %both_str571 = and i1 %l_str570, true
  br i1 %both_int567, label %add_int_int561, label %check_float572

check_float540:                                   ; preds = %str_merge474
  br i1 %either_float537, label %add_float530, label %check_string541

check_string541:                                  ; preds = %check_float540
  br i1 %both_str539, label %add_string531, label %add_error532

add_int_int561:                                   ; preds = %add_merge533
  %sum574 = add i64 %data560, ptrtoint (ptr @str.54 to i64)
  %v2575 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %sum574, 1
  br label %add_merge565

add_float562:                                     ; preds = %check_float572
  %lf576 = bitcast i64 %data560 to double
  %li2f577 = sitofp i64 %data560 to double
  %left_as_float578 = select i1 %l_float568, double %lf576, double %li2f577
  %fsum579 = fadd double %left_as_float578, sitofp (i64 ptrtoint (ptr @str.54 to i64) to double)
  %float_bits580 = bitcast double %fsum579 to i64
  %v2581 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits580, 1
  br label %add_merge565

add_string563:                                    ; preds = %check_string573
  %lstr582 = inttoptr i64 %data560 to ptr
  %llen583 = call i64 @strlen(ptr %lstr582)
  %rlen584 = call i64 @strlen(ptr @str.54)
  %total585 = add i64 %llen583, %rlen584
  %alloc_size586 = add i64 %total585, 1
  %new_str587 = call ptr @malloc(i64 %alloc_size586)
  %110 = call ptr @strcpy(ptr %new_str587, ptr %lstr582)
  %111 = call ptr @strcat(ptr %new_str587, ptr @str.54)
  %str_ptr_int588 = ptrtoint ptr %new_str587 to i64
  %v2589 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int588, 1
  br label %add_merge565

add_error564:                                     ; preds = %check_string573
  br label %add_merge565

add_merge565:                                     ; preds = %add_error564, %add_string563, %add_float562, %add_int_int561
  %add_result590 = phi { i8, i64 } [ %v2575, %add_int_int561 ], [ %v2581, %add_float562 ], [ %v2589, %add_string563 ], [ zeroinitializer, %add_error564 ]
  %tag591 = extractvalue { i8, i64 } %add_result590, 0
  %data592 = extractvalue { i8, i64 } %add_result590, 1
  switch i8 %tag591, label %print_default598 [
    i8 0, label %print_nil593
    i8 1, label %print_bool594
    i8 2, label %print_int595
    i8 3, label %print_float596
    i8 4, label %print_string597
  ]

check_float572:                                   ; preds = %add_merge533
  br i1 %either_float569, label %add_float562, label %check_string573

check_string573:                                  ; preds = %check_float572
  br i1 %both_str571, label %add_string563, label %add_error564

print_nil593:                                     ; preds = %add_merge565
  %112 = call i32 (ptr, ...) @printf(ptr @fmt_nil)
  br label %print_done599

print_bool594:                                    ; preds = %add_merge565
  %is_true600 = icmp ne i64 %data592, 0
  %bool_str601 = select i1 %is_true600, ptr @fmt_true, ptr @fmt_false
  %113 = call i32 (ptr, ...) @printf(ptr %bool_str601)
  br label %print_done599

print_int595:                                     ; preds = %add_merge565
  %114 = call i32 (ptr, ...) @printf(ptr @fmt_int, i64 %data592)
  br label %print_done599

print_float596:                                   ; preds = %add_merge565
  %f602 = bitcast i64 %data592 to double
  %115 = call i32 (ptr, ...) @printf(ptr @fmt_float, double %f602)
  br label %print_done599

print_string597:                                  ; preds = %add_merge565
  %str603 = inttoptr i64 %data592 to ptr
  %116 = call i32 (ptr, ...) @printf(ptr @fmt_string, ptr %str603)
  br label %print_done599

print_default598:                                 ; preds = %add_merge565
  br label %print_done599

print_done599:                                    ; preds = %print_default598, %print_string597, %print_float596, %print_int595, %print_bool594, %print_nil593
  %117 = call i32 (ptr, ...) @printf(ptr @fmt_newline)
  %timespec604 = alloca { i64, i64 }, align 8
  %clock_result605 = call i32 @clock_gettime(i32 1, ptr %timespec604)
  %sec_ptr606 = getelementptr inbounds { i64, i64 }, ptr %timespec604, i32 0, i32 0
  %tv_sec607 = load i64, ptr %sec_ptr606, align 8
  %nsec_ptr608 = getelementptr inbounds { i64, i64 }, ptr %timespec604, i32 0, i32 1
  %tv_nsec609 = load i64, ptr %nsec_ptr608, align 8
  %sec_ns610 = mul i64 %tv_sec607, 1000000000
  %total_ns611 = add i64 %sec_ns610, %tv_nsec609
  %v2612 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %total_ns611, 1
  store { i8, i64 } %v2612, ptr %start, align 8
  %call613 = tail call { i8, i64 } @fib_iter({ i8, i64 } { i8 2, i64 50000 })
  store { i8, i64 } %call613, ptr %result, align 8
  %timespec614 = alloca { i64, i64 }, align 8
  %clock_result615 = call i32 @clock_gettime(i32 1, ptr %timespec614)
  %sec_ptr616 = getelementptr inbounds { i64, i64 }, ptr %timespec614, i32 0, i32 0
  %tv_sec617 = load i64, ptr %sec_ptr616, align 8
  %nsec_ptr618 = getelementptr inbounds { i64, i64 }, ptr %timespec614, i32 0, i32 1
  %tv_nsec619 = load i64, ptr %nsec_ptr618, align 8
  %sec_ns620 = mul i64 %tv_sec617, 1000000000
  %total_ns621 = add i64 %sec_ns620, %tv_nsec619
  %v2622 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %total_ns621, 1
  %start623 = load { i8, i64 }, ptr %start, align 8
  %tag624 = extractvalue { i8, i64 } %v2622, 0
  %tag625 = extractvalue { i8, i64 } %start623, 0
  %data626 = extractvalue { i8, i64 } %v2622, 1
  %data627 = extractvalue { i8, i64 } %start623, 1
  %l_int631 = icmp eq i8 %tag624, 2
  %r_int632 = icmp eq i8 %tag625, 2
  %both_int633 = and i1 %l_int631, %r_int632
  br i1 %both_int633, label %sub_int628, label %sub_float629

sub_int628:                                       ; preds = %print_done599
  %diff634 = sub i64 %data626, %data627
  %v2635 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %diff634, 1
  br label %sub_merge630

sub_float629:                                     ; preds = %print_done599
  %lf636 = icmp eq i8 %tag624, 3
  %rf637 = icmp eq i8 %tag625, 3
  %lf638 = bitcast i64 %data626 to double
  %li2f639 = sitofp i64 %data626 to double
  %left_as_float640 = select i1 %lf636, double %lf638, double %li2f639
  %rf641 = bitcast i64 %data627 to double
  %ri2f642 = sitofp i64 %data627 to double
  %right_as_float643 = select i1 %rf637, double %rf641, double %ri2f642
  %fdiff644 = fsub double %left_as_float640, %right_as_float643
  %float_bits645 = bitcast double %fdiff644 to i64
  %v2646 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits645, 1
  br label %sub_merge630

sub_merge630:                                     ; preds = %sub_float629, %sub_int628
  %sub_result647 = phi { i8, i64 } [ %v2635, %sub_int628 ], [ %v2646, %sub_float629 ]
  store { i8, i64 } %sub_result647, ptr %elapsed, align 8
  %elapsed648 = load { i8, i64 }, ptr %elapsed, align 8
  %tag649 = extractvalue { i8, i64 } %elapsed648, 0
  %data650 = extractvalue { i8, i64 } %elapsed648, 1
  switch i8 %tag649, label %str_default656 [
    i8 0, label %str_nil651
    i8 1, label %str_bool652
    i8 2, label %str_int653
    i8 3, label %str_float654
    i8 4, label %str_string655
    i8 5, label %str_list658
  ]

str_nil651:                                       ; preds = %sub_merge630
  br label %str_merge657

str_bool652:                                      ; preds = %sub_merge630
  %is_true659 = icmp ne i64 %data650, 0
  %bool_ptr660 = select i1 %is_true659, ptr @true_str.57, ptr @false_str.58
  %str_ptr_int661 = ptrtoint ptr %bool_ptr660 to i64
  %v2662 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int661, 1
  br label %str_merge657

str_int653:                                       ; preds = %sub_merge630
  %int_buf663 = call ptr @malloc(i64 32)
  %118 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %int_buf663, i64 32, ptr @int_fmt.59, i64 %data650)
  %str_ptr_int664 = ptrtoint ptr %int_buf663 to i64
  %v2665 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int664, 1
  br label %str_merge657

str_float654:                                     ; preds = %sub_merge630
  %float_buf666 = call ptr @malloc(i64 32)
  %f667 = bitcast i64 %data650 to double
  %119 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %float_buf666, i64 32, ptr @float_fmt.60, double %f667)
  %str_ptr_int668 = ptrtoint ptr %float_buf666 to i64
  %v2669 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int668, 1
  br label %str_merge657

str_string655:                                    ; preds = %sub_merge630
  br label %str_merge657

str_default656:                                   ; preds = %sub_merge630
  br label %str_merge657

str_merge657:                                     ; preds = %str_default656, %list_loop_end677, %str_string655, %str_float654, %str_int653, %str_bool652, %str_nil651
  %str_result709 = phi { i8, i64 } [ { i8 4, i64 ptrtoint (ptr @nil_str.56 to i64) }, %str_nil651 ], [ %v2662, %str_bool652 ], [ %v2665, %str_int653 ], [ %v2669, %str_float654 ], [ %elapsed648, %str_string655 ], [ %v2708, %list_loop_end677 ], [ { i8 4, i64 ptrtoint (ptr @empty_str.66 to i64) }, %str_default656 ]
  %tag710 = extractvalue { i8, i64 } %str_result709, 0
  %data711 = extractvalue { i8, i64 } %str_result709, 1
  %r_int717 = icmp eq i8 %tag710, 2
  %both_int718 = and i1 false, %r_int717
  %r_float719 = icmp eq i8 %tag710, 3
  %either_float720 = or i1 false, %r_float719
  %r_str721 = icmp eq i8 %tag710, 4
  %both_str722 = and i1 true, %r_str721
  br i1 %both_int718, label %add_int_int712, label %check_float723

str_list658:                                      ; preds = %sub_merge630
  %list_ptr670 = inttoptr i64 %data650 to ptr
  %list_len671 = load i64, ptr %list_ptr670, align 8
  %buf_size_mul672 = mul i64 %list_len671, 25
  %list_buf_size673 = add i64 %buf_size_mul672, 3
  %list_buf674 = call ptr @malloc(i64 %list_buf_size673)
  %120 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %list_buf674, i64 %list_buf_size673, ptr @open_bracket.61)
  %idx_ptr678 = alloca i64, align 8
  store i64 0, ptr %idx_ptr678, align 8
  br label %list_loop_header675

list_loop_header675:                              ; preds = %elem_done697, %str_list658
  %idx679 = load i64, ptr %idx_ptr678, align 8
  %loop_cond680 = icmp ult i64 %idx679, %list_len671
  br i1 %loop_cond680, label %list_loop_body676, label %list_loop_end677

list_loop_body676:                                ; preds = %list_loop_header675
  %is_first681 = icmp eq i64 %idx679, 0
  br i1 %is_first681, label %elem_block683, label %sep_block682

list_loop_end677:                                 ; preds = %list_loop_header675
  %121 = call ptr @strcat(ptr %list_buf674, ptr @close_bracket.65)
  %str_ptr_int707 = ptrtoint ptr %list_buf674 to i64
  %v2708 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int707, 1
  br label %str_merge657

sep_block682:                                     ; preds = %list_loop_body676
  %122 = call ptr @strcat(ptr %list_buf674, ptr @comma_sep.62)
  br label %elem_block683

elem_block683:                                    ; preds = %sep_block682, %list_loop_body676
  %idx_in_elem684 = load i64, ptr %idx_ptr678, align 8
  %elements_base685 = getelementptr i64, ptr %list_ptr670, i64 1
  %elem_ptr686 = getelementptr { i8, i64 }, ptr %elements_base685, i64 %idx_in_elem684
  %elem_val687 = load { i8, i64 }, ptr %elem_ptr686, align 8
  %elem_tag688 = extractvalue { i8, i64 } %elem_val687, 0
  %elem_data689 = extractvalue { i8, i64 } %elem_val687, 1
  %elem_data_ptr690 = alloca i64, align 8
  store i64 %elem_data689, ptr %elem_data_ptr690, align 8
  %elem_is_float691 = icmp eq i8 %elem_tag688, 3
  %elem_is_string692 = icmp eq i8 %elem_tag688, 4
  br i1 %elem_is_float691, label %elem_float_block693, label %elem_string_check694

elem_float_block693:                              ; preds = %elem_block683
  %elem_data_float700 = load i64, ptr %elem_data_ptr690, align 8
  %elem_float_buf701 = call ptr @malloc(i64 25)
  %elem_as_float702 = bitcast i64 %elem_data_float700 to double
  %123 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_float_buf701, i64 25, ptr @float_fmt2.63, double %elem_as_float702)
  %124 = call ptr @strcat(ptr %list_buf674, ptr %elem_float_buf701)
  br label %elem_done697

elem_string_check694:                             ; preds = %elem_block683
  br i1 %elem_is_string692, label %elem_string_print695, label %elem_int_block696

elem_string_print695:                             ; preds = %elem_string_check694
  %elem_data_str698 = load i64, ptr %elem_data_ptr690, align 8
  %elem_str_ptr699 = inttoptr i64 %elem_data_str698 to ptr
  %125 = call ptr @strcat(ptr %list_buf674, ptr %elem_str_ptr699)
  br label %elem_done697

elem_int_block696:                                ; preds = %elem_string_check694
  %elem_data_int703 = load i64, ptr %elem_data_ptr690, align 8
  %elem_int_buf704 = call ptr @malloc(i64 25)
  %126 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_int_buf704, i64 25, ptr @int_fmt2.64, i64 %elem_data_int703)
  %127 = call ptr @strcat(ptr %list_buf674, ptr %elem_int_buf704)
  br label %elem_done697

elem_done697:                                     ; preds = %elem_int_block696, %elem_float_block693, %elem_string_print695
  %idx_for_incr705 = load i64, ptr %idx_ptr678, align 8
  %next_idx706 = add i64 %idx_for_incr705, 1
  store i64 %next_idx706, ptr %idx_ptr678, align 8
  br label %list_loop_header675

add_int_int712:                                   ; preds = %str_merge657
  %sum725 = add i64 ptrtoint (ptr @str.55 to i64), %data711
  %v2726 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %sum725, 1
  br label %add_merge716

add_float713:                                     ; preds = %check_float723
  %rf727 = bitcast i64 %data711 to double
  %ri2f728 = sitofp i64 %data711 to double
  %right_as_float729 = select i1 %r_float719, double %rf727, double %ri2f728
  %fsum730 = fadd double sitofp (i64 ptrtoint (ptr @str.55 to i64) to double), %right_as_float729
  %float_bits731 = bitcast double %fsum730 to i64
  %v2732 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits731, 1
  br label %add_merge716

add_string714:                                    ; preds = %check_string724
  %rstr733 = inttoptr i64 %data711 to ptr
  %llen734 = call i64 @strlen(ptr @str.55)
  %rlen735 = call i64 @strlen(ptr %rstr733)
  %total736 = add i64 %llen734, %rlen735
  %alloc_size737 = add i64 %total736, 1
  %new_str738 = call ptr @malloc(i64 %alloc_size737)
  %128 = call ptr @strcpy(ptr %new_str738, ptr @str.55)
  %129 = call ptr @strcat(ptr %new_str738, ptr %rstr733)
  %str_ptr_int739 = ptrtoint ptr %new_str738 to i64
  %v2740 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int739, 1
  br label %add_merge716

add_error715:                                     ; preds = %check_string724
  br label %add_merge716

add_merge716:                                     ; preds = %add_error715, %add_string714, %add_float713, %add_int_int712
  %add_result741 = phi { i8, i64 } [ %v2726, %add_int_int712 ], [ %v2732, %add_float713 ], [ %v2740, %add_string714 ], [ zeroinitializer, %add_error715 ]
  %tag742 = extractvalue { i8, i64 } %add_result741, 0
  %data743 = extractvalue { i8, i64 } %add_result741, 1
  %l_int749 = icmp eq i8 %tag742, 2
  %both_int750 = and i1 %l_int749, false
  %l_float751 = icmp eq i8 %tag742, 3
  %either_float752 = or i1 %l_float751, false
  %l_str753 = icmp eq i8 %tag742, 4
  %both_str754 = and i1 %l_str753, true
  br i1 %both_int750, label %add_int_int744, label %check_float755

check_float723:                                   ; preds = %str_merge657
  br i1 %either_float720, label %add_float713, label %check_string724

check_string724:                                  ; preds = %check_float723
  br i1 %both_str722, label %add_string714, label %add_error715

add_int_int744:                                   ; preds = %add_merge716
  %sum757 = add i64 %data743, ptrtoint (ptr @str.67 to i64)
  %v2758 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %sum757, 1
  br label %add_merge748

add_float745:                                     ; preds = %check_float755
  %lf759 = bitcast i64 %data743 to double
  %li2f760 = sitofp i64 %data743 to double
  %left_as_float761 = select i1 %l_float751, double %lf759, double %li2f760
  %fsum762 = fadd double %left_as_float761, sitofp (i64 ptrtoint (ptr @str.67 to i64) to double)
  %float_bits763 = bitcast double %fsum762 to i64
  %v2764 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits763, 1
  br label %add_merge748

add_string746:                                    ; preds = %check_string756
  %lstr765 = inttoptr i64 %data743 to ptr
  %llen766 = call i64 @strlen(ptr %lstr765)
  %rlen767 = call i64 @strlen(ptr @str.67)
  %total768 = add i64 %llen766, %rlen767
  %alloc_size769 = add i64 %total768, 1
  %new_str770 = call ptr @malloc(i64 %alloc_size769)
  %130 = call ptr @strcpy(ptr %new_str770, ptr %lstr765)
  %131 = call ptr @strcat(ptr %new_str770, ptr @str.67)
  %str_ptr_int771 = ptrtoint ptr %new_str770 to i64
  %v2772 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int771, 1
  br label %add_merge748

add_error747:                                     ; preds = %check_string756
  br label %add_merge748

add_merge748:                                     ; preds = %add_error747, %add_string746, %add_float745, %add_int_int744
  %add_result773 = phi { i8, i64 } [ %v2758, %add_int_int744 ], [ %v2764, %add_float745 ], [ %v2772, %add_string746 ], [ zeroinitializer, %add_error747 ]
  %tag774 = extractvalue { i8, i64 } %add_result773, 0
  %data775 = extractvalue { i8, i64 } %add_result773, 1
  switch i8 %tag774, label %print_default781 [
    i8 0, label %print_nil776
    i8 1, label %print_bool777
    i8 2, label %print_int778
    i8 3, label %print_float779
    i8 4, label %print_string780
  ]

check_float755:                                   ; preds = %add_merge716
  br i1 %either_float752, label %add_float745, label %check_string756

check_string756:                                  ; preds = %check_float755
  br i1 %both_str754, label %add_string746, label %add_error747

print_nil776:                                     ; preds = %add_merge748
  %132 = call i32 (ptr, ...) @printf(ptr @fmt_nil)
  br label %print_done782

print_bool777:                                    ; preds = %add_merge748
  %is_true783 = icmp ne i64 %data775, 0
  %bool_str784 = select i1 %is_true783, ptr @fmt_true, ptr @fmt_false
  %133 = call i32 (ptr, ...) @printf(ptr %bool_str784)
  br label %print_done782

print_int778:                                     ; preds = %add_merge748
  %134 = call i32 (ptr, ...) @printf(ptr @fmt_int, i64 %data775)
  br label %print_done782

print_float779:                                   ; preds = %add_merge748
  %f785 = bitcast i64 %data775 to double
  %135 = call i32 (ptr, ...) @printf(ptr @fmt_float, double %f785)
  br label %print_done782

print_string780:                                  ; preds = %add_merge748
  %str786 = inttoptr i64 %data775 to ptr
  %136 = call i32 (ptr, ...) @printf(ptr @fmt_string, ptr %str786)
  br label %print_done782

print_default781:                                 ; preds = %add_merge748
  br label %print_done782

print_done782:                                    ; preds = %print_default781, %print_string780, %print_float779, %print_int778, %print_bool777, %print_nil776
  %137 = call i32 (ptr, ...) @printf(ptr @fmt_newline)
  %timespec787 = alloca { i64, i64 }, align 8
  %clock_result788 = call i32 @clock_gettime(i32 1, ptr %timespec787)
  %sec_ptr789 = getelementptr inbounds { i64, i64 }, ptr %timespec787, i32 0, i32 0
  %tv_sec790 = load i64, ptr %sec_ptr789, align 8
  %nsec_ptr791 = getelementptr inbounds { i64, i64 }, ptr %timespec787, i32 0, i32 1
  %tv_nsec792 = load i64, ptr %nsec_ptr791, align 8
  %sec_ns793 = mul i64 %tv_sec790, 1000000000
  %total_ns794 = add i64 %sec_ns793, %tv_nsec792
  %v2795 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %total_ns794, 1
  store { i8, i64 } %v2795, ptr %start, align 8
  %call796 = tail call { i8, i64 } @fib_iter({ i8, i64 } { i8 2, i64 100000 })
  store { i8, i64 } %call796, ptr %result, align 8
  %timespec797 = alloca { i64, i64 }, align 8
  %clock_result798 = call i32 @clock_gettime(i32 1, ptr %timespec797)
  %sec_ptr799 = getelementptr inbounds { i64, i64 }, ptr %timespec797, i32 0, i32 0
  %tv_sec800 = load i64, ptr %sec_ptr799, align 8
  %nsec_ptr801 = getelementptr inbounds { i64, i64 }, ptr %timespec797, i32 0, i32 1
  %tv_nsec802 = load i64, ptr %nsec_ptr801, align 8
  %sec_ns803 = mul i64 %tv_sec800, 1000000000
  %total_ns804 = add i64 %sec_ns803, %tv_nsec802
  %v2805 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %total_ns804, 1
  %start806 = load { i8, i64 }, ptr %start, align 8
  %tag807 = extractvalue { i8, i64 } %v2805, 0
  %tag808 = extractvalue { i8, i64 } %start806, 0
  %data809 = extractvalue { i8, i64 } %v2805, 1
  %data810 = extractvalue { i8, i64 } %start806, 1
  %l_int814 = icmp eq i8 %tag807, 2
  %r_int815 = icmp eq i8 %tag808, 2
  %both_int816 = and i1 %l_int814, %r_int815
  br i1 %both_int816, label %sub_int811, label %sub_float812

sub_int811:                                       ; preds = %print_done782
  %diff817 = sub i64 %data809, %data810
  %v2818 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %diff817, 1
  br label %sub_merge813

sub_float812:                                     ; preds = %print_done782
  %lf819 = icmp eq i8 %tag807, 3
  %rf820 = icmp eq i8 %tag808, 3
  %lf821 = bitcast i64 %data809 to double
  %li2f822 = sitofp i64 %data809 to double
  %left_as_float823 = select i1 %lf819, double %lf821, double %li2f822
  %rf824 = bitcast i64 %data810 to double
  %ri2f825 = sitofp i64 %data810 to double
  %right_as_float826 = select i1 %rf820, double %rf824, double %ri2f825
  %fdiff827 = fsub double %left_as_float823, %right_as_float826
  %float_bits828 = bitcast double %fdiff827 to i64
  %v2829 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits828, 1
  br label %sub_merge813

sub_merge813:                                     ; preds = %sub_float812, %sub_int811
  %sub_result830 = phi { i8, i64 } [ %v2818, %sub_int811 ], [ %v2829, %sub_float812 ]
  store { i8, i64 } %sub_result830, ptr %elapsed, align 8
  %elapsed831 = load { i8, i64 }, ptr %elapsed, align 8
  %tag832 = extractvalue { i8, i64 } %elapsed831, 0
  %data833 = extractvalue { i8, i64 } %elapsed831, 1
  switch i8 %tag832, label %str_default839 [
    i8 0, label %str_nil834
    i8 1, label %str_bool835
    i8 2, label %str_int836
    i8 3, label %str_float837
    i8 4, label %str_string838
    i8 5, label %str_list841
  ]

str_nil834:                                       ; preds = %sub_merge813
  br label %str_merge840

str_bool835:                                      ; preds = %sub_merge813
  %is_true842 = icmp ne i64 %data833, 0
  %bool_ptr843 = select i1 %is_true842, ptr @true_str.70, ptr @false_str.71
  %str_ptr_int844 = ptrtoint ptr %bool_ptr843 to i64
  %v2845 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int844, 1
  br label %str_merge840

str_int836:                                       ; preds = %sub_merge813
  %int_buf846 = call ptr @malloc(i64 32)
  %138 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %int_buf846, i64 32, ptr @int_fmt.72, i64 %data833)
  %str_ptr_int847 = ptrtoint ptr %int_buf846 to i64
  %v2848 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int847, 1
  br label %str_merge840

str_float837:                                     ; preds = %sub_merge813
  %float_buf849 = call ptr @malloc(i64 32)
  %f850 = bitcast i64 %data833 to double
  %139 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %float_buf849, i64 32, ptr @float_fmt.73, double %f850)
  %str_ptr_int851 = ptrtoint ptr %float_buf849 to i64
  %v2852 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int851, 1
  br label %str_merge840

str_string838:                                    ; preds = %sub_merge813
  br label %str_merge840

str_default839:                                   ; preds = %sub_merge813
  br label %str_merge840

str_merge840:                                     ; preds = %str_default839, %list_loop_end860, %str_string838, %str_float837, %str_int836, %str_bool835, %str_nil834
  %str_result892 = phi { i8, i64 } [ { i8 4, i64 ptrtoint (ptr @nil_str.69 to i64) }, %str_nil834 ], [ %v2845, %str_bool835 ], [ %v2848, %str_int836 ], [ %v2852, %str_float837 ], [ %elapsed831, %str_string838 ], [ %v2891, %list_loop_end860 ], [ { i8 4, i64 ptrtoint (ptr @empty_str.79 to i64) }, %str_default839 ]
  %tag893 = extractvalue { i8, i64 } %str_result892, 0
  %data894 = extractvalue { i8, i64 } %str_result892, 1
  %r_int900 = icmp eq i8 %tag893, 2
  %both_int901 = and i1 false, %r_int900
  %r_float902 = icmp eq i8 %tag893, 3
  %either_float903 = or i1 false, %r_float902
  %r_str904 = icmp eq i8 %tag893, 4
  %both_str905 = and i1 true, %r_str904
  br i1 %both_int901, label %add_int_int895, label %check_float906

str_list841:                                      ; preds = %sub_merge813
  %list_ptr853 = inttoptr i64 %data833 to ptr
  %list_len854 = load i64, ptr %list_ptr853, align 8
  %buf_size_mul855 = mul i64 %list_len854, 25
  %list_buf_size856 = add i64 %buf_size_mul855, 3
  %list_buf857 = call ptr @malloc(i64 %list_buf_size856)
  %140 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %list_buf857, i64 %list_buf_size856, ptr @open_bracket.74)
  %idx_ptr861 = alloca i64, align 8
  store i64 0, ptr %idx_ptr861, align 8
  br label %list_loop_header858

list_loop_header858:                              ; preds = %elem_done880, %str_list841
  %idx862 = load i64, ptr %idx_ptr861, align 8
  %loop_cond863 = icmp ult i64 %idx862, %list_len854
  br i1 %loop_cond863, label %list_loop_body859, label %list_loop_end860

list_loop_body859:                                ; preds = %list_loop_header858
  %is_first864 = icmp eq i64 %idx862, 0
  br i1 %is_first864, label %elem_block866, label %sep_block865

list_loop_end860:                                 ; preds = %list_loop_header858
  %141 = call ptr @strcat(ptr %list_buf857, ptr @close_bracket.78)
  %str_ptr_int890 = ptrtoint ptr %list_buf857 to i64
  %v2891 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int890, 1
  br label %str_merge840

sep_block865:                                     ; preds = %list_loop_body859
  %142 = call ptr @strcat(ptr %list_buf857, ptr @comma_sep.75)
  br label %elem_block866

elem_block866:                                    ; preds = %sep_block865, %list_loop_body859
  %idx_in_elem867 = load i64, ptr %idx_ptr861, align 8
  %elements_base868 = getelementptr i64, ptr %list_ptr853, i64 1
  %elem_ptr869 = getelementptr { i8, i64 }, ptr %elements_base868, i64 %idx_in_elem867
  %elem_val870 = load { i8, i64 }, ptr %elem_ptr869, align 8
  %elem_tag871 = extractvalue { i8, i64 } %elem_val870, 0
  %elem_data872 = extractvalue { i8, i64 } %elem_val870, 1
  %elem_data_ptr873 = alloca i64, align 8
  store i64 %elem_data872, ptr %elem_data_ptr873, align 8
  %elem_is_float874 = icmp eq i8 %elem_tag871, 3
  %elem_is_string875 = icmp eq i8 %elem_tag871, 4
  br i1 %elem_is_float874, label %elem_float_block876, label %elem_string_check877

elem_float_block876:                              ; preds = %elem_block866
  %elem_data_float883 = load i64, ptr %elem_data_ptr873, align 8
  %elem_float_buf884 = call ptr @malloc(i64 25)
  %elem_as_float885 = bitcast i64 %elem_data_float883 to double
  %143 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_float_buf884, i64 25, ptr @float_fmt2.76, double %elem_as_float885)
  %144 = call ptr @strcat(ptr %list_buf857, ptr %elem_float_buf884)
  br label %elem_done880

elem_string_check877:                             ; preds = %elem_block866
  br i1 %elem_is_string875, label %elem_string_print878, label %elem_int_block879

elem_string_print878:                             ; preds = %elem_string_check877
  %elem_data_str881 = load i64, ptr %elem_data_ptr873, align 8
  %elem_str_ptr882 = inttoptr i64 %elem_data_str881 to ptr
  %145 = call ptr @strcat(ptr %list_buf857, ptr %elem_str_ptr882)
  br label %elem_done880

elem_int_block879:                                ; preds = %elem_string_check877
  %elem_data_int886 = load i64, ptr %elem_data_ptr873, align 8
  %elem_int_buf887 = call ptr @malloc(i64 25)
  %146 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_int_buf887, i64 25, ptr @int_fmt2.77, i64 %elem_data_int886)
  %147 = call ptr @strcat(ptr %list_buf857, ptr %elem_int_buf887)
  br label %elem_done880

elem_done880:                                     ; preds = %elem_int_block879, %elem_float_block876, %elem_string_print878
  %idx_for_incr888 = load i64, ptr %idx_ptr861, align 8
  %next_idx889 = add i64 %idx_for_incr888, 1
  store i64 %next_idx889, ptr %idx_ptr861, align 8
  br label %list_loop_header858

add_int_int895:                                   ; preds = %str_merge840
  %sum908 = add i64 ptrtoint (ptr @str.68 to i64), %data894
  %v2909 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %sum908, 1
  br label %add_merge899

add_float896:                                     ; preds = %check_float906
  %rf910 = bitcast i64 %data894 to double
  %ri2f911 = sitofp i64 %data894 to double
  %right_as_float912 = select i1 %r_float902, double %rf910, double %ri2f911
  %fsum913 = fadd double sitofp (i64 ptrtoint (ptr @str.68 to i64) to double), %right_as_float912
  %float_bits914 = bitcast double %fsum913 to i64
  %v2915 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits914, 1
  br label %add_merge899

add_string897:                                    ; preds = %check_string907
  %rstr916 = inttoptr i64 %data894 to ptr
  %llen917 = call i64 @strlen(ptr @str.68)
  %rlen918 = call i64 @strlen(ptr %rstr916)
  %total919 = add i64 %llen917, %rlen918
  %alloc_size920 = add i64 %total919, 1
  %new_str921 = call ptr @malloc(i64 %alloc_size920)
  %148 = call ptr @strcpy(ptr %new_str921, ptr @str.68)
  %149 = call ptr @strcat(ptr %new_str921, ptr %rstr916)
  %str_ptr_int922 = ptrtoint ptr %new_str921 to i64
  %v2923 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int922, 1
  br label %add_merge899

add_error898:                                     ; preds = %check_string907
  br label %add_merge899

add_merge899:                                     ; preds = %add_error898, %add_string897, %add_float896, %add_int_int895
  %add_result924 = phi { i8, i64 } [ %v2909, %add_int_int895 ], [ %v2915, %add_float896 ], [ %v2923, %add_string897 ], [ zeroinitializer, %add_error898 ]
  %tag925 = extractvalue { i8, i64 } %add_result924, 0
  %data926 = extractvalue { i8, i64 } %add_result924, 1
  %l_int932 = icmp eq i8 %tag925, 2
  %both_int933 = and i1 %l_int932, false
  %l_float934 = icmp eq i8 %tag925, 3
  %either_float935 = or i1 %l_float934, false
  %l_str936 = icmp eq i8 %tag925, 4
  %both_str937 = and i1 %l_str936, true
  br i1 %both_int933, label %add_int_int927, label %check_float938

check_float906:                                   ; preds = %str_merge840
  br i1 %either_float903, label %add_float896, label %check_string907

check_string907:                                  ; preds = %check_float906
  br i1 %both_str905, label %add_string897, label %add_error898

add_int_int927:                                   ; preds = %add_merge899
  %sum940 = add i64 %data926, ptrtoint (ptr @str.80 to i64)
  %v2941 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %sum940, 1
  br label %add_merge931

add_float928:                                     ; preds = %check_float938
  %lf942 = bitcast i64 %data926 to double
  %li2f943 = sitofp i64 %data926 to double
  %left_as_float944 = select i1 %l_float934, double %lf942, double %li2f943
  %fsum945 = fadd double %left_as_float944, sitofp (i64 ptrtoint (ptr @str.80 to i64) to double)
  %float_bits946 = bitcast double %fsum945 to i64
  %v2947 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits946, 1
  br label %add_merge931

add_string929:                                    ; preds = %check_string939
  %lstr948 = inttoptr i64 %data926 to ptr
  %llen949 = call i64 @strlen(ptr %lstr948)
  %rlen950 = call i64 @strlen(ptr @str.80)
  %total951 = add i64 %llen949, %rlen950
  %alloc_size952 = add i64 %total951, 1
  %new_str953 = call ptr @malloc(i64 %alloc_size952)
  %150 = call ptr @strcpy(ptr %new_str953, ptr %lstr948)
  %151 = call ptr @strcat(ptr %new_str953, ptr @str.80)
  %str_ptr_int954 = ptrtoint ptr %new_str953 to i64
  %v2955 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int954, 1
  br label %add_merge931

add_error930:                                     ; preds = %check_string939
  br label %add_merge931

add_merge931:                                     ; preds = %add_error930, %add_string929, %add_float928, %add_int_int927
  %add_result956 = phi { i8, i64 } [ %v2941, %add_int_int927 ], [ %v2947, %add_float928 ], [ %v2955, %add_string929 ], [ zeroinitializer, %add_error930 ]
  %tag957 = extractvalue { i8, i64 } %add_result956, 0
  %data958 = extractvalue { i8, i64 } %add_result956, 1
  switch i8 %tag957, label %print_default964 [
    i8 0, label %print_nil959
    i8 1, label %print_bool960
    i8 2, label %print_int961
    i8 3, label %print_float962
    i8 4, label %print_string963
  ]

check_float938:                                   ; preds = %add_merge899
  br i1 %either_float935, label %add_float928, label %check_string939

check_string939:                                  ; preds = %check_float938
  br i1 %both_str937, label %add_string929, label %add_error930

print_nil959:                                     ; preds = %add_merge931
  %152 = call i32 (ptr, ...) @printf(ptr @fmt_nil)
  br label %print_done965

print_bool960:                                    ; preds = %add_merge931
  %is_true966 = icmp ne i64 %data958, 0
  %bool_str967 = select i1 %is_true966, ptr @fmt_true, ptr @fmt_false
  %153 = call i32 (ptr, ...) @printf(ptr %bool_str967)
  br label %print_done965

print_int961:                                     ; preds = %add_merge931
  %154 = call i32 (ptr, ...) @printf(ptr @fmt_int, i64 %data958)
  br label %print_done965

print_float962:                                   ; preds = %add_merge931
  %f968 = bitcast i64 %data958 to double
  %155 = call i32 (ptr, ...) @printf(ptr @fmt_float, double %f968)
  br label %print_done965

print_string963:                                  ; preds = %add_merge931
  %str969 = inttoptr i64 %data958 to ptr
  %156 = call i32 (ptr, ...) @printf(ptr @fmt_string, ptr %str969)
  br label %print_done965

print_default964:                                 ; preds = %add_merge931
  br label %print_done965

print_done965:                                    ; preds = %print_default964, %print_string963, %print_float962, %print_int961, %print_bool960, %print_nil959
  %157 = call i32 (ptr, ...) @printf(ptr @fmt_newline)
  switch i8 4, label %print_default975 [
    i8 0, label %print_nil970
    i8 1, label %print_bool971
    i8 2, label %print_int972
    i8 3, label %print_float973
    i8 4, label %print_string974
  ]

print_nil970:                                     ; preds = %print_done965
  %158 = call i32 (ptr, ...) @printf(ptr @fmt_nil)
  br label %print_done976

print_bool971:                                    ; preds = %print_done965
  %159 = call i32 (ptr, ...) @printf(ptr select (i1 icmp ne (i64 ptrtoint (ptr @str.81 to i64), i64 0), ptr @fmt_true, ptr @fmt_false))
  br label %print_done976

print_int972:                                     ; preds = %print_done965
  %160 = call i32 (ptr, ...) @printf(ptr @fmt_int, i64 ptrtoint (ptr @str.81 to i64))
  br label %print_done976

print_float973:                                   ; preds = %print_done965
  %161 = call i32 (ptr, ...) @printf(ptr @fmt_float, double bitcast (i64 ptrtoint (ptr @str.81 to i64) to double))
  br label %print_done976

print_string974:                                  ; preds = %print_done965
  %162 = call i32 (ptr, ...) @printf(ptr @fmt_string, ptr @str.81)
  br label %print_done976

print_default975:                                 ; preds = %print_done965
  br label %print_done976

print_done976:                                    ; preds = %print_default975, %print_string974, %print_float973, %print_int972, %print_bool971, %print_nil970
  %163 = call i32 (ptr, ...) @printf(ptr @fmt_newline)
  switch i8 4, label %print_default982 [
    i8 0, label %print_nil977
    i8 1, label %print_bool978
    i8 2, label %print_int979
    i8 3, label %print_float980
    i8 4, label %print_string981
  ]

print_nil977:                                     ; preds = %print_done976
  %164 = call i32 (ptr, ...) @printf(ptr @fmt_nil)
  br label %print_done983

print_bool978:                                    ; preds = %print_done976
  %165 = call i32 (ptr, ...) @printf(ptr select (i1 icmp ne (i64 ptrtoint (ptr @str.82 to i64), i64 0), ptr @fmt_true, ptr @fmt_false))
  br label %print_done983

print_int979:                                     ; preds = %print_done976
  %166 = call i32 (ptr, ...) @printf(ptr @fmt_int, i64 ptrtoint (ptr @str.82 to i64))
  br label %print_done983

print_float980:                                   ; preds = %print_done976
  %167 = call i32 (ptr, ...) @printf(ptr @fmt_float, double bitcast (i64 ptrtoint (ptr @str.82 to i64) to double))
  br label %print_done983

print_string981:                                  ; preds = %print_done976
  %168 = call i32 (ptr, ...) @printf(ptr @fmt_string, ptr @str.82)
  br label %print_done983

print_default982:                                 ; preds = %print_done976
  br label %print_done983

print_done983:                                    ; preds = %print_default982, %print_string981, %print_float980, %print_int979, %print_bool978, %print_nil977
  %169 = call i32 (ptr, ...) @printf(ptr @fmt_newline)
  %timespec984 = alloca { i64, i64 }, align 8
  %clock_result985 = call i32 @clock_gettime(i32 1, ptr %timespec984)
  %sec_ptr986 = getelementptr inbounds { i64, i64 }, ptr %timespec984, i32 0, i32 0
  %tv_sec987 = load i64, ptr %sec_ptr986, align 8
  %nsec_ptr988 = getelementptr inbounds { i64, i64 }, ptr %timespec984, i32 0, i32 1
  %tv_nsec989 = load i64, ptr %nsec_ptr988, align 8
  %sec_ns990 = mul i64 %tv_sec987, 1000000000
  %total_ns991 = add i64 %sec_ns990, %tv_nsec989
  %v2992 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %total_ns991, 1
  store { i8, i64 } %v2992, ptr %start, align 8
  store i64 0, ptr %i_shadow, align 8
  store { i8, i64 } { i8 2, i64 0 }, ptr %i, align 8
  br label %loop

loop:                                             ; preds = %body, %print_done983
  %i_i64 = load i64, ptr %i_shadow, align 8
  %cmp_direct = icmp slt i64 %i_i64, 10000
  br i1 %cmp_direct, label %body, label %after

body:                                             ; preds = %loop
  %call993 = tail call { i8, i64 } @fib_iter({ i8, i64 } { i8 2, i64 100 })
  %i_i64994 = load i64, ptr %i_shadow, align 8
  %add_i64 = add i64 %i_i64994, 1
  store i64 %add_i64, ptr %i_shadow, align 8
  %v2995 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %add_i64, 1
  br label %loop

after:                                            ; preds = %loop
  %i_sync = load i64, ptr %i_shadow, align 8
  %v2996 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %i_sync, 1
  store { i8, i64 } %v2996, ptr %i, align 8
  %timespec997 = alloca { i64, i64 }, align 8
  %clock_result998 = call i32 @clock_gettime(i32 1, ptr %timespec997)
  %sec_ptr999 = getelementptr inbounds { i64, i64 }, ptr %timespec997, i32 0, i32 0
  %tv_sec1000 = load i64, ptr %sec_ptr999, align 8
  %nsec_ptr1001 = getelementptr inbounds { i64, i64 }, ptr %timespec997, i32 0, i32 1
  %tv_nsec1002 = load i64, ptr %nsec_ptr1001, align 8
  %sec_ns1003 = mul i64 %tv_sec1000, 1000000000
  %total_ns1004 = add i64 %sec_ns1003, %tv_nsec1002
  %v21005 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %total_ns1004, 1
  %start1006 = load { i8, i64 }, ptr %start, align 8
  %tag1007 = extractvalue { i8, i64 } %v21005, 0
  %tag1008 = extractvalue { i8, i64 } %start1006, 0
  %data1009 = extractvalue { i8, i64 } %v21005, 1
  %data1010 = extractvalue { i8, i64 } %start1006, 1
  %l_int1014 = icmp eq i8 %tag1007, 2
  %r_int1015 = icmp eq i8 %tag1008, 2
  %both_int1016 = and i1 %l_int1014, %r_int1015
  br i1 %both_int1016, label %sub_int1011, label %sub_float1012

sub_int1011:                                      ; preds = %after
  %diff1017 = sub i64 %data1009, %data1010
  %v21018 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %diff1017, 1
  br label %sub_merge1013

sub_float1012:                                    ; preds = %after
  %lf1019 = icmp eq i8 %tag1007, 3
  %rf1020 = icmp eq i8 %tag1008, 3
  %lf1021 = bitcast i64 %data1009 to double
  %li2f1022 = sitofp i64 %data1009 to double
  %left_as_float1023 = select i1 %lf1019, double %lf1021, double %li2f1022
  %rf1024 = bitcast i64 %data1010 to double
  %ri2f1025 = sitofp i64 %data1010 to double
  %right_as_float1026 = select i1 %rf1020, double %rf1024, double %ri2f1025
  %fdiff1027 = fsub double %left_as_float1023, %right_as_float1026
  %float_bits1028 = bitcast double %fdiff1027 to i64
  %v21029 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits1028, 1
  br label %sub_merge1013

sub_merge1013:                                    ; preds = %sub_float1012, %sub_int1011
  %sub_result1030 = phi { i8, i64 } [ %v21018, %sub_int1011 ], [ %v21029, %sub_float1012 ]
  store { i8, i64 } %sub_result1030, ptr %elapsed, align 8
  %elapsed1031 = load { i8, i64 }, ptr %elapsed, align 8
  %tag1032 = extractvalue { i8, i64 } %elapsed1031, 0
  %data1033 = extractvalue { i8, i64 } %elapsed1031, 1
  %l_int1034 = icmp eq i8 %tag1032, 2
  %both_int1035 = and i1 %l_int1034, true
  br i1 %both_int1035, label %div_int, label %div_float

div_int:                                          ; preds = %sub_merge1013
  %quot = sdiv i64 %data1033, 1000
  %v21036 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %quot, 1
  br label %div_merge

div_float:                                        ; preds = %sub_merge1013
  %lf1037 = icmp eq i8 %tag1032, 3
  %lf1038 = bitcast i64 %data1033 to double
  %li2f1039 = sitofp i64 %data1033 to double
  %left_as_float1040 = select i1 %lf1037, double %lf1038, double %li2f1039
  %fquot = fdiv double %left_as_float1040, 1.000000e+03
  %float_bits1041 = bitcast double %fquot to i64
  %v21042 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits1041, 1
  br label %div_merge

div_merge:                                        ; preds = %div_float, %div_int
  %div_result = phi { i8, i64 } [ %v21036, %div_int ], [ %v21042, %div_float ]
  %tag1043 = extractvalue { i8, i64 } %div_result, 0
  %data1044 = extractvalue { i8, i64 } %div_result, 1
  switch i8 %tag1043, label %str_default1050 [
    i8 0, label %str_nil1045
    i8 1, label %str_bool1046
    i8 2, label %str_int1047
    i8 3, label %str_float1048
    i8 4, label %str_string1049
    i8 5, label %str_list1052
  ]

str_nil1045:                                      ; preds = %div_merge
  br label %str_merge1051

str_bool1046:                                     ; preds = %div_merge
  %is_true1053 = icmp ne i64 %data1044, 0
  %bool_ptr1054 = select i1 %is_true1053, ptr @true_str.85, ptr @false_str.86
  %str_ptr_int1055 = ptrtoint ptr %bool_ptr1054 to i64
  %v21056 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int1055, 1
  br label %str_merge1051

str_int1047:                                      ; preds = %div_merge
  %int_buf1057 = call ptr @malloc(i64 32)
  %170 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %int_buf1057, i64 32, ptr @int_fmt.87, i64 %data1044)
  %str_ptr_int1058 = ptrtoint ptr %int_buf1057 to i64
  %v21059 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int1058, 1
  br label %str_merge1051

str_float1048:                                    ; preds = %div_merge
  %float_buf1060 = call ptr @malloc(i64 32)
  %f1061 = bitcast i64 %data1044 to double
  %171 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %float_buf1060, i64 32, ptr @float_fmt.88, double %f1061)
  %str_ptr_int1062 = ptrtoint ptr %float_buf1060 to i64
  %v21063 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int1062, 1
  br label %str_merge1051

str_string1049:                                   ; preds = %div_merge
  br label %str_merge1051

str_default1050:                                  ; preds = %div_merge
  br label %str_merge1051

str_merge1051:                                    ; preds = %str_default1050, %list_loop_end1071, %str_string1049, %str_float1048, %str_int1047, %str_bool1046, %str_nil1045
  %str_result1103 = phi { i8, i64 } [ { i8 4, i64 ptrtoint (ptr @nil_str.84 to i64) }, %str_nil1045 ], [ %v21056, %str_bool1046 ], [ %v21059, %str_int1047 ], [ %v21063, %str_float1048 ], [ %div_result, %str_string1049 ], [ %v21102, %list_loop_end1071 ], [ { i8 4, i64 ptrtoint (ptr @empty_str.94 to i64) }, %str_default1050 ]
  %tag1104 = extractvalue { i8, i64 } %str_result1103, 0
  %data1105 = extractvalue { i8, i64 } %str_result1103, 1
  %r_int1111 = icmp eq i8 %tag1104, 2
  %both_int1112 = and i1 false, %r_int1111
  %r_float1113 = icmp eq i8 %tag1104, 3
  %either_float1114 = or i1 false, %r_float1113
  %r_str1115 = icmp eq i8 %tag1104, 4
  %both_str1116 = and i1 true, %r_str1115
  br i1 %both_int1112, label %add_int_int1106, label %check_float1117

str_list1052:                                     ; preds = %div_merge
  %list_ptr1064 = inttoptr i64 %data1044 to ptr
  %list_len1065 = load i64, ptr %list_ptr1064, align 8
  %buf_size_mul1066 = mul i64 %list_len1065, 25
  %list_buf_size1067 = add i64 %buf_size_mul1066, 3
  %list_buf1068 = call ptr @malloc(i64 %list_buf_size1067)
  %172 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %list_buf1068, i64 %list_buf_size1067, ptr @open_bracket.89)
  %idx_ptr1072 = alloca i64, align 8
  store i64 0, ptr %idx_ptr1072, align 8
  br label %list_loop_header1069

list_loop_header1069:                             ; preds = %elem_done1091, %str_list1052
  %idx1073 = load i64, ptr %idx_ptr1072, align 8
  %loop_cond1074 = icmp ult i64 %idx1073, %list_len1065
  br i1 %loop_cond1074, label %list_loop_body1070, label %list_loop_end1071

list_loop_body1070:                               ; preds = %list_loop_header1069
  %is_first1075 = icmp eq i64 %idx1073, 0
  br i1 %is_first1075, label %elem_block1077, label %sep_block1076

list_loop_end1071:                                ; preds = %list_loop_header1069
  %173 = call ptr @strcat(ptr %list_buf1068, ptr @close_bracket.93)
  %str_ptr_int1101 = ptrtoint ptr %list_buf1068 to i64
  %v21102 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int1101, 1
  br label %str_merge1051

sep_block1076:                                    ; preds = %list_loop_body1070
  %174 = call ptr @strcat(ptr %list_buf1068, ptr @comma_sep.90)
  br label %elem_block1077

elem_block1077:                                   ; preds = %sep_block1076, %list_loop_body1070
  %idx_in_elem1078 = load i64, ptr %idx_ptr1072, align 8
  %elements_base1079 = getelementptr i64, ptr %list_ptr1064, i64 1
  %elem_ptr1080 = getelementptr { i8, i64 }, ptr %elements_base1079, i64 %idx_in_elem1078
  %elem_val1081 = load { i8, i64 }, ptr %elem_ptr1080, align 8
  %elem_tag1082 = extractvalue { i8, i64 } %elem_val1081, 0
  %elem_data1083 = extractvalue { i8, i64 } %elem_val1081, 1
  %elem_data_ptr1084 = alloca i64, align 8
  store i64 %elem_data1083, ptr %elem_data_ptr1084, align 8
  %elem_is_float1085 = icmp eq i8 %elem_tag1082, 3
  %elem_is_string1086 = icmp eq i8 %elem_tag1082, 4
  br i1 %elem_is_float1085, label %elem_float_block1087, label %elem_string_check1088

elem_float_block1087:                             ; preds = %elem_block1077
  %elem_data_float1094 = load i64, ptr %elem_data_ptr1084, align 8
  %elem_float_buf1095 = call ptr @malloc(i64 25)
  %elem_as_float1096 = bitcast i64 %elem_data_float1094 to double
  %175 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_float_buf1095, i64 25, ptr @float_fmt2.91, double %elem_as_float1096)
  %176 = call ptr @strcat(ptr %list_buf1068, ptr %elem_float_buf1095)
  br label %elem_done1091

elem_string_check1088:                            ; preds = %elem_block1077
  br i1 %elem_is_string1086, label %elem_string_print1089, label %elem_int_block1090

elem_string_print1089:                            ; preds = %elem_string_check1088
  %elem_data_str1092 = load i64, ptr %elem_data_ptr1084, align 8
  %elem_str_ptr1093 = inttoptr i64 %elem_data_str1092 to ptr
  %177 = call ptr @strcat(ptr %list_buf1068, ptr %elem_str_ptr1093)
  br label %elem_done1091

elem_int_block1090:                               ; preds = %elem_string_check1088
  %elem_data_int1097 = load i64, ptr %elem_data_ptr1084, align 8
  %elem_int_buf1098 = call ptr @malloc(i64 25)
  %178 = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %elem_int_buf1098, i64 25, ptr @int_fmt2.92, i64 %elem_data_int1097)
  %179 = call ptr @strcat(ptr %list_buf1068, ptr %elem_int_buf1098)
  br label %elem_done1091

elem_done1091:                                    ; preds = %elem_int_block1090, %elem_float_block1087, %elem_string_print1089
  %idx_for_incr1099 = load i64, ptr %idx_ptr1072, align 8
  %next_idx1100 = add i64 %idx_for_incr1099, 1
  store i64 %next_idx1100, ptr %idx_ptr1072, align 8
  br label %list_loop_header1069

add_int_int1106:                                  ; preds = %str_merge1051
  %sum1119 = add i64 ptrtoint (ptr @str.83 to i64), %data1105
  %v21120 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %sum1119, 1
  br label %add_merge1110

add_float1107:                                    ; preds = %check_float1117
  %rf1121 = bitcast i64 %data1105 to double
  %ri2f1122 = sitofp i64 %data1105 to double
  %right_as_float1123 = select i1 %r_float1113, double %rf1121, double %ri2f1122
  %fsum1124 = fadd double sitofp (i64 ptrtoint (ptr @str.83 to i64) to double), %right_as_float1123
  %float_bits1125 = bitcast double %fsum1124 to i64
  %v21126 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits1125, 1
  br label %add_merge1110

add_string1108:                                   ; preds = %check_string1118
  %rstr1127 = inttoptr i64 %data1105 to ptr
  %llen1128 = call i64 @strlen(ptr @str.83)
  %rlen1129 = call i64 @strlen(ptr %rstr1127)
  %total1130 = add i64 %llen1128, %rlen1129
  %alloc_size1131 = add i64 %total1130, 1
  %new_str1132 = call ptr @malloc(i64 %alloc_size1131)
  %180 = call ptr @strcpy(ptr %new_str1132, ptr @str.83)
  %181 = call ptr @strcat(ptr %new_str1132, ptr %rstr1127)
  %str_ptr_int1133 = ptrtoint ptr %new_str1132 to i64
  %v21134 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int1133, 1
  br label %add_merge1110

add_error1109:                                    ; preds = %check_string1118
  br label %add_merge1110

add_merge1110:                                    ; preds = %add_error1109, %add_string1108, %add_float1107, %add_int_int1106
  %add_result1135 = phi { i8, i64 } [ %v21120, %add_int_int1106 ], [ %v21126, %add_float1107 ], [ %v21134, %add_string1108 ], [ zeroinitializer, %add_error1109 ]
  %tag1136 = extractvalue { i8, i64 } %add_result1135, 0
  %data1137 = extractvalue { i8, i64 } %add_result1135, 1
  %l_int1143 = icmp eq i8 %tag1136, 2
  %both_int1144 = and i1 %l_int1143, false
  %l_float1145 = icmp eq i8 %tag1136, 3
  %either_float1146 = or i1 %l_float1145, false
  %l_str1147 = icmp eq i8 %tag1136, 4
  %both_str1148 = and i1 %l_str1147, true
  br i1 %both_int1144, label %add_int_int1138, label %check_float1149

check_float1117:                                  ; preds = %str_merge1051
  br i1 %either_float1114, label %add_float1107, label %check_string1118

check_string1118:                                 ; preds = %check_float1117
  br i1 %both_str1116, label %add_string1108, label %add_error1109

add_int_int1138:                                  ; preds = %add_merge1110
  %sum1151 = add i64 %data1137, ptrtoint (ptr @str.95 to i64)
  %v21152 = insertvalue { i8, i64 } { i8 2, i64 undef }, i64 %sum1151, 1
  br label %add_merge1142

add_float1139:                                    ; preds = %check_float1149
  %lf1153 = bitcast i64 %data1137 to double
  %li2f1154 = sitofp i64 %data1137 to double
  %left_as_float1155 = select i1 %l_float1145, double %lf1153, double %li2f1154
  %fsum1156 = fadd double %left_as_float1155, sitofp (i64 ptrtoint (ptr @str.95 to i64) to double)
  %float_bits1157 = bitcast double %fsum1156 to i64
  %v21158 = insertvalue { i8, i64 } { i8 3, i64 undef }, i64 %float_bits1157, 1
  br label %add_merge1142

add_string1140:                                   ; preds = %check_string1150
  %lstr1159 = inttoptr i64 %data1137 to ptr
  %llen1160 = call i64 @strlen(ptr %lstr1159)
  %rlen1161 = call i64 @strlen(ptr @str.95)
  %total1162 = add i64 %llen1160, %rlen1161
  %alloc_size1163 = add i64 %total1162, 1
  %new_str1164 = call ptr @malloc(i64 %alloc_size1163)
  %182 = call ptr @strcpy(ptr %new_str1164, ptr %lstr1159)
  %183 = call ptr @strcat(ptr %new_str1164, ptr @str.95)
  %str_ptr_int1165 = ptrtoint ptr %new_str1164 to i64
  %v21166 = insertvalue { i8, i64 } { i8 4, i64 undef }, i64 %str_ptr_int1165, 1
  br label %add_merge1142

add_error1141:                                    ; preds = %check_string1150
  br label %add_merge1142

add_merge1142:                                    ; preds = %add_error1141, %add_string1140, %add_float1139, %add_int_int1138
  %add_result1167 = phi { i8, i64 } [ %v21152, %add_int_int1138 ], [ %v21158, %add_float1139 ], [ %v21166, %add_string1140 ], [ zeroinitializer, %add_error1141 ]
  %tag1168 = extractvalue { i8, i64 } %add_result1167, 0
  %data1169 = extractvalue { i8, i64 } %add_result1167, 1
  switch i8 %tag1168, label %print_default1175 [
    i8 0, label %print_nil1170
    i8 1, label %print_bool1171
    i8 2, label %print_int1172
    i8 3, label %print_float1173
    i8 4, label %print_string1174
  ]

check_float1149:                                  ; preds = %add_merge1110
  br i1 %either_float1146, label %add_float1139, label %check_string1150

check_string1150:                                 ; preds = %check_float1149
  br i1 %both_str1148, label %add_string1140, label %add_error1141

print_nil1170:                                    ; preds = %add_merge1142
  %184 = call i32 (ptr, ...) @printf(ptr @fmt_nil)
  br label %print_done1176

print_bool1171:                                   ; preds = %add_merge1142
  %is_true1177 = icmp ne i64 %data1169, 0
  %bool_str1178 = select i1 %is_true1177, ptr @fmt_true, ptr @fmt_false
  %185 = call i32 (ptr, ...) @printf(ptr %bool_str1178)
  br label %print_done1176

print_int1172:                                    ; preds = %add_merge1142
  %186 = call i32 (ptr, ...) @printf(ptr @fmt_int, i64 %data1169)
  br label %print_done1176

print_float1173:                                  ; preds = %add_merge1142
  %f1179 = bitcast i64 %data1169 to double
  %187 = call i32 (ptr, ...) @printf(ptr @fmt_float, double %f1179)
  br label %print_done1176

print_string1174:                                 ; preds = %add_merge1142
  %str1180 = inttoptr i64 %data1169 to ptr
  %188 = call i32 (ptr, ...) @printf(ptr @fmt_string, ptr %str1180)
  br label %print_done1176

print_default1175:                                ; preds = %add_merge1142
  br label %print_done1176

print_done1176:                                   ; preds = %print_default1175, %print_string1174, %print_float1173, %print_int1172, %print_bool1171, %print_nil1170
  %189 = call i32 (ptr, ...) @printf(ptr @fmt_newline)
  switch i8 4, label %print_default1186 [
    i8 0, label %print_nil1181
    i8 1, label %print_bool1182
    i8 2, label %print_int1183
    i8 3, label %print_float1184
    i8 4, label %print_string1185
  ]

print_nil1181:                                    ; preds = %print_done1176
  %190 = call i32 (ptr, ...) @printf(ptr @fmt_nil)
  br label %print_done1187

print_bool1182:                                   ; preds = %print_done1176
  %191 = call i32 (ptr, ...) @printf(ptr select (i1 icmp ne (i64 ptrtoint (ptr @str.96 to i64), i64 0), ptr @fmt_true, ptr @fmt_false))
  br label %print_done1187

print_int1183:                                    ; preds = %print_done1176
  %192 = call i32 (ptr, ...) @printf(ptr @fmt_int, i64 ptrtoint (ptr @str.96 to i64))
  br label %print_done1187

print_float1184:                                  ; preds = %print_done1176
  %193 = call i32 (ptr, ...) @printf(ptr @fmt_float, double bitcast (i64 ptrtoint (ptr @str.96 to i64) to double))
  br label %print_done1187

print_string1185:                                 ; preds = %print_done1176
  %194 = call i32 (ptr, ...) @printf(ptr @fmt_string, ptr @str.96)
  br label %print_done1187

print_default1186:                                ; preds = %print_done1176
  br label %print_done1187

print_done1187:                                   ; preds = %print_default1186, %print_string1185, %print_float1184, %print_int1183, %print_bool1182, %print_nil1181
  %195 = call i32 (ptr, ...) @printf(ptr @fmt_newline)
  switch i8 4, label %print_default1193 [
    i8 0, label %print_nil1188
    i8 1, label %print_bool1189
    i8 2, label %print_int1190
    i8 3, label %print_float1191
    i8 4, label %print_string1192
  ]

print_nil1188:                                    ; preds = %print_done1187
  %196 = call i32 (ptr, ...) @printf(ptr @fmt_nil)
  br label %print_done1194

print_bool1189:                                   ; preds = %print_done1187
  %197 = call i32 (ptr, ...) @printf(ptr select (i1 icmp ne (i64 ptrtoint (ptr @str.97 to i64), i64 0), ptr @fmt_true, ptr @fmt_false))
  br label %print_done1194

print_int1190:                                    ; preds = %print_done1187
  %198 = call i32 (ptr, ...) @printf(ptr @fmt_int, i64 ptrtoint (ptr @str.97 to i64))
  br label %print_done1194

print_float1191:                                  ; preds = %print_done1187
  %199 = call i32 (ptr, ...) @printf(ptr @fmt_float, double bitcast (i64 ptrtoint (ptr @str.97 to i64) to double))
  br label %print_done1194

print_string1192:                                 ; preds = %print_done1187
  %200 = call i32 (ptr, ...) @printf(ptr @fmt_string, ptr @str.97)
  br label %print_done1194

print_default1193:                                ; preds = %print_done1187
  br label %print_done1194

print_done1194:                                   ; preds = %print_default1193, %print_string1192, %print_float1191, %print_int1190, %print_bool1189, %print_nil1188
  %201 = call i32 (ptr, ...) @printf(ptr @fmt_newline)
  ret i32 0
}
