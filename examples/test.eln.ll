source_filename = "test.eln"

declare i32 @puts(i8*)

define i32 @main() {
entry:
  %x = alloca i32
  store i32 10, i32* %x
  store i32 100, i32* %x
  %tmpload = load i32, i32* %x
  ret i32 %tmpload
}
