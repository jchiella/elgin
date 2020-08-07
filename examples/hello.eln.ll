source_filename = "hello.eln"

@tmpstr = private unnamed_addr constant [11 x i8] c"WHOOOO!!!!\00", align 1

declare i32 @puts(i8*)

define void @main() {
entry:
  %x = alloca i32
  store i32 10, i32* %x
  br label %whilecond

whilecond:                                        ; preds = %whilebody, %entry
  %tmpload = load i32, i32* %x
  %tmplt = icmp slt i32 %tmpload, 100
  br i1 %tmplt, label %whilebody, label %whileend

whilebody:                                        ; preds = %whilecond
  %tmpcall = call i32 @puts(i8* getelementptr inbounds ([11 x i8], [11 x i8]* @tmpstr, i32 0, i32 0))
  br label %whilecond

whileend:                                         ; preds = %whilecond
  ret void
}
