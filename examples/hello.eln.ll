source_filename = "hello.eln"

declare i32 @puts(i8*)

declare i32 @printf(i8*, ...)

define void @main() {
entry:
  %x = alloca i64
  store i64 10, i64* %x
  %tmpload = load i64, i64* %x
  %tmpadd = add i64 1, %tmpload
  ret void
}
