source_filename = "hello.eln"

@tmpstr = private unnamed_addr constant [13 x i8] c"Hello world!\00", align 1

declare i32 @puts(i8*)

define i32 @main() {
entry:
  %x = alloca i32
  store i32 10, i32* %x
  %tmpcall = call i32 @puts(i8* getelementptr inbounds ([13 x i8], [13 x i8]* @tmpstr, i32 0, i32 0))
  %tmpload = load i32, i32* %x
  ret i32 %tmpload
}
