source_filename = "test.eln"

declare i32 @puts(i8*)

define i32 @main() {
entry:
  %x = alloca [10 x i32]
  store [10 x i32] undef, [10 x i32]* %x
  %tmpload = load [10 x i32], [10 x i32]* %x
  %tmpgep = getelementptr [10 x i32], [10 x i32]* %x, i32 0, i32 3
  %tmpload1 = load i32, i32* %tmpgep
  ret i32 %tmpload1
}
