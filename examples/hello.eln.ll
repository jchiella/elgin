source_filename = "hello.eln"

declare i32 @puts(i8*)

declare i32 @printf(i8*, ...)

define void @main() {
entry:
  %x = alloca i128
  store i128 1000000000, i128* %x
  %y = alloca i64
  store i128 123456789, i64* %y
  ret void
}
