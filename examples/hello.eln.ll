source_filename = "hello.eln"

declare i32 @puts(i8*)

declare i32 @printf(i8*, ...)

define void @main() {
entry:
  %x = alloca i64
  store i64 10, i64* %x
  %y = alloca float
  store float 0x40289999A0000000, float* %y
  %tmpload = load i64, i64* %x
  %tmpadd = add i64 1, %tmpload
  %tmpload1 = load float, float* %y
  %tmpadd2 = add float 0x4003333340000000, %tmpload1
  ret void
}
