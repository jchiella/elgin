source_filename = "hello.eln"

@tmpstr = private unnamed_addr constant [4 x i8] c"%d \00", align 1

declare i32 @puts(i8*)

declare i32 @printf(i8*, ...)

define void @main() {
entry:
  %x = alloca i32
  store i32 10, i32* %x
  br label %whilecond

whilecond:                                        ; preds = %whilebody, %entry
  %tmpload = load i32, i32* %x
  %tmpgt = icmp sgt i32 %tmpload, 0
  br i1 %tmpgt, label %whilebody, label %whileend

whilebody:                                        ; preds = %whilecond
  %tmpload1 = load i32, i32* %x
  %tmpcall = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @tmpstr, i32 0, i32 0), i32 %tmpload1)
  %tmpload2 = load i32, i32* %x
  %tmpsub = sub i32 %tmpload2, 1
  store i32 %tmpsub, i32* %x
  br label %whilecond

whileend:                                         ; preds = %whilecond
  ret void
}
