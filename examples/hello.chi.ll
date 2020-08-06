source_filename = "hello.chi"

@s = private unnamed_addr constant [13 x i8] c"Hello world!\00", align 1

declare i32 @puts(i8*)

define i32 @say_twice(i8* %0) {
entry:
  %s = alloca i8*
  store i8* %0, i8** %s
  %tmpload = load i8*, i8** %s
  %tmpcall = call i32 @puts(i8* %tmpload)
  %tmpload1 = load i8*, i8** %s
  %tmpcall2 = call i32 @puts(i8* %tmpload1)
  ret i32 %tmpcall2
}

define void @main() {
entry:
  %i = alloca i32
  %tmpcall = call i32 @say_twice(i8* getelementptr inbounds ([13 x i8], [13 x i8]* @s, i32 0, i32 0))
  store i32 %tmpcall, i32* %i
  ret void
}
