source_filename = "hello.eln"

@tmpstr = private unnamed_addr constant [4 x i8] c"%d \00", align 1

declare i32 @puts(i8*)

declare i32 @printf(i8*, ...)

define void @main() {
entry:
  %n = alloca i128
  store i128 0, i128* %n
  br label %whilecond

whilecond:                                        ; preds = %whilebody, %entry
  %tmpload = load i128, i128* %n
  %tmple = icmp sle i128 %tmpload, 20
  br i1 %tmple, label %whilebody, label %whileend

whilebody:                                        ; preds = %whilecond
  %tmpload1 = load i128, i128* %n
  %tmpcall = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @tmpstr, i32 0, i32 0), i128 %tmpload1)
  %tmpload2 = load i128, i128* %n
  %tmpadd = add i128 %tmpload2, 1
  store i128 %tmpadd, i128* %n
  br label %whilecond

whileend:                                         ; preds = %whilecond
  ret void
}
