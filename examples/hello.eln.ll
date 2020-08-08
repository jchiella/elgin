source_filename = "hello.eln"

declare i32 @puts(i8*)

declare i32 @printf(i8*, ...)

define void @main() {
entry:
  %x = alloca i32
  store i128 10, i32* %x
  %tmpload = load i32, i32* %x
  %tmpadd = add i32 %tmpload, i128 10
  %tmpsub = sub i32 %tmpadd, i128 2
  ret void
}
