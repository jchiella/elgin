source_filename = "test.eln"

declare i32 @puts(i8*)

define i32 @main() {
entry:
  %arr = alloca [10 x i32]
  store [10 x i32] undef, [10 x i32]* %arr
  ret i32 0
}
