source_filename = "test.eln"

@tmpstr = private unnamed_addr constant [10 x i8] c"Hi there!\00", align 1

declare i32 @puts(i8*)

define i32 @main() {
entry:
  %s = alloca i8*
  store i8* getelementptr inbounds ([10 x i8], [10 x i8]* @tmpstr, i32 0, i32 0), i8** %s
  %tmpload = load i8*, i8** %s
  %tmpcall = call i32 @puts(i8* %tmpload)
  ret i32 0
}
