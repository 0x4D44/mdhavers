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
@str = private unnamed_addr constant [11 x i8] c"Press key:\00", align 1
@k = global { i8, i64 } zeroinitializer

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

declare { i8, i64 } @__mdh_random(i64, i64)

define i32 @main() {
entry:
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
  %key_result = call { i8, i64 } @__mdh_get_key()
  store { i8, i64 } %key_result, ptr @k, align 8
  %k = load { i8, i64 }, ptr @k, align 8
  %tag = extractvalue { i8, i64 } %k, 0
  %data = extractvalue { i8, i64 } %k, 1
  switch i8 %tag, label %print_default6 [
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
  %is_true = icmp ne i64 %data, 0
  %bool_str = select i1 %is_true, ptr @fmt_true, ptr @fmt_false
  %7 = call i32 (ptr, ...) @printf(ptr %bool_str)
  br label %print_done7

print_int3:                                       ; preds = %print_done
  %8 = call i32 (ptr, ...) @printf(ptr @fmt_int, i64 %data)
  br label %print_done7

print_float4:                                     ; preds = %print_done
  %f = bitcast i64 %data to double
  %9 = call i32 (ptr, ...) @printf(ptr @fmt_float, double %f)
  br label %print_done7

print_string5:                                    ; preds = %print_done
  %str = inttoptr i64 %data to ptr
  %10 = call i32 (ptr, ...) @printf(ptr @fmt_string, ptr %str)
  br label %print_done7

print_default6:                                   ; preds = %print_done
  br label %print_done7

print_done7:                                      ; preds = %print_default6, %print_string5, %print_float4, %print_int3, %print_bool2, %print_nil1
  %11 = call i32 (ptr, ...) @printf(ptr @fmt_newline)
  ret i32 0
}
